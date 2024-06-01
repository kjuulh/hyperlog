use hyperlog_core::log::ItemState;
use itertools::Itertools;

use crate::{
    commander::{self, Commander},
    models::{IOEvent, Msg},
    state::SharedState,
};

pub struct ArchiveCommand {
    commander: Commander,
}

impl ArchiveCommand {
    pub fn new(commander: Commander) -> Self {
        Self { commander }
    }

    pub fn command(self, root: &str, path: &[&str]) -> super::Command {
        let root = root.to_owned();
        let path = path.iter().map(|s| s.to_string()).collect_vec();

        super::Command::new(|dispatch| {
            tokio::spawn(async move {
                dispatch.send(Msg::Archive(IOEvent::Initialized));

                match self
                    .commander
                    .execute(commander::Command::Archive { root, path })
                    .await
                {
                    Ok(()) => {
                        #[cfg(debug_assertions)]
                        {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }

                        dispatch.send(Msg::Archive(IOEvent::Success(())));
                    }
                    Err(e) => {
                        dispatch.send(Msg::Archive(IOEvent::Failure(e.to_string())));
                    }
                }
            });
            None
        })
    }
}

pub trait ArchiveCommandExt {
    fn archive_command(&self) -> ArchiveCommand;
}

impl ArchiveCommandExt for SharedState {
    fn archive_command(&self) -> ArchiveCommand {
        ArchiveCommand::new(self.commander.clone())
    }
}
