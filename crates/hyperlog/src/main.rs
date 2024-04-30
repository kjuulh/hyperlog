mod cli;
pub(crate) mod server;
pub(crate) mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    cli::execute().await?;

    Ok(())
}
