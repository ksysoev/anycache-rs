use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands, Client};

use crate::Storable;

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
    async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.get_conn().await;
        conn.get(key).await.unwrap()
    }

    async fn set(&self, key: &str, value: &str) {
        let mut conn = self.get_conn().await;
        let _: () = conn.set(key, value).await.unwrap();
    }

    async fn del(&self, key: &str) {
        let mut conn = self.get_conn().await;
        let _: () = conn.del(key).await.unwrap();
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
        let mut redis_url = env::var("REDIS_URL").unwrap();
        if redis_url == "" {
            redis_url = "redis://127.0.0.1:6379".to_string();
        }

        let redis = Client::open(redis_url).unwrap();

        let storage = RedisStorage::new(redis);
        let _: () = storage.del("get_set_values").await;

        assert_eq!(storage.get("get_set_values").await, None);

        storage.set("get_set_values", "test").await;

        assert_eq!(
            storage.get("get_set_values").await,
            Some("test".to_string())
        );

        // Cleanup
        let _: () = storage.del("get_set_values").await;
    }

    #[tokio::test]
    async fn del_values() {
        let mut redis_url = env::var("REDIS_URL").unwrap();
        if redis_url == "" {
            redis_url = "redis://127.0.0.1:6379".to_string();
        }

        let redis = Client::open(redis_url).unwrap();

        let storage = RedisStorage::new(redis);

        storage.set("del_values", "test").await;

        assert_eq!(storage.get("del_values").await, Some("test".to_string()));

        storage.del("del_values").await;

        assert_eq!(storage.get("del_values").await, None);
    }
}
