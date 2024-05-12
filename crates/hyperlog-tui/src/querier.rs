use hyperlog_core::log::GraphItem;
use tonic::transport::Channel;

use crate::shared_engine::SharedEngine;

mod local;
mod remote;

#[derive(Clone)]
enum QuerierVariant {
    Local(local::Querier),
    Remote(remote::Querier),
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

    pub async fn remote(channel: Channel) -> anyhow::Result<Self> {
        Ok(Self {
            variant: QuerierVariant::Remote(remote::Querier::new(channel).await?),
        })
    }

    pub fn get(
        &self,
        root: &str,
        path: impl IntoIterator<Item = impl Into<String>>,
    ) -> Option<GraphItem> {
        match &self.variant {
            QuerierVariant::Local(querier) => querier.get(root, path),
            QuerierVariant::Remote(_) => todo!(),
        }
    }

    pub async fn get_async(
        &self,
        root: &str,
        path: impl IntoIterator<Item = impl Into<String>>,
    ) -> anyhow::Result<Option<GraphItem>> {
        match &self.variant {
            QuerierVariant::Local(querier) => Ok(querier.get(root, path)),
            QuerierVariant::Remote(querier) => querier.get(root, path).await,
        }
    }

    pub fn get_available_roots(&self) -> Option<Vec<String>> {
        match &self.variant {
            QuerierVariant::Local(querier) => querier.get_available_roots(),
            QuerierVariant::Remote(_) => todo!(),
        }
    }

    pub async fn get_available_roots_async(&self) -> anyhow::Result<Option<Vec<String>>> {
        match &self.variant {
            QuerierVariant::Local(querier) => Ok(querier.get_available_roots()),
            QuerierVariant::Remote(querier) => querier.get_available_roots().await,
        }
    }
}
