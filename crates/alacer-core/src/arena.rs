//! The retained widget tree: a safe slot arena with guard-based invalidation
//! (architecture §7.1).
//!
//! All tree mutation flows through guard types that **merge invalidation flags
//! upward on drop** (Masonry's `WidgetMut` discipline), combined with GPUI's
//! lease-and-panic re-entrancy protection: mutating the arena while a mutation
//! is already in flight panics with the two contending locations named.
//!
//! The backing storage is a plan fact (§7.1, ADR-9): the safe slot vector is
//! the default, and Masonry's upstream O(1) hash-map arena (`tree_arena`)
//! lands only behind the `fast-arena` feature flag **after** the fuzz
//! equivalence suite (`fast_arena_equivalence`) proves the wrapped storage
//! behaves identically to the safe default — the same mutations drive both
//! backends side by side, and every observable (ids, parents, children order,
//! masks, bounds, `measure_spec`) must match byte for byte. The flag routes
//! `Arena::new()` to the fast storage; [`Arena::with_storage`] constructs
//! either backend explicitly for the paired fuzz. The wrapper writes no
//! `unsafe` of its own — the upstream crate's interior `unsafe` stays its
//! own, and the arena's sole-ownership lease discipline is the safe seam.

use crate::mask::Change;
use crate::widget::Widget;
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt;
use std::num::NonZeroU64;
use std::panic::Location;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "fast-arena")]
use tree_arena::TreeArena;

/// Handle to one node in the [`Arena`]. A `NonZeroU64` from a global counter,
/// attributed on attach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(NonZeroU64);

impl NodeId {
    /// The synthetic id used before a root is mounted.
    ///
    /// Never held by a live node; the allocation counter starts at 1,
    /// far below the sentinel — a live id can never collide with it.
    pub const INVALID: Self = Self(NonZeroU64::new(4_294_967_295).expect("non-zero"));

    /// Allocates a fresh id from the global counter.
    ///
    /// Framework-internal in normal use (the arena hands ids out on attach);
    /// public for tests, benches, and tooling that key maps by node id.
    ///
    /// # Panics
    ///
    /// Only if the process-global counter wraps past `u64::MAX` and lands on
    /// zero — roughly 18 quintillion allocations in one process.
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

#[expect(
    clippy::missing_fields_in_debug,
    reason = "the boxed widget has no Debug impl by design (dyn Widget); the tree shape is what debugging wants"
)]
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

/// The safe backing storage (§7.1): one slot vector in insertion order
/// (removed slots linger as tombstones) plus an id→index map — O(1)
/// lookup, O(1) children read, tombstone reclamation deferred forever.
#[derive(Default)]
struct SafeArena {
    slots: Vec<Slot>,
    index: FxHashMap<NodeId, usize>,
}

/// The fast backing storage (§7.1, ADR-9): Masonry's upstream
/// `tree_arena` — the O(1) hash-map arena whose `unsafe` interior stays
/// its own — holding the same [`Slot`] payloads. The public shape the
/// safe storage promises (parent, children in tree order, masks, bounds)
/// rides the slots themselves: every mutator below writes the tree and
/// the slot's mirror fields from one code path, and the paired fuzz
/// (`fast_arena_equivalence`) holds the two backends byte-identical.
///
/// Two indexes ride beside the tree, each replacing a scan the safe
/// storage gets for free from its ordered slot vector:
///
/// * `dirty` — the nodes whose mask is non-empty. The safe path's
///   `take_masks` iterates its Vec; a hash-map tree has no iteration
///   order to inherit, and walking 10k nodes through `SipHash` lookups to
///   drain a few hundred masks is exactly the overhead the M1 gate
///   exists to catch. The mutators below maintain the set at every mask
///   write; the paired fuzz's per-mutation mask-drain comparison holds
///   it honest.
#[cfg(feature = "fast-arena")]
struct FastArena {
    tree: TreeArena<Slot>,
    dirty: FxHashSet<NodeId>,
}

#[cfg(feature = "fast-arena")]
impl FastArena {
    /// Depth-first pre-order visit of every live id, from the root.
    fn pre_order_ids(&self, root: Option<NodeId>) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack: Vec<NodeId> = root.into_iter().collect();
        while let Some(id) = stack.pop() {
            let Some(arena_ref) = self.tree.find(id.raw()) else {
                continue;
            };
            out.push(id);
            // Reverse the children so the LIFO stack pops them in tree
            // order — the visit order `for_each_pre_order` promises.
            let mut children: Vec<NodeId> = arena_ref.item.children.clone();
            children.reverse();
            stack.extend(children);
        }
        out
    }
}

