use crate::state::SharedState;

#[derive(Clone)]
pub struct CreateSection {
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

impl CreateSection {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let Root { id: root_id, .. } =
            sqlx::query_as(r#"SELECT * FROM roots WHERE root_name = $1"#)
                .bind(req.root)
                .fetch_one(&self.db)
                .await?;

        // FIXME: implement consistency check on path

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
        .bind("SECTION".to_string())
        .bind(None::<serde_json::Value>)
        .execute(&self.db)
        .await?;

        Ok(Response {})
    }
}

pub trait CreateSectionExt {
    fn create_section_service(&self) -> CreateSection;
}

impl CreateSectionExt for SharedState {
    fn create_section_service(&self) -> CreateSection {
        CreateSection::new(self.db.clone())
    }
}
