use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateAuthorDto {
    /// Author's full name
    #[validate(length(min = 1, max = 255))]
    #[schema(example = "J.K. Rowling")]
    pub name: String,
    /// Author biography
    #[schema(example = "British author, best known for the Harry Potter series")]
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateAuthorDto {
    /// Author's full name
    #[validate(length(min = 1, max = 255))]
    #[schema(example = "J.K. Rowling")]
    pub name: Option<String>,
    /// Author biography
    #[schema(example = "British author and philanthropist")]
    pub bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, proc_macros::SelectableFields)]
pub struct AuthorResponseDto {
    /// Unique identifier
    pub id: Uuid,
    /// Author's name
    pub name: String,
    /// Author biography
    pub bio: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl From<crate::entities::author::Model> for AuthorResponseDto {
    fn from(author: crate::entities::author::Model) -> Self {
        Self {
            id: author.id,
            name: author.name,
            bio: author.bio,
            created_at: author.created_at.naive_utc().and_utc(),
            updated_at: author.updated_at.naive_utc().and_utc(),
        }
    }
}
