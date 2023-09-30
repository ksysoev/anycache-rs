#[tokio::test]
async fn cache_redis() {
    use anycache::redis::RedisStorage;
    use anycache::Cache;
    use redis::Client;
    use std::env;

    let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
    let storage = RedisStorage::new(Client::open(redis_url).unwrap());
    let cache = Cache::new::<RedisStorage>(storage);

    cache.invalidate("cache_redis".to_string()).await.unwrap();

    let data = cache
        .cache(
            "cache_redis".to_string(),
            || async { Ok("test".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    let data = cache
        .cache(
            "cache_redis".to_string(),
            || async { Ok("test2".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    cache.invalidate("cache_redis".to_string()).await.unwrap();
}

#[tokio::test]
async fn invalidate_redis() {
    use anycache::redis::RedisStorage;
    use anycache::Cache;
    use redis::Client;
    use std::env;

    let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
    let storage = RedisStorage::new(Client::open(redis_url).unwrap());
    let cache = Cache::new::<RedisStorage>(storage);

    cache
        .invalidate("invalidate_redis".to_string())
        .await
        .unwrap();

    let data = cache
        .cache(
            "invalidate_redis".to_string(),
            || async { Ok("test".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    cache
        .invalidate("invalidate_redis".to_string())
        .await
        .unwrap();

    let data = cache
        .cache(
            "invalidate_redis".to_string(),
            || async { Ok("test2".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test2".to_string());

    cache
        .invalidate("invalidate_redis".to_string())
        .await
        .unwrap();
}

#[tokio::test]
async fn concurrent_redis() {
    use anycache::redis::RedisStorage;
    use anycache::Cache;
    use redis::Client;
    use std::env;
    use std::sync::Arc;

    let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
    let storage = RedisStorage::new(Client::open(redis_url).unwrap());
    let cache = Arc::new(Cache::new::<RedisStorage>(storage));

    let cache1 = cache.clone();
    let cache2 = cache.clone();

    cache
        .invalidate("concurrent_redis".to_string())
        .await
        .unwrap();

    let data1 = tokio::spawn(async move {
        cache1
            .cache(
                "concurrent_redis".to_string(),
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
                "concurrent_redis".to_string(),
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
        .invalidate("concurrent_redis".to_string())
        .await
        .unwrap();
}

#[tokio::test]
async fn cache_with_ttl_redis() {
    use anycache::redis::RedisStorage;
    use anycache::Cache;
    use anycache::CacheOptions;
    use redis::Client;
    use std::env;

    let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());
    let storage = RedisStorage::new(Client::open(redis_url).unwrap());
    let cache = Cache::new::<RedisStorage>(storage);

    let data = cache
        .cache(
            "cache_with_ttl_moka".to_string(),
            || async { Ok("test".to_string()) },
            &[CacheOptions::TTL(std::time::Duration::from_millis(10))],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    let data = cache
        .cache(
            "cache_with_ttl_moka".to_string(),
            || async { Ok("test2".to_string()) },
            &[CacheOptions::TTL(std::time::Duration::from_millis(10))],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    tokio::time::sleep(std::time::Duration::from_millis(11)).await;

    let data = cache
        .cache(
            "cache_with_ttl_moka".to_string(),
            || async { Ok("test3".to_string()) },
            &[CacheOptions::TTL(std::time::Duration::from_millis(10))],
        )
        .await
        .unwrap();
    assert_eq!(data, "test3".to_string());
}
