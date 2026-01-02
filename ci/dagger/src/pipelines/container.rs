use dagger_sdk::{Container, Query};
use eyre::Result;
use tracing::{info, warn};

use crate::utils::{config::CiConfig, nx, trivy};

/// Known Rust apps that can be containerized
const RUST_APPS: &[(&str, &str)] = &[
    ("zerg-api", "zerg_api"),
    ("zerg-tasks", "zerg_tasks"),
    ("products-api", "products_api"),
    ("zerg-mongo-api", "zerg_mongo_api"),
];

const DOCKERFILE: &str = "manifests/dockers/rust.Dockerfile";

/// Run container pipeline: build, scan, and optionally push
pub async fn run(
    client: &Query,
    config: &CiConfig,
    apps: &str,
    push: bool,
    skip_scan: bool,
) -> Result<()> {
    info!("=== Running Container Pipeline ===");
    info!("Apps: {}, Push: {}, Skip scan: {}", apps, push, skip_scan);

    let apps_to_build = resolve_apps(client, config, apps).await?;

    if apps_to_build.is_empty() {
        info!("No apps to containerize");
        return Ok(());
    }

    info!("Building containers for: {:?}", apps_to_build);

    for (app_name, cargo_name) in &apps_to_build {
        info!("--- Processing {} ---", app_name);

        // Build container image
        let image = build_image(client, cargo_name).await?;

        // Scan with Trivy
        if !skip_scan {
            let scan_result = trivy::scan_image(client, &image, app_name).await?;
            if scan_result.has_critical {
                warn!(
                    "Container {} has {} critical vulnerabilities!",
                    app_name, scan_result.vulnerabilities_count
                );
            }
        }

        // Push to registries
        if push {
            push_image(client, &image, app_name).await?;
        }
    }

    info!("=== Container Pipeline Completed ===");
    Ok(())
}

async fn resolve_apps(
    client: &Query,
    config: &CiConfig,
    apps: &str,
) -> Result<Vec<(String, String)>> {
    match apps {
        "all" => Ok(RUST_APPS
            .iter()
            .map(|(name, cargo)| (name.to_string(), cargo.to_string()))
            .collect()),
        "affected" => {
            let affected = nx::get_affected_packages(client, config, "container").await?;
            Ok(RUST_APPS
                .iter()
                .filter(|(name, _)| affected.iter().any(|a| a == *name))
                .map(|(name, cargo)| (name.to_string(), cargo.to_string()))
                .collect())
        }
        _ => {
            // Comma-separated list
            let requested: Vec<&str> = apps.split(',').map(|s| s.trim()).collect();
            Ok(RUST_APPS
                .iter()
                .filter(|(name, _)| requested.contains(name))
                .map(|(name, cargo)| (name.to_string(), cargo.to_string()))
                .collect())
        }
    }
}

async fn build_image(client: &Query, cargo_name: &str) -> Result<Container> {
    info!("Building Docker image for {}", cargo_name);

    let source = client.host().directory_opts(
        ".",
        dagger_sdk::HostDirectoryOpts {
            exclude: Some(vec!["target", "dist/target", "node_modules", ".git"]),
            include: None,
            gitignore: None,
            no_cache: None,
        },
    );

    // Build using the Rust Dockerfile
    // Note: Since the Dagger Rust SDK has limited build options,
    // we use a shell command approach to invoke docker build
    let image = client
        .container()
        .from("docker:dind")
        .with_mounted_directory("/workspace", source)
        .with_workdir("/workspace")
        .with_exec(vec![
            "docker",
            "build",
            "-f",
            DOCKERFILE,
            "--build-arg",
            &format!("APP_NAME={}", cargo_name),
            "--target",
            "rust",
            "-t",
            &format!("{}:latest", cargo_name),
            ".",
        ]);

    info!("Image built successfully for {}", cargo_name);
    Ok(image)
}

async fn push_image(client: &Query, image: &Container, app_name: &str) -> Result<()> {
    let short_sha = std::env::var("GITHUB_SHA")
        .map(|s| s.chars().take(7).collect::<String>())
        .unwrap_or_else(|_| "latest".to_string());

    // Push to Docker Hub
    if let (Ok(username), Ok(token)) = (
        std::env::var("DOCKERHUB_USERNAME"),
        std::env::var("DOCKERHUB_TOKEN"),
    ) {
        info!("Pushing {} to Docker Hub", app_name);

        let secret = client.set_secret("dockerhub_token", &token);
        let authed = image.with_registry_auth("docker.io", &username, secret);

        let tag = format!("docker.io/{}/{}:sha-{}", username, app_name, short_sha);
        info!("Pushing to {}", tag);
        authed.publish(&tag).await?;

        let latest_tag = format!("docker.io/{}/{}:latest", username, app_name);
        info!("Pushing to {}", latest_tag);
        authed.publish(&latest_tag).await?;
    }

    // Push to GitHub Container Registry
    if let (Ok(repo), Ok(token)) = (
        std::env::var("GITHUB_REPOSITORY"),
        std::env::var("GITHUB_TOKEN"),
    ) {
        info!("Pushing {} to GHCR", app_name);

        let secret = client.set_secret("ghcr_token", &token);
        let actor = std::env::var("GITHUB_ACTOR").unwrap_or_else(|_| "token".to_string());

        let authed = image.with_registry_auth("ghcr.io", &actor, secret);

        let tag = format!("ghcr.io/{}/{}:sha-{}", repo, app_name, short_sha);
        info!("Pushing to {}", tag);
        authed.publish(&tag).await?;

        let latest_tag = format!("ghcr.io/{}/{}:latest", repo, app_name);
        info!("Pushing to {}", latest_tag);
        authed.publish(&latest_tag).await?;
    }

    Ok(())
}
