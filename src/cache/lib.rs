mod lru;

pub trait Cache<K, V>
where
    V: Clone,
{
    fn put(&self, key: K, value: V);

    fn get(&self, key: &K) -> Option<V>;
}
