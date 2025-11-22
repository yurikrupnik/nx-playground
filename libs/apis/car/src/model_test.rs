#[cfg(test)]
mod tests {
    use super::super::model::*;
    use mongodb::bson::oid::ObjectId;

    #[test]
    fn test_create_car_validation() {
        let valid_car = CreateCar {
            make: "Toyota".to_string(),
            model: "Camry".to_string(),
            year: 2024,
            color: "Blue".to_string(),
        };

        // Should pass validation
        use validator::Validate;
        assert!(valid_car.validate().is_ok());
    }

    #[test]
    fn test_create_car_validation_fails_empty_make() {
        let invalid_car = CreateCar {
            make: "".to_string(),
            model: "Camry".to_string(),
            year: 2024,
            color: "Blue".to_string(),
        };

        use validator::Validate;
        assert!(invalid_car.validate().is_err());
    }

    #[test]
    fn test_create_car_validation_fails_invalid_year() {
        let invalid_car = CreateCar {
            make: "Toyota".to_string(),
            model: "Camry".to_string(),
            year: 1800, // Too old
            color: "Blue".to_string(),
        };

        use validator::Validate;
        assert!(invalid_car.validate().is_err());
    }

    #[test]
    fn test_car_to_response_conversion() {
        use mongodb::bson::DateTime as BsonDateTime;

        let now = BsonDateTime::now();
        let car = Car {
            id: Some(ObjectId::new()),
            make: "Honda".to_string(),
            model: "Civic".to_string(),
            year: 2023,
            color: "Red".to_string(),
            created_at: now,
            updated_at: now,
        };

        let response: CarResponse = car.clone().into();

        assert_eq!(response.make, car.make);
        assert_eq!(response.model, car.model);
        assert_eq!(response.year, car.year);
        assert_eq!(response.color, car.color);
        assert!(response.id.is_some());
        assert_eq!(response.id.unwrap(), car.id.unwrap().to_hex());
    }

    #[test]
    fn test_update_car_partial_fields() {
        let update = UpdateCar {
            make: Some("Tesla".to_string()),
            model: None,
            year: Some(2024),
            color: None,
        };

        use validator::Validate;
        assert!(update.validate().is_ok());
        assert!(update.make.is_some());
        assert!(update.model.is_none());
    }

    #[test]
    fn test_car_response_serialization() {
        use chrono::Utc;

        let response = CarResponse {
            id: Some("507f1f77bcf86cd799439011".to_string()),
            make: "Ford".to_string(),
            model: "Mustang".to_string(),
            year: 2022,
            color: "Yellow".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Should serialize to JSON successfully
        let json = serde_json::to_string(&response);
        assert!(json.is_ok());

        // Should deserialize back
        let deserialized: Result<CarResponse, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
}
