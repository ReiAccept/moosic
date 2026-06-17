use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

#[derive(Clone)]
pub struct DashMapCache {
    map: Arc<DashMap<String, CacheEntry>>,
}

impl DashMapCache {
    pub fn new() -> Self {
        Self {
            map: Arc::new(DashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let entry = self.map.get(key)?;

        if let Some(exp) = entry.expires_at {
            if Instant::now() >= exp {
                drop(entry);
                self.map.remove(key);
                return None;
            }
        }

        Some(entry.value.clone())
    }

    pub fn set(&self, key: &str, value: &str, ttl: Option<Duration>) {
        let expires_at = ttl.map(|t| Instant::now() + t);
        self.map
            .insert(key.to_owned(), CacheEntry {
                value: value.to_owned(),
                expires_at,
            });
    }

    pub fn del(&self, key: &str) {
        self.map.remove(key);
    }

    pub fn exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}
