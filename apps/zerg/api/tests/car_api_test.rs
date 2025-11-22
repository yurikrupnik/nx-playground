use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use mongodb::Client;
use redis::aio::ConnectionManager;
use serde_json::json;
use tower::ServiceExt;
use zerg_api::{api, state::AppState};

use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::{mongo::Mongo, postgres::Postgres, redis::Redis};

/// Test context that holds containers and state
struct TestContext {
    #[allow(dead_code)]
    postgres_container: ContainerAsync<Postgres>,
    #[allow(dead_code)]
    mongo_container: ContainerAsync<Mongo>,
    #[allow(dead_code)]
    redis_container: ContainerAsync<Redis>,
    state: AppState,
}

/// Helper to create test state with all required connections using testcontainers
async fn create_test_state() -> TestContext {
    // Start containers with latest versions
    let postgres_container = Postgres::default()
        .with_tag("17-alpine")
        .start()
        .await
        .expect("Failed to start Postgres container");

    let mongo_container = Mongo::default()
        .with_tag("7")
        .with_startup_timeout(std::time::Duration::from_secs(180))
        .start()
        .await
        .expect("Failed to start MongoDB container");

    let redis_container = Redis::default()
        .with_tag("7-alpine")
        .with_startup_timeout(std::time::Duration::from_secs(180))
        .start()
        .await
        .expect("Failed to start Redis container");

    // Get connection info
    let postgres_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
    let postgres_url = format!(
        "postgresql://postgres:postgres@localhost:{}/postgres",
        postgres_port
    );

    let mongo_port = mongo_container.get_host_port_ipv4(27017).await.unwrap();
    let mongo_uri = format!("mongodb://localhost:{}/", mongo_port);

    let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
    let redis_url = format!("redis://localhost:{}/", redis_port);

    // Connect to databases
    let db = services::postgres::connect(&postgres_url)
        .await
        .expect("Failed to connect to test Postgres");

    let mongo_client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to test MongoDB");
    let mongo = mongo_client.database("zerg_test");

    let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let redis = ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to test Redis");

    let sqlx_pool = sqlx::PgPool::connect(&postgres_url).await.expect("Failed to create sqlx pool");
    let state = AppState::new(db, mongo, redis, sqlx_pool);

    TestContext {
        postgres_container,
        mongo_container,
        redis_container,
        state,
    }
}

#[tokio::test]
async fn test_create_car() {
    let ctx = create_test_state().await;
    let app = api::routes().with_state(ctx.state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/car")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "make": "Tesla",
                        "model": "Model 3",
                        "year": 2024,
                        "color": "White"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let car: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(car["make"], "Tesla");
    assert_eq!(car["model"], "Model 3");
    assert_eq!(car["year"], 2024);
    assert!(car["id"].is_string());
}

#[tokio::test]
async fn test_list_cars() {
    let ctx = create_test_state().await;
    let app = api::routes().with_state(ctx.state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/cars")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let cars: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(cars.is_array());
}

#[tokio::test]
async fn test_create_car_validation_fails() {
    let ctx = create_test_state().await;
    let app = api::routes().with_state(ctx.state);

    // Invalid year
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/car")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "make": "Tesla",
                        "model": "Model 3",
                        "year": 1800,  // Too old
                        "color": "White"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_car_not_found() {
    let ctx = create_test_state().await;
    let app = api::routes().with_state(ctx.state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/car/507f1f77bcf86cd799439011") // Non-existent ID
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_car() {
    let ctx = create_test_state().await;
    let app = api::routes().with_state(ctx.state.clone());

    // First create a car
    let create_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/car")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "make": "Honda",
                        "model": "Civic",
                        "year": 2023,
                        "color": "Blue"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let car: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let car_id = car["id"].as_str().unwrap();

    // Now update it
    let app = api::routes().with_state(ctx.state);
    let update_response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/car/{}", car_id))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "color": "Red"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(update_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated_car: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated_car["color"], "Red");
    assert_eq!(updated_car["make"], "Honda"); // Unchanged
}

#[tokio::test]
async fn test_delete_car() {
    let ctx = create_test_state().await;
    let app = api::routes().with_state(ctx.state.clone());

    // First create a car
    let create_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/car")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "make": "Ford",
                        "model": "F-150",
                        "year": 2022,
                        "color": "Black"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let car: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let car_id = car["id"].as_str().unwrap();

    // Now delete it
    let app = api::routes().with_state(ctx.state);
    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/car/{}", car_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);
}
