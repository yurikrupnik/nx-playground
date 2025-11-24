use sea_orm::Database;
use services::postgres;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::{mongo::Mongo, postgres::Postgres, redis::Redis};
use tokio::sync::OnceCell;
use zerg_api::{
    migrator::Migrator,
    state::{AppState, AppStateBuilder},
};

/// Shared test containers that are created once and reused across all tests
struct SharedTestContainers {
    #[allow(dead_code)]
    postgres_container: ContainerAsync<Postgres>,
    #[allow(dead_code)]
    mongo_container: ContainerAsync<Mongo>,
    #[allow(dead_code)]
    redis_container: ContainerAsync<Redis>,
    postgres_url: String,
    mongo_uri: String,
    redis_url: String,
}

impl SharedTestContainers {
    async fn new() -> Self {
        println!("🚀 Starting shared test containers...");

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

        let postgres_url = format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        // Start MongoDB container with latest version
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

        println!("✅ Shared test containers started successfully");

        // Run migrations once when containers are created
        let connection = Database::connect(&postgres_url)
            .await
            .expect("Failed to connect to test database");

        postgres::run_migrations::<Migrator>(&connection, env!("CARGO_PKG_NAME"))
            .await
            .expect("Failed to run migrations");

        println!("✅ Database migrations completed");

        Self {
            postgres_container,
            mongo_container,
            redis_container,
            postgres_url,
            mongo_uri,
            redis_url,
        }
    }
}

// Global shared containers - initialized once and reused by all tests
static SHARED_CONTAINERS: OnceCell<SharedTestContainers> = OnceCell::const_new();

/// Test container for a PostgreSQL database with Redis and MongoDB
/// This now uses shared containers but provides isolated test state
#[allow(dead_code)]
pub struct TestDb {
    pub app_state: AppState,
    pub connection_string: String,
}

impl TestDb {
    /// Creates a new test database with clean state using shared containers
    pub async fn new() -> Self {
        let containers = SHARED_CONTAINERS
            .get_or_init(|| async { SharedTestContainers::new().await })
            .await;

        let connection_string = containers.postgres_url.clone();
        let mongo_uri = containers.mongo_uri.clone();
        let redis_url = containers.redis_url.clone();

        // Connect to database
        let connection = Database::connect(&connection_string)
            .await
            .expect("Failed to connect to test database");

        // Create sqlx pool
        let sqlx_pool = sqlx::PgPool::connect(&connection_string)
            .await
            .expect("Failed to create sqlx pool");

        // Clean PostgreSQL tables for test isolation (using sqlx)
        cleanup_postgres_database_sqlx(&sqlx_pool).await;

        // Connect to MongoDB
        let mongo_client = mongodb::Client::with_uri_str(&mongo_uri)
            .await
            .expect("Failed to connect to MongoDB");
        let mongo = mongo_client.database("test");

        // Clean MongoDB collections for test isolation
        cleanup_mongo_database(&mongo).await;

        // Connect to Redis
        let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");
        let redis_manager = redis::aio::ConnectionManager::new(redis_client)
            .await
            .expect("Failed to create Redis connection manager");

        // Clean Redis for test isolation
        cleanup_redis(&redis_manager).await;

        // Build AppState with real testcontainer connections
        let app_state = AppStateBuilder::new()
            .with_db(connection)
            .with_mongo(mongo)
            .with_redis(redis_manager)
            .with_sqlx_pool(sqlx_pool)
            .build();

        Self {
            app_state,
            connection_string,
        }
    }

    /// Returns the AppState for testing
    pub fn state(&self) -> AppState {
        self.app_state.clone()
    }
}

/// Helper function to create test state using shared containers
#[allow(dead_code)]
pub async fn create_test_state() -> AppState {
    let containers = SHARED_CONTAINERS
        .get_or_init(|| async { SharedTestContainers::new().await })
        .await;

    let postgres_url = containers.postgres_url.clone();
    let mongo_uri = containers.mongo_uri.clone();
    let redis_url = containers.redis_url.clone();

    // Connect to databases
    let db = services::postgres::connect(&postgres_url)
        .await
        .expect("Failed to connect to test Postgres");

    let sqlx_pool = sqlx::PgPool::connect(&postgres_url)
        .await
        .expect("Failed to create sqlx pool");

    // Clean PostgreSQL tables for test isolation
    cleanup_postgres_database_sqlx(&sqlx_pool).await;

    let mongo_client = mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to test MongoDB");
    let mongo = mongo_client.database("zerg_test");

    // Clean MongoDB collections for test isolation
    cleanup_mongo_database(&mongo).await;

    let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let redis = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to test Redis");

    // Clean Redis for test isolation
    cleanup_redis(&redis).await;

    AppState::new(db, mongo, redis, sqlx_pool)
}

/// Clean all MongoDB collections in the test database
async fn cleanup_mongo_database(mongo: &mongodb::Database) {
    let collection_names = mongo
        .list_collection_names()
        .await
        .expect("Failed to list MongoDB collections");

    for collection_name in collection_names {
        mongo
            .collection::<mongodb::bson::Document>(&collection_name)
            .drop()
            .await
            .expect(&format!("Failed to drop collection: {}", collection_name));
    }
}

/// Clean all Redis keys in the test database
async fn cleanup_redis(redis: &redis::aio::ConnectionManager) {
    use redis::AsyncCommands;
    let mut conn = redis.clone();
    let _: () = conn
        .flushdb()
        .await
        .expect("Failed to flush Redis database");
}

