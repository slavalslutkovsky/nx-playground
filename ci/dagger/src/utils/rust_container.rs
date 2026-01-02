use dagger_sdk::{Container, Query};
use eyre::Result;
use tracing::info;

use super::config::CiConfig;
use super::sccache;

const RUST_IMAGE: &str = "rust:1.83-bookworm";
const WORKDIR: &str = "/workspace";

/// Create a base Rust container with all required tooling
pub async fn create(client: &Query, config: &CiConfig) -> Result<Container> {
    info!("Creating Rust container with tooling");

    let source = client.host().directory_opts(
        ".",
        dagger_sdk::HostDirectoryOpts {
            exclude: Some(vec!["target", "dist/target", "node_modules", ".git"]),
            include: None,
            gitignore: None,
            no_cache: None,
        },
    );

    let mut container = client
        .container()
        .from(RUST_IMAGE)
        // Install system dependencies
        .with_exec(vec!["apt-get", "update"])
        .with_exec(vec![
            "apt-get",
            "install",
            "-y",
            "musl-dev",
            "musl-tools",
            "openssl",
            "libssl-dev",
            "pkg-config",
            "protobuf-compiler",
        ])
        // Install Rust components
        .with_exec(vec!["rustup", "component", "add", "clippy", "rustfmt"])
        // Install cargo tools
        .with_exec(vec!["cargo", "install", "--locked", "cargo-nextest"]);

    // Configure sccache if bucket is provided
    if config.has_sccache() {
        container = sccache::configure(client, container, config).await?;
    }

    // Mount Dagger cache volumes for cargo
    container = container
        .with_mounted_cache(
            format!("{}/target", WORKDIR),
            client.cache_volume("cargo-target"),
        )
        .with_mounted_cache(
            "/usr/local/cargo/registry",
            client.cache_volume("cargo-registry"),
        )
        .with_mounted_cache("/usr/local/cargo/git/db", client.cache_volume("cargo-git"));

    // Mount source code
    container = container
        .with_mounted_directory(WORKDIR, source)
        .with_workdir(WORKDIR);

    Ok(container)
}
