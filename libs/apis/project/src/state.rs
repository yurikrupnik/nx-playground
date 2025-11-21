use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
// use sqlx::PgPool;

pub trait ProjectState: Clone + Send + Sync + 'static {
    fn db(&self) -> &DatabaseConnection;
    fn redis(&self) -> &ConnectionManager;
}
