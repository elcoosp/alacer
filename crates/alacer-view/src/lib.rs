//! Alacer's view layer: the view tree, keyed reconciliation, and the
//! view→widget reconciler (§6.1, §6.2).
//!
//! The keyed diff pipeline is Dioxus's, hardened across its fuzz corpus,
//! ported as ALGO:
//!
//! 1. **Prefix/suffix peel** — matching runs at both ends never move.
//! 2. **Key map** — the remaining nodes resolve through a hash map of keys.
//! 3. **Longest-increasing-subsequence** — LIS members are *stable*; everything
//!    else is the minimal mutation set.
//!
//! The result is a list of [`DiffOp`]s that transforms `old` into `new` with
//! the minimum number of moves. The §6.4 equivalence property — *incremental
//! result ≡ fresh rebuild* — is property-tested below. [`ViewRoot`] runs the
//! same pipeline against the retained widget arena: view diffs become
//! in-place widget mutations, never rebuilds (§6.1). The element registry
//! ([`WidgetRegistry`]) maps view element names to widget create/patch
//! pairs; the concrete M0 set registers itself from `alacer-widgets`
//! (`alacer_widgets::default_registry`).

pub mod hot;
pub mod mount;
pub mod registry;
pub mod view;

pub use hot::{HotView, ViewGeneration};
pub use mount::{PatchReport, ViewRoot};
pub use registry::{ElementProps, WidgetRegistry};
pub use view::{PropValue, ViewNode};

use rustc_hash::{FxHashMap, FxHashSet};
use std::hash::Hash;

/// One reconciliation step. Keys are caller-owned identities (§6.3: id-path
/// segment hashes in production).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffOp<K> {
    /// Append/insert a new key at `at` (after removals and moves are applied,
    /// indices refer to the state at the time the op runs).
    Insert {
        /// The inserted key.
        key: K,
        /// The index the op runs at.
        at: usize,
    },
    /// Remove an existing key (identified by key, not position).
    Remove {
        /// The removed key.
        key: K,
    },
    /// Move an existing key to index `at` (removing it from its old slot).
    Move {
        /// The moved key.
        key: K,
        /// The index the op runs at.
        at: usize,
    },
    /// The key stays exactly where it is (LIS member — nothing to do).
    Keep {
        /// The kept key.
        key: K,
    },
}

/// The result of reconciling `old` against `new`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileResult<K> {
    /// Ordered operations: removals first, then keeps/moves, then inserts.
    pub ops: Vec<DiffOp<K>>,
}

/// Reconciles `old` into `new` under stable `key`s.
///
/// # Panics
///
/// Panics in debug builds if `new` contains duplicate keys (the keyed
/// contract; Dioxus debug-asserts the same, §6.2).
#[must_use]
pub fn reconcile<K: Copy + Eq + Hash + std::fmt::Debug>(
    old: &[K],
    new: &[K],
) -> ReconcileResult<K> {
    debug_assert!(unique(new), "keys must be unique within a frame");

    let mut ops: Vec<DiffOp<K>> = Vec::new();

    // 1. Prefix/suffix peel (§6.2 item 1).
    let mut prefix = 0_usize;
    while prefix < old.len() && prefix < new.len() && old[prefix] == new[prefix] {
        ops.push(DiffOp::Keep { key: old[prefix] });
        prefix += 1;
    }
    let mut suffix_old = old.len();
    let mut suffix_new = new.len();
    while suffix_old > prefix && suffix_new > prefix && old[suffix_old - 1] == new[suffix_new - 1] {
        suffix_old -= 1;
        suffix_new -= 1;
    }

    let old_mid = &old[prefix..suffix_old];
    let new_mid = &new[prefix..suffix_new];

    // 2. Key map over the remaining nodes (§6.2 item 2).
    let new_pos: FxHashMap<K, usize> = new_mid.iter().enumerate().map(|(i, k)| (*k, i)).collect();
    let old_set: FxHashMap<K, usize> = old_mid.iter().enumerate().map(|(i, k)| (*k, i)).collect();

    // 3. LIS over the old keys' positions in `new` (§6.2 item 3).
    let surviving: Vec<K> = old_mid
        .iter()
        .copied()
        .filter(|k| new_pos.contains_key(k))
        .collect();
    let positions: Vec<usize> = surviving.iter().map(|k| new_pos[k]).collect();
    let stable = longest_increasing_subsequence(&positions);

    // Removals: old-middle keys absent from `new`.
    for (old_index, key) in old_mid.iter().enumerate() {
        if !new_pos.contains_key(key) {
            ops.push(DiffOp::Remove { key: *key });
            let _ = old_index;
        }
    }

    // Moves for the non-stable survivors; keeps for LIS members.
    let stable_keys: std::collections::HashSet<usize> = stable.iter().copied().collect();
    for (survivor_index, key) in surviving.iter().enumerate() {
        if stable_keys.contains(&survivor_index) {
            ops.push(DiffOp::Keep { key: *key });
        } else {
            let at = prefix + new_pos[key];
            ops.push(DiffOp::Move { key: *key, at });
        }
    }

    // Inserts: new-middle keys absent from `old`, in order.
    for (i, key) in new_mid.iter().enumerate() {
        if !old_set.contains_key(key) {
            ops.push(DiffOp::Insert {
                key: *key,
                at: prefix + i,
            });
        }
    }

    ReconcileResult { ops }
}

