/// Database connection cleanup utilities.
///
/// This module provides helpers for properly closing database connections
/// during graceful shutdown.

use tracing::{error, info};

/// Cleanup handler for PostgreSQL connections (SeaORM).
///
/// SeaORM's `DatabaseConnection` closes automatically on drop, but
/// we can explicitly close it to ensure proper cleanup logging.
///
/// # Example
/// ```ignore
/// use axum_helpers::cleanup::close_postgres;
/// use database::postgres::DatabaseConnection;
///
/// close_postgres(db, "main").await;
/// ```
pub async fn close_postgres(db: sea_orm::DatabaseConnection, name: &str) {
    match db.close().await {
        Ok(_) => info!("PostgreSQL connection '{}' closed successfully", name),
        Err(e) => error!("Error closing PostgreSQL connection '{}': {}", name, e),
    }
}

/// Cleanup handler for Redis connections.
///
/// Closes the connection manager gracefully.
///
/// Note: ConnectionManager doesn't expose a quit() method directly.
/// The underlying connection is closed when the ConnectionManager is dropped,
/// but we log the operation for observability.
///
/// # Example
/// ```ignore
/// use axum_helpers::cleanup::close_redis;
/// use redis::aio::ConnectionManager;
///
/// close_redis(redis, "main").await;
/// ```
#[cfg(feature = "redis")]
pub async fn close_redis(redis: redis::aio::ConnectionManager, name: &str) {
    // ConnectionManager closes automatically on drop
    // Just log that we're releasing it
    drop(redis);
    info!("Redis connection '{}' closed successfully", name);
}

/// Generic cleanup coordinator for multiple database connections.
///
/// Runs all cleanup tasks concurrently and waits for all to complete.
///
/// # Example
/// ```ignore
/// use axum_helpers::cleanup::CleanupCoordinator;
///
/// let mut cleanup = CleanupCoordinator::new();
/// cleanup.add_task("postgres", async { close_postgres(db, "main").await });
/// cleanup.add_task("redis", async { close_redis(redis, "main").await });
/// cleanup.run().await;
/// ```
pub struct CleanupCoordinator {
    tasks: Vec<(&'static str, tokio::task::JoinHandle<()>)>,
}

impl CleanupCoordinator {
    /// Create a new cleanup coordinator.
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Add a cleanup task with a name.
    ///
    /// The task will be spawned immediately and tracked for completion.
    pub fn add_task<F>(&mut self, name: &'static str, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(task);
        self.tasks.push((name, handle));
    }

    /// Run all cleanup tasks and wait for completion.
    ///
    /// Tasks are run concurrently. If any task panics or fails,
    /// it's logged but doesn't stop other tasks.
    pub async fn run(self) {
        info!("Running {} cleanup tasks", self.tasks.len());

        for (name, handle) in self.tasks {
            match handle.await {
                Ok(_) => {
                    info!("Cleanup task '{}' completed successfully", name);
                }
                Err(e) => {
                    error!("Cleanup task '{}' failed: {}", name, e);
                }
            }
        }

        info!("All cleanup tasks completed");
    }
}

impl Default for CleanupCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
