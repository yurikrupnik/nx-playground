use axum::{extract::FromRequestParts, http::request::Parts};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User roles for authorization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        match (self, required) {
            (UserRole::Admin, _) => true,
            (UserRole::User, UserRole::User) => true,
            (UserRole::User, UserRole::Anonymous) => true,
            (UserRole::Anonymous, UserRole::Anonymous) => true,
            _ => false,
        }
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

/// Extractor for AuthContext from request
/// For now, this creates an anonymous context
/// TODO: Implement actual authentication (JWT, session, etc.)
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // TODO: Extract from Authorization header, session cookie, etc.
        // For now, check for a simple x-user-role header for testing
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
