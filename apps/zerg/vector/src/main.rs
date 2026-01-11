//! Vector gRPC Service - Entry Point
//!
//! Minimal entry point that delegates to the server module.

#[tokio::main]
async fn main() -> eyre::Result<()> {
    zerg_vector::run().await
}
