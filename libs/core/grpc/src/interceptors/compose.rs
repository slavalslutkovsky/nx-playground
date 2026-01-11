use tonic::{Request, Status};

/// Compose two interceptors into a single interceptor
///
/// Interceptors are called in order: first, then second. This allows you
/// to chain multiple interceptors together.
///
/// # Example
/// ```ignore
/// use grpc_client::interceptors::{AuthInterceptor, TracingInterceptor, compose_interceptors};
///
/// let auth = AuthInterceptor::bearer("token");
/// let tracing = TracingInterceptor::new();
/// let composed = compose_interceptors(auth, tracing);
///
/// let client = TasksServiceClient::with_interceptor(channel, composed);
/// ```
pub fn compose_interceptors<A, B>(first: A, second: B) -> ComposedInterceptor<A, B>
where
  A: tonic::service::Interceptor,
  B: tonic::service::Interceptor,
{
  ComposedInterceptor { first, second }
}

/// A composed interceptor that applies two interceptors in sequence
///
/// This is the return type of `compose_interceptors()`. You typically
/// don't need to construct this directly.
#[derive(Clone, Debug)]
pub struct ComposedInterceptor<A, B> {
  first: A,
  second: B,
}

impl<A, B> tonic::service::Interceptor for ComposedInterceptor<A, B>
where
  A: tonic::service::Interceptor,
  B: tonic::service::Interceptor,
{
  fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
    let request = self.first.call(request)?;
    self.second.call(request)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::interceptors::{AuthInterceptor, TracingInterceptor};
  use tonic::service::Interceptor;

  #[test]
  fn test_compose_interceptors() {
    let auth = AuthInterceptor::bearer("test-token");
    let tracing = TracingInterceptor::new();
    let mut composed = compose_interceptors(auth, tracing);

    let request = Request::new(());
    let result = composed.call(request);
    assert!(result.is_ok());

    let req = result.unwrap();
    // Both interceptors should have added their headers
    assert!(req.metadata().get("authorization").is_some());
    assert!(req.metadata().get("x-request-id").is_some());
  }

  #[test]
  fn test_compose_three_interceptors() {
    use crate::interceptors::MetricsInterceptor;

    let auth = AuthInterceptor::bearer("token");
    let tracing = TracingInterceptor::new();
    let metrics = MetricsInterceptor::new();

    // Compose all three: auth -> (tracing -> metrics)
    let mut composed = compose_interceptors(auth, compose_interceptors(tracing, metrics));

    let request = Request::new(());
    let result = composed.call(request);
    assert!(result.is_ok());

    let req = result.unwrap();
    assert!(req.metadata().get("authorization").is_some());
    assert!(req.metadata().get("x-request-id").is_some());
  }
}
