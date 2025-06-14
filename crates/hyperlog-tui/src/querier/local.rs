use hyperlog_core::log::GraphItem;

use crate::shared_engine::SharedEngine;

#[derive(Clone)]
pub struct Querier {
    engine: SharedEngine,
}

impl Querier {
    pub fn new(engine: &SharedEngine) -> Self {
        Self {
            engine: engine.clone(),
        }
    }

    pub fn get_available_roots(&self) -> Option<Vec<String>> {
        self.engine.get_roots()
    }

    pub fn get(
        &self,
        root: &str,
        path: impl IntoIterator<Item = impl Into<String>>,
    ) -> Option<GraphItem> {
        let path = path
            .into_iter()
            .map(|i| i.into())
            .filter(|i| !i.is_empty())
            .collect::<Vec<String>>();

        tracing::debug!(
            "quering: root:({}), path:({}), len: ({}))",
            root,
            path.join("."),
            path.len()
        );

        let item = self
            .engine
            .get(root, &path.iter().map(|i| i.as_str()).collect::<Vec<_>>());

        item
    }
}
