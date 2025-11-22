use sea_orm::Database;
use services::postgres;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::{mongo::Mongo, postgres::Postgres, redis::Redis};
use zerg_api::{
    migrator::Migrator,
    state::{AppState, AppStateBuilder},
};

/// Test container for a PostgreSQL database with Redis and MongoDB
#[allow(dead_code)]
pub struct TestDb {
    pub postgres_container: ContainerAsync<Postgres>,
    pub mongo_container: ContainerAsync<Mongo>,
    pub redis_container: ContainerAsync<Redis>,
    pub app_state: AppState,
    pub connection_string: String,
}

impl TestDb {
    /// Creates a new test database with migrations applied and testcontainers for all services
    pub async fn new() -> Self {
        // Start PostgreSQL container with latest version
        let postgres_image = Postgres::default().with_tag("17-alpine");
        let postgres_container = postgres_image
            .start()
            .await
            .expect("Failed to start postgres container");

        let host_port = postgres_container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get postgres host port");

        let connection_string = format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        // Start MongoDB container with latest version
        // Using specific startup timeout to handle parallel test execution
        let mongo_image = Mongo::default()
            .with_tag("7")
            .with_startup_timeout(std::time::Duration::from_secs(180));
        let mongo_container = mongo_image
            .start()
            .await
            .expect("Failed to start MongoDB container");

        let mongo_port = mongo_container
            .get_host_port_ipv4(27017)
            .await
            .expect("Failed to get MongoDB host port");

        let mongo_uri = format!("mongodb://localhost:{}/", mongo_port);

        // Start Redis container with latest version
        let redis_image = Redis::default()
            .with_tag("7-alpine")
            .with_startup_timeout(std::time::Duration::from_secs(180));
        let redis_container = redis_image
            .start()
            .await
            .expect("Failed to start Redis container");

        let redis_port = redis_container
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to get Redis host port");

        let redis_url = format!("redis://localhost:{}/", redis_port);

        // Connect to database
        let connection = Database::connect(&connection_string)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        postgres::run_migrations::<Migrator>(&connection, env!("CARGO_PKG_NAME"))
            .await
            .expect("Failed to run migrations");

        // Create sqlx pool
        let sqlx_pool = sqlx::PgPool::connect(&connection_string)
            .await
            .expect("Failed to create sqlx pool");

        // Connect to MongoDB
        let mongo_client = mongodb::Client::with_uri_str(&mongo_uri)
            .await
            .expect("Failed to connect to MongoDB");
        let mongo = mongo_client.database("test");

        // Connect to Redis
        let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");
        let redis_manager = redis::aio::ConnectionManager::new(redis_client)
            .await
            .expect("Failed to create Redis connection manager");

        // Build AppState with real testcontainer connections
        let app_state = AppStateBuilder::new()
            .with_db(connection)
            .with_mongo(mongo)
            .with_redis(redis_manager)
            .with_sqlx_pool(sqlx_pool)
            .build();

        Self {
            postgres_container,
            mongo_container,
            redis_container,
            app_state,
            connection_string,
        }
    }

    /// Returns the AppState for testing
    pub fn state(&self) -> AppState {
        self.app_state.clone()
    }
}

/// Helper to create sample user data for testing
#[allow(dead_code)]
pub mod fixtures {
    use chrono::Utc;
    use uuid::Uuid;
    use zerg_api::{dto::user::CreateUserDto, entities::user};

    pub fn create_user_dto() -> CreateUserDto {
        CreateUserDto {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        }
    }

    pub fn user_model() -> user::Model {
        user::Model {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        }
    }
}
