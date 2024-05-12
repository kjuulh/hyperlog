use itertools::Itertools;

use crate::{
    commander::{self, Commander},
    models::IOEvent,
    state::SharedState,
};

pub struct ToggleItemCommand {
    commander: Commander,
}

impl ToggleItemCommand {
    pub fn new(commander: Commander) -> Self {
        Self { commander }
    }

    pub fn command(self, root: &str, path: &[&str]) -> super::Command {
        let root = root.to_owned();
        let path = path.iter().map(|s| s.to_string()).collect_vec();

        super::Command::new(|dispatch| {
            tokio::spawn(async move {
                dispatch.send(crate::models::Msg::ItemToggled(IOEvent::Initialized));

                match self
                    .commander
                    .execute(commander::Command::ToggleItem { root, path })
                    .await
                {
                    Ok(()) => {
                        #[cfg(debug_assertions)]
                        {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }

                        dispatch.send(crate::models::Msg::ItemToggled(IOEvent::Success(())));
                    }
                    Err(e) => {
                        dispatch.send(crate::models::Msg::ItemToggled(IOEvent::Failure(
                            e.to_string(),
                        )));
                    }
                }
            });
            None
        })
    }
}

pub trait ToggleItemCommandExt {
    fn toggle_item_command(&self) -> ToggleItemCommand;
}

impl ToggleItemCommandExt for SharedState {
    fn toggle_item_command(&self) -> ToggleItemCommand {
        ToggleItemCommand::new(self.commander.clone())
    }
}
