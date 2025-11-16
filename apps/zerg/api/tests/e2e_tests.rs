mod common;

use axum_test::TestServer;
use pretty_assertions::assert_eq;
use serde_json::json;
use uuid::Uuid;
use zerg_api::{dto::user::UserResponseDto, routes};

use common::TestDb;

/// Helper to create a test server with a real database
async fn create_test_server() -> (TestServer, TestDb) {
    let test_db = TestDb::new().await;
    let app_state = test_db.state();
    let app = routes::create_router().with_state(app_state);
    let server = TestServer::new(app).expect("Failed to create test server");
    (server, test_db)
}

#[tokio::test]
async fn test_e2e_health_check() {
    let (server, _db) = create_test_server().await;

    let response = server.get("/health").await;

    response.assert_status_ok();
    response.assert_text("OK");
}

#[tokio::test]
async fn test_e2e_readiness_check() {
    let (server, _db) = create_test_server().await;

    let response = server.get("/ready").await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "ready");
    assert_eq!(body["database"], "connected");
}

#[tokio::test]
async fn test_e2e_create_and_get_user() {
    let (server, _db) = create_test_server().await;

    // Create a user
    let create_payload = json!({
        "username": "e2e_user",
        "email": "e2e@example.com",
        "password": "securepass123"
    });

    let create_response = server.post("/api/users").json(&create_payload).await;

    assert_eq!(create_response.status_code(), 201);
    let created_user: UserResponseDto = create_response.json();
    assert_eq!(created_user.username, "e2e_user");
    assert_eq!(created_user.email, "e2e@example.com");

    // Get the created user
    let get_response = server.get(&format!("/api/users/{}", created_user.id)).await;

    get_response.assert_status_ok();
    let fetched_user: UserResponseDto = get_response.json();
    assert_eq!(fetched_user.id, created_user.id);
    assert_eq!(fetched_user.username, "e2e_user");
    assert_eq!(fetched_user.email, "e2e@example.com");
}

#[tokio::test]
async fn test_e2e_list_users() {
    let (server, _db) = create_test_server().await;

    // Create multiple users
    for i in 1..=3 {
        let payload = json!({
            "username": format!("user{}", i),
            "email": format!("user{}@example.com", i),
            "password": "password123"
        });
        let response = server.post("/api/users").json(&payload).await;
        assert_eq!(response.status_code(), 201);
    }

    // List all users
    let response = server.get("/api/users").await;

    response.assert_status_ok();
    let users: Vec<UserResponseDto> = response.json();
    assert_eq!(users.len(), 3);

    let usernames: Vec<&str> = users.iter().map(|u| u.username.as_str()).collect();
    assert!(usernames.contains(&"user1"));
    assert!(usernames.contains(&"user2"));
    assert!(usernames.contains(&"user3"));
}

#[tokio::test]
async fn test_e2e_update_user() {
    let (server, _db) = create_test_server().await;

    // Create a user
    let create_payload = json!({
        "username": "original_name",
        "email": "original@example.com",
        "password": "password123"
    });
    let create_response = server.post("/api/users").json(&create_payload).await;
    assert_eq!(create_response.status_code(), 201);
    let created_user: UserResponseDto = create_response.json();

    // Update the user
    let update_payload = json!({
        "username": "updated_name",
        "email": "updated@example.com"
    });
    let update_response = server
        .put(&format!("/api/users/{}", created_user.id))
        .json(&update_payload)
        .await;

    update_response.assert_status_ok();
    let updated_user: UserResponseDto = update_response.json();
    assert_eq!(updated_user.username, "updated_name");
    assert_eq!(updated_user.email, "updated@example.com");

    // Verify the update by fetching
    let get_response = server.get(&format!("/api/users/{}", created_user.id)).await;
    get_response.assert_status_ok();
    let fetched_user: UserResponseDto = get_response.json();
    assert_eq!(fetched_user.username, "updated_name");
    assert_eq!(fetched_user.email, "updated@example.com");
}