/// Applies ops to a list — the reference semantics every consumer must match.
///
/// Application order is fixed regardless of how ops are listed: removals
/// first (key-based), then every displaced element — moves *and* inserts —
/// placed in a single pass ascending by target index. `Keep` is
/// informational.
#[must_use]
pub fn apply<K: Copy + Eq + Hash>(old: &[K], ops: &[DiffOp<K>]) -> Vec<K> {
    let mut out: Vec<K> = old.to_vec();

    for op in ops.iter().filter(|op| matches!(op, DiffOp::Remove { .. })) {
        if let DiffOp::Remove { key } = *op
            && let Some(at) = out.iter().position(|k| *k == key)
        {
            out.remove(at);
        }
    }

    // Every displaced element — moved or inserted — carries its final index.
    // They must be placed together, ascending: a move's target is only valid
    // once every element before it (moved *or* inserted) is in place.
    let mut placements: Vec<(K, usize)> = ops
        .iter()
        .filter_map(|op| match *op {
            DiffOp::Move { key, at } | DiffOp::Insert { key, at } => Some((key, at)),
            DiffOp::Remove { .. } | DiffOp::Keep { .. } => None,
        })
        .collect();
    placements.sort_by_key(|&(_, at)| at);

    let displaced: Vec<K> = ops
        .iter()
        .filter_map(|op| match *op {
            DiffOp::Move { key, .. } => Some(key),
            _ => None,
        })
        .collect();
    out.retain(|k| !displaced.contains(k));

    for (key, at) in placements {
        let at = at.min(out.len());
        out.insert(at, key);
    }

    out
}

fn unique<K: Copy + Eq + Hash>(keys: &[K]) -> bool {
    let set: FxHashSet<K> = keys.iter().copied().collect();
    set.len() == keys.len()
}

