use std::time::Duration;

use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands, Client};

use crate::{Result, Storable, StorableTTL, StorageError};

pub struct RedisStorage {
    redis: Client,
}

impl RedisStorage {
    // TODO: For some reason here compiler complains about unused code... not sure why
    #[allow(dead_code)]
    pub fn new(redis: Client) -> Self {
        Self { redis }
    }

    async fn get_conn(&self) -> Connection {
        self.redis.get_async_connection().await.unwrap()
    }
}

#[async_trait]
impl Storable for RedisStorage {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_conn().await;
        conn.get(key)
            .await
            .map_err(|_| StorageError::ConnectionError)
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<()> {
        let mut conn = self.get_conn().await;

        match ttl {
            Some(ttl) => conn
                .pset_ex(key, value, ttl.as_millis() as usize)
                .await
                .map_err(|_| StorageError::ConnectionError),
            None => conn
                .set(key, value)
                .await
                .map_err(|_| StorageError::ConnectionError),
        }
    }

    async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.get_conn().await;
        conn.del(key)
            .await
            .map_err(|_| StorageError::ConnectionError)
    }

    async fn get_with_ttl(&self, key: &str) -> Result<Option<(String, StorableTTL)>> {
        let mut conn = self.get_conn().await;

        let result: Option<String> = conn
            .get(key)
            .await
            .map_err(|_| StorageError::ConnectionError)?;

        match result {
            None => Ok(None),
            Some(val) => {
                let ttl: isize = conn
                    .pttl(key)
                    .await
                    .map_err(|_| StorageError::ConnectionError)?;

                match ttl {
                    -1 => return Ok(Some((val, StorableTTL::NoTTL))),
                    -2 => {
                        return Ok(Some((
                            val,
                            StorableTTL::TTL(Duration::from_millis(ttl as u64)),
                        )))
                    }
                    _ => {
                        return Ok(Some((
                            val,
                            StorableTTL::TTL(Duration::from_millis(ttl as u64)),
                        )))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
