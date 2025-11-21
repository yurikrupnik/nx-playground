use app::app::{create_app, create_router};
use app_config::Config;
use config::tracing::init_tracing;
use eyre::{Result, WrapErr};
use services::{postgres, redis};
use zerg_api::{api, config as app_config, migrator::Migrator, openapi::ApiDoc, state};

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
    postgres::run_migrations::<Migrator>(&postgres_pool, env!("CARGO_PKG_NAME")).await?;

    // Create an application state
    let app_state = state::AppState::new(postgres_pool, redis_manager);

    // Create API routes
    let api_routes = api::routes();

    // Build router with middleware using generic create_router
    let app = create_router::<ApiDoc, state::AppState>(app_state, api_routes).await?;

    // Start server using generic create_app
    create_app(app, &config.server).await?;

    Ok(())
}
