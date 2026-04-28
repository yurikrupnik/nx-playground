mod common;

use axum::extract::{Path, Query, State};
use pretty_assertions::assert_eq;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;
use zerg_api::{
    auth_context::{AuthContext, UserRole},
    dto::user::{CreateUserDto, UpdateUserDto, UserResponseDto},
    entities::{prelude::*, user},
    handlers::users::{create_user, delete_user, get_user, list_users, update_user},
    utils::field_selector::FieldSelector,
};

use common::TestDb;

#[tokio::test]
async fn test_integration_create_user() {
    let test_db = TestDb::new().await;
    let state = test_db.state();

    let create_dto = CreateUserDto {
        username: "integration_user".to_string(),
        email: "integration@example.com".to_string(),
        password: "securepassword123".to_string(),
    };

    let result = create_user(State(state.clone()), axum::Json(create_dto))
        .await
        .expect("Failed to create user");

    let user: UserResponseDto = result.1 .0;
    assert_eq!(user.username, "integration_user");
    assert_eq!(user.email, "integration@example.com");

    // Verify user exists in database
    let db_user = User::find_by_id(user.id)
        .one(state.db())
        .await
        .expect("Failed to query database")
        .expect("User not found in database");

    assert_eq!(db_user.username, "integration_user");
    assert_eq!(db_user.email, "integration@example.com");
    // Password should be hashed
    assert_ne!(db_user.password_hash, "securepassword123");
}

#[tokio::test]
async fn test_integration_list_users() {
    let test_db = TestDb::new().await;
    let state = test_db.state();

    // Create test users
    let user1 = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set("alice".to_string()),
        email: Set("alice@example.com".to_string()),
        password_hash: Set("hashed1".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    user1
        .insert(state.db())
        .await
        .expect("Failed to insert user1");

    let user2 = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set("bob".to_string()),
        email: Set("bob@example.com".to_string()),
        password_hash: Set("hashed2".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    user2
        .insert(state.db())
        .await
        .expect("Failed to insert user2");

    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };

    let field_selector = FieldSelector { fields: None };

    let result = list_users(State(state), auth, Query(field_selector))
        .await
        .expect("Failed to list users");

    let users: Vec<UserResponseDto> = serde_json::from_value(result.0).unwrap();
    // 3 seeded users + 2 created in test = 5 total
    assert_eq!(users.len(), 5);

    let usernames: Vec<&str> = users.iter().map(|u| u.username.as_str()).collect();
    assert!(usernames.contains(&"alice"));
    assert!(usernames.contains(&"bob"));
    assert!(usernames.contains(&"admin"));
    assert!(usernames.contains(&"john_doe"));
    assert!(usernames.contains(&"jane_smith"));
}

#[tokio::test]
async fn test_integration_get_user() {
    let test_db = TestDb::new().await;
    let state = test_db.state();

    // Create a test user
    let user_model = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set("charlie".to_string()),
        email: Set("charlie@example.com".to_string()),
        password_hash: Set("hashed_password".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    let created_user = user_model
        .insert(state.db())
        .await
        .expect("Failed to insert user");

    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };

    let field_selector = FieldSelector { fields: None };

    let result = get_user(
        State(state),
        auth,
        Path(created_user.id),
        Query(field_selector),
    )
    .await
    .expect("Failed to get user");

    let user: UserResponseDto = serde_json::from_value(result.0).unwrap();
    assert_eq!(user.username, "charlie");
    assert_eq!(user.email, "charlie@example.com");
    assert_eq!(user.id, created_user.id);
}

#[tokio::test]
async fn test_integration_update_user() {
    let test_db = TestDb::new().await;
    let state = test_db.state();

    // Create a test user
    let user_model = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set("diana".to_string()),
        email: Set("diana@example.com".to_string()),
        password_hash: Set("hashed_password".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    let created_user = user_model
        .insert(state.db())
        .await
        .expect("Failed to insert user");

    let update_dto = UpdateUserDto {
        username: Some("diana_updated".to_string()),
        email: Some("diana_new@example.com".to_string()),
        password: None,
    };

    let result = update_user(
        State(state.clone()),
        Path(created_user.id),
        axum::Json(update_dto),
    )
    .await
    .expect("Failed to update user");

    let user: UserResponseDto = result.0;
    assert_eq!(user.username, "diana_updated");
    assert_eq!(user.email, "diana_new@example.com");

    // Verify in database
    let db_user = User::find_by_id(created_user.id)
        .one(state.db())
        .await
        .expect("Failed to query database")
        .expect("User not found");

    assert_eq!(db_user.username, "diana_updated");
    assert_eq!(db_user.email, "diana_new@example.com");
}

#[tokio::test]
async fn test_integration_delete_user() {
    let test_db = TestDb::new().await;
    let state = test_db.state();

    // Create a test user
    let user_model = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set("to_delete".to_string()),
        email: Set("delete@example.com".to_string()),
        password_hash: Set("hashed_password".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    let created_user = user_model
        .insert(state.db())
        .await
        .expect("Failed to insert user");

    // Delete the user
    delete_user(State(state.clone()), Path(created_user.id))
        .await
        .expect("Failed to delete user");

    // Verify user is deleted
    let db_user = User::find_by_id(created_user.id)
        .one(state.db())
        .await
        .expect("Failed to query database");

    assert!(db_user.is_none(), "User should be deleted");
}

#[tokio::test]
async fn test_integration_get_nonexistent_user() {
    let test_db = TestDb::new().await;
    let state = test_db.state();

    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };

    let field_selector = FieldSelector { fields: None };
    let nonexistent_id = Uuid::new_v4();

    let result = get_user(
        State(state),
        auth,
        Path(nonexistent_id),
        Query(field_selector),
    )
    .await;

    assert!(result.is_err(), "Should return error for nonexistent user");
}

#[tokio::test]
async fn test_integration_field_selection() {
    let test_db = TestDb::new().await;
    let state = test_db.state();

    // Create a test user
    let user_model = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set("field_test".to_string()),
        email: Set("field@example.com".to_string()),
        password_hash: Set("hashed".to_string()),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    user_model
        .insert(state.db())
        .await
        .expect("Failed to insert user");

    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };

    // Request only id and username
    let field_selector = FieldSelector {
        fields: Some("id,username".to_string()),
    };

    let result = list_users(State(state), auth, Query(field_selector))
        .await
        .expect("Failed to list users");

    let users: Vec<serde_json::Value> = serde_json::from_value(result.0).unwrap();
    // 3 seeded users + 1 created in test = 4 total
    assert_eq!(users.len(), 4);

    // Verify only requested fields are present (check first user)
    assert!(users[0].get("id").is_some());
    assert!(users[0].get("username").is_some());
    assert!(users[0].get("email").is_none());
    assert!(users[0].get("created_at").is_none());
}
