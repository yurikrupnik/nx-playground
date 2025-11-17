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
    dto::todo::{CreateTodoDto, TodoResponseDto, UpdateTodoDto},
    entities::todo,
    handlers::todos::{create_todo, delete_todo, get_todo, list_todos, update_todo},
    state::AppState,
    utils::field_selector::FieldSelector,
};

/// Helper to create a mock database with predefined todos
fn setup_mock_state_with_todos() -> AppState {
    let todo1 = todo::Model {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
        name: "Buy groceries".to_string(),
        description: Some("Milk, eggs, bread".to_string()),
        completed: false,
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let todo2 = todo::Model {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
        name: "Write tests".to_string(),
        description: None,
        completed: true,
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            vec![todo1.clone(), todo2.clone()], // For list_todos
            vec![todo1.clone()],                // For get_todo
        ])
        .append_exec_results([
            MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            },
            MockExecResult {
                last_insert_id: 2,
                rows_affected: 1,
            },
        ])
        .into_connection();

    AppState::new(db)
}

#[tokio::test]
async fn test_list_todos_returns_all_todos() {
    let state = setup_mock_state_with_todos();

    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector { fields: None };

    let result = list_todos(State(state), auth, Query(field_selector))
        .await
        .expect("list_todos should succeed");

    let todos: Vec<TodoResponseDto> = serde_json::from_value(result.0).unwrap();
    assert_eq!(todos.len(), 2);
    assert_eq!(todos[0].name, "Buy groceries");
    assert_eq!(todos[1].name, "Write tests");
    assert!(!todos[0].completed);
    assert!(todos[1].completed);
}

#[tokio::test]
async fn test_list_todos_with_field_selection() {
    let state = setup_mock_state_with_todos();

    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    // Request only specific fields
    let field_selector = FieldSelector {
        fields: Some("id,name,completed".to_string()),
    };

    let result = list_todos(State(state), auth, Query(field_selector))
        .await
        .expect("list_todos should succeed");

    let todos: Vec<serde_json::Value> = serde_json::from_value(result.0).unwrap();
    assert_eq!(todos.len(), 2);

    // Should only contain requested fields
    assert!(todos[0].get("id").is_some());
    assert!(todos[0].get("name").is_some());
    assert!(todos[0].get("completed").is_some());
    // Should NOT contain description or timestamps
    assert!(todos[0].get("description").is_none());
    assert!(todos[0].get("created_at").is_none());
    assert!(todos[0].get("updated_at").is_none());
}

#[tokio::test]
async fn test_get_todo_by_id() {
    let todo_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![todo::Model {
            id: todo_id,
            name: "Buy groceries".to_string(),
            description: Some("Milk, eggs, bread".to_string()),
            completed: false,
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

    let result = get_todo(State(state), auth, Path(todo_id), Query(field_selector))
        .await
        .expect("get_todo should succeed");

    let todo: TodoResponseDto = serde_json::from_value(result.0).unwrap();
    assert_eq!(todo.name, "Buy groceries");
    assert_eq!(todo.description, Some("Milk, eggs, bread".to_string()));
    assert!(!todo.completed);
}

#[tokio::test]
async fn test_get_todo_not_found() {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<todo::Model>::new()]) // Empty result
        .into_connection();

    let state = AppState::new(db);
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector { fields: None };
    let todo_id = Uuid::new_v4();

    let result = get_todo(State(state), auth, Path(todo_id), Query(field_selector)).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_todo_validation() {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results([vec![todo::Model {
            id: Uuid::new_v4(),
            name: "New task".to_string(),
            description: None,
            completed: false,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }]])
        .into_connection();

    let state = AppState::new(db);
    let payload = CreateTodoDto {
        name: "New task".to_string(),
        description: None,
        completed: None,
    };

    let result = create_todo(State(state), Json(payload))
        .await
        .expect("create_todo should succeed");

    assert_eq!(result.0, StatusCode::CREATED);
    assert_eq!(result.1 .0.name, "New task");
}

#[tokio::test]
async fn test_update_todo() {
    let todo_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            vec![todo::Model {
                id: todo_id,
                name: "Old name".to_string(),
                description: None,
                completed: false,
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            }],
            vec![todo::Model {
                id: todo_id,
                name: "Updated name".to_string(),
                description: Some("New description".to_string()),
                completed: true,
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
    let payload = UpdateTodoDto {
        name: Some("Updated name".to_string()),
        description: Some("New description".to_string()),
        completed: Some(true),
    };

    let result = update_todo(State(state), Path(todo_id), Json(payload))
        .await
        .expect("update_todo should succeed");

    assert_eq!(result.0.name, "Updated name");
    assert_eq!(result.0.description, Some("New description".to_string()));
    assert!(result.0.completed);
}

#[tokio::test]
async fn test_delete_todo() {
    let todo_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![todo::Model {
            id: todo_id,
            name: "To be deleted".to_string(),
            description: None,
            completed: false,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let state = AppState::new(db);

    let result = delete_todo(State(state), Path(todo_id))
        .await
        .expect("delete_todo should succeed");

    assert_eq!(result, StatusCode::NO_CONTENT);
}

#[cfg(test)]
mod validation_tests {
    use validator::Validate;
    use zerg_api::dto::todo::{CreateTodoDto, UpdateTodoDto};

    #[test]
    fn test_create_todo_dto_validation_valid() {
        let dto = CreateTodoDto {
            name: "Valid task name".to_string(),
            description: Some("Optional description".to_string()),
            completed: Some(false),
        };

        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_create_todo_dto_validation_empty_name() {
        let dto = CreateTodoDto {
            name: "".to_string(), // Empty string (min length 1)
            description: None,
            completed: None,
        };

        assert!(dto.validate().is_err());
    }

    #[test]
    fn test_create_todo_dto_validation_name_too_long() {
        let dto = CreateTodoDto {
            name: "a".repeat(256), // Exceeds max length of 255
            description: None,
            completed: None,
        };

        assert!(dto.validate().is_err());
    }

    #[test]
    fn test_update_todo_dto_validation_valid() {
        let dto = UpdateTodoDto {
            name: Some("Updated name".to_string()),
            description: None,
            completed: Some(true),
        };

        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_update_todo_dto_validation_empty_name() {
        let dto = UpdateTodoDto {
            name: Some("".to_string()), // Empty string not allowed
            description: None,
            completed: None,
        };

        assert!(dto.validate().is_err());
    }
}
