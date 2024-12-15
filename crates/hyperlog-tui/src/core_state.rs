use std::path::PathBuf;

use tonic::transport::{Channel, ClientTlsConfig};

use crate::{
    commander::Commander, events::Events, querier::Querier, shared_engine::SharedEngine,
    storage::Storage,
};

#[allow(dead_code)]
pub struct State {
    pub commander: Commander,
    pub querier: Querier,

    backend: Backend,
}

pub enum Backend {
    Local { path_override: Option<PathBuf> },
    Remote { url: String },
}

impl State {
    pub async fn new(backend: Backend) -> anyhow::Result<Self> {
        let (querier, commander) = match &backend {
            Backend::Local { path_override } => {
                let mut storage = Storage::new();
                if let Some(path_override) = path_override {
                    storage.with_base(path_override);
                }
                let engine = storage.load()?;
                let events = Events::default();
                let engine = SharedEngine::from(engine);
                (
                    Querier::local(&engine),
                    Commander::local(engine.clone(), storage.clone(), events.clone())?,
                )
            }
            Backend::Remote { url } => {
                let tls = ClientTlsConfig::new();
                let channel = Channel::from_shared(url.clone())?
                    .tls_config(tls.with_native_roots())?
                    .connect()
                    .await?;

                (
                    Querier::remote(channel.clone()).await?,
                    Commander::remote(channel)?,
                )
            }
        };

        Ok(Self {
            commander,
            querier,
            backend,
        })
    }

    pub fn unlock(&self) {
        if let Backend::Local { path_override } = &self.backend {
            let mut storage = Storage::new();
            if let Some(path_override) = path_override {
                storage.with_base(path_override);
            }
            storage.clear_lock_file();
        }
    }

    pub fn info(&self) -> Option<anyhow::Result<String>> {
        if let Backend::Local { path_override } = &self.backend {
            let mut storage = Storage::new();
            if let Some(path_override) = path_override {
                storage.with_base(path_override);
            }
            return Some(storage.info());
        }

        None
    }
}
