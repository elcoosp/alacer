//! The retained widget tree: a safe slot arena with guard-based invalidation
//! (architecture §7.1).
//!
//! All tree mutation flows through guard types that **merge invalidation flags
//! upward on drop** (Masonry's `WidgetMut` discipline), combined with GPUI's
//! lease-and-panic re-entrancy protection: mutating the arena while a mutation
//! is already in flight panics with the two contending locations named.

use crate::mask::Change;
use crate::widget::Widget;
use rustc_hash::FxHashMap;
use std::fmt;
use std::num::NonZeroU64;
use std::panic::Location;
use std::sync::atomic::{AtomicU64, Ordering};

/// Handle to one node in the [`Arena`]. A `NonZeroU64` from a global counter,
/// attributed on attach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(NonZeroU64);

impl NodeId {
    /// The synthetic id used before a root is mounted.
    ///
    /// Never held by a live node; the arena's counter starts above it.
    pub const INVALID: Self = Self(NonZeroU64::new(4_294_967_295).expect("non-zero"));

    /// Allocates a fresh id from the global counter.
    ///
    /// Framework-internal in normal use (the arena hands ids out on attach);
    /// public for tests, benches, and tooling that key maps by node id.
    #[must_use]
    pub fn allocate() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let raw = NEXT.fetch_add(1, Ordering::Relaxed);
        Self(NonZeroU64::new(raw).expect("node id counter never yields zero"))
    }

    /// The raw numeric id.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0.get())
    }
}

/// One retained node: parent link, children, widget, merged mask, geometry.
pub struct Slot {
    /// This slot's id.
    pub id: NodeId,
    /// Parent, or `None` for the root.
    pub parent: Option<NodeId>,
    /// Children in tree order — the paint order.
    pub children: Vec<NodeId>,
    /// The retained widget.
    pub widget: Box<dyn Widget>,
    /// Merged `Change` mask awaiting pass consumption.
    pub mask: Change,
    /// Absolute composed bounds, written by the layout engine and adjusted by
    /// the compose pass. Unset before the first layout.
    pub bounds: Option<crate::geom::Rect>,
}

impl Slot {
    /// Shared borrow of the widget.
    #[must_use]
    pub fn widget(&self) -> &dyn Widget {
        &*self.widget
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Slot")
            .field("id", &self.id)
            .field("parent", &self.parent)
            .field("children", &self.children.len())
            .field("mask", &self.mask)
            .field("bounds", &self.bounds)
            .finish()
    }
}

/// A deferred widget mutation closure (pass 4).
pub type MutationFn = Box<dyn FnOnce(&mut dyn Widget)>;

struct PendingMutation {
    node: NodeId,
    closure: MutationFn,
}

/// The arena: ownership of every retained widget, plus the tree's shape.
#[derive(Default)]
pub struct Arena {
    slots: Vec<Slot>,
    index: FxHashMap<NodeId, usize>,
    root: Option<NodeId>,
    mutation_lease: Option<(&'static Location<'static>, u64)>,
    pending: Vec<PendingMutation>,
}

impl Arena {
    /// An empty arena.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The root node, once mounted.
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Mounts the root widget (the arena must be empty).
    ///
    /// # Panics
    ///
    /// Panics if a root is already mounted, or while another mutation holds the
    /// arena lease (see [`Self::mutate`]).
    #[track_caller]
    pub fn set_root(&mut self, widget: Box<dyn Widget>) -> NodeId {
        self.assert_lease_free();
        assert!(self.root.is_none(), "root already mounted");
        let id = NodeId::allocate();
        self.slots.push(Slot {
            id,
            parent: None,
            children: Vec::new(),
            widget,
            mask: Change::STRUCTURE
                .union(Change::LAYOUT)
                .union(Change::PAINT)
                .union(Change::A11Y),
            bounds: None,
        });
        self.index.insert(id, 0);
        self.root = Some(id);
        id
    }

    /// Appends `widget` as the last child of `parent`.
    ///
    /// # Panics
    ///
    /// Panics if `parent` is not live, or while another mutation holds the lease.
    #[track_caller]
    pub fn add_child(&mut self, parent: NodeId, widget: Box<dyn Widget>) -> NodeId {
        self.assert_lease_free();
        let id = NodeId::allocate();
        {
            let parent_slot = self.slot_mut(parent);
            parent_slot.children.push(id);
            parent_slot.mask |= Change::STRUCTURE.propagated_to_ancestors();
        }
        let mask = Change::STRUCTURE;
        self.slots.push(Slot {
            id,
            parent: Some(parent),
            children: Vec::new(),
            widget,
            mask,
            bounds: None,
        });
        self.index.insert(id, self.slots.len() - 1);
        id
    }

