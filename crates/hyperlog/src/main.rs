#![feature(map_try_insert)]
use std::{net::SocketAddr, ops::Deref, sync::Arc};

use anyhow::Context;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use clap::{Parser, Subcommand};
use sqlx::{Pool, Postgres};
use tower_http::trace::TraceLayer;

pub mod commander;
pub mod querier;

pub mod engine;
pub mod events;
pub mod log;
pub mod shared_engine;
pub mod state;
pub mod storage;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cli = Command::parse();

    let state = state::State::new()?;

    match cli.command {
        Some(Commands::Serve { host }) => {
            tracing::info!("Starting service");

            let state = SharedState(Arc::new(State::new().await?));

            let app = Router::new()
                .route("/", get(root))
                .with_state(state.clone())
                .layer(
                    TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                        // Log the matched route's path (with placeholders not filled in).
                        // Use request.uri() or OriginalUri if you want the real path.
                        let matched_path = request
                            .extensions()
                            .get::<MatchedPath>()
                            .map(MatchedPath::as_str);

                        tracing::info_span!(
                            "http_request",
                            method = ?request.method(),
                            matched_path,
                            some_other_field = tracing::field::Empty,
                        )
                    }), // ...
                );

            tracing::info!("listening on {}", host);
            let listener = tokio::net::TcpListener::bind(host).await.unwrap();
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
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

async fn root() -> &'static str {
    "Hello, hyperlog!"
}

#[derive(Clone)]
pub struct SharedState(Arc<State>);

impl Deref for SharedState {
    type Target = Arc<State>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct State {
    pub db: Pool<Postgres>,
}

impl State {
    pub async fn new() -> anyhow::Result<Self> {
        let db = sqlx::PgPool::connect(
            &std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?,
        )
        .await?;

        sqlx::migrate!("migrations/crdb")
            .set_locking(false)
            .run(&db)
            .await?;

        let _ = sqlx::query("SELECT 1;").fetch_one(&db).await?;

        Ok(Self { db })
    }
}