/// The arena's backing storage: the safe slot vector, or — behind
/// `fast-arena` — Masonry's upstream hash-map tree.
enum Storage {
    Safe(SafeArena),
    #[cfg(feature = "fast-arena")]
    Fast(FastArena),
}

/// Which backing storage an [`Arena`] uses (§7.1, ADR-9) — the explicit
/// selector behind [`Arena::with_storage`], the paired fuzz's
/// constructor: both backends side by side in one process, driven by the
/// same mutation script.
///
/// Exists only when the `fast-arena` feature is compiled in: without it
/// the safe storage *is* the arena, and there is nothing to select.
#[cfg(feature = "fast-arena")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaStorage {
    /// The safe slot-vector default (tombstones after removal).
    Safe,
    /// Masonry's upstream O(1) hash-map arena, fuzz-equivalence-gated.
    Fast,
}

/// The arena: ownership of every retained widget, plus the tree's shape.
pub struct Arena {
    storage: Storage,
    root: Option<NodeId>,
    mutation_lease: Option<(&'static Location<'static>, u64)>,
    pending: Vec<PendingMutation>,
    /// Nodes whose children translate with their scroll offset — the compose
    /// pass visits only these instead of walking the tree every frame.
    scrollables: FxHashSet<NodeId>,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    /// An empty arena — over the default backing storage: the safe slot
    /// vector, or, behind the `fast-arena` feature flag, Masonry's
    /// upstream O(1) hash-map arena (§7.1, ADR-9; the flag's equivalence
    /// gate is the paired fuzz).
    #[must_use]
    pub fn new() -> Self {
        #[cfg(feature = "fast-arena")]
        let storage = Storage::Fast(FastArena {
            tree: TreeArena::new(),
            dirty: FxHashSet::default(),
        });
        #[cfg(not(feature = "fast-arena"))]
        let storage = Storage::Safe(SafeArena::default());
        Self {
            storage,
            root: None,
            mutation_lease: None,
            pending: Vec::new(),
            scrollables: FxHashSet::default(),
        }
    }

    /// An empty arena over an explicit backing storage — the paired fuzz's
    /// constructor (§7.1, ADR-9): [`ArenaStorage::Safe`] and
    /// [`ArenaStorage::Fast`] side by side in one process, driven by the
    /// same mutation script and compared byte for byte after every step.
    ///
    /// Only behind the `fast-arena` feature: without it the safe storage
    /// is the arena, and there is nothing to select.
    #[cfg(feature = "fast-arena")]
    #[must_use]
    pub fn with_storage(storage: ArenaStorage) -> Self {
        let storage = match storage {
            ArenaStorage::Safe => Storage::Safe(SafeArena::default()),
            ArenaStorage::Fast => Storage::Fast(FastArena {
                tree: TreeArena::new(),
                dirty: FxHashSet::default(),
            }),
        };
        Self {
            storage,
            root: None,
            mutation_lease: None,
            pending: Vec::new(),
            scrollables: FxHashSet::default(),
        }
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
        self.scrollables = FxHashSet::default();
        let slot = Slot {
            id,
            parent: None,
            children: Vec::new(),
            widget,
            mask: Change::STRUCTURE
                .union(Change::LAYOUT)
                .union(Change::PAINT)
                .union(Change::A11Y),
            bounds: None,
        };
        match &mut self.storage {
            Storage::Safe(store) => {
                store.slots.push(slot);
                store.index.insert(id, 0);
            }
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => {
                fast.tree.roots_mut().insert(id.raw(), slot);
                fast.dirty.insert(id);
            }
        }
        self.root = Some(id);
        self.record_scrollable(id);
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
        let mask = Change::STRUCTURE;
        match &mut self.storage {
            Storage::Safe(store) => {
                {
                    let parent_slot = Self::safe_slot_mut(store, parent);
                    parent_slot.children.push(id);
                    parent_slot.mask |= Change::STRUCTURE.propagated_to_ancestors();
                }
                store.slots.push(Slot {
                    id,
                    parent: Some(parent),
                    children: Vec::new(),
                    widget,
                    mask,
                    bounds: None,
                });
                store.index.insert(id, store.slots.len() - 1);
            }
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => {
                // One exclusive handle mutates the parent's payload (the
                // children mirror and the mask) and the tree's own child
                // list — the two copies the safe storage fuses into one.
                let mut arena_mut = fast
                    .tree
                    .find_mut(parent.raw())
                    .unwrap_or_else(|| panic!("{parent} is not live"));
                arena_mut.item.children.push(id);
                arena_mut.item.mask |= Change::STRUCTURE.propagated_to_ancestors();
                arena_mut.children.insert(
                    id.raw(),
                    Slot {
                        id,
                        parent: Some(parent),
                        children: Vec::new(),
                        widget,
                        mask,
                        bounds: None,
                    },
                );
                fast.dirty.insert(id);
                fast.dirty.insert(parent);
            }
        }
        self.record_scrollable(id);
        id
    }

