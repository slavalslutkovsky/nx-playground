use tonic::{Request, Status};

/// Interceptor for distributed tracing
///
/// Injects trace context headers (x-request-id) for correlation across services.
/// In a production system, this would integrate with OpenTelemetry or similar
/// tracing systems to propagate trace IDs and span IDs.
///
/// # Example
/// ```ignore
/// use grpc_client::interceptors::TracingInterceptor;
/// use rpc::tasks::tasks_service_client::TasksServiceClient;
///
/// let tracing_interceptor = TracingInterceptor::new();
/// let channel = create_channel("http://[::1]:50051").await?;
/// let client = TasksServiceClient::with_interceptor(channel, tracing_interceptor);
/// ```
#[derive(Clone, Debug, Default)]
pub struct TracingInterceptor;

impl TracingInterceptor {
    /// Create a new tracing interceptor
    pub fn new() -> Self {
        Self
    }
}

impl tonic::service::Interceptor for TracingInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // Generate a unique request ID for this call
        let request_id = uuid::Uuid::new_v4().to_string();

        // Inject request ID into metadata
        request.metadata_mut().insert(
            "x-request-id",
            request_id
                .parse()
                .map_err(|_| Status::internal("Failed to create request ID"))?,
        );

        // Log the outgoing request
        // Note: request.uri() is not available for tonic::Request in interceptors
        tracing::debug!(
            target: "grpc_client",
            request_id = %request_id,
            "Outgoing gRPC request"
        );

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::service::Interceptor;

    #[test]
    fn test_tracing_interceptor() {
        let mut tracing = TracingInterceptor::new();
        let request = Request::new(());
        let result = tracing.call(request);
        assert!(result.is_ok());
        let req = result.unwrap();
        let request_id = req.metadata().get("x-request-id");
        assert!(request_id.is_some());
        // Verify it's a valid UUID
        let id_str = request_id.unwrap().to_str().unwrap();
        assert!(uuid::Uuid::parse_str(id_str).is_ok());
    }
}
