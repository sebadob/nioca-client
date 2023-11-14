mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::execute().await?;
    Ok(())
}
