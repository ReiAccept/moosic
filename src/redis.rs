use redis::aio::MultiplexedConnection;

use crate::config::Redis;

/// Connect to Redis and return a multiplexed async connection.
pub async fn connect(config: &Redis) -> MultiplexedConnection {
    let client = redis::Client::open(config.url.as_str())
        .expect("Failed to parse Redis URL");

    client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis")
}
