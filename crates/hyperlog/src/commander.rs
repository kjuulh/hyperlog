use std::{collections::BTreeMap, sync::RwLock};

use serde::Serialize;

use crate::{
    engine::Engine,
    events::Events,
    log::{GraphItem, ItemState},
};

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
    Move {
        root: String,
        src: Vec<String>,
        dest: Vec<String>,
    },
}

#[derive(Default)]
pub struct Commander {
    engine: RwLock<Engine>,
    events: Events,
}

impl Commander {
    pub fn execute(&self, cmd: Command) -> anyhow::Result<()> {
        tracing::debug!("executing event: {}", serde_json::to_string(&cmd)?);

        match cmd {
            Command::CreateRoot { root } => {
                self.engine.write().unwrap().create_root(&root)?;
            }
            Command::CreateSection { root, path } => {
                self.engine.write().unwrap().create(
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
            } => self.engine.write().unwrap().create(
                &root,
                &path.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                GraphItem::Item {
                    title,
                    description,
                    state,
                },
            )?,
            Command::Move { root, src, dest } => self.engine.write().unwrap().section_move(
                &root,
                &src.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                &dest.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            )?,
        }

        Ok(())
    }
}
