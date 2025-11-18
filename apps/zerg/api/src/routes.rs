use app::health::run_health_checks;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use sea_orm::DatabaseConnection;
use serde_json::Value;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    handlers::{authors, books, fields, todos, users},
    openapi::ApiDoc,
    state::AppState,
};

pub fn create_router() -> Router<AppState> {
    Router::new()
        // API routes
        .route("/api/users", post(users::create_user))
        .route("/api/users", get(users::list_users))
        .route("/api/users/{id}", get(users::get_user))
        .route("/api/users/{id}", put(users::update_user))
        .route("/api/users/{id}", delete(users::delete_user))
        .route("/api/users/fields", get(fields::list_user_fields))
        .route("/api/todos", post(todos::create_todo))
        .route("/api/todos", get(todos::list_todos))
        .route("/api/todos/{id}", get(todos::get_todo))
        .route("/api/todos/{id}", put(todos::update_todo))
        .route("/api/todos/{id}", delete(todos::delete_todo))
        .route("/api/todos/fields", get(fields::list_todo_fields))
        // Author routes
        .route("/api/authors", post(authors::create_author))
        .route("/api/authors", get(authors::list_authors))
        .route("/api/authors/{id}", get(authors::get_author))
        .route("/api/authors/{id}", put(authors::update_author))
        .route("/api/authors/{id}", delete(authors::delete_author))
        // Book routes
        .route("/api/books", post(books::create_book))
        .route("/api/books", get(books::list_books))
        .route(
            "/api/books/with-authors",
            get(books::list_books_with_authors),
        )
        .route("/api/books/{id}", get(books::get_book))
        .route(
            "/api/books/{id}/with-author",
            get(books::get_book_with_author),
        )
        .route("/api/books/{id}", put(books::update_book))
        .route("/api/books/{id}", delete(books::delete_book))
        // Meta endpoints
        .route("/api/fields", get(fields::list_all_fields))
        // Health checks
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// Health check endpoint for Kubernetes liveness probe
/// Just checks if the application is running
async fn health_check() -> &'static str {
    "OK"
}

/// Readiness check endpoint for Kubernetes readiness probe
/// Checks if the application is ready to serve traffic (DB and Redis connections are healthy)
async fn readiness_check(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let db = state.db();
    let redis = state.redis().clone();

    let checks = vec![
        (
            "database",
            Box::pin(async move {
                check_database_connection(db)
                    .await
                    .map_err(|e| e.to_string())
            }) as app::health::HealthCheckFuture,
        ),
        (
            "redis",
            Box::pin(async move {
                check_redis_connection(redis)
                    .await
                    .map_err(|e| e.to_string())
            }) as app::health::HealthCheckFuture,
        ),
    ];

    run_health_checks(checks).await
}

/// Helper function to check database connection
async fn check_database_connection(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    // Execute a simple query to verify database connectivity
    db.ping().await?;
    Ok(())
}

/// Helper function to check Redis connection
async fn check_redis_connection(
    mut redis: redis::aio::ConnectionManager,
) -> Result<(), redis::RedisError> {
    // Send PING command to verify Redis connectivity
    redis::cmd("PING").query_async(&mut redis).await
}
