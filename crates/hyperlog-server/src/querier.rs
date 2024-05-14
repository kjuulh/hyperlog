use hyperlog_core::log::GraphItem;

use crate::{
    services::get_available_roots::{self, GetAvailableRoots, GetAvailableRootsExt},
    state::SharedState,
};

pub struct Querier {
    get_available_roots: GetAvailableRoots,
}

impl Querier {
    pub fn new(get_available_roots: GetAvailableRoots) -> Self {
        Self {
            get_available_roots,
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

    pub fn get(
        &self,
        root: &str,
        path: impl IntoIterator<Item = impl Into<String>>,
    ) -> Option<GraphItem> {
        todo!()
    }
}

pub trait QuerierExt {
    fn querier(&self) -> Querier;
}

impl QuerierExt for SharedState {
    fn querier(&self) -> Querier {
        Querier::new(self.get_available_roots_service())
    }
}
