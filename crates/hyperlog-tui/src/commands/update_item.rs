use hyperlog_core::log::ItemState;
use itertools::Itertools;

use crate::{
    commander::{self, Commander},
    models::IOEvent,
    state::SharedState,
};

pub struct UpdateItemCommand {
    commander: Commander,
}

impl UpdateItemCommand {
    pub fn new(commander: Commander) -> Self {
        Self { commander }
    }

    pub fn command(
        self,
        root: &str,
        path: &[&str],
        title: &str,
        description: &str,
        state: ItemState,
    ) -> super::Command {
        let root = root.to_owned();
        let path = path.iter().map(|s| s.to_string()).collect_vec();
        let title = title.to_string();
        let description = description.to_string();
        let state = state.clone();

        super::Command::new(|dispatch| {
            tokio::spawn(async move {
                dispatch.send(crate::models::Msg::ItemUpdated(IOEvent::Initialized));

                match self
                    .commander
                    .execute(commander::Command::UpdateItem {
                        root,
                        path,
                        title,
                        description,
                        state,
                    })
                    .await
                {
                    Ok(()) => {
                        #[cfg(debug_assertions)]
                        {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }

                        dispatch.send(crate::models::Msg::ItemUpdated(IOEvent::Success(())));
                    }
                    Err(e) => {
                        dispatch.send(crate::models::Msg::ItemUpdated(IOEvent::Failure(
                            e.to_string(),
                        )));
                    }
                }
            });
            None
        })
    }
}

pub trait UpdateItemCommandExt {
    fn update_item_command(&self) -> UpdateItemCommand;
}

impl UpdateItemCommandExt for SharedState {
    fn update_item_command(&self) -> UpdateItemCommand {
        UpdateItemCommand::new(self.commander.clone())
    }
}
