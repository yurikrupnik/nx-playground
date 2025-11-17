use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{NaiveDate, Utc};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use uuid::Uuid;

use zerg_api::{
    auth_context::{AuthContext, UserRole},
    dto::book::{BookResponseDto, BookWithAuthorDto, CreateBookDto, UpdateBookDto},
    entities::{author, book},
    handlers::books::{
        create_book, delete_book, get_book, get_book_with_author, list_books,
        list_books_with_authors, update_book, ListBooksParams,
    },
    state::{AppState, AppStateBuilder},
    utils::field_selector::FieldSelector,
};

async fn setup_mock_state_for_create_book() -> (AppState, Uuid, Uuid) {
    let author_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let book_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010").unwrap();

    let author = author::Model {
        id: author_id,
        name: "J.K. Rowling".to_string(),
        bio: Some("British author".to_string()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let book = book::Model {
        id: book_id,
        title: "New Book".to_string(),
        description: Some("A great book".to_string()),
        author_id,
        published_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        isbn: Some("978-1234567890".to_string()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![author.clone()]]) // For author existence check
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .append_query_results([vec![book.clone()]]) // For returning the inserted book
        .into_connection();

    (
        AppStateBuilder::new()
            .with_db(db)
            .with_redis_mock()
            .await
            .build(),
        author_id,
        book_id,
    )
}

async fn setup_mock_state_with_books() -> (AppState, Uuid, Uuid) {
    let author_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let book_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010").unwrap();

    let book = book::Model {
        id: book_id,
        title: "Harry Potter and the Philosopher's Stone".to_string(),
        description: Some("The first novel in the Harry Potter series".to_string()),
        author_id,
        published_date: Some(NaiveDate::from_ymd_opt(1997, 6, 26).unwrap()),
        isbn: Some("978-0747532699".to_string()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![book.clone()]]) // For list/get book queries
        .into_connection();

    (
        AppStateBuilder::new()
            .with_db(db)
            .with_redis_mock()
            .await
            .build(),
        author_id,
        book_id,
    )
}

#[tokio::test]
async fn test_create_book() {
    let (state, author_id, _) = setup_mock_state_for_create_book().await;

    let payload = CreateBookDto {
        title: "New Book".to_string(),
        description: Some("A great book".to_string()),
        author_id,
        published_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        isbn: Some("978-1234567890".to_string()),
    };

    let result = create_book(State(state), Json(payload))
        .await
        .expect("create_book should succeed");

    assert_eq!(result.0, StatusCode::CREATED);
}

#[tokio::test]
async fn test_list_books() {
    let (state, _, _) = setup_mock_state_with_books().await;
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };

    let params = ListBooksParams {
        field_selector: FieldSelector { fields: None },
        author_id: None,
    };

    let result = list_books(State(state), auth, Query(params))
        .await
        .expect("list_books should succeed");

    let books: Vec<BookResponseDto> = serde_json::from_value(result.0).unwrap();
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].title, "Harry Potter and the Philosopher's Stone");
}

#[tokio::test]
async fn test_list_books_filtered_by_author() {
    let (state, author_id, _) = setup_mock_state_with_books().await;
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };

    let params = ListBooksParams {
        field_selector: FieldSelector { fields: None },
        author_id: Some(author_id),
    };

    let result = list_books(State(state), auth, Query(params))
        .await
        .expect("list_books should succeed");

    let books: Vec<BookResponseDto> = serde_json::from_value(result.0).unwrap();
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].author_id, author_id);
}

#[tokio::test]
async fn test_get_book_by_id() {
    let (state, author_id, book_id) = setup_mock_state_with_books().await;
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector { fields: None };

    let result = get_book(State(state), auth, Path(book_id), Query(field_selector))
        .await
        .expect("get_book should succeed");

    let book: BookResponseDto = serde_json::from_value(result.0).unwrap();
    assert_eq!(book.title, "Harry Potter and the Philosopher's Stone");
    assert_eq!(book.author_id, author_id);
}

