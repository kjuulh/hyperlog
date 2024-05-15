use hyperlog_core::log::ItemState;
use sqlx::types::Json;

use crate::state::SharedState;

#[derive(Clone)]
pub struct CreateItem {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub path: Vec<String>,

    pub title: String,
    pub description: String,
    pub state: ItemState,
}
pub struct Response {}

#[derive(serde::Serialize)]
struct ItemContent {
    pub title: String,
    pub description: String,
    pub state: ItemState,
}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
    root_name: String,
}

#[derive(sqlx::FromRow)]
struct Section {
    id: uuid::Uuid,
}

impl CreateItem {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let Root { id: root_id, .. } =
            sqlx::query_as(r#"SELECT * FROM roots WHERE root_name = $1"#)
                .bind(req.root)
                .fetch_one(&self.db)
                .await?;

        match req.path.split_last() {
            Some((_, section_path)) => {
                if !section_path.is_empty() {
                    let Section { .. } = sqlx::query_as(
                        r#"
            SELECT 
                * 
            FROM 
                nodes 
            WHERE 
                root_id = $1 AND 
                path = $2 AND 
                item_type = 'SECTION'
                "#,
                    )
                    .bind(root_id)
                    .bind(section_path.join("."))
                    .fetch_one(&self.db)
                    .await?;
                }

                let node_id = uuid::Uuid::new_v4();
                sqlx::query(
                    r#"
    INSERT INTO nodes 
         (id, root_id, path, item_type, item_content) 
     VALUES 
         ($1, $2, $3, $4, $5)"#,
                )
                .bind(node_id)
                .bind(root_id)
                .bind(req.path.join("."))
                .bind("ITEM".to_string())
                .bind(Json(ItemContent {
                    title: req.title,
                    description: req.description,
                    state: req.state,
                }))
                .execute(&self.db)
                .await?;
            }
            None => anyhow::bail!("path most contain at least one item"),
        }

        Ok(Response {})
    }
}

pub trait CreateItemExt {
    fn create_item_service(&self) -> CreateItem;
}

impl CreateItemExt for SharedState {
    fn create_item_service(&self) -> CreateItem {
        CreateItem::new(self.db.clone())
    }
}
