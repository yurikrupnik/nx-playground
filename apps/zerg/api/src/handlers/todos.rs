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
    dto::todo::{CreateTodoDto, TodoResponseDto, UpdateTodoDto},
    entities::{prelude::Todo, todo},
    error::{AppError, Result},
    state::AppState,
    utils::field_selector::FieldSelector,
};

/// Create a new todo
#[utoipa::path(
    post,
    path = "/api/todos",
    request_body = CreateTodoDto,
    responses(
        (status = 201, description = "Todo created successfully", body = TodoResponseDto),
        (status = 400, description = "Invalid input")
    ),
    tag = "todos"
)]
pub async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodoDto>,
) -> Result<(StatusCode, Json<TodoResponseDto>)> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let now = Utc::now().fixed_offset();

    let todo = todo::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        description: Set(payload.description),
        completed: Set(payload.completed.unwrap_or(false)),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let todo = todo.insert(state.db()).await?;

    Ok((StatusCode::CREATED, Json(todo.into())))
}

/// List all todos
/// Supports field selection via query parameter: ?fields=id,name,completed
#[utoipa::path(
    get,
    path = "/api/todos",
    params(
        ("fields" = Option<String>, Query,
         description = "Comma-separated list of fields to return.\n\n**Available fields:**\n- `id` - Unique identifier\n- `name` - Todo name\n- `description` - Optional description\n- `completed` - Completion status\n- `created_at` - Creation timestamp\n- `updated_at` - Last update timestamp\n\n**Examples:**\n- `?fields=id,name` - Only ID and name\n- `?fields=id,name,completed` - ID, name, and status\n- No parameter returns all fields",
         example = "id,name,completed"
        )
    ),
    responses(
        (status = 200, description = "List of todos", body = Vec<TodoResponseDto>),
        (status = 400, description = "Invalid field names requested")
    ),
    tag = "todos"
)]
pub async fn list_todos(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(field_selector): Query<FieldSelector>,
) -> Result<Json<Value>> {
    let todos = Todo::find().all(state.db()).await?;

    let response: Vec<TodoResponseDto> = todos.into_iter().map(|t| t.into()).collect();

    let filtered = field_selector
        .filter_list_secure(&response, &auth)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    Ok(Json(filtered))
}

/// Get a todo by ID
/// Supports field selection via query parameter: ?fields=id,name,completed
#[utoipa::path(
    get,
    path = "/api/todos/{id}",
    params(
        ("id" = Uuid, Path, description = "Todo ID"),
        ("fields" = Option<String>, Query,
         description = "Comma-separated list of fields to return.\n\n**Available fields:** `id`, `name`, `description`, `completed`, `created_at`, `updated_at`",
         example = "id,name,completed"
        )
    ),
    responses(
        (status = 200, description = "Todo found", body = TodoResponseDto),
        (status = 404, description = "Todo not found"),
        (status = 400, description = "Invalid field names requested")
    ),
    tag = "todos"
)]
pub async fn get_todo(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Query(field_selector): Query<FieldSelector>,
) -> Result<Json<Value>> {
    let todo = Todo::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    let dto: TodoResponseDto = todo.into();
    let filtered = field_selector
        .filter_secure(&dto, &auth)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    Ok(Json(filtered))
}

/// Update a todo
#[utoipa::path(
    put,
    path = "/api/todos/{id}",
    params(
        ("id" = Uuid, Path, description = "Todo ID")
    ),
    request_body = UpdateTodoDto,
    responses(
        (status = 200, description = "Todo updated successfully", body = TodoResponseDto),
        (status = 404, description = "Todo not found"),
        (status = 400, description = "Invalid input")
    ),
    tag = "todos"
)]
pub async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTodoDto>,
) -> Result<Json<TodoResponseDto>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let todo = Todo::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    let mut todo: todo::ActiveModel = todo.into();

    if let Some(name) = payload.name {
        todo.name = Set(name);
    }

    if let Some(description) = payload.description {
        todo.description = Set(Some(description));
    }

    if let Some(completed) = payload.completed {
        todo.completed = Set(completed);
    }

    todo.updated_at = Set(Utc::now().fixed_offset());

    let todo = todo.update(state.db()).await?;

    Ok(Json(todo.into()))
}

/// Delete a todo
#[utoipa::path(
    delete,
    path = "/api/todos/{id}",
    params(
        ("id" = Uuid, Path, description = "Todo ID")
    ),
    responses(
        (status = 204, description = "Todo deleted successfully"),
        (status = 404, description = "Todo not found")
    ),
    tag = "todos"
)]
pub async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let todo = Todo::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    let todo: todo::ActiveModel = todo.into();
    todo.delete(state.db()).await?;

    Ok(StatusCode::NO_CONTENT)
}
