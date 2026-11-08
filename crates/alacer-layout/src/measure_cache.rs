//! The LRU measure cache (§8.1: Masonry's 10-slot discipline, generalized).
//!
//! §14.3: *every cache has an eviction policy and a test that proves it.* The
//! policy here: least-recently-used, fixed capacity, keyed by a content hash
//! plus the width bucket that shaped the result.

use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};

/// Cache key: content identity + shaping width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeasureKey {
    content: u64,
    width_bucket: u32,
}

impl MeasureKey {
    /// Builds a key from a text spec identity and the wrap width.
    ///
    /// Widths bucket to 8 px so near-identical widths share entries without
    /// changing results measurably (egui's integer wrap-width lesson, §8.3).
    #[must_use]
    pub fn from_spec(spec: &alacer_core::style::TextSpec, max_width: Option<f32>) -> Self {
        let mut hasher = rustc_hash::FxHasher::default();
        spec.text.hash(&mut hasher);
        spec.font_id.hash(&mut hasher);
        spec.font_size.to_bits().hash(&mut hasher);
        Self {
            content: hasher.finish(),
            width_bucket: bucket(max_width),
        }
    }
}

/// Buckets a wrap width to 8 px steps; `None` (unbounded) is bucket 0.
#[must_use]
pub fn bucket(max_width: Option<f32>) -> u32 {
    match max_width {
        None => 0,
        Some(w) => {
            let clamped = w.max(0.0);
            alacer_core::num::f32_to_u32((clamped / 8.0).floor() * 8.0)
        }
    }
}

/// A fixed-capacity LRU cache for measured sizes.
#[derive(Debug, Default)]
pub struct LruMeasureCache {
    capacity: usize,
    entries: FxHashMap<MeasureKey, (alacer_core::geom::Size, u64)>,
    tick: u64,
    hits: u64,
    misses: u64,
}

impl LruMeasureCache {
    /// A cache holding at most `capacity` entries.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is zero.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "measure cache capacity must be positive");
        Self {
            capacity,
            ..Self::default()
        }
    }

    /// Looks `key` up, refreshing its recency.
    #[must_use]
    pub fn get(&mut self, key: &MeasureKey) -> Option<alacer_core::geom::Size> {
        if let Some((size, stamp)) = self.entries.get_mut(key) {
            self.tick += 1;
            *stamp = self.tick;
            self.hits += 1;
            Some(*size)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Inserts (or refreshes) `key`, evicting the least-recently-used entry
    /// when at capacity.
    ///
    /// # Panics
    ///
    /// Never for well-formed input: the `expect` guards the just-checked
    /// non-emptiness of the map at capacity, not caller state.
    pub fn put(&mut self, key: MeasureKey, size: alacer_core::geom::Size) {
        self.tick += 1;
        if self.entries.len() >= self.capacity && !self.entries.contains_key(&key) {
            let victim = self
                .entries
                .iter()
                .min_by_key(|(_, (_, stamp))| *stamp)
                .map(|(k, _)| *k)
                .expect("non-empty above capacity");
            self.entries.remove(&victim);
        }
        self.entries.insert(key, (size, self.tick));
    }

    /// Entry count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Cache hits since construction.
    #[must_use]
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Cache misses since construction.
    #[must_use]
    pub fn misses(&self) -> u64 {
        self.misses
    }
}

#[cfg(test)]
mod tests {
    use super::{LruMeasureCache, MeasureKey, bucket};
    use alacer_core::geom::Size;
    use alacer_core::style::TextSpec;

    fn spec(text: &str) -> TextSpec {
        TextSpec {
            text: text.to_string(),
            font_id: 1,
            font_size: 14.0,
        }
    }

    #[test]
    fn width_buckets_are_stable_and_ordered() {
        assert_eq!(bucket(None), 0);
        assert_eq!(bucket(Some(7.9)), bucket(Some(0.1)), "both floor to 0");
        assert_eq!(bucket(Some(8.0)), bucket(Some(15.9)));
        assert_ne!(bucket(Some(7.9)), bucket(Some(8.0)));
    }

    #[test]
    fn get_refreshes_recency_and_evicts_lru() {
        let mut cache = LruMeasureCache::new(3);
        let size = Size::new(10.0, 14.0);

        let k1 = MeasureKey::from_spec(&spec("one"), None);
        let k2 = MeasureKey::from_spec(&spec("two"), None);
        let k3 = MeasureKey::from_spec(&spec("three"), None);
        let k4 = MeasureKey::from_spec(&spec("four"), None);

        cache.put(k1, size);
        cache.put(k2, size);
        cache.put(k3, size);
        assert_eq!(cache.len(), 3);

        // Touch k1: it becomes most recent; k2 is now the LRU victim.
        assert!(cache.get(&k1).is_some());
        cache.put(k4, size);

        assert_eq!(cache.len(), 3, "capacity enforced");
        assert!(cache.get(&k1).is_some(), "recently touched survivor");
        assert!(cache.get(&k3).is_some(), "never-touched survivor");
        assert!(cache.get(&k2).is_none(), "LRU entry evicted");
    }

    #[test]
    fn reinserting_a_live_key_does_not_evict() {
        let mut cache = LruMeasureCache::new(2);
        let k1 = MeasureKey::from_spec(&spec("a"), None);
        let k2 = MeasureKey::from_spec(&spec("b"), None);
        cache.put(k1, alacer_core::geom::Size::zero());
        cache.put(k2, alacer_core::geom::Size::zero());
        cache.put(k1, alacer_core::geom::Size::zero()); // refresh, not evict
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn hit_and_miss_counters_track_reality() {
        let mut cache = LruMeasureCache::new(1);
        let k = MeasureKey::from_spec(&spec("x"), None);
        assert!(cache.get(&k).is_none());
        cache.put(k, Size::new(1.0, 1.0));
        assert!(cache.get(&k).is_some());
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);
    }
}
