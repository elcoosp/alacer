//! The pass contract (architecture §7.2): each cross-cutting concern is one
//! explicit, ordered, independently traceable stage — never scattered through
//! callbacks.
//!
//! M0 runs the pipeline over the mask currency:
//!
//! | # | Pass | Responsibility |
//! |---|------|----------------|
//! | 1 | `event` | hit-test + bubble routing (pointer only this milestone) |
//! | 4 | `mutate` | drain queued `WidgetMut` closures; flags merge on drop |
//! | 5–6 | `style→layout` | the injected [`LayoutEngine`] recomputes geometry from dirty nodes |
//! | 7 | `compose` | translation-only scroll updates — no re-measure (§8.2) |
//! | 8 | `paint` | re-emit dirty layers into the [`SceneJournal`]; clean layers replay |
//! | 9 | `accessibility` | build the a11y update from dirty nodes only (§11.1) |
//!
//! Passes 2–3 (focusable/focus) and 10 (actions) land with M1/M3; their slots
//! in the ordering are reserved and documented.

use crate::arena::{Arena, NodeId};
use crate::events::PointerEvent;
use crate::flux::Flux;
use crate::geom::{Point, Rect, Size, Vector};
use crate::mask::Change;
use crate::scene::{PaintCtx, Primitive, SceneSnapshot};
use crate::widget::{EventResponse, Role, Widget};
use rustc_hash::{FxHashMap, FxHashSet};

/// Errors surfaced by the layout engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutError {
    /// Human-readable failure description, widget ids included.
    pub message: String,
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "layout error: {}", self.message)
    }
}

impl std::error::Error for LayoutError {}

/// The layout stage, injected so `alacer-core` stays independent of Taffy.
pub trait LayoutEngine {
    /// Syncs structural changes into the persistent tree, styles dirty nodes,
    /// and recomputes geometry from the closest common ancestor of the dirty
    /// set — the persistent tree is never cleared (§8.1).
    ///
    /// Writes absolute bounds back into the arena.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutError`] when the persistent tree cannot absorb a
    /// structural or style change.
    fn sync_and_layout(
        &mut self,
        arena: &mut Arena,
        root: NodeId,
        dirty: &FxHashMap<NodeId, Change>,
        viewport: Size,
    ) -> Result<(), LayoutError>;

    /// Number of leaf measure invocations since the last
    /// [`Self::reset_measure_calls`] — the translation-only-update probe.
    #[must_use]
    fn measure_calls(&self) -> u64;

    /// Resets the measure-call counter (harness/bench convenience).
    fn reset_measure_calls(&mut self);
}

/// One node of the accessibility snapshot: role, name, bounds, children — the
/// neutral vocabulary `alacer-a11y` converts into AccessKit nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode {
    /// AccessKit-stable node id: the id-path hash (§6.3).
    pub id: u64,
    /// Semantic role.
    pub role: Role,
    /// Resolved accessible name.
    pub label: String,
    /// Absolute bounds.
    pub bounds: Option<Rect>,
    /// Child ids in tree order.
    pub children: Vec<u64>,
}

/// The full accessibility snapshot of the widget tree, built by pass 9.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TreeSnapshot {
    /// Nodes in pre-order (the tree walk order).
    pub nodes: Vec<TreeNode>,
    /// Root node id, if a root is mounted.
    pub root: Option<u64>,
}

impl TreeSnapshot {
    /// Finds the first node with `role` whose name contains `name` — the
    /// a11y-driven query API (§11.5, `find(ROLE).named(...)` semantics).
    #[must_use]
    pub fn find(&self, role: Role, name: &str) -> Option<&TreeNode> {
        let needle = name.to_lowercase();
        self.nodes
            .iter()
            .find(|n| n.role == role && n.label.to_lowercase().contains(&needle))
    }
}

/// The accessibility stage, injected so `alacer-core` stays independent of
/// AccessKit.
pub trait A11ySink {
    /// Applies the snapshot incrementally: only `dirty` ids (id-path hashes)
    /// are re-emitted downstream (§11.1 item 4).
    fn apply(&mut self, snapshot: &TreeSnapshot, dirty: &FxHashSet<u64>);

    /// The sink's resolved tree, for harness queries.
    #[must_use]
    fn resolved_tree(&self) -> Option<&TreeSnapshot> {
        None
    }
}

