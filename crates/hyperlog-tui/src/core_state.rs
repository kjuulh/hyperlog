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

impl State {
    pub fn new() -> anyhow::Result<Self> {
        let storage = Storage::new();
        let engine = storage.load()?;
        let events = Events::default();
        let engine = SharedEngine::from(engine);

        Ok(Self {
            engine: engine.clone(),
            storage: storage.clone(),
            events: events.clone(),

            commander: Commander::new(engine.clone(), storage, events)?,
            querier: Querier::local(&engine),
        })
    }
}
