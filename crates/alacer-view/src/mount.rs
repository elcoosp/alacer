//! The view reconciler: diff view against view, mutate widgets in place
//! (§6.1 — the Xilem/Masonry bridge discipline).
//!
//! The [`ViewRoot`] owns the mounted shape (element ↔ arena node). A
//! [`ViewRoot::update`] diffs the previous [`ViewNode`] tree against the new
//! one and applies exactly the difference:
//!
//! - same element → registry patch (props in place), children reconciled;
//! - different element → subtree replace (the one re-creation path; the
//!   root's element type is fixed for the lifetime of the mount);
//! - children all keyed → the LIS pipeline (`reconcile` + `apply`, §6.2):
//!   LIS members stay, the rest move minimally, new keys mount, dropped keys
//!   unmount;
//! - otherwise positional → index pairing with appends and removals.
//!
//! Mask discipline: patches merge `LAYOUT | PAINT` on the patched node when a
//! visual prop changed; structural churn merges `STRUCTURE` through the
//! arena ops themselves. The keyed path debug-asserts the §6.2 contract —
//! `apply(old, ops) == new` — on every update.

use crate::registry::WidgetRegistry;
use crate::view::ViewNode;
use crate::{DiffOp, apply, reconcile};
use alacer_core::arena::NodeId;
use alacer_core::mask::Change;
use alacer_core::passes::PassPipeline;
use std::borrow::Cow;

