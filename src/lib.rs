pub mod moka;
pub mod redis;
use async_trait::async_trait;
use futures::channel::oneshot;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum CacheOptions {
    TTL(Duration),
    WarmUpTTL(Duration),
}

#[derive(Debug)]
pub enum StorageError {
    ConnectionError,
    GeneratorError,
}

#[derive(Debug, PartialEq)]
pub enum StorableTTL {
    TTL(Duration),
    NoTTL,
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[async_trait]
pub trait Storable {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()>;
    async fn del(&self, key: &str) -> Result<()>;
    async fn get_with_ttl(&self, key: &str) -> Result<Option<(String, StorableTTL)>>;
}

pub struct Cache<S: Storable> {
    storage: S,
    queue: Arc<Mutex<HashMap<String, Vec<oneshot::Sender<String>>>>>,
}

impl<S: Storable> Cache<S> {
    pub fn new<T>(storage: S) -> Self {
        Self {
            storage,
            queue: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn cache<C, F>(
        &self,
        key: String,
        get_data: C,
        opts: &[CacheOptions],
    ) -> Result<String>
    where
        F: Future<Output = String>,
        C: Fn() -> F,
    {
        let mut ttl = None;
        for opt in opts.iter() {
            match opt {
                CacheOptions::TTL(t) => ttl = Some(*t),
                CacheOptions::WarmUpTTL(_) => {}
            }
        }

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

                let data = match self.storage.get(&key).await? {
                    Some(data) => data,
                    None => {
                        let data = get_data().await;
                        self.storage.set(&key, &data, ttl).await?;
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
        Ok(data)
    }

    pub async fn invalidate(&self, key: String) -> Result<()> {
        self.storage.del(&key).await
    }
}
