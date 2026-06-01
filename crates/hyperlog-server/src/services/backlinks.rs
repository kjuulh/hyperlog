//! "What links here" — items whose body contains a `[[wiki-link]]` that
//! resolves to a target node. Computed server-side (one query over the root,
//! resolved in Rust) so clients don't download the whole graph just to show
//! backlinks. Mirrors the client's resolution exactly: a link target resolves
//! by relative slash-path first, then by item title / section key (first match
//! in path order), case-insensitive.

use std::collections::HashMap;

use sqlx::types::Json;

use crate::state::SharedState;

#[derive(Clone)]
pub struct Backlinks {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub user_id: Option<uuid::Uuid>,
    pub path: Vec<String>, // root-relative path of the target node
}

pub struct Hit {
    pub key: String,
    pub path: Vec<String>, // full incl. root
    pub title: String,
    pub description: String,
    pub done: bool,
}

pub struct Response {
    pub items: Vec<Hit>,
}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

#[derive(sqlx::FromRow)]
struct NodeRow {
    path: String, // dotted, root-relative
    item_type: String,
    item_content: Option<Json<serde_json::Value>>,
}

fn norm(s: &str) -> String {
    s.trim().to_lowercase()
}

/// The link targets written in [body] (`[[target]]` / `[[target|alias]]`),
/// matching the client regex `\[\[([^\[\]\n]+?)\]\]` (no brackets/newlines).
fn parse_targets(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(open) = rest.find("[[") {
        let after = &rest[open + 2..];
        if let Some(close) = after.find("]]") {
            let inner = &after[..close];
            if !inner.is_empty()
                && !inner.contains('[')
                && !inner.contains(']')
                && !inner.contains('\n')
            {
                let target = inner.split('|').next().unwrap_or("").trim();
                if !target.is_empty() {
                    out.push(target.to_string());
                }
            }
            rest = &after[close + 2..];
        } else {
            break;
        }
    }
    out
}

fn display_name(item_type: &str, key: &str, content: &Option<Json<serde_json::Value>>) -> String {
    if item_type == "ITEM" {
        if let Some(t) = content
            .as_ref()
            .and_then(|j| j.0.get("title"))
            .and_then(|v| v.as_str())
        {
            let t = t.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    key.to_string()
}

impl Backlinks {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let target_rel = req.path.join(".");
        if target_rel.is_empty() {
            return Ok(Response { items: Vec::new() });
        }

        let Root { id: root_id } = sqlx::query_as(
            r#"SELECT * FROM roots WHERE root_name = $1 AND user_id IS NOT DISTINCT FROM $2"#,
        )
        .bind(&req.root)
        .bind(req.user_id)
        .fetch_one(&self.db)
        .await?;

        // path order ≈ the client's DFS-with-sorted-siblings walk, so "first
        // match" resolution lines up.
        let rows: Vec<NodeRow> = sqlx::query_as(
            r#"SELECT path, item_type, item_content FROM nodes
               WHERE root_id = $1 AND status = 'active' ORDER BY path"#,
        )
        .bind(root_id)
        .fetch_all(&self.db)
        .await?;

        // Build resolution maps (first match wins) over all nodes.
        let mut by_path: HashMap<String, usize> = HashMap::new();
        let mut by_name: HashMap<String, usize> = HashMap::new();
        let mut target_idx: Option<usize> = None;
        for (i, r) in rows.iter().enumerate() {
            if r.path == target_rel {
                target_idx = Some(i);
            }
            by_path.entry(norm(&r.path.replace('.', "/"))).or_insert(i);
            let key = r.path.rsplit('.').next().unwrap_or(&r.path);
            let name = display_name(&r.item_type, key, &r.item_content);
            by_name.entry(norm(&name)).or_insert(i);
            by_name.entry(norm(key)).or_insert(i);
        }
        let Some(target_idx) = target_idx else {
            return Ok(Response { items: Vec::new() });
        };

        let resolve = |t: &str| -> Option<usize> {
            let t = t.trim();
            if t.is_empty() {
                return None;
            }
            let n = norm(t);
            if t.contains('/') {
                if let Some(&i) = by_path.get(&n) {
                    return Some(i);
                }
            }
            by_name.get(&n).or_else(|| by_path.get(&n)).copied()
        };

        let mut items = Vec::new();
        for (i, r) in rows.iter().enumerate() {
            if i == target_idx || r.item_type != "ITEM" {
                continue;
            }
            let body = r
                .item_content
                .as_ref()
                .and_then(|j| j.0.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !body.contains("[[") {
                continue;
            }
            let links_here = parse_targets(body)
                .iter()
                .any(|t| resolve(t) == Some(target_idx));
            if !links_here {
                continue;
            }
            let key = r.path.rsplit('.').next().unwrap_or(&r.path).to_string();
            let mut path = vec![req.root.clone()];
            path.extend(r.path.split('.').map(|s| s.to_string()));
            let (title, description, done) = match &r.item_content {
                Some(j) => (
                    j.0.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    body.to_string(),
                    j.0.get("state").and_then(|v| v.as_str()) == Some("done"),
                ),
                None => (String::new(), String::new(), false),
            };
            items.push(Hit { key, path, title, description, done });
        }

        Ok(Response { items })
    }
}

pub trait BacklinksExt {
    fn backlinks_service(&self) -> Backlinks;
}

impl BacklinksExt for SharedState {
    fn backlinks_service(&self) -> Backlinks {
        Backlinks::new(self.db.clone())
    }
}
