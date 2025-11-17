use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
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
    dto::user::{CreateUserDto, UpdateUserDto, UserResponseDto},
    entities::{prelude::User, user},
    error::{AppError, Result},
    state::AppState,
    utils::field_selector::FieldSelector,
};

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created successfully", body = UserResponseDto),
        (status = 400, description = "Invalid input")
    ),
    tag = "users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> Result<(StatusCode, Json<UserResponseDto>)> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();

    let now = Utc::now().fixed_offset();

    let user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(payload.username),
        email: Set(payload.email),
        password_hash: Set(password_hash),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let user = user.insert(state.db()).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}

/// List all users
/// Supports field selection via query parameter: ?fields=id,username,email
#[utoipa::path(
    get,
    path = "/api/users",
    params(
        ("fields" = Option<String>, Query,
         description = "Comma-separated list of fields to return.\n\n**Available fields:**\n- `id` - Unique identifier (public)\n- `username` - Username (public)\n- `email` - Email address (public)\n- `created_at` - Creation timestamp (public)\n- `updated_at` - Last update timestamp (public)\n\n**Examples:**\n- `?fields=id,username` - Only ID and username\n- `?fields=id,username,email` - Include email\n- No parameter returns all accessible fields",
         example = "id,username,email"
        )
    ),
    responses(
        (status = 200, description = "List of users", body = Vec<UserResponseDto>),
        (status = 400, description = "Invalid field names requested")
    ),
    tag = "users"
)]
pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(field_selector): Query<FieldSelector>,
) -> Result<Json<Value>> {
    let users = User::find().all(state.db()).await?;

    let response: Vec<UserResponseDto> = users.into_iter().map(|u| u.into()).collect();

    let filtered = field_selector
        .filter_list_secure(&response, &auth)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    Ok(Json(filtered))
}

/// Get a user by ID
/// Supports field selection via query parameter: ?fields=id,username,email
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID"),
        ("fields" = Option<String>, Query,
         description = "Comma-separated list of fields to return.\n\n**Available fields:** `id`, `username`, `email`, `created_at`, `updated_at`\n\n**Note:** All fields are currently public. Use headers `x-user-role: user` for authenticated access.",
         example = "id,username,email"
        )
    ),
    responses(
        (status = 200, description = "User found", body = UserResponseDto),
        (status = 404, description = "User not found"),
        (status = 400, description = "Invalid field names requested")
    ),
    tag = "users"
)]
pub async fn get_user(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Query(field_selector): Query<FieldSelector>,
) -> Result<Json<Value>> {
    let user = User::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", id)))?;

    let dto: UserResponseDto = user.into();
    let filtered = field_selector
        .filter_secure(&dto, &auth)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    Ok(Json(filtered))
}

/// Update a user
#[utoipa::path(
    put,
    path = "/api/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponseDto),
        (status = 404, description = "User not found"),
        (status = 400, description = "Invalid input")
    ),
    tag = "users"
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserDto>,
) -> Result<Json<UserResponseDto>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = User::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", id)))?;

    let mut user: user::ActiveModel = user.into();

    if let Some(username) = payload.username {
        user.username = Set(username);
    }

    if let Some(email) = payload.email {
        user.email = Set(email);
    }

    if let Some(password) = payload.password {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
            .to_string();
        user.password_hash = Set(password_hash);
    }

    user.updated_at = Set(Utc::now().fixed_offset());

    let user = user.update(state.db()).await?;

    Ok(Json(user.into()))
}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 404, description = "User not found")
    ),
    tag = "users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let user = User::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", id)))?;

    let user: user::ActiveModel = user.into();
    user.delete(state.db()).await?;

    Ok(StatusCode::NO_CONTENT)
}
