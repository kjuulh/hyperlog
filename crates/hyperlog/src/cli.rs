use clap::{Parser, Subcommand, ValueEnum};
use hyperlog_tui::{
    commander,
    core_state::{Backend, State},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Command {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, default_value = "local")]
    backend: BackendArg,

    #[arg(long = "backend-url", required_if_eq("backend", "remote"))]
    backend_url: Option<String>,
}

#[derive(ValueEnum, Clone)]
enum BackendArg {
    Local,
    Remote,
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "include_server")]
    Serve {
        #[arg(env = "EXTERNAL_HOST", long, default_value = "127.0.0.1:3000")]
        external_host: std::net::SocketAddr,
        #[arg(env = "INTERNAL_HOST", long, default_value = "127.0.0.1:3001")]
        internal_host: std::net::SocketAddr,
        #[arg(env = "EXTERNAL_GRPC_HOST", long, default_value = "127.0.0.1:4000")]
        external_grpc_host: std::net::SocketAddr,
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

    let backend = cli.backend;
    let backend_url = cli.backend_url;

    let backend = match backend {
        BackendArg::Local => Backend::Local,
        BackendArg::Remote => Backend::Remote {
            url: backend_url.expect("backend-url to be set"),
        },
    };

    match cli.command {
        #[cfg(feature = "include_server")]
        Some(Commands::Serve {
            external_host,
            internal_host,
            external_grpc_host,
        }) => {
            tracing::info!("Starting service");

            hyperlog_server::serve(hyperlog_server::ServeOptions {
                external_http: external_host,
                internal_http: internal_host,
                external_grpc: external_grpc_host,
            })
            .await?;
        }
        Some(Commands::Exec { commands }) => {
            let state = State::new(backend).await?;
            match commands {
                ExecCommands::CreateRoot { root } => {
                    state
                        .commander
                        .execute(commander::Command::CreateRoot { root })
                        .await?
                }
                ExecCommands::CreateSection { root, path } => {
                    state
                        .commander
                        .execute(commander::Command::CreateSection {
                            root,
                            path: path
                                .unwrap_or_default()
                                .split('.')
                                .map(|s| s.to_string())
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<String>>(),
                        })
                        .await?
                }
            }
        }
        Some(Commands::Query { commands }) => {
            let state = State::new(backend).await?;
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
            let state = State::new(backend).await?;
            state
                .commander
                .execute(commander::Command::CreateRoot { root: name })
                .await?;
            println!("Root was successfully created, now run:\n\n$ hyperlog");
        }
        Some(Commands::Info {}) => {
            let state = State::new(backend).await?;
            println!("graph stored at: {}", state.storage.info()?)
        }
        Some(Commands::ClearLock {}) => {
            let state = State::new(backend).await?;
            state.storage.clear_lock_file();
            println!("cleared lock file");
        }
        None => {
            let state = State::new(backend).await?;
            hyperlog_tui::execute(state).await?;
        }
    }

    Ok(())
}
