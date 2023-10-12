#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "cli")]
    nioca_client::cli::execute().await?;
    Ok(())
}
