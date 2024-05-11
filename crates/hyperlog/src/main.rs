mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    cli::execute().await?;

    Ok(())
}
