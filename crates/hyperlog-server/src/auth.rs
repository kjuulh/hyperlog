//! Users & authentication: argon2id passwords, HS256 access JWTs, and opaque
//! rotating refresh tokens (stored hashed) with reuse detection.

use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use hyperlog_protos::hyperlog::{
    auth_server::Auth, AuthResponse, LoginRequest, LogoutRequest, LogoutResponse, MeRequest,
    RefreshRequest, RegisterRequest, User as PbUser,
};

const ACCESS_TTL_SECS: i64 = 15 * 60;
const REFRESH_TTL_SECS: i64 = 30 * 24 * 60 * 60;

/// Authenticated user id, injected into request extensions by the auth
/// interceptor (see external_grpc) for downstream services.
#[derive(Clone, Copy, Debug)]
pub struct AuthedUser(pub Uuid);

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

#[derive(Clone)]
pub struct AuthService {
    db: PgPool,
    jwt_secret: Arc<Vec<u8>>,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    username: String,
    password_hash: String,
}

impl AuthService {
    pub fn new(db: PgPool) -> Self {
        let secret = std::env::var("HYPERLOG_JWT_SECRET")
            .unwrap_or_else(|_| "dev-insecure-secret-change-me".to_string());
        if secret == "dev-insecure-secret-change-me" {
            tracing::warn!("HYPERLOG_JWT_SECRET not set — using an insecure dev secret");
        }
        Self {
            db,
            jwt_secret: Arc::new(secret.into_bytes()),
        }
    }

    pub fn jwt_secret(&self) -> Arc<Vec<u8>> {
        self.jwt_secret.clone()
    }

    /// Validate an access JWT and return the user id. Used by the interceptor.
    pub fn verify_access(secret: &[u8], token: &str) -> anyhow::Result<Uuid> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret),
            &Validation::new(Algorithm::HS256),
        )?;
        Ok(Uuid::parse_str(&data.claims.sub)?)
    }

    fn make_access(&self, uid: Uuid) -> anyhow::Result<(String, i64)> {
        let now = OffsetDateTime::now_utc();
        let exp = now + Duration::seconds(ACCESS_TTL_SECS);
        let claims = Claims {
            sub: uid.to_string(),
            exp: exp.unix_timestamp() as usize,
            iat: now.unix_timestamp() as usize,
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.jwt_secret),
        )?;
        Ok((token, ACCESS_TTL_SECS))
    }

    async fn issue_refresh(&self, uid: Uuid) -> anyhow::Result<(String, i64)> {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let raw = hex::encode(bytes);
        let hash = sha256_hex(&raw);
        let expires = OffsetDateTime::now_utc() + Duration::seconds(REFRESH_TTL_SECS);
        sqlx::query(
            "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at) VALUES ($1,$2,$3,$4)",
        )
        .bind(Uuid::new_v4())
        .bind(uid)
        .bind(&hash)
        .bind(expires)
        .execute(&self.db)
        .await?;
        Ok((raw, REFRESH_TTL_SECS))
    }

    async fn auth_response(&self, user: UserRow) -> anyhow::Result<AuthResponse> {
        let (access_token, access_expires_in) = self.make_access(user.id)?;
        let (refresh_token, refresh_expires_in) = self.issue_refresh(user.id).await?;
        Ok(AuthResponse {
            user: Some(PbUser {
                id: user.id.to_string(),
                email: user.email,
                username: user.username,
            }),
            access_token,
            access_expires_in,
            refresh_token,
            refresh_expires_in,
        })
    }
}

fn sha256_hex(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}

fn hash_password(pw: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow::anyhow!("hash: {e}"))
}

