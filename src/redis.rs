use std::time::Duration;

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

        storage.set("get_set_values", "test", None).await.unwrap();

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

        storage.set("del_values", "test", None).await.unwrap();

        assert_eq!(
            storage.get("del_values").await.unwrap(),
            Some("test".to_string())
        );

        storage.del("del_values").await.unwrap();

        assert_eq!(storage.get("del_values").await.unwrap(), None);
    }

    #[tokio::test]
    async fn set_with_ttl() {
        let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
        let redis = Client::open(redis_url).unwrap();

        let storage = RedisStorage::new(redis);

        storage.del("set_with_ttl").await.unwrap();

        storage
            .set(
                "set_with_ttl",
                "test",
                Some(std::time::Duration::from_millis(2)),
            )
            .await
            .unwrap();

        assert_eq!(
            storage.get("set_with_ttl").await.unwrap(),
            Some("test".to_string())
        );

        tokio::time::sleep(std::time::Duration::from_millis(3)).await;

        assert_eq!(storage.get("set_with_ttl").await.unwrap(), None);
    }
}
