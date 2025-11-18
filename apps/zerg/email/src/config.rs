use config::{redis::RedisConfig, server::ServerConfig, Environment, FromEnv};
use eyre::Result;

/// Email worker configuration
#[derive(Clone, Debug)]
pub struct Config {
    pub environment: Environment,
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub email: EmailConfig,
}

impl FromEnv for Config {
    fn from_env() -> Result<Self> {
        // Initialize dotenv to load .env file
        config::init();

        Ok(Self {
            environment: Environment::from_env(),
            server: ServerConfig::from_env()?,
            redis: RedisConfig::from_env()?,
            email: EmailConfig::from_env()?,
        })
    }
}

/// Email-specific configuration
#[derive(Clone, Debug)]
pub struct EmailConfig {
    /// SMTP host
    pub smtp_host: String,
    /// SMTP port
    pub smtp_port: u16,
    /// SMTP username
    pub smtp_username: String,
    /// SMTP password
    pub smtp_password: String,
    /// From email address
    pub from_email: String,
    /// From name
    pub from_name: String,
    /// Use TLS for SMTP
    pub smtp_use_tls: bool,
    /// Number of worker tasks
    pub worker_count: usize,
    /// Batch size for reading from stream
    pub batch_size: usize,
    /// Block timeout in milliseconds
    pub block_timeout_ms: usize,
}

impl FromEnv for EmailConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            smtp_host: config::env_or_default("SMTP_HOST", "localhost"),
            smtp_port: config::env_or_default("SMTP_PORT", "587").parse()?,
            smtp_username: config::env_or_default("SMTP_USERNAME", ""),
            smtp_password: config::env_or_default("SMTP_PASSWORD", ""),
            from_email: config::env_or_default("EMAIL_FROM_ADDRESS", "noreply@example.com"),
            from_name: config::env_or_default("EMAIL_FROM_NAME", "Zerg"),
            smtp_use_tls: config::env_or_default("SMTP_USE_TLS", "true").parse()?,
            worker_count: config::env_or_default("EMAIL_WORKER_COUNT", "4").parse()?,
            batch_size: config::env_or_default("EMAIL_BATCH_SIZE", "10").parse()?,
            block_timeout_ms: config::env_or_default("EMAIL_BLOCK_TIMEOUT_MS", "5000").parse()?,
        })
    }
}
