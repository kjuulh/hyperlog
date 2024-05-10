use std::net::SocketAddr;

use clap::{Parser, Subcommand};
use hyperlog_core::{commander, state};

use crate::server::serve;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Command {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(env = "SERVICE_HOST", long, default_value = "127.0.0.1:3000")]
        host: SocketAddr,
    },
    Exec {
        #[command(subcommand)]
        commands: ExecCommands,
    },
    Query {
        #[command(subcommand)]
        commands: QueryCommands,
    },
    Info {},

    CreateRoot {
        #[arg(long)]
        name: String,
    },

    ClearLock {},
}

#[derive(Subcommand)]
enum ExecCommands {
    CreateRoot {
        #[arg(long = "root")]
        root: String,
    },

    CreateSection {
        #[arg(long = "root")]
        root: String,

        #[arg(long = "path")]
        path: Option<String>,
    },
}

#[derive(Subcommand)]
enum QueryCommands {
    Get {
        #[arg(long = "root")]
        root: String,

        #[arg(long = "path")]
        path: Option<String>,
    },
}

pub async fn execute() -> anyhow::Result<()> {
    let cli = Command::parse();

    if cli.command.is_some() {
        tracing_subscriber::fmt::init();
    }

    let state = state::State::new()?;

    match cli.command {
        Some(Commands::Serve { host }) => {
            tracing::info!("Starting service");

            serve(host).await?;
        }
        Some(Commands::Exec { commands }) => match commands {
            ExecCommands::CreateRoot { root } => state
                .commander
                .execute(commander::Command::CreateRoot { root })?,
            ExecCommands::CreateSection { root, path } => {
                state.commander.execute(commander::Command::CreateSection {
                    root,
                    path: path
                        .unwrap_or_default()
                        .split('.')
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>(),
                })?
            }
        },
        Some(Commands::Query { commands }) => match commands {
            QueryCommands::Get { root, path } => {
                let res = state.querier.get(
                    &root,
                    path.unwrap_or_default()
                        .split('.')
                        .filter(|s| !s.is_empty()),
                );

                let output = serde_json::to_string_pretty(&res)?;

                println!("{}", output);
            }
        },
        Some(Commands::CreateRoot { name }) => {
            state
                .commander
                .execute(commander::Command::CreateRoot { root: name })?;
            println!("Root was successfully created, now run:\n\n$ hyperlog");
        }
        Some(Commands::Info {}) => {
            println!("graph stored at: {}", state.storage.info()?)
        }
        Some(Commands::ClearLock {}) => {
            state.storage.clear_lock_file();
            println!("cleared lock file");
        }
        None => {
            hyperlog_tui::execute(state).await?;
        }
    }

    Ok(())
}
