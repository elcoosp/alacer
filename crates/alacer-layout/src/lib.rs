//! Alacer's layout: a persistent Taffy tree with torin-style incremental
//! relayout (architecture §8).
//!
//! Three commitments, all test-enforced:
//!
//! 1. **The tree persists** — never cleared between frames (the anti-pattern is
//!    GPUI's per-frame `TaffyTree` rebuild, `window.rs:3743`).
//! 2. **The closest common ancestor of the dirty set bounds the re-measure
//!    region** (torin's root-candidate algorithm, §8.2 item 3).
//! 3. **Every cache has an eviction policy and a test that proves it** (§14.3).

pub mod host;
pub mod incremental;
pub mod measure_cache;
pub mod style_map;

pub use host::{LayoutHost, SpecCell};
pub use incremental::{DirtyReason, closest_common_ancestor};
pub use measure_cache::LruMeasureCache;
