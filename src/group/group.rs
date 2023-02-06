use std::sync::Mutex;
use crate::cache::cache;
use crate::cache::cache::Cache;

struct Group {
    name: String,
    cache: Mutex<cache::LRUCache<Vec<u8>>>,

}

impl Group {
    pub fn new(s: String) -> Self {
        Self { name: s, cache: Mutex::new(cache::LRUCache::new(10)) }
    }
}

impl Group {
    fn put(&mut self, key: String, value: Vec<u8>) {
        self.cache.lock().unwrap().set(key, value);
    }
}