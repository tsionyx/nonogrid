use cached::Cached;
use std::collections::HashMap;
use std::hash::Hash;

/// The copy of 'cached' crate's structure
/// https://github.com/jaemk/cached/blob/master/src/stores.rs#L20
/// but using https://github.com/Amanieu/hashbrown
/// instead of default HashMap for speeding up
#[derive(Default)]
pub struct UnboundCache<K, V>
where
    K: Eq + Hash,
{
    store: HashMap<K, V>,
    hits: u32,
    misses: u32,
}

impl<K: Hash + Eq, V> UnboundCache<K, V> {
    /// Creates an empty `UnboundCache` with a given pre-allocated capacity
    pub fn with_capacity(size: usize) -> UnboundCache<K, V> {
        UnboundCache {
            store: HashMap::with_capacity(size),
            hits: 0,
            misses: 0,
        }
    }
}

impl<K: Hash + Eq, V> Cached<K, V> for UnboundCache<K, V> {
    fn cache_get(&mut self, key: &K) -> Option<&V> {
        match self.store.get(key) {
            Some(v) => {
                self.hits += 1;
                Some(v)
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }
    fn cache_set(&mut self, key: K, val: V) {
        self.store.insert(key, val);
    }
    fn cache_remove(&mut self, k: &K) -> Option<V> {
        self.store.remove(k)
    }
    fn cache_clear(&mut self) {
        self.store.clear();
    }
    fn cache_size(&self) -> usize {
        self.store.len()
    }
    fn cache_hits(&self) -> Option<u32> {
        Some(self.hits)
    }
    fn cache_misses(&self) -> Option<u32> {
        Some(self.misses)
    }
}
