mod memory;

use std::future::Future;

pub trait Storable {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn del(&self, key: &str);
}

pub struct Cache<S: Storable> {
    storage: S,
}

impl<S: Storable> Cache<S> {
    pub fn new<T>(storage: S) -> Self {
        Self { storage }
    }

    pub async fn cache<F, Fut>(&self, key: String, get_data: F) -> String
    where
        F: Fn() -> Fut,
        Fut: Future<Output = String>,
    {
        let data = self.storage.get(&key);
        match data {
            Some(data) => data,
            None => {
                let data = get_data().await;
                self.storage.set(&key, &data);
                data
            }
        }
    }

    pub fn invalidate(&self, key: String) {
        self.storage.del(&key);
    }
}

mod tests {
    #[tokio::test]
    async fn cache() {
        use super::memory::InMemoryStorage;
        use super::Cache;

        let storage = InMemoryStorage::new(10);
        let cache = Cache::new::<InMemoryStorage>(storage);

        let data = cache
            .cache("foo".to_string(), || async { "test".to_string() })
            .await;
        assert_eq!(data, "test".to_string());

        let data = cache
            .cache("foo".to_string(), || async { "test2".to_string() })
            .await;
        assert_eq!(data, "test".to_string());
    }

    #[tokio::test]
    async fn invalidate() {
        use super::memory::InMemoryStorage;
        use super::Cache;

        let storage = InMemoryStorage::new(10);
        let cache = Cache::new::<InMemoryStorage>(storage);

        let data = cache
            .cache("foo".to_string(), || async { "test".to_string() })
            .await;
        assert_eq!(data, "test".to_string());

        cache.invalidate("foo".to_string());

        let data = cache
            .cache("foo".to_string(), || async { "test2".to_string() })
            .await;
        assert_eq!(data, "test2".to_string());
    }
}
