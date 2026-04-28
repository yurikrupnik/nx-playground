mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use zerg_api::api;

#[tokio::test]
async fn test_create_car() {
    let state = common::create_test_state().await;
    let app = api::routes().with_state(state);

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
    let state = common::create_test_state().await;
    let app = api::routes().with_state(state);

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
    let state = common::create_test_state().await;
    let app = api::routes().with_state(state);

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
    let state = common::create_test_state().await;
    let app = api::routes().with_state(state);

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
    let state = common::create_test_state().await;
    let app = api::routes().with_state(state.clone());

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
    let app = api::routes().with_state(state);
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
    let state = common::create_test_state().await;
    let app = api::routes().with_state(state.clone());

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
    let app = api::routes().with_state(state);
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
