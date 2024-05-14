use hyperlog_core::log::ItemState;

use crate::{
    services::{
        create_item::{self, CreateItem, CreateItemExt},
        create_root::{self, CreateRoot, CreateRootExt},
        create_section::{self, CreateSection, CreateSectionExt},
        update_item::{UpdateItem, UpdateItemExt},
    },
    state::SharedState,
};

#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct Commander {
    create_root: CreateRoot,
    create_section: CreateSection,
    create_item: CreateItem,
    update_item: UpdateItem,
}

impl Commander {
    pub fn new(
        create_root: CreateRoot,
        create_section: CreateSection,
        create_item: CreateItem,
        update_item: UpdateItem,
    ) -> Self {
        Self {
            create_root,
            create_section,
            create_item,
            update_item,
        }
    }

    pub async fn execute(&self, cmd: Command) -> anyhow::Result<()> {
        match cmd {
            Command::CreateRoot { root } => {
                self.create_root
                    .execute(create_root::Request { root })
                    .await?;

                Ok(())
            }
            Command::CreateSection { root, path } => {
                self.create_section
                    .execute(create_section::Request { root, path })
                    .await?;

                Ok(())
            }
            Command::CreateItem {
                root,
                path,
                title,
                description,
                state,
            } => {
                self.create_item
                    .execute(create_item::Request {
                        root,
                        path,
                        title,
                        description,
                        state,
                    })
                    .await?;

                Ok(())
            }
            Command::UpdateItem {
                root,
                path,
                title,
                description,
                state,
            } => {
                self.update_item
                    .execute(crate::services::update_item::Request {
                        root,
                        path,
                        title,
                        description,
                        state,
                    })
                    .await?;

                Ok(())
            }
            Command::ToggleItem { root, path } => todo!(),
            Command::Move { root, src, dest } => todo!(),
        }
    }
}

pub trait CommanderExt {
    fn commander(&self) -> Commander;
}

impl CommanderExt for SharedState {
    fn commander(&self) -> Commander {
        Commander::new(
            self.create_root_service(),
            self.create_section_service(),
            self.create_item_service(),
            self.update_item_service(),
        )
    }
}
