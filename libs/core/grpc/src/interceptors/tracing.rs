use tonic::{Request, Status};

/// Interceptor for distributed tracing with W3C Trace Context propagation
///
/// Injects W3C trace context headers (traceparent, tracestate) for correlation across services.
/// This enables distributed tracing with OpenTelemetry-compatible backends.
///
/// Headers injected:
/// - `traceparent`: W3C Trace Context format (version-trace_id-span_id-flags)
/// - `tracestate`: Optional vendor-specific trace data
/// - `x-request-id`: UUID for request correlation (fallback)
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

  /// Generate a W3C traceparent header value
  ///
  /// Format: {version}-{trace_id}-{parent_id}-{trace_flags}
  /// Example: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
  fn generate_traceparent() -> String {
    let trace_id = uuid::Uuid::new_v4().as_simple().to_string();
    let span_id = &uuid::Uuid::new_v4().as_simple().to_string()[..16];
    format!("00-{trace_id}-{span_id}-01")
  }
}

impl tonic::service::Interceptor for TracingInterceptor {
  fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
    // Generate a unique request ID for this call
    let request_id = uuid::Uuid::new_v4().to_string();

    // Generate W3C traceparent header
    let traceparent = Self::generate_traceparent();

    // Inject traceparent header for W3C Trace Context
    request.metadata_mut().insert(
      "traceparent",
      traceparent
        .parse()
        .map_err(|_| Status::internal("Failed to create traceparent header"))?,
    );

    // Inject request ID into metadata (for backward compatibility)
    request.metadata_mut().insert(
      "x-request-id",
      request_id
        .parse()
        .map_err(|_| Status::internal("Failed to create request ID"))?,
    );

    // Log the outgoing request with trace context
    tracing::debug!(
            target: "grpc_client",
            request_id = %request_id,
            traceparent = %traceparent,
            "Outgoing gRPC request with trace context"
        );

    Ok(request)
  }
}

/// Interceptor that extracts trace context from incoming requests
///
/// Use this on the server side to extract trace context from incoming gRPC requests
/// and continue the trace.
#[derive(Clone, Debug, Default)]
pub struct TraceContextExtractor;

impl TraceContextExtractor {
  /// Create a new trace context extractor
  pub fn new() -> Self {
    Self
  }

  /// Extract trace ID from traceparent header
  pub fn extract_trace_id(traceparent: &str) -> Option<String> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() >= 2 {
      Some(parts[1].to_string())
    } else {
      None
    }
  }

  /// Extract span ID from traceparent header
  pub fn extract_span_id(traceparent: &str) -> Option<String> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() >= 3 {
      Some(parts[2].to_string())
    } else {
      None
    }
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

    // Verify x-request-id header
    let request_id = req.metadata().get("x-request-id");
    assert!(request_id.is_some());
    let id_str = request_id.unwrap().to_str().unwrap();
    assert!(uuid::Uuid::parse_str(id_str).is_ok());

    // Verify traceparent header
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

  #[test]
  fn test_generate_traceparent_format() {
    let traceparent = TracingInterceptor::generate_traceparent();
    let parts: Vec<&str> = traceparent.split('-').collect();

    assert_eq!(parts.len(), 4, "traceparent should have 4 parts");
    assert_eq!(parts[0], "00", "version should be 00");
    assert_eq!(parts[1].len(), 32, "trace_id should be 32 hex chars");
    assert_eq!(parts[2].len(), 16, "span_id should be 16 hex chars");
    assert_eq!(parts[3], "01", "flags should be 01");
  }

  #[test]
  fn test_extract_trace_id() {
    let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
    let trace_id = TraceContextExtractor::extract_trace_id(traceparent);
    assert_eq!(
      trace_id,
      Some("0af7651916cd43dd8448eb211c80319c".to_string())
    );
  }

  #[test]
  fn test_extract_span_id() {
    let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
    let span_id = TraceContextExtractor::extract_span_id(traceparent);
    assert_eq!(span_id, Some("b7ad6b7169203331".to_string()));
  }

  #[test]
  fn test_extract_from_invalid_traceparent() {
    let invalid = "invalid";
    assert_eq!(TraceContextExtractor::extract_trace_id(invalid), None);
    assert_eq!(TraceContextExtractor::extract_span_id(invalid), None);
  }
}
