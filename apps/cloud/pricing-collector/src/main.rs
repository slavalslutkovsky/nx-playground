//! Pricing Collector
//!
//! A service that collects cloud pricing data from AWS, Azure, and GCP.
//! Can run as a one-shot collection or as a scheduled cron job.

use clap::{Parser, Subcommand};
use core_config::tracing::{init_tracing, install_color_eyre};
use core_config::Environment;
use eyre::Result;
use tracing::info;

mod collector;
mod config;
mod providers;

use collector::PriceCollector;
use config::Config;

#[derive(Parser)]
#[command(name = "pricing-collector")]
#[command(about = "Collect cloud pricing data from AWS, Azure, and GCP")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a one-time collection
    Collect {
        /// Cloud providers to collect from (aws, azure, gcp). Defaults to all.
        #[arg(short, long, value_delimiter = ',')]
        providers: Option<Vec<String>>,

        /// Resource types to collect (compute, storage, database, etc.). Defaults to all.
        #[arg(short, long, value_delimiter = ',')]
        resource_types: Option<Vec<String>>,

        /// Regions to collect. Defaults to configured regions.
        #[arg(short = 'R', long, value_delimiter = ',')]
        regions: Option<Vec<String>>,

        /// Force collection even if recent data exists
        #[arg(short, long)]
        force: bool,
    },

    /// Run as a scheduled service
    Schedule {
        /// Cron expression for scheduling (default: every 6 hours)
        #[arg(short, long, default_value = "0 0 */6 * * *")]
        cron: String,
    },

    /// Show collection status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    install_color_eyre();

    let config = Config::from_env()?;
    let environment = Environment::from_env();
    init_tracing(&environment);

    // Initialize metrics
    observability::init_metrics();

    let cli = Cli::parse();

    // Connect to database
    info!("Connecting to database...");
    let db = database::postgres::connect_from_config_with_retry(config.database.clone(), None)
        .await
        .map_err(|e| eyre::eyre!("Database connection failed: {}", e))?;

    // Create collector
    let collector = PriceCollector::new(db, config.clone());

    match cli.command {
        Commands::Collect {
            providers,
            resource_types,
            regions,
            force,
        } => {
            info!("Starting one-time price collection");

            let result = collector
                .collect(
                    providers.as_deref(),
                    resource_types.as_deref(),
                    regions.as_deref(),
                    force,
                )
                .await?;

            info!(
                "Collection complete: {} prices collected, {} updated, {} errors",
                result.prices_collected, result.prices_updated, result.errors
            );
        }

        Commands::Schedule { cron } => {
            info!("Starting scheduled collection with cron: {}", cron);
            collector.run_scheduled(&cron).await?;
        }

        Commands::Status => {
            let status = collector.get_status().await?;
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
    }

    Ok(())
}
