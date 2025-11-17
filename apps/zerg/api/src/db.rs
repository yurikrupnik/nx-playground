use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use tracing::log;

use crate::migrator::Migrator;

pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    let db = Database::connect(opt).await?;

    log::info!("Successfully connected to database");

    Ok(db)
}

pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    log::info!("Running database migrations...");
    Migrator::up(db, None).await?;
    log::info!("Migrations completed successfully");
    Ok(())
}
