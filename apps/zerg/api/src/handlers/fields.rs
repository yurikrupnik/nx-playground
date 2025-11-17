use axum::Json;
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::{
    auth_context::{AuthContext, UserRole},
    dto::{todo::TodoResponseDto, user::UserResponseDto},
    error::Result,
    utils::field_selector::{FieldAccess, SelectableFields},
};

/// Field information for API documentation
#[derive(Debug, Serialize, ToSchema)]
pub struct FieldInfo {
    /// Field name
    pub name: String,
    /// Minimum role required to access this field
    pub required_role: String,
    /// Whether this field is restricted (never accessible)
    pub is_restricted: bool,
}

/// Available fields response for a resource
#[derive(Debug, Serialize, ToSchema)]
pub struct ResourceFields {
    /// Resource name (e.g., "todos", "users")
    pub resource: String,
    /// List of available fields with access requirements
    pub fields: Vec<FieldInfo>,
    /// Fields accessible to the current user
    pub accessible_fields: Vec<String>,
}

/// Response containing all resources and their fields
#[derive(Debug, Serialize, ToSchema)]
pub struct FieldsResponse {
    /// Map of resource name to field information
    pub resources: HashMap<String, ResourceFields>,
}

/// Get available fields for all resources
/// This endpoint shows which fields can be selected for each resource
/// and indicates the role requirements for field access
#[utoipa::path(
    get,
    path = "/api/fields",
    responses(
        (status = 200, description = "Available fields for all resources", body = FieldsResponse)
    ),
    tag = "meta"
)]
pub async fn list_all_fields(auth: AuthContext) -> Result<Json<FieldsResponse>> {
    let mut resources = HashMap::new();

    // Add todos fields
    resources.insert(
        "todos".to_string(),
        get_resource_fields::<TodoResponseDto>("todos", &auth),
    );

    // Add users fields
    resources.insert(
        "users".to_string(),
        get_resource_fields::<UserResponseDto>("users", &auth),
    );

    Ok(Json(FieldsResponse { resources }))
}

/// Get available fields for todos resource
#[utoipa::path(
    get,
    path = "/api/todos/fields",
    responses(
        (status = 200, description = "Available fields for todos", body = ResourceFields)
    ),
    tag = "todos"
)]
pub async fn list_todo_fields(auth: AuthContext) -> Result<Json<ResourceFields>> {
    Ok(Json(get_resource_fields::<TodoResponseDto>("todos", &auth)))
}

/// Get available fields for users resource
#[utoipa::path(
    get,
    path = "/api/users/fields",
    responses(
        (status = 200, description = "Available fields for users", body = ResourceFields)
    ),
    tag = "users"
)]
pub async fn list_user_fields(auth: AuthContext) -> Result<Json<ResourceFields>> {
    Ok(Json(get_resource_fields::<UserResponseDto>("users", &auth)))
}

/// Helper function to get field information for a resource
fn get_resource_fields<T: SelectableFields>(resource: &str, auth: &AuthContext) -> ResourceFields {
    let restricted_set: std::collections::HashSet<String> = T::restricted_fields()
        .into_iter()
        .map(String::from)
        .collect();

    let field_access: Vec<FieldAccess> = T::field_access();

    let fields: Vec<FieldInfo> = field_access
        .iter()
        .map(|fa| FieldInfo {
            name: fa.field.to_string(),
            required_role: role_to_string(&fa.required_role),
            is_restricted: restricted_set.contains(fa.field),
        })
        .collect();

    // Determine which fields are accessible to current user
    let all_fields: std::collections::HashSet<String> = T::available_fields()
        .into_iter()
        .map(String::from)
        .collect();

    let accessible_fields: Vec<String> = T::filter_by_role(&all_fields, auth).into_iter().collect();

    ResourceFields {
        resource: resource.to_string(),
        fields,
        accessible_fields,
    }
}

/// Convert UserRole to string representation
fn role_to_string(role: &UserRole) -> String {
    match role {
        UserRole::Anonymous => "anonymous".to_string(),
        UserRole::User => "user".to_string(),
        UserRole::Admin => "admin".to_string(),
    }
}
