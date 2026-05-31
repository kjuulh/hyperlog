use hyperlog_core::log::{ItemState, Link};
use sqlx::types::Json;

use crate::state::SharedState;

#[derive(Clone)]
pub struct UpdateItem {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub path: Vec<String>,
    pub user_id: Option<uuid::Uuid>,

    pub title: String,
    pub description: String,
    pub state: ItemState,
    pub due: Option<String>,
    pub links: Vec<Link>,
}
pub struct Response {}

#[derive(serde::Serialize)]
struct ItemContent {
    pub title: String,
    pub description: String,
    pub state: ItemState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

impl UpdateItem {
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
        let Root { id: node_id } = sqlx::query_as(
            r#"
SELECT
    id
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

        let (_, rest) = req
            .path
            .split_last()
            .ok_or(anyhow::anyhow!("expected path to have at least one item"))?;

        let mut rest = rest.to_vec();
        rest.push(req.title.replace(".", "-"));

        let res = sqlx::query(
            r#"
UPDATE 
    nodes
SET 
    item_content = $1,
    path = $2
WHERE 
    id = $3
            "#,
        )
        .bind(Json(ItemContent {
            title: req.title,
            description: req.description,
            state: req.state,
            due: req.due,
            links: req.links,
        }))
        .bind(rest.join("."))
        .bind(node_id)
        .execute(&self.db)
        .await?;

        if res.rows_affected() != 1 {
            anyhow::bail!("failed to update item");
        }

        Ok(Response {})
    }
}

pub trait UpdateItemExt {
    fn update_item_service(&self) -> UpdateItem;
}

impl UpdateItemExt for SharedState {
    fn update_item_service(&self) -> UpdateItem {
        UpdateItem::new(self.db.clone())
    }
}
