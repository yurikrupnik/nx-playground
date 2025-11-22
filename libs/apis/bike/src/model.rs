use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bike {
    pub id: Uuid,
    pub brand: String,
    pub model: String,
    pub bike_type: String,
    pub frame_size: i32,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateBike {
    #[validate(length(min = 1, max = 100, message = "Brand must be between 1 and 100 characters"))]
    pub brand: String,

    #[validate(length(min = 1, max = 100, message = "Model must be between 1 and 100 characters"))]
    pub model: String,

    #[validate(length(min = 1, max = 50, message = "Type must be between 1 and 50 characters"))]
    pub bike_type: String,

    #[validate(range(min = 30, max = 70, message = "Frame size must be between 30 and 70 cm"))]
    pub frame_size: i32,

    #[validate(length(min = 1, max = 50, message = "Color must be between 1 and 50 characters"))]
    pub color: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateBike {
    #[validate(length(min = 1, max = 100, message = "Brand must be between 1 and 100 characters"))]
    pub brand: Option<String>,

    #[validate(length(min = 1, max = 100, message = "Model must be between 1 and 100 characters"))]
    pub model: Option<String>,

    #[validate(length(min = 1, max = 50, message = "Type must be between 1 and 50 characters"))]
    pub bike_type: Option<String>,

    #[validate(range(min = 30, max = 70, message = "Frame size must be between 30 and 70 cm"))]
    pub frame_size: Option<i32>,

    #[validate(length(min = 1, max = 50, message = "Color must be between 1 and 50 characters"))]
    pub color: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BikeResponse {
    pub id: String,
    pub brand: String,
    pub model: String,
    #[serde(rename = "type")]
    pub bike_type: String,
    pub frame_size: i32,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Bike> for BikeResponse {
    fn from(bike: Bike) -> Self {
        Self {
            id: bike.id.to_string(),
            brand: bike.brand,
            model: bike.model,
            bike_type: bike.bike_type,
            frame_size: bike.frame_size,
            color: bike.color,
            created_at: bike.created_at,
            updated_at: bike.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_bike_validation_success() {
        let bike = CreateBike {
            brand: "Trek".to_string(),
            model: "Domane SL 7".to_string(),
            bike_type: "Road".to_string(),
            frame_size: 56,
            color: "Black".to_string(),
        };

        assert!(bike.validate().is_ok());
    }

    #[test]
    fn test_create_bike_validation_fails_empty_brand() {
        let bike = CreateBike {
            brand: "".to_string(),
            model: "Domane SL 7".to_string(),
            bike_type: "Road".to_string(),
            frame_size: 56,
            color: "Black".to_string(),
        };

        assert!(bike.validate().is_err());
    }

    #[test]
    fn test_create_bike_validation_fails_invalid_frame_size() {
        let bike = CreateBike {
            brand: "Trek".to_string(),
            model: "Domane SL 7".to_string(),
            bike_type: "Road".to_string(),
            frame_size: 100,
            color: "Black".to_string(),
        };

        assert!(bike.validate().is_err());
    }

    #[test]
    fn test_bike_to_response_conversion() {
        let bike = Bike {
            id: Uuid::new_v4(),
            brand: "Specialized".to_string(),
            model: "Roubaix".to_string(),
            bike_type: "Road".to_string(),
            frame_size: 58,
            color: "Red".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response: BikeResponse = bike.clone().into();

        assert_eq!(response.id, bike.id.to_string());
        assert_eq!(response.brand, bike.brand);
        assert_eq!(response.model, bike.model);
        assert_eq!(response.bike_type, bike.bike_type);
        assert_eq!(response.frame_size, bike.frame_size);
        assert_eq!(response.color, bike.color);
    }

    #[test]
    fn test_update_bike_partial_fields() {
        let update = UpdateBike {
            brand: Some("Giant".to_string()),
            model: None,
            bike_type: None,
            frame_size: Some(54),
            color: None,
        };

        assert!(update.validate().is_ok());
        assert!(update.brand.is_some());
        assert!(update.model.is_none());
    }
}
