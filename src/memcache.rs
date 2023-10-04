use async_trait::async_trait;
use futures::io::AllowStdIo;
use memcache_async::ascii::Protocol;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::time::Duration;

use crate::{CacheError, Result, Storable, StorableTTL};
use std::sync::Arc;
use tokio::sync::Mutex;

type MemcacheClient = Protocol<AllowStdIo<TcpStream>>;

struct MemcacheStorage {
    client: Arc<Mutex<MemcacheClient>>,
}

impl MemcacheStorage {
    pub fn new(client: MemcacheClient) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }
}

#[async_trait]
impl Storable for MemcacheStorage {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut client = self.client.lock().await;
        match client.get(key).await {
            Ok(value) => Ok(Some(String::from_utf8(value).unwrap())),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(_) => Err(CacheError::ConnectionError),
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()> {
        let ttl_in_sec = match ttl {
            Some(ttl) => ttl.as_secs() as u32,
            None => 0,
        };

        let mut client = self.client.lock().await;
        match client.set(key, value.as_bytes(), ttl_in_sec).await {
            Ok(_) => Ok(()),
            Err(_) => Err(CacheError::ConnectionError),
        }
    }

    async fn del(&self, key: &str) -> Result<()> {
        let mut client = self.client.lock().await;
        match client.delete(key).await {
            Ok(_) => Ok(()),
            Err(_) => Err(CacheError::ConnectionError),
        }
    }

    async fn get_with_ttl(&self, key: &str) -> Result<Option<(String, StorableTTL)>> {
        match self.get(key).await? {
            Some(value) => Ok(Some((value, StorableTTL::NoTTL))),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests;
