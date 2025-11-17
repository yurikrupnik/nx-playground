use config::{database::DatabaseConfig, server::ServerConfig, FromEnv};

// Re-export Environment for use in other modules
pub use config::Environment;

/// Application-specific configuration
/// Composes shared config components from the `config` library
#[derive(Clone, Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub environment: Environment,
}

impl Config {
    pub fn from_env() -> eyre::Result<Self> {
        // Initialize dotenv once
        config::init();

        let environment = Environment::from_env();
        let database = DatabaseConfig::from_env()?; // Required - will fail if not set
        let server = ServerConfig::from_env()?; // Uses defaults: HOST=0.0.0.0, PORT=8080

        Ok(Self {
            database,
            server,
            environment,
        })
    }
}
