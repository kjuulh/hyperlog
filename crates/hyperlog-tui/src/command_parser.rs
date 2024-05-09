use itertools::Itertools;

pub enum Commands {
    Write,
    Quit,
    WriteQuit,
    Archive,
    CreateSection { name: String },
}

impl Commands {
    pub fn is_write(&self) -> bool {
        matches!(self, Commands::Write | Commands::WriteQuit)
    }

    pub fn is_quit(&self) -> bool {
        matches!(self, Commands::Quit | Commands::WriteQuit)
    }
}

pub struct CommandParser {}

impl CommandParser {
    pub fn parse(raw_command: &str) -> Option<Commands> {
        let prepared = raw_command.trim();
        // TODO: respect quotes
        let parts = prepared.split_whitespace().collect_vec();

        match parts.split_first() {
            Some((command, rest)) => match *command {
                "w" | "write" => Some(Commands::Write),
                "q" | "quit" => Some(Commands::Quit),
                "wq" | "write-quit" => Some(Commands::WriteQuit),
                "a" | "archive" => Some(Commands::Archive),
                "cs" | "create-section" => rest.first().map(|name| Commands::CreateSection {
                    name: name.to_string(),
                }),
                _ => None,
            },
            None => None,
        }
    }
}
