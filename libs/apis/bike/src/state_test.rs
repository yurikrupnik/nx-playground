#[cfg(test)]
mod tests {
    use super::super::state::BikeState;
    use app::state::{HasRedis, HasSqlxPool};
    use redis::aio::ConnectionManager;
    use sqlx::PgPool;
    use testcontainers::{runners::AsyncRunner, ImageExt};
    use testcontainers_modules::{postgres::Postgres, redis::Redis};

    #[derive(Clone)]
    struct MockBikeState {
        sqlx_pool: PgPool,
        redis: ConnectionManager,
    }

    impl HasSqlxPool for MockBikeState {
        fn sqlx_pool(&self) -> &PgPool {
            &self.sqlx_pool
        }
    }

    impl HasRedis for MockBikeState {
        fn redis(&self) -> &ConnectionManager {
            &self.redis
        }
    }

    #[tokio::test]
    async fn test_bike_state_composition() {
        let postgres_container = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .expect("Failed to start Postgres container");

        let redis_container = Redis::default()
            .start()
            .await
            .expect("Failed to start Redis container");

        let postgres_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
        let db_url = format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            postgres_port
        );

        let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();
        let redis_url = format!("redis://localhost:{}/", redis_port);

        let sqlx_pool = PgPool::connect(&db_url).await.unwrap();
        let redis_client = redis::Client::open(redis_url).unwrap();
        let redis = ConnectionManager::new(redis_client).await.unwrap();

        let mock_state = MockBikeState {
            sqlx_pool,
            redis: redis.clone(),
        };

        // Verify that MockBikeState satisfies BikeState trait requirements
        fn requires_bike_state<S: BikeState>(_state: S) {}
        requires_bike_state(mock_state);
    }

    #[test]
    fn test_state_traits_are_composable() {
        // Compile-time verification that the composition pattern works
        fn check_composition<S>(_state: S)
        where
            S: HasSqlxPool + HasRedis + Clone + Send + Sync + 'static,
        {
        }
    }
}
