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
    Remote,
}

impl State {
    pub async fn new(backend: Backend) -> anyhow::Result<Self> {
        let storage = Storage::new();
        let engine = storage.load()?;
        let events = Events::default();
        let engine = SharedEngine::from(engine);
        let querier = match backend {
            Backend::Local => Querier::local(&engine),
            Backend::Remote => Querier::remote().await?,
        };

        Ok(Self {
            engine: engine.clone(),
            storage: storage.clone(),
            events: events.clone(),

            commander: Commander::new(engine.clone(), storage, events)?,
            querier,
        })
    }
}
