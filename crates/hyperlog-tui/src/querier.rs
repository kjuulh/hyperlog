use hyperlog_core::log::GraphItem;

use crate::shared_engine::SharedEngine;

mod local;
mod remote;

#[derive(Clone)]
enum QuerierVariant {
    Local(local::Querier),
}

#[derive(Clone)]
pub struct Querier {
    variant: QuerierVariant,
}

impl Querier {
    pub fn local(engine: &SharedEngine) -> Self {
        Self {
            variant: QuerierVariant::Local(local::Querier::new(engine)),
        }
    }

    pub fn get(
        &self,
        root: &str,
        path: impl IntoIterator<Item = impl Into<String>>,
    ) -> Option<GraphItem> {
        match &self.variant {
            QuerierVariant::Local(querier) => querier.get(root, path),
        }
    }

    pub async fn get_async(
        &self,
        root: &str,
        path: impl IntoIterator<Item = impl Into<String>>,
    ) -> Option<GraphItem> {
        match &self.variant {
            QuerierVariant::Local(querier) => querier.get(root, path),
        }
    }

    pub fn get_available_roots(&self) -> Option<Vec<String>> {
        match &self.variant {
            QuerierVariant::Local(querier) => querier.get_available_roots(),
        }
    }
}
