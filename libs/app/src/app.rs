use crate::middleware::security::security_headers;
use crate::shutdown::shutdown_signal;
use axum::{
    http::{HeaderName, Method},
    middleware, Router,
};
use config::server::ServerConfig;
use eyre::Result;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable as RedocServable};
use utoipa_scalar::{Scalar, Servable as scalarServable};
use utoipa_swagger_ui::SwaggerUi;

/// Starts the Axum server with a graceful shutdown.
///
/// # Arguments
/// * `router` - The configured Axum router
/// * `server_config` - Server configuration with host and port
pub async fn create_app(router: Router, server_config: &ServerConfig) -> Result<()> {
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

pub async fn create_router<T, S: 'static + Clone + Send + Sync>(
    state: S,
    apis: Router<S>,
    // redis_manager: redis::aio::ConnectionManager,
) -> Result<Router>
where
    T: OpenApi + 'static,
{
    // let csrf = CsrfProtection::new(redis_manager);
    //
    // #[derive(Clone)]
    // struct CsrfState {
    //     csrf: CsrfProtection,
    // }

    // impl axum::extract::FromRef<CsrfState> for CsrfProtection {
    //     fn from_ref(state: &CsrfState) -> Self {
    //         state.csrf.clone()
    //     }
    // }
    //
    // let csrf_state = CsrfState { csrf: csrf.clone() };
    //
    // let csrf_route = Router::new()
    //     .route("/csrf-token", get(get_csrf_token))
    //     .with_state(csrf_state);

    // let cors_origin = Env::get_cors_allowed_origin()?
    //     .parse::<axum::http::HeaderValue>()?;

    // let cors = CorsLayer::new()
    //     .allow_origin(cors_origin)
    //     .allow_methods([
    //         Method::GET,
    //         Method::POST,
    //         Method::PUT,
    //         Method::DELETE,
    //         Method::PATCH,
    //         Method::OPTIONS,
    //     ])
    //     .allow_headers([
    //         axum::http::header::CONTENT_TYPE,
    //         axum::http::header::AUTHORIZATION,
    //         axum::http::header::ACCEPT,
    //         axum::http::header::COOKIE,
    //         HeaderName::from_static("x-csrf-token"),
    //     ])
    //     .allow_credentials(true)
    //     .max_age(Duration::from_secs(3600));

    // let protected_apis = apis.layer(middleware::from_fn_with_state(
    //     csrf.clone(),
    //     csrf_validation_middleware,
    // ));

    Ok(Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", T::openapi()))
        .merge(Redoc::with_url("/redoc", T::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .merge(Scalar::with_url("/scalar", T::openapi()))
        // .route("/health", get(
        // ))
        // .merge(csrf_route)
        .nest("/api", apis)
        // .nest("/api", protected_apis)
        .with_state(state)
        // .fallback(not_found)
        // .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(middleware::from_fn(security_headers)))
}
