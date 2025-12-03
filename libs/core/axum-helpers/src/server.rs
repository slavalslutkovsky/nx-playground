use crate::errors::handlers::not_found;
use crate::middleware::security::security_headers;
use crate::shutdown::{coordinated_shutdown, shutdown_signal, ShutdownCoordinator};
use axum::{middleware, Router};
use core_config::server::ServerConfig;
use std::io;
use std::time::Duration;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};
use utoipa::OpenApi;

/// Starts the Axum server with graceful shutdown.
///
/// # Arguments
/// * `router` - The configured Axum router
/// * `server_config` - Server configuration with host and port
///
/// # Errors
/// Returns an error if:
/// - The TCP listener fails to bind to the configured address
/// - The server encounters an error during operation
///
/// # Example
/// ```ignore
/// use axum::Router;
/// use core_config::server::ServerConfig;
/// use axum_helpers::server::create_app;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let router = Router::new();
///     let config = ServerConfig::default();
///     create_app(router, &config).await?;
///     Ok(())
/// }
/// ```
pub async fn create_app(router: Router, server_config: &ServerConfig) -> io::Result<()> {
    let listener = tokio::net::TcpListener::bind(server_config.address()).await?;

    info!("Server starting on {}", listener.local_addr()?);
    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .inspect_err(|e| {
            tracing::error!("Server encountered an error: {:?}", e);
        })?;

    Ok(())
}

/// Creates a configured Axum router with common middleware and documentation.
///
/// This function sets up:
/// - OpenAPI documentation (Swagger UI, ReDoc, RapiDoc, Scalar)
/// - Health and readiness endpoints
/// - API routes nested under `/api`
/// - Common middleware (tracing, security headers)
/// - 404 fallback handler
///
/// Use this when your API routes already have state applied internally.
/// For clean architecture, domain routers should apply their own state,
/// and this function combines them with cross-cutting concerns.
///
/// # Type Parameters
/// * `T` - A type implementing `utoipa::OpenApi` for API documentation
///
/// # Arguments
/// * `apis` - Router with all routes (state already applied to individual routes)
///
/// # Example
/// ```ignore
/// use axum::Router;
/// use utoipa::OpenApi;
/// use axum_helpers::server::create_router;
///
/// #[derive(OpenApi)]
/// #[openapi(paths(/* your paths */))]
/// struct ApiDoc;
///
/// #[tokio::main]
/// async fn main() {
///     // Routes with state already applied
///     let api_routes = Router::new()
///         .route("/example", get(handler))
///         .with_state(my_state);
///
///     let router = create_router::<ApiDoc>(api_routes).await.unwrap();
/// }
/// ```
pub async fn create_router<T>(apis: Router) -> io::Result<Router>
where
    T: OpenApi + 'static,
{
    use utoipa_rapidoc::RapiDoc;
    use utoipa_redoc::{Redoc, Servable as RedocServable};
    use utoipa_scalar::{Scalar, Servable as ScalarServable};
    use utoipa_swagger_ui::SwaggerUi;
    use crate::health::health_handler;
    use axum::routing::get;

    let router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", T::openapi()))
        .merge(Redoc::with_url("/redoc", T::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .merge(Scalar::with_url("/scalar", T::openapi()))
        .route("/health", get(health_handler))
        .nest("/api", apis)
        .fallback(not_found)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(middleware::from_fn(security_headers));

    Ok(router)
}


/// Production-ready server with coordinated shutdown and cleanup.
///
/// This provides:
/// - Graceful shutdown with configurable timeout
/// - Connection cleanup coordination
/// - Proper error handling and logging
///
/// # Arguments
/// * `router` - The configured Axum router
/// * `server_config` - Server configuration
/// * `shutdown_timeout` - Maximum time to wait for graceful shutdown (recommended: 30s)
/// * `cleanup` - Async cleanup function for database connections, etc.
///
/// # Example
/// ```ignore
/// use std::time::Duration;
/// use axum_helpers::server::create_production_app;
///
/// let cleanup = async move {
///     // Close connections
///     db.close().await.ok();
/// };
///
/// create_production_app(
///     router,
///     &config,
///     Duration::from_secs(30),
///     cleanup
/// ).await?;
/// ```
pub async fn create_production_app<F>(
    router: Router,
    server_config: &ServerConfig,
    shutdown_timeout: Duration,
    cleanup: F,
) -> io::Result<()>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let (coordinator, _rx) = ShutdownCoordinator::new();
    let shutdown_handle = coordinator.clone();

    let listener = tokio::net::TcpListener::bind(server_config.address()).await?;
    info!("Server starting on {}", listener.local_addr()?);

    // Spawn cleanup task
    let cleanup_handle = tokio::spawn(async move {
        shutdown_handle.wait_for_signal().await;

        info!("Starting cleanup tasks (timeout: {:?})", shutdown_timeout);
        let cleanup_result =
            tokio::time::timeout(shutdown_timeout, cleanup).await;

        match cleanup_result {
            Ok(_) => info!("Cleanup completed successfully"),
            Err(_) => {
                tracing::warn!(
                    "Cleanup exceeded timeout of {:?}, forcing shutdown",
                    shutdown_timeout
                );
            }
        }
    });

    // Start server with graceful shutdown
    let serve_result = axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(coordinated_shutdown(coordinator))
        .await
        .inspect_err(|e| {
            tracing::error!("Server encountered an error: {:?}", e);
        });

    // Wait for cleanup to complete
    cleanup_handle.await.ok();

    serve_result
}
