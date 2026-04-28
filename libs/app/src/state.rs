use mongodb::Database;
use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
use sqlx::PgPool;

/// Trait for state that provides PostgreSQL/SeaORM database access
///
/// Implement this once on your AppState, and all APIs that need
/// PostgreSQL access can use it automatically.
pub trait HasDatabase: Clone + Send + Sync + 'static {
    /// Get a reference to the database connection
    fn db(&self) -> &DatabaseConnection;
}

/// Trait for state that provides MongoDB database access
///
/// Implement this once on your AppState, and all APIs that need
/// MongoDB access can use it automatically.
pub trait HasMongoDB: Clone + Send + Sync + 'static {
    /// Get a reference to the MongoDB database
    fn mongo(&self) -> &Database;
}

/// Trait for state that provides Redis access
///
/// Implement this once on your AppState, and all APIs that need
/// Redis access can use it automatically.
pub trait HasRedis: Clone + Send + Sync + 'static {
    /// Get a reference to the Redis connection manager
    fn redis(&self) -> &ConnectionManager;
}

/// Trait for state that provides sqlx PostgreSQL pool access
///
/// Implement this once on your AppState, and all APIs that need
/// direct sqlx access can use it automatically.
pub trait HasSqlxPool: Clone + Send + Sync + 'static {
    /// Get a reference to the sqlx PostgreSQL pool
    fn sqlx_pool(&self) -> &PgPool;
}