/// The scene journal, injected so `alacer-core` owns the paint pass without
/// owning journal storage (§9.1).
pub trait SceneJournal {
    /// Replaces node `node`'s recorded layer with `primitives`.
    fn commit_layer(&mut self, node: NodeId, primitives: Vec<Primitive>);

    /// Removes node `node`'s layer (subtree removals).
    fn drop_layer(&mut self, node: NodeId);

    /// Materializes the ordered scene: layers concatenated in `order` (the
    /// arena's pre-order walk — the paint order).
    #[must_use]
    fn snapshot(&self, order: &[NodeId]) -> SceneSnapshot;

    /// Keeps only the layers of nodes in `order` (structural churn cleanup).
    fn retain(&mut self, order: &[NodeId]);

    /// Drops every layer (structural rebuilds).
    fn clear(&mut self);
}

/// Counters for the frame-level scheduling contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameCounters {
    /// Layout passes run.
    pub layout_runs: u64,
    /// Paint passes run.
    pub paint_runs: u64,
    /// Accessibility passes run.
    pub a11y_runs: u64,
    /// Pointer events routed.
    pub events_routed: u64,
}

/// One frame's outcome, returned by [`PassPipeline::tick`].
#[derive(Debug, Clone)]
pub struct FrameReport {
    /// `true` when the layout pass ran.
    pub layout_ran: bool,
    /// `true` when the paint pass ran.
    pub paint_ran: bool,
    /// `true` when the accessibility pass ran.
    pub a11y_ran: bool,
    /// The materialized scene (empty when nothing painted).
    pub scene: SceneSnapshot,
}

impl FrameReport {
    /// `true` when no pass had work — the idle contract (A7).
    #[must_use]
    pub fn is_idle(&self) -> bool {
        !self.layout_ran && !self.paint_ran && !self.a11y_ran
    }
}

/// The pass pipeline: owns the arena, the flux, and the injected stages.
pub struct PassPipeline {
    arena: Arena,
    flux: Flux,
    layout: Box<dyn LayoutEngine>,
    journal: Box<dyn SceneJournal>,
    a11y: Box<dyn A11ySink>,
    counters: FrameCounters,
    scroll_offsets: FxHashMap<NodeId, Vector>,
    last_compose_events: u64,
}

impl PassPipeline {
    /// Assembles the pipeline from its stages.
    #[must_use]
    pub fn new(
        flux: Flux,
        layout: Box<dyn LayoutEngine>,
        journal: Box<dyn SceneJournal>,
        a11y: Box<dyn A11ySink>,
    ) -> Self {
        Self {
            arena: Arena::new(),
            flux,
            layout,
            journal,
            a11y,
            counters: FrameCounters::default(),
            scroll_offsets: FxHashMap::default(),
            last_compose_events: 0,
        }
    }

    /// Mounts the root widget.
    #[must_use]
    pub fn mount_root(&mut self, widget: Box<dyn Widget>) -> NodeId {
        self.arena.set_root(widget)
    }

    /// Appends a child widget.
    #[must_use]
    pub fn add_child(&mut self, parent: NodeId, widget: Box<dyn Widget>) -> NodeId {
        self.arena.add_child(parent, widget)
    }

    /// Shared borrow of the arena.
    #[must_use]
    pub const fn arena(&self) -> &Arena {
        &self.arena
    }

    /// Mutable borrow of the arena (widget wiring between frames).
    #[must_use]
    pub const fn arena_mut(&mut self) -> &mut Arena {
        &mut self.arena
    }

    /// The flux (signal handles stay valid; bindings are wired through here).
    #[must_use]
    pub const fn flux(&self) -> &Flux {
        &self.flux
    }

    /// Mutable access to the flux for binding new signals.
    #[must_use]
    pub const fn flux_mut(&mut self) -> &mut Flux {
        &mut self.flux
    }

    /// Frame counters.
    #[must_use]
    pub const fn counters(&self) -> FrameCounters {
        self.counters
    }

    /// The a11y sink's resolved tree (harness queries).
    #[must_use]
    pub fn a11y_snapshot(&self) -> Option<&TreeSnapshot> {
        self.a11y.resolved_tree()
    }

    /// The layout engine's measure-call counter.
    #[must_use]
    pub fn measure_calls(&self) -> u64 {
        self.layout.measure_calls()
    }

    /// Resets the layout engine's measure-call counter.
    pub fn reset_measure_calls(&mut self) {
        self.layout.reset_measure_calls();
    }

