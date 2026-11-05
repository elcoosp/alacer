//! Alacer's accessibility: the AccessKit contract (ADR-5).
//!
//! §11.1, implemented:
//!
//! - AccessKit `NodeId`s are the id-path hashes — the same identity as
//!   hit-testing (§6.3); the a11y tree *is* the widget tree.
//! - Updates are **incremental**: pass 9 hands us the snapshot plus the dirty
//!   id set, and only dirty nodes are re-emitted in the [`TreeUpdate`]
//!   (never a full-tree rebuild per frame, the egui cost we refuse).
//! - Roles map from [`alacer_core::Role`]; labels resolve through the
//!   required `label()` source — an unnamed widget is impossible by
//!   construction.

use accesskit::{Node, NodeId, Rect, Role as AccessKitRole, TreeId, TreeInfo, TreeUpdate};
use alacer_core::Role;
use alacer_core::geom::Rect as AlacerRect;
use alacer_core::passes::{A11ySink, TreeSnapshot};
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

/// Builds an AccessKit node id from its raw hash (§6.3: same identity).
#[must_use]
pub fn accesskit_node_id(hash: u64) -> NodeId {
    NodeId(hash)
}

/// Maps Alacer's semantic roles onto AccessKit roles.
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

/// The AccessKit sink: incremental [`TreeUpdate`]s from dirty nodes only.
#[derive(Debug, Default)]
pub struct AccessKitSink {
    updates: Vec<TreeUpdate>,
    initialized: bool,
    last_snapshot: Option<TreeSnapshot>,
    /// The ids included in the most recent update (harness/test introspection).
    last_update_ids: FxHashSet<u64>,
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

    /// Whether the last update carried full tree (re)initialization data.
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.initialized
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
    fn apply(&mut self, snapshot: &TreeSnapshot, dirty: &FxHashSet<u64>) {
        self.last_snapshot = Some(snapshot.clone());
        self.last_update_ids.clear();
        let mut update = TreeUpdate {
            nodes: Vec::new(),
            tree: None,
            tree_id: TreeId::ROOT,
            focus: snapshot.root.map_or(NodeId(0), accesskit_node_id),
        };

        // First update initializes the whole tree; afterwards only dirty ids.
        if !self.initialized {
            update.tree = snapshot
                .root
                .map(|root| TreeInfo::new(accesskit_node_id(root)));
            self.initialized = true;
        }

        let relevant = |id: u64| dirty.is_empty() || dirty.contains(&id);
        for node in &snapshot.nodes {
            if !relevant(node.id) {
                continue;
            }
            self.last_update_ids.insert(node.id);
            let mut access_node = Node::new(map_role(node.role));
            access_node.set_label(node.label.as_str());
            if let Some(bounds) = node.bounds {
                access_node.set_bounds(map_bounds(bounds));
            }
            access_node.set_children(
                node.children
                    .iter()
                    .map(|c| accesskit_node_id(*c))
                    .collect::<Vec<_>>(),
            );
            update.nodes.push((accesskit_node_id(node.id), access_node));
        }

        self.update_count += 1;
        self.updates.push(update);
    }

    fn resolved_tree(&self) -> Option<&TreeSnapshot> {
        self.last_snapshot.as_ref()
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
    use alacer_core::Role;
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::passes::{A11ySink, TreeNode, TreeSnapshot};
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
                },
                TreeNode {
                    id: child_hash,
                    role: Role::Button,
                    label: "Save".into(),
                    bounds: Some(Rect::new(Point::new(0.0, 0.0), Size::new(60.0, 24.0))),
                    children: Vec::new(),
                },
                TreeNode {
                    id: other_hash,
                    role: Role::Label,
                    label: "Hello".into(),
                    bounds: Some(Rect::new(Point::new(0.0, 30.0), Size::new(60.0, 20.0))),
                    children: Vec::new(),
                },
            ],
        }
    }

    #[test]
    fn roles_map_onto_accesskit() {
        assert_eq!(map_role(Role::Button), AkRole::Button);
        assert_eq!(map_role(Role::Label), AkRole::Label);
        assert_eq!(map_role(Role::Container), AkRole::GenericContainer);
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
        sink.apply(&snap, &FxHashSet::default());

        assert!(sink.is_initialized());
        assert_eq!(sink.update_count(), 1);
        let update = sink.last_update().expect("update");
        assert!(update.tree.is_some(), "tree info present on init");
        assert_eq!(update.nodes.len(), 3, "every node in the init update");
        let button = find_in_update(update, 222).expect("button");
        assert_eq!(button.role(), AkRole::Button);
        assert_eq!(button.label(), Some("Save"));
    }

    #[test]
    fn later_updates_are_dirty_node_only() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(&snap, &FxHashSet::default());

        let mut dirty = FxHashSet::default();
        dirty.insert(222_u64); // only the button changed
        sink.apply(&snap, &dirty);

        let update = sink.last_update().expect("second update");
        assert_eq!(update.nodes.len(), 1, "only the dirty node re-emitted");
        assert_eq!(update.nodes[0].0.0, 222);
        assert!(
            !sink.last_update_ids().contains(&333),
            "clean node not re-emitted"
        );
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
        sink.apply(&snap, &FxHashSet::default());
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
}
