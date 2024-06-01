use hyperlog_core::log::ItemState;
use serde::Serialize;
use tonic::transport::Channel;

use crate::{events::Events, shared_engine::SharedEngine, storage::Storage};

mod local;
mod remote;

#[derive(Serialize, PartialEq, Eq, Debug, Clone)]
pub enum Command {
    CreateRoot {
        root: String,
    },
    CreateSection {
        root: String,
        path: Vec<String>,
    },
    CreateItem {
        root: String,
        path: Vec<String>,
        title: String,
        description: String,
        state: ItemState,
    },
    UpdateItem {
        root: String,
        path: Vec<String>,
        title: String,
        description: String,
        state: ItemState,
    },
    ToggleItem {
        root: String,
        path: Vec<String>,
    },
    Move {
        root: String,
        src: Vec<String>,
        dest: Vec<String>,
    },
    Archive {
        root: String,
        path: Vec<String>,
    },
}

#[derive(Clone)]
enum CommanderVariant {
    Local(local::Commander),
    Remote(remote::Commander),
}

#[derive(Clone)]
pub struct Commander {
    variant: CommanderVariant,
}

impl Commander {
    pub fn local(engine: SharedEngine, storage: Storage, events: Events) -> anyhow::Result<Self> {
        Ok(Self {
            variant: CommanderVariant::Local(local::Commander::new(engine, storage, events)?),
        })
    }

    pub fn remote(channel: Channel) -> anyhow::Result<Self> {
        Ok(Self {
            variant: CommanderVariant::Remote(remote::Commander::new(channel)?),
        })
    }

    pub async fn execute(&self, cmd: Command) -> anyhow::Result<()> {
        match &self.variant {
            CommanderVariant::Local(commander) => commander.execute(cmd),
            CommanderVariant::Remote(commander) => commander.execute(cmd).await,
        }
    }
}
