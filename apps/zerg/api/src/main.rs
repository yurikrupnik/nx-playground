// Import library modules
use zerg_api::{config as app_config, db, routes, state};

use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use app_config::{Config, Environment};
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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    // Load configuration from environment variables
    let config = Config::from_env()?;

    // Initialize tracing with environment-aware configuration
    init_tracing(&config.environment);

    tracing::info!("Starting Zerg API server...");

    // Connect to database
    let db = db::connect(&config.database.url).await?;
    tracing::info!("Connected to database");

    // Run migrations
    db::run_migrations(&db).await?;

    // Create an application state
    let app_state = state::AppState::new(db);

    // Build router with middleware
    let app = routes::create_router()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start server
    let address = config.server.address();
    tracing::info!("Listening on {}", address);

    let listener = tokio::net::TcpListener::bind(&address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
