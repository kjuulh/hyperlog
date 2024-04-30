use std::{net::SocketAddr, ops::Deref, sync::Arc};

use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use clap::{Parser, Subcommand};
use hyperlog_core::{commander, state};
use tower_http::trace::TraceLayer;

use crate::{
    server::serve,
    state::{SharedState, State},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None, subcommand_required = true)]
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
        Some(Commands::Info {}) => {
            println!("graph stored at: {}", state.storage.info()?)
        }

        None => {}
    }

    Ok(())
}
