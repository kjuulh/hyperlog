use itertools::Itertools;

use crate::{
    commander::{self, Commander},
    models::IOEvent,
    state::SharedState,
};

pub struct CreateSectionCommand {
    commander: Commander,
}

impl CreateSectionCommand {
    pub fn new(commander: Commander) -> Self {
        Self { commander }
    }

    pub fn command(self, root: &str, path: &[&str]) -> super::Command {
        let root = root.to_owned();
        let path = path.iter().map(|s| s.to_string()).collect_vec();

        super::Command::new(|dispatch| {
            tokio::spawn(async move {
                dispatch.send(crate::models::Msg::SectionCreated(IOEvent::Initialized));

                match self
                    .commander
                    .execute(commander::Command::CreateSection { root, path })
                    .await
                {
                    Ok(()) => {
                        #[cfg(debug_assertions)]
                        {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }

                        dispatch.send(crate::models::Msg::SectionCreated(IOEvent::Success(())));
                    }
                    Err(e) => {
                        dispatch.send(crate::models::Msg::SectionCreated(IOEvent::Failure(
                            e.to_string(),
                        )));
                    }
                }
            });
            None
        })
    }
}

pub trait CreateSectionCommandExt {
    fn create_section_command(&self) -> CreateSectionCommand;
}

impl CreateSectionCommandExt for SharedState {
    fn create_section_command(&self) -> CreateSectionCommand {
        CreateSectionCommand::new(self.commander.clone())
    }
}
