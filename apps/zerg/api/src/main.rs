use zerg_api::{config as app_config, migrator::Migrator, routes, state};

use app_config::Config;
use config::tracing::init_tracing;
use eyre::{Result, WrapErr};
use services::{postgres, redis};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Load configuration from environment variables
    let config = Config::from_env()?;

    // Initialize tracing with environment-aware configuration
    init_tracing(&config.environment);

    // Connect to a database and Redis
    let (postgres_pool, redis_manager) = tokio::try_join!(
        async {
            postgres::connect(&config.database.url)
                .await
                .wrap_err("Failed to connect to Postgres")
        },
        async {
            redis::connect(&config.redis.host)
                .await
                .wrap_err("Failed to connect to Redis")
        }
    )?;

    // Run migrations
    postgres::run_migrations::<Migrator>(&postgres_pool, "zerg_api").await?;

    // Create an application state
    let app_state = state::AppState::new(postgres_pool, redis_manager);

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

    let listener = tokio::net::TcpListener::bind(&address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
