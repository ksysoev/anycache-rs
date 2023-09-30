use async_trait::async_trait;
use moka::future::Cache as MokaCache;
use moka::Expiry;
use std::time::{Duration, Instant};

use crate::{Result, Storable};

#[derive(Debug)]
pub struct InMemoryStorage {
    cache: MokaCache<String, (Option<Duration>, String)>,
}

struct MokaExpiry;

impl Expiry<String, (Option<Duration>, String)> for MokaExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &(Option<Duration>, String),
        _current_time: Instant,
    ) -> Option<Duration> {
        value.0
    }
}

impl InMemoryStorage {
    // TODO: For some reason here compiler complains about unused code... not sure why
    #[allow(dead_code)]
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
impl Storable for InMemoryStorage {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        if let Some((_, val)) = self.cache.get(key).await {
            return Ok(Some(val.to_string()));
        }
        Ok(None)
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()> {
        self.cache
            .insert(key.to_string(), (ttl, value.to_string()))
            .await;
        Ok(())
    }

    async fn del(&self, key: &str) -> Result<()> {
        self.cache.remove(key).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Storable;

    #[tokio::test]
    async fn get_set_values() {
        use super::InMemoryStorage;
        let storage = InMemoryStorage::new(10);
        assert_eq!(storage.get("foo").await.unwrap(), None);

        storage.set("foo", "test", None).await.unwrap();

        assert_eq!(storage.get("foo").await.unwrap(), Some("test".to_string()));
    }

    #[tokio::test]
    async fn del_values() {
        use super::InMemoryStorage;
        let storage = InMemoryStorage::new(10);

        storage.set("foo", "test", None).await.unwrap();

        assert_eq!(storage.get("foo").await.unwrap(), Some("test".to_string()));

        storage.del("foo").await.unwrap();

        assert_eq!(storage.get("foo").await.unwrap(), None);
    }
}
