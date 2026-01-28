//! Client-side tracing interceptor with W3C Trace Context propagation
//!
//! Injects trace context headers into outgoing gRPC requests for distributed tracing.

use tonic::{Request, Status};

/// Interceptor for distributed tracing with W3C Trace Context propagation
///
/// When the `opentelemetry` feature is enabled, this interceptor extracts
/// the current trace context from the active span and propagates it to
/// the downstream service. This enables proper distributed tracing.
///
/// When `opentelemetry` is disabled, it generates a new trace ID for
/// request correlation (legacy behavior).
///
/// Headers injected:
/// - `traceparent`: W3C Trace Context format (version-trace_id-span_id-flags)
/// - `tracestate`: Optional vendor-specific trace data (when using OTel)
/// - `x-request-id`: UUID for request correlation (fallback)
///
/// # Example
/// ```ignore
/// use grpc_client::interceptors::client::ClientTracingInterceptor;
/// use rpc::tasks::tasks_service_client::TasksServiceClient;
///
/// let tracing = ClientTracingInterceptor::new();
/// let channel = create_channel("http://[::1]:50051").await?;
/// let client = TasksServiceClient::with_interceptor(channel, tracing);
/// ```
#[derive(Clone, Debug, Default)]
pub struct ClientTracingInterceptor {
    /// Service name to include in spans
    service_name: Option<String>,
}

impl ClientTracingInterceptor {
    /// Create a new tracing interceptor
    pub fn new() -> Self {
        Self { service_name: None }
    }

    /// Create a tracing interceptor with a service name
    pub fn with_service_name(service_name: impl Into<String>) -> Self {
        Self {
            service_name: Some(service_name.into()),
        }
    }

    /// Generate a W3C traceparent header value
    ///
    /// Format: {version}-{trace_id}-{parent_id}-{trace_flags}
    /// Example: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
    #[cfg(not(feature = "opentelemetry"))]
    fn generate_traceparent() -> String {
        let trace_id = uuid::Uuid::new_v4().as_simple().to_string();
        let span_id = &uuid::Uuid::new_v4().as_simple().to_string()[..16];
        format!("00-{trace_id}-{span_id}-01")
    }

    /// Inject trace context into request metadata
    #[cfg(feature = "opentelemetry")]
    fn inject_trace_context(request: &mut Request<()>) -> Result<(), Status> {
        use opentelemetry::propagation::TextMapPropagator;
        use opentelemetry_sdk::propagation::TraceContextPropagator;
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        // Get the current span's context
        let cx = tracing::Span::current().context();

        // Create a propagator
        let propagator = TraceContextPropagator::new();

        // Inject into metadata
        let mut injector = MetadataInjector(request.metadata_mut());
        propagator.inject_context(&cx, &mut injector);

        // Also inject x-request-id for backwards compatibility
        let request_id = uuid::Uuid::new_v4().to_string();
        request.metadata_mut().insert(
            "x-request-id",
            request_id
                .parse()
                .map_err(|_| Status::internal("Failed to create request ID"))?,
        );

        tracing::debug!(
            target: "grpc_client",
            request_id = %request_id,
            "Injected OpenTelemetry trace context"
        );

        Ok(())
    }

    /// Fallback trace context injection when OpenTelemetry is not enabled
    #[cfg(not(feature = "opentelemetry"))]
    fn inject_trace_context(request: &mut Request<()>) -> Result<(), Status> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let traceparent = Self::generate_traceparent();

        request.metadata_mut().insert(
            "traceparent",
            traceparent
                .parse()
                .map_err(|_| Status::internal("Failed to create traceparent header"))?,
        );

        request.metadata_mut().insert(
            "x-request-id",
            request_id
                .parse()
                .map_err(|_| Status::internal("Failed to create request ID"))?,
        );

        tracing::debug!(
            target: "grpc_client",
            request_id = %request_id,
            traceparent = %traceparent,
            "Outgoing gRPC request with trace context (OTel disabled)"
        );

        Ok(())
    }
}

impl tonic::service::Interceptor for ClientTracingInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // Inject service name if configured
        if let Some(ref name) = self.service_name {
            if let Ok(value) = name.parse() {
                request.metadata_mut().insert("x-source-service", value);
            }
        }

        Self::inject_trace_context(&mut request)?;
        Ok(request)
    }
}

