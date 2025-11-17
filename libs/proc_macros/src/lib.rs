// Re-export proc macros when their features are enabled
#[cfg(feature = "selectable_fields")]
pub use selectable_fields::SelectableFields;

#[cfg(feature = "api_resource")]
pub use api_resource::ApiResource;

/// Trait for REST API resource metadata.
///
/// This trait provides constants for resource URLs, database collection names,
/// and API documentation tags. It is typically derived using the `ApiResource` macro.
///
/// # Examples
///
/// ```ignore
/// use proc_macros::ApiResource;
///
/// #[derive(ApiResource)]
/// pub struct User {
///     id: Uuid,
///     email: String,
/// }
///
/// assert_eq!(User::URL, "/user");
/// assert_eq!(User::COLLECTION, "users");
/// ```
pub trait ApiResource {
    /// The base URL path for this resource (e.g., "/user")
    const URL: &'static str;
    /// The URL path with an ID parameter (e.g., "/user/{id}")
    const URL_WITH_ID: &'static str;
    /// The database collection or table name (e.g., "users")
    const COLLECTION: &'static str;
    /// The API documentation tag (e.g., "Users")
    const TAG: &'static str;
}
