use std::collections::BTreeMap;

use hyperlog_core::log::{GraphItem, ItemState};
use serde::Deserialize;
use sqlx::types::Json;

use crate::state::SharedState;

use self::engine::Engine;

#[derive(Clone)]
pub struct GetGraph {
    db: sqlx::PgPool,
}

pub struct Request {
    pub root: String,
    pub path: Vec<String>,
}
pub struct Response {
    pub item: GraphItem,
}

#[derive(sqlx::FromRow)]
struct Root {
    id: uuid::Uuid,
}

#[derive(Deserialize)]
struct Item {
    title: String,
    description: String,
    state: ItemState,
}

#[derive(sqlx::FromRow, Debug)]
struct Node {
    path: String,
    item_type: String,
    item_content: Option<Json<serde_json::Value>>,
}

impl GetGraph {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn execute(&self, req: Request) -> anyhow::Result<Response> {
        let Root { id: root_id, .. } =
            sqlx::query_as(r#"SELECT * FROM roots WHERE root_name = $1"#)
                .bind(&req.root)
                .fetch_one(&self.db)
                .await?;

        let nodes: Vec<Node> = sqlx::query_as(
            r#"
    SELECT
        *
    FROM
        nodes
    WHERE
        root_id = $1
    LIMIT
        1000
            "#,
        )
        .bind(root_id)
        .fetch_all(&self.db)
        .await?;

        let item = self.build_graph(req.root, req.path, nodes)?;

        Ok(Response { item })
    }

