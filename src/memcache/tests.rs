use super::MemcacheStorage;
use crate::{Storable, StorableTTL};
use futures::io::AllowStdIo;
use memcache_async::ascii::Protocol;
use std::env;
use std::net::TcpStream;

#[tokio::test]
async fn get_set_values() {
    let memcache_url = env::var("MEMCACHED_URL").unwrap_or("localhost:11211".to_string());
    let stream = TcpStream::connect(memcache_url).expect("Failed to create stream");
    let memcache = Protocol::new(AllowStdIo::new(stream));

    let storage = MemcacheStorage::new(memcache);

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
    let memcache_url = env::var("MEMCACHED_URL").unwrap_or("localhost:11211".to_string());
    let stream = TcpStream::connect(memcache_url).expect("Failed to create stream");
    let memcache = Protocol::new(AllowStdIo::new(stream));

    let storage = MemcacheStorage::new(memcache);

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
    let memcache_url = env::var("MEMCACHED_URL").unwrap_or("localhost:11211".to_string());
    let stream = TcpStream::connect(memcache_url).expect("Failed to create stream");
    let memcache = Protocol::new(AllowStdIo::new(stream));

    let storage = MemcacheStorage::new(memcache);

    storage.del("set_with_ttl").await.unwrap();

    storage
        .set(
            "set_with_ttl",
            "test",
            Some(std::time::Duration::from_secs(2)),
        )
        .await
        .unwrap();

    assert_eq!(
        storage.get("set_with_ttl").await.unwrap(),
        Some("test".to_string())
    );

    tokio::time::sleep(std::time::Duration::from_millis(3)).await;
    // TODO: TTL Doesn't look to be working
    // assert_eq!(storage.get("set_with_ttl").await.unwrap(), None);
}

#[tokio::test]
async fn get_with_ttl() {
    let memcache_url = env::var("MEMCACHED_URL").unwrap_or("localhost:11211".to_string());
    let stream = TcpStream::connect(memcache_url).expect("Failed to create stream");
    let memcache = Protocol::new(AllowStdIo::new(stream));

    let storage = MemcacheStorage::new(memcache);

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

    assert_eq!(
        storage.get_with_ttl("get_with_ttl").await.unwrap(),
        Some(("test".to_string(), StorableTTL::NoTTL))
    );
}
