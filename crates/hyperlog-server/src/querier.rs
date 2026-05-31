use hyperlog_core::log::GraphItem;

use std::collections::HashSet;

use crate::{
    services::{
        get_archived::{self, ArchivedItem, GetArchived, GetArchivedExt},
        get_available_roots::{self, GetAvailableRoots, GetAvailableRootsExt},
        get_graph::{GetGraph, GetGraphExt},
        get_view::{self, GetView, GetViewExt, ViewItem},
    },
    state::SharedState,
};

pub struct Querier {
    get_available_roots: GetAvailableRoots,
    get_graph: GetGraph,
    get_archived: GetArchived,
    get_view: GetView,
}

impl Querier {
    pub fn new(
        get_available_roots: GetAvailableRoots,
        get_graph: GetGraph,
        get_archived: GetArchived,
        get_view: GetView,
    ) -> Self {
        Self {
            get_available_roots,
            get_graph,
            get_archived,
            get_view,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_view(
        &self,
        root: &str,
        user_id: Option<uuid::Uuid>,
        focus: String,
        expanded: HashSet<String>,
        max_depth: i32,
        limits: Vec<i32>,
    ) -> anyhow::Result<ViewItem> {
        let res = self
            .get_view
            .execute(get_view::Request {
                root: root.into(),
                user_id,
                focus,
                expanded,
                max_depth,
                limits,
            })
            .await?;
        Ok(res.root)
    }

    pub async fn get_archived(
        &self,
        root: &str,
        user_id: Option<uuid::Uuid>,
    ) -> anyhow::Result<Vec<ArchivedItem>> {
        let res = self
            .get_archived
            .execute(get_archived::Request {
                root: root.into(),
                user_id,
            })
            .await?;
        Ok(res.items)
    }

    pub async fn get_available_roots(
        &self,
        user_id: Option<uuid::Uuid>,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let res = self
            .get_available_roots
            .execute(get_available_roots::Request { user_id })
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
        user_id: Option<uuid::Uuid>,
    ) -> anyhow::Result<Option<GraphItem>> {
        let graph = self
            .get_graph
            .execute(crate::services::get_graph::Request {
                root: root.into(),
                path: path.into_iter().map(|s| s.into()).collect(),
                user_id,
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
        Querier::new(
            self.get_available_roots_service(),
            self.get_graph_service(),
            self.get_archived_service(),
            self.get_view_service(),
        )
    }
}
