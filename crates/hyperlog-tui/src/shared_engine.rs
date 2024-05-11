use std::sync::{Arc, RwLock};

use hyperlog_core::log::GraphItem;

use crate::engine::Engine;

#[derive(Clone)]
pub struct SharedEngine {
    inner: Arc<RwLock<Engine>>,
}

impl From<Engine> for SharedEngine {
    fn from(value: Engine) -> Self {
        SharedEngine {
            inner: Arc::new(RwLock::new(value)),
        }
    }
}

impl SharedEngine {
    pub fn to_str(&self) -> anyhow::Result<String> {
        self.inner.read().unwrap().to_str()
    }

    pub fn create_root(&self, root: &str) -> anyhow::Result<()> {
        self.inner.write().unwrap().create_root(root)
    }

    pub fn create(&self, root: &str, path: &[&str], item: GraphItem) -> anyhow::Result<()> {
        self.inner.write().unwrap().create(root, path, item)
    }

    pub fn get(&self, root: &str, path: &[&str]) -> Option<GraphItem> {
        self.inner.read().unwrap().get(root, path).cloned()
    }

    pub fn section_move(
        &self,
        root: &str,
        src_path: &[&str],
        dest_path: &[&str],
    ) -> anyhow::Result<()> {
        self.inner
            .write()
            .unwrap()
            .section_move(root, src_path, dest_path)
    }

    pub fn toggle_item(&self, root: &str, path: &[&str]) -> anyhow::Result<()> {
        self.inner.write().unwrap().toggle_item(root, path)?;

        Ok(())
    }

    pub(crate) fn update_item(
        &self,
        root: &str,
        path: &[&str],
        state: GraphItem,
    ) -> anyhow::Result<()> {
        self.inner
            .write()
            .unwrap()
            .update_item(root, path, &state)?;

        Ok(())
    }

    pub(crate) fn get_roots(&self) -> Option<Vec<String>> {
        self.inner.read().unwrap().get_roots()
    }
}
