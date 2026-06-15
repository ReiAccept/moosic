use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::time::Duration;

/// Redis-backed cache.
///
/// Wraps a [`MultiplexedConnection`] for async Redis operations.
/// Cheap to clone — the connection is multiplexed.
#[derive(Clone)]
pub struct RedisCache {
    conn: MultiplexedConnection,
}

impl RedisCache {
    /// Wrap an existing Redis multiplexed connection.
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }

    /// Look up a key. Returns `Some(value)` on hit, `None` on miss.
    pub async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.conn.clone();
        let result: redis::RedisResult<String> = conn.get(key).await;
        match result {
            Ok(val) => Some(val),
            Err(e) => {
                tracing::warn!("Redis GET error for key={key}: {e}");
                None
            }
        }
    }

    /// Store a value under a key, with an optional time-to-live.
    pub async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) {
        let mut conn = self.conn.clone();
        let result: redis::RedisResult<()> = match ttl {
            Some(t) => {
                let seconds = t.as_secs();
                // SETEX: set with expiration in seconds
                conn.set_ex(key, value, seconds).await
            }
            None => {
                conn.set(key, value).await
            }
        };
        if let Err(e) = result {
            tracing::warn!("Redis SET error for key={key}: {e}");
        }
    }

    /// Delete a key. No-op if the key does not exist.
    pub async fn del(&self, key: &str) {
        let mut conn = self.conn.clone();
        let result: redis::RedisResult<()> = conn.del(key).await;
        if let Err(e) = result {
            tracing::warn!("Redis DEL error for key={key}: {e}");
        }
    }

    /// Check whether a key exists.
    pub async fn exists(&self, key: &str) -> bool {
        let mut conn = self.conn.clone();
        let result: redis::RedisResult<bool> = conn.exists(key).await;
        match result {
            Ok(exists) => exists,
            Err(e) => {
                tracing::warn!("Redis EXISTS error for key={key}: {e}");
                false
            }
        }
    }
}
