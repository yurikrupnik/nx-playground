use crate::{env_required, FromEnv};

/// Redis configuration
#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub host: String,
}

impl RedisConfig {
    pub fn new(host: String) -> Self {
        Self { host }
    }

    /// Get the Redis connection URL
    pub fn url(&self) -> &str {
        &self.host
    }
}

impl FromEnv for RedisConfig {
    /// Requires REDIS_HOST to be set (no default)
    fn from_env() -> eyre::Result<Self> {
        Ok(Self {
            host: env_required("REDIS_HOST")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_config_from_env_success() {
        temp_env::with_var("REDIS_HOST", Some("redis://localhost:6379"), || {
            let config = RedisConfig::from_env();
            assert!(config.is_ok());
            let config = config.unwrap();
            assert_eq!(config.host, "redis://localhost:6379");
            assert_eq!(config.url(), "redis://localhost:6379");
        });
    }

    #[test]
    fn test_redis_config_from_env_missing() {
        temp_env::with_var_unset("REDIS_HOST", || {
            let config = RedisConfig::from_env();
            assert!(config.is_err());
            let err = config.unwrap_err();
            assert!(err.to_string().contains("REDIS_HOST"));
            assert!(err.to_string().contains("required"));
        });
    }

    #[test]
    fn test_redis_config_new() {
        let config = RedisConfig::new("redis://prod-host:6379".to_string());
        assert_eq!(config.host, "redis://prod-host:6379");
    }

    #[test]
    fn test_redis_config_url() {
        let config = RedisConfig::new("redis://localhost:6379".to_string());
        assert_eq!(config.url(), "redis://localhost:6379");
    }
}
