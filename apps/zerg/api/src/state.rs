use sea_orm::DatabaseConnection;
use std::sync::Arc;

// Axum requires Clone for State extractor.
// DatabaseConnection doesn't implement Clone, so we wrap it in Arc.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Helper to get a reference to the database connection
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}
