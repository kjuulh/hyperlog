use hyperlog_core::log::GraphItem;

use crate::state::SharedState;

pub struct Querier {}

impl Querier {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_available_roots(&self) -> Option<Vec<String>> {
        todo!()
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
        Querier::new()
    }
}
