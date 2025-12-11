use tonic::{Request, Status};
use std::sync::atomic::{AtomicU64, Ordering};

/// Simple metrics interceptor for counting requests
///
/// In a production system, this would integrate with Prometheus or similar
/// metrics collection systems to track request counts, latencies, and error rates.
///
/// # Example
/// ```ignore
/// use grpc_client::interceptors::MetricsInterceptor;
/// use std::sync::Arc;
///
/// let metrics = Arc::new(MetricsInterceptor::new());
/// let channel = create_channel("http://[::1]:50051").await?;
/// let client = TasksServiceClient::with_interceptor(channel, metrics.clone());
///
/// // Later, check metrics
/// println!("Total requests: {}", metrics.total_requests());
/// ```
#[derive(Debug, Default)]
pub struct MetricsInterceptor {
    total_requests: AtomicU64,
}

impl MetricsInterceptor {
    /// Create a new metrics interceptor
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the total number of requests processed
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Reset the request counter (useful for testing)
    #[cfg(test)]
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
    }
}

impl Clone for MetricsInterceptor {
    fn clone(&self) -> Self {
        Self {
            total_requests: AtomicU64::new(self.total_requests.load(Ordering::Relaxed)),
        }
    }
}

impl tonic::service::Interceptor for MetricsInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        tracing::trace!(
            target: "grpc_client",
            count = self.total_requests.load(Ordering::Relaxed),
            "gRPC request metrics"
        );

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::service::Interceptor;

    #[test]
    fn test_metrics_counting() {
        let mut metrics = MetricsInterceptor::new();
        assert_eq!(metrics.total_requests(), 0);

        let request = Request::new(());
        let _ = metrics.call(request);
        assert_eq!(metrics.total_requests(), 1);

        let request = Request::new(());
        let _ = metrics.call(request);
        assert_eq!(metrics.total_requests(), 2);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = MetricsInterceptor::new();
        let mut m = metrics.clone();
        let _ = m.call(Request::new(()));
        assert_eq!(m.total_requests(), 1);

        m.reset();
        assert_eq!(m.total_requests(), 0);
    }
}
