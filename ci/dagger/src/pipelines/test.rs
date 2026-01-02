use dagger_sdk::Query;
use eyre::Result;
use tracing::info;

use crate::utils::{config::CiConfig, nx, rust_container};

/// Run test pipeline: cargo nextest run
pub async fn run(client: &Query, config: &CiConfig) -> Result<()> {
    info!("=== Running Test Pipeline ===");

    let _packages = nx::get_affected_packages(client, config, "test").await?;

    let container = rust_container::create(client, config).await?;

    // Run tests with nextest on entire workspace
    // (affected detection logged but workspace run for simplicity)
    info!("Running tests with cargo nextest...");

    container
        .with_exec(vec!["cargo", "nextest", "run", "--workspace"])
        .stdout()
        .await?;

    info!("=== Test Pipeline Completed ===");
    Ok(())
}
