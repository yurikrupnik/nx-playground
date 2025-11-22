use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Car model stored in MongoDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Car {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: String,
    pub created_at: mongodb::bson::DateTime,
    pub updated_at: mongodb::bson::DateTime,
}

/// Car response model for API (uses String for ID for compatibility with OpenAPI)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CarResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Car> for CarResponse {
    fn from(car: Car) -> Self {
        use chrono::{DateTime as ChronoDateTime, Utc};

        Self {
            id: car.id.map(|oid| oid.to_hex()),
            make: car.make,
            model: car.model,
            year: car.year,
            color: car.color,
            created_at: ChronoDateTime::from(car.created_at.to_system_time()),
            updated_at: ChronoDateTime::from(car.updated_at.to_system_time()),
        }
    }
}

/// Request to create a new car
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateCar {
    #[validate(length(min = 1, max = 100))]
    pub make: String,
    #[validate(length(min = 1, max = 100))]
    pub model: String,
    #[validate(range(min = 1900, max = 2100))]
    pub year: i32,
    #[validate(length(min = 1, max = 50))]
    pub color: String,
}

/// Request to update an existing car
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateCar {
    #[validate(length(min = 1, max = 100))]
    pub make: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub model: Option<String>,
    #[validate(range(min = 1900, max = 2100))]
    pub year: Option<i32>,
    #[validate(length(min = 1, max = 50))]
    pub color: Option<String>,
}

