use async_trait::async_trait;
use futures::io::AllowStdIo;
use memcache_async::ascii::Protocol;
use std::net::TcpStream;
use std::time::Duration;

use crate::{CacheError, Result, Storable, StorableTTL};

type MemcacheClient = Protocol<AllowStdIo<TcpStream>>;

struct MemcacheStorage {
    client: MemcacheClient,
}

impl MemcacheStorage {
    pub fn new(client: MemcacheClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Storable for MemcacheStorage {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        match self.client.get(key).await {
            Ok(value) => Ok(Some(String::from_utf8(value).unwrap())),
            Err(_) => Err(CacheError::ConnectionError),
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()> {
        let ttl_in_sec = match ttl {
            Some(ttl) => ttl.as_secs() as u32,
            None => 0,
        };

        match self.client.set(key, value.as_bytes(), ttl_in_sec).await {
            Ok(_) => Ok(()),
            Err(_) => Err(CacheError::ConnectionError),
        }
    }

    async fn del(&self, key: &str) -> Result<()> {
        match self.client.delete(key).await {
            Ok(_) => Ok(()),
            Err(_) => Err(CacheError::ConnectionError),
        }
    }

    async fn get_with_ttl(&self, key: &str) -> Result<Option<(String, StorableTTL)>> {
        match self.client.get(key).await {
            Ok(value) => Ok(Some((
                String::from_utf8(value).unwrap(),
                StorableTTL::NoTTL,
            ))),
            Err(_) => Err(CacheError::ConnectionError),
        }
    }
}

#[cfg(test)]
mod tests;
