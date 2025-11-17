use crate::Environment;
use tracing::info;
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

    if is_production {
        // Production: JSON format for log aggregation
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_target(false) // Hide module paths in production
            .init();
    } else {
        // Development: Pretty format for readability
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true) // Show module paths for debugging
            .pretty()
            .init();
    }

    info!("Tracing initialized. Environment: {:?}", environment);
}
