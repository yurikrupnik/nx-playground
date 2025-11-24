use super::model::{Car, CarResponse, CreateCar, UpdateCar};
use super::state::CarState;
use app::{errors::AppError, extractors::ValidatedJson, responses::*};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::{doc, oid::ObjectId, DateTime as BsonDateTime};

const COLLECTION_NAME: &str = "cars";

#[utoipa::path(
    get,
    path = "/api/cars",
    tag = "Cars",
    responses(
        (status = 200, description = "List of all cars retrieved successfully", body = [CarResponse]),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn get_cars<S: CarState>(
    State(state): State<S>,
) -> Result<Json<Vec<CarResponse>>, AppError> {
    let collection = state.mongo().collection::<Car>(COLLECTION_NAME);

    let mut cursor = collection
        .find(doc! {})
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut cars = Vec::new();
    while cursor
        .advance()
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
    {
        let car = cursor
            .deserialize_current()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        cars.push(car);
    }

    let response: Vec<CarResponse> = cars.into_iter().map(|c| c.into()).collect();
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/car",
    tag = "Cars",
    request_body = CreateCar,
    responses(
        (status = 201, description = "Car created successfully", body = CarResponse),
        (status = 400, response = BadRequestValidationResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn create_car<S: CarState>(
    State(state): State<S>,
    ValidatedJson(body): ValidatedJson<CreateCar>,
) -> Result<(StatusCode, Json<CarResponse>), AppError> {
    let collection = state.mongo().collection::<Car>(COLLECTION_NAME);

    let now = BsonDateTime::now();
    let car = Car {
        id: None,
        make: body.make,
        model: body.model,
        year: body.year,
        color: body.color,
        created_at: now,
        updated_at: now,
    };

    let result = collection
        .insert_one(&car)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let id = result
        .inserted_id
        .as_object_id()
        .ok_or_else(|| AppError::DatabaseError("Failed to get inserted ID".to_string()))?;

    let created_car = Car {
        id: Some(id),
        ..car
    };

    Ok((StatusCode::CREATED, Json(created_car.into())))
}

#[utoipa::path(
    get,
    path = "/api/car/{id}",
    tag = "Cars",
    params(
        ("id", description = "MongoDB ObjectId of the car")
    ),
    responses(
        (status = 200, description = "Car found successfully", body = CarResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn get_car<S: CarState>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<Json<CarResponse>, AppError> {
    let collection = state.mongo().collection::<Car>(COLLECTION_NAME);

    let object_id = ObjectId::parse_str(&id)
        .map_err(|_| AppError::NotFound(format!("Invalid ObjectId: {}", id)))?;

    let car = collection
        .find_one(doc! { "_id": object_id })
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Car with id {} not found", id)))?;

    Ok(Json(car.into()))
}

#[utoipa::path(
    put,
    path = "/api/car/{id}",
    tag = "Cars",
    params(
        ("id", description = "MongoDB ObjectId of the car")
    ),
    request_body = UpdateCar,
    responses(
        (status = 200, description = "Car updated successfully", body = CarResponse),
        (status = 400, response = BadRequestValidationResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn update_car<S: CarState>(
    State(state): State<S>,
    Path(id): Path<String>,
    ValidatedJson(body): ValidatedJson<UpdateCar>,
) -> Result<Json<CarResponse>, AppError> {
    let collection = state.mongo().collection::<Car>(COLLECTION_NAME);

    let object_id = ObjectId::parse_str(&id)
        .map_err(|_| AppError::NotFound(format!("Invalid ObjectId: {}", id)))?;

    let now = BsonDateTime::now();
    let mut update_doc = doc! { "$set": { "updated_at": now } };

    if let Some(make) = body.make {
        update_doc
            .get_document_mut("$set")
            .unwrap()
            .insert("make", make);
    }
    if let Some(model) = body.model {
        update_doc
            .get_document_mut("$set")
            .unwrap()
            .insert("model", model);
    }
    if let Some(year) = body.year {
        update_doc
            .get_document_mut("$set")
            .unwrap()
            .insert("year", year);
    }
    if let Some(color) = body.color {
        update_doc
            .get_document_mut("$set")
            .unwrap()
            .insert("color", color);
    }

    let result = collection
        .find_one_and_update(doc! { "_id": object_id }, update_doc)
        .return_document(mongodb::options::ReturnDocument::After)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Car with id {} not found", id)))?;

    Ok(Json(result.into()))
}

#[utoipa::path(
    delete,
    path = "/api/car/{id}",
    tag = "Cars",
    params(
        ("id", description = "MongoDB ObjectId of the car")
    ),
    responses(
        (status = 200, description = "Car deleted successfully"),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn delete_car<S: CarState>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let collection = state.mongo().collection::<Car>(COLLECTION_NAME);

    let object_id = ObjectId::parse_str(&id)
        .map_err(|_| AppError::NotFound(format!("Invalid ObjectId: {}", id)))?;

    let result = collection
        .delete_one(doc! { "_id": object_id })
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound(format!("Car with id {} not found", id)));
    }

    Ok(StatusCode::OK)
}
