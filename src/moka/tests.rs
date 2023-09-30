use crate::{Storable, StorableTTL};

#[tokio::test]
async fn get_set_values() {
    use super::MokaStorage;
    let storage = MokaStorage::new(10);
    assert_eq!(storage.get("foo").await.unwrap(), None);

    storage.set("foo", "test", None).await.unwrap();

    assert_eq!(storage.get("foo").await.unwrap(), Some("test".to_string()));
}

#[tokio::test]
async fn del_values() {
    use super::MokaStorage;
    let storage = MokaStorage::new(10);

    storage.set("foo", "test", None).await.unwrap();

    assert_eq!(storage.get("foo").await.unwrap(), Some("test".to_string()));

    storage.del("foo").await.unwrap();

    assert_eq!(storage.get("foo").await.unwrap(), None);
}

#[tokio::test]
async fn set_with_ttl() {
    use super::MokaStorage;
    let storage = MokaStorage::new(10);

    storage.del("set_with_ttl").await.unwrap();

    storage
        .set(
            "set_with_ttl",
            "test",
            Some(std::time::Duration::from_millis(1)),
        )
        .await
        .unwrap();

    assert_eq!(
        storage.get("set_with_ttl").await.unwrap(),
        Some("test".to_string())
    );

    tokio::time::sleep(std::time::Duration::from_millis(2)).await;

    assert_eq!(storage.get("set_with_ttl").await.unwrap(), None);
}

#[tokio::test]
async fn get_with_ttl() {
    use super::MokaStorage;
    let storage = MokaStorage::new(10);

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

    match storage.get_with_ttl("get_with_ttl").await.unwrap() {
        Some((value, StorableTTL::TTL(ttl))) => {
            assert_eq!(value, "test".to_string());
            assert!(ttl.as_millis() <= 10);
            assert!(ttl.as_millis() > 0);
        }
        _ => panic!("Expected TTL"),
    }
}
