use crate::state::SharedState;

#[derive(Clone)]
pub struct Restore {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub path: Vec<String>,
    pub user_id: Option<uuid::Uuid>,
}
pub struct Response {}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

impl Restore {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let Root { id: root_id, .. } = sqlx::query_as(
            r#"SELECT * FROM roots WHERE root_name = $1 AND user_id IS NOT DISTINCT FROM $2"#,
        )
        .bind(req.root)
        .bind(req.user_id)
        .fetch_one(&self.db)
        .await?;

        // Restore the node, all its ancestors (so it's reachable in the active
        // tree), and all its descendants.
        let mut prefixes: Vec<String> = Vec::new();
        for i in 1..=req.path.len() {
            prefixes.push(req.path[..i].join("."));
        }

        sqlx::query(r#"UPDATE nodes SET status = 'active' WHERE root_id = $1 AND path = ANY($2)"#)
            .bind(root_id)
            .bind(&prefixes)
            .execute(&self.db)
            .await?;

        sqlx::query(r#"UPDATE nodes SET status = 'active' WHERE root_id = $1 AND path LIKE $2"#)
            .bind(root_id)
            .bind(format!("{}.%", req.path.join(".")))
            .execute(&self.db)
            .await?;

        Ok(Response {})
    }
}

pub trait RestoreExt {
    fn restore_service(&self) -> Restore;
}

impl RestoreExt for SharedState {
    fn restore_service(&self) -> Restore {
        Restore::new(self.db.clone())
    }
}
