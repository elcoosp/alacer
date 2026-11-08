//! The frame-windowed layout cache (§9.4: caches evict — the ecosystem's
//! unbounded-cache club stays at zero members).
//!
//! Policy: entries carry the frame they were last touched; `advance_frame`
//! drops everything older than the window. A hard capacity bounds memory even
//! within a single frame. Both policies are test-proven.

use crate::TextLayout;
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};

/// Cache key: text identity + shaping width bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey {
    content: u64,
    size_bits: u32,
    width_bucket: u32,
}

impl CacheKey {
    /// Builds a key; widths bucket to 8 px (§8.3 integer wrap discipline).
    #[must_use]
    pub fn new(text: &str, font_size: f32, max_width: Option<f32>) -> Self {
        let mut hasher = rustc_hash::FxHasher::default();
        text.hash(&mut hasher);
        Self {
            content: hasher.finish(),
            size_bits: font_size.to_bits(),
            width_bucket: alacer_core::num::f32_to_u32((max_width.unwrap_or(0.0) / 8.0).floor()),
        }
    }

    /// Width bucket used (exposed for tests).
    #[must_use]
    pub const fn width_bucket(&self) -> u32 {
        self.width_bucket
    }
}

/// An LRU + frame-windowed cache of shaped text layouts.
#[derive(Debug)]
pub struct LayoutCache {
    capacity: usize,
    frame_window: u64,
    frame: u64,
    tick: u64,
    entries: FxHashMap<CacheKey, Entry>,
    hits: u64,
    misses: u64,
}

#[derive(Debug)]
struct Entry {
    layout: TextLayout,
    frame_stamp: u64,
    tick: u64,
}

impl LayoutCache {
    /// A cache bounded by `capacity` entries and a `frame_window` frame recency.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is zero.
    #[must_use]
    pub fn new(capacity: usize, frame_window: u64) -> Self {
        assert!(capacity > 0, "layout cache capacity must be positive");
        Self {
            capacity,
            frame_window,
            frame: 0,
            tick: 0,
            entries: FxHashMap::default(),
            hits: 0,
            misses: 0,
        }
    }

    /// Looks `key` up, refreshing recency.
    #[must_use]
    pub fn get(&mut self, key: &CacheKey) -> Option<TextLayout> {
        if let Some(entry) = self.entries.get_mut(key) {
            self.tick += 1;
            entry.tick = self.tick;
            entry.frame_stamp = self.frame;
            self.hits += 1;
            Some(entry.layout.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// Inserts `key`, evicting the LRU entry when at capacity.
    ///
    /// # Panics
    ///
    /// Never for well-formed input: the `expect` guards the just-established
    /// non-emptiness of the map at capacity, not caller state.
    pub fn put(&mut self, key: CacheKey, layout: TextLayout) {
        self.tick += 1;
        self.entries.insert(
            key,
            Entry {
                layout,
                frame_stamp: self.frame,
                tick: self.tick,
            },
        );
        while self.entries.len() > self.capacity {
            let victim = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.tick)
                .map(|(k, _)| *k)
                .expect("over capacity");
            self.entries.remove(&victim);
        }
    }

    /// Advances the frame clock, dropping entries outside the window.
    pub fn advance_frame(&mut self) {
        self.frame += 1;
        let cutoff = self.frame.saturating_sub(self.frame_window);
        self.entries.retain(|_, entry| entry.frame_stamp >= cutoff);
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

    /// Cache hits.
    #[must_use]
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Cache misses.
    #[must_use]
    pub fn misses(&self) -> u64 {
        self.misses
    }
}

#[cfg(test)]
mod tests {
    use super::{CacheKey, LayoutCache};
    use crate::TextLayout;

    fn key(text: &str) -> CacheKey {
        CacheKey::new(text, 16.0, None)
    }

    #[test]
    fn keys_bucket_widths_to_8px() {
        assert_eq!(CacheKey::new("x", 16.0, Some(0.0)).width_bucket(), 0);
        assert_eq!(CacheKey::new("x", 16.0, Some(7.9)).width_bucket(), 0);
        assert_eq!(CacheKey::new("x", 16.0, Some(8.0)).width_bucket(), 1);
    }

    #[test]
    fn entries_expire_after_the_frame_window() {
        let mut cache = LayoutCache::new(16, 2);
        cache.put(key("a"), TextLayout::default());
        assert!(cache.get(&key("a")).is_some());

        cache.advance_frame(); // frame 1
        cache.advance_frame(); // frame 2
        assert!(cache.get(&key("a")).is_some(), "touched entries refresh");

        cache.advance_frame(); // frame 3: last touch was frame 2 — kept
        assert!(cache.get(&key("a")).is_some(), "inside the window");

        cache.advance_frame(); // frame 4: last touch was frame 3 — still inside (window 2 covers frames 2..4)
        assert!(cache.get(&key("a")).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn untouched_entries_expire_once_out_of_the_window() {
        let mut cache = LayoutCache::new(16, 2);
        cache.put(key("a"), TextLayout::default());
        for _ in 0..5 {
            cache.advance_frame();
        }
        assert_eq!(cache.len(), 0, "no touch in 5 frames: evicted");
    }

    #[test]
    fn capacity_bounds_memory_within_a_frame() {
        let mut cache = LayoutCache::new(2, 100);
        cache.put(key("a"), TextLayout::default());
        cache.put(key("b"), TextLayout::default());
        cache.put(key("c"), TextLayout::default());
        assert_eq!(cache.len(), 2, "hard capacity enforced");
        assert!(cache.get(&key("c")).is_some(), "most recent survivor");
        assert!(cache.get(&key("a")).is_none(), "oldest evicted");
    }

    #[test]
    fn hit_and_miss_counters_track_reality() {
        let mut cache = LayoutCache::new(4, 4);
        let k = key("hit");
        let _ = cache.get(&k);
        cache.put(k, TextLayout::default());
        let _ = cache.get(&k);
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);
    }
}