#[tokio::test]
async fn test_e2e_delete_user() {
    let (server, _db) = create_test_server().await;

    // Create a user
    let create_payload = json!({
        "username": "to_delete",
        "email": "delete@example.com",
        "password": "password123"
    });
    let create_response = server.post("/api/users").json(&create_payload).await;
    assert_eq!(create_response.status_code(), 201);
    let created_user: UserResponseDto = create_response.json();

    // Delete the user
    let delete_response = server
        .delete(&format!("/api/users/{}", created_user.id))
        .await;
    assert_eq!(delete_response.status_code(), 204);

    // Verify user is deleted (should return 404 or error)
    let get_response = server.get(&format!("/api/users/{}", created_user.id)).await;
    assert_ne!(get_response.status_code(), 200);
}

#[tokio::test]
async fn test_e2e_get_nonexistent_user() {
    let (server, _db) = create_test_server().await;

    let nonexistent_id = Uuid::new_v4();
    let response = server.get(&format!("/api/users/{}", nonexistent_id)).await;

    // Should return an error status (404 or 500)
    assert_ne!(response.status_code(), 200);
}

#[tokio::test]
async fn test_e2e_field_selection() {
    let (server, _db) = create_test_server().await;

    // Create a user
    let create_payload = json!({
        "username": "field_test",
        "email": "field@example.com",
        "password": "password123"
    });
    server.post("/api/users").json(&create_payload).await;

    // Request with field selection
    let response = server.get("/api/users?fields=id,username").await;

    response.assert_status_ok();
    let users: Vec<serde_json::Value> = response.json();
    assert!(!users.is_empty());

    // Verify only requested fields are present
    assert!(users[0].get("id").is_some());
    assert!(users[0].get("username").is_some());
    assert!(users[0].get("email").is_none());
    assert!(users[0].get("created_at").is_none());
}

#[tokio::test]
async fn test_e2e_invalid_field_selection() {
    let (server, _db) = create_test_server().await;

    // Create a user first
    let create_payload = json!({
        "username": "test_user",
        "email": "test@example.com",
        "password": "password123"
    });
    let response = server.post("/api/users").json(&create_payload).await;
    assert_eq!(response.status_code(), 201);

    // Request with invalid field
    let response = server.get("/api/users?fields=id,invalid_field").await;

    // Should return an error
    assert_ne!(response.status_code(), 200);
}

#[tokio::test]
async fn test_e2e_validation_errors() {
    let (server, _db) = create_test_server().await;

    // Test invalid email
    let invalid_email = json!({
        "username": "validuser",
        "email": "not-an-email",
        "password": "password123"
    });
    let response = server.post("/api/users").json(&invalid_email).await;
    assert_ne!(response.status_code(), 200);

    // Test short username
    let short_username = json!({
        "username": "ab",
        "email": "valid@example.com",
        "password": "password123"
    });
    let response = server.post("/api/users").json(&short_username).await;
    assert_ne!(response.status_code(), 200);

    // Test short password
    let short_password = json!({
        "username": "validuser",
        "email": "valid@example.com",
        "password": "short"
    });
    let response = server.post("/api/users").json(&short_password).await;
    assert_ne!(response.status_code(), 200);
}

#[tokio::test]
async fn test_e2e_swagger_ui() {
    let (server, _db) = create_test_server().await;

    // Test that Swagger UI is accessible (redirects to /swagger-ui/)
    let response = server.get("/swagger-ui").await;
    // Swagger UI typically redirects, so we expect a redirect status
    assert!(
        response.status_code() == 303 || response.status_code() == 200,
        "Expected 200 or 303, got {}",
        response.status_code()
    );
}

#[tokio::test]
async fn test_e2e_openapi_spec() {
    let (server, _db) = create_test_server().await;

    // Test that OpenAPI spec is accessible
    let response = server.get("/api-docs/openapi.json").await;
    response.assert_status_ok();

    let spec: serde_json::Value = response.json();
    assert!(spec.get("openapi").is_some());
    assert!(spec.get("paths").is_some());
}

#[tokio::test]
async fn test_e2e_user_fields_endpoint() {
    let (server, _db) = create_test_server().await;

    let response = server.get("/api/users/fields").await;
    response.assert_status_ok();

    let fields: serde_json::Value = response.json();
    assert_eq!(fields["resource"], "users");
    assert!(fields["fields"].is_array());
    assert!(fields["accessible_fields"].is_array());

    let accessible_fields = fields["accessible_fields"].as_array().unwrap();
    let field_names: Vec<&str> = accessible_fields
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();

    assert!(field_names.contains(&"id"));
    assert!(field_names.contains(&"username"));
    assert!(field_names.contains(&"email"));
}
