/// Tests demonstrating the composition pattern benefits
mod common;

use app::state::{HasDatabase, HasMongoDB, HasRedis};
use mongodb::Database;
use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

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

    // Use shared containers for better performance
    let app_state = common::create_test_state().await;

    let state = TestState {
        db: Arc::new(app_state.db().clone()),
        mongo: app_state.mongo().clone(),
        redis: app_state.redis().clone(),
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
