use crate::models::Msg;

pub trait IntoCommand {
    fn into_command(self) -> Command;
}

impl IntoCommand for () {
    fn into_command(self) -> Command {
        Command::new(|| None)
    }
}

impl IntoCommand for Command {
    fn into_command(self) -> Command {
        self
    }
}

pub struct Command {
    func: Box<dyn FnOnce() -> Option<Msg>>,
}

impl Command {
    pub fn new<T: FnOnce() -> Option<Msg> + 'static>(f: T) -> Self {
        Self { func: Box::new(f) }
    }

    pub fn execute(self) -> Option<Msg> {
        self.func.call_once(())
    }
}
