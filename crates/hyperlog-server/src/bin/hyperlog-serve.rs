//! Minimal server-only entrypoint (bypasses the TUI crate). Runs the gRPC +
//! http stack via hyperlog_server::serve. Config via env:
//!   EXTERNAL_GRPC_HOST (default 127.0.0.1:4000)
//!   EXTERNAL_HOST      (default 127.0.0.1:3000)
//!   INTERNAL_HOST      (default 127.0.0.1:3001)
//!   DATABASE_URL, HYPERLOG_JWT_SECRET
use std::net::SocketAddr;

fn env_addr(key: &str, default: &str) -> anyhow::Result<SocketAddr> {
    Ok(std::env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .parse()?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("starting hyperlog-serve");
    hyperlog_server::serve(hyperlog_server::ServeOptions {
        external_http: env_addr("EXTERNAL_HOST", "127.0.0.1:3000")?,
        internal_http: env_addr("INTERNAL_HOST", "127.0.0.1:3001")?,
        external_grpc: env_addr("EXTERNAL_GRPC_HOST", "127.0.0.1:4000")?,
    })
    .await
}
