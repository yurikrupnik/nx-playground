use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashSet;

/// Query parameter extractor for field selection
/// Usage: GET /api/todos?fields=id,name,completed
/// In handlers, use: Query(field_selector): Query<FieldSelector>
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

    /// Securely filter a serializable value with validation and role-based access control
    pub fn filter_secure<T>(
        &self,
        value: &T,
        auth: &AuthContext,
    ) -> Result<Value, FieldSelectionError>
    where
        T: Serialize + SelectableFields,
    {
        let requested_fields = self.get_fields();

        // If no fields requested, return all allowed fields for the user's role
        let fields_to_include = match requested_fields {
            Some(ref fields) => {
                // Validate that requested fields exist
                T::validate_fields(fields).map_err(FieldSelectionError::InvalidFields)?;

                // Filter by role and restrictions
                T::filter_by_role(fields, auth)
            }
            None => {
                // Return all fields the user has access to
                let all_fields: HashSet<String> = T::available_fields()
                    .into_iter()
                    .map(String::from)
                    .collect();
                T::filter_by_role(&all_fields, auth)
            }
        };

        // Log field access for audit
        if let Some(ref requested) = requested_fields {
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
        let requested_fields = self.get_fields();

        // If no fields requested, return all allowed fields for the user's role
        let fields_to_include = match requested_fields {
            Some(ref fields) => {
                // Validate that requested fields exist
                T::validate_fields(fields).map_err(FieldSelectionError::InvalidFields)?;

                // Filter by role and restrictions
                T::filter_by_role(fields, auth)
            }
            None => {
                // Return all fields the user has access to
                let all_fields: HashSet<String> = T::available_fields()
                    .into_iter()
                    .map(String::from)
                    .collect();
                T::filter_by_role(&all_fields, auth)
            }
        };

        // Log field access for audit
        if let Some(ref requested) = requested_fields {
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

    /// Filter a serializable value to only include requested fields (legacy, non-secure)
    /// DEPRECATED: Use filter_secure instead for security
    #[deprecated(note = "Use filter_secure for security validation")]
    pub fn filter<T: Serialize>(&self, value: &T) -> Result<Value, serde_json::Error> {
        let json_value = serde_json::to_value(value)?;

        match (json_value, self.get_fields()) {
            (Value::Object(obj), Some(fields)) => Ok(Value::Object(filter_object(obj, &fields))),
            (value, _) => Ok(value),
        }
    }

    /// Filter a list of serializable values (legacy, non-secure)
    /// DEPRECATED: Use filter_list_secure instead for security
    #[deprecated(note = "Use filter_list_secure for security validation")]
    pub fn filter_list<T: Serialize>(&self, values: &[T]) -> Result<Value, serde_json::Error> {
        let fields = self.get_fields();

        let filtered: Result<Vec<Value>, _> = values
            .iter()
            .map(|v| {
                let json_value = serde_json::to_value(v)?;
                match (json_value, &fields) {
                    (Value::Object(obj), Some(fields)) => {
                        Ok(Value::Object(filter_object(obj, fields)))
                    }
                    (value, _) => Ok(value),
                }
            })
            .collect();

        Ok(Value::Array(filtered?))
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

/// Helper function to filter JSON object by field names
fn filter_object(obj: Map<String, Value>, fields: &HashSet<String>) -> Map<String, Value> {
    obj.into_iter()
        .filter(|(k, _)| fields.contains(k))
        .collect()
}

use crate::auth_context::{AuthContext, UserRole};

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

        let access_map: std::collections::HashMap<String, UserRole> = Self::field_access()
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

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
}
