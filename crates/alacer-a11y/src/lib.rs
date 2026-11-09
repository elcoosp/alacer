//! Alacer's accessibility: the AccessKit contract (ADR-5, M3 edition).
//!
//! §11.1, implemented:
//!
//! - AccessKit `NodeId`s are the id-path hashes — the same identity as
//!   hit-testing (§6.3); the a11y tree *is* the widget tree.
//! - Updates are **incremental deltas** (§11.1 item 4): pass 9 hands us an
//!   [`A11yDelta`] — dirty ids, removals, focus — and only dirty nodes are
//!   re-emitted in the [`TreeUpdate`] (never a full-tree rebuild per frame,
//!   the egui cost we refuse). The M3 exit gate measures exactly this:
//!   **`TreeUpdate` p99 < 0.5 ms @ 1k nodes** (`tests/m3_exit_criteria.rs`).
//! - Roles map from the core `Role` enum; labels resolve through the
//!   required `label()` source — an unnamed widget is impossible by
//!   construction. Editable widgets expose their content as the node's value.
//! - Focus travels in the delta: `TreeUpdate::focus` follows the pipeline's
//!   pass-3 output every a11y frame.

use accesskit::{
    Live, Node, NodeId, Rect, Role as AccessKitRole, TextPosition, TextSelection, Toggled, TreeId,
    TreeInfo, TreeUpdate,
};
use alacer_core::Role;
use alacer_core::announce::Politeness;
use alacer_core::geom::Rect as AlacerRect;
use alacer_core::passes::{A11yDelta, A11ySink, TreeSnapshot};
use alacer_core::widget::TableSlot;
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

/// Builds an AccessKit node id from its raw hash (§6.3: same identity).
#[must_use]
pub fn accesskit_node_id(hash: u64) -> NodeId {
    NodeId(hash)
}

/// Maps Alacer's semantic roles onto AccessKit roles. The M6 canon roles
/// (checkbox, table family) land on their AccessKit counterparts so a real
/// platform adapter sees a checkbox and a navigable grid.
#[must_use]
pub fn map_role(role: Role) -> AccessKitRole {
    match role {
        Role::Decoration => AccessKitRole::Unknown,
        Role::Container => AccessKitRole::GenericContainer,
        Role::Label => AccessKitRole::Label,
        Role::Button => AccessKitRole::Button,
        Role::TextInput => AccessKitRole::TextInput,
        Role::ScrollArea => AccessKitRole::ScrollView,
        Role::ListItem => AccessKitRole::ListItem,
        Role::List => AccessKitRole::List,
        Role::CheckBox => AccessKitRole::CheckBox,
        Role::Table => AccessKitRole::Table,
        Role::TableRow => AccessKitRole::Row,
        Role::TableCell => AccessKitRole::Cell,
        Role::ColumnHeader => AccessKitRole::ColumnHeader,
        // The announcement region (§11.6, ADR-0024): the polite case's ARIA
        // role ("status"). The assertive case re-roles to "alert" in
        // [`map_announcement_role`], where the politeness is known.
        Role::LiveRegion => AccessKitRole::Status,
    }
}

/// Maps an announcement's politeness onto the AccessKit live priority —
/// the `aria-live` property a screen reader watches (`Live::Polite` speaks
/// at the next break, `Live::Assertive` interrupts).
#[must_use]
pub const fn map_live(politeness: Politeness) -> Live {
    match politeness {
        Politeness::Polite => Live::Polite,
        Politeness::Assertive => Live::Assertive,
    }
}

/// The announcement node's role for its politeness: the ARIA pairing —
/// `role="status"` speaks politely, `role="alert"` interrupts. The base
/// mapping ([`map_role`]) answers `Status` for the role alone; the sink
/// re-roles where the politeness is known.
#[must_use]
pub const fn map_announcement_role(politeness: Politeness) -> AccessKitRole {
    match politeness {
        Politeness::Polite => AccessKitRole::Status,
        Politeness::Assertive => AccessKitRole::Alert,
    }
}

