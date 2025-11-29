use crate::Environment;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

/// Initialize tracing with environment-aware configuration
///
/// - **Production** (`APP_ENV=production`):
///   - JSON format (for log aggregation tools like ELK, Datadog, CloudWatch)
///   - Hides module targets for cleaner logs
///
/// - **Development** (default):
///   - Pretty-printed format (human-readable)
///   - Shows module targets for debugging
///
/// Environment variables:
/// - `APP_ENV`: Set to "production" for JSON logs (default: "development")
/// - `RUST_LOG`: Override log levels (e.g., "debug", "zerg_api=trace")
///
/// This function is infallible - if tracing is already initialized, it silently continues.
pub fn init_tracing(environment: &Environment) {
    let is_production = environment.is_production();

    // Create a filter with granular defaults
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if is_production {
            // Production: Less verbose, focus on warnings and errors
            EnvFilter::new("info,tower_http=info,sea_orm=warn")
        } else {
            // Development: More verbose for debugging
            EnvFilter::new("debug,tower_http=debug,sea_orm=info")
        }
    });

    let result = if is_production {
        // Production: JSON format for log aggregation
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_target(false) // Hide module paths in production
            .try_init()
    } else {
        // Development: Pretty format for readability
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true) // Show module paths for debugging
            .pretty()
            .try_init()
    };

    // Handle initialization result
    match result {
        Ok(_) => {
            info!("Tracing initialized. Environment: {:?}", environment);
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
