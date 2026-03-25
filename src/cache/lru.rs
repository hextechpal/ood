use std::{
    collections::{HashMap, btree_map::Keys},
    hash::Hash,
    io::Seek,
};

use crate::Cache;

#[derive(Debug, PartialEq, Eq)]
struct Node<K, V> {
    key: K,
    value: V,
    next: Option<usize>,
    prev: Option<usize>,
}

pub struct LRUCache<K, V> {
    size: usize,
    capacity: usize,
    head: Option<usize>,
    tail: Option<usize>,
    nodes: Vec<Option<Node<K, V>>>,
    cache: HashMap<K, usize>,
}

impl<K, V> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            size: 0,
            head: None,
            tail: None,
            nodes: Vec::with_capacity(capacity),
            cache: HashMap::with_capacity(capacity),
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
            None => self.tail = prev,
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
}

impl<K, V> Cache<K, V> for LRUCache<K, V>
where
    K: Hash + Eq + Clone,
{
    // Logic
    //  - if key does not exist in the cache
    //      - if capacity is full evict the tail and insert at head
    //      - if capacity is not full insert the node at the head
    //
    // - if key exists in the cache
    //      - Update the value of the node and move to the head
    fn put(&mut self, key: K, value: V) {
        match self.cache.get(&key) {
            Some(&idx) => {
                self.node_mut(idx).value = value;
                self.move_to_head(idx);
            }
            None => {
                if self.is_full() {
                    // Delete the tail
                    let prev_tail_idx = self.tail.unwrap();
                    let prev_tail_key = self.node(prev_tail_idx).key.clone();
                    self.tail = self.node(prev_tail_idx).next;

                    self.nodes[prev_tail_idx] = None;
                    self.cache.remove(&prev_tail_key);
                    if let Some(new_tail_idx) = self.tail {
                        self.node_mut(new_tail_idx).prev = None;
                    }

                    if self.tail.is_none() {
                        self.head = None;
                    }
                    self.size -= 1;
                }
                // Insert at head
                let node = Node {
                    key: key.clone(),
                    value,
                    prev: self.head,
                    next: None,
                };
                self.nodes.push(Some(node));
                let push_idx = self.nodes.len() - 1;
                self.cache.insert(key, push_idx);

                match self.head {
                    Some(head) => {
                        self.node_mut(head).next = Some(push_idx);
                        self.head = Some(push_idx);
                    }

                    None => {
                        self.head = Some(push_idx);
                        self.tail = Some(push_idx);
                    }
                }
                self.size += 1;
            }
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        match self.cache.get(&key) {
            Some(&nidx) => {
                self.move_to_head(nidx);
                Some(&self.nodes[nidx].as_ref().unwrap().value)
            }
            None => None,
        }
    }
}
