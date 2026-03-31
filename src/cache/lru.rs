use std::{collections::HashMap, hash::Hash, sync::Mutex};

use crate::Cache;

#[derive(Debug, PartialEq, Eq)]
struct Node<K, V> {
    key: K,
    value: V,
    next: Option<usize>,
    prev: Option<usize>,
}

struct Inner<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone
{
    size: usize,
    capacity: usize,
    head: Option<usize>,
    tail: Option<usize>,
    nodes: Vec<Option<Node<K, V>>>,
    cache: HashMap<K, usize>,
    free: Vec<usize>,
}

impl<K, V> Inner<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone
{
    fn new(capacity: usize) -> Self {
        Inner {
            capacity,
            size: 0,
            head: None,
            tail: None,
            nodes: Vec::with_capacity(capacity),
            cache: HashMap::with_capacity(capacity),
            free: Vec::new(),
        }
    }

    fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    fn node(&self, idx: usize) -> &Node<K, V> {
        self.nodes[idx].as_ref().unwrap()
    }

    fn node_mut(&mut self, idx: usize) -> &mut Node<K, V> {
        self.nodes[idx].as_mut().unwrap()
    }

    fn detach(&mut self, idx: usize) {
        let (prev, next) = {
            let node = self.node(idx);
            (node.prev, node.next)
        };

        self.node_mut(idx).prev = None;
        self.node_mut(idx).next = None;

        match prev {
            Some(prev_idx) => self.node_mut(prev_idx).next = next,
            None => self.tail = next,
        };

        match next {
            Some(next_idx) => self.node_mut(next_idx).prev = prev,
            None => self.head = prev,
        };
    }

    fn attach_at_head(&mut self, idx: usize) {
        match self.head {
            Some(head_idx) => {
                self.node_mut(head_idx).next = Some(idx);
                self.node_mut(idx).prev = Some(head_idx);
                self.node_mut(idx).next = None;
                self.head = Some(idx);
            }
            None => {
                self.node_mut(idx).prev = None;
                self.node_mut(idx).next = None;
                self.head = Some(idx);
                self.tail = Some(idx);
            }
        }
    }

    fn move_to_head(&mut self, idx: usize) {
        // This idx is the head
        if self.head == Some(idx) {
            return;
        }
        self.detach(idx);
        self.attach_at_head(idx);
    }

    fn evict_tail(&mut self) {
        let tail_idx = self.tail.unwrap();
        let tail_key = self.node(tail_idx).key.clone();
        self.detach(tail_idx);
        self.nodes[tail_idx] = None;
        self.cache.remove(&tail_key);
        self.size -= 1;
        self.free.push(tail_idx);
    }

    fn insert_at_head(&mut self, key: K, value: V) {
        let node = Node {
            key: key.clone(),
            value,
            prev: self.head,
            next: None,
        };

        let push_idx = match self.free.pop() {
            Some(free_idx) => {
                self.nodes[free_idx] = Some(node);
                free_idx
            }
            None => {
                self.nodes.push(Some(node));
                self.nodes.len() - 1
            }
        };

        self.attach_at_head(push_idx);
        self.cache.insert(key, push_idx);
        self.size += 1;
    }
}

pub struct LRUCache<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    inner: Mutex<Inner<K, V>>,
}

impl<K, V> LRUCache<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    fn new(capacity: usize) -> Self {
        LRUCache {
            inner: Mutex::new(Inner::new(capacity)),
        }
    }
}

