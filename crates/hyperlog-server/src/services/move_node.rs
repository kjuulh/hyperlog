use crate::state::SharedState;

#[derive(Clone)]
pub struct MoveNode {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub src: Vec<String>,
    pub dest: Vec<String>,
    pub user_id: Option<uuid::Uuid>,
}
pub struct Response {}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

#[derive(sqlx::FromRow)]
struct Count {
    count: i64,
}

impl MoveNode {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    /// Move the node at `src` (and its whole subtree) to `dest` by rewriting the
    /// dotted materialized path prefix. Validates the move is consistent.
    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        if req.src.is_empty() || req.dest.is_empty() {
            anyhow::bail!("src and dest must be non-empty");
        }
        let src = req.src.join(".");
        let dest = req.dest.join(".");
        if src == dest {
            return Ok(Response {}); // no-op
        }
        // Can't move a node into its own subtree (would orphan/cycle).
        if dest.starts_with(&format!("{src}.")) {
            anyhow::bail!("cannot move a node into its own subtree");
        }

        let Root { id: root_id, .. } = sqlx::query_as(
            r#"SELECT * FROM roots WHERE root_name = $1 AND user_id IS NOT DISTINCT FROM $2"#,
        )
        .bind(&req.root)
        .bind(req.user_id)
        .fetch_one(&self.db)
        .await?;

        // src must exist (active).
        let Count { count: src_count } = sqlx::query_as(
            r#"SELECT count(*) as count FROM nodes WHERE root_id = $1 AND path = $2 AND status = 'active'"#,
        )
        .bind(root_id)
        .bind(&src)
        .fetch_one(&self.db)
        .await?;
        if src_count == 0 {
            anyhow::bail!("source not found: {src}");
        }

        // dest must be free.
        let Count { count: dest_count } =
            sqlx::query_as(r#"SELECT count(*) as count FROM nodes WHERE root_id = $1 AND path = $2"#)
                .bind(root_id)
                .bind(&dest)
                .fetch_one(&self.db)
                .await?;
        if dest_count > 0 {
            anyhow::bail!("destination already exists: {dest}");
        }

        // dest's parent (if nested) must exist as a section.
        if req.dest.len() > 1 {
            let parent = req.dest[..req.dest.len() - 1].join(".");
            let Count { count: parent_count } = sqlx::query_as(
                r#"SELECT count(*) as count FROM nodes WHERE root_id = $1 AND path = $2 AND item_type = 'SECTION' AND status = 'active'"#,
            )
            .bind(root_id)
            .bind(&parent)
            .fetch_one(&self.db)
            .await?;
            if parent_count == 0 {
                anyhow::bail!("destination parent section not found: {parent}");
            }
        }

        // Rewrite the path prefix for the node + every descendant.
        sqlx::query(
            r#"
            UPDATE nodes
            SET path = $3 || substring(path from char_length($2) + 1)
            WHERE root_id = $1 AND (path = $2 OR path LIKE $2 || '.%')
            "#,
        )
        .bind(root_id)
        .bind(&src)
        .bind(&dest)
        .execute(&self.db)
        .await?;

        Ok(Response {})
    }
}

pub trait MoveNodeExt {
    fn move_node_service(&self) -> MoveNode;
}

impl MoveNodeExt for SharedState {
    fn move_node_service(&self) -> MoveNode {
        MoveNode::new(self.db.clone())
    }
}
