use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands, Client};

use crate::{Result, Storable, StorageError};

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

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.get_conn().await;
        conn.set(key, value)
            .await
            .map_err(|_| StorageError::ConnectionError)
    }

    async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.get_conn().await;
        conn.del(key)
            .await
            .map_err(|_| StorageError::ConnectionError)
    }
}

#[cfg(test)]
mod tests {
    use super::RedisStorage;
    use crate::Storable;
    use redis::Client;
    use std::env;

    #[tokio::test]
    async fn get_set_values() {
        let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
        let redis = Client::open(redis_url).unwrap();

        let storage = RedisStorage::new(redis);

        assert_eq!(storage.get("get_set_values").await.unwrap(), None);

        storage.set("get_set_values", "test").await.unwrap();

        assert_eq!(
            storage.get("get_set_values").await.unwrap(),
            Some("test".to_string())
        );

        // Cleanup
        storage.del("get_set_values").await.unwrap();
    }

    #[tokio::test]
    async fn del_values() {
        let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
        let redis = Client::open(redis_url).unwrap();

        let storage = RedisStorage::new(redis);

        storage.set("del_values", "test").await.unwrap();

        assert_eq!(
            storage.get("del_values").await.unwrap(),
            Some("test".to_string())
        );

        storage.del("del_values").await.unwrap();

        assert_eq!(storage.get("del_values").await.unwrap(), None);
    }
}
