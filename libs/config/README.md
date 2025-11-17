# Shared Config Library

A reusable configuration library for all applications in the workspace.

## Purpose

This library provides:
- Common configuration types (Database, Server, etc.)
- Environment variable loading utilities
- Type-safe configuration with sensible defaults
- Composable config components

## Usage

### In Your App

#### 1. Add dependency to `Cargo.toml`:
```toml
[dependencies]
config = { workspace = true }
```

#### 2. Compose your app-specific config:

```rust
// apps/your-app/src/config.rs
use config::{database::DatabaseConfig, server::ServerConfig};

#[derive(Clone, Debug)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    // Add app-specific fields here
    pub app_name: String,
}

impl Config {
    pub fn from_env() -> eyre::Result<Self> {
        config::init(); // Initialize dotenv

        let database = DatabaseConfig::with_default("postgres://localhost/myapp");
        let server = ServerConfig::with_defaults("0.0.0.0", 8080);

        Ok(Self {
            database,
            server,
            app_name: "my-app".to_string(),
        })
    }

    // Convenience methods
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    pub fn server_address(&self) -> String {
        self.server.address()
    }
}
```

#### 3. Use in your app:

```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = Config::from_env()?;

    // Connect to database
    let db = connect(&config.database.url).await?;

    // Start server
    let listener = TcpListener::bind(config.server.address()).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

## Components

### DatabaseConfig

```rust
use config::database::DatabaseConfig;

// With default
let db = DatabaseConfig::with_default("postgres://localhost/mydb");

// From environment (reads DATABASE_URL)
let db = DatabaseConfig::from_env()?;

// Access URL
println!("Connecting to: {}", db.url);
```

**Environment Variable:**
- `DATABASE_URL` - PostgreSQL connection string

### ServerConfig

```rust
use config::server::ServerConfig;

// With defaults
let server = ServerConfig::with_defaults("0.0.0.0", 3000);

// From environment (reads HOST and PORT)
let server = ServerConfig::from_env()?;

// Access
println!("Server listening on {}", server.address());
```

**Environment Variables:**
- `HOST` - Server host (default: `0.0.0.0`)
- `PORT` - Server port (default: `3000`)

## Helper Functions

### `config::init()`
Initializes dotenv (loads `.env` file). Call once at app startup.

### `config::env_or_default(key, default)`
Gets environment variable or returns default value.

### `config::env_required(key)`
Gets environment variable or returns error if not set.

## Examples

### Example 1: API Server
```rust
use config::{database::DatabaseConfig, server::ServerConfig};

pub struct ApiConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

impl ApiConfig {
    pub fn from_env() -> eyre::Result<Self> {
        config::init();
        Ok(Self {
            database: DatabaseConfig::with_default("postgres://localhost/api_db"),
            server: ServerConfig::with_defaults("0.0.0.0", 3000),
        })
    }
}
```

### Example 2: Worker Service
```rust
use config::database::DatabaseConfig;

pub struct WorkerConfig {
    pub database: DatabaseConfig,
    pub queue_name: String,
}

impl WorkerConfig {
    pub fn from_env() -> eyre::Result<Self> {
        config::init();
        Ok(Self {
            database: DatabaseConfig::from_env()?,
            queue_name: config::env_or_default("QUEUE_NAME", "default"),
        })
    }
}
```

### Example 3: Multiple Databases
```rust
use config::database::DatabaseConfig;

pub struct MultiDbConfig {
    pub primary_db: DatabaseConfig,
    pub analytics_db: DatabaseConfig,
}

impl MultiDbConfig {
    pub fn from_env() -> eyre::Result<Self> {
        config::init();

        let primary = DatabaseConfig::new(
            config::env_or_default("PRIMARY_DB_URL", "postgres://localhost/primary")
        );

        let analytics = DatabaseConfig::new(
            config::env_or_default("ANALYTICS_DB_URL", "postgres://localhost/analytics")
        );

        Ok(Self {
            primary_db: primary,
            analytics_db: analytics,
        })
    }
}
```

## Benefits

✅ **DRY** - Write config logic once, use everywhere
✅ **Type-safe** - Compile-time checking
✅ **Consistent** - Same patterns across all apps
✅ **Testable** - Easy to mock and test
✅ **Extensible** - Add new shared config types easily
✅ **Composable** - Mix and match config components

## Adding New Config Types

To add a new shared config type:

1. Create a new module (e.g., `src/redis.rs`)
2. Implement the config struct
3. Implement `FromEnv` trait
4. Export from `lib.rs`

Example:

```rust
// src/redis.rs
use crate::{env_or_default, FromEnv};

#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub url: String,
}

impl FromEnv for RedisConfig {
    fn from_env() -> eyre::Result<Self> {
        Ok(Self {
            url: env_or_default("REDIS_URL", "redis://localhost:6379"),
        })
    }
}

// src/lib.rs
pub mod redis;
```

Now any app can use `RedisConfig`!
