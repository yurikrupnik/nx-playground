use chrono::{DateTime, Utc};
use proc_macros::SeaOrmResource;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use validator::Validate;

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, Default, SeaOrmResource,
)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
#[typeshare]
pub struct UpdateProject {
    /// Title of the project
    #[validate(length(min = 2))]
    pub title: Option<String>,
    /// Description of the project
    #[validate(length(min = 4))]
    pub description: Option<String>,
    /// Completed status of the project
    pub completed: Option<bool>,
}

#[derive(Debug, Validate, Deserialize, Serialize, ToSchema)]
#[typeshare]
pub struct CreateProject {
    #[validate(length(min = 2))]
    pub title: String,
    #[validate(length(min = 4))]
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, proc_macros::SelectableFields)]
pub struct ProjectResponse {
    /// Unique identifier
    pub id: i64,
    /// Project title
    pub title: String,
    /// Project description
    pub description: String,
    /// Project state
    pub completed: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl From<Model> for ProjectResponse {
    fn from(item: Model) -> Self {
        Self {
            id: item.id,
            title: item.title,
            description: item.description,
            completed: item.completed,
            created_at: item.created_at.naive_utc().and_utc(),
            updated_at: item.updated_at.naive_utc().and_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_project_valid() {
        let project = CreateProject {
            title: "Valid Title".to_string(),
            description: "Valid description that is long enough".to_string(),
            completed: false,
        };

        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_create_project_title_too_short() {
        let project = CreateProject {
            title: "A".to_string(),
            description: "Valid description".to_string(),
            completed: false,
        };

        let result = project.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("title"));
    }

    #[test]
    fn test_create_project_title_minimum_length() {
        let project = CreateProject {
            title: "AB".to_string(),
            description: "Valid description".to_string(),
            completed: false,
        };

        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_create_project_description_too_short() {
        let project = CreateProject {
            title: "Valid Title".to_string(),
            description: "abc".to_string(),
            completed: false,
        };

        let result = project.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("description"));
    }

    #[test]
    fn test_create_project_description_minimum_length() {
        let project = CreateProject {
            title: "Valid Title".to_string(),
            description: "abcd".to_string(),
            completed: false,
        };

        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_create_project_both_fields_invalid() {
        let project = CreateProject {
            title: "A".to_string(),
            description: "ab".to_string(),
            completed: false,
        };

        let result = project.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("title"));
        assert!(errors.field_errors().contains_key("description"));
    }

    #[test]
    fn test_update_project_valid_all_fields() {
        let update = UpdateProject {
            title: Some("Updated Title".to_string()),
            description: Some("Updated description".to_string()),
            completed: Some(true),
        };

        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_update_project_valid_partial() {
        let update = UpdateProject {
            title: Some("Updated".to_string()),
            description: None,
            completed: Some(true),
        };

        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_update_project_title_too_short() {
        let update = UpdateProject {
            title: Some("A".to_string()),
            description: None,
            completed: None,
        };

        let result = update.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("title"));
    }

    #[test]
    fn test_update_project_all_none() {
        let update = UpdateProject {
            title: None,
            description: None,
            completed: None,
        };

        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_update_project_description_too_short() {
        let update = UpdateProject {
            title: None,
            description: Some("abc".to_string()),
            completed: None,
        };

        let result = update.validate();
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.field_errors().contains_key("description"));
    }

    #[test]
    fn test_update_project_description_minimum_length() {
        let update = UpdateProject {
            title: None,
            description: Some("abcd".to_string()),
            completed: None,
        };

        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_update_project_only_description() {
        let update = UpdateProject {
            title: None,
            description: Some("Valid description".to_string()),
            completed: None,
        };

        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_update_project_only_completed() {
        let update = UpdateProject {
            title: None,
            description: None,
            completed: Some(true),
        };

        assert!(update.validate().is_ok());
    }

    #[test]
    fn test_project_response_from_model() {
        use chrono::TimeZone;

        let model = Model {
            id: 1,
            title: "Test Project".to_string(),
            description: "Test Description".to_string(),
            completed: true,
            created_at: Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap().into(),
            updated_at: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap().into(),
        };

        let response: ProjectResponse = model.clone().into();

        assert_eq!(response.id, model.id);
        assert_eq!(response.title, model.title);
        assert_eq!(response.description, model.description);
        assert_eq!(response.completed, model.completed);
    }

    #[test]
    fn test_model_default() {
        let model = Model::default();

        assert_eq!(model.id, 0);
        assert_eq!(model.title, "");
        assert_eq!(model.description, "");
        assert!(!model.completed);
    }
}
