mod memory;
mod redis;
use async_trait::async_trait;
use futures::channel::oneshot;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait Storable {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: &str);
    async fn del(&self, key: &str);
}

pub struct Cache<S: Storable> {
    storage: S,
    queue: Arc<Mutex<HashMap<String, Vec<oneshot::Sender<String>>>>>,
}

impl<S: Storable> Cache<S> {
    pub fn new<T>(storage: S) -> Self {
        Self {
            storage: storage,
            queue: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn cache<C, F>(&self, key: String, get_data: C) -> String
    where
        F: Future<Output = String>,
        C: Fn() -> F,
    {
        let queue = self.queue.clone();
        let mut guard = queue.lock().await;
        let data = match guard.get_mut(&key) {
            Some(data) => {
                let (sender, receiver) = oneshot::channel::<String>();
                data.push(sender);
                drop(guard);

                receiver.await.unwrap()
            }
            None => {
                guard.insert(key.clone(), vec![]);
                drop(guard);

                let data = match self.storage.get(&key).await {
                    Some(data) => data,
                    None => {
                        let data = get_data().await;
                        self.storage.set(&key, &data).await;
                        data
                    }
                };

                let mut guard = queue.lock().await;
                let senders = match guard.remove(&key) {
                    Some(data) => data,
                    None => vec![],
                };
                drop(guard);

                for sender in senders {
                    sender.send(data.clone()).unwrap();
                }

                data
            }
        };
        data
    }

    pub async fn invalidate(&self, key: String) {
        self.storage.del(&key).await;
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

        cache.invalidate("foo".to_string()).await;

        let data = cache
            .cache("foo".to_string(), || async { "test2".to_string() })
            .await;
        assert_eq!(data, "test2".to_string());
    }

    #[tokio::test]
    async fn concurrent_requests() {
        use super::memory::InMemoryStorage;
        use super::Cache;
        use std::sync::Arc;
        // use tokio::sync::Mutex;

        let storage = InMemoryStorage::new(10);
        let cache = Arc::new(Cache::new::<InMemoryStorage>(storage));

        let cache1 = cache.clone();
        let cache2 = cache.clone();

        let data1 = tokio::spawn(async move {
            cache1
                .cache("foo".to_string(), || async {
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    "test".to_string()
                })
                .await
        });

        let data2 = tokio::spawn(async move {
            cache2
                .cache("foo".to_string(), || async {
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    "test2".to_string()
                })
                .await
        });

        let data1 = data1.await.unwrap();
        let data2 = data2.await.unwrap();

        assert_eq!(data1, data2);
    }
}
