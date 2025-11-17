use sea_orm::Database;
use testcontainers::{runners::AsyncRunner, ContainerAsync};
use testcontainers_modules::postgres::Postgres;
use zerg_api::{
    db,
    state::{AppState, AppStateBuilder},
};

/// Test container for a PostgreSQL database
#[allow(dead_code)]
pub struct TestDb {
    pub container: ContainerAsync<Postgres>,
    pub app_state: AppState,
    pub connection_string: String,
}

impl TestDb {
    /// Creates a new test database with migrations applied
    ///
    /// Redis is mocked - tests that don't use Redis will work fine.
    /// When you add Redis business logic, you can add a real Redis testcontainer.
    pub async fn new() -> Self {
        // Start PostgreSQL container
        let postgres_image = Postgres::default();
        let container = postgres_image
            .start()
            .await
            .expect("Failed to start postgres container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get host port");

        let connection_string = format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        // Connect to database
        let connection = Database::connect(&connection_string)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        db::run_migrations(&connection)
            .await
            .expect("Failed to run migrations");

        // Build AppState with mock Redis (safe for tests that don't use Redis)
        let app_state = AppStateBuilder::new()
            .with_db(connection)
            .with_redis_mock()
            .await
            .build();

        Self {
            container,
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
