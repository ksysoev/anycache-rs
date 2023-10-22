#[tokio::test]
async fn cache_moka() {
    use anycache::moka::MokaStorage;
    use anycache::Cache;

    let storage = MokaStorage::new(10);
    let cache = Cache::new::<MokaStorage>(storage);
    let data = cache
        .cache(
            "cache_moka".to_string(),
            || async { Ok("test".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    let data = cache
        .cache(
            "cache_moka".to_string(),
            || async { Ok("test2".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());
}

#[tokio::test]
async fn invalidate_moka() {
    use anycache::moka::MokaStorage;
    use anycache::Cache;

    let storage = MokaStorage::new(10);
    let cache = Cache::new::<MokaStorage>(storage);

    let data = cache
        .cache(
            "invalidate_moka".to_string(),
            || async { Ok("test".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    cache
        .invalidate("invalidate_moka".to_string())
        .await
        .unwrap();

    let data = cache
        .cache(
            "invalidate_moka".to_string(),
            || async { Ok("test2".to_string()) },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(data, "test2".to_string());
}

#[tokio::test]
async fn concurrent_moka() {
    use anycache::moka::MokaStorage;
    use anycache::Cache;
    use std::sync::Arc;
    // use tokio::sync::Mutex;

    let storage = MokaStorage::new(10);
    let cache = Arc::new(Cache::new::<MokaStorage>(storage));

    let cache1 = cache.clone();
    let cache2 = cache.clone();

    let data1 = tokio::spawn(async move {
        cache1
            .cache(
                "concurrent_moka".to_string(),
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
                "concurrent_moka".to_string(),
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
}

#[tokio::test]
async fn cache_with_ttl_moka() {
    use anycache::moka::MokaStorage;
    use anycache::Cache;
    use anycache::CacheOptions;

    let storage = MokaStorage::new(10);
    let cache = Cache::new::<MokaStorage>(storage);

    let data = cache
        .cache(
            "cache_with_ttl_moka".to_string(),
            || async { Ok("test".to_string()) },
            &[CacheOptions::TTL(std::time::Duration::from_millis(1))],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    let data = cache
        .cache(
            "cache_with_ttl_moka".to_string(),
            || async { Ok("test2".to_string()) },
            &[CacheOptions::TTL(std::time::Duration::from_millis(1))],
        )
        .await
        .unwrap();
    assert_eq!(data, "test".to_string());

    tokio::time::sleep(std::time::Duration::from_millis(2)).await;

    let data = cache
        .cache(
            "cache_with_ttl_moka".to_string(),
            || async { Ok("test3".to_string()) },
            &[CacheOptions::TTL(std::time::Duration::from_millis(1))],
        )
        .await
        .unwrap();
    assert_eq!(data, "test3".to_string());
}

#[tokio::test]
async fn cache_json_moka() {
    use anycache::moka::MokaStorage;
    use anycache::Cache;
    use serde::{Deserialize, Serialize};

    let storage = MokaStorage::new(10);
    let cache = Cache::new::<MokaStorage>(storage);

    cache
        .invalidate("cache_json_moka".to_string())
        .await
        .unwrap();

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Test {
        test: String,
    }

    let data: Test = cache
        .cache_json(
            "cache_json_moka".to_string(),
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
            "cache_json_moka".to_string(),
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
        .invalidate("cache_json_moka".to_string())
        .await
        .unwrap();
}
