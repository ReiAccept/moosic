use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// An entry stored in the cache with an optional expiration.
#[derive(Clone)]
struct CacheEntry {
    value: String,
    /// Instant after which this entry is considered expired.
    expires_at: Option<Instant>,
}

/// In-memory cache backed by [`DashMap`] inside an [`Arc`].
///
/// Cheap to clone — clones share the same underlying map.
#[derive(Clone)]
pub struct MemoryCache {
    map: Arc<DashMap<String, CacheEntry>>,
}

impl MemoryCache {
    /// Create a new empty in-memory cache.
    pub fn new() -> Self {
        Self {
            map: Arc::new(DashMap::new()),
        }
    }

    /// Look up a key. Returns `Some(value)` on hit, `None` on miss or
    /// expired entry (expired entries are removed on access).
    pub fn get(&self, key: &str) -> Option<String> {
        let entry = self.map.get(key)?;

        // Check expiration
        if let Some(exp) = entry.expires_at {
            if Instant::now() >= exp {
                drop(entry); // release the read lock before removing
                self.map.remove(key);
                return None;
            }
        }

        Some(entry.value.clone())
    }

    /// Store a value under a key, with an optional time-to-live.
    pub fn set(&self, key: &str, value: &str, ttl: Option<Duration>) {
        let expires_at = ttl.map(|t| Instant::now() + t);
        self.map
            .insert(key.to_owned(), CacheEntry {
                value: value.to_owned(),
                expires_at,
            });
    }

    /// Delete a key. No-op if the key does not exist.
    pub fn del(&self, key: &str) {
        self.map.remove(key);
    }

    /// Check whether a key exists and has not expired.
    pub fn exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}