/// Helper for injecting trace context into gRPC metadata
#[cfg(feature = "opentelemetry")]
struct MetadataInjector<'a>(&'a mut tonic::metadata::MetadataMap);

#[cfg(feature = "opentelemetry")]
impl opentelemetry::propagation::Injector for MetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = key.parse() {
            if let Ok(value) = value.parse() {
                self.0.insert(key, value);
            }
        }
    }
}

/// Helper for extracting trace context from gRPC metadata
pub struct MetadataExtractor<'a>(pub &'a tonic::metadata::MetadataMap);

impl MetadataExtractor<'_> {
    /// Extract trace ID from traceparent header
    pub fn trace_id(&self) -> Option<String> {
        self.0
            .get("traceparent")
            .and_then(|v| v.to_str().ok())
            .and_then(|tp| {
                let parts: Vec<&str> = tp.split('-').collect();
                if parts.len() >= 2 {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            })
    }

    /// Extract span ID from traceparent header
    pub fn span_id(&self) -> Option<String> {
        self.0
            .get("traceparent")
            .and_then(|v| v.to_str().ok())
            .and_then(|tp| {
                let parts: Vec<&str> = tp.split('-').collect();
                if parts.len() >= 3 {
                    Some(parts[2].to_string())
                } else {
                    None
                }
            })
    }

    /// Extract request ID
    pub fn request_id(&self) -> Option<String> {
        self.0
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::service::Interceptor;

    #[test]
    fn test_tracing_interceptor() {
        let mut tracing = ClientTracingInterceptor::new();
        let request = Request::new(());
        let result = tracing.call(request);
        assert!(result.is_ok());
        let req = result.unwrap();

        // Verify x-request-id header
        let request_id = req.metadata().get("x-request-id");
        assert!(request_id.is_some());
        let id_str = request_id.unwrap().to_str().unwrap();
        assert!(uuid::Uuid::parse_str(id_str).is_ok());

        // Verify traceparent header (only when OTel is disabled)
        #[cfg(not(feature = "opentelemetry"))]
        {
            let traceparent = req.metadata().get("traceparent");
            assert!(traceparent.is_some());
            let tp_str = traceparent.unwrap().to_str().unwrap();
            let parts: Vec<&str> = tp_str.split('-').collect();
            assert_eq!(parts.len(), 4);
            assert_eq!(parts[0], "00"); // version
            assert_eq!(parts[1].len(), 32); // trace_id (32 hex chars)
            assert_eq!(parts[2].len(), 16); // span_id (16 hex chars)
            assert_eq!(parts[3], "01"); // flags
        }
    }

    #[test]
    fn test_with_service_name() {
        let mut tracing = ClientTracingInterceptor::with_service_name("my-service");
        let request = Request::new(());
        let result = tracing.call(request);
        assert!(result.is_ok());
        let req = result.unwrap();

        let source = req.metadata().get("x-source-service");
        assert!(source.is_some());
        assert_eq!(source.unwrap().to_str().unwrap(), "my-service");
    }

    #[test]
    fn test_metadata_extractor() {
        let mut metadata = tonic::metadata::MetadataMap::new();
        metadata.insert(
            "traceparent",
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
                .parse()
                .unwrap(),
        );
        metadata.insert("x-request-id", "test-request-id".parse().unwrap());

        let extractor = MetadataExtractor(&metadata);
        assert_eq!(
            extractor.trace_id(),
            Some("0af7651916cd43dd8448eb211c80319c".to_string())
        );
        assert_eq!(extractor.span_id(), Some("b7ad6b7169203331".to_string()));
        assert_eq!(
            extractor.request_id(),
            Some("test-request-id".to_string())
        );
    }

    #[cfg(not(feature = "opentelemetry"))]
    #[test]
    fn test_generate_traceparent_format() {
        let traceparent = ClientTracingInterceptor::generate_traceparent();
        let parts: Vec<&str> = traceparent.split('-').collect();

        assert_eq!(parts.len(), 4, "traceparent should have 4 parts");
        assert_eq!(parts[0], "00", "version should be 00");
        assert_eq!(parts[1].len(), 32, "trace_id should be 32 hex chars");
        assert_eq!(parts[2].len(), 16, "span_id should be 16 hex chars");
        assert_eq!(parts[3], "01", "flags should be 01");
    }
}
