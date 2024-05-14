use hyperlog_core::log::GraphItem;

use crate::{
    services::{
        get_available_roots::{self, GetAvailableRoots, GetAvailableRootsExt},
        get_graph::{GetGraph, GetGraphExt},
    },
    state::SharedState,
};

pub struct Querier {
    get_available_roots: GetAvailableRoots,
    get_graph: GetGraph,
}

impl Querier {
    pub fn new(get_available_roots: GetAvailableRoots, get_graph: GetGraph) -> Self {
        Self {
            get_available_roots,
            get_graph,
        }
    }

    pub async fn get_available_roots(&self) -> anyhow::Result<Option<Vec<String>>> {
        let res = self
            .get_available_roots
            .execute(get_available_roots::Request {})
            .await?;

        if res.roots.is_empty() {
            return Ok(None);
        }

        Ok(Some(res.roots))
    }

    pub async fn get(
        &self,
        root: &str,
        path: impl IntoIterator<Item = impl Into<String>>,
    ) -> anyhow::Result<Option<GraphItem>> {
        let graph = self
            .get_graph
            .execute(crate::services::get_graph::Request {
                root: root.into(),
                path: path.into_iter().map(|s| s.into()).collect(),
            })
            .await?;

        Ok(Some(graph.item))
    }
}

pub trait QuerierExt {
    fn querier(&self) -> Querier;
}

impl QuerierExt for SharedState {
    fn querier(&self) -> Querier {
        Querier::new(self.get_available_roots_service(), self.get_graph_service())
    }
}
