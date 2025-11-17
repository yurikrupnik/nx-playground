use crate::{env_required, FromEnv};

/// MongoDB configuration
#[derive(Clone, Debug)]
pub struct MongoConfig {
    pub uri: String,
}

impl MongoConfig {
    pub fn new(uri: String) -> Self {
        Self { uri }
    }

    /// Get the MongoDB connection URI
    pub fn uri(&self) -> &str {
        &self.uri
    }
}

impl FromEnv for MongoConfig {
    /// Requires MONGO_URI to be set (no default)
    fn from_env() -> eyre::Result<Self> {
        Ok(Self {
            uri: env_required("MONGO_URI")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mongo_config_from_env_success() {
        temp_env::with_var(
            "MONGO_URI",
            Some("mongodb://localhost:27017/testdb"),
            || {
                let config = MongoConfig::from_env();
                assert!(config.is_ok());
                let config = config.unwrap();
                assert_eq!(config.uri, "mongodb://localhost:27017/testdb");
                assert_eq!(config.uri(), "mongodb://localhost:27017/testdb");
            },
        );
    }

    #[test]
    fn test_mongo_config_from_env_missing() {
        temp_env::with_var_unset("MONGO_URI", || {
            let config = MongoConfig::from_env();
            assert!(config.is_err());
            let err = config.unwrap_err();
            assert!(err.to_string().contains("MONGO_URI"));
            assert!(err.to_string().contains("required"));
        });
    }

    #[test]
    fn test_mongo_config_new() {
        let config = MongoConfig::new("mongodb://prod-host:27017/db".to_string());
        assert_eq!(config.uri, "mongodb://prod-host:27017/db");
    }

    #[test]
    fn test_mongo_config_uri() {
        let config = MongoConfig::new("mongodb://localhost:27017/mydb".to_string());
        assert_eq!(config.uri(), "mongodb://localhost:27017/mydb");
    }
}
