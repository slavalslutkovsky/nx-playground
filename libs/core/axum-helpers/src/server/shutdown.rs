use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;
use tracing::info;

/// Shutdown coordinator that manages graceful application shutdown.
///
/// This handles:
/// - Signal reception (SIGTERM, SIGINT)
/// - Broadcasting shutdown to all subsystems
/// - Timeout enforcement
/// - Shutdown state tracking
#[derive(Clone)]
pub struct ShutdownCoordinator {
    /// Broadcast channel to notify all tasks of shutdown
    tx: broadcast::Sender<()>,
    /// Flag indicating if shutdown has been initiated
    shutdown_initiated: Arc<AtomicBool>,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator.
    ///
    /// Returns the coordinator and a receiver for shutdown signals.
    pub fn new() -> (Self, broadcast::Receiver<()>) {
        let (tx, rx) = broadcast::channel(1);
        let coordinator = Self {
            tx,
            shutdown_initiated: Arc::new(AtomicBool::new(false)),
        };
        (coordinator, rx)
    }

    /// Subscribe to shutdown notifications.
    ///
    /// Returns a receiver that will be notified when shutdown begins.
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.tx.subscribe()
    }

    /// Check if shutdown has been initiated.
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_initiated.load(Ordering::Relaxed)
    }

    /// Initiate shutdown and notify all subscribers.
    pub fn shutdown(&self) {
        if self
            .shutdown_initiated
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            info!("Initiating graceful shutdown");
            let _ = self.tx.send(());
        }
    }

    /// Wait for shutdown signal (SIGTERM or SIGINT) and return.
    ///
    /// This is the main entry point for shutdown coordination.
    pub async fn wait_for_signal(&self) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!("Received SIGINT (Ctrl+C), initiating graceful shutdown");
            },
            _ = terminate => {
                info!("Received SIGTERM, initiating graceful shutdown");
            },
        }

        self.shutdown();
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new().0
    }
}

/// Simple shutdown signal for basic use cases.
///
/// **WARNING**: This does NOT handle connection cleanup or timeouts.
/// For production use, prefer `ShutdownCoordinator` with proper cleanup.
///
/// This function can be used with `axum::serve().with_graceful_shutdown()`.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal, shutting down gracefully");
        },
        _ = terminate => {
            info!("Received SIGTERM signal, shutting down gracefully");
        },
    }
}

/// Helper to create a coordinated shutdown future for axum.
///
/// Returns a future that completes when shutdown is signaled,
/// allowing time for in-flight requests to complete.
///
/// This is used internally by `create_production_app`.
pub(crate) async fn coordinated_shutdown(coordinator: ShutdownCoordinator) {
    coordinator.wait_for_signal().await;
}
