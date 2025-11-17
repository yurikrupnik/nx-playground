use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use uuid::Uuid;

use zerg_api::{
    auth_context::{AuthContext, UserRole},
    dto::user::{CreateUserDto, UpdateUserDto, UserResponseDto},
    entities::user,
    handlers::users::{create_user, delete_user, get_user, list_users, update_user},
    state::AppState,
    utils::field_selector::FieldSelector,
};

/// Helper to create a mock database with predefined users
fn setup_mock_state_with_users() -> AppState {
    let user1 = user::Model {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        password_hash: "hashed_password_1".to_string(),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let user2 = user::Model {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
        password_hash: "hashed_password_2".to_string(),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            vec![user1.clone(), user2.clone()], // For list_users
            vec![user1.clone()],                // For get_user
        ])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    AppState::new(db)
}

#[tokio::test]
async fn test_list_users_returns_all_users() {
    let state = setup_mock_state_with_users();
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector { fields: None };

    let result = list_users(State(state), auth, Query(field_selector))
        .await
        .expect("list_users should succeed");

    let users: Vec<UserResponseDto> = serde_json::from_value(result.0).unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].username, "alice");
    assert_eq!(users[1].username, "bob");
}

#[tokio::test]
async fn test_list_users_with_field_selection() {
    let state = setup_mock_state_with_users();
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    // Request only id and username fields
    let field_selector = FieldSelector {
        fields: Some("id,username".to_string()),
    };

    let result = list_users(State(state), auth, Query(field_selector))
        .await
        .expect("list_users should succeed");

    let users: Vec<serde_json::Value> = serde_json::from_value(result.0).unwrap();
    assert_eq!(users.len(), 2);

    // Should only contain id and username, not email or timestamps
    assert!(users[0].get("id").is_some());
    assert!(users[0].get("username").is_some());
    assert!(users[0].get("email").is_none());
    assert!(users[0].get("created_at").is_none());
}

#[tokio::test]
async fn test_get_user_by_id() {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![user::Model {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }]])
        .into_connection();

    let state = AppState::new(db);
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector { fields: None };
    let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

    let result = get_user(State(state), auth, Path(user_id), Query(field_selector))
        .await
        .expect("get_user should succeed");

    let user: UserResponseDto = serde_json::from_value(result.0).unwrap();
    assert_eq!(user.username, "alice");
    assert_eq!(user.email, "alice@example.com");
}

#[tokio::test]
async fn test_get_user_not_found() {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<user::Model>::new()]) // Empty result
        .into_connection();

    let state = AppState::new(db);
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector { fields: None };
    let user_id = Uuid::new_v4();

    let result = get_user(State(state), auth, Path(user_id), Query(field_selector)).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_field_selector_validates_invalid_fields() {
    let state = setup_mock_state_with_users();
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    // Request invalid field
    let field_selector = FieldSelector {
        fields: Some("id,invalid_field,username".to_string()),
    };

    let result = list_users(State(state), auth, Query(field_selector)).await;

    // Should return validation error for invalid field
    assert!(result.is_err());
}

#[tokio::test]
async fn test_rbac_field_filtering() {
    let state = setup_mock_state_with_users();

    // Anonymous user requesting all fields
    let auth_anon = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector { fields: None };

    let result = list_users(State(state), auth_anon, Query(field_selector))
        .await
        .expect("list_users should succeed");

    // Verify that all fields accessible to anonymous are present
    let users: Vec<serde_json::Value> = serde_json::from_value(result.0).unwrap();
    assert!(users[0].get("id").is_some());
    assert!(users[0].get("username").is_some());
    assert!(users[0].get("email").is_some()); // Email is public in current impl
}

#[tokio::test]
async fn test_create_user() {
    let user_id = Uuid::new_v4();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results([vec![user::Model {
            id: user_id,
            username: "newuser".to_string(),
            email: "newuser@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }]])
        .into_connection();

    let state = AppState::new(db);
    let payload = CreateUserDto {
        username: "newuser".to_string(),
        email: "newuser@example.com".to_string(),
        password: "securepassword123".to_string(),
    };

    let result = create_user(State(state), Json(payload))
        .await
        .expect("create_user should succeed");

    assert_eq!(result.0, StatusCode::CREATED);
    assert_eq!(result.1 .0.username, "newuser");
    assert_eq!(result.1 .0.email, "newuser@example.com");
}

#[tokio::test]
async fn test_update_user() {
    let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            vec![user::Model {
                id: user_id,
                username: "oldusername".to_string(),
                email: "old@example.com".to_string(),
                password_hash: "hashed_password".to_string(),
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            }],
            vec![user::Model {
                id: user_id,
                username: "updatedusername".to_string(),
                email: "updated@example.com".to_string(),
                password_hash: "hashed_password".to_string(),
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            }],
        ])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let state = AppState::new(db);
    let payload = UpdateUserDto {
        username: Some("updatedusername".to_string()),
        email: Some("updated@example.com".to_string()),
        password: None,
    };

    let result = update_user(State(state), Path(user_id), Json(payload))
        .await
        .expect("update_user should succeed");

    assert_eq!(result.0.username, "updatedusername");
    assert_eq!(result.0.email, "updated@example.com");
}

#[tokio::test]
async fn test_delete_user() {
    let user_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![user::Model {
            id: user_id,
            username: "userToDelete".to_string(),
            email: "delete@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let state = AppState::new(db);

    let result = delete_user(State(state), Path(user_id))
        .await
        .expect("delete_user should succeed");

    assert_eq!(result, StatusCode::NO_CONTENT);
}

#[cfg(test)]
mod validation_tests {
    use validator::Validate;
    use zerg_api::dto::user::CreateUserDto;

    #[test]
    fn test_create_user_dto_validation_valid() {
        let dto = CreateUserDto {
            username: "validuser".to_string(),
            email: "valid@example.com".to_string(),
            password: "securepassword123".to_string(),
        };

        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_create_user_dto_validation_short_username() {
        let dto = CreateUserDto {
            username: "ab".to_string(), // Too short (min 3)
            email: "valid@example.com".to_string(),
            password: "securepassword123".to_string(),
        };

        assert!(dto.validate().is_err());
    }

    #[test]
    fn test_create_user_dto_validation_invalid_email() {
        let dto = CreateUserDto {
            username: "validuser".to_string(),
            email: "not-an-email".to_string(), // Invalid email
            password: "securepassword123".to_string(),
        };

        assert!(dto.validate().is_err());
    }

    #[test]
    fn test_create_user_dto_validation_short_password() {
        let dto = CreateUserDto {
            username: "validuser".to_string(),
            email: "valid@example.com".to_string(),
            password: "short".to_string(), // Too short (min 8)
        };

        assert!(dto.validate().is_err());
    }
}
