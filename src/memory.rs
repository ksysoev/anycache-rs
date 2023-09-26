use moka::sync::Cache as MokaCache;

use crate::Storable;

#[derive(Debug)]
pub struct InMemoryStorage {
    cache: MokaCache<String, String>,
}

impl InMemoryStorage {
    pub fn new(capacity: u64) -> Self {
        Self {
            cache: MokaCache::new(capacity),
        }
    }
}

impl Storable for InMemoryStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key)
    }

    fn set(&self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    fn del(&self, key: &str) {
        self.cache.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use crate::Storable;

    #[test]
    fn get_set_values() {
        use super::InMemoryStorage;
        let storage = InMemoryStorage::new(10);
        assert_eq!(storage.get("foo"), None);

        storage.set("foo", "test");

        assert_eq!(storage.get("foo"), Some("test".to_string()));
    }

    #[test]
    fn del_values() {
        use super::InMemoryStorage;
        let storage = InMemoryStorage::new(10);

        storage.set("foo", "test");

        assert_eq!(storage.get("foo"), Some("test".to_string()));

        storage.del("foo");

        assert_eq!(storage.get("foo"), None);
    }
}
