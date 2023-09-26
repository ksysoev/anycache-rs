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
}
