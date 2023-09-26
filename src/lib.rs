mod memory;

pub trait Storable {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn del(&self, key: &str);
}

pub struct Cache {
    storage: Box<dyn Storable>, // Can we do this without dynamic dispatch?
}

impl Cache {
    pub fn new(storage: Box<dyn Storable>) -> Self {
        Self { storage }
    }

    pub fn cache(&self, key: String, get_data: Box<dyn FnOnce() -> String>) -> String {
        let data = self.storage.get(&key);
        match data {
            Some(data) => data,
            None => {
                let data = get_data();
                self.storage.set(&key, &data);
                data
            }
        }
    }

    pub fn invalidate(&self, key: String) {
        self.storage.del(&key);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn cache() {
        use super::memory::InMemoryStorage;
        use super::Cache;

        let storage = InMemoryStorage::new(10);
        let cache = Cache::new(Box::new(storage));

        let data = cache.cache("foo".to_string(), Box::new(|| "test".to_string()));
        assert_eq!(data, "test".to_string());

        let data = cache.cache("foo".to_string(), Box::new(|| "test2".to_string()));
        assert_eq!(data, "test".to_string());
    }

    #[test]
    fn invalidate() {
        use super::memory::InMemoryStorage;
        use super::Cache;

        let storage = InMemoryStorage::new(10);
        let cache = Cache::new(Box::new(storage));

        let data = cache.cache("foo".to_string(), Box::new(|| "test".to_string()));
        assert_eq!(data, "test".to_string());

        cache.invalidate("foo".to_string());

        let data = cache.cache("foo".to_string(), Box::new(|| "test2".to_string()));
        assert_eq!(data, "test2".to_string());
    }
}