impl<K, V> Cache<K, V> for LRUCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    // Logic
    //  - if key does not exist in the cache
    //      - if capacity is full evict the tail and insert at head
    //      - if capacity is not full insert the node at the head
    //
    // - if key exists in the cache
    //      - Update the value of the node and move to the head
    fn put(&self, key: K, value: V) {
        let mut inner = self.inner.lock().expect("poisoned lock");
        if let Some(&idx) = inner.cache.get(&key) {
            inner.node_mut(idx).value = value;
            inner.move_to_head(idx);
            return;
        }

        if inner.is_full() {
            inner.evict_tail();
        }
        // Insert at head
        inner.insert_at_head(key, value);
    }

    fn get(&self, key: &K) -> Option<V> {
        let mut inner = self.inner.lock().expect("poisoned lock");
        match inner.cache.get(&key) {
            Some(&idx) => {
                inner.move_to_head(idx);
                let value = inner.node(idx).value.clone();
                Some(value)
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn head_key<K, V>(cache: &LRUCache<K, V>) -> Option<K>
    where
        K: Hash + Eq + Clone,
        V: Clone,
    {
        let inner = cache.inner.lock().expect("inner should be");
        inner.head.map(|idx| inner.node(idx).key.clone())
    }

    fn tail_key<K, V>(cache: &LRUCache<K, V>) -> Option<K>
    where
        K: Hash + Eq + Clone,
        V: Clone,
    {
        let inner = cache.inner.lock().expect("inner should be");
        inner.tail.map(|idx| inner.node(idx).key.clone())
    }

    #[test]
    fn test_capacity_one() {
        let cache = LRUCache::new(1);
        let key = String::from("name");
        let value = String::from("prashant");
        let update = String::from("Prashant");
        cache.put(key.clone(), value.clone());
        assert_eq!(cache.get(&key), Some(value));

        cache.put(key.clone(), update.clone());
        assert_eq!(cache.get(&key), Some(update.clone()));

        let new_key = String::from("Name");
        cache.put(new_key.clone(), update.clone());
        assert_eq!(cache.get(&key), None);
        assert_eq!(cache.get(&new_key), Some(update));
    }

    #[test]
    fn test_evicts_least_recently_used_when_full() {
        let cache = LRUCache::new(2);

        cache.put(1, "one");
        assert_eq!(head_key(&cache), Some(1));
        assert_eq!(tail_key(&cache), Some(1));

        cache.put(2, "two");
        assert_eq!(head_key(&cache), Some(2));
        assert_eq!(tail_key(&cache), Some(1));

        cache.put(3, "three");

        assert_eq!(head_key(&cache), Some(3));
        assert_eq!(tail_key(&cache), Some(2));
        // We reuse the slot of the nodes rather than keeping it empty
        let inner = cache.inner.lock().expect("cache should be accessible");
        assert_eq!(inner.nodes.len(), 2);
        assert!(!inner.cache.contains_key(&1));
        assert_eq!(inner.node(*inner.cache.get(&2).unwrap()).value, "two");
        assert_eq!(inner.node(*inner.cache.get(&3).unwrap()).value, "three");
    }

    #[test]
    fn test_get_moves_item_to_head() {
        let cache = LRUCache::new(2);

        cache.put(1, "one");
        cache.put(2, "two");
        assert_eq!(head_key(&cache), Some(2));
        assert_eq!(tail_key(&cache), Some(1));

        assert_eq!(cache.get(&1), Some("one"));
        assert_eq!(head_key(&cache), Some(1));
        assert_eq!(tail_key(&cache), Some(2));

        cache.put(3, "three");

        assert_eq!(head_key(&cache), Some(3));
        assert_eq!(tail_key(&cache), Some(1));
        // we reuse the same slot of the vector
        let inner = cache.inner.lock().expect("cache should be accessible");

        assert_eq!(inner.nodes.len(), 2);
        assert!(inner.cache.contains_key(&1));
        assert!(!inner.cache.contains_key(&2));
        assert_eq!(inner.node(*inner.cache.get(&3).unwrap()).value, "three");
    }

    #[test]
    fn test_updating_existing_key_refreshes_recency() {
        let cache = LRUCache::new(2);

        cache.put(1, "one");
        cache.put(2, "two");
        assert_eq!(head_key(&cache), Some(2));
        assert_eq!(tail_key(&cache), Some(1));

        cache.put(1, "updated");
        assert_eq!(head_key(&cache), Some(1));
        assert_eq!(tail_key(&cache), Some(2));

        cache.put(3, "three");

        assert_eq!(head_key(&cache), Some(3));
        assert_eq!(tail_key(&cache), Some(1));
        let inner = cache.inner.lock().expect("cache should be accesible");
        assert_eq!(inner.node(*inner.cache.get(&1).unwrap()).value, "updated");
        assert!(!inner.cache.contains_key(&2));
        assert_eq!(inner.node(*inner.cache.get(&3).unwrap()).value, "three");
    }

    #[test]
    fn test_missing_get_does_not_change_recency_order() {
        let cache = LRUCache::new(2);

        cache.put(1, "one");
        cache.put(2, "two");
        assert_eq!(head_key(&cache), Some(2));
        assert_eq!(tail_key(&cache), Some(1));

        assert_eq!(cache.get(&99), None);
        assert_eq!(head_key(&cache), Some(2));
        assert_eq!(tail_key(&cache), Some(1));

        cache.put(3, "three");

        assert_eq!(head_key(&cache), Some(3));
        assert_eq!(tail_key(&cache), Some(2));

        let inner = cache.inner.lock().expect("cache should be accesible");
        assert!(!inner.cache.contains_key(&1));
        assert_eq!(inner.node(*inner.cache.get(&2).unwrap()).value, "two");
        assert_eq!(inner.node(*inner.cache.get(&3).unwrap()).value, "three");
    }
}
