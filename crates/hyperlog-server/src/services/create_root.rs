use crate::state::SharedState;

#[derive(Clone)]
pub struct CreateRoot {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub user_id: Option<uuid::Uuid>,
}
pub struct Response {}

impl CreateRoot {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let root_id = uuid::Uuid::new_v4();
        sqlx::query(r#"INSERT INTO roots (id, root_name, user_id) VALUES ($1, $2, $3)"#)
            .bind(root_id)
            .bind(req.root)
            .bind(req.user_id)
            .execute(&self.db)
            .await?;

        Ok(Response {})
    }
}

pub trait CreateRootExt {
    fn create_root_service(&self) -> CreateRoot;
}

impl CreateRootExt for SharedState {
    fn create_root_service(&self) -> CreateRoot {
        CreateRoot::new(self.db.clone())
    }
}
