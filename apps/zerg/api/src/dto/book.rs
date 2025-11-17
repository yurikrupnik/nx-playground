use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::author::AuthorResponseDto;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateBookDto {
    /// Book title
    #[validate(length(min = 1, max = 255))]
    #[schema(example = "Harry Potter and the Philosopher's Stone")]
    pub title: String,
    /// Book description
    #[schema(example = "The first novel in the Harry Potter series")]
    pub description: Option<String>,
    /// Author ID (foreign key)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub author_id: Uuid,
    /// Publication date
    #[schema(example = "1997-06-26")]
    pub published_date: Option<NaiveDate>,
    /// ISBN number
    #[validate(length(max = 20))]
    #[schema(example = "978-0747532699")]
    pub isbn: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateBookDto {
    /// Book title
    #[validate(length(min = 1, max = 255))]
    #[schema(example = "Harry Potter and the Philosopher's Stone")]
    pub title: Option<String>,
    /// Book description
    #[schema(example = "Updated description")]
    pub description: Option<String>,
    /// Author ID (foreign key)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub author_id: Option<Uuid>,
    /// Publication date
    #[schema(example = "1997-06-26")]
    pub published_date: Option<NaiveDate>,
    /// ISBN number
    #[validate(length(max = 20))]
    #[schema(example = "978-0747532699")]
    pub isbn: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, proc_macros::SelectableFields)]
pub struct BookResponseDto {
    /// Unique identifier
    pub id: Uuid,
    /// Book title
    pub title: String,
    /// Book description
    pub description: Option<String>,
    /// Author ID (foreign key reference)
    pub author_id: Uuid,
    /// Publication date
    pub published_date: Option<NaiveDate>,
    /// ISBN number
    pub isbn: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Book with author information included (for joined queries)
#[derive(Debug, Serialize, ToSchema)]
pub struct BookWithAuthorDto {
    /// Unique identifier
    pub id: Uuid,
    /// Book title
    pub title: String,
    /// Book description
    pub description: Option<String>,
    /// Author ID
    pub author_id: Uuid,
    /// Publication date
    pub published_date: Option<NaiveDate>,
    /// ISBN number
    pub isbn: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Author information
    pub author: AuthorResponseDto,
}

impl From<crate::entities::book::Model> for BookResponseDto {
    fn from(book: crate::entities::book::Model) -> Self {
        Self {
            id: book.id,
            title: book.title,
            description: book.description,
            author_id: book.author_id,
            published_date: book.published_date,
            isbn: book.isbn,
            created_at: book.created_at.naive_utc().and_utc(),
            updated_at: book.updated_at.naive_utc().and_utc(),
        }
    }
}

impl From<(crate::entities::book::Model, crate::entities::author::Model)> for BookWithAuthorDto {
    fn from(
        (book, author): (crate::entities::book::Model, crate::entities::author::Model),
    ) -> Self {
        Self {
            id: book.id,
            title: book.title,
            description: book.description,
            author_id: book.author_id,
            published_date: book.published_date,
            isbn: book.isbn,
            created_at: book.created_at.naive_utc().and_utc(),
            updated_at: book.updated_at.naive_utc().and_utc(),
            author: author.into(),
        }
    }
}