/// One mounted element: the arena node it lives in, plus the mounted shape
/// of its children and the props it currently carries (the reconciler's
/// persistent state — prop snapshots drive the change detection).
#[derive(Debug, Clone)]
struct MountedView {
    node: NodeId,
    parent: NodeId,
    element: Cow<'static, str>,
    key: Option<u64>,
    props: Vec<(Cow<'static, str>, crate::view::PropValue)>,
    children: Vec<MountedView>,
}

impl MountedView {
    /// A hollow placeholder for a slot about to be overwritten by `diff`.
    fn hollow(node: NodeId) -> Self {
        Self {
            node,
            parent: node,
            element: Cow::Borrowed(""),
            key: None,
            props: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Whether any prop value differs between the snapshots (handlers
    /// compare by identity — a re-created closure is a change).
    fn props_changed(&self, new: &ViewNode) -> bool {
        self.props.len() != new.props.len()
            || self
                .props
                .iter()
                .zip(new.props.iter())
                .any(|(a, b)| a.0 != b.0 || a.1 != b.1)
    }

    /// Whether any *visual* (non-handler) prop value differs — the mask and
    /// accounting hint: handler churn repaints nothing.
    fn visual_props_changed(&self, new: &ViewNode) -> bool {
        self.props.len() != new.props.len()
            || self.props.iter().zip(new.props.iter()).any(|(a, b)| {
                (a.0 != b.0 || a.1 != b.1)
                    && !matches!(
                        a.1,
                        crate::view::PropValue::Handler(_)
                            | crate::view::PropValue::TextHandler(_)
                            | crate::view::PropValue::FlagHandler(_)
                    )
                    && !matches!(
                        b.1,
                        crate::view::PropValue::Handler(_)
                            | crate::view::PropValue::TextHandler(_)
                            | crate::view::PropValue::FlagHandler(_)
                    )
            })
    }
}

/// What one [`ViewRoot::update`] did — the reconciler's accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PatchReport {
    /// Elements created (subtree mounts).
    pub created: usize,
    /// Subtrees removed.
    pub removed: usize,
    /// Elements patched in place (a visual prop applied).
    pub patched: usize,
    /// Keyed children moved (LIS non-members).
    pub moved: usize,
}

/// A mounted view tree: the retained counterpart of one [`ViewNode`] tree.
pub struct ViewRoot {
    root_id: NodeId,
    mounted: MountedView,
    registry: WidgetRegistry,
}

impl ViewRoot {
    /// Mounts `view` as the pipeline's root element.
    ///
    /// # Panics
    ///
    /// Panics when `view` names an unregistered element (see
    /// [`WidgetRegistry::create`]).
    #[must_use]
    pub fn mount(view: &ViewNode, pipeline: &mut PassPipeline, registry: WidgetRegistry) -> Self {
        let root_id = pipeline.mount_root(registry.create(view));
        let children = Self::mount_children(view, root_id, pipeline, &registry);
        Self {
            root_id,
            mounted: MountedView {
                node: root_id,
                parent: root_id,
                element: view.element.clone(),
                key: view.key,
                props: view.props.clone(),
                children,
            },
            registry,
        }
    }

    /// The mounted root's arena node.
    #[must_use]
    pub const fn root_id(&self) -> NodeId {
        self.root_id
    }

    /// Diffs `new` against the previously mounted tree, applying the
    /// difference in place.
    ///
    /// # Panics
    ///
    /// Panics when `new` names an unregistered element, when a keyed sibling
    /// list contains duplicate keys (the keyed contract, §6.2), or when the
    /// root's element type changes (mount a new [`ViewRoot`] instead).
    pub fn update(&mut self, new: &ViewNode, pipeline: &mut PassPipeline) -> PatchReport {
        let mut report = PatchReport::default();
        let old = std::mem::replace(&mut self.mounted, MountedView::hollow(self.root_id));
        self.mounted = self.diff(old, new, pipeline, &mut report);
        report
    }

    /// Mounts `view`'s children (recursively) under `parent`.
    fn mount_children(
        view: &ViewNode,
        parent: NodeId,
        pipeline: &mut PassPipeline,
        registry: &WidgetRegistry,
    ) -> Vec<MountedView> {
        view.children
            .iter()
            .map(|child| Self::mount_one(child, parent, pipeline, registry))
            .collect()
    }

    /// Mounts one child element and its subtree.
    fn mount_one(
        child: &ViewNode,
        parent: NodeId,
        pipeline: &mut PassPipeline,
        registry: &WidgetRegistry,
    ) -> MountedView {
        let node = pipeline.add_child(parent, registry.create(child));
        let children = Self::mount_children(child, node, pipeline, registry);
        MountedView {
            node,
            parent,
            element: child.element.clone(),
            key: child.key,
            props: child.props.clone(),
            children,
        }
    }

    /// The core diff at one position: `old` is the mounted shape, `new` the
    /// declared view. Returns the new mounted shape for the position.
    fn diff(
        &mut self,
        old: MountedView,
        new: &ViewNode,
        pipeline: &mut PassPipeline,
        report: &mut PatchReport,
    ) -> MountedView {
        if old.element != new.element {
            return self.replace(&old, new, pipeline, report);
        }
        self.patch_and_reconcile(old, new, pipeline, report)
    }

    /// Subtree replace: the one path that re-creates widgets (§6.1).
    fn replace(
        &mut self,
        old: &MountedView,
        new: &ViewNode,
        pipeline: &mut PassPipeline,
        report: &mut PatchReport,
    ) -> MountedView {
        assert!(
            old.parent != old.node,
            "the root element's type is fixed for the lifetime of a ViewRoot \
             (mounted {:?}, requested {:?}) — mount a new ViewRoot instead",
            old.element,
            new.element
        );
        pipeline.arena_mut().remove_subtree(old.node);
        report.removed += Self::count_of(old);
        let mounted = Self::mount_one(new, old.parent, pipeline, &self.registry);
        report.created += Self::count(new);
        mounted
    }

    /// Same element: patch props in place when they changed, then reconcile
    /// children.
    fn patch_and_reconcile(
        &mut self,
        old: MountedView,
        new: &ViewNode,
        pipeline: &mut PassPipeline,
        report: &mut PatchReport,
    ) -> MountedView {
        let visual_changed = old.visual_props_changed(new);
        if old.props_changed(new) {
            // Apply the full prop set (idempotent setters) — the patcher
            // re-hangs handlers and rewrites visuals in one call.
            let mut guard = pipeline.arena_mut().mutate(old.node);
            self.registry.patch(guard.widget_mut(), new);
        }
        if visual_changed {
            pipeline
                .arena_mut()
                .merge_mask(old.node, Change::LAYOUT.union(Change::PAINT));
            report.patched += 1;
        }
        let children = self.diff_children(old.children, &new.children, old.node, pipeline, report);
        MountedView {
            node: old.node,
            parent: old.parent,
            element: old.element,
            key: new.key,
            props: new.props.clone(),
            children,
        }
    }

    /// The children diff: keyed when both sides are fully keyed (§6.2's LIS
    /// pipeline), positional otherwise.
    fn diff_children(
        &mut self,
        old: Vec<MountedView>,
        new: &[ViewNode],
        parent: NodeId,
        pipeline: &mut PassPipeline,
        report: &mut PatchReport,
    ) -> Vec<MountedView> {
        let both_keyed = (!old.is_empty() || !new.is_empty())
            && old.iter().all(|m| m.key.is_some())
            && new.iter().all(|child| child.key.is_some());
        if both_keyed {
            self.diff_keyed(old, new, parent, pipeline, report)
        } else {
            self.diff_positional(old, new, parent, pipeline, report)
        }
    }

    /// The keyed path: peel → key map → LIS. Removals unmount; survivors
    /// patch in place; the final order is `apply(old, ops) == new` — the
    /// §6.2 contract, debug-asserted on every update.
    fn diff_keyed(
        &mut self,
        old: Vec<MountedView>,
        new: &[ViewNode],
        parent: NodeId,
        pipeline: &mut PassPipeline,
        report: &mut PatchReport,
    ) -> Vec<MountedView> {
        let old_keys: Vec<u64> = old.iter().map(|m| m.key.unwrap_or_default()).collect();
        let new_keys: Vec<u64> = new
            .iter()
            .map(|child| child.key.unwrap_or_default())
            .collect();
        let ops = reconcile(&old_keys, &new_keys).ops;

        // Unmount the dropped keys.
        let dropped_keys: Vec<u64> = ops
            .iter()
            .filter_map(|op| match op {
                DiffOp::Remove { key } => Some(*key),
                _ => None,
            })
            .collect();
        let mut survivors: Vec<MountedView> = old
            .into_iter()
            .filter(|mounted| {
                let keep = !dropped_keys.contains(&mounted.key.unwrap_or_default());
                if !keep {
                    pipeline.arena_mut().remove_subtree(mounted.node);
                    report.removed += Self::count_of(mounted);
                }
                keep
            })
            .collect();
        report.moved = ops
            .iter()
            .filter(|op| matches!(op, DiffOp::Move { .. }))
            .count();

        // Patch the survivors against their new views, by key.
        let view_by_key: std::collections::HashMap<u64, &ViewNode> = new
            .iter()
            .map(|child| (child.key.unwrap_or_default(), child))
            .collect();
        for mounted in &mut survivors {
            let key = mounted.key.unwrap_or_default();
            if let Some(child_view) = view_by_key.get(&key) {
                let taken = std::mem::replace(mounted, MountedView::hollow(mounted.node));
                *mounted = self.diff(taken, child_view, pipeline, report);
            }
        }

        // Final order: apply the ops to the old keys (the contract's
        // reference semantics), then walk `new` in order to place survivors
        // and mount inserts exactly where they belong.
        debug_assert_eq!(
            apply(&old_keys, &ops),
            new_keys,
            "the LIS pipeline's ops must transform old into new (§6.2)"
        );
        let mut by_key: std::collections::HashMap<u64, MountedView> = survivors
            .into_iter()
            .map(|mounted| (mounted.key.unwrap_or_default(), mounted))
            .collect();
        let mut ordered: Vec<MountedView> = Vec::with_capacity(new.len());
        for child in new {
            let key = child.key.unwrap_or_default();
            if let Some(mounted) = by_key.remove(&key) {
                ordered.push(mounted);
            } else {
                report.created += Self::count(child);
                ordered.push(Self::mount_one(child, parent, pipeline, &self.registry));
            }
        }

        // Persist the order in the arena (LIS members stay; the rest move).
        let order: Vec<NodeId> = ordered.iter().map(|mounted| mounted.node).collect();
        pipeline.arena_mut().reorder_children(parent, &order);
        ordered
    }

    /// The positional path: index pairing with appends and removals.
    fn diff_positional(
        &mut self,
        old: Vec<MountedView>,
        new: &[ViewNode],
        parent: NodeId,
        pipeline: &mut PassPipeline,
        report: &mut PatchReport,
    ) -> Vec<MountedView> {
        let mut mounted = old;
        // Extra old children unmount (they are not in `new`).
        while mounted.len() > new.len() {
            let dropped = mounted.pop().expect("len checked");
            pipeline.arena_mut().remove_subtree(dropped.node);
            report.removed += Self::count_of(&dropped);
        }
        // Pair by index: patch the survivors, mount the rest. A swap (element
        // replace) re-adds at the end of the arena, so the final arena order
        // is re-synced to the logical order — the same discipline the keyed
        // path applies.
        let mut out = Vec::with_capacity(new.len());
        for child in new {
            if out.len() < mounted.len() {
                let index = out.len();
                let node = mounted[index].node;
                let taken = std::mem::replace(&mut mounted[index], MountedView::hollow(node));
                out.push(self.diff(taken, child, pipeline, report));
            } else {
                report.created += Self::count(child);
                out.push(Self::mount_one(child, parent, pipeline, &self.registry));
            }
        }
        let order: Vec<NodeId> = out.iter().map(|mounted| mounted.node).collect();
        pipeline.arena_mut().reorder_children(parent, &order);
        out
    }

    /// Nodes in a view subtree, including the root (create accounting).
    fn count(view: &ViewNode) -> usize {
        1 + view.children.iter().map(Self::count).sum::<usize>()
    }

    /// Nodes in a mounted subtree (remove accounting).
    fn count_of(mounted: &MountedView) -> usize {
        1 + mounted.children.iter().map(Self::count_of).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::ViewRoot;
    use crate::registry::WidgetRegistry;
    use crate::view::ViewNode;
    use alacer_core::arena::NodeId;
    use alacer_core::passes::{LayoutEngine, LayoutError, PassPipeline};
    use alacer_core::widget::{LabelSource, Role, Widget};
    use rustc_hash::FxHashMap;

    /// A text-carrying probe element: label resolves to its text, patching
    /// rewrites it in place. Enough to observe identity and content.
    struct BoxProbe {
        text: String,
    }

    impl Widget for BoxProbe {
        fn role(&self) -> Role {
            Role::Label
        }

        fn label(&self) -> LabelSource {
            LabelSource::text(self.text.clone())
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    fn box_create(props: &crate::ElementProps<'_>) -> BoxProbe {
        BoxProbe {
            text: props.text("text").unwrap_or_default().to_string(),
        }
    }

    fn box_patch(probe: &mut BoxProbe, props: &crate::ElementProps<'_>) -> bool {
        match props.text("text") {
            Some(text) => {
                probe.text = text.to_string();
                true
            }
            None => false,
        }
    }

    /// The test registry: two probe elements (the second proves element swaps).
    fn probe_registry() -> WidgetRegistry {
        let mut registry = WidgetRegistry::new();
        registry.register("box", box_create, box_patch);
        registry.register("cell", box_create, box_patch);
        registry
    }

    /// A no-op layout engine (mount tests drive the arena, never a tick).
    struct NullLayout;

    impl LayoutEngine for NullLayout {
        fn sync_and_layout(
            &mut self,
            _arena: &mut alacer_core::Arena,
            _root: NodeId,
            _dirty: &FxHashMap<NodeId, alacer_core::mask::Change>,
            _viewport: alacer_core::geom::Size,
        ) -> Result<(), LayoutError> {
            Ok(())
        }

        fn measure_calls(&self) -> u64 {
            0
        }

        fn reset_measure_calls(&mut self) {}
    }

    fn pipeline() -> PassPipeline {
        PassPipeline::new(
            alacer_core::Flux::new(),
            Box::new(NullLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        )
    }

    #[derive(Default)]
    struct NullJournal;

    impl alacer_core::passes::SceneJournal for NullJournal {
        fn commit_layer(&mut self, _node: NodeId, _primitives: Vec<alacer_core::scene::Primitive>) {
        }

        fn drop_layer(&mut self, _node: NodeId) {}

        fn layer(&self, _node: NodeId) -> Option<Vec<alacer_core::scene::Primitive>> {
            None
        }

        fn clear(&mut self) {}

        fn snapshot(&self, _order: &[NodeId]) -> alacer_core::scene::SceneSnapshot {
            alacer_core::scene::SceneSnapshot {
                primitives: Vec::new(),
            }
        }

        fn retain(&mut self, _order: &[NodeId]) {}
    }

    #[derive(Default)]
    struct NullA11y;

    impl alacer_core::passes::A11ySink for NullA11y {
        fn apply(
            &mut self,
            _snapshot: std::sync::Arc<alacer_core::passes::TreeSnapshot>,
            _delta: &alacer_core::passes::A11yDelta,
        ) {
        }
    }

    /// The label tree of an arena subtree — the churn/fresh equivalence
    /// discriminator.
    fn label_tree(arena: &alacer_core::Arena, node: NodeId) -> String {
        let label = arena.widget(node).label().resolve();
        let children: Vec<String> = arena
            .children(node)
            .iter()
            .map(|child| label_tree(arena, *child))
            .collect();
        if children.is_empty() {
            label
        } else {
            format!("{label}[{}]", children.join(","))
        }
    }

    #[test]
    fn mount_builds_the_declared_shape() {
        let view = ViewNode::element("box")
            .prop("text", "root")
            .child(ViewNode::element("box").prop("text", "a"))
            .child(ViewNode::element("box").prop("text", "b"));
        let mut pipeline = pipeline();
        let root = ViewRoot::mount(&view, &mut pipeline, probe_registry());

        assert_eq!(pipeline.arena().len(), 3);
        assert_eq!(label_tree(pipeline.arena(), root.root_id()), "root[a,b]");
    }

    #[test]
    fn update_patches_in_place_and_preserves_identity() {
        let view = ViewNode::element("box")
            .prop("text", "before")
            .child(ViewNode::element("box").prop("text", "child"));
        let mut pipeline = pipeline();
        let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());
        let child_before = pipeline.arena().children(root.root_id())[0];

        let updated = ViewNode::element("box")
            .prop("text", "after")
            .child(ViewNode::element("box").prop("text", "child"));
        let report = root.update(&updated, &mut pipeline);

        assert_eq!(report.patched, 1, "only the root's text changed");
        assert_eq!(report.created, 0);
        assert_eq!(report.removed, 0);
        assert_eq!(root.root_id(), pipeline.arena().root().expect("root"));
        assert_eq!(
            pipeline.arena().children(root.root_id()),
            &[child_before],
            "the child keeps its identity"
        );
        assert_eq!(label_tree(pipeline.arena(), root.root_id()), "after[child]");
    }

    #[test]
    fn keyed_reorder_minimizes_moves_and_preserves_identity() {
        let view = ViewNode::element("box")
            .prop("text", "root")
            .children((0_u64..4).map(|i| {
                ViewNode::element("box")
                    .key(i)
                    .prop("text", format!("k{i}"))
            }));
        let mut pipeline = pipeline();
        let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());
        let nodes_before: Vec<NodeId> = pipeline.arena().children(root.root_id()).to_vec();

        // Rotate by one: LIS keeps three of four stable — one move.
        let rotated = ViewNode::element("box")
            .prop("text", "root")
            .children([1_u64, 2, 3, 0].map(|i| {
                ViewNode::element("box")
                    .key(i)
                    .prop("text", format!("k{i}"))
            }));
        let report = root.update(&rotated, &mut pipeline);

        assert_eq!(report.moved, 1, "LIS keeps three of four stable");
        assert_eq!(report.created, 0);
        assert_eq!(report.removed, 0);
        assert_eq!(
            label_tree(pipeline.arena(), root.root_id()),
            "root[k1,k2,k3,k0]"
        );
        let nodes_after: Vec<NodeId> = pipeline.arena().children(root.root_id()).to_vec();
        for node in &nodes_before {
            assert!(
                nodes_after.contains(node),
                "every widget survives a rotation: {node}"
            );
        }
    }

    #[test]
    fn keyed_insert_and_remove_touch_exactly_the_diff() {
        let view = ViewNode::element("box")
            .prop("text", "root")
            .children([1_u64, 2, 3].map(|i| {
                ViewNode::element("box")
                    .key(i)
                    .prop("text", format!("k{i}"))
            }));
        let mut pipeline = pipeline();
        let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());
        let nodes_before: Vec<NodeId> = pipeline.arena().children(root.root_id()).to_vec();

        let updated = ViewNode::element("box")
            .prop("text", "root")
            .children([1_u64, 4, 2].map(|i| {
                ViewNode::element("box")
                    .key(i)
                    .prop("text", format!("k{i}"))
            }));
        let report = root.update(&updated, &mut pipeline);

        assert_eq!(report.created, 1, "k4 mounts");
        assert_eq!(report.removed, 1, "k3 unmounts");
        assert_eq!(report.patched, 0, "survivors' text did not change");
        let nodes_after: Vec<NodeId> = pipeline.arena().children(root.root_id()).to_vec();
        assert_eq!(nodes_after.len(), 3);
        assert!(
            nodes_before.contains(&nodes_after[0]) && nodes_before.contains(&nodes_after[2]),
            "k1 and k2 keep their widgets"
        );
        assert_eq!(
            label_tree(pipeline.arena(), root.root_id()),
            "root[k1,k4,k2]"
        );
    }

    #[test]
    fn element_swap_replaces_the_subtree() {
        let view = ViewNode::element("box")
            .prop("text", "root")
            .child(ViewNode::element("box").prop("text", "old"));
        let mut pipeline = pipeline();
        let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());
        let old_child = pipeline.arena().children(root.root_id())[0];

        let updated = ViewNode::element("box")
            .prop("text", "root")
            .child(ViewNode::element("cell").prop("text", "new"));
        let report = root.update(&updated, &mut pipeline);

        assert_eq!(report.removed, 1);
        assert_eq!(report.created, 1);
        let new_child = pipeline.arena().children(root.root_id())[0];
        assert_ne!(new_child, old_child, "a different element re-creates");
        assert_eq!(label_tree(pipeline.arena(), root.root_id()), "root[new]");
    }

    #[test]
    fn positional_children_append_and_trim() {
        let view = ViewNode::element("box").prop("text", "root").children([
            ViewNode::element("box").prop("text", "a"),
            ViewNode::element("box").prop("text", "b"),
        ]);
        let mut pipeline = pipeline();
        let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());

        let appended = ViewNode::element("box").prop("text", "root").children([
            ViewNode::element("box").prop("text", "a"),
            ViewNode::element("box").prop("text", "b"),
            ViewNode::element("box").prop("text", "c"),
        ]);
        let report = root.update(&appended, &mut pipeline);
        assert_eq!(report.created, 1);
        assert_eq!(label_tree(pipeline.arena(), root.root_id()), "root[a,b,c]");

        let trimmed = ViewNode::element("box").prop("text", "root");
        let report = root.update(&trimmed, &mut pipeline);
        assert_eq!(report.removed, 3);
        assert_eq!(label_tree(pipeline.arena(), root.root_id()), "root");
    }

    #[test]
    fn handler_only_updates_patch_silently() {
        let view = ViewNode::element("box")
            .prop("text", "stable")
            .prop("on_click", || {});
        let mut pipeline = pipeline();
        let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());

        let updated = ViewNode::element("box")
            .prop("text", "stable")
            .prop("on_click", || {});
        let report = root.update(&updated, &mut pipeline);
        assert_eq!(report.patched, 0, "handlers are not visual");
    }

