/// Tests demonstrating the composition pattern benefits
use app::state::{HasDatabase, HasMongoDB, HasRedis};
use mongodb::Database;
use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::{mongo::Mongo, postgres::Postgres, redis::Redis};

/// Demonstrates that a single state can support multiple APIs with different requirements
#[tokio::test]
async fn test_composition_pattern_flexibility() {
    // Create a state that implements all three traits
    #[derive(Clone)]
    struct TestState {
        db: Arc<DatabaseConnection>,
        mongo: Database,
        redis: ConnectionManager,
    }

    impl HasDatabase for TestState {
        fn db(&self) -> &DatabaseConnection {
            &self.db
        }
    }

    impl HasMongoDB for TestState {
        fn mongo(&self) -> &Database {
            &self.mongo
        }
    }

    impl HasRedis for TestState {
        fn redis(&self) -> &ConnectionManager {
            &self.redis
        }
    }

    // Start containers with latest versions
    let postgres_container = Postgres::default()
        .with_tag("17-alpine")
        .start()
        .await
        .expect("Failed to start Postgres container");

    let mongo_container = Mongo::default()
        .with_tag("7")
        .with_startup_timeout(std::time::Duration::from_secs(180))
        .start()
        .await
        .expect("Failed to start MongoDB container");

    let redis_container = Redis::default()
        .with_tag("7-alpine")
        .with_startup_timeout(std::time::Duration::from_secs(180))
        .start()
        .await
        .expect("Failed to start Redis container");

    // Get connection info
    let postgres_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!(
        "postgresql://postgres:postgres@localhost:{}/postgres",
        postgres_port
    );

    let mongo_port = mongo_container.get_host_port_ipv4(27017).await.unwrap();
    let mongo_uri = format!("mongodb://localhost:{}/", mongo_port);

    let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
    let redis_url = format!("redis://localhost:{}/", redis_port);

    // Create test connections
    let db = services::postgres::connect(&db_url).await.unwrap();

    let mongo_client = mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .unwrap();
    let mongo = mongo_client.database("test");

    let redis_client = redis::Client::open(redis_url).unwrap();
    let redis = ConnectionManager::new(redis_client).await.unwrap();

    let state = TestState {
        db: Arc::new(db),
        mongo,
        redis: redis.clone(),
    };

    // This state can now be used with ANY API that requires these traits!

    // Example: ProjectState requires HasDatabase + HasRedis
    fn use_with_project_api<S: HasDatabase + HasRedis>(_state: S) {
        // ProjectState is automatically satisfied
    }

    // Example: CarState requires HasMongoDB + HasRedis
    fn use_with_car_api<S: HasMongoDB + HasRedis>(_state: S) {
        // CarState is automatically satisfied
    }

    // Example: Hypothetical FileState might require HasDatabase + HasMongoDB
    fn use_with_file_api<S: HasDatabase + HasMongoDB>(_state: S) {
        // Would also work without any new implementations!
    }

    // All three work with the same state!
    use_with_project_api(state.clone());
    use_with_car_api(state.clone());
    use_with_file_api(state);
}

/// Demonstrates that you can create specialized states for testing
#[test]
fn test_minimal_state_for_specific_api() {
    // A minimal state that only implements what's needed for Car API
    #[derive(Clone)]
    struct MinimalCarState {
        mongo: mongodb::Database,
        redis: redis::aio::ConnectionManager,
    }

    // Only implement what's needed
    impl HasMongoDB for MinimalCarState {
        fn mongo(&self) -> &Database {
            &self.mongo
        }
    }

    impl HasRedis for MinimalCarState {
        fn redis(&self) -> &ConnectionManager {
            &self.redis
        }
    }

    // This automatically implements CarState through the blanket impl
    fn requires_car_state<S: apis_car::state::CarState>(_state: S) {}

    // Would compile (if we had actual connections)
    // requires_car_state(minimal_state);
}

/// Demonstrates compile-time safety
#[test]
fn test_composition_provides_compile_time_safety() {
    // This demonstrates that you CAN'T use a state with the wrong traits

    #[derive(Clone)]
    struct OnlyHasDatabase {
        db: Arc<DatabaseConnection>,
    }

    impl HasDatabase for OnlyHasDatabase {
        fn db(&self) -> &DatabaseConnection {
            &self.db
        }
    }

    // This would NOT compile:
    // fn try_to_use_with_car_api<S: apis_car::state::CarState>(state: S) {}
    // try_to_use_with_car_api(OnlyHasDatabase { ... });
    //
    // Error: the trait bound `OnlyHasDatabase: HasMongoDB` is not satisfied
    //
    // This is GOOD! Compile-time safety prevents runtime errors.
}

/// Demonstrates that adding new APIs doesn't require modifying existing state
#[test]
fn test_no_modification_needed_for_new_apis() {
    // Imagine we want to add a new "Analytics" API that needs MongoDB + Redis

    pub trait AnalyticsState: HasMongoDB + HasRedis {}
    impl<T: HasMongoDB + HasRedis> AnalyticsState for T {}

    // That's it! Any state that has HasMongoDB + HasRedis (like AppState)
    // automatically implements AnalyticsState without ANY modifications!

    // No need to add:
    // impl AnalyticsState for AppState { ... }
    //
    // It just works! ✨
}
