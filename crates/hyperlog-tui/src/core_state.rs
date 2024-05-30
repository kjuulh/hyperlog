use tonic::transport::{Channel, ClientTlsConfig};

use crate::{
    commander::Commander, events::Events, querier::Querier, shared_engine::SharedEngine,
    storage::Storage,
};

#[allow(dead_code)]
pub struct State {
    engine: SharedEngine,
    pub storage: Storage,
    events: Events,

    pub commander: Commander,
    pub querier: Querier,
}

pub enum Backend {
    Local,
    Remote { url: String },
}

impl State {
    pub async fn new(backend: Backend) -> anyhow::Result<Self> {
        let storage = Storage::new();
        let engine = storage.load()?;
        let events = Events::default();
        let engine = SharedEngine::from(engine);

        let (querier, commander) = match backend {
            Backend::Local => (
                Querier::local(&engine),
                Commander::local(engine.clone(), storage.clone(), events.clone())?,
            ),
            Backend::Remote { url } => {
                let channel = Channel::from_shared(url)?
                    .tls_config(ClientTlsConfig::new())?
                    .connect()
                    .await?;

                (
                    Querier::remote(channel.clone()).await?,
                    Commander::remote(channel)?,
                )
            }
        };

        Ok(Self {
            engine: engine.clone(),
            storage: storage.clone(),
            events: events.clone(),

            commander,
            querier,
        })
    }
}
