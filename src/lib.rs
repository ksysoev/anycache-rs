/// This module defines a cache implementation that can use different storage backends.
/// It provides a trait `Storable` that defines the interface for interacting with the storage backend.
/// The `Cache` struct is the main entry point for using the cache and it takes a `Storable` implementation as a parameter.
/// The `Cache` struct provides a `cache` method that can be used to cache data and a `invalidate` method that can be used to invalidate cached data.
/// The `CacheOptions` enum defines the options that can be passed to the `cache` method.
/// The `StorageError` enum defines the errors that can occur when interacting with the storage backend.
/// The `StorableTTL` enum defines the TTL (time to live) options that can be used when storing data.
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

#[derive(Debug, Clone)]
pub enum CacheError {
    ConnectionError,
    GeneratorError,
}

#[derive(Debug, PartialEq)]
pub enum StorableTTL {
    TTL(Duration),
    NoTTL,
}

pub type Result<T> = std::result::Result<T, CacheError>;

#[async_trait]
pub trait Storable {
    /// Get the value associated with the given key.
    async fn get(&self, key: &str) -> Result<Option<String>>;

    /// Set the value associated with the given key.
    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()>;

    /// Delete the value associated with the given key.
    async fn del(&self, key: &str) -> Result<()>;

    /// Get the value and TTL associated with the given key.
    async fn get_with_ttl(&self, key: &str) -> Result<Option<(String, StorableTTL)>>;
}

pub struct Cache<S: Storable> {
    storage: S,
    queue: Arc<Mutex<HashMap<String, Vec<oneshot::Sender<Result<String>>>>>>,
}

impl<S: Storable> Cache<S> {
    /// Create a new cache instance with the given storage backend.
    pub fn new<T>(storage: S) -> Self {
        Self {
            storage,
            queue: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Cache the data associated with the given key.
    ///
    /// If the data is already cached, return the cached data.
    /// If the data is not cached, call the `get_data` closure to get the data and cache it.
    /// The `opts` parameter can be used to specify cache options.
    pub async fn cache<C, F>(
        &self,
        key: String,
        get_data: C,
        opts: &[CacheOptions],
    ) -> Result<String>
    where
        F: Future<Output = Result<String>>,
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
        match guard.get_mut(&key) {
            Some(data) => {
                let (sender, receiver) = oneshot::channel::<Result<String>>();
                data.push(sender);
                drop(guard);

                receiver.await.unwrap()
            }
            None => {
                guard.insert(key.clone(), vec![]);
                drop(guard);

                let result = match self.storage.get(&key).await? {
                    Some(data) => Ok(data),
                    None => match get_data().await {
                        Ok(data) => match self.storage.set(&key, &data, ttl).await {
                            Ok(_) => Ok(data),
                            Err(e) => Err(e),
                        },
                        Err(e) => Err(e),
                    },
                };

                let mut guard = queue.lock().await;
                let senders = match guard.remove(&key) {
                    Some(data) => data,
                    None => vec![],
                };
                drop(guard);

                for sender in senders {
                    sender.send(result.clone()).unwrap();
                }

                result
            }
        }
    }

    /// Invalidate the data associated with the given key.
    pub async fn invalidate(&self, key: String) -> Result<()> {
        self.storage.del(&key).await
    }
}
