use crate::Environment;
use tracing::{debug, info};
use tracing_subscriber::{prelude::*, EnvFilter};

/// Install color-eyre with a project-standard configuration.
///
/// Call this early in the main() before any fallible operations to ensure
/// colored error output. Safe to call multiple times.
///
/// Configuration:
/// - Shows file:line where errors occur
/// - Hides environment variables (less noise)
pub fn install_color_eyre() {
    let _ = color_eyre::config::HookBuilder::default()
        .display_location_section(true)
        .display_env_section(false)
        .install();
}

/// Initialize tracing with environment-aware configuration and error span capture.
///
/// This function sets up:
/// 1. **color-eyre** for beautiful error display with span traces
/// 2. **tracing-subscriber** with ErrorLayer for span capture
///
/// When errors occur, you'll see the full execution path with structured data
/// from your instrumented spans.
///
/// - **Production** (`APP_ENV=production`):
///   - JSON format (for log aggregation tools like ELK, Datadog, CloudWatch)
///   - Hides module targets for cleaner logs
///   - Includes ErrorLayer for span trace capture
///
/// - **Development** (default):
///   - Pretty-printed format (human-readable)
///   - Shows module targets for debugging
///   - Includes ErrorLayer for span trace capture
///   - Shows file locations in errors
///
/// Environment variables:
/// - `APP_ENV`: Set to "production" for JSON logs (default: "development")
/// - `RUST_LOG`: Override log levels (e.g., "debug", "zerg_api=trace")
///
/// # Multiple Calls
///
/// This function is safe to call multiple times:
/// - If color-eyre is already installed, silently continues (common in tests)
/// - If tracing is already initialized, silently continues (common in tests)
///
/// # Example with instrumentation
///
/// ```ignore
/// use tracing::instrument;
/// use eyre::{Result, WrapErr};
///
/// #[instrument(skip(pool), fields(user_id = %user_id))]
/// async fn get_user(pool: &PgPool, user_id: Uuid) -> Result<User> {
///     sqlx::query_as!(/*...*/)
///         .fetch_one(pool)
///         .await
///         .wrap_err("Failed to fetch user")
/// }
/// ```
pub fn init_tracing(environment: &Environment) {
    // Install color-eyre FIRST - it must be set up before ErrorLayer
    // This allows ErrorLayer to capture span traces when errors occur
    // Silently ignore if already installed (common in tests or if called early in the main)
    // install_color_eyre();

    let is_production = environment.is_production();

    // Create a filter with granular defaults
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if is_production {
            // Production: Less verbose, focus on warnings and errors
            EnvFilter::new("error")
            // EnvFilter::new("info,tower_http=info,sea_orm=warn")
        } else {
            // Development: More verbose for debugging
            EnvFilter::new("trace")
        }
    });
    tracing::warn!("filter:?");
    let result = if is_production {
        // Production: JSON format for log aggregation
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(false) // Hide module paths in production
                    .flatten_event(true),
            )
            .with(tracing_error::ErrorLayer::default()) // Capture span traces on errors
            .with(filter)
            .try_init()
    } else {
        // Development: Pretty format for readability
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(false) // Show module paths for debugging
                    .with_file(false)
                    .with_line_number(false)
                    .pretty(),
            )
            .with(tracing_error::ErrorLayer::default()) // Capture span traces on errors
            .with(filter)
            .try_init()
    };

    // Handle initialization result
    match result {
        Ok(_) => {
            info!(
                "Tracing initialized with ErrorLayer. Environment: {:?}",
                environment
            );
        }
        Err(_) => {
            // Tracing already initialized, which is fine (common in tests)
            debug!("Tracing already initialized, skipping re-initialization");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing_development() {
        let env = Environment::Development;
        // Should not panic
        init_tracing(&env);
    }

    #[test]
    fn test_init_tracing_production() {
        let env = Environment::Production;
        // Should not panic
        init_tracing(&env);
    }

    #[test]
    fn test_init_tracing_multiple_calls() {
        // Should not panic when called multiple times
        let env = Environment::Development;
        init_tracing(&env);
        init_tracing(&env);
    }

    #[test]
    fn test_init_tracing_with_rust_log_env() {
        temp_env::with_var("RUST_LOG", Some("trace"), || {
            let env = Environment::Development;
            // Should not panic
            init_tracing(&env);
        });
    }

    #[test]
    fn test_init_tracing_production_with_custom_log_level() {
        temp_env::with_var("RUST_LOG", Some("warn"), || {
            let env = Environment::Production;
            // Should not panic
            init_tracing(&env);
        });
    }
}
