//! Identity: one hash, four consumers (architecture §6.3).
//!
//! Every element gets an **id path** from the view structure; each path prefix
//! carries a pre-folded hash. That single hash serves as element-state slot,
//! hit-testing identity, AccessKit `NodeId`, and hot-reload template key — one
//! identity system kills the whole class of drift bugs where the a11y tree
//! disagrees with the hit-test tree about what a node *is*.

use rustc_hash::FxHasher;
use std::hash::Hasher;

/// An element identity path: a sequence of segment hashes with pre-folded
/// prefix hashes (GPUI's `GlobalElementId` design, §6.3).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IdPath {
    segments: Vec<u64>,
    prefix_hashes: Vec<u64>,
}

/// Folds a new segment into a prefix hash. Deterministic, allocation-free.
#[must_use]
pub fn fold_hash(prefix: u64, segment: u64) -> u64 {
    let mut hasher = FxHasher::default();
    hasher.write_u64(prefix ^ segment);
    hasher.finish()
}

impl IdPath {
    /// Creates a root path from the first segment.
    #[must_use]
    pub fn root(segment: u64) -> Self {
        let mut path = Self::default();
        path.push(segment);
        path
    }

    /// Extends the path with one segment, pre-folding its prefix hash.
    pub fn push(&mut self, segment: u64) {
        let prefix = self.prefix_hashes.last().copied().unwrap_or(0);
        self.segments.push(segment);
        self.prefix_hashes.push(fold_hash(prefix, segment));
    }

    /// Returns a child path with one extra segment, leaving `self` untouched.
    #[must_use]
    pub fn child(&self, segment: u64) -> Self {
        let mut path = self.clone();
        path.push(segment);
        path
    }

    /// The full-path hash — the element's identity everywhere.
    #[must_use]
    pub fn hash(&self) -> u64 {
        self.prefix_hashes.last().copied().unwrap_or(0)
    }

    /// The pre-folded hash of the first `depth` segments.
    ///
    /// # Panics
    ///
    /// Panics if `depth` exceeds the path length.
    #[must_use]
    pub fn prefix_hash(&self, depth: usize) -> u64 {
        self.prefix_hashes[..depth].last().copied().unwrap_or(0)
    }

    /// The raw segments.
    #[must_use]
    pub fn segments(&self) -> &[u64] {
        &self.segments
    }

    /// Path length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// `true` when the path is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{IdPath, fold_hash};

    #[test]
    fn prefix_hashes_are_prefolded_consistently() {
        let mut path = IdPath::root(7);
        path.push(42);
        path.push(99);
        assert_eq!(path.len(), 3);
        assert_eq!(path.hash(), path.prefix_hash(3));
        assert_eq!(path.prefix_hash(2), fold_hash(fold_hash(0, 7), 42));
        assert_eq!(path.prefix_hash(1), fold_hash(0, 7));
    }

    #[test]
    fn distinct_paths_hash_distinctly() {
        let mut a = IdPath::root(1);
        a.push(2);
        let mut b = IdPath::root(2);
        b.push(1);
        assert_ne!(a.hash(), b.hash());

        let mut deep = a.child(3);
        deep.push(4);
        let mut sibling = a.child(3);
        sibling.push(5);
        assert_ne!(deep.hash(), sibling.hash());
        assert_ne!(deep.hash(), a.hash());
    }

    #[test]
    fn child_path_extends_without_mutating_parent() {
        let parent = IdPath::root(10);
        let child = parent.child(20);
        assert_eq!(parent.segments(), &[10]);
        assert_eq!(child.segments(), &[10, 20]);
    }

    #[test]
    fn empty_path_hashes_to_zero() {
        assert_eq!(IdPath::default().hash(), 0);
        assert!(IdPath::default().is_empty());
    }
}
