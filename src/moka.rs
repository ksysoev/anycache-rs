use async_trait::async_trait;
use moka::future::Cache as MokaCache;
use moka::Expiry;
use std::time::{Duration, Instant, SystemTime};

use crate::{Result, Storable, StorableTTL};

/// A struct representing a storage using the Moka cache.
#[derive(Debug)]
pub struct MokaStorage {
    cache: MokaCache<String, CacheItem>,
}

#[derive(Debug, Clone)]
struct CacheItem {
    value: String,
    expiry_time: Option<SystemTime>,
}

impl CacheItem {
    fn new(value: String, expiry_time: Option<SystemTime>) -> Self {
        Self { value, expiry_time }
    }

    fn ttl(&self) -> Option<Duration> {
        match self.expiry_time {
            Some(expiry_time) => {
                let now = SystemTime::now();
                if expiry_time > now {
                    Some(expiry_time.duration_since(now).unwrap())
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

struct MokaExpiry;

impl Expiry<String, CacheItem> for MokaExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &CacheItem,
        _current_time: Instant,
    ) -> Option<Duration> {
        value.ttl()
    }
}

impl MokaStorage {
    /// Creates a new `MokaStorage` instance with the specified capacity.
    pub fn new(capacity: u64) -> Self {
        let expiry = MokaExpiry;
        let cache = MokaCache::builder()
            .max_capacity(capacity)
            .expire_after(expiry)
            .build();
        Self { cache }
    }
}

#[async_trait]
impl Storable for MokaStorage {
    /// Retrieves the value associated with the specified key from the cache.
    async fn get(&self, key: &str) -> Result<Option<String>> {
        if let Some(item) = self.cache.get(key).await {
            return Ok(Some(item.value));
        }
        Ok(None)
    }

    /// Inserts a key-value pair into the cache with an optional time-to-live (TTL).
    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()> {
        match ttl {
            Some(ttl) => {
                let expiry_time = SystemTime::now() + ttl;
                let item = CacheItem::new(value.to_string(), Some(expiry_time));
                self.cache.insert(key.to_string(), item).await;
            }
            None => {
                let item = CacheItem::new(value.to_string(), None);
                self.cache.insert(key.to_string(), item).await;
            }
        }

        Ok(())
    }

    /// Removes the key-value pair associated with the specified key from the cache.
    async fn del(&self, key: &str) -> Result<()> {
        self.cache.remove(key).await;
        Ok(())
    }

    /// Retrieves the value associated with the specified key from the cache along with its TTL.
    async fn get_with_ttl(&self, key: &str) -> Result<Option<(String, StorableTTL)>> {
        if let Some(item) = self.cache.get(key).await {
            match item.ttl() {
                Some(ttl) => {
                    return Ok(Some((item.value, StorableTTL::TTL(ttl))));
                }
                None => {
                    return Ok(Some((item.value, StorableTTL::NoTTL)));
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
