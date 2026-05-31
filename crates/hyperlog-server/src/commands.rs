use hyperlog_core::log::ItemState;

use crate::{
    services::{
        archive::{self, Archive, ArchiveExt},
        create_item::{self, CreateItem, CreateItemExt},
        create_root::{self, CreateRoot, CreateRootExt},
        create_section::{self, CreateSection, CreateSectionExt},
        move_node::{self, MoveNode, MoveNodeExt},
        reorder::{self, Reorder, ReorderExt},
        restore::{self, Restore, RestoreExt},
        toggle_item::{self, ToggleItem, ToggleItemExt},
        update_item::{self, UpdateItem, UpdateItemExt},
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
        due: Option<String>,
        links: Vec<hyperlog_core::log::Link>,
    },
    UpdateItem {
        root: String,
        path: Vec<String>,
        title: String,
        description: String,
        state: ItemState,
        due: Option<String>,
        links: Vec<hyperlog_core::log::Link>,
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
    Reorder {
        root: String,
        path: Vec<String>,
        order: Vec<String>,
    },
    Archive {
        root: String,
        path: Vec<String>,
    },
    Restore {
        root: String,
        path: Vec<String>,
    },
}

#[allow(dead_code)]
pub struct Commander {
    create_root: CreateRoot,
    create_section: CreateSection,
    create_item: CreateItem,
    update_item: UpdateItem,
    toggle_item: ToggleItem,
    archive: Archive,
    restore: Restore,
    move_node: MoveNode,
    reorder: Reorder,
}

impl Commander {
    pub fn new(
        create_root: CreateRoot,
        create_section: CreateSection,
        create_item: CreateItem,
        update_item: UpdateItem,
        toggle_item: ToggleItem,
        archive: Archive,
        restore: Restore,
        move_node: MoveNode,
        reorder: Reorder,
    ) -> Self {
        Self {
            create_root,
            create_section,
            create_item,
            update_item,
            toggle_item,
            archive,
            restore,
            move_node,
            reorder,
        }
    }

    pub async fn execute(
        &self,
        cmd: Command,
        user_id: Option<uuid::Uuid>,
    ) -> anyhow::Result<()> {
        match cmd {
            Command::CreateRoot { root } => {
                self.create_root
                    .execute(create_root::Request { root, user_id })
                    .await?;

                Ok(())
            }
            Command::CreateSection { root, path } => {
                self.create_section
                    .execute(create_section::Request {
                        root,
                        path,
                        user_id,
                    })
                    .await?;

                Ok(())
            }
            Command::CreateItem {
                root,
                path,
                title,
                description,
                state,
                due,
                links,
            } => {
                self.create_item
                    .execute(create_item::Request {
                        root,
                        path,
                        user_id,
                        title,
                        description,
                        state,
                        due,
                        links,
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
                due,
                links,
            } => {
                self.update_item
                    .execute(update_item::Request {
                        root,
                        path,
                        user_id,
                        title,
                        description,
                        state,
                        due,
                        links,
                    })
                    .await?;

                Ok(())
            }
            Command::ToggleItem { root, path } => {
                self.toggle_item
                    .execute(toggle_item::Request {
                        root,
                        path,
                        user_id,
                    })
                    .await?;

                Ok(())
            }
            Command::Move { root, src, dest } => {
                self.move_node
                    .execute(move_node::Request {
                        root,
                        src,
                        dest,
                        user_id,
                    })
                    .await?;

                Ok(())
            }
            Command::Reorder { root, path, order } => {
                self.reorder
                    .execute(reorder::Request {
                        root,
                        path,
                        order,
                        user_id,
                    })
                    .await?;

                Ok(())
            }
            Command::Archive { root, path } => {
                self.archive
                    .execute(archive::Request {
                        root,
                        path,
                        user_id,
                    })
                    .await?;

                Ok(())
            }
            Command::Restore { root, path } => {
                self.restore
                    .execute(restore::Request {
                        root,
                        path,
                        user_id,
                    })
                    .await?;

                Ok(())
            }
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
            self.toggle_item_service(),
            self.archive_service(),
            self.restore_service(),
            self.move_node_service(),
            self.reorder_service(),
        )
    }
}
