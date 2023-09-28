use async_trait::async_trait;
use moka::future::Cache as MokaCache;

use crate::{Result, Storable};

#[derive(Debug)]
pub struct InMemoryStorage {
    cache: MokaCache<String, String>,
}

impl InMemoryStorage {
    // TODO: For some reason here compiler complains about unused code... not sure why
    #[allow(dead_code)]
    pub fn new(capacity: u64) -> Self {
        Self {
            cache: MokaCache::new(capacity),
        }
    }
}

#[async_trait]
impl Storable for InMemoryStorage {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let val = self.cache.get(key).await;
        Ok(val)
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        self.cache.insert(key.to_string(), value.to_string()).await;
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

        storage.set("foo", "test").await.unwrap();

        assert_eq!(storage.get("foo").await.unwrap(), Some("test".to_string()));
    }

    #[tokio::test]
    async fn del_values() {
        use super::InMemoryStorage;
        let storage = InMemoryStorage::new(10);

        storage.set("foo", "test").await.unwrap();

        assert_eq!(storage.get("foo").await.unwrap(), Some("test".to_string()));

        storage.del("foo").await.unwrap();

        assert_eq!(storage.get("foo").await.unwrap(), None);
    }
}
