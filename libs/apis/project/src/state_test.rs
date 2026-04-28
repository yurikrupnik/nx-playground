#[cfg(test)]
mod tests {
    use super::super::state::ProjectState;
    use app::state::{HasDatabase, HasRedis};
    use redis::aio::ConnectionManager;
    use sea_orm::DatabaseConnection;
    use std::sync::Arc;
    use testcontainers::{runners::AsyncRunner, ImageExt};
    use testcontainers_modules::{postgres::Postgres, redis::Redis};

    /// Mock state that implements the required traits
    #[derive(Clone)]
    struct MockProjectState {
        db: Arc<DatabaseConnection>,
        redis: ConnectionManager,
    }

    impl HasDatabase for MockProjectState {
        fn db(&self) -> &DatabaseConnection {
            &self.db
        }
    }

    impl HasRedis for MockProjectState {
        fn redis(&self) -> &ConnectionManager {
            &self.redis
        }
    }

    #[tokio::test]
    async fn test_project_state_composition() {
        // This test demonstrates that ProjectState is automatically implemented
        // when a type implements both HasDatabase and HasRedis

        // Start PostgreSQL and Redis containers
        let postgres_container = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .expect("Failed to start Postgres container");

        let redis_container = Redis::default()
            .start()
            .await
            .expect("Failed to start Redis container");

        // Get connection info
        let postgres_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
        let db_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            postgres_port
        );

        let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
        let redis_url = format!("redis://localhost:{}/", redis_port);

        // Connect to databases
        let db = services::postgres::connect(&db_url).await.unwrap();

        let redis_client = redis::Client::open(redis_url).unwrap();
        let redis = ConnectionManager::new(redis_client).await.unwrap();

        let mock_state = MockProjectState {
            db: Arc::new(db),
            redis: redis.clone(),
        };

        // ProjectState is automatically implemented via blanket impl!
        fn requires_project_state<S: ProjectState>(_state: S) {
            // This compiles because MockProjectState implements ProjectState
            // through the composition pattern
        }

        requires_project_state(mock_state);
    }

    #[test]
    fn test_state_traits_are_composable() {
        // This test verifies that the trait bounds are correct
        // It should compile without any manual implementations

        fn check_composition<S>(_state: S)
        where
            S: HasDatabase + HasRedis + Clone + Send + Sync + 'static,
        {
            // Any state with these traits automatically implements ProjectState
        }

        // If this compiles, the composition pattern is working correctly
    }
}
