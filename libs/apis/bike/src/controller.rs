use super::model::{Bike, BikeResponse, CreateBike, UpdateBike};
use super::state::BikeState;
use app::{errors::AppError, extractors::ValidatedJson, responses::*};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

/// Initialize the bike table.
/// This should be called once during application startup or in tests.
pub async fn init_bikes_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS bikes (
            id UUID PRIMARY KEY,
            brand VARCHAR(100) NOT NULL,
            model VARCHAR(100) NOT NULL,
            bike_type VARCHAR(50) NOT NULL,
            frame_size INTEGER NOT NULL CHECK (frame_size >= 30 AND frame_size <= 70),
            color VARCHAR(50) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/bikes",
    tag = "Bikes",
    responses(
        (status = 200, description = "List of all bikes retrieved successfully", body = [BikeResponse]),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn get_bikes<S: BikeState>(
    State(state): State<S>,
) -> Result<Json<Vec<BikeResponse>>, AppError> {
    let pool = state.sqlx_pool();

    let bikes = sqlx::query_as::<_, Bike>("SELECT * FROM bikes ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let response: Vec<BikeResponse> = bikes.into_iter().map(|b| b.into()).collect();
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/bike",
    tag = "Bikes",
    request_body = CreateBike,
    responses(
        (status = 201, description = "Bike created successfully", body = BikeResponse),
        (status = 400, response = BadRequestValidationResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn create_bike<S: BikeState>(
    State(state): State<S>,
    ValidatedJson(body): ValidatedJson<CreateBike>,
) -> Result<(StatusCode, Json<BikeResponse>), AppError> {
    let pool = state.sqlx_pool();

    let id = Uuid::new_v4();
    let now = Utc::now();

    let bike = sqlx::query_as::<_, Bike>(
        r#"
        INSERT INTO bikes (id, brand, model, bike_type, frame_size, color, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&body.brand)
    .bind(&body.model)
    .bind(&body.bike_type)
    .bind(body.frame_size)
    .bind(&body.color)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(bike.into())))
}

#[utoipa::path(
    get,
    path = "/api/bike/{id}",
    tag = "Bikes",
    params(
        ("id", description = "UUID of the bike")
    ),
    responses(
        (status = 200, description = "Bike found successfully", body = BikeResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn get_bike<S: BikeState>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<Json<BikeResponse>, AppError> {
    let pool = state.sqlx_pool();

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound(format!("Invalid UUID: {}", id)))?;

    let bike = sqlx::query_as::<_, Bike>("SELECT * FROM bikes WHERE id = $1")
        .bind(uuid)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Bike with id {} not found", id)))?;

    Ok(Json(bike.into()))
}

#[utoipa::path(
    put,
    path = "/api/bike/{id}",
    tag = "Bikes",
    params(
        ("id", description = "UUID of the bike")
    ),
    request_body = UpdateBike,
    responses(
        (status = 200, description = "Bike updated successfully", body = BikeResponse),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn update_bike<S: BikeState>(
    State(state): State<S>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateBike>,
) -> Result<Json<BikeResponse>, AppError> {
    let pool = state.sqlx_pool();

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound(format!("Invalid UUID: {}", id)))?;

    // Check if bike exists
    let existing = sqlx::query_as::<_, Bike>("SELECT * FROM bikes WHERE id = $1")
        .bind(uuid)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Bike with id {} not found", id)))?;

    // Build update query dynamically based on provided fields
    let brand = body.brand.unwrap_or(existing.brand);
    let model = body.model.unwrap_or(existing.model);
    let bike_type = body.bike_type.unwrap_or(existing.bike_type);
    let frame_size = body.frame_size.unwrap_or(existing.frame_size);
    let color = body.color.unwrap_or(existing.color);
    let now = Utc::now();

    let bike = sqlx::query_as::<_, Bike>(
        r#"
        UPDATE bikes
        SET brand = $2, model = $3, bike_type = $4, frame_size = $5, color = $6, updated_at = $7
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(uuid)
    .bind(brand)
    .bind(model)
    .bind(bike_type)
    .bind(frame_size)
    .bind(color)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(bike.into()))
}

#[utoipa::path(
    delete,
    path = "/api/bike/{id}",
    tag = "Bikes",
    params(
        ("id", description = "UUID of the bike")
    ),
    responses(
        (status = 200, description = "Bike deleted successfully"),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn delete_bike<S: BikeState>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let pool = state.sqlx_pool();

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::NotFound(format!("Invalid UUID: {}", id)))?;

    let result = sqlx::query("DELETE FROM bikes WHERE id = $1")
        .bind(uuid)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Bike with id {} not found", id)));
    }

    Ok(StatusCode::OK)
}
