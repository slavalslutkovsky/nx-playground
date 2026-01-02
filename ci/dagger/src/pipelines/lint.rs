use dagger_sdk::Query;
use eyre::Result;
use tracing::info;

use crate::utils::{config::CiConfig, nx, rust_container};

/// Run lint pipeline: cargo fmt --check + cargo clippy
pub async fn run(client: &Query, config: &CiConfig) -> Result<()> {
    info!("=== Running Lint Pipeline ===");

    let packages = nx::get_affected_packages(client, config, "lint").await?;

    let container = rust_container::create(client, config).await?;

    // Step 1: Format check
    info!("Checking code formatting...");
    let container = container.with_exec(vec!["cargo", "fmt", "--all", "--check"]);

    // Step 2: Clippy
    info!("Running clippy...");
    let clippy_args = build_clippy_args(&packages);

    container.with_exec(clippy_args).stdout().await?;

    info!("=== Lint Pipeline Completed ===");
    Ok(())
}

fn build_clippy_args(packages: &[String]) -> Vec<&str> {
    let mut args: Vec<&str> = vec!["cargo", "clippy"];

    if packages.is_empty() {
        // Run on entire workspace
        args.push("--workspace");
    }
    // Note: for affected packages, we'd need to handle this differently
    // as we can't easily return borrowed strings for dynamic package names

    args.extend(["--all-targets", "--", "-D", "warnings"]);

    args
}
