use crate::state::SharedState;

#[derive(Clone)]
pub struct Reorder {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub path: Vec<String>,   // parent (root-relative); empty = top level
    pub order: Vec<String>,  // child keys in the desired order
    pub user_id: Option<uuid::Uuid>,
}
pub struct Response {}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

impl Reorder {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    /// Assign sort_order = 0,1,2,… to the given child keys of `path`.
    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let Root { id: root_id, .. } = sqlx::query_as(
            r#"SELECT * FROM roots WHERE root_name = $1 AND user_id IS NOT DISTINCT FROM $2"#,
        )
        .bind(&req.root)
        .bind(req.user_id)
        .fetch_one(&self.db)
        .await?;

        let parent = req.path.join(".");
        for (i, key) in req.order.iter().enumerate() {
            let child = if parent.is_empty() {
                key.clone()
            } else {
                format!("{parent}.{key}")
            };
            sqlx::query(
                r#"UPDATE nodes SET sort_order = $3 WHERE root_id = $1 AND path = $2"#,
            )
            .bind(root_id)
            .bind(&child)
            .bind(i as f64)
            .execute(&self.db)
            .await?;
        }

        Ok(Response {})
    }
}

pub trait ReorderExt {
    fn reorder_service(&self) -> Reorder;
}

impl ReorderExt for SharedState {
    fn reorder_service(&self) -> Reorder {
        Reorder::new(self.db.clone())
    }
}
