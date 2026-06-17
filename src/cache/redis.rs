use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::time::Duration;

#[derive(Clone)]
pub struct RedisCache {
    conn: MultiplexedConnection,
}

impl RedisCache {
    pub async fn connect(url: &str) -> Self {
        let client =
            redis::Client::open(url).expect("Failed to parse Redis URL");
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis");
        Self { conn }
    }

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


    pub async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) {
        let mut conn = self.conn.clone();
        let result: redis::RedisResult<()> = match ttl {
            Some(t) => {
                let seconds = t.as_secs();
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

    pub async fn del(&self, key: &str) {
        let mut conn = self.conn.clone();
        let result: redis::RedisResult<()> = conn.del(key).await;
        if let Err(e) = result {
            tracing::warn!("Redis DEL error for key={key}: {e}");
        }
    }

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
