//! Email Worker Service Entry Point

use core_config::tracing::install_color_eyre;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Install color-eyre first for colored error output
    install_color_eyre();

    // Run the email worker
    zerg_email::run().await
}
