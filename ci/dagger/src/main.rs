use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use tracing::info;

mod pipelines;
mod utils;

pub use utils::config::CiConfig;

#[derive(Parser)]
#[command(name = "ci")]
#[command(about = "Dagger CI pipelines for nx-playground monorepo")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Run in CI mode (enables affected detection)
    #[arg(long, env = "CI", global = true)]
    ci: bool,

    /// GCS bucket for sccache
    #[arg(long, env = "SCCACHE_GCS_BUCKET", global = true)]
    sccache_bucket: Option<String>,

    /// Base SHA for affected detection
    #[arg(long, env = "NX_BASE", global = true)]
    base: Option<String>,

    /// Head SHA for affected detection
    #[arg(long, env = "NX_HEAD", global = true)]
    head: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all CI pipelines (lint, test, build)
    All,
    /// Run lint checks (cargo fmt --check, clippy)
    Lint,
    /// Run tests (cargo nextest)
    Test,
    /// Build workspace
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build, scan, and optionally push container images
    Container {
        /// Apps to containerize (comma-separated, "all", or "affected")
        #[arg(long, default_value = "affected")]
        apps: String,
        /// Push images to registries
        #[arg(long)]
        push: bool,
        /// Skip Trivy security scan
        #[arg(long)]
        skip_scan: bool,
    },
    /// Run NX targets for JavaScript/TypeScript projects
    Nx {
        /// NX target to run (lint, build, test, etc.)
        #[arg(long, default_value = "build")]
        target: String,
        /// Projects to run (comma-separated, "all", or "affected")
        #[arg(long, default_value = "affected")]
        projects: String,
        /// Additional arguments to pass to NX
        #[arg(last = true)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ci=info".parse()?)
                .add_directive("dagger_sdk=warn".parse()?),
        )
        .init();

    let cli = Cli::parse();

    info!("Starting Dagger CI pipeline");
    info!(
        "Mode: {}",
        if cli.ci {
            "CI (affected)"
        } else {
            "Local (all)"
        }
    );

    dagger_sdk::connect(move |client| async move {
        let config = CiConfig::new(cli.ci, cli.sccache_bucket, cli.base, cli.head);

        match cli.command {
            Commands::All => {
                pipelines::lint::run(&client, &config).await?;
                pipelines::test::run(&client, &config).await?;
                pipelines::build::run(&client, &config, false).await?;
                Ok(())
            }
            Commands::Lint => pipelines::lint::run(&client, &config).await,
            Commands::Test => pipelines::test::run(&client, &config).await,
            Commands::Build { release } => pipelines::build::run(&client, &config, release).await,
            Commands::Container {
                apps,
                push,
                skip_scan,
            } => pipelines::container::run(&client, &config, &apps, push, skip_scan).await,
            Commands::Nx {
                target,
                projects,
                args,
            } => pipelines::nx::run(&client, &config, &target, &projects, &args).await,
        }
    })
    .await?;

    info!("CI pipeline completed successfully");
    Ok(())
}
