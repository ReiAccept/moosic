mod memory;
mod redis;

pub use memory::MemoryCache;
pub use redis::RedisCache;

use std::time::Duration;

/// Cache backend abstraction.
///
/// Dispatches between an in-memory [`MemoryCache`] (backed by `DashMap`)
/// and a [`RedisCache`] (backed by a Redis connection), depending on
/// configuration.
#[derive(Clone)]
pub enum CacheBackend {
    /// In-memory cache powered by `DashMap`.
    Memory(MemoryCache),
    /// Redis-backed cache.
    Redis(RedisCache),
}

impl CacheBackend {
    /// Human-readable name of the backend in use.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Memory(_) => "memory",
            Self::Redis(_) => "redis",
        }
    }

    /// Look up a key. Returns `Some(value)` on hit, `None` on miss or
    /// expired entry.
    pub async fn get(&self, key: &str) -> Option<String> {
        match self {
            Self::Memory(c) => c.get(key),
            Self::Redis(c) => c.get(key).await,
        }
    }

    /// Store a value under a key, with an optional time-to-live.
    pub async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) {
        match self {
            Self::Memory(c) => c.set(key, value, ttl),
            Self::Redis(c) => c.set(key, value, ttl).await,
        }
    }

    /// Delete a key. No-op if the key does not exist.
    pub async fn del(&self, key: &str) {
        match self {
            Self::Memory(c) => c.del(key),
            Self::Redis(c) => c.del(key).await,
        }
    }

    /// Check whether a key exists and has not expired.
    pub async fn exists(&self, key: &str) -> bool {
        match self {
            Self::Memory(c) => c.exists(key),
            Self::Redis(c) => c.exists(key).await,
        }
    }
}
