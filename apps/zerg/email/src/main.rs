use axum::{routing::get, Router};
use config::tracing::init_tracing;
use config::FromEnv;
use eyre::{Result, WrapErr};
use tower_http::trace::TraceLayer;
use zerg_email::{config::Config, handlers, state::AppState, worker};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Load configuration
    let config = Config::from_env()?;
    init_tracing(&config.environment);

    tracing::info!("Starting email worker service");

    // Connect to Redis
    tracing::info!(redis_host = %config.redis.host, "Connecting to Redis...");
    let redis_manager = services::redis::connect(&config.redis.host)
        .await
        .wrap_err("Failed to connect to Redis")?;

    // Create an SMTP provider
    let smtp_config = email::provider::smtp::SmtpConfig {
        host: config.email.smtp_host.clone(),
        port: config.email.smtp_port,
        username: config.email.smtp_username.clone(),
        password: config.email.smtp_password.clone(),
        from_email: config.email.from_email.clone(),
        from_name: config.email.from_name.clone(),
        use_tls: config.email.smtp_use_tls,
    };
    let provider = email::provider::SmtpProvider::new(smtp_config)
        .wrap_err("Failed to create SMTP provider")?;

    // Create app state
    let state = AppState::with_defaults(redis_manager, provider);

    // Start worker tasks
    let worker_handles: Vec<_> = (0..config.email.worker_count)
        .map(|worker_id| {
            let state = state.clone();
            let email_config = config.email.clone();
            tokio::spawn(async move {
                if let Err(e) = worker::run_worker(state, email_config, worker_id).await {
                    tracing::error!(worker_id = worker_id, error = %e, "Worker failed");
                }
            })
        })
        .collect();

    // Create a health check router
    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/stream/info", get(handlers::stream_info))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start a health check server
    let addr = config.server.address();
    tracing::info!("Health check server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Run server and workers concurrently
    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                tracing::error!(error = %e, "Server error");
            }
        }
        _ = futures::future::join_all(worker_handles) => {
            tracing::info!("All workers completed");
        }
    }

    Ok(())
}