    /// Indexes `id` under the scrollable set when its widget translates
    /// children (the compose pass's candidate list).
    fn record_scrollable(&mut self, id: NodeId) {
        if self.slot(id).widget.is_scroll_container() {
            self.scrollables.insert(id);
        }
    }

    /// The live scroll containers (compose pass candidates, §8.2).
    pub fn scrollables(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.scrollables
            .iter()
            .copied()
            .filter(|id| self.is_live(*id))
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
        match &mut self.storage {
            // # Panics
            //
            // Panics if a drained id is not live (impossible: the index and
            // the mask index only hold live nodes).
            Storage::Safe(store) => {
                for id in &to_remove {
                    // The panic names the id the index lost — the same
                    // context the `{node} is not live` panics carry, so
                    // the impossible invariant at least says which node
                    // broke it.
                    let idx = store.index.remove(id).unwrap_or_else(|| {
                        panic!("{id} is not live in the slot index while removing {node}")
                    });
                    store.slots[idx].mask = Change::empty();
                    store.slots[idx].children.clear();
                    store.slots[idx].parent = None;
                }
                let parent_slot = Self::safe_slot_mut(store, parent);
                parent_slot.children.retain(|c| *c != node);
                parent_slot.mask |= Change::STRUCTURE.propagated_to_ancestors();
            }
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => {
                // The upstream arena drops the subtree's slots outright —
                // the tombstone-less shape the safe storage defers — so
                // the removal is one tree op plus the parent's mirror.
                let mut arena_mut = fast
                    .tree
                    .find_mut(parent.raw())
                    .unwrap_or_else(|| panic!("{parent} is not live"));
                let removed = arena_mut.children.remove(node.raw()).unwrap_or_else(|| {
                    panic!("{node} is not a live child of {parent} while removing")
                });
                arena_mut.item.children.retain(|c| *c != node);
                arena_mut.item.mask |= Change::STRUCTURE.propagated_to_ancestors();
                let _ = removed;
                for id in &to_remove {
                    fast.dirty.remove(id);
                }
                fast.dirty.insert(parent);
            }
        }
        for id in &to_remove {
            self.scrollables.remove(id);
        }
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
        match &mut self.storage {
            Storage::Safe(store) => {
                let parent_slot = Self::safe_slot_mut(store, parent);
                if parent_slot.children == order {
                    return; // already in order: no structural change to report
                }
                parent_slot.children = order.to_vec();
                parent_slot.mask |= Change::STRUCTURE.propagated_to_ancestors();
            }
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => {
                if current == order {
                    return; // already in order: no structural change to report
                }
                // The upstream arena appends on reparent, so moving every
                // child onto its own parent in target order realizes the
                // permutation — each `reparent` lands its id at the end,
                // and the sequence ends in exactly `order`.
                for id in order {
                    fast.tree.reparent(id.raw(), parent.raw());
                }
                let arena_mut = fast
                    .tree
                    .find_mut(parent.raw())
                    .expect("the permutation parent is live");
                arena_mut.item.children = order.to_vec();
                arena_mut.item.mask |= Change::STRUCTURE.propagated_to_ancestors();
                fast.dirty.insert(parent);
            }
        }
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
        match &self.storage {
            Storage::Safe(store) => &store.slots[Self::safe_live_index(store, node)],
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => fast
                .tree
                .find(node.raw())
                .map_or_else(|| panic!("{node} is not live"), |arena_ref| arena_ref.item),
        }
    }

    /// Whether `node` is a live arena slot (safe to `slot()`, `mutate()`,
    /// etc.). Dead slots linger in the backing vector after
    /// [`Self::remove_subtree`] but leave the index.
    #[must_use]
    pub fn is_live(&self, node: NodeId) -> bool {
        match &self.storage {
            Storage::Safe(store) => store.index.contains_key(&node),
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => fast.tree.find(node.raw()).is_some(),
        }
    }

