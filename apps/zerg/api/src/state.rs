use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// Application state containing shared resources
///
/// Axum requires Clone for State extractor.
/// DatabaseConnection doesn't implement Clone, so we wrap it in Arc.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub redis: ConnectionManager,
}

impl AppState {
    /// Create a new AppState with database and Redis connections
    pub fn new(db: DatabaseConnection, redis: ConnectionManager) -> Self {
        Self {
            db: Arc::new(db),
            redis,
        }
    }

    /// Get a reference to the database connection
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Get a reference to the Redis connection manager
    pub fn redis(&self) -> &ConnectionManager {
        &self.redis
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
// use apis_project::state::ProjectState;
impl apis_project::state::ProjectState for AppState {
  fn db(&self) -> &DatabaseConnection { &self.db }

  fn redis(&self) -> &ConnectionManager { &self.redis }
}

// ✅ Builder pattern for flexible AppState construction
// This prevents tests from breaking when new fields are added to AppState
// Also useful for partial initialization in different environments
pub struct AppStateBuilder {
    db: Option<DatabaseConnection>,
    redis: Option<ConnectionManager>,
}

impl AppStateBuilder {
    pub fn new() -> Self {
        Self {
            db: None,
            redis: None,
        }
    }

    pub fn with_db(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    pub fn with_redis(mut self, redis: ConnectionManager) -> Self {
        self.redis = Some(redis);
        self
    }

    /// Build AppState with the provided components
    ///
    /// For tests that don't use Redis, you must still call `.with_redis_mock()`
    /// or provide a real Redis connection with testcontainers.
    pub fn build(self) -> AppState {
        AppState {
            db: Arc::new(self.db.expect("Database is required for AppState")),
            redis: self.redis.expect(
                "Redis is required. For tests, use .with_redis_mock() or provide a real connection",
            ),
        }
    }

    /// Creates a mock Redis connection manager for tests that don't use Redis
    ///
    /// This must be called in an async context (like #[tokio::test])
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

        let client =
            Client::open("redis://127.0.0.1:6379/").expect("Failed to create mock Redis client");

        let manager = ConnectionManager::new(client)
            .await
            .expect("Failed to create mock Redis connection manager");

        self.with_redis(manager)
    }
}

impl Default for AppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}
