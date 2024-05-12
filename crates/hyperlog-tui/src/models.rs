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
    OpenEditItemDialog { item: GraphItem },
    Interact,

    EnterInsertMode,
    EnterViewMode,
    EnterCommandMode,

    SubmitCommand { command: String },

    Edit(EditMsg),
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