    /// Removes a node and its whole subtree; the parent absorbs `STRUCTURE`.
    ///
    /// # Panics
    ///
    /// Panics if `node` is the root (detach the whole app instead) or not live.
    #[track_caller]
    pub fn remove_subtree(&mut self, node: NodeId) {
        self.assert_lease_free();
        let parent = self
            .slot(node)
            .parent
            .expect("cannot remove the root; drop the arena instead");
        let mut to_remove = Vec::new();
        self.collect_subtree(node, &mut to_remove);
        for id in &to_remove {
            let idx = self.index.remove(id).expect("indexed slot");
            self.slots[idx].mask = Change::empty();
            self.slots[idx].children.clear();
            self.slots[idx].parent = None;
        }
        let parent_slot = self.slot_mut(parent);
        parent_slot.children.retain(|c| *c != node);
        parent_slot.mask |= Change::STRUCTURE.propagated_to_ancestors();
    }

    /// Permutes `parent`'s children into exactly `order` — the move op for
    /// keyed reconciliation (§6.2: LIS members stay, the rest move).
    ///
    /// The list must be a permutation of the current children: dropping,
    /// duplicating, or importing a foreign id is a reconciler bug and panics.
    ///
    /// # Panics
    ///
    /// Panics if `order` is not a permutation of the live children, or while
    /// another mutation holds the lease.
    #[track_caller]
    pub fn reorder_children(&mut self, parent: NodeId, order: &[NodeId]) {
        self.assert_lease_free();
        let current = self.children(parent).to_vec();
        let same = current.len() == order.len()
            && order.iter().all(|id| current.contains(id))
            && current.iter().all(|id| order.contains(id));
        assert!(
            same,
            "reorder_children: the list must be a permutation of the live \
             children (got {order:?}, have {current:?})"
        );
        let parent_slot = self.slot_mut(parent);
        if parent_slot.children == order {
            return; // already in order: no structural change to report
        }
        parent_slot.children = order.to_vec();
        parent_slot.mask |= Change::STRUCTURE.propagated_to_ancestors();
    }

    /// Depth-first pre-order visit of every live node — the paint order.
    pub fn for_each_pre_order(&self, f: &mut impl FnMut(NodeId, &Slot)) {
        if let Some(root) = self.root {
            self.visit(root, f);
        }
    }

    fn visit(&self, node: NodeId, f: &mut impl FnMut(NodeId, &Slot)) {
        let slot = self.slot(node);
        f(node, slot);
        for child in slot.children.clone() {
            self.visit(child, f);
        }
    }

    /// Shared borrow of a live slot.
    ///
    /// # Panics
    ///
    /// Panics if `node` is not live.
    #[must_use]
    #[track_caller]
    pub fn slot(&self, node: NodeId) -> &Slot {
        let idx = self
            .index
            .get(&node)
            .copied()
            .unwrap_or_else(|| panic!("{node} is not live"));
        &self.slots[idx]
    }

    /// Whether `node` is a live arena slot (safe to `slot()`, `mutate()`,
    /// etc.). Dead slots linger in the backing vector after
    /// [`Self::remove_subtree`] but leave the index.
    #[must_use]
    pub fn is_live(&self, node: NodeId) -> bool {
        self.index.contains_key(&node)
    }

    fn slot_mut(&mut self, node: NodeId) -> &mut Slot {
        let idx = self
            .index
            .get(&node)
            .copied()
            .unwrap_or_else(|| panic!("{node} is not live"));
        &mut self.slots[idx]
    }

    /// Shared borrow of a live node's widget.
    ///
    /// # Panics
    ///
    /// Panics if `node` is not live.
    #[must_use]
    #[track_caller]
    pub fn widget(&self, node: NodeId) -> &dyn Widget {
        self.slot(node).widget()
    }

    /// The composed bounds of a live node.
    ///
    /// # Panics
    ///
    /// Panics if `node` is not live.
    #[must_use]
    #[track_caller]
    pub fn bounds(&self, node: NodeId) -> Option<crate::geom::Rect> {
        self.slot(node).bounds
    }

    /// The children of a live node.
    ///
    /// # Panics
    ///
    /// Panics if `node` is not live.
    #[must_use]
    #[track_caller]
    pub fn children(&self, node: NodeId) -> &[NodeId] {
        &self.slot(node).children
    }

