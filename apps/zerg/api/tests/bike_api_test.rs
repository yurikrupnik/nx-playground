mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use zerg_api::{api, state::AppState};

/// Helper to create test state with bikes table initialized
async fn create_bike_test_state() -> AppState {
    let state = common::create_test_state().await;

    // Initialize the bikes table
    apis_bike::controller::init_bikes_table(state.sqlx_pool())
        .await
        .expect("Failed to initialize bikes table");

    state
}

#[tokio::test]
async fn test_create_bike() {
    let state = create_bike_test_state().await;
    let app = api::routes().with_state(state);

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
    let state = create_bike_test_state().await;
    let app = api::routes().with_state(state.clone());

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

    let app_clone = api::routes().with_state(state.clone());
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
    let state = create_bike_test_state().await;
    let app = api::routes().with_state(state);

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
    let state = create_bike_test_state().await;
    let app = api::routes().with_state(state.clone());

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

    let app_clone = api::routes().with_state(state.clone());
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
    let state = create_bike_test_state().await;
    let app = api::routes().with_state(state.clone());

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

    let app_clone = api::routes().with_state(state.clone());
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
    let state = create_bike_test_state().await;
    let app = api::routes().with_state(state);

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
