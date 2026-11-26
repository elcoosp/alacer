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
            // `None` (unbounded) is bucket 0 and every bounded width lives
            // strictly above it — the old `unwrap_or(0.0)` fold keyed a
            // bounded width below the 8 px grid (`Some(4.0)`, or a negative
            // one, which also saturated through the floor) identically to
            // the unbounded sentinel, and the unwrapped one-line shaping
            // then served a wrapped query (the layout measure cache's
            // same lesson, one file over).
            width_bucket: match max_width {
                None => 0,
                Some(w) => {
                    let clamped = w.max(0.0);
                    alacer_core::num::f32_to_u32((clamped / 8.0).floor() * 8.0).saturating_add(1)
                }
            },
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

/// The glyph-atlas LRU book (§9.4's pre-registered re-open, shipped): the
/// tick-ordered eviction policy the GPU glyph atlas drives. Every hit
/// refreshes the key's stamp, every miss admits one, and eviction always
/// takes the oldest stamp — so a hot working set survives a cold stream
/// that would thrash a full-clear, the egui-shaped failure §9.4 names.
///
/// The book is pure policy: no storage, no geometry. The atlas owns the
/// shelf and the bytes; the book owns the order and the count. The
/// eviction counter rides the observability idiom the caches already
/// speak (`hits`, `misses`) — the ledger read a consumer tests against.
pub struct Lru<K> {
    tick: u64,
    stamps: FxHashMap<K, u64>,
    evictions: u64,
}

impl<K: Eq + std::hash::Hash + Copy> Lru<K> {
    /// An empty book.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tick: 0,
            stamps: FxHashMap::default(),
            evictions: 0,
        }
    }

    /// Records a hit: refreshes `key`'s stamp. Answers whether the key was
    /// resident — the answer the atlas's hit path rides (a miss re-places).
    pub fn touch(&mut self, key: &K) -> bool {
        if let Some(stamp) = self.stamps.get_mut(key) {
            self.tick += 1;
            *stamp = self.tick;
            true
        } else {
            false
        }
    }

    /// Records a miss: admits `key` with a fresh stamp (the newest tick).
    pub fn admit(&mut self, key: K) {
        self.tick += 1;
        self.stamps.insert(key, self.tick);
    }

    /// Withdraws `key` explicitly — the caller dropped the entry without
    /// the policy evicting it (a stale-identity replacement, the atlas's
    /// pointer-miss rule). Answers whether it was resident. Never counted:
    /// the counter names policy evictions, not housekeeping.
    pub fn withdraw(&mut self, key: &K) -> bool {
        self.stamps.remove(key).is_some()
    }

    /// Takes the least-recently-used key — the eviction's victim — and
    /// counts it. `None` when the book is empty (the atlas's stop signal:
    /// nothing left to evict, the glyph oversized).
    pub fn evict_lru(&mut self) -> Option<K> {
        // Stamps are unique: every admit and touch takes a fresh tick, so
        // the minimum is one key's, and the order is total.
        let victim = self
            .stamps
            .iter()
            .min_by_key(|(_, stamp)| *stamp)
            .map(|(key, _)| *key)?;
        self.stamps.remove(&victim);
        self.evictions += 1;
        Some(victim)
    }

    /// Policy evictions the book took (the ledger read).
    #[must_use]
    pub fn evictions(&self) -> u64 {
        self.evictions
    }

    /// Resident keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stamps.len()
    }

    /// `true` when nothing is resident.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stamps.is_empty()
    }
}

