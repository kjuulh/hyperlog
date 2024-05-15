use hyperlog_core::log::ItemState;
use sqlx::types::Json;

use crate::state::SharedState;

#[derive(Clone)]
pub struct ToggleItem {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub path: Vec<String>,
}
pub struct Response {}

#[derive(serde::Serialize, serde::Deserialize)]
struct ItemContent {
    pub title: String,
    pub description: String,
    pub state: ItemState,
}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

#[derive(sqlx::FromRow)]
struct Node {
    id: uuid::Uuid,
    item_content: Option<Json<ItemContent>>,
}

impl ToggleItem {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let Root { id: root_id, .. } =
            sqlx::query_as(r#"SELECT * FROM roots WHERE root_name = $1"#)
                .bind(req.root)
                .fetch_one(&self.db)
                .await?;
        let Node {
            id: node_id,
            mut item_content,
        } = sqlx::query_as(
            r#"
SELECT
    *
FROM
    nodes
WHERE 
    root_id = $1
    AND path = $2
    AND item_type = $3
            "#,
        )
        .bind(root_id)
        .bind(req.path.join("."))
        .bind("ITEM")
        .fetch_one(&self.db)
        .await?;

        if let Some(ref mut content) = item_content {
            content.state = match content.state {
                ItemState::NotDone => ItemState::Done,
                ItemState::Done => ItemState::NotDone,
            }
        }

        let res = sqlx::query(
            r#"
UPDATE 
    nodes
SET 
    item_content = $1
WHERE 
    id = $2
            "#,
        )
        .bind(item_content)
        .bind(node_id)
        .execute(&self.db)
        .await?;

        if res.rows_affected() != 1 {
            anyhow::bail!("failed to update item");
        }

        Ok(Response {})
    }
}

pub trait ToggleItemExt {
    fn toggle_item_service(&self) -> ToggleItem;
}

impl ToggleItemExt for SharedState {
    fn toggle_item_service(&self) -> ToggleItem {
        ToggleItem::new(self.db.clone())
    }
}