    /// The parent of a live node.
    ///
    /// # Panics
    ///
    /// Panics if `node` is not live.
    #[must_use]
    #[track_caller]
    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.slot(node).parent
    }

    /// The merged mask of a live node.
    ///
    /// # Panics
    ///
    /// Panics if `node` is not live.
    #[must_use]
    #[track_caller]
    pub fn mask(&self, node: NodeId) -> Change {
        self.slot(node).mask
    }

    /// Merges `change` into `node`'s mask and propagates ancestor consequences
    /// up the parent chain (A1).
    ///
    /// # Panics
    ///
    /// Panics if `node` is not live.
    #[track_caller]
    pub fn merge_mask(&mut self, node: NodeId, change: Change) {
        let mut cursor = Some(node);
        let mut up = change;
        while let Some(id) = cursor {
            let slot = self.slot_mut(id);
            slot.mask |= up;
            up = up.propagated_to_ancestors();
            if up.is_empty() {
                break;
            }
            cursor = slot.parent;
        }
    }

    /// Drains every node's mask that intersects `filter`, returning
    /// `(node, mask)` pairs and clearing the drained bits.
    #[must_use]
    pub fn take_masks(&mut self, filter: Change) -> Vec<(NodeId, Change)> {
        let mut out = Vec::new();
        for slot in &mut self.slots {
            if slot.mask.intersects(filter) {
                out.push((slot.id, slot.mask & filter));
                slot.mask &= !filter;
            }
        }
        out
    }

    /// Writes composed bounds without touching any mask: the layout engine owns
    /// geometry (§7.2 pass 6).
    pub fn update_bounds(&mut self, node: NodeId, bounds: crate::geom::Rect) {
        if let Some(idx) = self.index.get(&node).copied() {
            self.slots[idx].bounds = Some(bounds);
        }
    }

    /// Queues a deferred widget mutation, drained by the mutate pass (§7.2
    /// pass 4): the closure runs with the arena lease held, and any masks it
    /// requests merge upward on drop of the enclosing `WidgetMut`.
    pub fn queue_mutation(&mut self, node: NodeId, closure: MutationFn) {
        self.pending.push(PendingMutation { node, closure });
    }

    /// The number of queued-but-not-drained mutations.
    #[must_use]
    pub fn pending_mutations(&self) -> usize {
        self.pending.len()
    }

    /// Drains all queued mutations, returning the nodes that were mutated
    /// (pass 4 feeds them to the style pass's recalc candidates, §7.2).
    ///
    /// # Panics
    ///
    /// Panics if the closure targets a node that is no longer live.
    pub fn drain_mutations(&mut self) -> Vec<NodeId> {
        let pending = std::mem::take(&mut self.pending);
        let mut mutated = Vec::with_capacity(pending.len());
        for mutation in pending {
            let node = mutation.node;
            let mut guard = self.mutate(node);
            (mutation.closure)(guard.widget_mut());
            mutated.push(node);
        }
        mutated
    }

    /// Takes the arena lease for one node mutation.
    ///
    /// The returned guard merges any masks it requested upward **on drop**
    /// (Masonry `WidgetMut` discipline). Re-entrant leasing panics with both
    /// contending locations (GPUI lease-and-panic, §7.1).
    ///
    /// # Panics
    ///
    /// Panics if the arena lease is already held elsewhere, or `node` is not live.
    #[track_caller]
    #[must_use]
    pub fn mutate(&mut self, node: NodeId) -> WidgetMut<'_> {
        self.assert_lease_free();
        let here = Location::caller();
        let serial = self.mutation_lease.map_or(1, |(_, s)| s + 1);
        self.mutation_lease = Some((here, serial));
        let _ = self.slot(node);
        WidgetMut {
            arena: self,
            node,
            requested: Change::empty(),
            serial,
        }
    }

    #[track_caller]
    fn assert_lease_free(&self) {
        if let Some((location, _)) = self.mutation_lease {
            panic!(
                "arena mutation already in progress at {location}; second mutation attempted here"
            );
        }
    }

    fn collect_subtree(&self, node: NodeId, out: &mut Vec<NodeId>) {
        out.push(node);
        for child in self.slot(node).children.clone() {
            self.collect_subtree(child, out);
        }
    }

    /// Number of live slots (including any removed-but-uncompacted).
    #[must_use]
    pub fn len(&self) -> usize {
        self.slots
            .iter()
            .filter(|s| self.index.contains_key(&s.id))
            .count()
    }

    /// `true` when no node is live.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Debug for Arena {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Arena")
            .field("root", &self.root)
            .field("live_slots", &self.len())
            .field("pending_mutations", &self.pending.len())
            .finish()
    }
}

