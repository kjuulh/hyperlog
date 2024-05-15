use crate::{
    models::{IOEvent, Msg},
    querier::Querier,
    state::SharedState,
};

pub struct OpenUpdateItemDialogCommand {
    querier: Querier,
}

impl OpenUpdateItemDialogCommand {
    pub fn new(querier: Querier) -> Self {
        Self { querier }
    }

    pub fn command(self, root: &str, path: Vec<String>) -> super::Command {
        let root = root.to_string();

        super::Command::new(|dispatch| {
            tokio::spawn(async move {
                dispatch.send(Msg::OpenUpdateItemDialog(IOEvent::Initialized));

                let item = match self.querier.get_async(&root, path).await {
                    Ok(item) => match item {
                        Some(item) => {
                            dispatch.send(Msg::OpenUpdateItemDialog(IOEvent::Success(())));
                            item
                        }
                        None => {
                            dispatch.send(Msg::OpenUpdateItemDialog(IOEvent::Failure(
                                "failed to find a valid item for path".into(),
                            )));

                            return;
                        }
                    },
                    Err(e) => {
                        dispatch.send(Msg::OpenUpdateItemDialog(IOEvent::Failure(e.to_string())));

                        return;
                    }
                };

                dispatch.send(Msg::OpenEditItemDialog { item });
            });
            None
        })
    }
}

pub trait OpenUpdateItemDialogCommandExt {
    fn open_update_item_dialog_command(&self) -> OpenUpdateItemDialogCommand;
}

impl OpenUpdateItemDialogCommandExt for SharedState {
    fn open_update_item_dialog_command(&self) -> OpenUpdateItemDialogCommand {
        OpenUpdateItemDialogCommand::new(self.querier.clone())
    }
}
