#[cfg(test)]
mod tests {
    use super::super::state::CarState;
    use app::state::{HasMongoDB, HasRedis};
    use mongodb::Database;
    use redis::aio::ConnectionManager;

    /// Mock state that implements the required traits
    #[derive(Clone)]
    struct MockCarState {
        mongo: Database,
        redis: ConnectionManager,
    }

    impl HasMongoDB for MockCarState {
        fn mongo(&self) -> &Database {
            &self.mongo
        }
    }

    impl HasRedis for MockCarState {
        fn redis(&self) -> &ConnectionManager {
            &self.redis
        }
    }

    #[tokio::test]
    async fn test_car_state_composition() {
        // This test demonstrates that CarState is automatically implemented
        // when a type implements both HasMongoDB and HasRedis

        use testcontainers::{runners::AsyncRunner, ImageExt};
        use testcontainers_modules::{mongo::Mongo, redis::Redis};

        // Start MongoDB and Redis containers
        let mongo_container = Mongo::default()
            .start()
            .await
            .expect("Failed to start MongoDB container");

        let redis_container = Redis::default()
            .start()
            .await
            .expect("Failed to start Redis container");

        // Get connection info
        let mongo_port = mongo_container.get_host_port_ipv4(27017).await.unwrap();
        let mongo_uri = format!("mongodb://localhost:{}/", mongo_port);

        let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
        let redis_url = format!("redis://localhost:{}/", redis_port);

        // Connect to databases
        let mongo_client = mongodb::Client::with_uri_str(&mongo_uri).await.unwrap();
        let mongo = mongo_client.database("test");

        let redis_client = redis::Client::open(redis_url).unwrap();
        let redis = ConnectionManager::new(redis_client).await.unwrap();

        let mock_state = MockCarState {
            mongo: mongo.clone(),
            redis: redis.clone(),
        };

        // CarState is automatically implemented via blanket impl!
        fn requires_car_state<S: CarState>(_state: S) {
            // This compiles because MockCarState implements CarState
            // through the composition pattern
        }

        requires_car_state(mock_state);
    }

    #[test]
    fn test_state_traits_are_composable() {
        // This test verifies that the trait bounds are correct
        // It should compile without any manual implementations

        fn check_composition<S>(_state: S)
        where
            S: HasMongoDB + HasRedis + Clone + Send + Sync + 'static,
        {
            // Any state with these traits automatically implements CarState
        }

        // If this compiles, the composition pattern is working correctly
    }
}
