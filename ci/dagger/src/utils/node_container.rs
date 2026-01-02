use dagger_sdk::{Container, Query};
use eyre::Result;
use tracing::info;

const BUN_IMAGE: &str = "oven/bun:1";
const WORKDIR: &str = "/app";

/// Create a Bun container with NX and all dependencies installed
pub async fn create(client: &Query) -> Result<Container> {
    info!("Creating Bun container for NX");

    let source = client.host().directory_opts(
        ".",
        dagger_sdk::HostDirectoryOpts {
            exclude: Some(vec!["target", "dist/target", ".git"]),
            include: None,
            gitignore: None,
            no_cache: None,
        },
    );

    let container = client
        .container()
        .from(BUN_IMAGE)
        // Mount Dagger cache volumes for Bun/Node
        .with_mounted_cache("/root/.bun/install/cache", client.cache_volume("bun-cache"))
        .with_mounted_cache(
            format!("{}/node_modules", WORKDIR),
            client.cache_volume("node-modules"),
        )
        // Mount source code
        .with_mounted_directory(WORKDIR, source)
        .with_workdir(WORKDIR)
        // Install dependencies
        .with_exec(vec!["bun", "install", "--frozen-lockfile"]);

    Ok(container)
}

/// Create a Bun container with dependencies already installed (for running commands)
#[allow(dead_code)]
pub async fn create_with_deps(client: &Query) -> Result<Container> {
    let container = create(client).await?;
    // Sync to ensure dependencies are installed before returning
    container.sync().await?;
    Ok(create(client).await?)
}
