use super::IntoCommand;

#[derive(Default)]
pub struct BatchCommand {
    commands: Vec<super::Command>,
}

impl BatchCommand {
    pub fn with(&mut self, cmd: impl IntoCommand) -> &mut Self {
        self.commands.push(cmd.into_command());

        self
    }
}

impl IntoCommand for Vec<super::Command> {
    fn into_command(self) -> super::Command {
        BatchCommand::from(self).into_command()
    }
}

impl From<Vec<super::Command>> for BatchCommand {
    fn from(value: Vec<super::Command>) -> Self {
        BatchCommand { commands: value }
    }
}

impl IntoCommand for BatchCommand {
    fn into_command(self) -> super::Command {
        super::Command::new(|dispatch| {
            for command in self.commands {
                let msg = command.execute(dispatch.clone());
                if let Some(msg) = msg {
                    dispatch.send(msg);
                }
            }

            None
        })
    }
}