impl<K: Eq + std::hash::Hash + Copy> Default for Lru<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{CacheKey, LayoutCache, Lru};
    use crate::TextLayout;

    fn key(text: &str) -> CacheKey {
        CacheKey::new(text, 16.0, None)
    }

    #[test]
    fn keys_bucket_widths_to_8px() {
        assert_eq!(CacheKey::new("x", 16.0, Some(0.0)).width_bucket(), 1);
        assert_eq!(
            CacheKey::new("x", 16.0, Some(7.9)).width_bucket(),
            CacheKey::new("x", 16.0, Some(0.0)).width_bucket(),
            "both floor into the same sub-grid bucket"
        );
        assert_eq!(CacheKey::new("x", 16.0, Some(8.0)).width_bucket(), 9);
        assert_ne!(CacheKey::new("x", 16.0, Some(7.9)).width_bucket(), 9);
    }

    #[test]
    fn a_bounded_width_never_hashes_as_the_unbounded_sentinel() {
        // The old `unwrap_or(0.0)` fold keyed a bounded sub-grid width
        // (and a negative one, saturating through the floor) identically
        // to `None` — the unwrapped one-line shaping served a wrapped
        // query. Bucket 0 is the unbounded sentinel alone.
        assert_eq!(CacheKey::new("x", 16.0, None).width_bucket(), 0);
        assert_ne!(
            CacheKey::new("x", 16.0, None).width_bucket(),
            CacheKey::new("x", 16.0, Some(4.0)).width_bucket()
        );
        assert_ne!(
            CacheKey::new("x", 16.0, None).width_bucket(),
            CacheKey::new("x", 16.0, Some(-4.0)).width_bucket(),
            "a negative width clamps to the zero-width bucket, never the sentinel"
        );
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

    /// Atlas-shaped glyph keys (the book's consumer is the GPU atlas).
    fn glyph_key(glyph_id: u32) -> alacer_core::scene::GlyphKey {
        alacer_core::scene::GlyphKey {
            font_id: 1,
            glyph_id,
            size_bits: 16.0_f32.to_bits(),
        }
    }

    #[test]
    fn eviction_takes_the_least_recently_used_key_in_order() {
        let mut book = Lru::new();
        book.admit(glyph_key(1));
        book.admit(glyph_key(2));
        book.admit(glyph_key(3));
        // The refresh moves a key behind the two that stayed cold.
        assert!(book.touch(&glyph_key(1)), "resident keys answer true");
        assert_eq!(book.evict_lru(), Some(glyph_key(2)));
        assert_eq!(book.evict_lru(), Some(glyph_key(3)));
        assert_eq!(book.evict_lru(), Some(glyph_key(1)));
        assert_eq!(book.evict_lru(), None, "an empty book refuses to evict");
    }

    #[test]
    fn evictions_count_only_policy_evictions() {
        let mut book = Lru::new();
        book.admit(glyph_key(1));
        book.admit(glyph_key(2));
        assert!(book.evict_lru().is_some());
        assert_eq!(book.evictions(), 1, "the policy eviction counts");
        // The explicit withdrawal is housekeeping, not policy.
        assert!(book.withdraw(&glyph_key(2)));
        assert_eq!(book.evictions(), 1, "withdrawals never count");
        assert!(!book.withdraw(&glyph_key(2)), "a dead key withdraws false");
        book.admit(glyph_key(3));
        assert!(book.evict_lru().is_some());
        assert_eq!(book.evictions(), 2);
    }

    #[test]
    fn touch_answers_residency_and_misses_stay_out() {
        let mut book = Lru::new();
        assert!(!book.touch(&glyph_key(7)), "an unknown key is no hit");
        book.admit(glyph_key(7));
        assert!(book.touch(&glyph_key(7)));
        assert_eq!(book.len(), 1);
        assert!(!book.is_empty());
    }

    #[test]
    fn a_hot_working_set_survives_a_cold_stream() {
        // The atlas's drive: admit a hot set, then stream cold keys past
        // it, touching the hot set every cycle. A full-clear would wipe
        // the hot keys at every capacity bound; the LRU's evictions take
        // the cold tail instead.
        let hot: Vec<_> = (0..8).map(glyph_key).collect();
        let mut book = Lru::new();
        for glyph in &hot {
            book.admit(*glyph);
        }
        for cold in 100..140_u32 {
            for glyph in &hot {
                assert!(book.touch(glyph), "the hot set stays resident");
            }
            book.admit(glyph_key(cold));
            // The capacity discipline the atlas drives: hold the hot set
            // by evicting the LRU tail.
            while book.len() > hot.len() + 1 {
                assert!(book.evict_lru().is_some());
            }
        }
        for glyph in &hot {
            assert!(book.touch(glyph), "every hot key survived the stream");
        }
        // The first cold admission fills the book exactly (8 hot + 1 cold
        // = the held bound); every later cycle evicts exactly one — 39 of
        // the 40 cold keys paid an eviction, and no hot key did.
        assert_eq!(book.evictions(), 39, "the cold stream paid the evictions");
    }
}
