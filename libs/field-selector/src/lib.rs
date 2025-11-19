//! Field selector library for dynamic field selection with role-based access control.
//!
//! This library provides types and traits for implementing field selection
//! in REST APIs with security features like role-based access control.
//!
//! # Features
//!
//! - `axum` - Enables Axum integration with `FromRequestParts` extractor

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// Re-export for convenience
pub use serde;
pub use serde_json;

/// User roles for authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    /// Anonymous/unauthenticated user
    Anonymous,
    /// Regular authenticated user
    User,
    /// Administrator with elevated privileges
    Admin,
}

impl UserRole {
    /// Check if this role has at least the required role level
    pub fn has_permission(&self, required: &UserRole) -> bool {
        matches!(
            (self, required),
            (UserRole::Admin, _)
                | (UserRole::User, UserRole::User)
                | (UserRole::User, UserRole::Anonymous)
                | (UserRole::Anonymous, UserRole::Anonymous)
        )
    }
}

/// Authentication context containing user information
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// User ID (None for anonymous)
    pub user_id: Option<Uuid>,
    /// User role
    pub role: UserRole,
    /// Username (None for anonymous)
    pub username: Option<String>,
}

impl AuthContext {
    /// Create an anonymous context (no authentication)
    pub fn anonymous() -> Self {
        Self {
            user_id: None,
            role: UserRole::Anonymous,
            username: None,
        }
    }

    /// Create an authenticated user context
    pub fn user(user_id: Uuid, username: String) -> Self {
        Self {
            user_id: Some(user_id),
            role: UserRole::User,
            username: Some(username),
        }
    }

    /// Create an admin context
    pub fn admin(user_id: Uuid, username: String) -> Self {
        Self {
            user_id: Some(user_id),
            role: UserRole::Admin,
            username: Some(username),
        }
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Check if user has required role
    pub fn has_role(&self, required: &UserRole) -> bool {
        self.role.has_permission(required)
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::anonymous()
    }
}

/// Field access level configuration
#[derive(Debug, Clone)]
pub struct FieldAccess {
    /// Field name
    pub field: &'static str,
    /// Minimum role required to access this field
    pub required_role: UserRole,
}

/// Trait for DTOs that support field selection with security
/// Implement this to get compile-time field validation and role-based access control
pub trait SelectableFields: Serialize {
    /// Get all available field names for this type
    fn available_fields() -> Vec<&'static str>;

    /// Get fields that should NEVER be exposed (blacklist)
    /// These fields will be filtered out regardless of request
    fn restricted_fields() -> Vec<&'static str> {
        vec![]
    }

    /// Get role-based field access configuration
    /// By default, all fields require Anonymous role (accessible to everyone)
    fn field_access() -> Vec<FieldAccess> {
        Self::available_fields()
            .into_iter()
            .map(|field| FieldAccess {
                field,
                required_role: UserRole::Anonymous,
            })
            .collect()
    }

    /// Validate that requested fields are valid
    fn validate_fields(fields: &HashSet<String>) -> Result<(), Vec<String>> {
        let available: HashSet<String> = Self::available_fields()
            .into_iter()
            .map(String::from)
            .collect();

        let invalid: Vec<String> = fields
            .iter()
            .filter(|f| !available.contains(*f))
            .cloned()
            .collect();

        if invalid.is_empty() {
            Ok(())
        } else {
            Err(invalid)
        }
    }