/// Converts an Alacer rect into AccessKit's `f64`-based rect.
#[must_use]
pub fn map_bounds(rect: AlacerRect) -> Rect {
    Rect::new(
        f64::from(rect.min_x()),
        f64::from(rect.min_y()),
        f64::from(rect.max_x()),
        f64::from(rect.max_y()),
    )
}

/// The AccessKit sink: incremental [`TreeUpdate`]s from [`A11yDelta`]s.
///
/// Produced updates accumulate in a bounded ring (see [`UPDATE_RING_CAP`]):
/// a windowed app drains them at frame boundaries (ADR-0015), and an
/// undrained sink — an app with no adapter attached — is capped instead of
/// growing forever.
#[derive(Debug, Default)]
pub struct AccessKitSink {
    updates: Vec<TreeUpdate>,
    initialized: bool,
    last_snapshot: Option<std::sync::Arc<TreeSnapshot>>,
    /// The ids included in the most recent update (harness/test introspection).
    last_update_ids: FxHashSet<u64>,
    /// The ids the most recent update dropped (dead widgets; introspection).
    last_removed_ids: Vec<u64>,
    update_count: u64,
}

/// How many produced updates the sink retains before the oldest one is
/// dropped. An assistive technology that connects mid-run re-requests the
/// whole tree ([`AccessKitSink::full_tree_update`]), so dropping stale
/// deltas never loses correctness — it only bounds memory.
pub const UPDATE_RING_CAP: usize = 8;

impl AccessKitSink {
    /// An empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of updates produced so far.
    #[must_use]
    pub const fn update_count(&self) -> u64 {
        self.update_count
    }

    /// The most recent update (test/platform consumption).
    #[must_use]
    pub fn last_update(&self) -> Option<&TreeUpdate> {
        self.updates.last()
    }

    /// The ids re-emitted by the most recent update.
    #[must_use]
    pub fn last_update_ids(&self) -> &FxHashSet<u64> {
        &self.last_update_ids
    }

    /// The ids dropped by the most recent update (dead widgets).
    #[must_use]
    pub fn last_removed_ids(&self) -> &[u64] {
        &self.last_removed_ids
    }

    /// Whether the last update carried full tree (re)initialization data.
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Takes every update produced since the last drain, in order — the
    /// platform's steady-state frame-boundary step (ADR-0015): the loop
    /// hands each one to the AccessKit adapter, which forwards it while an
    /// assistive technology is connected and drops it otherwise.
    ///
    /// Draining is the memory contract, too: a windowed app that never
    /// calls this (no adapter attached) is still bounded by
    /// [`UPDATE_RING_CAP`].
    #[must_use]
    pub fn drain_updates(&mut self) -> Vec<TreeUpdate> {
        std::mem::take(&mut self.updates)
    }

    /// The full current tree as one `TreeUpdate` — the response to an
    /// adapter-activation request (`InitialTreeRequested`), built by the
    /// same node builder the deltas use. Pure: no bookkeeping changes, no
    /// update enters the ledger. `None` before the first
    /// [`A11ySink::apply`] (no resolved tree yet — the platform defers to
    /// the first frame, whose init delta covers it).
    #[must_use]
    pub fn full_tree_update(&self) -> Option<TreeUpdate> {
        let snapshot = self.last_snapshot.as_ref()?;
        let delta = A11yDelta {
            initializes: true,
            ..A11yDelta::empty()
        };
        Some(Self::build_update(
            snapshot,
            &delta,
            true,
            &mut FxHashSet::default(),
        ))
    }

