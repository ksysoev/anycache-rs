use super::RedisStorage;
use crate::{Storable, StorableTTL};
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
            Some(std::time::Duration::from_millis(10)),
        )
        .await
        .unwrap();

    assert_eq!(
        storage.get("set_with_ttl").await.unwrap(),
        Some("test".to_string())
    );

    tokio::time::sleep(std::time::Duration::from_millis(11)).await;

    assert_eq!(storage.get("set_with_ttl").await.unwrap(), None);
}

#[tokio::test]
async fn get_with_ttl() {
    let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
    let redis = Client::open(redis_url).unwrap();

    let storage = RedisStorage::new(redis);

    storage.del("get_with_ttl").await.unwrap();

    assert_eq!(storage.get_with_ttl("get_with_ttl").await.unwrap(), None);

    storage.set("get_with_ttl", "test", None).await.unwrap();

    assert_eq!(
        storage.get_with_ttl("get_with_ttl").await.unwrap(),
        Some(("test".to_string(), StorableTTL::NoTTL))
    );

    storage
        .set(
            "get_with_ttl",
            "test",
            Some(std::time::Duration::from_millis(10)),
        )
        .await
        .unwrap();

    let result = storage.get_with_ttl("get_with_ttl").await.unwrap();

    match result {
        Some((value, StorableTTL::TTL(ttl))) => {
            assert_eq!(value, "test".to_string());
            assert!(ttl.as_millis() <= 10);
            assert!(ttl.as_millis() > 0);
        }
        _ => panic!("Expected TTL"),
    }
}
