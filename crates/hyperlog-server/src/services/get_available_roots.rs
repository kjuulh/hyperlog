use crate::state::SharedState;

#[derive(Clone)]
pub struct GetAvailableRoots {
    db: sqlx::PgPool,
}

pub struct Request {
    pub user_id: Option<uuid::Uuid>,
}
pub struct Response {
    pub roots: Vec<String>,
}

#[derive(sqlx::FromRow)]
pub struct Root {
    root_name: String,
}

impl GetAvailableRoots {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        // Scope to the user's workspaces; legacy (NULL) rows for unauthenticated.
        let roots: Vec<Root> = sqlx::query_as(
            r#"
    SELECT
        *
    FROM
        roots
    WHERE
        user_id IS NOT DISTINCT FROM $1
    LIMIT
        100
            "#,
        )
        .bind(req.user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(Response {
            roots: roots.into_iter().map(|i| i.root_name).collect(),
        })
    }
}

pub trait GetAvailableRootsExt {
    fn get_available_roots_service(&self) -> GetAvailableRoots;
}

impl GetAvailableRootsExt for SharedState {
    fn get_available_roots_service(&self) -> GetAvailableRoots {
        GetAvailableRoots::new(self.db.clone())
    }
}
