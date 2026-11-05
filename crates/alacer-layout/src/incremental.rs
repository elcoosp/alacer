//! Torin-style incremental semantics, ported onto a persistent Taffy tree
//! (§8.2, ADR-3).

use alacer_core::arena::{Arena, NodeId};
use alacer_core::mask::Change;

/// Why a node is dirty for layout — torin's `DirtyReason` extended per §8.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirtyReason {
    /// Style-affecting change: re-measure the node, then reflow.
    Style,
    /// Text content changed: re-measure this text node only — never restyle
    /// the tree (§8.2 item 5, Floem's `taffy.mark_dirty` pattern).
    Text,
    /// Children added/removed/reordered.
    Reorder,
    /// Geometry churn below: ancestors reposition, sizes stable.
    InnerLayout,
}

impl DirtyReason {
    /// Maps drained masks onto layout reasons.
    #[must_use]
    pub fn from_change(change: Change) -> Option<Self> {
        if change.contains(Change::STRUCTURE) {
            Some(Self::Reorder)
        } else if change.contains(Change::LAYOUT) {
            Some(Self::Style)
        } else {
            None
        }
    }
}

/// The closest common ancestor of `nodes` (inclusive) — the root candidate that
/// bounds the re-measure region (§8.2 item 3).
///
/// Returns `None` when `nodes` is empty or contains non-live ids.
#[must_use]
pub fn closest_common_ancestor(arena: &Arena, nodes: &[NodeId]) -> Option<NodeId> {
    if nodes.is_empty() {
        return None;
    }
    let first = nodes[0];
    let mut candidate = Some(first);
    while let Some(c) = candidate {
        if nodes[1..].iter().all(|n| is_ancestor_or_self(arena, c, *n)) {
            return Some(c);
        }
        candidate = arena.parent(c);
    }
    None
}

/// `true` when `ancestor` is `node` or an ancestor of it.
#[must_use]
fn is_ancestor_or_self(arena: &Arena, ancestor: NodeId, node: NodeId) -> bool {
    let mut cursor = Some(node);
    while let Some(c) = cursor {
        if c == ancestor {
            return true;
        }
        cursor = arena.parent(c);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{DirtyReason, closest_common_ancestor};
    use alacer_core::arena::Arena;
    use alacer_core::mask::Change;
    use alacer_core::style::MeasureSpec;
    use alacer_core::widget::{LabelSource, Role, Widget};

    struct Probe;

    impl Widget for Probe {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn role(&self) -> Role {
            Role::Container
        }

        fn label(&self) -> LabelSource {
            LabelSource::text("probe")
        }

        fn measure_spec(&self) -> Option<MeasureSpec> {
            None
        }
    }

    fn tree() -> (Arena, [alacer_core::arena::NodeId; 5]) {
        let mut arena = Arena::new();
        let root = arena.set_root(Box::new(Probe));
        let a = arena.add_child(root, Box::new(Probe));
        let b = arena.add_child(root, Box::new(Probe));
        let a1 = arena.add_child(a, Box::new(Probe));
        let a2 = arena.add_child(a, Box::new(Probe));
        (arena, [root, a, b, a1, a2])
    }

    #[test]
    fn reasons_map_from_changes() {
        assert_eq!(
            DirtyReason::from_change(Change::STRUCTURE),
            Some(DirtyReason::Reorder)
        );
        assert_eq!(
            DirtyReason::from_change(Change::LAYOUT),
            Some(DirtyReason::Style)
        );
        assert_eq!(DirtyReason::from_change(Change::PAINT), None);
    }

    #[test]
    fn siblings_share_their_parent_as_root_candidate() {
        let (arena, [root, a, b, a1, a2]) = tree();
        assert_eq!(closest_common_ancestor(&arena, &[a1, a2]), Some(a));
        assert_eq!(closest_common_ancestor(&arena, &[a, b]), Some(root));
        let _ = root;
    }

    #[test]
    fn a_single_node_is_its_own_candidate() {
        let (arena, [_root, _a, _b, a1, _a2]) = tree();
        assert_eq!(closest_common_ancestor(&arena, &[a1]), Some(a1));
    }

    #[test]
    fn ancestor_inclusive_of_self() {
        let (arena, [_root, a, _b, a1, _a2]) = tree();
        assert_eq!(closest_common_ancestor(&arena, &[a, a1]), Some(a));
    }

    #[test]
    fn empty_input_is_none() {
        let (arena, [_root, _a, _b, _a1, _a2]) = tree();
        assert_eq!(closest_common_ancestor(&arena, &[]), None);
    }

    #[test]
    fn deep_descendants_converge_on_the_common_root() {
        let (mut arena, [root, a, b, a1, a2]) = tree();
        let leaf = arena.add_child(a1, Box::new(Probe));
        assert_eq!(closest_common_ancestor(&arena, &[leaf, a2]), Some(a));
        assert_eq!(closest_common_ancestor(&arena, &[leaf, b]), Some(root));
    }
}
