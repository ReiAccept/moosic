mod moka;
mod dashmap;
mod redis;

pub use moka::MokaCache;
pub use dashmap::DashMapCache;
pub use redis::RedisCache;

use std::time::Duration;

#[derive(Clone)]
pub enum CacheBackend {
    DashMap(DashMapCache),
    Moka(MokaCache),
    Redis(RedisCache),
}

impl CacheBackend {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Moka(_) => "moka",
            Self::DashMap(_) => "dashmap",
            Self::Redis(_) => "redis",
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        match self {
            Self::Moka(c) => c.get(key).await,
            Self::DashMap(c) => c.get(key),
            Self::Redis(c) => c.get(key).await,
        }
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) {
        match self {
            Self::Moka(c) => c.set(key, value, ttl).await,
            Self::DashMap(c) => c.set(key, value, ttl),
            Self::Redis(c) => c.set(key, value, ttl).await,
        }
    }

    pub async fn del(&self, key: &str) {
        match self {
            Self::Moka(c) => c.del(key).await,
            Self::DashMap(c) => c.del(key),
            Self::Redis(c) => c.del(key).await,
        }
    }

    pub async fn exists(&self, key: &str) -> bool {
        match self {
            Self::Moka(c) => c.exists(key).await,
            Self::DashMap(c) => c.exists(key),
            
            Self::Redis(c) => c.exists(key).await,
        }
    }
}
