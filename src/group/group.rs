use std::sync::Mutex;
use crate::cache::cache;
use crate::cache::cache::Cache;
use crate::single_flight;

struct Group {
    name: String,
    cache: Mutex<cache::LRUCache<Vec<u8>>>,
    single_flight: single_flight::single_flight::Group<Vec<u8>>,
}

impl Group {
    pub fn new(name: String) -> Self {
        Self { name, cache: Mutex::new(cache::LRUCache::new(10)), single_flight: Default::default() }
    }
}

impl Group {
    fn put(&mut self, key: String, value: Vec<u8>) {
        self.cache.lock().unwrap().set(key, value);
    }

    fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        self.single_flight.do_fn(key.to_owned().as_str(),
                                 || {
                                     let mut guard = self.cache.lock().unwrap();
                                     match guard.get(key) {
                                         None => {
                                             self.get_local(key)
                                         }
                                         Some(v) => {
                                             Some(v.clone())
                                         }
                                     }
                                 },
        )
    }
    fn delete(&mut self) {}

    fn name(self) -> String {
        self.name.clone()
    }


    fn get_local(&self, _: &str) -> Option<Vec<u8>> {
        Some("不存在".as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::group::group::Group;

    #[test]
    fn group() {
        let mut group = Group::new("1".to_string());
        group.put("1".to_string(), "1".as_bytes().to_vec());

        println!("{}", String::from_utf8(group.get("1").unwrap()).unwrap());
        println!("{}", String::from_utf8(group.get("2").unwrap()).unwrap());
    }
}