    #[test]
    #[should_panic(expected = "the root element's type is fixed")]
    fn the_root_element_type_is_fixed_for_the_mount() {
        let view = ViewNode::element("box").prop("text", "root");
        let mut pipeline = pipeline();
        let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());
        let swapped = ViewNode::element("cell").prop("text", "root");
        let _ = root.update(&swapped, &mut pipeline);
    }

    /// The §6.4 equivalence gate, mirrored for the reconciler: *incremental
    /// updates ≡ fresh mount* on random view-tree churn. The churned tree's
    /// label shape must equal the fresh mount of the same final view.
    #[test]
    fn incremental_updates_equal_fresh_mounts_on_random_churn() {
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

        // Random view tree: `node budget` boxes, depth-bounded; siblings are
        // either all keyed or all positional.
        fn random_tree(rng: &mut Lcg, budget: usize, depth: usize, prefix: &str) -> ViewNode {
            let mut node = ViewNode::element(if rng.next().is_multiple_of(4) {
                "cell"
            } else {
                "box"
            })
            .prop("text", prefix.to_string());
            if depth == 0 || budget == 0 {
                return node;
            }
            let count = (rng.next() % 4) as usize;
            let keyed = rng.next().is_multiple_of(2);
            let key_base = (prefix.len() as u64) * 1_000;
            for i in 0..count {
                let child_budget = budget.saturating_sub(1) / count.max(1);
                let child = random_tree(rng, child_budget, depth - 1, &format!("{prefix}.{i}"));
                node = if keyed {
                    let index = alacer_core::num::usize_to_u32(i);
                    node.child(child.key(key_base + u64::from(index)))
                } else {
                    node.child(child)
                };
            }
            node
        }

        // Churn one tree into another: rewrite texts, add/remove/reorder
        // children, flip keyed-ness — the shapes a real re-render produces.
        fn churn(rng: &mut Lcg, node: ViewNode, depth: usize, prefix: &str) -> ViewNode {
            let mut node = node.prop("text", format!("{prefix}~{}", rng.next() % 100));
            node.clear_children();
            if depth == 0 {
                return node;
            }
            let count = (rng.next() % 4) as usize;
            let keyed = rng.next().is_multiple_of(2);
            // A per-list random base keeps keys unique within siblings.
            let key_base = (rng.next() % 1_000) * 100;
            for i in 0..count {
                let child = ViewNode::element(if rng.next().is_multiple_of(4) {
                    "cell"
                } else {
                    "box"
                })
                .prop("text", format!("{prefix}.{i}~{}", rng.next() % 100));
                node = if keyed {
                    let index = alacer_core::num::usize_to_u32(i);
                    node.child(child.key(key_base + u64::from(index)))
                } else {
                    node.child(child)
                };
            }
            node
        }

        let mut rng = Lcg(0xBEEF);
        for case in 0..48_u64 {
            let initial = random_tree(&mut rng, 24, 3, "n");
            let final_view = churn(&mut rng, initial.clone(), 3, "n");

            let mut churned_pipeline = pipeline();
            let mut root = ViewRoot::mount(&initial, &mut churned_pipeline, probe_registry());
            let report = root.update(&final_view, &mut churned_pipeline);
            let _ = report;

            // Fresh mount of the same final view in a second pipeline.
            let mut fresh_pipeline = pipeline();
            let fresh_root = ViewRoot::mount(&final_view, &mut fresh_pipeline, probe_registry());

            let churned_shape = label_tree(churned_pipeline.arena(), root.root_id());
            let fresh_shape = label_tree(fresh_pipeline.arena(), fresh_root.root_id());
            assert_eq!(
                churned_shape, fresh_shape,
                "case {case}: incremental == fresh rebuild\ninitial: {initial:?}\nfinal:   {final_view:?}"
            );
        }
    }
}
