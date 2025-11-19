use crate::errors::{messages, ApiErrorMessage};
#[allow(unused_imports)]
use serde_json::json;
use utoipa::ToResponse;

#[derive(ToResponse)]
#[response(
    description = "Internal Server Error",
    content_type = "application/json",
    example = json!({
        "status": 500,
        "error": "Internal Server Error",
        "message": messages::INTERNAL_ERROR,
        "details": null,
        "code": messages::CODE_INTERNAL
    })
)]
pub struct InternalServerErrorResponse(pub ApiErrorMessage);

#[derive(ToResponse)]
#[response(
    description = "Bad Request - Validation Error",
    content_type = "application/json",
    example = json!({
        "status": 400,
        "error": "Bad Request",
        "message": messages::VALIDATION_FAILED,
        "details": {
            "title": [{
                "code": "length",
                "message": "length is less than 3",
                "params": {"min": 3, "value": "ab"}
            }]
        },
        "code": messages::CODE_VALIDATION
    })
)]
pub struct BadRequestValidationResponse(pub ApiErrorMessage);

#[derive(ToResponse)]
#[response(
    description = "Bad Request - Invalid UUID",
    content_type = "application/json",
    example = json!({
        "status": 400,
        "error": "Bad Request",
        "message": messages::INVALID_UUID,
        "details": null,
        "code": messages::CODE_UUID
    })
)]
pub struct BadRequestUuidResponse(pub ApiErrorMessage);

#[derive(ToResponse)]
#[response(
    description = "Resource not found",
    content_type = "application/json",
    example = json!({
        "status": 404,
        "error": "Not Found",
        "message": messages::NOT_FOUND_RESOURCE,
        "details": null,
        "code": messages::CODE_SQLX_NOT_FOUND
    })
)]
pub struct NotFoundResponse(pub ApiErrorMessage);
