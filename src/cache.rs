use cached::Cached;
use std::hash::Hash;

use hashbrown::HashMap;

/// The copy of 'cached' crate's structure
/// <https://github.com/jaemk/cached/blob/master/src/stores.rs#L20>
/// but using <https://github.com/Amanieu/hashbrown>
/// instead of default `HashMap` for speeding up
#[derive(Default)]
pub struct GrowableCache<K, V>
where
    K: Eq + Hash,
{
    store: HashMap<K, V>,
    capacity: usize,
    increase_in: u8,
    max_size: usize,
    hits: u32,
    misses: u32,
}

impl<K: Hash + Eq, V> GrowableCache<K, V> {
    #[allow(dead_code)]
    pub fn with_capacity(size: usize) -> Self {
        Self::with_capacity_and_increase(size, 1)
    }

    pub fn with_capacity_and_increase(size: usize, increase_in: u8) -> Self {
        Self::with_capacity_increase_and_max_size(size, increase_in, size * 10)
    }

    pub fn with_capacity_increase_and_max_size(
        size: usize,
        increase_in: u8,
        max_size: usize,
    ) -> Self {
        Self {
            store: HashMap::with_capacity(size),
            capacity: size,
            increase_in,
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    fn increase_size(&mut self) {
        if self.capacity >= self.max_size {
            return;
        }

        if self.increase_in > 1 {
            let new_capacity = self.capacity * (self.increase_in as usize);
            self.capacity = new_capacity.min(self.max_size);
        }
    }
}

impl<K: Hash + Eq, V> Cached<K, V> for GrowableCache<K, V> {
    fn cache_get(&mut self, key: &K) -> Option<&V> {
        if let Some(v) = self.store.get(key) {
            self.hits += 1;
            Some(v)
        } else {
            self.misses += 1;
            None
        }
    }
    fn cache_set(&mut self, key: K, val: V) {
        if self.store.len() >= self.capacity {
            warn!("Maximum size for cache reached ({}).", self.capacity);
            self.store.clear();
            self.increase_size();
        }
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

pub fn cache_info<K, V>(cache: &Cached<K, V>) -> (usize, u32, f32)
where
    K: Hash + Eq,
{
    if cache.cache_size() > 0 {
        let hits = cache.cache_hits().unwrap_or(0);
        let misses = cache.cache_misses().unwrap_or(0);
        let hit_rate = if hits == 0 {
            0.0
        } else {
            hits as f32 / (hits + misses) as f32
        };

        (cache.cache_size(), hits, hit_rate)
    } else {
        (0, 0, 0.0)
    }
}
