use crate::commands::{Command, IntoCommand};

#[derive(Debug)]
pub enum Msg {
    MoveRight,
    MoveLeft,
    MoveDown,
    MoveUp,
    OpenCreateItemDialog,
    Interact,

    EnterInsertMode,
    EnterViewMode,
    EnterCommandMode,

    SubmitCommand { command: String },

    Edit(EditMsg),
}

impl IntoCommand for Msg {
    fn into_command(self) -> crate::commands::Command {
        Command::new(|| Some(self))
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
