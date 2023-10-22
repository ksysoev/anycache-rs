use anycache::memcache::MemcacheStorage;
use anycache::{Cache, CacheOptions};
use futures::io::AllowStdIo;
use memcache_async::ascii::Protocol;
use std::env;
use std::net::TcpStream;
use std::sync::Arc;

#[tokio::test]
async fn cache_memcache() {
    let memcache_url = env::var("MEMCACHED_URL").unwrap_or("localhost:11211".to_string());
    let stream = TcpStream::connect(memcache_url).expect("Failed to create stream");
    let memcache = Protocol::new(AllowStdIo::new(stream));
    let storage = MemcacheStorage::new(memcache);
    let cache = Cache::new::<MemcacheStorage>(storage);

    cache
        .invalidate("cache_memcache".to_string())
        .await
        .unwrap();

    let data = cache
        .cache(
            "cache_memcache".to_string(),
            || async { Ok("test".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    let data = cache
        .cache(
            "cache_memcache".to_string(),
            || async { Ok("test2".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    cache
        .invalidate("cache_memcache".to_string())
        .await
        .unwrap();
}

#[tokio::test]
async fn invalidate_memcache() {
    let memcache_url = env::var("MEMCACHED_URL").unwrap_or("localhost:11211".to_string());
    let stream = TcpStream::connect(memcache_url).expect("Failed to create stream");
    let memcache = Protocol::new(AllowStdIo::new(stream));
    let storage = MemcacheStorage::new(memcache);
    let cache = Cache::new::<MemcacheStorage>(storage);

    cache
        .invalidate("invalidate_memcache".to_string())
        .await
        .unwrap();

    let data = cache
        .cache(
            "invalidate_memcache".to_string(),
            || async { Ok("test".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    cache
        .invalidate("invalidate_memcache".to_string())
        .await
        .unwrap();

    let data = cache
        .cache(
            "invalidate_memcache".to_string(),
            || async { Ok("test2".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test2".to_string());

    cache
        .invalidate("invalidate_memcache".to_string())
        .await
        .unwrap();
}

#[tokio::test]
async fn concurrent_memcache() {
    let memcache_url = env::var("MEMCACHED_URL").unwrap_or("localhost:11211".to_string());
    let stream = TcpStream::connect(memcache_url).expect("Failed to create stream");
    let memcache = Protocol::new(AllowStdIo::new(stream));
    let storage = MemcacheStorage::new(memcache);
    let cache = Arc::new(Cache::new::<MemcacheStorage>(storage));

    let cache1 = cache.clone();
    let cache2 = cache.clone();

    cache
        .invalidate("concurrent_memcache".to_string())
        .await
        .unwrap();

    let data1 = tokio::spawn(async move {
        cache1
            .cache(
                "concurrent_memcache".to_string(),
                || async {
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    Ok("test".to_string())
                },
                &[],
            )
            .await
            .unwrap()
    });

    let data2 = tokio::spawn(async move {
        cache2
            .cache(
                "concurrent_memcache".to_string(),
                || async {
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    Ok("test2".to_string())
                },
                &[],
            )
            .await
            .unwrap()
    });

    let data1 = data1.await.unwrap();
    let data2 = data2.await.unwrap();

    assert_eq!(data1, data2);

    cache
        .invalidate("concurrent_memcache".to_string())
        .await
        .unwrap();
}

#[tokio::test]
async fn cache_with_ttl_memcache() {
    let memcache_url = env::var("MEMCACHED_URL").unwrap_or("localhost:11211".to_string());
    let stream = TcpStream::connect(memcache_url).expect("Failed to create stream");
    let memcache = Protocol::new(AllowStdIo::new(stream));
    let storage = MemcacheStorage::new(memcache);
    let cache = Cache::new::<MemcacheStorage>(storage);

    let data = cache
        .cache(
            "cache_with_ttl_memcache".to_string(),
            || async { Ok("test".to_string()) },
            &[CacheOptions::TTL(std::time::Duration::from_secs(2))],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    let data = cache
        .cache(
            "cache_with_ttl_memcache".to_string(),
            || async { Ok("test2".to_string()) },
            &[CacheOptions::TTL(std::time::Duration::from_secs(2))],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let data = cache
        .cache(
            "cache_with_ttl_memcache".to_string(),
            || async { Ok("test3".to_string()) },
            &[CacheOptions::TTL(std::time::Duration::from_secs(2))],
        )
        .await
        .unwrap();
    assert_eq!(data, "test3".to_string());
}

#[tokio::test]
async fn cache_json_memcache() {
    use serde::{Deserialize, Serialize};

    let memcache_url = env::var("MEMCACHED_URL").unwrap_or("localhost:11211".to_string());
    let stream = TcpStream::connect(memcache_url).expect("Failed to create stream");
    let memcache = Protocol::new(AllowStdIo::new(stream));
    let storage = MemcacheStorage::new(memcache);
    let cache = Cache::new::<MemcacheStorage>(storage);

    cache
        .invalidate("cache_json_memcache".to_string())
        .await
        .unwrap();

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Test {
        test: String,
    }

    let data: Test = cache
        .cache_json(
            "cache_json_memcache".to_string(),
            || async {
                Ok(Test {
                    test: "test".to_string(),
                })
            },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(
        data,
        Test {
            test: "test".to_string(),
        }
    );

    let data: Test = cache
        .cache_json(
            "cache_json_memcache".to_string(),
            || async {
                Ok(Test {
                    test: "test1".to_string(),
                })
            },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(
        data,
        Test {
            test: "test".to_string(),
        }
    );

    cache
        .invalidate("cache_json_memcache".to_string())
        .await
        .unwrap();
}
