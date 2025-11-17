pub mod database;
pub mod mongodb;
pub mod redis;
pub mod server;
pub mod tracing;

use std::env;

/// Application environment (dev = local/kind, prod = full k8s)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Environment {
    Development, // Local dev or kind cluster (no HTTPS)
    Production,  // Full k8s cluster (with HTTPS)
}

impl Environment {
    pub fn from_env() -> Self {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        if app_env.eq_ignore_ascii_case("production") {
            Environment::Production
        } else {
            Environment::Development
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }

    // Whether HTTPS features should be enabled
    pub fn use_https(&self) -> bool {
        self.is_production()
    }
}

/// Trait for configuration that can be loaded from environment variables
pub trait FromEnv: Sized {
    fn from_env() -> eyre::Result<Self>;
}

/// Helper to load and parse environment variable with a default value
pub fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Helper to load and parse environment variable or return error
pub fn env_required(key: &str) -> eyre::Result<String> {
    env::var(key).map_err(|_| eyre::eyre!("Environment variable {} is required", key))
}

/// Initialize configuration (call once at app startup)
/// Only loads .env file in development (not in production k8s)
pub fn init() {
    // Only load .env in development
    // In production, env vars are set by k8s ConfigMaps/Secrets
    let env = Environment::from_env();
    if env.is_development() {
        dotenvy::dotenv().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_defaults_to_development() {
        temp_env::with_var_unset("APP_ENV", || {
            let env = Environment::from_env();
            assert_eq!(env, Environment::Development);
            assert!(env.is_development());
            assert!(!env.is_production());
            assert!(!env.use_https());
        });
    }

    #[test]
    fn test_environment_production() {
        temp_env::with_var("APP_ENV", Some("production"), || {
            let env = Environment::from_env();
            assert_eq!(env, Environment::Production);
            assert!(env.is_production());
            assert!(!env.is_development());
            assert!(env.use_https());
        });
    }

    #[test]
    fn test_environment_production_case_insensitive() {
        temp_env::with_var("APP_ENV", Some("PRODUCTION"), || {
            let env = Environment::from_env();
            assert_eq!(env, Environment::Production);
        });

        temp_env::with_var("APP_ENV", Some("Production"), || {
            let env = Environment::from_env();
            assert_eq!(env, Environment::Production);
        });
    }

    #[test]
    fn test_environment_unknown_defaults_to_development() {
        temp_env::with_var("APP_ENV", Some("staging"), || {
            let env = Environment::from_env();
            assert_eq!(env, Environment::Development);
        });
    }

    #[test]
    fn test_env_or_default_with_value() {
        temp_env::with_var("TEST_VAR", Some("test_value"), || {
            let result = env_or_default("TEST_VAR", "default");
            assert_eq!(result, "test_value");
        });
    }

    #[test]
    fn test_env_or_default_without_value() {
        temp_env::with_var_unset("MISSING_VAR", || {
            let result = env_or_default("MISSING_VAR", "default_value");
            assert_eq!(result, "default_value");
        });
    }

    #[test]
    fn test_env_required_success() {
        temp_env::with_var("REQUIRED_VAR", Some("required_value"), || {
            let result = env_required("REQUIRED_VAR");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "required_value");
        });
    }

    #[test]
    fn test_env_required_missing() {
        temp_env::with_var_unset("MISSING_REQUIRED", || {
            let result = env_required("MISSING_REQUIRED");
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("MISSING_REQUIRED"));
            assert!(err.to_string().contains("required"));
        });
    }
}
