use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub mod batch;

pub mod create_item;
pub mod create_section;
pub mod toggle_item;
pub mod update_graph;
pub mod update_item;

use crate::models::Msg;

pub trait IntoCommand {
    fn into_command(self) -> Command;
}

impl IntoCommand for () {
    fn into_command(self) -> Command {
        Command::new(|_| None)
    }
}

impl IntoCommand for Command {
    fn into_command(self) -> Command {
        self
    }
}

type CommandFunc = dyn FnOnce(Dispatch) -> Option<Msg>;

pub struct Command {
    func: Box<CommandFunc>,
}

impl Command {
    pub fn new<T: FnOnce(Dispatch) -> Option<Msg> + 'static>(f: T) -> Self {
        Self { func: Box::new(f) }
    }

    pub fn execute(self, dispatch: Dispatch) -> Option<Msg> {
        self.func.call_once((dispatch,))
    }
}

pub fn create_dispatch() -> (Dispatch, Receiver) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    (Dispatch { sender: tx }, Receiver { receiver: rx })
}

#[derive(Clone)]
pub struct Dispatch {
    sender: UnboundedSender<Msg>,
}

impl Dispatch {
    pub fn send(&self, msg: Msg) {
        if let Err(e) = self.sender.send(msg) {
            tracing::warn!("failed to send event: {}", e);
        }
    }
}

pub struct Receiver {
    receiver: UnboundedReceiver<Msg>,
}

impl Receiver {
    pub async fn next(&mut self) -> Option<Msg> {
        self.receiver.recv().await
    }
}
