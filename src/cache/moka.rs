use moka::future::Cache;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

#[derive(Clone)]
pub struct MokaCache {
    cache: Cache<String, CacheEntry>,
}

impl MokaCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder().build(),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let entry = self.cache.get(key).await?;

        // Check expiration
        if let Some(exp) = entry.expires_at {
            if Instant::now() >= exp {
                self.cache.remove(key).await;
                return None;
            }
        }

        Some(entry.value)
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) {
        let expires_at = ttl.map(|t| Instant::now() + t);
        self.cache
            .insert(
                key.to_owned(),
                CacheEntry {
                    value: value.to_owned(),
                    expires_at,
                },
            )
            .await;
    }

    pub async fn del(&self, key: &str) {
        self.cache.remove(key).await;
    }

    pub async fn exists(&self, key: &str) -> bool {
        self.get(key).await.is_some()
    }
}

impl Default for MokaCache {
    fn default() -> Self {
        Self::new()
    }
}