    /// Builds one [`TreeUpdate`] from a snapshot and its delta — the single
    /// materialization path for every update this crate ever produces.
    /// `force_full` (an initializing delta, or the sink's first frame) builds
    /// the whole tree; otherwise only the updated ids re-emit (§11.1 item 4 —
    /// the p99 budget's whole point). `ids` receives the emitted ids (the
    /// sink's introspection bookkeeping; a scratch set when the caller wants
    /// a pure build, as [`Self::full_tree_update`] does).
    fn build_update(
        snapshot: &TreeSnapshot,
        delta: &A11yDelta,
        force_full: bool,
        ids: &mut FxHashSet<u64>,
    ) -> TreeUpdate {
        ids.clear();
        let mut update = TreeUpdate {
            nodes: Vec::new(),
            tree: None,
            tree_id: TreeId::ROOT,
            focus: delta.focus.map_or(NodeId(0), accesskit_node_id),
        };

        if force_full {
            update.tree = snapshot
                .root
                .map(|root| TreeInfo::new(accesskit_node_id(root)));
        }
        for node in &snapshot.nodes {
            if !force_full && !delta.updated.contains(&node.id) {
                continue;
            }
            ids.insert(node.id);
            update
                .nodes
                .push((accesskit_node_id(node.id), Self::access_node(node)));
        }
        update
    }

    /// Converts one neutral snapshot node into an AccessKit node. The M6
    /// vocabulary rides along: toggle state becomes the `Toggled` property,
    /// and table slots become the grid attributes (row/column counts and
    /// zero-based indexes) that make a table navigable, not just visible.
    fn access_node(node: &alacer_core::passes::TreeNode) -> Node {
        let mut access_node = Node::new(map_role(node.role));
        access_node.set_label(node.label.as_str());
        if let Some(bounds) = node.bounds {
            access_node.set_bounds(map_bounds(bounds));
        }
        // The announcement region (§11.6, ADR-0024): the politeness rides
        // the `aria-live` property, and the role follows the ARIA pairing —
        // polite is "status", assertive interrupts as "alert".
        if let Some(politeness) = node.live {
            access_node.set_live(map_live(politeness));
            access_node.set_role(map_announcement_role(politeness));
        }
        if let Some(value) = &node.value {
            access_node.set_value(value.as_str());
        }
        if let Some(toggled) = node.toggled {
            access_node.set_toggled(match toggled {
                alacer_core::widget::ToggleState::Off => Toggled::False,
                alacer_core::widget::ToggleState::On => Toggled::True,
                alacer_core::widget::ToggleState::Mixed => Toggled::Mixed,
            });
        }
        match node.table_slot {
            Some(TableSlot::Table { rows, columns }) => {
                access_node.set_row_count(rows);
                access_node.set_column_count(columns);
            }
            Some(TableSlot::Header { column }) => {
                access_node.set_row_index(0);
                access_node.set_column_index(column);
            }
            Some(TableSlot::Cell { row, column }) => {
                access_node.set_row_index(row);
                access_node.set_column_index(column);
            }
            None => {}
        }
        // The selection (M11, ADR-0017): anchor and focus as character
        // indexes on the value-bearing node itself. AccessKit's full rich
        // text model points `TextPosition`s at `TextRun` children; Alacer's
        // a11y tree is coarse (the value lives on the editable node), so the
        // positions point at the node that owns the string — a documented
        // simplification, UNVERIFIED at strict platform adapters, recorded
        // in the ADR like every other unverified seam.
        if let Some(selection) = node.selection {
            let node_id = accesskit_node_id(node.id);
            access_node.set_text_selection(TextSelection {
                anchor: TextPosition {
                    node: node_id,
                    character_index: selection.anchor,
                },
                focus: TextPosition {
                    node: node_id,
                    character_index: selection.focus,
                },
            });
        }
        access_node.set_children(
            node.children
                .iter()
                .map(|c| accesskit_node_id(*c))
                .collect::<Vec<_>>(),
        );
        access_node
    }
}

