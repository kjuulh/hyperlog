//! Bounded, tapering, prefix-scoped fetch for scalable tree views.
//!
//! Instead of loading the whole graph, fetch the focus node's children capped at
//! `limits[0]`, each of their children capped at `limits[1]`, and so on, stopping
//! at `max_depth` — except nodes listed in `expanded`, whose direct children are
//! fetched fully. Each section carries its true `child_count` + a `truncated`
//! flag so the UI can show "+N more". Only children of INCLUDED nodes are
//! fetched, so the result is always a consistent subtree.

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

use hyperlog_core::log::Link;
use sqlx::types::Json;

use crate::state::SharedState;

#[derive(Clone)]
pub struct GetView {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub user_id: Option<uuid::Uuid>,
    pub focus: String, // dot-joined root-relative path; "" = root
    pub expanded: HashSet<String>,
    pub max_depth: i32,
    pub limits: Vec<i32>,
}

pub struct ViewItem {
    pub key: String,
    pub path: Vec<String>, // full incl. root
    pub kind: String,      // root | section | item
    pub title: String,
    pub description: String,
    pub done: bool,
    pub child_count: i32,
    pub truncated: bool,
    pub children: Vec<ViewItem>,
    // PM metadata (items only; empty/0 for sections + root).
    pub due: Option<String>,
    pub created_unix: i64,
    pub links: Vec<Link>,
}

pub struct Response {
    pub root: ViewItem,
}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

#[derive(sqlx::FromRow)]
struct ChildRow {
    path: String,
    item_type: String,
    item_content: Option<Json<serde_json::Value>>,
    created_unix: i64,
    sibling_total: i64,
    own_child_count: i64,
}

const CHILDREN_SQL: &str = r#"
SELECT
    c.path,
    c.item_type,
    c.item_content,
    COALESCE(extract(epoch from c.created_at)::bigint, 0) AS created_unix,
    count(*) OVER () AS sibling_total,
    CASE WHEN c.item_type = 'SECTION' THEN (
        SELECT count(*) FROM nodes g
        WHERE g.root_id = $1 AND g.status = 'active'
          AND g.path LIKE c.path || '.%'
          AND g.path NOT LIKE c.path || '.%.%'
    ) ELSE 0 END AS own_child_count
FROM nodes c
WHERE c.root_id = $1 AND c.status = 'active'
  AND (CASE WHEN $2 = '' THEN c.path NOT LIKE '%.%'
            ELSE c.path LIKE $2 || '.%' AND c.path NOT LIKE $2 || '.%.%' END)
ORDER BY c.sort_order ASC NULLS LAST, c.path
LIMIT $3
"#;

struct ParsedItem {
    title: String,
    description: String,
    done: bool,
    due: Option<String>,
    links: Vec<Link>,
}

fn parse_item(c: &Option<Json<serde_json::Value>>) -> ParsedItem {
    match c {
        Some(j) => {
            let title = j.0.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let description =
                j.0.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let done = j.0.get("state").and_then(|v| v.as_str()) == Some("done");
            let due = j
                .0
                .get("due")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let links = j
                .0
                .get("links")
                .and_then(|v| serde_json::from_value::<Vec<Link>>(v.clone()).ok())
                .unwrap_or_default();
            ParsedItem { title, description, done, due, links }
        }
        None => ParsedItem {
            title: String::new(),
            description: String::new(),
            done: false,
            due: None,
            links: Vec::new(),
        },
    }
}

fn full_path(root_name: &str, rel: &str) -> Vec<String> {
    let mut p = vec![root_name.to_string()];
    if !rel.is_empty() {
        p.extend(rel.split('.').map(|s| s.to_string()));
    }
    p
}

impl GetView {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    /// Fetch the (capped) children of `parent_rel` at `child_depth`. Returns the
    /// built children and the parent's true direct-child total.
    fn children_of<'a>(
        &'a self,
        root_id: uuid::Uuid,
        parent_rel: String,
        child_depth: i32,
        parent_expanded: bool,
        req: &'a Request,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<(Vec<ViewItem>, i64)>> + Send + 'a>> {
        Box::pin(async move {
            let cap: i64 = if parent_expanded {
                100000
            } else {
                req.limits
                    .get((child_depth - 1).max(0) as usize)
                    .copied()
                    .unwrap_or(0) as i64
            };
            if cap <= 0 {
                return Ok((Vec::new(), 0));
            }

            let rows: Vec<ChildRow> = sqlx::query_as(CHILDREN_SQL)
                .bind(root_id)
                .bind(&parent_rel)
                .bind(cap)
                .fetch_all(&self.db)
                .await?;

            let total = rows.first().map(|r| r.sibling_total).unwrap_or(0);
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                let key = r.path.rsplit('.').next().unwrap_or(&r.path).to_string();
                let path = full_path(&req.root, &r.path);
                if r.item_type == "ITEM" {
                    let item = parse_item(&r.item_content);
                    out.push(ViewItem {
                        key,
                        path,
                        kind: "item".into(),
                        title: item.title,
                        description: item.description,
                        done: item.done,
                        child_count: 0,
                        truncated: false,
                        children: Vec::new(),
                        due: item.due,
                        created_unix: r.created_unix,
                        links: item.links,
                    });
                } else {
                    let child_expanded = req.expanded.contains(&r.path);
                    let recurse = child_depth < req.max_depth || child_expanded;
                    let children = if recurse {
                        self.children_of(root_id, r.path.clone(), child_depth + 1, child_expanded, req)
                            .await?
                            .0
                    } else {
                        Vec::new()
                    };
                    let truncated = (r.own_child_count as usize) > children.len();
                    out.push(ViewItem {
                        key,
                        path,
                        kind: "section".into(),
                        title: String::new(),
                        description: String::new(),
                        done: false,
                        child_count: r.own_child_count as i32,
                        truncated,
                        children,
                        due: None,
                        created_unix: r.created_unix,
                        links: Vec::new(),
                    });
                }
            }
            Ok((out, total))
        })
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let Root { id: root_id } = sqlx::query_as(
            r#"SELECT * FROM roots WHERE root_name = $1 AND user_id IS NOT DISTINCT FROM $2"#,
        )
        .bind(&req.root)
        .bind(req.user_id)
        .fetch_one(&self.db)
        .await?;

        let (children, total) = self
            .children_of(root_id, req.focus.clone(), 1, false, &req)
            .await?;

        let root = ViewItem {
            key: full_path(&req.root, &req.focus).last().cloned().unwrap_or_default(),
            path: full_path(&req.root, &req.focus),
            kind: if req.focus.is_empty() { "root".into() } else { "section".into() },
            title: String::new(),
            description: String::new(),
            done: false,
            child_count: total as i32,
            truncated: total as usize > children.len(),
            children,
            due: None,
            created_unix: 0,
            links: Vec::new(),
        };

        Ok(Response { root })
    }
}

pub trait GetViewExt {
    fn get_view_service(&self) -> GetView;
}

impl GetViewExt for SharedState {
    fn get_view_service(&self) -> GetView {
        GetView::new(self.db.clone())
    }
}
