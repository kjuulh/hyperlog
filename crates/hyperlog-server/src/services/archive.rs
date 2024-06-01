use crate::state::SharedState;

#[derive(Clone)]
pub struct Archive {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub path: Vec<String>,
}
pub struct Response {}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

impl Archive {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let Root { id: root_id, .. } =
            sqlx::query_as(r#"SELECT * FROM roots WHERE root_name = $1"#)
                .bind(req.root)
                .fetch_one(&self.db)
                .await?;

        sqlx::query(
            r#"
UPDATE nodes
SET status = 'archive'
WHERE 
    root_id = $1
    AND path = $2;
            "#,
        )
        .bind(root_id)
        .bind(req.path.join("."))
        .execute(&self.db)
        .await?;

        sqlx::query(
            r#"
UPDATE nodes
SET status = 'archive'
WHERE root_id = $1
AND path LIKE $2;
            "#,
        )
        .bind(root_id)
        .bind(format!("{}.%", req.path.join(".")))
        .execute(&self.db)
        .await?;

        Ok(Response {})
    }
}

pub trait ArchiveExt {
    fn archive_service(&self) -> Archive;
}

impl ArchiveExt for SharedState {
    fn archive_service(&self) -> Archive {
        Archive::new(self.db.clone())
    }
}