    fn build_graph(
        &self,
        root: String,
        path: Vec<String>,
        mut nodes: Vec<Node>,
    ) -> anyhow::Result<GraphItem> {
        nodes.sort_by(|a, b| a.path.cmp(&b.path));
        let mut engine = Engine::default();
        engine.create_root(&root)?;

        self.get_graph_items(&root, &mut engine, &nodes)?;

        engine
            .get(&root, &path.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .ok_or(anyhow::anyhow!("failed to find a valid graph"))
            .cloned()
    }

    fn get_graph_items(
        &self,
        root: &str,
        engine: &mut Engine,
        nodes: &Vec<Node>,
    ) -> anyhow::Result<()> {
        for node in nodes {
            if let Some(item) = self.get_graph_item(node) {
                let path = node.path.split('.').collect::<Vec<_>>();
                engine.create(root, &path, item)?;
            }
        }

        Ok(())
    }

    fn get_graph_item(&self, node: &Node) -> Option<GraphItem> {
        match node.item_type.as_str() {
            "SECTION" => Some(GraphItem::Section(BTreeMap::default())),
            "ITEM" => {
                if let Some(content) = &node.item_content {
                    let item: Item = serde_json::from_value(content.0.clone()).ok()?;

                    Some(GraphItem::Item {
                        title: item.title,
                        description: item.description,
                        state: item.state,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub trait GetGraphExt {
    fn get_graph_service(&self) -> GetGraph;
}

impl GetGraphExt for SharedState {
    fn get_graph_service(&self) -> GetGraph {
        GetGraph::new(self.db.clone())
    }
}

mod engine {

    use std::{collections::BTreeMap, fmt::Display};

    use anyhow::{anyhow, Context};
    use hyperlog_core::log::{Graph, GraphItem, ItemState};

    #[derive(Default)]
    pub struct Engine {
        graph: Graph,
    }

    impl Engine {
        #[allow(dead_code)]
        pub fn engine_from_str(input: &str) -> anyhow::Result<Self> {
            let graph: Graph = serde_json::from_str(input)?;

            Ok(Self { graph })
        }

        #[allow(dead_code)]
        pub fn to_str(&self) -> anyhow::Result<String> {
            serde_json::to_string_pretty(&self.graph).context("failed to serialize graph")
        }

        pub fn create_root(&mut self, root: &str) -> anyhow::Result<()> {
            self.graph
                .try_insert(root.to_string(), GraphItem::User(BTreeMap::default()))
                .map_err(|_| anyhow!("entry was already found, aborting"))?;

            Ok(())
        }

        pub fn create(&mut self, root: &str, path: &[&str], item: GraphItem) -> anyhow::Result<()> {
            let graph = &mut self.graph;

            let (last, items) = path.split_last().ok_or(anyhow!(
                "path cannot be empty, must contain at least one item"
            ))?;

            let root = graph
                .get_mut(root)
                .ok_or(anyhow!("root was missing a user, aborting"))?;

            let mut current_item = root;
            for section in items {
                match current_item {
                    GraphItem::User(u) => match u.get_mut(section.to_owned()) {
                        Some(graph_item) => {
                            current_item = graph_item;
                        }
                        None => anyhow::bail!("path: {} section was not found", section),
                    },
                    GraphItem::Item { .. } => anyhow::bail!("path: {} was already found", section),
                    GraphItem::Section(s) => match s.get_mut(section.to_owned()) {
                        Some(graph_item) => {
                            current_item = graph_item;
                        }
                        None => anyhow::bail!("path: {} section was not found", section),
                    },
                }
            }

            match current_item {
                GraphItem::User(u) => {
                    u.insert(last.to_string(), item);
                }
                GraphItem::Section(s) => {
                    s.insert(last.to_string(), item);
                }
                GraphItem::Item { .. } => anyhow::bail!("cannot insert an item into an item"),
            }

            Ok(())
        }

        pub fn get(&self, root: &str, path: &[&str]) -> Option<&GraphItem> {
            let root = self.graph.get(root)?;

            root.get(path)
        }

        #[allow(dead_code)]
        pub fn get_mut(&mut self, root: &str, path: &[&str]) -> Option<&mut GraphItem> {
            let root = self.graph.get_mut(root)?;

            root.get_mut(path)
        }

        #[allow(dead_code)]
        pub fn take(&mut self, root: &str, path: &[&str]) -> Option<GraphItem> {
            let root = self.graph.get_mut(root)?;

            root.take(path)
        }

        #[allow(dead_code)]
        pub fn section_move(
            &mut self,
            root: &str,
            src_path: &[&str],
            dest_path: &[&str],
        ) -> anyhow::Result<()> {
            let src = self
                .take(root, src_path)
                .ok_or(anyhow!("failed to find source path"))?;

            let dest = self
                .get_mut(root, dest_path)
                .ok_or(anyhow!("failed to find destination"))?;

            let src_item = src_path
                .last()
                .ok_or(anyhow!("src path must have at least one item"))?;

            match dest {
                GraphItem::User(u) => {
                    u.try_insert(src_item.to_string(), src)
                        .map_err(|_e| anyhow!("key was already found, aborting: {}", src_item))?;
                }
                GraphItem::Section(s) => {
                    s.try_insert(src_item.to_string(), src)
                        .map_err(|_e| anyhow!("key was already found, aborting: {}", src_item))?;
                }
                GraphItem::Item { .. } => {
                    anyhow::bail!(
                        "failed to insert src at item, item doesn't support arbitrary items"
                    )
                }
            }

            Ok(())
        }

        #[allow(dead_code)]
        pub fn delete(&mut self, root: &str, path: &[&str]) -> anyhow::Result<()> {
            self.take(root, path)
                .map(|_| ())
                .ok_or(anyhow!("item was not found"))
        }

        #[allow(dead_code)]
        pub fn toggle_item(&mut self, root: &str, path: &[&str]) -> anyhow::Result<()> {
            if let Some(item) = self.get_mut(root, path) {
                match item {
                    GraphItem::Item { state, .. } => match state {
                        ItemState::NotDone => *state = ItemState::Done,
                        ItemState::Done => *state = ItemState::NotDone,
                    },
                    _ => {
                        anyhow::bail!("{}.{:?} is not an item", root, path)
                    }
                }
            }

            Ok(())
        }

        #[allow(dead_code)]
        pub fn update_item(
            &mut self,
            root: &str,
            path: &[&str],
            item: &GraphItem,
        ) -> anyhow::Result<()> {
            if let Some((name, dest_last)) = path.split_last() {
                if let Some(parent) = self.get_mut(root, dest_last) {
                    match parent {
                        GraphItem::User(s) | GraphItem::Section(s) => {
                            if let Some(mut existing) = s.remove(*name) {
                                match (&mut existing, item) {
                                    (
                                        GraphItem::Item {
                                            title: ex_title,
                                            description: ex_desc,
                                            state: ex_state,
                                        },
                                        GraphItem::Item {
                                            title,
                                            description,
                                            state,
                                        },
                                    ) => {
                                        ex_title.clone_from(title);
                                        ex_desc.clone_from(description);
                                        ex_state.clone_from(state);

                                        let title = title.replace(".", "-");
                                        s.insert(title, existing.clone());
                                    }
                                    _ => {
                                        anyhow::bail!(
                                            "path: {}.{} found is not an item",
                                            root,
                                            path.join(".")
                                        )
                                    }
                                }
                            }
                        }
                        GraphItem::Item { .. } => {
                            anyhow::bail!("cannot rename when item is placed in an item")
                        }
                    }
                }
            }

            Ok(())
        }

        #[allow(dead_code)]
        pub fn get_roots(&self) -> Option<Vec<String>> {
            let items = self.graph.keys().cloned().collect::<Vec<_>>();
            if items.is_empty() {
                None
            } else {
                Some(items)
            }
        }
    }

    impl Display for Engine {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let output = serde_json::to_string_pretty(&self.graph).unwrap();
            f.write_str(&output)
        }
    }
}
