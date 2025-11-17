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
    dto::author::{AuthorResponseDto, CreateAuthorDto, UpdateAuthorDto},
    entities::author,
    handlers::authors::{create_author, delete_author, get_author, list_authors, update_author},
    state::AppState,
    utils::field_selector::FieldSelector,
};

fn setup_mock_state_with_authors() -> AppState {
    let author1 = author::Model {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
        name: "J.K. Rowling".to_string(),
        bio: Some("British author, best known for the Harry Potter series".to_string()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let author2 = author::Model {
        id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
        name: "George R.R. Martin".to_string(),
        bio: Some("American novelist and screenwriter".to_string()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            vec![author1.clone(), author2.clone()], // For list
            vec![author1.clone()],                  // For get
        ])
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    AppState::new(db)
}

#[tokio::test]
async fn test_list_authors() {
    let state = setup_mock_state_with_authors();
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector { fields: None };

    let result = list_authors(State(state), auth, Query(field_selector))
        .await
        .expect("list_authors should succeed");

    let authors: Vec<AuthorResponseDto> = serde_json::from_value(result.0).unwrap();
    assert_eq!(authors.len(), 2);
    assert_eq!(authors[0].name, "J.K. Rowling");
    assert_eq!(authors[1].name, "George R.R. Martin");
}

#[tokio::test]
async fn test_list_authors_with_field_selection() {
    let state = setup_mock_state_with_authors();
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector {
        fields: Some("id,name".to_string()),
    };

    let result = list_authors(State(state), auth, Query(field_selector))
        .await
        .expect("list_authors should succeed");

    let authors: Vec<serde_json::Value> = serde_json::from_value(result.0).unwrap();
    assert_eq!(authors.len(), 2);

    // Should only contain id and name, not bio or timestamps
    assert!(authors[0].get("id").is_some());
    assert!(authors[0].get("name").is_some());
    assert!(authors[0].get("bio").is_none());
    assert!(authors[0].get("created_at").is_none());
}

#[tokio::test]
async fn test_get_author_by_id() {
    let author_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![author::Model {
            id: author_id,
            name: "J.K. Rowling".to_string(),
            bio: Some("British author".to_string()),
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

    let result = get_author(State(state), auth, Path(author_id), Query(field_selector))
        .await
        .expect("get_author should succeed");

    let author: AuthorResponseDto = serde_json::from_value(result.0).unwrap();
    assert_eq!(author.name, "J.K. Rowling");
    assert_eq!(author.bio, Some("British author".to_string()));
}

#[tokio::test]
async fn test_get_author_not_found() {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([Vec::<author::Model>::new()])
        .into_connection();

    let state = AppState::new(db);
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector { fields: None };
    let author_id = Uuid::new_v4();

    let result = get_author(State(state), auth, Path(author_id), Query(field_selector)).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_author() {
    let author_id = Uuid::new_v4();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results([vec![author::Model {
            id: author_id,
            name: "New Author".to_string(),
            bio: Some("A new author biography".to_string()),
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }]])
        .into_connection();

    let state = AppState::new(db);
    let payload = CreateAuthorDto {
        name: "New Author".to_string(),
        bio: Some("A new author biography".to_string()),
    };

    let result = create_author(State(state), Json(payload))
        .await
        .expect("create_author should succeed");

    assert_eq!(result.0, StatusCode::CREATED);
    assert_eq!(result.1 .0.name, "New Author");
    assert_eq!(result.1 .0.bio, Some("A new author biography".to_string()));
}

#[tokio::test]
async fn test_update_author() {
    let author_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            vec![author::Model {
                id: author_id,
                name: "Old Name".to_string(),
                bio: Some("Old bio".to_string()),
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            }],
            vec![author::Model {
                id: author_id,
                name: "Updated Name".to_string(),
                bio: Some("Updated bio".to_string()),
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
    let payload = UpdateAuthorDto {
        name: Some("Updated Name".to_string()),
        bio: Some("Updated bio".to_string()),
    };

    let result = update_author(State(state), Path(author_id), Json(payload))
        .await
        .expect("update_author should succeed");

    assert_eq!(result.0.name, "Updated Name");
    assert_eq!(result.0.bio, Some("Updated bio".to_string()));
}

#[tokio::test]
async fn test_delete_author() {
    let author_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![author::Model {
            id: author_id,
            name: "Author to Delete".to_string(),
            bio: Some("Will be deleted".to_string()),
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let state = AppState::new(db);

    let result = delete_author(State(state), Path(author_id))
        .await
        .expect("delete_author should succeed");

    assert_eq!(result, StatusCode::NO_CONTENT);
}

#[cfg(test)]
mod validation_tests {
    use validator::Validate;
    use zerg_api::dto::author::CreateAuthorDto;

    #[test]
    fn test_create_author_dto_validation_valid() {
        let dto = CreateAuthorDto {
            name: "J.K. Rowling".to_string(),
            bio: Some("British author".to_string()),
        };

        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_create_author_dto_validation_empty_name() {
        let dto = CreateAuthorDto {
            name: "".to_string(),
            bio: None,
        };

        assert!(dto.validate().is_err());
    }

    #[test]
    fn test_create_author_dto_validation_name_too_long() {
        let dto = CreateAuthorDto {
            name: "a".repeat(256),
            bio: None,
        };

        assert!(dto.validate().is_err());
    }
}
