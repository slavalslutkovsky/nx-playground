//! Email Worker Service - Entry Point
//!
//! Background worker that processes email jobs from the Redis stream.

#[tokio::main]
async fn main() -> eyre::Result<()> {
    zerg_email_worker::run().await
}