impl AccessKitSink {
    /// Queries the resolved tree: first node with `role` and name containing
    /// `name` — the `find(ROLE).named(...)` API (§11.5).
    #[must_use]
    pub fn find(&self, role: Role, name: &str) -> Option<&alacer_core::passes::TreeNode> {
        self.last_snapshot.as_ref()?.find(role, name)
    }
}

impl A11ySink for AccessKitSink {
    fn apply(&mut self, snapshot: std::sync::Arc<TreeSnapshot>, delta: &A11yDelta) {
        self.last_snapshot = Some(std::sync::Arc::clone(&snapshot));
        self.last_removed_ids.clone_from(&delta.removed);
        // The first apply is always a full build, initializing delta or not
        // (the adapter may connect before pass 9 ever marks one).
        let force_full = delta.initializes || !self.initialized;
        let update = Self::build_update(&snapshot, delta, force_full, &mut self.last_update_ids);
        self.initialized = true;
        self.update_count += 1;
        self.updates.push(update);
        if self.updates.len() > UPDATE_RING_CAP {
            self.updates.remove(0);
        }
    }

    fn resolved_tree(&self) -> Option<&TreeSnapshot> {
        self.last_snapshot.as_deref()
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

/// Finds a node in an update by id-path hash (the harness query helper).
#[must_use]
pub fn find_in_update(update: &TreeUpdate, id: u64) -> Option<&Node> {
    update
        .nodes
        .iter()
        .find(|(node_id, _)| node_id.0 == id)
        .map(|(_, node)| node)
}

/// Depth-first walk over the resolved nodes of an update, following children.
///
/// Returns ids in visit order — the harness builds queries from this.
#[must_use]
pub fn walk_update(update: &TreeUpdate) -> Vec<u64> {
    let map: std::collections::HashMap<u64, Vec<u64>> = update
        .nodes
        .iter()
        .map(|(id, node)| (id.0, node.children().iter().map(|c| c.0).collect()))
        .collect();
    let Some(&root) = map.keys().min() else {
        return Vec::new();
    };
    let mut order = Vec::new();
    let mut queue = VecDeque::from([root]);
    while let Some(id) = queue.pop_front() {
        order.push(id);
        if let Some(children) = map.get(&id) {
            for child in children.iter().rev() {
                queue.push_front(*child);
            }
        }
    }
    order
}

#[cfg(test)]
mod tests {
    use super::{
        AccessKitSink, Live, UPDATE_RING_CAP, accesskit_node_id, find_in_update,
        map_announcement_role, map_bounds, map_live, map_role, walk_update,
    };
    use accesskit::NodeId;
    use accesskit::Role as AkRole;
    use accesskit::Toggled;
    use alacer_core::Role;
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::passes::{A11yDelta, A11ySink, TreeNode, TreeSnapshot};
    use alacer_core::widget::TableSlot;
    use rustc_hash::FxHashSet;
    use std::collections::HashMap;

    fn snapshot() -> TreeSnapshot {
        let root_hash = 111_u64;
        let child_hash = 222_u64;
        let other_hash = 333_u64;
        TreeSnapshot {
            root: Some(root_hash),
            nodes: vec![
                TreeNode {
                    id: root_hash,
                    role: Role::Container,
                    label: "app".into(),
                    bounds: Some(Rect::new(Point::zero(), Size::new(200.0, 100.0))),
                    children: vec![child_hash, other_hash],
                    value: None,
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                },
                TreeNode {
                    id: child_hash,
                    role: Role::Button,
                    label: "Save".into(),
                    bounds: Some(Rect::new(Point::new(0.0, 0.0), Size::new(60.0, 24.0))),
                    children: Vec::new(),
                    value: None,
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                },
                TreeNode {
                    id: other_hash,
                    role: Role::TextInput,
                    label: "Search".into(),
                    bounds: Some(Rect::new(Point::new(0.0, 30.0), Size::new(60.0, 20.0))),
                    children: Vec::new(),
                    value: Some("hello".into()),
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                },
            ],
        }
    }

    #[test]
    fn roles_map_onto_accesskit() {
        assert_eq!(map_role(Role::Button), AkRole::Button);
        assert_eq!(map_role(Role::Label), AkRole::Label);
        assert_eq!(map_role(Role::Container), AkRole::GenericContainer);
        // The M6 canon roles.
        assert_eq!(map_role(Role::CheckBox), AkRole::CheckBox);
        assert_eq!(map_role(Role::Table), AkRole::Table);
        assert_eq!(map_role(Role::TableRow), AkRole::Row);
        assert_eq!(map_role(Role::TableCell), AkRole::Cell);
        assert_eq!(map_role(Role::ColumnHeader), AkRole::ColumnHeader);
    }

    #[test]
    fn bounds_map_with_f64_edges() {
        let r = map_bounds(Rect::new(Point::new(1.0, 2.0), Size::new(30.0, 40.0)));
        assert_eq!(r, accesskit::Rect::new(1.0, 2.0, 31.0, 42.0));
    }

    #[test]
    fn the_announcement_region_maps_onto_the_aria_pairing() {
        // §11.6, ADR-0024: the politeness rides the `aria-live` property
        // and the role follows the ARIA pairing — polite speaks as
        // "status", assertive interrupts as "alert".
        assert_eq!(
            map_role(Role::LiveRegion),
            AkRole::Status,
            "the base mapping answers the polite case's role"
        );
        assert_eq!(
            map_live(alacer_core::announce::Politeness::Polite),
            Live::Polite
        );
        assert_eq!(
            map_live(alacer_core::announce::Politeness::Assertive),
            Live::Assertive
        );
        assert_eq!(
            map_announcement_role(alacer_core::announce::Politeness::Polite),
            AkRole::Status
        );
        assert_eq!(
            map_announcement_role(alacer_core::announce::Politeness::Assertive),
            AkRole::Alert
        );

        // End to end through the sink: the region node carries both.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes.push(alacer_core::passes::TreeNode {
            id: alacer_core::announce::ANNOUNCEMENT_NODE_HASH,
            role: Role::LiveRegion,
            label: "Saved".into(),
            bounds: None,
            children: Vec::new(),
            value: None,
            toggled: None,
            table_slot: None,
            selection: None,
            live: Some(alacer_core::announce::Politeness::Assertive),
        });
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());
        let update = sink.last_update().expect("update");
        let region = find_in_update(update, alacer_core::announce::ANNOUNCEMENT_NODE_HASH)
            .expect("the region in the update");
        assert_eq!(
            region.live(),
            Some(Live::Assertive),
            "the AT reads the priority"
        );
        assert_eq!(region.role(), AkRole::Alert, "the AT reads the ARIA role");
        assert_eq!(region.label(), Some("Saved"));
    }

    #[test]
    fn first_update_initializes_the_whole_tree() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());

        assert!(sink.is_initialized());
        assert_eq!(sink.update_count(), 1);
        let update = sink.last_update().expect("update");
        assert!(update.tree.is_some(), "tree info present on init");
        assert_eq!(update.nodes.len(), 3, "every node in the init update");
        let button = find_in_update(update, 222).expect("button");
        assert_eq!(button.role(), AkRole::Button);
        assert_eq!(button.label(), Some("Save"));
        let input = find_in_update(update, 333).expect("input");
        assert_eq!(input.value(), Some("hello"), "value exposed");
    }

    #[test]
    fn later_updates_are_dirty_node_only() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());

        let mut updated = FxHashSet::default();
        updated.insert(222_u64); // only the button changed
        let mut delta = A11yDelta::empty();
        delta.updated = updated;
        sink.apply(std::sync::Arc::new(snap.clone()), &delta);

        let update = sink.last_update().expect("second update");
        assert_eq!(update.nodes.len(), 1, "only the dirty node re-emitted");
        assert_eq!(update.nodes[0].0.0, 222);
        assert!(
            !sink.last_update_ids().contains(&333),
            "clean node not re-emitted"
        );
    }

    #[test]
    fn focus_travels_in_the_delta() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());

        let mut delta = A11yDelta::empty();
        delta.focus = Some(222);
        sink.apply(std::sync::Arc::new(snap.clone()), &delta);
        let update = sink.last_update().expect("update");
        assert_eq!(update.focus, accesskit::NodeId(222));
    }

    #[test]
    fn removals_surface_in_the_sink() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());

        // The tree loses the text input; its parent is updated (STRUCTURE
        // propagation), the removed id surfaces in `delta.removed`.
        let mut smaller = snapshot();
        smaller.nodes.retain(|n| n.id != 333);
        smaller.nodes[0].children = vec![222];
        let mut delta = A11yDelta::empty();
        delta.updated.insert(111);
        delta.removed.push(333);
        sink.apply(std::sync::Arc::new(smaller), &delta);

        assert_eq!(sink.last_removed_ids(), &[333]);
        let update = sink.last_update().expect("update");
        let root = find_in_update(update, 111).expect("root re-emitted");
        assert_eq!(root.children(), &[accesskit::NodeId(222)]);
    }

    #[test]
    fn node_ids_are_the_id_path_hashes() {
        // §6.3: the same hash serves hit-testing and AccessKit.
        let mut path = alacer_core::identity::IdPath::default();
        path.push(7);
        path.push(9);
        assert_eq!(path.hash(), path.hash(), "stable");
        assert_ne!(path.hash(), 0);
    }

    #[test]
    fn walk_follows_children_in_order() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());
        let update = sink.last_update().expect("update");
        let order = walk_update(update);
        assert_eq!(order, vec![111, 222, 333]);

        // Parent-child consistency: every child id resolves to a node.
        let map: HashMap<u64, &accesskit::Node> =
            update.nodes.iter().map(|(id, n)| (id.0, n)).collect();
        for (id, node) in &map {
            for child in node.children() {
                assert!(map.contains_key(&child.0), "child of {id} exists");
            }
        }
    }

    /// The M6 vocabulary: a checkbox's toggle state and a data table's grid
    /// geometry must survive the neutral snapshot → AccessKit conversion.
    #[test]
    fn toggled_and_table_slots_map_onto_accesskit_attributes() {
        let mut sink = AccessKitSink::new();
        let snap = TreeSnapshot {
            root: Some(1),
            nodes: vec![
                TreeNode {
                    id: 1,
                    role: Role::Table,
                    label: "Inventory".into(),
                    bounds: None,
                    children: vec![2, 3],
                    value: None,
                    toggled: None,
                    table_slot: Some(TableSlot::Table {
                        rows: 2,
                        columns: 3,
                    }),
                    selection: None,
                    live: None,
                },
                TreeNode {
                    id: 2,
                    role: Role::ColumnHeader,
                    label: "Name".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: None,
                    toggled: None,
                    table_slot: Some(TableSlot::Header { column: 0 }),
                    selection: None,
                    live: None,
                },
                TreeNode {
                    id: 3,
                    role: Role::CheckBox,
                    label: "Ship in bulk".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: None,
                    toggled: Some(alacer_core::widget::ToggleState::On),
                    table_slot: Some(TableSlot::Cell { row: 1, column: 2 }),
                    selection: None,
                    live: None,
                },
            ],
        };
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());

        let update = sink.last_update().expect("update");
        let table = find_in_update(update, 1).expect("table node");
        assert_eq!(table.role(), AkRole::Table);
        assert_eq!(table.row_count(), Some(2), "grid rows published");
        assert_eq!(table.column_count(), Some(3), "grid columns published");

        let header = find_in_update(update, 2).expect("header node");
        assert_eq!(header.role(), AkRole::ColumnHeader);
        assert_eq!(header.column_index(), Some(0), "header column index");

        let checkbox = find_in_update(update, 3).expect("checkbox node");
        assert_eq!(checkbox.role(), AkRole::CheckBox);
        assert_eq!(checkbox.toggled(), Some(Toggled::True), "checked state");
        assert_eq!(checkbox.row_index(), Some(1), "cell row position");
        assert_eq!(checkbox.column_index(), Some(2), "cell column position");
    }

    // -------------------------------------------------------------------
    // M9 (ADR-0015): the windowed path drains these deltas. The sink owns
    // the whole protocol surface — `drain_updates` for the steady-state
    // frame, `full_tree_update` for the adapter-activation request — so the
    // platform layer never builds a `TreeUpdate` of its own again.
    // -------------------------------------------------------------------

    #[test]
    fn drain_updates_returns_the_produced_updates_and_empties_the_buffer() {
        let mut sink = AccessKitSink::new();
        let snap = std::sync::Arc::new(snapshot());
        sink.apply(
            std::sync::Arc::clone(&snap),
            &A11yDelta {
                initializes: true,
                ..A11yDelta::empty()
            },
        );

        let drained = sink.drain_updates();
        assert_eq!(drained.len(), 1, "the init update drains exactly once");
        assert!(
            drained[0].tree.is_some(),
            "the init update carries the tree"
        );
        assert_eq!(
            sink.drain_updates(),
            Vec::new(),
            "the buffer is empty after the drain"
        );

        // One incremental frame: only the dirty id re-emits, and the drain
        // hands over exactly that update.
        let mut delta = A11yDelta::empty();
        delta.updated.insert(222);
        sink.apply(snap, &delta);
        let drained = sink.drain_updates();
        assert_eq!(drained.len(), 1, "one frame, one update");
        assert!(
            drained[0].tree.is_none(),
            "incremental deltas never rebuild the tree"
        );
        assert_eq!(
            drained[0].nodes.len(),
            1,
            "dirty nodes only — the p99 budget's whole point"
        );
        assert_eq!(drained[0].nodes[0].0, accesskit_node_id(222));
    }

    #[test]
    fn full_tree_update_mirrors_the_resolved_tree_without_mutating_the_sink() {
        let mut sink = AccessKitSink::new();
        // The AT may connect before the first tick: no resolved tree, no
        // update to push.
        assert!(
            sink.full_tree_update().is_none(),
            "no resolved tree yet — the platform defers to the first frame"
        );

        let snap = std::sync::Arc::new(snapshot());
        sink.apply(std::sync::Arc::clone(&snap), &A11yDelta::empty());

        let before = sink.update_count();
        let update = sink.full_tree_update().expect("the tree is resolved");
        assert_eq!(update.nodes.len(), 3, "every resolved node rides along");
        assert_eq!(
            update.tree.as_ref().map(|t| t.root),
            Some(accesskit_node_id(111))
        );
        assert_eq!(
            update.focus,
            NodeId(0),
            "no focus in the delta — the full tree reports none"
        );
        assert_eq!(
            sink.update_count(),
            before,
            "the full-tree build is pure: no phantom update lands in the ledger"
        );
        assert!(
            find_in_update(&update, 333).is_some_and(|n| n.value() == Some("hello")),
            "the full vocabulary rides along"
        );
    }

    #[test]
    fn the_updates_ring_caps_stored_updates() {
        let mut sink = AccessKitSink::new();
        let snap = std::sync::Arc::new(snapshot());
        for _ in 0..(UPDATE_RING_CAP + 6) {
            sink.apply(std::sync::Arc::clone(&snap), &A11yDelta::empty());
        }
        let drained = sink.drain_updates();
        assert!(
            drained.len() <= UPDATE_RING_CAP,
            "{} updates stored, ring cap is {} — unbounded growth was the pre-M9 leak",
            drained.len(),
            UPDATE_RING_CAP
        );
        assert_eq!(sink.drain_updates(), Vec::new());
    }
}
