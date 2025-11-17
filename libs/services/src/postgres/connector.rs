use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use tracing::{info, log::LevelFilter};

/// Connect to PostgreSQL database with optimized connection pool settings
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string
///
/// # Example
/// ```ignore
/// let db = connect("postgresql://user:pass@localhost/db").await?;
/// ```
pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Info); // SeaORM requires log::LevelFilter

    let db = Database::connect(opt).await?;

    info!("Successfully connected to PostgreSQL database");

    Ok(db)
}

/// Run database migrations using the provided Migrator
///
/// This is a generic function that works with any app's Migrator.
/// The migration files remain in the app, but the running logic is here.
///
/// # Arguments
/// * `db` - Database connection
/// * `app_name` - Name of the app for logging (e.g., "zerg_api")
///
/// # Example
/// ```ignore
/// use my_app::migrator::Migrator;
///
/// run_migrations::<Migrator>(&db, "my_app").await?;
/// ```
pub async fn run_migrations<M: MigratorTrait>(
    db: &DatabaseConnection,
    app_name: &str,
) -> Result<(), DbErr> {
    info!("Running {} database migrations...", app_name);
    M::up(db, None).await?;
    info!("Migrations completed successfully for {}", app_name);
    Ok(())
}
