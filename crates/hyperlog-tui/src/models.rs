use hyperlog_core::log::GraphItem;

use crate::commands::{Command, IntoCommand};

#[derive(Debug)]
pub enum Msg {
    MoveRight,
    MoveLeft,
    MoveDown,
    MoveUp,
    QuitApp,
    OpenCreateItemDialog,
    OpenCreateItemDialogBelow,
    OpenEditItemDialog { item: GraphItem },
    Interact,

    EnterInsertMode,
    EnterViewMode,
    EnterCommandMode,

    SubmitCommand { command: String },

    Edit(EditMsg),

    GraphUpdated(IOEvent<GraphItem>),
    ItemCreated(IOEvent<()>),
    ItemUpdated(IOEvent<()>),
    SectionCreated(IOEvent<()>),
    ItemToggled(IOEvent<()>),

    OpenUpdateItemDialog(IOEvent<()>),
}

#[derive(Debug)]
pub enum IOEvent<T> {
    Initialized,
    Optimistic(T),
    Success(T),
    Failure(String),
}

impl IntoCommand for Msg {
    fn into_command(self) -> crate::commands::Command {
        Command::new(|_| Some(self))
    }
}

#[derive(Debug)]
pub enum EditMsg {
    Delete,
    InsertNewLine,
    InsertTab,
    DeleteNext,
    InsertChar(char),
    MoveLeft,
    MoveRight,
}
