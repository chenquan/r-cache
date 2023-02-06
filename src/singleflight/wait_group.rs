use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Condvar, Mutex};

pub struct WaitGroup {
    inner: Arc<Inner>,
}

struct Inner {
    cvar: Condvar,
    count: Mutex<usize>,
}

impl Default for WaitGroup {
    fn default() -> Self {
        Self { inner: Arc::new(Inner { cvar: Default::default(), count: Mutex::new(1) }) }
    }
}

impl Drop for WaitGroup {
    fn drop(&mut self) {
        let mut c = self.inner.count.lock().unwrap();
        *c -= 1;

        if *c == 0 {
            self.inner.cvar.notify_all()
        }
    }
}

impl Clone for WaitGroup {
    fn clone(&self) -> Self {
        let mut count = self.inner.count.lock().unwrap();
        *count += 1;

        Self { inner: self.inner.clone() }
    }
}

impl Debug for WaitGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let count: &usize = &self.inner.count.lock().unwrap();
        f.debug_struct("WaitGroup").field("count", count).finish()
    }
}

impl WaitGroup {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn wait(self) {
        if *self.inner.count.lock().unwrap() == 1 {
            return;
        }

        let inner = self.inner.clone();
        drop(self);

        let mut count = inner.count.lock().unwrap();
        while *count > 0 {
            count = inner.cvar.wait(count).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use crate::singleflight::wait_group::WaitGroup;

    const THREADS: usize = 10;

    #[test]
    fn test1() {
        let wg = WaitGroup::new();
        let (tx, rx) = mpsc::channel();

        for _ in 0..THREADS {
            let wg = wg.clone();
            let tx = tx.clone();

            thread::spawn(move || {
                wg.wait();
                tx.send(()).unwrap();
            });
        }

        thread::sleep(Duration::from_millis(100));

        // At this point, all spawned threads should be blocked, so we shouldn't get anything from the
        // channel.
        assert!(rx.try_recv().is_err());

        wg.wait();

        // Now, the wait group is cleared and we should receive messages.
        for _ in 0..THREADS {
            rx.recv().unwrap();
        }
    }
}