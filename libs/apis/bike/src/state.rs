use app::state::{HasRedis, HasSqlxPool};

/// BikeState trait using composition pattern.
/// Requires PostgreSQL (via sqlx using HasSqlxPool) and Redis (via HasRedis).
pub trait BikeState: HasSqlxPool + HasRedis {}

/// Blanket implementation: any type with HasSqlxPool + HasRedis automatically implements BikeState.
impl<T> BikeState for T where T: HasSqlxPool + HasRedis {}