    /// Pass 1 (`event`): hit-test the pointer position and bubble the event
    /// from the target to the root; the first `Handled` stops bubbling
    /// (§10.1: z-order-aware; capture phase lands with M1).
    pub fn dispatch_pointer(&mut self, event: &PointerEvent) -> EventResponse {
        self.counters.events_routed += 1;
        let Some(position) = event.position() else {
            return EventResponse::Ignored;
        };
        let mut chain = Vec::new();
        self.arena.for_each_pre_order(&mut |id, slot| {
            let contains = slot.bounds.is_some_and(|b| b.contains(position));
            let accepts = slot.widget().accepts_pointer();
            if contains && accepts {
                chain.push(id);
            }
        });
        // Reverse pre-order tail = deepest, topmost (last painted) hit.
        for node in chain.into_iter().rev() {
            let (response, requested) = {
                let mut guard = self.arena.mutate(node);
                let mut ctx = crate::events::EventCtx::new();
                let response = guard.widget_mut().on_pointer_event(&mut ctx, event);
                (response, ctx.take())
            };
            if !requested.is_empty() {
                self.arena.merge_mask(node, requested);
            }
            if response == EventResponse::Handled {
                return EventResponse::Handled;
            }
        }
        EventResponse::Ignored
    }

    /// Runs exactly one frame: mutate → flux drain → layout → compose → paint
    /// → a11y. When no mask is pending every stage is skipped — idle cost zero
    /// (A7).
    ///
    /// # Errors
    ///
    /// Propagates [`LayoutError`] from the layout stage.
    pub fn tick(&mut self, viewport: Size) -> Result<FrameReport, LayoutError> {
        // Pass 4 — mutate: drain queued widget mutations; flags merge on drop.
        self.arena.drain_mutations();

        // Flux drain: the mask bridge lands masks in the arena (A1).
        for (node, change) in self.flux.take_pending_masks() {
            self.arena.merge_mask(node, change);
        }

        // Drain the arena's masks into the frame's dirty map.
        let dirty: FxHashMap<NodeId, Change> =
            self.arena.take_masks(Change::ALL).into_iter().collect();
        let root = self.arena.root();
        let first_frame = root.is_some_and(|r| self.arena.bounds(r).is_none());

        // Passes 5–6 — style → layout (incremental; the persistent Taffy tree
        // is never cleared between frames, §8.1).
        let layout_ran = first_frame || dirty.values().any(|m| m.forces_layout());
        let mut translated = Vec::new();
        if layout_ran {
            let root = root.expect("root mounted for layout");
            self.layout
                .sync_and_layout(&mut self.arena, root, &dirty, viewport)?;
            self.counters.layout_runs += 1;
        }

        // Pass 7 — compose: translation-only scroll updates. Runs after layout
        // AND after any routed input: a wheel event moves a scroll offset
        // without touching layout — that is the whole point of translation-only
        // updates (§8.2 item 4).
        let events_since = self.counters.events_routed - self.last_compose_events;
        if layout_ran || events_since > 0 {
            translated = self.compose_scrolls();
            self.last_compose_events = self.counters.events_routed;
        }

        // Pass 8 — paint: re-emit dirty layers; clean layers replay (§9.1).
        let structure_changed = dirty.values().any(|m| m.contains(Change::STRUCTURE));
        let paint_set: FxHashSet<NodeId> = dirty
            .iter()
            .filter(|(_, m)| m.intersects(Change::PAINT.union(Change::LAYOUT)))
            .map(|(n, _)| *n)
            .collect();
        let paint_ran =
            first_frame || structure_changed || !paint_set.is_empty() || !translated.is_empty();
        if paint_ran {
            if structure_changed {
                self.journal.clear();
            }
            self.paint_dirty(structure_changed, &paint_set, &translated);
            self.counters.paint_runs += 1;
        }

        // Pass 9 — accessibility: incremental updates from dirty nodes only
        // (§11.1); layout changes move a11y bounds too.
        let a11y_set: FxHashSet<u64> = dirty
            .iter()
            .filter(|(_, m)| m.forces_a11y() || m.intersects(Change::LAYOUT))
            .map(|(n, _)| self.id_hash(*n))
            .chain(translated.iter().map(|n| self.id_hash(*n)))
            .collect();
        let a11y_ran = first_frame || !a11y_set.is_empty();
        if a11y_ran {
            let snapshot = self.build_tree_snapshot();
            self.a11y.apply(&snapshot, &a11y_set);
            self.counters.a11y_runs += 1;
        }

        let order = self.paint_order();
        let scene = self.journal.snapshot(&order);
        Ok(FrameReport {
            layout_ran,
            paint_ran,
            a11y_ran,
            scene,
        })
    }