    /// Filter fields based on user role and restrictions
    fn filter_by_role(fields: &HashSet<String>, auth: &AuthContext) -> HashSet<String> {
        let restricted: HashSet<String> = Self::restricted_fields()
            .into_iter()
            .map(String::from)
            .collect();

        let access_map: HashMap<String, UserRole> = Self::field_access()
            .into_iter()
            .map(|fa| (fa.field.to_string(), fa.required_role))
            .collect();

        fields
            .iter()
            .filter(|field| {
                // Filter out restricted fields
                if restricted.contains(*field) {
                    tracing::warn!(
                        field = field.as_str(),
                        "Attempted access to restricted field"
                    );
                    return false;
                }

                // Check role-based access
                if let Some(required_role) = access_map.get(*field) {
                    if !auth.has_role(required_role) {
                        tracing::warn!(
                            field = field.as_str(),
                            user_role = ?auth.role,
                            required_role = ?required_role,
                            "Insufficient permissions for field access"
                        );
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }
}

/// Errors that can occur during field selection
#[derive(Debug, thiserror::Error)]
pub enum FieldSelectionError {
    #[error("Invalid fields requested: {0:?}")]
    InvalidFields(Vec<String>),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Query parameter extractor for field selection
/// Usage: GET /api/todos?fields=id,name,completed
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FieldSelector {
    #[serde(default)]
    pub fields: Option<String>,
}

impl FieldSelector {
    /// Get the set of requested fields
    pub fn get_fields(&self) -> Option<HashSet<String>> {
        self.fields.as_ref().map(|f| {
            f.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
    }

    /// Check if a specific field is requested
    pub fn includes(&self, field: &str) -> bool {
        match self.get_fields() {
            Some(fields) => fields.contains(field),
            None => true, // If no fields specified, include all
        }
    }

    /// Resolve which fields to include based on request and auth context
    fn resolve_fields<T>(&self, auth: &AuthContext) -> Result<HashSet<String>, FieldSelectionError>
    where
        T: SelectableFields,
    {
        match self.get_fields() {
            Some(ref fields) => {
                // Validate that requested fields exist
                T::validate_fields(fields).map_err(FieldSelectionError::InvalidFields)?;
                // Filter by role and restrictions
                Ok(T::filter_by_role(fields, auth))
            }
            None => {
                // Return all fields the user has access to
                let all_fields: HashSet<String> = T::available_fields()
                    .into_iter()
                    .map(String::from)
                    .collect();
                Ok(T::filter_by_role(&all_fields, auth))
            }
        }
    }

    /// Securely filter a serializable value with validation and role-based access control
    pub fn filter_secure<T>(
        &self,
        value: &T,
        auth: &AuthContext,
    ) -> Result<Value, FieldSelectionError>
    where
        T: Serialize + SelectableFields,
    {
        let fields_to_include = self.resolve_fields::<T>(auth)?;

        // Log field access for audit
        if let Some(ref requested) = self.get_fields() {
            tracing::info!(
                user_id = ?auth.user_id,
                user_role = ?auth.role,
                requested_fields = ?requested,
                allowed_fields = ?fields_to_include,
                "Field selection applied"
            );
        }

        // Serialize and filter
        let json_value = serde_json::to_value(value)
            .map_err(|e| FieldSelectionError::SerializationError(e.to_string()))?;

        match json_value {
            Value::Object(obj) => Ok(Value::Object(filter_object(obj, &fields_to_include))),
            value => Ok(value),
        }
    }

    /// Securely filter a list of serializable values
    pub fn filter_list_secure<T>(
        &self,
        values: &[T],
        auth: &AuthContext,
    ) -> Result<Value, FieldSelectionError>
    where
        T: Serialize + SelectableFields,
    {
        let fields_to_include = self.resolve_fields::<T>(auth)?;

        // Log field access for audit
        if let Some(ref requested) = self.get_fields() {
            tracing::info!(
                user_id = ?auth.user_id,
                user_role = ?auth.role,
                requested_fields = ?requested,
                allowed_fields = ?fields_to_include,
                count = values.len(),
                "Field selection applied to list"
            );
        }

        let filtered: Result<Vec<Value>, _> = values
            .iter()
            .map(|v| {
                let json_value = serde_json::to_value(v)
                    .map_err(|e| FieldSelectionError::SerializationError(e.to_string()))?;
                match json_value {
                    Value::Object(obj) => Ok(Value::Object(filter_object(obj, &fields_to_include))),
                    value => Ok(value),
                }
            })
            .collect();

        Ok(Value::Array(filtered?))
    }
}

/// Helper function to filter JSON object by field names
fn filter_object(obj: Map<String, Value>, fields: &HashSet<String>) -> Map<String, Value> {
    obj.into_iter()
        .filter(|(k, _)| fields.contains(k))
        .collect()
}

// Axum integration - only available with the "axum" feature
#[cfg(feature = "axum")]
mod axum_integration {
    use super::*;
    use axum::{extract::FromRequestParts, http::request::Parts};

    /// Extractor for AuthContext from request
    impl<S> FromRequestParts<S> for AuthContext
    where
        S: Send + Sync,
    {
        type Rejection = std::convert::Infallible;

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            // Extract from headers for testing/development
            // TODO: Implement proper authentication (JWT, session, etc.)
            let role = parts
                .headers
                .get("x-user-role")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| match s.to_lowercase().as_str() {
                    "admin" => Some(UserRole::Admin),
                    "user" => Some(UserRole::User),
                    _ => None,
                })
                .unwrap_or(UserRole::Anonymous);

            let user_id = parts
                .headers
                .get("x-user-id")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| Uuid::parse_str(s).ok());

            let username = parts
                .headers
                .get("x-username")
                .and_then(|v| v.to_str().ok())
                .map(String::from);

            Ok(AuthContext {
                user_id,
                role,
                username,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct TestDto {
        id: i32,
        name: String,
        email: String,
    }

    impl SelectableFields for TestDto {
        fn available_fields() -> Vec<&'static str> {
            vec!["id", "name", "email"]
        }
    }

    #[test]
    fn test_field_selector_filter() {
        let dto = TestDto {
            id: 1,
            name: "test".to_string(),
            email: "test@example.com".to_string(),
        };

        let selector = FieldSelector {
            fields: Some("id,name".to_string()),
        };

        let auth = AuthContext::anonymous();

        let filtered = selector.filter_secure(&dto, &auth).unwrap();
        let obj = filtered.as_object().unwrap();

        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("name"));
        assert!(!obj.contains_key("email"));
    }

    #[test]
    fn test_user_role_permissions() {
        assert!(UserRole::Admin.has_permission(&UserRole::Admin));
        assert!(UserRole::Admin.has_permission(&UserRole::User));
        assert!(UserRole::Admin.has_permission(&UserRole::Anonymous));

        assert!(!UserRole::User.has_permission(&UserRole::Admin));
        assert!(UserRole::User.has_permission(&UserRole::User));
        assert!(UserRole::User.has_permission(&UserRole::Anonymous));

        assert!(!UserRole::Anonymous.has_permission(&UserRole::Admin));
        assert!(!UserRole::Anonymous.has_permission(&UserRole::User));
        assert!(UserRole::Anonymous.has_permission(&UserRole::Anonymous));
    }

    #[test]
    fn test_auth_context_roles() {
        let anon = AuthContext::anonymous();
        assert!(!anon.is_authenticated());
        assert!(anon.has_role(&UserRole::Anonymous));
        assert!(!anon.has_role(&UserRole::User));

        let user = AuthContext::user(Uuid::new_v4(), "testuser".to_string());
        assert!(user.is_authenticated());
        assert!(user.has_role(&UserRole::User));
        assert!(!user.has_role(&UserRole::Admin));

        let admin = AuthContext::admin(Uuid::new_v4(), "admin".to_string());
        assert!(admin.is_authenticated());
        assert!(admin.has_role(&UserRole::Admin));
    }
}
