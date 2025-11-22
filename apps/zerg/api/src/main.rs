use app::app::{create_app, create_router};
use app_config::Config;
use config::tracing::init_tracing;
use eyre::{Result, WrapErr};
use mongodb::Client;
use services::{postgres, redis};
use sqlx::PgPool;
use zerg_api::{api, config as app_config, migrator::Migrator, openapi::ApiDoc, state};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Load configuration from environment variables
    let config = Config::from_env()?;

    // Initialize tracing with environment-aware configuration
    init_tracing(&config.environment);

    // Connect to PostgreSQL, MongoDB, and Redis
    let (postgres_pool, mongo_db, redis_manager, sqlx_pool) = tokio::try_join!(
        async {
            postgres::connect(&config.database.url)
                .await
                .wrap_err("Failed to connect to Postgres")
        },
        async {
            let mongo_uri = std::env::var("MONGODB_URI")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
            let mongo_client = Client::with_uri_str(&mongo_uri)
                .await
                .wrap_err("Failed to connect to MongoDB")?;
            let db_name = std::env::var("MONGODB_DB_NAME")
                .unwrap_or_else(|_| "zerg".to_string());
            Ok::<_, eyre::Report>(mongo_client.database(&db_name))
        },
        async {
            redis::connect(&config.redis.host)
                .await
                .wrap_err("Failed to connect to Redis")
        },
        async {
            PgPool::connect(&config.database.url)
                .await
                .wrap_err("Failed to connect to Postgres via sqlx")
        }
    )?;

    // Run migrations
    postgres::run_migrations::<Migrator>(&postgres_pool, env!("CARGO_PKG_NAME")).await?;

    // Initialize bikes table
    apis_bike::controller::init_bikes_table(&sqlx_pool)
        .await
        .wrap_err("Failed to initialize bikes table")?;

    // Create an application state
    let app_state = state::AppState::new(postgres_pool, mongo_db, redis_manager, sqlx_pool);

    // Create API routes
    let api_routes = api::routes();

    // Build a router with middleware using generic create_router
    let app = create_router::<ApiDoc, state::AppState>(app_state, api_routes).await?;

    // Start a server using generic create_app
    create_app(app, &config.server).await?;

    Ok(())
}
