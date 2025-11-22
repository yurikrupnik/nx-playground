use mongodb::Database;
use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
use sqlx::PgPool;
use std::sync::Arc;

/// Application state containing shared resources
///
/// Axum requires Clone for State extractor.
/// DatabaseConnection doesn't implement Clone, so we wrap it in Arc.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub mongo: Database,
    pub redis: ConnectionManager,
    pub sqlx_pool: PgPool,
}

impl AppState {
    /// Create a new AppState with database, MongoDB, Redis, and sqlx pool connections
    pub fn new(db: DatabaseConnection, mongo: Database, redis: ConnectionManager, sqlx_pool: PgPool) -> Self {
        Self {
            db: Arc::new(db),
            mongo,
            redis,
            sqlx_pool,
        }
    }

    /// Get a reference to the database connection
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Get a reference to the MongoDB database
    pub fn mongo(&self) -> &Database {
        &self.mongo
    }

    /// Get a reference to the Redis connection manager
    pub fn redis(&self) -> &ConnectionManager {
        &self.redis
    }

    /// Get a reference to the sqlx PostgreSQL pool
    pub fn sqlx_pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}

// ✅ FromRef implementations for partial state extraction
// This allows handlers to extract only the dependencies they need
// Similar to the pattern used in terran API
impl axum::extract::FromRef<AppState> for Arc<DatabaseConnection> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl axum::extract::FromRef<AppState> for ConnectionManager {
    fn from_ref(state: &AppState) -> Self {
        state.redis.clone()
    }
}
// ✅ Implement composition traits once - works for ALL APIs automatically!
impl app::state::HasDatabase for AppState {
    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

impl app::state::HasMongoDB for AppState {
    fn mongo(&self) -> &Database {
        &self.mongo
    }
}

impl app::state::HasRedis for AppState {
    fn redis(&self) -> &ConnectionManager {
        &self.redis
    }
}

impl app::state::HasSqlxPool for AppState {
    fn sqlx_pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}

// ✨ ProjectState, CarState, and BikeState are automatically implemented via blanket impl!
// No need to write explicit impl blocks - composition traits handle it!

// ✅ Builder pattern for flexible AppState construction
// This prevents tests from breaking when new fields are added to AppState
// Also useful for partial initialization in different environments
pub struct AppStateBuilder {
    db: Option<DatabaseConnection>,
    mongo: Option<Database>,
    redis: Option<ConnectionManager>,
    sqlx_pool: Option<PgPool>,
}

impl AppStateBuilder {
    pub fn new() -> Self {
        Self {
            db: None,
            mongo: None,
            redis: None,
            sqlx_pool: None,
        }
    }

    pub fn with_db(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    pub fn with_mongo(mut self, mongo: Database) -> Self {
        self.mongo = Some(mongo);
        self
    }

    pub fn with_redis(mut self, redis: ConnectionManager) -> Self {
        self.redis = Some(redis);
        self
    }

    pub fn with_sqlx_pool(mut self, sqlx_pool: PgPool) -> Self {
        self.sqlx_pool = Some(sqlx_pool);
        self
    }

    /// Build AppState with the provided components
    ///
    /// For tests that don't use Redis, you must still call `.with_redis_mock()`
    /// or provide a real Redis connection with testcontainers.
    /// For tests that don't use MongoDB, you must still call `.with_mongo_mock()`
    /// or provide a real MongoDB connection with testcontainers.
    /// For tests that don't use sqlx, you can omit `.with_sqlx_pool()` and a
    /// minimal pool will be created automatically.
    pub fn build(self) -> AppState {
        let sqlx_pool = self.sqlx_pool.unwrap_or_else(|| {
            // Create a minimal pool that won't be used in tests
            // This is a blocking operation but only happens in test scenarios
            PgPool::connect_lazy("postgresql://localhost:5432/unused")
                .expect("Failed to create lazy sqlx pool")
        });

        AppState {
            db: Arc::new(self.db.expect("Database is required for AppState")),
            mongo: self.mongo.expect("MongoDB is required for AppState. Use .with_mongo_mock() for tests"),
            redis: self.redis.expect(
                "Redis is required. For tests, use .with_redis_mock() or provide a real connection",
            ),
            sqlx_pool,
        }
    }

    /// Creates a MongoDB database using testcontainers for tests
    ///
    /// This must be called in an async context (like #[tokio::test])
    ///
    /// Note: This uses testcontainers and requires Docker to be running.
    /// For true unit tests, consider using integration tests with TestDb from common/mod.rs
    pub async fn with_mongo_mock(self) -> Self {
        use mongodb::Client;
        use testcontainers::{runners::AsyncRunner, ImageExt};
        use testcontainers_modules::mongo::Mongo;

        // Start MongoDB container with latest version
        // Using specific startup timeout to handle parallel test execution
        let mongo_image = Mongo::default()
            .with_tag("7")
            .with_startup_timeout(std::time::Duration::from_secs(180));
        let container = mongo_image
            .start()
            .await
            .expect("Failed to start MongoDB container");

        let mongo_port = container
            .get_host_port_ipv4(27017)
            .await
            .expect("Failed to get MongoDB host port");

        let mongo_uri = format!("mongodb://localhost:{}/", mongo_port);

        let client = Client::with_uri_str(&mongo_uri)
            .await
            .expect("Failed to create MongoDB client");

        let mongo = client.database("test");

        // Keep the container alive by leaking it (it will be cleaned up when test ends)
        Box::leak(Box::new(container));

        self.with_mongo(mongo)
    }

    /// Creates a Redis connection manager using testcontainers for tests
    ///
    /// This must be called in an async context (like #[tokio::test])
    ///
    /// Note: This uses testcontainers and requires Docker to be running.
    /// For true unit tests, consider using integration tests with TestDb from common/mod.rs
    ///
    /// # Example
    /// ```ignore
    /// let state = AppStateBuilder::new()
    ///     .with_db(db)
    ///     .with_redis_mock().await
    ///     .build();
    /// ```
    pub async fn with_redis_mock(self) -> Self {
        use redis::Client;
        use testcontainers::{runners::AsyncRunner, ImageExt};
        use testcontainers_modules::redis::Redis;

        // Start Redis container with latest version
        // Using specific startup timeout to handle parallel test execution
        let redis_image = Redis::default()
            .with_tag("7-alpine")
            .with_startup_timeout(std::time::Duration::from_secs(180));
        let container = redis_image
            .start()
            .await
            .expect("Failed to start Redis container");

        let redis_port = container
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to get Redis host port");

        let redis_url = format!("redis://localhost:{}/", redis_port);

        let client = Client::open(redis_url).expect("Failed to create Redis client");

        let manager = ConnectionManager::new(client)
            .await
            .expect("Failed to create Redis connection manager");

        // Keep the container alive by leaking it (it will be cleaned up when the test ends)
        Box::leak(Box::new(container));

        self.with_redis(manager)
    }
}

impl Default for AppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}
