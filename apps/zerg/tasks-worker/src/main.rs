//! Tasks Worker Service - Entry Point
//!
//! Background worker that processes task commands from the Redis stream.

#[tokio::main]
async fn main() -> eyre::Result<()> {
    zerg_tasks_worker::run().await
}