/// Guard for one node mutation; requested masks merge upward on drop.
///
/// Borrowing this guard borrows the whole arena mutably — exactly the
/// single-writer discipline the lease check enforces at runtime for re-entrancy.
pub struct WidgetMut<'a> {
    arena: &'a mut Arena,
    node: NodeId,
    requested: Change,
    serial: u64,
}

impl WidgetMut<'_> {
    /// Requests `change` for this node; applied (with ancestor propagation)
    /// when the guard drops.
    pub fn request(&mut self, change: Change) {
        self.requested |= change;
    }

    /// Mutable borrow of the widget itself.
    ///
    /// # Panics
    ///
    /// Panics if the node was concurrently removed (impossible while the lease
    /// is held — the borrow checker makes this unreachable).
    #[must_use]
    pub fn widget_mut(&mut self) -> &mut dyn Widget {
        &mut *self.arena.slot_mut(self.node).widget
    }

    /// The node under mutation.
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    /// Serial number of this lease (monotonic per arena).
    #[must_use]
    pub const fn serial(&self) -> u64 {
        self.serial
    }
}

impl Drop for WidgetMut<'_> {
    fn drop(&mut self) {
        self.arena.mutation_lease = None;
        if !self.requested.is_empty() {
            self.arena.merge_mask(self.node, self.requested);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Arena, NodeId};
    use crate::mask::Change;
    use crate::widget::{EventResponse, LabelSource, Role, Widget};
    use crate::{PointerEvent, Rect, Size, Style};

    /// A widget that records paints, for mutation tests.
    struct Probe {
        role: Role,
        name: &'static str,
    }

    impl Probe {
        fn named(name: &'static str) -> Box<Self> {
            Box::new(Self {
                role: Role::Container,
                name,
            })
        }
    }

    impl Widget for Probe {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn role(&self) -> Role {
            self.role
        }

        fn label(&self) -> LabelSource {
            LabelSource::text(self.name)
        }

        fn style(&self) -> Style {
            Style::default()
        }

        fn on_pointer_event(
            &mut self,
            _ctx: &mut crate::events::EventCtx,
            _event: &PointerEvent,
        ) -> EventResponse {
            EventResponse::Ignored
        }
    }

    #[test]
    fn mount_root_and_children_shapes_the_tree() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let a = arena.add_child(root, Probe::named("a"));
        let b = arena.add_child(root, Probe::named("b"));
        let a1 = arena.add_child(a, Probe::named("a1"));

        assert_eq!(arena.root(), Some(root));
        assert_eq!(arena.children(root), &[a, b]);
        assert_eq!(arena.parent(a1), Some(a));
        assert_eq!(arena.len(), 4);
    }

    #[test]
    fn pre_order_walk_matches_paint_order() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let a = arena.add_child(root, Probe::named("a"));
        arena.add_child(a, Probe::named("a1"));
        let b = arena.add_child(root, Probe::named("b"));

        let mut order = Vec::new();
        arena.for_each_pre_order(&mut |id, slot| order.push((id, slot.widget().label().resolve())));
        assert_eq!(
            order.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>(),
            ["root", "a", "a1", "b"]
        );
        assert_eq!(order[3].0, b);
    }

    #[test]
    fn reorder_children_permutes_paint_order_and_flags_structure() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let a = arena.add_child(root, Probe::named("a"));
        let b = arena.add_child(root, Probe::named("b"));
        let c = arena.add_child(root, Probe::named("c"));
        let _ = arena.take_masks(Change::ALL); // isolate from mount masks

        arena.reorder_children(root, &[c, a, b]);

        let mut order = Vec::new();
        arena.for_each_pre_order(&mut |_, slot| order.push(slot.widget().label().resolve()));
        assert_eq!(
            order,
            ["root", "c", "a", "b"],
            "children follow the new order"
        );

        let masks = arena.take_masks(Change::ALL);
        assert!(
            masks
                .iter()
                .any(|(id, m)| *id == root && m.contains(Change::STRUCTURE)),
            "reorder marks the parent structure-dirty"
        );
    }

    #[test]
    fn reorder_children_rejects_foreign_and_missing_ids() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let a = arena.add_child(root, Probe::named("a"));
        let b = arena.add_child(root, Probe::named("b"));
        let a1 = arena.add_child(a, Probe::named("a1"));

        // A reordered list that drops a child, duplicates one, or adds a
        // stranger is a reconciler bug: refuse it wholesale.
        for bad in [
            &[][..],         // drops b
            &[a, a, b][..],  // duplicates a
            &[a, b, a1][..], // a1 is a stranger (child of a)
        ] {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                arena.reorder_children(root, bad);
            }));
            assert!(
                result.is_err(),
                "malformed reorder list must panic: {bad:?}"
            );
        }
    }

    #[test]
    fn widget_mut_merges_masks_upward_on_drop() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let a = arena.add_child(root, Probe::named("a"));
        let a1 = arena.add_child(a, Probe::named("a1"));
        let _ = arena.take_masks(Change::ALL); // isolate from mount masks

        {
            let mut w = arena.mutate(a1);
            w.request(Change::LAYOUT);
        }

        assert_eq!(arena.mask(a1), Change::LAYOUT);
        assert_eq!(
            arena.mask(a),
            Change::LAYOUT.union(Change::PAINT).union(Change::A11Y)
        );
        assert!(arena.mask(root).contains(Change::LAYOUT));
        assert!(arena.mask(root).contains(Change::PAINT));
        assert!(!arena.mask(root).contains(Change::STRUCTURE));
    }

    #[test]
    fn add_child_marks_ancestors_structural() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let a = arena.add_child(root, Probe::named("a"));
        assert_eq!(arena.mask(a), Change::STRUCTURE);
        assert!(arena.mask(root).contains(Change::LAYOUT));
        assert!(arena.mask(root).contains(Change::A11Y));
    }

    #[test]
    fn remove_subtree_unindexes_and_notifies_parent() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let a = arena.add_child(root, Probe::named("a"));
        let a1 = arena.add_child(a, Probe::named("a1"));
        arena.remove_subtree(a);

        assert_eq!(arena.children(root), &[]);
        assert_eq!(arena.len(), 1);
        let _ = a1; // subtree ids are no longer live
    }

    #[test]
    #[should_panic(expected = "arena mutation already in progress")]
    fn leaked_lease_still_panics_the_next_mutation() {
        // The borrow checker makes re-entrancy through a live `WidgetMut`
        // statically impossible; the runtime lease is defense in depth for
        // paths that bypass the borrow (leaked guards, future interior-mutability
        // handles). `mem::forget` drops the guard without running its cleanup,
        // exactly such a path.
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let mut guard = arena.mutate(root);
        let _ = guard.widget_mut();
        std::mem::forget(guard); // lease never released
        let _child = arena.add_child(root, Probe::named("child")); // panics
    }

    #[test]
    fn queued_mutations_drain_through_the_lease() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let a = arena.add_child(root, Probe::named("a"));
        arena.queue_mutation(
            a,
            Box::new(|w| {
                let _ = w;
            }),
        );
        assert_eq!(arena.pending_mutations(), 1);
        let _ = arena.take_masks(Change::ALL); // isolate from mount masks
        let mutated = arena.drain_mutations();
        assert_eq!(mutated, vec![a], "the mutated node is reported to pass 5");
        assert_eq!(arena.pending_mutations(), 0);
        assert_eq!(arena.mask(a), Change::empty());
    }

    #[test]
    fn take_masks_drains_only_the_filtered_bits() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let _ = arena.take_masks(Change::ALL); // isolate from mount masks
        arena.merge_mask(root, Change::PAINT.union(Change::A11Y));

        let drained = arena.take_masks(Change::PAINT);
        assert_eq!(drained, vec![(root, Change::PAINT)]);
        assert_eq!(arena.mask(root), Change::A11Y);
    }

    #[test]
    fn update_bounds_writes_geometry_without_dirtying() {
        let mut arena = Arena::new();
        let root = arena.set_root(Probe::named("root"));
        let _ = arena.take_masks(Change::ALL); // isolate from mount masks
        let rect = Rect::new(crate::geom::Point::zero(), Size::new(100.0, 40.0));
        arena.update_bounds(root, rect);
        assert_eq!(arena.bounds(root), Some(rect));
        assert_eq!(
            arena.mask(root),
            Change::empty(),
            "geometry writes never dirty"
        );
    }

    #[test]
    fn node_ids_are_unique_and_displayed() {
        let a = NodeId::allocate();
        let b = NodeId::allocate();
        assert_ne!(a, b);
        assert_eq!(format!("{a}"), format!("#{}", a.raw()));
        assert_ne!(a.raw(), NodeId::INVALID.raw());
    }
}
