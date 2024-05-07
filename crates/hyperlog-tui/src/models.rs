#[derive(Debug)]
pub enum Msg {
    MoveRight,
    MoveLeft,
    MoveDown,
    MoveUp,
    OpenCreateItemDialog,

    EnterInsertMode,
    EnterCommandMode,
    Edit(EditMsg),
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