    /// Pass 7 (`compose`): applies scroll-offset deltas to descendants without
    /// re-measuring — the translation-only workhorse (§8.2 item 4).
    ///
    /// Returns the descendants whose bounds moved (they need repaint + a11y
    /// bounds updates, never layout).
    fn compose_scrolls(&mut self) -> Vec<NodeId> {
        let mut containers = Vec::new();
        self.arena.for_each_pre_order(&mut |id, slot| {
            let offset = slot.widget().scroll_offset();
            if offset != Vector::zero() {
                containers.push((id, offset));
            }
        });

        let mut moved = Vec::new();
        for (container, offset) in containers {
            let last = self
                .scroll_offsets
                .get(&container)
                .copied()
                .unwrap_or(Vector::zero());
            let delta = last - offset;
            if delta == Vector::zero() {
                continue;
            }
            let mut descendants = Vec::new();
            self.collect_subtree(container, &mut descendants);
            for id in descendants {
                if let Some(bounds) = self.arena.bounds(id) {
                    self.arena.update_bounds(id, bounds.translate(delta));
                    moved.push(id);
                }
            }
            self.scroll_offsets.insert(container, offset);
        }
        moved
    }

    fn collect_subtree(&self, node: NodeId, out: &mut Vec<NodeId>) {
        out.push(node);
        for child in self.arena.children(node).to_vec() {
            self.collect_subtree(child, out);
        }
    }

    /// Pass 8's re-emission: repaint exactly the dirty subtrees; clean layers
    /// are left in the journal untouched (cached-range replay, §9.1).
    fn paint_dirty(
        &mut self,
        full_rebuild: bool,
        paint_set: &FxHashSet<NodeId>,
        translated: &[NodeId],
    ) {
        let translated: FxHashSet<NodeId> = translated.iter().copied().collect();
        let mut targets = Vec::new();
        self.arena.for_each_pre_order(&mut |id, _| {
            if full_rebuild || paint_set.contains(&id) || translated.contains(&id) {
                targets.push(id);
            }
        });
        for node in targets {
            let primitives = self.paint_node(node);
            self.journal.commit_layer(node, primitives);
        }
        let order = self.paint_order();
        self.journal.retain(&order);
    }

    fn paint_node(&mut self, node: NodeId) -> Vec<Primitive> {
        struct JournalVec<'a> {
            out: &'a mut Vec<Primitive>,
        }
        impl crate::scene::PaintSink for JournalVec<'_> {
            fn push(&mut self, primitive: Primitive) {
                self.out.push(primitive);
            }
        }
        let bounds = self.arena.bounds(node);
        let origin = bounds.map_or(Point::zero(), |b| b.origin);
        let size = bounds.map_or(crate::geom::Size::zero(), |b| b.size);
        let clip = self.ancestor_clip(node);
        let mut out = Vec::new();
        {
            let mut sink = JournalVec { out: &mut out };
            let mut ctx = PaintCtx::new(&mut sink, origin, size, clip);
            let widget = &self.arena.slot(node).widget;
            widget.paint(&mut ctx);
        }
        out
    }

    /// The most restrictive ancestor clip in effect at `node` (scroll
    /// viewports clip children; A8).
    fn ancestor_clip(&self, node: NodeId) -> Option<Rect> {
        let mut cursor = self.arena.parent(node);
        let mut clip: Option<Rect> = None;
        while let Some(id) = cursor {
            let slot = self.arena.slot(id);
            if slot.widget().style().clip_overflow
                && let Some(bounds) = slot.bounds
            {
                clip = Some(match clip {
                    Some(c) => c.intersection(&bounds).unwrap_or_default(),
                    None => bounds,
                });
            }
            cursor = slot.parent;
        }
        clip
    }

    /// Builds the a11y snapshot from the arena (pass 9 input).
    fn build_tree_snapshot(&self) -> TreeSnapshot {
        let mut snapshot = TreeSnapshot::default();
        self.arena.for_each_pre_order(&mut |id, slot| {
            let node = crate::passes::TreeNode {
                id: self.id_hash(id),
                role: slot.widget().role(),
                label: slot.widget().label().resolve(),
                bounds: slot.bounds,
                children: slot.children.iter().map(|c| self.id_hash(*c)).collect(),
            };
            if snapshot.root.is_none() {
                snapshot.root = Some(node.id);
            }
            snapshot.nodes.push(node);
        });
        snapshot
    }

    /// Pass 10 (`action`): routes an accessibility action to the widget whose
    /// id-path hash is `target` (§7.2 pass 10: side-channel, never masks).
    pub fn dispatch_a11y_action(&mut self, target: u64, action: crate::widget::A11yAction) -> bool {
        let Some(node) = self.node_for_hash(target) else {
            return false;
        };
        let mut guard = self.arena.mutate(node);
        guard.widget_mut().on_a11y_action(action)
    }

    /// Maps an id-path hash back to its arena node (walk = id-path rebuild).
    fn node_for_hash(&self, target: u64) -> Option<NodeId> {
        let mut found = None;
        self.arena.for_each_pre_order(&mut |id, _| {
            if found.is_none() && self.id_hash(id) == target {
                found = Some(id);
            }
        });
        found
    }

    /// AccessKit-stable node id: the id-path hash of the arena path (§6.3).
    fn id_hash(&self, node: NodeId) -> u64 {
        let mut chain = Vec::new();
        let mut cursor = Some(node);
        while let Some(id) = cursor {
            chain.push(id);
            cursor = self.arena.parent(id);
        }
        let mut path = crate::identity::IdPath::default();
        for id in chain.into_iter().rev() {
            path.push(id.raw());
        }
        path.hash()
    }

    fn paint_order(&self) -> Vec<NodeId> {
        let mut order = Vec::new();
        self.arena.for_each_pre_order(&mut |id, _| order.push(id));
        order
    }
}

