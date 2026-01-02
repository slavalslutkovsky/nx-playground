use dagger_sdk::Query;
use eyre::Result;
use tracing::{info, warn};

use super::config::CiConfig;

/// Get list of affected packages for a given Nx target
pub async fn get_affected_packages(
    client: &Query,
    config: &CiConfig,
    target: &str,
) -> Result<Vec<String>> {
    if !config.ci_mode {
        info!("Not in CI mode, returning empty (will process all packages)");
        return Ok(vec![]);
    }

    info!(
        "Getting affected packages for target '{}' (base: {}, head: {})",
        target, config.base_sha, config.head_sha
    );

    let source = client.host().directory(".");

    // Create a Node container with Bun to run Nx
    let output = client
        .container()
        .from("oven/bun:1")
        .with_mounted_directory("/app", source)
        .with_workdir("/app")
        .with_exec(vec!["bun", "install", "--frozen-lockfile"])
        .with_exec(vec![
            "bun",
            "nx",
            "show",
            "projects",
            "--affected",
            &format!("--base={}", config.base_sha),
            &format!("--head={}", config.head_sha),
            &format!("--target={}", target),
        ])
        .stdout()
        .await;

    match output {
        Ok(stdout) => {
            let packages: Vec<String> = stdout
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('>'))
                .map(|s| s.trim().to_string())
                .collect();

            info!("Found {} affected packages: {:?}", packages.len(), packages);
            Ok(packages)
        }
        Err(e) => {
            warn!("Failed to get affected packages: {}. Processing all.", e);
            Ok(vec![])
        }
    }
}

/// Convert Nx project names to Cargo package names
/// (Nx uses kebab-case, Cargo uses snake_case)
#[allow(dead_code)]
pub fn nx_to_cargo_package(nx_name: &str) -> String {
    nx_name.replace('-', "_")
}
