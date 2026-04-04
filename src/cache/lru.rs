use std::{
    collections::{BinaryHeap, HashMap},
    hash::Hash,
    sync::{Arc, Mutex, atomic::AtomicU64},
    thread::{self, sleep},
    time::Duration,
};

use tracing::debug;

use crate::Cache;

const EVICTION_INTERVAL_MS: u64 = 10;

#[derive(Debug, PartialEq, Eq)]
struct Entry<V> {
    value: V,
    last_acess: u64,
}

struct Inner<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    capacity: usize,
    nodes: Vec<Option<Entry<V>>>,
    cache: HashMap<K, usize>,
    clock: AtomicU64,
    free: Vec<usize>,
}

impl<K, V> Inner<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    fn new(capacity: usize) -> Self {
        Inner {
            capacity,
            nodes: Vec::with_capacity(capacity),
            cache: HashMap::with_capacity(capacity),
            clock: AtomicU64::new(0),
            free: Vec::new(),
        }
    }

    fn node(&self, idx: usize) -> &Entry<V> {
        self.nodes[idx].as_ref().unwrap()
    }

    fn node_mut(&mut self, idx: usize) -> &mut Entry<V> {
        self.nodes[idx].as_mut().unwrap()
    }

    fn evict_entries(&mut self) {
        if self.cache.len() <= self.capacity {
            debug!(
                "[no eviction] size = {} capacity={}",
                self.cache.len(),
                self.capacity
            );
            return;
        }
        let count = self.cache.len() - self.capacity;
        let mut heap = BinaryHeap::with_capacity(count + 1);

        for (_, idx) in self.cache.iter() {
            heap.push((self.node(*idx).last_acess, *idx));
            if heap.len() > count {
                heap.pop();
            }
        }

        heap.into_iter().for_each(|(_, idx)| {
            self.nodes[idx] = None;
            self.free.push(idx);
        });

        self.cache.retain(|_, v| self.nodes[*v].is_some());
    }
}

pub struct LRUCache<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    inner: Arc<Mutex<Inner<K, V>>>,
}

impl<K, V> LRUCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn new(capacity: usize) -> Self {
        let inner = Arc::new(Mutex::new(Inner::new(capacity)));
        let cache = LRUCache { inner: inner };

        cache.start_eviction_loop();
        cache
    }

    fn start_eviction_loop(&self) {
        let inner = self.inner.clone();
        thread::spawn(move || {
            debug!("starting eviction loop");
            loop {
                {
                    let mut inner = inner.lock().expect("poisoned lock");
                    inner.evict_entries();
                }
                sleep(Duration::from_millis(EVICTION_INTERVAL_MS));
            }
        });
    }
}

impl<K, V> Cache<K, V> for LRUCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    fn put(&self, key: K, value: V) {
        let mut inner = self.inner.lock().expect("poisoned lock");
        let ts = inner
            .clock
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        debug!("updating or inserting entry with ts: {}", ts);
        match inner.cache.get(&key) {
            Some(&idx) => {
                inner.node_mut(idx).last_acess = ts;
                inner.node_mut(idx).value = value
            }

            None => {
                let node = Entry {
                    value,
                    last_acess: ts,
                };

                let push_idx = match inner.free.pop() {
                    None => {
                        inner.nodes.push(Some(node));
                        inner.nodes.len() - 1
                    }
                    Some(free_idx) => {
                        inner.nodes[free_idx] = Some(node);
                        free_idx
                    }
                };
                inner.cache.insert(key, push_idx);
            }
        }
    }

    fn get(&self, key: &K) -> Option<V> {
        let mut inner = self.inner.lock().expect("poisoned lock");
        match inner.cache.get(&key) {
            Some(&idx) => {
                let ts = inner
                    .clock
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let node = inner.node_mut(idx);
                node.last_acess = ts;
                Some(node.value.clone())
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_vanilla_insert() {
        let cache = LRUCache::new(5);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(4, 40);
        cache.put(5, 50);
        cache.put(6, 60);

        assert_eq!(cache.get(&2), Some(20));

        sleep(Duration::from_millis(2 * EVICTION_INTERVAL_MS));

        assert_eq!(cache.get(&1), None);
    }
}
