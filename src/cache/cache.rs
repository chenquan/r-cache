use std::collections::HashMap;

pub trait Cache<T> {
    fn get(&mut self, key: &str) -> Option<T>;
    fn set(&mut self, key: String, value: T) -> bool;
    fn delete(&mut self, key: &str) -> bool;
}

pub struct LRUCache<T>
    where T: Clone, {
    cap: i64,
    data_map: HashMap<String, T>,
    list: Vec<String>,
}

impl<T> LRUCache<T> where T: Clone, {
    pub fn new(cap: i64) -> LRUCache<T> {
        Self { cap, data_map: HashMap::new(), list: Vec::new() }
    }
}

impl<T> Cache<T> for LRUCache<T> where T: Clone, {
    fn get(&mut self, key: &str) -> Option<T> {
        for i in 0..self.list.len() {
            let v = self.list.get(i).unwrap();
            if v.eq(key) {
                let s = self.list.remove(i);
                self.list.push(s);
                break;
            }
        }

        self.data_map.get(key).cloned()
    }

    fn set(&mut self, key: String, value: T) -> bool {
        if self.list.len() >= self.cap as usize {
            let s = self.list.remove(0);
            self.data_map.remove(&s);
        }

        if !self.list.contains(&key) {
            self.list.push(key.clone());
        }


        self.data_map.insert(key, value).map_or(true, |_| false)
    }

    fn delete(&mut self, key: &str) -> bool {
        for i in 0..self.list.len() {
            if self.list[i].eq(key) {
                let s = self.list.remove(0);
                self.data_map.remove(&s);
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use crate::cache::cache::{Cache, LRUCache};

    #[test]
    fn cache() {
        let mut c: LRUCache<String> = LRUCache::new(3);
        assert_eq!(c.set("1".to_string(), "1".to_string()), true);
        assert_eq!(c.get("1").unwrap(), "1");

        assert_eq!(c.set("2".to_string(), "2".to_string()), true);
        assert_eq!(c.set("3".to_string(), "3".to_string()), true);
        assert_eq!(c.set("4".to_string(), "4".to_string()), true);
        assert_eq!(c.delete("4"), true);
        assert_eq!(c.delete("1"), false);
    }

    #[test]
    fn safe_cache() {
        let c = Arc::new(Mutex::new(LRUCache::new(3)));
        let mut handles = vec![];

        for _ in 0..10 {
            let c = Arc::clone(&c);
            // 创建子线程，并将`Mutex`的所有权拷贝传入到子线程中
            let handle = thread::spawn(move || {
                let mut guard = c.lock().unwrap();
                match guard.get("a") {
                    None => { guard.set("a".to_string(), "1".to_string()) }
                    Some(it) => {
                        let i = it.parse::<i32>().unwrap() + 1;
                        guard.set("a".to_string(), i.to_string())
                    }
                };
            });
            handles.push(handle);
        }

        // 等待所有子线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        println!("Result: {}", c.lock().unwrap().get("a").unwrap());
    }
}

