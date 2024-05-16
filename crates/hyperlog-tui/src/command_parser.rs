use itertools::Itertools;

pub enum Commands {
    Write,
    Quit,
    WriteQuit,
    Archive,
    CreateSection { name: String },
    CreateItem { name: String },
    CreateBelow { name: String },
    Edit,

    ShowAll,
    HideDone,
    Test,
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
                "ci" | "create-item" => Some(Commands::CreateItem {
                    name: rest.join(" ").to_string(),
                }),
                "cb" | "create-below" => Some(Commands::CreateBelow {
                    name: rest.join(" ").to_string(),
                }),
                "e" | "edit" => Some(Commands::Edit),
                "show-all" => Some(Commands::ShowAll),
                "hide-done" => Some(Commands::HideDone),
                "test" => Some(Commands::Test),
                _ => None,
            },
            None => None,
        }
    }
}
