use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth_context::AuthContext,
    dto::author::{AuthorResponseDto, CreateAuthorDto, UpdateAuthorDto},
    entities::{author, prelude::Author},
    error::{AppError, Result},
    state::AppState,
    utils::field_selector::FieldSelector,
};

/// Create a new author
#[utoipa::path(
    post,
    path = "/api/authors",
    request_body = CreateAuthorDto,
    responses(
        (status = 201, description = "Author created successfully", body = AuthorResponseDto),
        (status = 400, description = "Invalid input")
    ),
    tag = "authors"
)]
pub async fn create_author(
    State(state): State<AppState>,
    Json(payload): Json<CreateAuthorDto>,
) -> Result<(StatusCode, Json<AuthorResponseDto>)> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let author = author::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        bio: Set(payload.bio),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    let author = author.insert(state.db()).await?;

    Ok((StatusCode::CREATED, Json(author.into())))
}

/// List all authors with optional field selection
#[utoipa::path(
    get,
    path = "/api/authors",
    params(
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to include (e.g., id,name)")
    ),
    responses(
        (status = 200, description = "List of authors", body = Vec<AuthorResponseDto>)
    ),
    tag = "authors"
)]
pub async fn list_authors(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(field_selector): Query<FieldSelector>,
) -> Result<Json<Value>> {
    let authors = Author::find().all(state.db()).await?;

    let response: Vec<AuthorResponseDto> = authors.into_iter().map(|a| a.into()).collect();

    let filtered = field_selector
        .filter_list_secure(&response, &auth)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    Ok(Json(filtered))
}

/// Get a single author by ID
#[utoipa::path(
    get,
    path = "/api/authors/{id}",
    params(
        ("id" = Uuid, Path, description = "Author ID"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to include")
    ),
    responses(
        (status = 200, description = "Author found", body = AuthorResponseDto),
        (status = 404, description = "Author not found")
    ),
    tag = "authors"
)]
pub async fn get_author(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Query(field_selector): Query<FieldSelector>,
) -> Result<Json<Value>> {
    let author = Author::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or(AppError::NotFound(format!(
            "Author with id {} not found",
            id
        )))?;

    let response: AuthorResponseDto = author.into();

    let filtered = field_selector
        .filter_secure(&response, &auth)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    Ok(Json(filtered))
}

/// Update an author
#[utoipa::path(
    put,
    path = "/api/authors/{id}",
    params(
        ("id" = Uuid, Path, description = "Author ID")
    ),
    request_body = UpdateAuthorDto,
    responses(
        (status = 200, description = "Author updated successfully", body = AuthorResponseDto),
        (status = 404, description = "Author not found"),
        (status = 400, description = "Invalid input")
    ),
    tag = "authors"
)]
pub async fn update_author(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAuthorDto>,
) -> Result<Json<AuthorResponseDto>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let author = Author::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or(AppError::NotFound(format!(
            "Author with id {} not found",
            id
        )))?;

    let mut author: author::ActiveModel = author.into();

    if let Some(name) = payload.name {
        author.name = Set(name);
    }
    if payload.bio.is_some() {
        author.bio = Set(payload.bio);
    }
    author.updated_at = Set(Utc::now().into());

    let author = author.update(state.db()).await?;

    Ok(Json(author.into()))
}

/// Delete an author
#[utoipa::path(
    delete,
    path = "/api/authors/{id}",
    params(
        ("id" = Uuid, Path, description = "Author ID")
    ),
    responses(
        (status = 204, description = "Author deleted successfully"),
        (status = 404, description = "Author not found")
    ),
    tag = "authors"
)]
pub async fn delete_author(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let author = Author::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or(AppError::NotFound(format!(
            "Author with id {} not found",
            id
        )))?;

    let author: author::ActiveModel = author.into();
    author.delete(state.db()).await?;

    Ok(StatusCode::NO_CONTENT)
}