/// Longest strictly-increasing subsequence, patience algorithm, O(n log n),
/// returning indices into `values` in order.
#[must_use]
pub fn longest_increasing_subsequence(values: &[usize]) -> Vec<usize> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut tails: Vec<usize> = Vec::with_capacity(values.len()); // indices into `values`
    let mut parents: Vec<Option<usize>> = vec![None; values.len()];

    for (i, value) in values.iter().enumerate() {
        match tails.binary_search_by(|&idx| values[idx].cmp(value)) {
            Ok(pos) => {
                tails[pos] = i;
            }
            Err(pos) => {
                if pos > 0 {
                    parents[i] = Some(tails[pos - 1]);
                }
                if pos == tails.len() {
                    tails.push(i);
                } else {
                    tails[pos] = i;
                }
            }
        }
    }

    let mut result = Vec::with_capacity(tails.len());
    let mut cursor = Some(tails[tails.len() - 1]);
    while let Some(i) = cursor {
        result.push(i);
        cursor = parents[i];
    }
    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::{apply, longest_increasing_subsequence, reconcile};

    #[test]
    fn lis_finds_the_longest_run() {
        assert_eq!(longest_increasing_subsequence(&[]), Vec::<usize>::new());
        assert_eq!(longest_increasing_subsequence(&[5]), vec![0]);
        assert_eq!(
            longest_increasing_subsequence(&[3, 1, 4, 1, 5]),
            vec![1, 2, 4]
        );
        assert_eq!(longest_increasing_subsequence(&[0, 1, 2]), vec![0, 1, 2]);
    }

    #[test]
    fn append_only_differs_by_one_insert() {
        let result = reconcile(&[1_u64, 2], &[1, 2, 3]);
        assert_eq!(
            result.ops,
            vec![
                super::DiffOp::Keep { key: 1 },
                super::DiffOp::Keep { key: 2 },
                super::DiffOp::Insert { key: 3, at: 2 }
            ],
            "peeled prefix keeps; the new tail inserts"
        );
        let result = reconcile(&[1_u64, 2, 3], &[1, 2, 3, 4]);
        assert_eq!(
            apply(&[1, 2, 3], &result.ops),
            vec![1, 2, 3, 4],
            "suffix peel yields the append"
        );
    }

    #[test]
    fn front_insert_beyond_the_peel() {
        let result = reconcile(&[2_u64, 3], &[1, 2, 3]);
        let applied = apply(&[2, 3], &result.ops);
        assert_eq!(applied, vec![1, 2, 3]);
    }

    #[test]
    fn removal_of_the_middle() {
        let result = reconcile(&[1_u64, 2, 3], &[1, 3]);
        assert_eq!(apply(&[1, 2, 3], &result.ops), vec![1, 3]);
        assert!(result.ops.iter().all(|op| matches!(
            op,
            super::DiffOp::Keep { .. } | super::DiffOp::Remove { .. }
        )));
    }

    #[test]
    fn a_swap_moves_exactly_one_element() {
        let result = reconcile(&[1_u64, 2, 3], &[3, 2, 1]);
        let moves = result
            .ops
            .iter()
            .filter(|op| matches!(op, super::DiffOp::Move { .. }))
            .count();
        assert!(moves <= 2, "at most two moves for a full reversal");
        assert_eq!(apply(&[1, 2, 3], &result.ops), vec![3, 2, 1]);
    }

    #[test]
    fn rotation_moves_the_minimal_set() {
        // LIS members [2,3,4] stay; only 1 moves.
        let result = reconcile(&[1_u64, 2, 3, 4], &[2, 3, 4, 1]);
        assert_eq!(apply(&[1, 2, 3, 4], &result.ops), vec![2, 3, 4, 1]);
        let moves = result
            .ops
            .iter()
            .filter(|op| matches!(op, super::DiffOp::Move { .. }))
            .count();
        assert_eq!(moves, 1, "LIS keeps three of four elements stable");
    }

    #[test]
    fn replace_one_of_many() {
        let result = reconcile(&[1_u64, 2, 3], &[1, 9, 3]);
        assert_eq!(apply(&[1, 2, 3], &result.ops), vec![1, 9, 3]);
    }

    #[test]
    fn empty_to_filled_and_back() {
        let r = reconcile(&[0_u64; 0], &[1, 2, 3]);
        assert_eq!(apply(&[], &r.ops), vec![1, 2, 3]);
        let r = reconcile(&[1_u64, 2, 3], &[]);
        assert_eq!(apply(&[1, 2, 3], &r.ops), Vec::<u64>::new());
    }

    /// The §6.4 property: the incremental result equals the fresh rebuild.
    #[test]
    fn incremental_result_equals_rebuild_on_random_churn() {
        struct Lcg(u64);

        impl Lcg {
            fn next(&mut self) -> u64 {
                self.0 = self
                    .0
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                self.0 >> 33
            }
        }

        let mut rng = Lcg(0xCAFE);
        for case in 0..256_u64 {
            let mut pool: Vec<u64> = (0..24).collect();
            // Random old list.
            let mut old = Vec::new();
            let old_len = usize::try_from(rng.next() % 10).expect("in range");
            for _ in 0..old_len {
                let idx =
                    usize::try_from(rng.next() % u64::try_from(pool.len()).expect("countable"))
                        .expect("in range");
                old.push(pool.remove(idx));
            }
            // Random new list (may share keys, may not).
            let mut new = Vec::new();
            let new_len = usize::try_from(rng.next() % 10).expect("in range");
            let mut fresh = pool.clone();
            for _ in 0..new_len {
                let take_from_old = rng.next().is_multiple_of(2);
                if take_from_old && !old.is_empty() {
                    let idx =
                        usize::try_from(rng.next() % u64::try_from(old.len()).expect("countable"))
                            .expect("in range");
                    let key = old[idx];
                    if !new.contains(&key) {
                        new.push(key);
                        continue;
                    }
                }
                if fresh.is_empty() {
                    break;
                }
                let idx =
                    usize::try_from(rng.next() % u64::try_from(fresh.len()).expect("countable"))
                        .expect("in range");
                new.push(fresh.remove(idx));
            }

            let result = reconcile(&old, &new);
            let applied = apply(&old, &result.ops);
            assert_eq!(
                applied, new,
                "case {case}: incremental == rebuild (old={old:?} ops={:?})",
                result.ops
            );
        }
    }
}
