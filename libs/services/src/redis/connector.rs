use redis::aio::ConnectionManager;
use redis::Client;

// fn get_redis_uri() -> String {
//   Env::get_redis().unwrap()
// }

#[derive(Clone)]
pub struct RedisConnector {
    manager: ConnectionManager,
}

impl RedisConnector {
    pub async fn new() -> redis::RedisResult<Self> {
        let client = Client::open("")?;
        let manager = ConnectionManager::new(client).await?;
        Ok(Self { manager })
    }

    pub fn manager(&self) -> ConnectionManager {
        self.manager.clone()
    }
}

pub async fn connect(url: &str) -> redis::RedisResult<ConnectionManager> {
    tracing::info!("Attempting to connect to Redis");
    let client = Client::open(url)?;
    let manager = ConnectionManager::new(client).await?;

    let mut conn = manager.clone();
    let _: String = redis::cmd("PING").query_async(&mut conn).await?;

    tracing::info!("Successfully connected to Redis");
    Ok(manager)
}