fn verify_password(pw: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(pw.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

fn internal<E: std::fmt::Display>(e: E) -> Status {
    Status::internal(e.to_string())
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();
        let email = req.email.trim().to_lowercase();
        let username = req.username.trim().to_string();
        if email.is_empty() || !email.contains('@') {
            return Err(Status::invalid_argument("a valid email is required"));
        }
        if username.len() < 2 {
            return Err(Status::invalid_argument("username too short"));
        }
        if req.password.len() < 8 {
            return Err(Status::invalid_argument("password must be at least 8 characters"));
        }

        let password_hash = hash_password(&req.password).map_err(internal)?;
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, email, username, password_hash) VALUES ($1,$2,$3,$4)")
            .bind(id)
            .bind(&email)
            .bind(&username)
            .bind(&password_hash)
            .execute(&self.db)
            .await
            .map_err(|e| {
                if is_unique_violation(&e) {
                    Status::already_exists("email or username already in use")
                } else {
                    internal(e)
                }
            })?;

        // Give the new user a default workspace.
        let _ = sqlx::query("INSERT INTO roots (id, root_name, user_id) VALUES ($1,$2,$3)")
            .bind(Uuid::new_v4())
            .bind("personal")
            .bind(id)
            .execute(&self.db)
            .await;

        let user = UserRow {
            id,
            email,
            username,
            password_hash,
        };
        Ok(Response::new(self.auth_response(user).await.map_err(internal)?))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();
        let ident = req.identifier.trim().to_lowercase();
        let user: Option<UserRow> = sqlx::query_as(
            "SELECT id, email, username, password_hash FROM users WHERE email = $1 OR username = $1",
        )
        .bind(&ident)
        .fetch_optional(&self.db)
        .await
        .map_err(internal)?;

        let user = user.ok_or_else(|| Status::unauthenticated("invalid credentials"))?;
        if !verify_password(&req.password, &user.password_hash) {
            return Err(Status::unauthenticated("invalid credentials"));
        }
        Ok(Response::new(self.auth_response(user).await.map_err(internal)?))
    }

    async fn refresh(
        &self,
        request: Request<RefreshRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();
        let hash = sha256_hex(&req.refresh_token);

        let row: Option<(Uuid, Uuid, bool, OffsetDateTime)> = sqlx::query_as(
            "SELECT id, user_id, revoked, expires_at FROM refresh_tokens WHERE token_hash = $1",
        )
        .bind(&hash)
        .fetch_optional(&self.db)
        .await
        .map_err(internal)?;

        let (token_id, user_id, revoked, expires_at) =
            row.ok_or_else(|| Status::unauthenticated("invalid refresh token"))?;

        if revoked {
            // Reuse of an already-rotated token => likely theft. Burn the chain.
            let _ = sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.db)
                .await;
            return Err(Status::unauthenticated("refresh token reuse detected"));
        }
        if expires_at < OffsetDateTime::now_utc() {
            return Err(Status::unauthenticated("refresh token expired"));
        }

        let user: UserRow =
            sqlx::query_as("SELECT id, email, username, password_hash FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&self.db)
                .await
                .map_err(internal)?;

        let resp = self.auth_response(user).await.map_err(internal)?;
        // Rotate: revoke the presented token.
        let _ = sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE id = $1")
            .bind(token_id)
            .execute(&self.db)
            .await;
        Ok(Response::new(resp))
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let req = request.into_inner();
        let hash = sha256_hex(&req.refresh_token);
        sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE token_hash = $1")
            .bind(&hash)
            .execute(&self.db)
            .await
            .map_err(internal)?;
        Ok(Response::new(LogoutResponse {}))
    }

    async fn me(&self, request: Request<MeRequest>) -> Result<Response<PbUser>, Status> {
        let uid = request
            .extensions()
            .get::<AuthedUser>()
            .map(|u| u.0)
            .ok_or_else(|| Status::unauthenticated("not authenticated"))?;
        let user: UserRow =
            sqlx::query_as("SELECT id, email, username, password_hash FROM users WHERE id = $1")
                .bind(uid)
                .fetch_one(&self.db)
                .await
                .map_err(internal)?;
        Ok(Response::new(PbUser {
            id: user.id.to_string(),
            email: user.email,
            username: user.username,
        }))
    }
}
