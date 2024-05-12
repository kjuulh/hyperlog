use hyperlog_core::log::ItemState;
use serde::Serialize;

use crate::{events::Events, shared_engine::SharedEngine, storage::Storage};

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
}

#[derive(Clone)]
enum CommanderVariant {
    Local(local::Commander),
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

    pub async fn execute(&self, cmd: Command) -> anyhow::Result<()> {
        match &self.variant {
            CommanderVariant::Local(commander) => commander.execute(cmd),
        }
    }
}

mod local {
    use std::collections::BTreeMap;

    use hyperlog_core::log::GraphItem;

    use crate::{events::Events, shared_engine::SharedEngine, storage::Storage};

    use super::Command;

    #[derive(Clone)]
    pub struct Commander {
        engine: SharedEngine,
        storage: Storage,
        events: Events,
    }

    impl Commander {
        pub fn new(engine: SharedEngine, storage: Storage, events: Events) -> anyhow::Result<Self> {
            Ok(Self {
                engine,
                storage,
                events,
            })
        }

        pub fn execute(&self, cmd: Command) -> anyhow::Result<()> {
            tracing::debug!("executing event: {}", serde_json::to_string(&cmd)?);

            match cmd.clone() {
                Command::CreateRoot { root } => {
                    self.engine.create_root(&root)?;
                }
                Command::CreateSection { root, path } => {
                    self.engine.create(
                        &root,
                        &path.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                        GraphItem::Section(BTreeMap::default()),
                    )?;
                }
                Command::CreateItem {
                    root,
                    path,
                    title,
                    description,
                    state,
                } => self.engine.create(
                    &root,
                    &path.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                    GraphItem::Item {
                        title,
                        description,
                        state,
                    },
                )?,
                Command::Move { root, src, dest } => self.engine.section_move(
                    &root,
                    &src.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                    &dest.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                )?,
                Command::ToggleItem { root, path } => self
                    .engine
                    .toggle_item(&root, &path.iter().map(|p| p.as_str()).collect::<Vec<_>>())?,
                Command::UpdateItem {
                    root,
                    path,
                    title,
                    description,
                    state,
                } => self.engine.update_item(
                    &root,
                    &path.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                    GraphItem::Item {
                        title,
                        description,
                        state,
                    },
                )?,
            }

            self.storage.store(&self.engine)?;

            self.events.enque_command(cmd)?;

            Ok(())
        }
    }
}
