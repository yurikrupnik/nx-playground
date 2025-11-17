use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, RelationTrait, Set,
};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth_context::AuthContext,
    dto::book::{BookResponseDto, BookWithAuthorDto, CreateBookDto, UpdateBookDto},
    entities::{
        book,
        prelude::{Author, Book},
    },
    error::{AppError, Result},
    state::AppState,
    utils::field_selector::FieldSelector,
};

/// Create a new book
#[utoipa::path(
    post,
    path = "/api/books",
    request_body = CreateBookDto,
    responses(
        (status = 201, description = "Book created successfully", body = BookResponseDto),
        (status = 400, description = "Invalid input"),
        (status = 404, description = "Author not found")
    ),
    tag = "books"
)]
pub async fn create_book(
    State(state): State<AppState>,
    Json(payload): Json<CreateBookDto>,
) -> Result<(StatusCode, Json<BookResponseDto>)> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Verify author exists
    let author_exists = Author::find_by_id(payload.author_id)
        .one(state.db())
        .await?
        .is_some();

    if !author_exists {
        return Err(AppError::NotFound(format!(
            "Author with id {} not found",
            payload.author_id
        )));
    }

    let book = book::ActiveModel {
        id: Set(Uuid::new_v4()),
        title: Set(payload.title),
        description: Set(payload.description),
        author_id: Set(payload.author_id),
        published_date: Set(payload.published_date),
        isbn: Set(payload.isbn),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    let book = book.insert(state.db()).await?;

    Ok((StatusCode::CREATED, Json(book.into())))
}

/// List all books with optional field selection
#[utoipa::path(
    get,
    path = "/api/books",
    params(
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to include (e.g., id,title,author_id)"),
        ("author_id" = Option<Uuid>, Query, description = "Filter by author ID")
    ),
    responses(
        (status = 200, description = "List of books", body = Vec<BookResponseDto>)
    ),
    tag = "books"
)]
pub async fn list_books(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(params): Query<ListBooksParams>,
) -> Result<Json<Value>> {
    let mut query = Book::find();

    // Filter by author if specified
    if let Some(author_id) = params.author_id {
        query = query.filter(book::Column::AuthorId.eq(author_id));
    }

    let books = query.all(state.db()).await?;

    let response: Vec<BookResponseDto> = books.into_iter().map(|b| b.into()).collect();

    let filtered = params
        .field_selector
        .filter_list_secure(&response, &auth)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    Ok(Json(filtered))
}

#[derive(serde::Deserialize)]
pub struct ListBooksParams {
    #[serde(flatten)]
    pub field_selector: FieldSelector,
    pub author_id: Option<Uuid>,
}

/// List all books WITH their author information (joined query)
#[utoipa::path(
    get,
    path = "/api/books/with-authors",
    params(
        ("author_id" = Option<Uuid>, Query, description = "Filter by author ID")
    ),
    responses(
        (status = 200, description = "List of books with author details", body = Vec<BookWithAuthorDto>)
    ),
    tag = "books"
)]
pub async fn list_books_with_authors(
    State(state): State<AppState>,
    Query(params): Query<AuthorFilterParams>,
) -> Result<Json<Vec<BookWithAuthorDto>>> {
    use crate::entities::book;
    use sea_orm::JoinType;

    let mut query = Book::find()
        .find_also_related(Author)
        .join(JoinType::InnerJoin, book::Relation::Author.def());

    // Filter by author if specified
    if let Some(author_id) = params.author_id {
        query = query.filter(book::Column::AuthorId.eq(author_id));
    }

    let books_with_authors = query.all(state.db()).await?;

    let response: Vec<BookWithAuthorDto> = books_with_authors
        .into_iter()
        .filter_map(|(book, author_opt)| {
            author_opt.map(|author| BookWithAuthorDto::from((book, author)))
        })
        .collect();

    Ok(Json(response))
}

#[derive(serde::Deserialize)]
pub struct AuthorFilterParams {
    pub author_id: Option<Uuid>,
}

/// Get a single book by ID
#[utoipa::path(
    get,
    path = "/api/books/{id}",
    params(
        ("id" = Uuid, Path, description = "Book ID"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to include")
    ),
    responses(
        (status = 200, description = "Book found", body = BookResponseDto),
        (status = 404, description = "Book not found")
    ),
    tag = "books"
)]
pub async fn get_book(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Query(field_selector): Query<FieldSelector>,
) -> Result<Json<Value>> {
    let book = Book::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or(AppError::NotFound(format!("Book with id {} not found", id)))?;

    let response: BookResponseDto = book.into();

    let filtered = field_selector
        .filter_secure(&response, &auth)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    Ok(Json(filtered))
}

/// Get a single book WITH author information
#[utoipa::path(
    get,
    path = "/api/books/{id}/with-author",
    params(
        ("id" = Uuid, Path, description = "Book ID")
    ),
    responses(
        (status = 200, description = "Book with author details", body = BookWithAuthorDto),
        (status = 404, description = "Book not found")
    ),
    tag = "books"
)]
pub async fn get_book_with_author(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BookWithAuthorDto>> {
    let (book, author) = Book::find_by_id(id)
        .find_also_related(Author)
        .one(state.db())
        .await?
        .ok_or(AppError::NotFound(format!("Book with id {} not found", id)))?;

    let author = author.ok_or(AppError::Internal(
        "Book has no associated author (data integrity issue)".to_string(),
    ))?;

    Ok(Json(BookWithAuthorDto::from((book, author))))
}

/// Update a book
#[utoipa::path(
    put,
    path = "/api/books/{id}",
    params(
        ("id" = Uuid, Path, description = "Book ID")
    ),
    request_body = UpdateBookDto,
    responses(
        (status = 200, description = "Book updated successfully", body = BookResponseDto),
        (status = 404, description = "Book or author not found"),
        (status = 400, description = "Invalid input")
    ),
    tag = "books"
)]
pub async fn update_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBookDto>,
) -> Result<Json<BookResponseDto>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let book = Book::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or(AppError::NotFound(format!("Book with id {} not found", id)))?;

    let mut book: book::ActiveModel = book.into();

    if let Some(title) = payload.title {
        book.title = Set(title);
    }
    if payload.description.is_some() {
        book.description = Set(payload.description);
    }
    if let Some(author_id) = payload.author_id {
        // Verify new author exists
        let author_exists = Author::find_by_id(author_id)
            .one(state.db())
            .await?
            .is_some();

        if !author_exists {
            return Err(AppError::NotFound(format!(
                "Author with id {} not found",
                author_id
            )));
        }

        book.author_id = Set(author_id);
    }
    if payload.published_date.is_some() {
        book.published_date = Set(payload.published_date);
    }
    if payload.isbn.is_some() {
        book.isbn = Set(payload.isbn);
    }
    book.updated_at = Set(Utc::now().into());

    let book = book.update(state.db()).await?;

    Ok(Json(book.into()))
}

/// Delete a book
#[utoipa::path(
    delete,
    path = "/api/books/{id}",
    params(
        ("id" = Uuid, Path, description = "Book ID")
    ),
    responses(
        (status = 204, description = "Book deleted successfully"),
        (status = 404, description = "Book not found")
    ),
    tag = "books"
)]
pub async fn delete_book(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let book = Book::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or(AppError::NotFound(format!("Book with id {} not found", id)))?;

    let book: book::ActiveModel = book.into();
    book.delete(state.db()).await?;

    Ok(StatusCode::NO_CONTENT)
}