#[tokio::test]
async fn test_list_books_with_authors() {
    use zerg_api::handlers::books::AuthorFilterParams;

    let author_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let book_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010").unwrap();

    let author = author::Model {
        id: author_id,
        name: "J.K. Rowling".to_string(),
        bio: Some("British author".to_string()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let book = book::Model {
        id: book_id,
        title: "Harry Potter and the Philosopher's Stone".to_string(),
        description: Some("The first novel".to_string()),
        author_id,
        published_date: Some(NaiveDate::from_ymd_opt(1997, 6, 26).unwrap()),
        isbn: Some("978-0747532699".to_string()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    // For joined queries, MockDatabase expects tuples
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![(book.clone(), Some(author.clone()))]])
        .into_connection();

    let state = AppStateBuilder::new()
        .with_db(db)
        .with_redis_mock()
        .await
        .build();
    let params = AuthorFilterParams { author_id: None };

    let result = list_books_with_authors(State(state), Query(params))
        .await
        .expect("list_books_with_authors should succeed");

    let books: Vec<BookWithAuthorDto> = result.0;
    assert_eq!(books.len(), 1);
    assert_eq!(books[0].title, "Harry Potter and the Philosopher's Stone");
    assert_eq!(books[0].author.name, "J.K. Rowling");
    assert_eq!(books[0].author_id, author_id);
}

#[tokio::test]
async fn test_get_book_with_author() {
    let author_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let book_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010").unwrap();

    let author = author::Model {
        id: author_id,
        name: "J.K. Rowling".to_string(),
        bio: Some("British author".to_string()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let book = book::Model {
        id: book_id,
        title: "Harry Potter and the Philosopher's Stone".to_string(),
        description: Some("The first novel".to_string()),
        author_id,
        published_date: Some(NaiveDate::from_ymd_opt(1997, 6, 26).unwrap()),
        isbn: Some("978-0747532699".to_string()),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![(book.clone(), Some(author.clone()))]])
        .into_connection();

    let state = AppStateBuilder::new()
        .with_db(db)
        .with_redis_mock()
        .await
        .build();

    let result = get_book_with_author(State(state), Path(book_id))
        .await
        .expect("get_book_with_author should succeed");

    let book: BookWithAuthorDto = result.0;
    assert_eq!(book.title, "Harry Potter and the Philosopher's Stone");
    assert_eq!(book.author.name, "J.K. Rowling");
    assert_eq!(book.author_id, author_id);
    assert_eq!(book.author.id, author_id);
}

#[tokio::test]
async fn test_book_with_field_selection() {
    let (state, _, book_id) = setup_mock_state_with_books().await;
    let auth = AuthContext {
        user_id: None,
        role: UserRole::Anonymous,
        username: None,
    };
    let field_selector = FieldSelector {
        fields: Some("id,title,author_id".to_string()),
    };

    let result = get_book(State(state), auth, Path(book_id), Query(field_selector))
        .await
        .expect("get_book should succeed");

    let book: serde_json::Value = result.0;
    // Should only have selected fields
    assert!(book.get("id").is_some());
    assert!(book.get("title").is_some());
    assert!(book.get("author_id").is_some());
    // Should NOT have other fields
    assert!(book.get("description").is_none());
    assert!(book.get("published_date").is_none());
    assert!(book.get("isbn").is_none());
}

#[tokio::test]
async fn test_update_book() {
    let author_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let book_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010").unwrap();

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([
            vec![book::Model {
                id: book_id,
                title: "Old Title".to_string(),
                description: Some("Old description".to_string()),
                author_id,
                published_date: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
                isbn: Some("978-1111111111".to_string()),
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            }],
            vec![book::Model {
                id: book_id,
                title: "Updated Title".to_string(),
                description: Some("Updated description".to_string()),
                author_id,
                published_date: Some(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap()),
                isbn: Some("978-2222222222".to_string()),
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            }],
        ])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let state = AppStateBuilder::new()
        .with_db(db)
        .with_redis_mock()
        .await
        .build();
    let payload = UpdateBookDto {
        title: Some("Updated Title".to_string()),
        description: Some("Updated description".to_string()),
        author_id: None,
        published_date: Some(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap()),
        isbn: Some("978-2222222222".to_string()),
    };

    let result = update_book(State(state), Path(book_id), Json(payload))
        .await
        .expect("update_book should succeed");

    assert_eq!(result.0.title, "Updated Title");
    assert_eq!(
        result.0.description,
        Some("Updated description".to_string())
    );
    assert_eq!(
        result.0.published_date,
        Some(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap())
    );
    assert_eq!(result.0.isbn, Some("978-2222222222".to_string()));
}

#[tokio::test]
async fn test_delete_book() {
    let author_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    let book_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440010").unwrap();

    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![book::Model {
            id: book_id,
            title: "Book to Delete".to_string(),
            description: Some("Will be deleted".to_string()),
            author_id,
            published_date: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            isbn: Some("978-1234567890".to_string()),
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }]])
        .append_exec_results([MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let state = AppStateBuilder::new()
        .with_db(db)
        .with_redis_mock()
        .await
        .build();

    let result = delete_book(State(state), Path(book_id))
        .await
        .expect("delete_book should succeed");

    assert_eq!(result, StatusCode::NO_CONTENT);
}

#[cfg(test)]
mod validation_tests {
    use chrono::NaiveDate;
    use uuid::Uuid;
    use validator::Validate;
    use zerg_api::dto::book::CreateBookDto;

    #[test]
    fn test_create_book_dto_validation_valid() {
        let dto = CreateBookDto {
            title: "Valid Book".to_string(),
            description: Some("A description".to_string()),
            author_id: Uuid::new_v4(),
            published_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
            isbn: Some("978-1234567890".to_string()),
        };

        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_create_book_dto_validation_empty_title() {
        let dto = CreateBookDto {
            title: "".to_string(),
            description: None,
            author_id: Uuid::new_v4(),
            published_date: None,
            isbn: None,
        };

        assert!(dto.validate().is_err());
    }

    #[test]
    fn test_create_book_dto_validation_title_too_long() {
        let dto = CreateBookDto {
            title: "a".repeat(256),
            description: None,
            author_id: Uuid::new_v4(),
            published_date: None,
            isbn: None,
        };

        assert!(dto.validate().is_err());
    }

    #[test]
    fn test_create_book_dto_validation_isbn_too_long() {
        let dto = CreateBookDto {
            title: "Valid Book".to_string(),
            description: None,
            author_id: Uuid::new_v4(),
            published_date: None,
            isbn: Some("a".repeat(21)), // Max is 20
        };

        assert!(dto.validate().is_err());
    }
}