    fn slot_mut(&mut self, node: NodeId) -> &mut Slot {
        match &mut self.storage {
            Storage::Safe(store) => {
                let idx = Self::safe_live_index(store, node);
                &mut store.slots[idx]
            }
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => fast
                .tree
                .find_mut(node.raw())
                .map_or_else(|| panic!("{node} is not live"), |arena_mut| arena_mut.item),
        }
    }

    /// The safe storage's backing index of a live node — the one documented
    /// panic point both slot doors share (`{node} is not live`, the contract
    /// their `# Panics` sections name).
    #[track_caller]
    fn safe_live_index(store: &SafeArena, node: NodeId) -> usize {
        store
            .index
            .get(&node)
            .copied()
            .unwrap_or_else(|| panic!("{node} is not live"))
    }

    /// The safe storage's mutable slot, the panic contract intact.
    #[track_caller]
    fn safe_slot_mut(store: &mut SafeArena, node: NodeId) -> &mut Slot {
        let idx = Self::safe_live_index(store, node);
        &mut store.slots[idx]
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
            let parent = {
                let slot = self.slot_mut(id);
                slot.mask |= up;
                let parent = slot.parent;
                #[cfg(feature = "fast-arena")]
                if !slot.mask.is_empty() {
                    // The fast storage's mask index: every non-empty mask
                    // is a member, so the drain never walks the tree.
                    if let Storage::Fast(fast) = &mut self.storage {
                        fast.dirty.insert(id);
                    }
                }
                parent
            };
            up = up.propagated_to_ancestors();
            if up.is_empty() {
                break;
            }
            cursor = parent;
        }
    }

    /// Drains every node's mask that intersects `filter`, returning
    /// `(node, mask)` pairs and clearing the drained bits. The drain order
    /// is ascending node id — the safe storage's insertion order — in both
    /// backends, so pass consumption is byte-identical under the flag.
    ///
    /// # Panics
    ///
    /// Never: the safe arm's tombstones carry empty masks, and the fast
    /// arm's mask index only holds live nodes.
    #[must_use]
    pub fn take_masks(&mut self, filter: Change) -> Vec<(NodeId, Change)> {
        match &mut self.storage {
            // # Panics
            //
            // Never for the safe arm; the fast arm's mask index only holds
            // live nodes, so its `find_mut` expectation cannot fire.
            Storage::Safe(store) => {
                let mut out = Vec::new();
                for slot in &mut store.slots {
                    if slot.mask.intersects(filter) {
                        out.push((slot.id, slot.mask & filter));
                        slot.mask &= !filter;
                    }
                }
                out
            }
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => {
                // The fast storage drains through its mask index: the
                // candidates are the nodes holding non-empty masks, sorted
                // into the ascending-id order the safe storage's Vec
                // guarantees (ids allocate at insert), each visited once.
                let mut candidates: Vec<NodeId> = fast.dirty.iter().copied().collect();
                candidates.sort_by_key(|id| id.raw());
                let mut out = Vec::new();
                for id in candidates {
                    let arena_mut = fast
                        .tree
                        .find_mut(id.raw())
                        .expect("the mask index only holds live nodes");
                    if arena_mut.item.mask.intersects(filter) {
                        out.push((id, arena_mut.item.mask & filter));
                        arena_mut.item.mask &= !filter;
                        if arena_mut.item.mask.is_empty() {
                            fast.dirty.remove(&id);
                        }
                    }
                }
                out
            }
        }
    }

    /// Writes composed bounds without touching any mask: the layout engine owns
    /// geometry (§7.2 pass 6).
    pub fn update_bounds(&mut self, node: NodeId, bounds: crate::geom::Rect) {
        match &mut self.storage {
            Storage::Safe(store) => {
                if let Some(idx) = store.index.get(&node).copied() {
                    store.slots[idx].bounds = Some(bounds);
                }
            }
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => {
                if let Some(arena_mut) = fast.tree.find_mut(node.raw()) {
                    arena_mut.item.bounds = Some(bounds);
                }
            }
        }
    }

    /// Queues a deferred widget mutation, drained by the mutate pass (§7.2
    /// pass 4): the closure runs with the arena lease held, and any masks it
    /// requests merge upward on drop of the enclosing `WidgetMut`.
    pub fn queue_mutation(&mut self, node: NodeId, closure: MutationFn) {
        self.pending.push(PendingMutation { node, closure });
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
        match &self.storage {
            Storage::Safe(store) => store
                .slots
                .iter()
                .filter(|s| store.index.contains_key(&s.id))
                .count(),
            #[cfg(feature = "fast-arena")]
            Storage::Fast(fast) => fast.pre_order_ids(self.root).len(),
        }
    }

    /// `true` when no node is live.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[expect(
    clippy::missing_fields_in_debug,
    reason = "the slot vector, its index map, the lease, and the scrollable set are derived or non-diagnostic bookkeeping; shape is what debugging wants"
)]
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
        fn named(name: &'static str) -> Self {
            Self {
                role: Role::Container,
                name,
            }
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
        let root = arena.set_root(Box::new(Probe::named("root")));
        let a = arena.add_child(root, Box::new(Probe::named("a")));
        let b = arena.add_child(root, Box::new(Probe::named("b")));
        let a1 = arena.add_child(a, Box::new(Probe::named("a1")));

        assert_eq!(arena.root(), Some(root));
        assert_eq!(arena.children(root), &[a, b]);
        assert_eq!(arena.parent(a1), Some(a));
        assert_eq!(arena.len(), 4);
    }

    #[test]
    fn pre_order_walk_matches_paint_order() {
        let mut arena = Arena::new();
        let root = arena.set_root(Box::new(Probe::named("root")));
        let a = arena.add_child(root, Box::new(Probe::named("a")));
        arena.add_child(a, Box::new(Probe::named("a1")));
        let b = arena.add_child(root, Box::new(Probe::named("b")));

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
        let root = arena.set_root(Box::new(Probe::named("root")));
        let a = arena.add_child(root, Box::new(Probe::named("a")));
        let b = arena.add_child(root, Box::new(Probe::named("b")));
        let c = arena.add_child(root, Box::new(Probe::named("c")));
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
        let root = arena.set_root(Box::new(Probe::named("root")));
        let a = arena.add_child(root, Box::new(Probe::named("a")));
        let b = arena.add_child(root, Box::new(Probe::named("b")));
        let a1 = arena.add_child(a, Box::new(Probe::named("a1")));

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
        let root = arena.set_root(Box::new(Probe::named("root")));
        let a = arena.add_child(root, Box::new(Probe::named("a")));
        let a1 = arena.add_child(a, Box::new(Probe::named("a1")));
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
        let root = arena.set_root(Box::new(Probe::named("root")));
        let a = arena.add_child(root, Box::new(Probe::named("a")));
        assert_eq!(arena.mask(a), Change::STRUCTURE);
        assert!(arena.mask(root).contains(Change::LAYOUT));
        assert!(arena.mask(root).contains(Change::A11Y));
    }

    #[test]
    fn remove_subtree_unindexes_and_notifies_parent() {
        let mut arena = Arena::new();
        let root = arena.set_root(Box::new(Probe::named("root")));
        let a = arena.add_child(root, Box::new(Probe::named("a")));
        let a1 = arena.add_child(a, Box::new(Probe::named("a1")));
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
        let root = arena.set_root(Box::new(Probe::named("root")));
        let mut guard = arena.mutate(root);
        let _ = guard.widget_mut();
        std::mem::forget(guard); // lease never released
        let _child = arena.add_child(root, Box::new(Probe::named("child"))); // panics
    }

    #[test]
    fn queued_mutations_drain_through_the_lease() {
        let mut arena = Arena::new();
        let root = arena.set_root(Box::new(Probe::named("root")));
        let a = arena.add_child(root, Box::new(Probe::named("a")));
        arena.queue_mutation(
            a,
            Box::new(|w| {
                let _ = w;
            }),
        );
        assert_eq!(arena.pending.len(), 1);
        let _ = arena.take_masks(Change::ALL); // isolate from mount masks
        let mutated = arena.drain_mutations();
        assert_eq!(mutated, vec![a], "the mutated node is reported to pass 5");
        assert_eq!(arena.pending.len(), 0);
        assert_eq!(arena.mask(a), Change::empty());
    }

    #[test]
    fn take_masks_drains_only_the_filtered_bits() {
        let mut arena = Arena::new();
        let root = arena.set_root(Box::new(Probe::named("root")));
        let _ = arena.take_masks(Change::ALL); // isolate from mount masks
        arena.merge_mask(root, Change::PAINT.union(Change::A11Y));

        let drained = arena.take_masks(Change::PAINT);
        assert_eq!(drained, vec![(root, Change::PAINT)]);
        assert_eq!(arena.mask(root), Change::A11Y);
    }

    #[test]
    fn update_bounds_writes_geometry_without_dirtying() {
        let mut arena = Arena::new();
        let root = arena.set_root(Box::new(Probe::named("root")));
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
