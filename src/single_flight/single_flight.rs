use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;
use crate::single_flight::wait_group::WaitGroup;


#[derive(Clone, Debug)]
struct Call<T> where T: Clone + Debug {
    wg: WaitGroup,
    result: Option<T>,
}


impl<T> Call<T> where T: Clone + Debug {
    fn new() -> Call<T> {
        Call {
            wg: Default::default(),
            result: None,
        }
    }
}

pub struct Group<T> where T: Clone + Debug {
    m: Mutex<HashMap<String, Box<Call<T>>>>,
}

impl<T> Default for Group<T> where T: Clone + Debug {
    fn default() -> Self {
        Self { m: Mutex::new(Default::default()) }
    }
}


impl<T> Group<T> where T: Clone + Debug {
    pub fn new() -> Group<T> {
        Group::default()
    }

    pub fn do_fn<F>(&self, key: &str, func: F) -> Option<T>
        where F: Fn() -> Option<T>, {
        let mut m = self.m.lock().unwrap();

        if let Some(c) = m.get(key) {
            let c = c.clone();
            drop(m);
            c.wg.wait();
            return c.result;
        }

        let call = Call::new();
        let wg = call.wg.clone();

        let job = m.entry(key.to_owned()).or_insert(Box::new(call));
        job.result = func();
        drop(m);
        drop(wg);

        let mut m = self.m.lock().unwrap();
        let call = m.remove(key).unwrap();
        drop(m);

        call.result.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::Group;

    const RESULT: usize = 0;

    #[test]
    fn test_simple() {
        let g = Group::<usize>::new();
        let res = g.do_fn("key", || Option::from(RESULT));
        assert_eq!(res.unwrap(), RESULT);
    }

    #[test]
    fn test_multiple_threads() {
        use std::time::Duration;

        fn expensive_fn() -> Option<usize> {
            thread::sleep(Duration::new(0, 500));
            Some(RESULT)
        }

        let g = Group::new();
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let res = g.do_fn("key", expensive_fn);
                    assert_eq!(res.unwrap(), RESULT);
                });
            }
        })
    }
}