use crate::{
    models::{IOEvent, Msg},
    querier::Querier,
    state::SharedState,
};

pub struct OpenItemCommand {
    querier: Querier,
}

impl OpenItemCommand {
    pub fn new(querier: Querier) -> Self {
        Self { querier }
    }

    pub fn command(self, root: &str, path: Vec<String>) -> super::Command {
        let root = root.to_string();

        super::Command::new(|dispatch| {
            tokio::spawn(async move {
                dispatch.send(Msg::OpenItem(IOEvent::Initialized));

                let item = match self.querier.get_async(&root, path).await {
                    Ok(item) => match item {
                        Some(item) => {
                            dispatch.send(Msg::OpenItem(IOEvent::Success(())));
                            item
                        }
                        None => {
                            dispatch.send(Msg::OpenItem(IOEvent::Failure(
                                "failed to find a valid item for path".into(),
                            )));

                            return;
                        }
                    },
                    Err(e) => {
                        dispatch.send(Msg::OpenItem(IOEvent::Failure(e.to_string())));

                        return;
                    }
                };

                dispatch.send(Msg::OpenEditor { item });
            });
            None
        })
    }
}

pub trait OpenItemCommandExt {
    fn open_item_command(&self) -> OpenItemCommand;
}

impl OpenItemCommandExt for SharedState {
    fn open_item_command(&self) -> OpenItemCommand {
        OpenItemCommand::new(self.querier.clone())
    }
}
