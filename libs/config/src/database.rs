use crate::{env_required, FromEnv};

/// Database configuration
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub url: String,
}

impl DatabaseConfig {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl FromEnv for DatabaseConfig {
    /// Requires DATABASE_URL to be set (no default)
    fn from_env() -> eyre::Result<Self> {
        Ok(Self {
            url: env_required("DATABASE_URL")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_from_env_success() {
        temp_env::with_var("DATABASE_URL", Some("postgres://localhost/testdb"), || {
            let config = DatabaseConfig::from_env();
            assert!(config.is_ok());
            assert_eq!(config.unwrap().url, "postgres://localhost/testdb");
        });
    }

    #[test]
    fn test_database_config_from_env_missing() {
        temp_env::with_var_unset("DATABASE_URL", || {
            let config = DatabaseConfig::from_env();
            assert!(config.is_err());
            let err = config.unwrap_err();
            assert!(err.to_string().contains("DATABASE_URL"));
            assert!(err.to_string().contains("required"));
        });
    }

    #[test]
    fn test_database_config_new() {
        let config = DatabaseConfig::new("postgres://user:pass@host/db".to_string());
        assert_eq!(config.url, "postgres://user:pass@host/db");
    }
}