#[cfg(test)]
mod tests {
    use super::{A11ySink, LayoutEngine, LayoutError, PassPipeline, SceneJournal, TreeSnapshot};
    use crate::arena::{Arena, NodeId};
    use crate::events::PointerEvent;
    use crate::flux::Flux;
    use crate::geom::{Point, Rect, Size, Vector};
    use crate::mask::Change;
    use crate::widget::{EventResponse, LabelSource, Role, Widget};
    use reactive_graph::traits::Set;
    use rustc_hash::{FxHashMap, FxHashSet};
    use std::sync::Mutex;

    struct Fixed {
        size: crate::geom::Size,
    }

    impl Widget for Fixed {
        fn role(&self) -> Role {
            Role::Container
        }

        fn label(&self) -> LabelSource {
            LabelSource::text("fixed")
        }

        fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
            Some(crate::style::MeasureSpec::Fixed(self.size))
        }
    }

    /// A layout engine that stacks children vertically with fixed sizes —
    /// deterministic geometry for pipeline tests, zero Taffy.
    struct StackLayout {
        measure_calls: u64,
    }

    impl StackLayout {
        fn new() -> Self {
            Self { measure_calls: 0 }
        }
    }

    impl LayoutEngine for StackLayout {
        fn sync_and_layout(
            &mut self,
            arena: &mut Arena,
            root: NodeId,
            _dirty: &FxHashMap<NodeId, Change>,
            viewport: Size,
        ) -> Result<(), LayoutError> {
            fn layout_subtree(
                engine: &mut StackLayout,
                arena: &mut Arena,
                node: NodeId,
                origin: Point,
                viewport: Size,
            ) {
                let spec = arena.widget(node).measure_spec();
                let (w, h) = match spec {
                    Some(crate::style::MeasureSpec::Fixed(size)) => {
                        engine.measure_calls += 1;
                        (size.width.min(viewport.width), size.height)
                    }
                    _ => (viewport.width, 0.0),
                };
                let bounds = Rect::new(origin, Size::new(w, h));
                arena.update_bounds(node, bounds);
                let mut y = origin.y;
                for child in arena.children(node).to_vec() {
                    layout_subtree(engine, arena, child, Point::new(origin.x, y), viewport);
                    y = arena.bounds(child).map_or(y, |b| b.max_y());
                }
            }
            layout_subtree(self, arena, root, Point::zero(), viewport);
            Ok(())
        }

        fn measure_calls(&self) -> u64 {
            self.measure_calls
        }

        fn reset_measure_calls(&mut self) {
            self.measure_calls = 0;
        }
    }

    #[derive(Default)]
    struct MemoryJournal {
        layers: Mutex<FxHashMap<NodeId, Vec<crate::scene::Primitive>>>,
        commits: Mutex<u64>,
    }

    impl SceneJournal for MemoryJournal {
        fn commit_layer(&mut self, node: NodeId, primitives: Vec<crate::scene::Primitive>) {
            *self.commits.lock().expect("journal lock") += 1;
            self.layers
                .lock()
                .expect("journal lock")
                .insert(node, primitives);
        }

        fn drop_layer(&mut self, node: NodeId) {
            self.layers.lock().expect("journal lock").remove(&node);
        }

        fn snapshot(&self, order: &[NodeId]) -> crate::scene::SceneSnapshot {
            let layers = self.layers.lock().expect("journal lock");
            let primitives = order
                .iter()
                .filter_map(|n| layers.get(n))
                .flatten()
                .cloned()
                .collect();
            crate::scene::SceneSnapshot { primitives }
        }

        fn retain(&mut self, order: &[NodeId]) {
            let keep: FxHashSet<NodeId> = order.iter().copied().collect();
            self.layers
                .lock()
                .expect("journal lock")
                .retain(|n, _| keep.contains(n));
        }

        fn clear(&mut self) {
            self.layers.lock().expect("journal lock").clear();
        }
    }

    #[derive(Default)]
    struct RecordingA11y {
        updates: u64,
        last_dirty: FxHashSet<u64>,
        tree: Option<TreeSnapshot>,
    }

    impl A11ySink for RecordingA11y {
        fn apply(&mut self, snapshot: &TreeSnapshot, dirty: &FxHashSet<u64>) {
            self.updates += 1;
            self.last_dirty = dirty.clone();
            self.tree = Some(snapshot.clone());
        }

        fn resolved_tree(&self) -> Option<&TreeSnapshot> {
            self.tree.as_ref()
        }
    }

    fn pipeline() -> PassPipeline {
        PassPipeline::new(
            Flux::new(),
            Box::new(StackLayout::new()),
            Box::new(MemoryJournal::default()),
            Box::new(RecordingA11y::default()),
        )
    }

    fn viewport() -> Size {
        Size::new(200.0, 400.0)
    }

    #[test]
    fn first_frame_runs_every_pass_and_reports_geometry() {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }));
        let report = p.tick(viewport()).expect("first frame");

        assert!(report.layout_ran && report.paint_ran && report.a11y_ran);
        let bounds = p.arena().bounds(root).expect("bounds after layout");
        assert_eq!(bounds, Rect::new(Point::zero(), Size::new(100.0, 30.0)));
        assert_eq!(p.counters().layout_runs, 1);
        assert_eq!(p.counters().paint_runs, 1);
        assert_eq!(p.counters().a11y_runs, 1);
    }

    #[test]
    fn idle_tick_skips_every_pass() {
        let mut p = pipeline();
        let _ = p.mount_root(Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }));
        p.tick(viewport()).expect("first frame");
        let counters = p.counters();

        let report = p.tick(viewport()).expect("idle frame");
        assert!(report.is_idle());
        assert_eq!(p.counters(), counters, "no pass ran when nothing changed");
    }

    #[test]
    fn a11y_snapshot_uses_id_path_hashes_as_node_ids() {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }));
        p.tick(viewport()).expect("frame");

        let tree = p.a11y_snapshot().expect("tree built");
        let root_node = tree
            .nodes
            .iter()
            .find(|n| n.id == root_id_hash(root))
            .expect("root in tree");
        assert_eq!(root_node.role, Role::Container);
        assert_eq!(root_node.label, "fixed");
    }

    fn root_id_hash(root: NodeId) -> u64 {
        let mut path = crate::identity::IdPath::default();
        path.push(root.raw());
        path.hash()
    }

    #[test]
    fn pointer_events_route_to_the_topmost_hit_and_stop_on_handled() {
        struct Handling {
            at: Rect,
        }

        impl Widget for Handling {
            fn role(&self) -> Role {
                Role::Button
            }

            fn label(&self) -> LabelSource {
                LabelSource::text("hit")
            }

            fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
                Some(crate::style::MeasureSpec::Fixed(self.at.size))
            }

            fn on_pointer_event(
                &mut self,
                _ctx: &mut crate::events::EventCtx,
                _event: &PointerEvent,
            ) -> EventResponse {
                EventResponse::Handled
            }
        }

        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 400.0),
        }));
        let _btn = p.add_child(
            root,
            Box::new(Handling {
                at: Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 30.0)),
            }),
        );
        p.tick(viewport()).expect("frame");

        let down = PointerEvent::Down {
            position: Point::new(10.0, 10.0),
            button: crate::events::MouseButton::Left,
        };
        assert_eq!(
            p.dispatch_pointer(&down),
            EventResponse::Handled,
            "button hit first, bubbling stops"
        );
        assert_eq!(p.counters().events_routed, 1);

        let miss = PointerEvent::Down {
            position: Point::new(199.0, 399.0),
            button: crate::events::MouseButton::Left,
        };
        assert_eq!(
            p.dispatch_pointer(&miss),
            EventResponse::Ignored,
            "routes to root, which ignores"
        );
    }

    #[test]
    fn compose_translates_scrolled_children_without_re_measuring() {
        struct Scroll {
            offset: std::sync::Arc<Mutex<f32>>,
        }

        impl Widget for Scroll {
            fn role(&self) -> Role {
                Role::ScrollArea
            }

            fn label(&self) -> LabelSource {
                LabelSource::text("scroll")
            }

            fn style(&self) -> crate::style::Style {
                crate::style::Style {
                    clip_overflow: true,
                    ..crate::style::Style::default()
                }
            }

            fn scroll_offset(&self) -> Vector {
                Vector::new(0.0, *self.offset.lock().expect("scroll lock"))
            }
        }

        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 400.0),
        }));
        let offset_cell = std::sync::Arc::new(Mutex::new(0.0_f32));
        let scroll = p.add_child(
            root,
            Box::new(Scroll {
                offset: std::sync::Arc::clone(&offset_cell),
            }),
        );
        let item = p.add_child(
            scroll,
            Box::new(Fixed {
                size: Size::new(100.0, 50.0),
            }),
        );
        p.tick(viewport()).expect("frame");
        let before = p.arena().bounds(item).expect("item bounds");
        p.reset_measure_calls();

        // A wheel event moved the scroll offset — no layout mask anywhere.
        *offset_cell.lock().expect("scroll lock") = 30.0;
        // Dispatching an event marks input as arrived (compose runs).
        let wheel = PointerEvent::Wheel {
            delta: Vector::new(0.0, -30.0),
            position: Point::new(10.0, 10.0),
        };
        p.dispatch_pointer(&wheel);
        let report = p.tick(viewport()).expect("scroll frame");

        assert!(
            !report.layout_ran,
            "scroll must not relayout (translation-only, §8.2)"
        );
        assert!(report.paint_ran, "translated nodes repaint");
        assert_eq!(
            p.measure_calls(),
            0,
            "zero measure calls for translation-only updates"
        );
        let after = p.arena().bounds(item).expect("item bounds");
        assert_eq!(
            after.max_y(),
            before.max_y() - 30.0,
            "child moved up by the delta"
        );
        assert_eq!(after.width(), before.width(), "size untouched by compose");
    }

    #[test]
    fn one_write_batch_means_one_layout_and_paint_cycle() {
        // Serialize against the process-global BATCH state (see
        // `flux::batch_test_lock` for why this is required).
        let _guard = crate::flux::batch_test_lock();

        // The ADR-6 kill-switch, at pipeline level: 5 synchronous writes inside
        // one batch ⇒ exactly one layout pass and one paint pass.
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 400.0),
        }));
        let child = p.add_child(
            root,
            Box::new(Fixed {
                size: Size::new(100.0, 30.0),
            }),
        );
        p.tick(viewport()).expect("first frame");

        let signal = p.flux().signal(0i32);
        p.flux_mut().bind(signal, child, |v: &i32| {
            let _ = v;
            Change::LAYOUT.union(Change::PAINT)
        });
        p.tick(viewport()).expect("binding frame");

        p.flux().batch(|| {
            for i in 1..=5 {
                signal.set(i);
            }
        });
        let report = p.tick(viewport()).expect("write-batch frame");

        assert!(report.layout_ran && report.paint_ran);
        assert_eq!(
            p.counters().layout_runs,
            3,
            "first + binding + batch — never 7"
        );
        assert_eq!(p.counters().paint_runs, 3);

        let idle = p.tick(viewport()).expect("idle frame");
        assert!(idle.is_idle());
        assert_eq!(p.counters().layout_runs, 3, "nothing pending: idle");
    }
}