/// Clean all PostgreSQL tables in the test database using sqlx
async fn cleanup_postgres_database_sqlx(pool: &sqlx::PgPool) {
    // Get all table names from the public schema
    let tables: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public'
        AND tablename != 'seaorm_migration_history'
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("Failed to query tables");

    // Truncate all tables (faster than DELETE)
    for (table_name,) in tables {
        let truncate_query = format!("TRUNCATE TABLE \"{}\" RESTART IDENTITY CASCADE", table_name);
        sqlx::query(&truncate_query)
            .execute(pool)
            .await
            .expect(&format!("Failed to truncate table: {}", table_name));
    }

    // Re-seed initial data after cleanup
    seed_initial_data(pool).await;
}

/// Re-seed initial test data (same as migration seed)
async fn seed_initial_data(pool: &sqlx::PgPool) {
    // Seed Users
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES
            ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 'admin', 'admin@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.V5ferDZKUqIW6K', NOW(), NOW()),
            ('b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'john_doe', 'john@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.V5ferDZKUqIW6K', NOW(), NOW()),
            ('c2eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'jane_smith', 'jane@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.V5ferDZKUqIW6K', NOW(), NOW())
        ON CONFLICT DO NOTHING
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed users");

    // Seed Todos
    sqlx::query(
        r#"
        INSERT INTO todos (id, name, description, completed, created_at, updated_at)
        VALUES
            ('d3eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', 'Setup development environment', 'Install all required tools and dependencies', true, NOW(), NOW()),
            ('e4eebc99-9c0b-4ef8-bb6d-6bb9bd380a55', 'Write unit tests', 'Add comprehensive test coverage for core modules', false, NOW(), NOW()),
            ('f5eebc99-9c0b-4ef8-bb6d-6bb9bd380a66', 'Review pull requests', 'Review pending PRs from team members', false, NOW(), NOW()),
            ('06eebc99-9c0b-4ef8-bb6d-6bb9bd380a77', 'Update documentation', 'Update API docs with new endpoints', false, NOW(), NOW()),
            ('17eebc99-9c0b-4ef8-bb6d-6bb9bd380a88', 'Deploy to staging', 'Deploy latest changes to staging environment', false, NOW(), NOW())
        ON CONFLICT DO NOTHING
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed todos");

    // Seed Authors
    sqlx::query(
        r#"
        INSERT INTO authors (id, name, bio, created_at, updated_at)
        VALUES
            ('28eebc99-9c0b-4ef8-bb6d-6bb9bd380a99', 'George Orwell', 'English novelist and essayist, known for his dystopian works.', NOW(), NOW()),
            ('39eebc99-9c0b-4ef8-bb6d-6bb9bd380aaa', 'Jane Austen', 'English novelist known for her witty social commentary.', NOW(), NOW()),
            ('4aeebc99-9c0b-4ef8-bb6d-6bb9bd380abb', 'Isaac Asimov', 'American writer and professor of biochemistry, known for science fiction.', NOW(), NOW())
        ON CONFLICT DO NOTHING
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed authors");

    // Seed Books
    sqlx::query(
        r#"
        INSERT INTO books (id, title, description, author_id, published_date, isbn, created_at, updated_at)
        VALUES
            ('5beebc99-9c0b-4ef8-bb6d-6bb9bd380acc', '1984', 'A dystopian social science fiction novel.', '28eebc99-9c0b-4ef8-bb6d-6bb9bd380a99', '1949-06-08', '978-0451524935', NOW(), NOW()),
            ('6ceebc99-9c0b-4ef8-bb6d-6bb9bd380add', 'Animal Farm', 'A satirical allegorical novella.', '28eebc99-9c0b-4ef8-bb6d-6bb9bd380a99', '1945-08-17', '978-0451526342', NOW(), NOW()),
            ('7deebc99-9c0b-4ef8-bb6d-6bb9bd380aee', 'Pride and Prejudice', 'A romantic novel of manners.', '39eebc99-9c0b-4ef8-bb6d-6bb9bd380aaa', '1813-01-28', '978-0141439518', NOW(), NOW()),
            ('8eeebc99-9c0b-4ef8-bb6d-6bb9bd380aff', 'Foundation', 'The first novel in the Foundation series.', '4aeebc99-9c0b-4ef8-bb6d-6bb9bd380abb', '1951-06-01', '978-0553293357', NOW(), NOW()),
            ('9feebc99-9c0b-4ef8-bb6d-6bb9bd380b00', 'I, Robot', 'A collection of nine science fiction short stories.', '4aeebc99-9c0b-4ef8-bb6d-6bb9bd380abb', '1950-12-02', '978-0553382563', NOW(), NOW())
        ON CONFLICT DO NOTHING
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed books");

    // Seed Projects
    sqlx::query(
        r#"
        INSERT INTO projects (title, description, completed, created_at, updated_at)
        VALUES
            ('Website Redesign', 'Complete overhaul of company website with modern UI/UX', false, NOW(), NOW()),
            ('Mobile App MVP', 'Develop minimum viable product for iOS and Android', false, NOW(), NOW()),
            ('API Integration', 'Integrate third-party payment gateway', true, NOW(), NOW()),
            ('Database Migration', 'Migrate from MySQL to PostgreSQL', true, NOW(), NOW()),
            ('CI/CD Pipeline', 'Setup automated testing and deployment pipeline', false, NOW(), NOW())
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to seed projects");
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
