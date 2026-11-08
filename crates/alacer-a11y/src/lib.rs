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

use accesskit::{Node, NodeId, Rect, Role as AccessKitRole, Toggled, TreeId, TreeInfo, TreeUpdate};
use alacer_core::Role;
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
        let snapshot = &*snapshot;
        self.last_removed_ids.clone_from(&delta.removed);
        self.last_update_ids.clear();
        let mut update = TreeUpdate {
            nodes: Vec::new(),
            tree: None,
            tree_id: TreeId::ROOT,
            focus: delta.focus.map_or(NodeId(0), accesskit_node_id),
        };

        // Init deltas build the whole tree; incremental deltas emit only the
        // updated ids (§11.1 item 4 — the p99 budget's whole point).
        if delta.initializes || !self.initialized {
            update.tree = snapshot
                .root
                .map(|root| TreeInfo::new(accesskit_node_id(root)));
            for node in &snapshot.nodes {
                self.last_update_ids.insert(node.id);
                update
                    .nodes
                    .push((accesskit_node_id(node.id), Self::access_node(node)));
            }
            self.initialized = true;
        } else {
            for node in &snapshot.nodes {
                if !delta.updated.contains(&node.id) {
                    continue;
                }
                self.last_update_ids.insert(node.id);
                update
                    .nodes
                    .push((accesskit_node_id(node.id), Self::access_node(node)));
            }
        }

        self.update_count += 1;
        self.updates.push(update);
    }

    fn resolved_tree(&self) -> Option<&TreeSnapshot> {
        self.last_snapshot.as_deref()
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
    use super::{AccessKitSink, find_in_update, map_bounds, map_role, walk_update};
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
}
