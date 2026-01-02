use dagger_sdk::Query;
use eyre::Result;
use tracing::info;

use crate::utils::{config::CiConfig, nx, rust_container};

/// Run build pipeline: cargo build
pub async fn run(client: &Query, config: &CiConfig, release: bool) -> Result<()> {
    info!("=== Running Build Pipeline (release: {}) ===", release);

    let _packages = nx::get_affected_packages(client, config, "build").await?;

    let container = rust_container::create(client, config).await?;

    // Run build on the entire workspace
    info!("Building workspace...");

    let build_args = if release {
        vec!["cargo", "build", "--workspace", "--release"]
    } else {
        vec!["cargo", "build", "--workspace"]
    };

    container.with_exec(build_args).stdout().await?;

    info!("=== Build Pipeline Completed ===");
    Ok(())
}
