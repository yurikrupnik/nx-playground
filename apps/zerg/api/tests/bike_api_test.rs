use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use mongodb::Client;
use serde_json::{json, Value};
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::{mongo::Mongo, postgres::Postgres, redis::Redis};
use tower::ServiceExt;
use zerg_api::{api, state::AppState};

struct TestContext {
    #[allow(dead_code)]
    postgres_container: ContainerAsync<Postgres>,
    #[allow(dead_code)]
    mongo_container: ContainerAsync<Mongo>,
    #[allow(dead_code)]
    redis_container: ContainerAsync<Redis>,
    state: AppState,
}

async fn create_test_state() -> TestContext {
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

    let postgres_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!(
        "postgresql://postgres:postgres@localhost:{}/postgres",
        postgres_port
    );

    let mongo_port = mongo_container.get_host_port_ipv4(27017).await.unwrap();
    let mongo_uri = format!("mongodb://localhost:{}/", mongo_port);

    let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
    let redis_url = format!("redis://localhost:{}/", redis_port);

    let db = services::postgres::connect(&db_url).await.unwrap();

    let mongo_client = mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to test MongoDB");
    let mongo = mongo_client.database("bikes_test");

    let redis_client = redis::Client::open(redis_url).unwrap();
    let redis = redis::aio::ConnectionManager::new(redis_client)
        .await
        .unwrap();

    let sqlx_pool = sqlx::PgPool::connect(&db_url).await.unwrap();

    // Initialize the bikes table
    apis_bike::controller::init_bikes_table(&sqlx_pool)
        .await
        .expect("Failed to initialize bikes table");

    let state = AppState::new(db, mongo, redis, sqlx_pool);

    TestContext {
        postgres_container,
        mongo_container,
        redis_container,
        state,
    }
}

#[tokio::test]
async fn test_create_bike() {
    let context = create_test_state().await;
    let app = api::routes().with_state(context.state);

    let request = Request::builder()
        .method("POST")
        .uri("/bike")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "brand": "Trek",
                "model": "Domane SL 7",
                "bike_type": "Road",
                "frame_size": 56,
                "color": "Black"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let bike: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(bike["brand"], "Trek");
    assert_eq!(bike["model"], "Domane SL 7");
    assert_eq!(bike["type"], "Road");
    assert_eq!(bike["frame_size"], 56);
    assert_eq!(bike["color"], "Black");
    assert!(bike["id"].is_string());
    assert!(bike["created_at"].is_string());
    assert!(bike["updated_at"].is_string());
}

#[tokio::test]
async fn test_list_bikes() {
    let context = create_test_state().await;
    let app = api::routes().with_state(context.state.clone());

    // Create a bike first
    let create_request = Request::builder()
        .method("POST")
        .uri("/bike")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "brand": "Specialized",
                "model": "Roubaix",
                "bike_type": "Road",
                "frame_size": 58,
                "color": "Red"
            })
            .to_string(),
        ))
        .unwrap();

    let app_clone = api::routes().with_state(context.state);
    let _ = app_clone.oneshot(create_request).await.unwrap();

    // Now list bikes
    let list_request = Request::builder()
        .method("GET")
        .uri("/bikes")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(list_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let bikes: Vec<Value> = serde_json::from_slice(&body).unwrap();

    assert!(!bikes.is_empty());
    assert_eq!(bikes[0]["brand"], "Specialized");
}

#[tokio::test]
async fn test_get_bike_not_found() {
    let context = create_test_state().await;
    let app = api::routes().with_state(context.state);

    let request = Request::builder()
        .method("GET")
        .uri("/api/bike/00000000-0000-0000-0000-000000000000")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_bike() {
    let context = create_test_state().await;
    let app = api::routes().with_state(context.state.clone());

    // Create a bike first
    let create_request = Request::builder()
        .method("POST")
        .uri("/bike")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "brand": "Giant",
                "model": "TCR",
                "bike_type": "Road",
                "frame_size": 54,
                "color": "Blue"
            })
            .to_string(),
        ))
        .unwrap();

    let app_clone = api::routes().with_state(context.state.clone());
    let create_response = app_clone.oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_bike: Value = serde_json::from_slice(&create_body).unwrap();
    let bike_id = created_bike["id"].as_str().unwrap();

    // Update the bike
    let update_request = Request::builder()
        .method("PUT")
        .uri(format!("/bike/{}", bike_id))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "color": "Green"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(update_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated_bike: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated_bike["color"], "Green");
    assert_eq!(updated_bike["brand"], "Giant");
}

#[tokio::test]
async fn test_delete_bike() {
    let context = create_test_state().await;
    let app = api::routes().with_state(context.state.clone());

    // Create a bike first
    let create_request = Request::builder()
        .method("POST")
        .uri("/bike")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "brand": "Cannondale",
                "model": "SuperSix",
                "bike_type": "Road",
                "frame_size": 52,
                "color": "Yellow"
            })
            .to_string(),
        ))
        .unwrap();

    let app_clone = api::routes().with_state(context.state);
    let create_response = app_clone.oneshot(create_request).await.unwrap();
    let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_bike: Value = serde_json::from_slice(&create_body).unwrap();
    let bike_id = created_bike["id"].as_str().unwrap();

    // Delete the bike
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!("/bike/{}", bike_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(delete_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_bike_validation_fails() {
    let context = create_test_state().await;
    let app = api::routes().with_state(context.state);

    // Invalid frame size (too large)
    let request = Request::builder()
        .method("POST")
        .uri("/bike")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "brand": "Trek",
                "model": "Domane",
                "bike_type": "Road",
                "frame_size": 100,
                "color": "Black"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
