use sqlx::types::Json;

use crate::state::SharedState;

#[derive(Clone)]
pub struct GetArchived {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub user_id: Option<uuid::Uuid>,
}

pub struct ArchivedItem {
    pub path: Vec<String>,
    pub item_type: String,
    pub title: String,
}
pub struct Response {
    pub items: Vec<ArchivedItem>,
}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

#[derive(sqlx::FromRow)]
struct Node {
    path: String,
    item_type: String,
    item_content: Option<Json<serde_json::Value>>,
}

impl GetArchived {
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

        let nodes: Vec<Node> = sqlx::query_as(
            r#"
    SELECT path, item_type, item_content
    FROM nodes
    WHERE root_id = $1 AND status = 'archive'
    ORDER BY path
    LIMIT 500
            "#,
        )
        .bind(root_id)
        .fetch_all(&self.db)
        .await?;

        let items = nodes
            .into_iter()
            .map(|n| {
                let title = n
                    .item_content
                    .as_ref()
                    .and_then(|c| c.0.get("title"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                ArchivedItem {
                    path: n.path.split('.').map(|s| s.to_string()).collect(),
                    item_type: n.item_type,
                    title,
                }
            })
            .collect();

        Ok(Response { items })
    }
}

pub trait GetArchivedExt {
    fn get_archived_service(&self) -> GetArchived;
}

impl GetArchivedExt for SharedState {
    fn get_archived_service(&self) -> GetArchived {
        GetArchived::new(self.db.clone())
    }
}
