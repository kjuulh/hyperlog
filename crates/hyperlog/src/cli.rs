use std::net::SocketAddr;

use clap::{Parser, Subcommand};
use hyperlog_core::{commander, state};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Command {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "include_server")]
    Serve {
        #[arg(env = "EXTERNAL_HOST", long, default_value = "127.0.0.1:3000")]
        external_host: SocketAddr,
        #[arg(env = "INTERNAL_HOST", long, default_value = "127.0.0.1:3001")]
        internal_host: SocketAddr,
        #[arg(env = "EXTERNAL_GRPC_HOST", long, default_value = "127.0.0.1:4000")]
        external_grpc_host: SocketAddr,
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

    match cli.command {
        #[cfg(feature = "include_server")]
        Some(Commands::Serve {
            external_host,
            internal_host,
            ..
        }) => {
            tracing::info!("Starting service");

            hyperlog_server::serve(hyperlog_server::ServeOptions {
                external_http: external_host,
                internal_http: internal_host,
            })
            .await?;
        }
        Some(Commands::Exec { commands }) => {
            let state = state::State::new()?;
            match commands {
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
            }
        }
        Some(Commands::Query { commands }) => {
            let state = state::State::new()?;
            match commands {
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
            }
        }
        Some(Commands::CreateRoot { name }) => {
            let state = state::State::new()?;
            state
                .commander
                .execute(commander::Command::CreateRoot { root: name })?;
            println!("Root was successfully created, now run:\n\n$ hyperlog");
        }
        Some(Commands::Info {}) => {
            let state = state::State::new()?;
            println!("graph stored at: {}", state.storage.info()?)
        }
        Some(Commands::ClearLock {}) => {
            let state = state::State::new()?;
            state.storage.clear_lock_file();
            println!("cleared lock file");
        }
        None => {
            let state = state::State::new()?;
            hyperlog_tui::execute(state).await?;
        }
    }

    Ok(())
}
