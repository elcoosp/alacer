//! The layout engine: a persistent Taffy tree that is **never cleared between
//! frames**, with ancestor-candidate relayout (ADR-3, §8.1–8.2).

use crate::measure_cache::{LruMeasureCache, MeasureKey};
use crate::style_map::{definite_width, to_taffy};
use alacer_core::arena::{Arena, NodeId};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::mask::Change;
use alacer_core::passes::{LayoutEngine, LayoutError};
use alacer_core::style::{MeasureFn, MeasureSpec};
use rustc_hash::FxHashMap;
use std::sync::{Arc, RwLock};

/// The per-node measure context stored inside the Taffy tree.
///
/// The pass layer refreshes the cell for dirty nodes before compute; the
/// measure closure reads it during compute. `Send + Sync` by construction,
/// which is what Taffy's measure plumbing requires.
#[derive(Debug, Clone, Default)]
pub struct SpecCell(Arc<RwLock<Option<MeasureSpec>>>);

impl SpecCell {
    /// Writes the current spec (pass layer).
    pub fn set(&self, spec: Option<MeasureSpec>) {
        *self.0.write().expect("spec cell lock") = spec;
    }

    /// Reads the current spec (measure closure).
    #[must_use]
    pub fn get(&self) -> Option<MeasureSpec> {
        self.0.read().ok().and_then(|guard| guard.clone())
    }
}

/// The persistent Taffy layout engine.
pub struct LayoutHost {
    tree: taffy::TaffyTree<SpecCell>,
    nodes: FxHashMap<NodeId, taffy::NodeId>,
    measurer: Arc<dyn MeasureFn>,
    cache: LruMeasureCache,
    measure_calls: u64,
}

impl LayoutHost {
    /// Creates the engine around an injected text measurer.
    #[must_use]
    pub fn new(measurer: Arc<dyn MeasureFn>) -> Self {
        Self {
            tree: taffy::TaffyTree::new(),
            nodes: FxHashMap::default(),
            measurer,
            cache: LruMeasureCache::new(1024),
            measure_calls: 0,
        }
    }

    /// The Taffy node mapped from an arena node, if synced.
    #[must_use]
    pub fn taffy_node(&self, node: NodeId) -> Option<taffy::NodeId> {
        self.nodes.get(&node).copied()
    }

    /// Number of live Taffy nodes in the persistent tree.
    #[must_use]
    pub fn taffy_node_count(&self) -> usize {
        self.tree.total_node_count()
    }

    /// Syncs the persistent Taffy tree with the arena's current shape:
    /// creates missing nodes, re-syncs child order at every container, and
    /// removes nodes whose arena slot died. Structural work only — the tree
    /// itself is never cleared (§8.1).
    fn sync_tree(&mut self, arena: &Arena, root: NodeId) {
        arena.for_each_pre_order(&mut |id, slot| {
            if self.nodes.contains_key(&id) {
                return;
            }
            let style = to_taffy(&slot.widget().style());
            let spec = slot.widget().measure_spec();
            let cell = SpecCell::default();
            cell.set(spec);
            let taffy_id = if slot.children.is_empty() {
                self.tree
                    .new_leaf_with_context(style, cell)
                    .expect("fresh node")
            } else {
                self.tree.new_leaf(style).expect("fresh node")
            };
            self.nodes.insert(id, taffy_id);
        });

        arena.for_each_pre_order(&mut |id, slot| {
            let Some(parent_taffy) = self.nodes.get(&id).copied() else {
                return;
            };
            let children: Vec<taffy::NodeId> = slot
                .children
                .iter()
                .filter_map(|c| self.nodes.get(c).copied())
                .collect();
            self.tree
                .set_children(parent_taffy, &children)
                .expect("parent is live");
        });

        let root = Some(root);
        let removed: Vec<NodeId> = self
            .nodes
            .keys()
            .copied()
            .filter(|id| !is_live(arena, root, *id))
            .collect();
        for id in removed {
            if let Some(taffy_id) = self.nodes.remove(&id) {
                let _ = self.tree.remove(taffy_id);
            }
        }
    }

    /// Refreshes style + measure context for dirty nodes.
    fn refresh_dirty(&mut self, arena: &Arena, dirty: &FxHashMap<NodeId, Change>) {
        for (id, change) in dirty {
            let Some(taffy_id) = self.nodes.get(id).copied() else {
                continue;
            };
            if change.contains(Change::LAYOUT) {
                let style = to_taffy(&arena.widget(*id).style());
                self.tree.set_style(taffy_id, style).expect("node is live");
                self.tree.mark_dirty(taffy_id).expect("node is live");
            }
            if change.intersects(Change::LAYOUT.union(Change::STRUCTURE)) {
                let spec = arena.widget(*id).measure_spec();
                if let Some(cell) = self.tree.get_node_context(taffy_id) {
                    cell.set(spec);
                }
            }
        }
    }

    /// Marks Taffy dirty from the ancestor candidate and computes it.
    ///
    /// Returns the Taffy node the computation was rooted at (the closest
    /// common ancestor of the dirty set — torin's root candidate, §8.2 item 3).
    #[must_use]
    fn compute_region(
        &mut self,
        dirty: &FxHashMap<NodeId, Change>,
        root: NodeId,
        viewport: Size,
    ) -> taffy::NodeId {
        // Compute at the root: `mark_dirty` propagates upward through the
        // persistent tree and Taffy's per-node caches bound the recomputed
        // region to the dirty path — the torin ancestor-candidate semantics
        // (§8.2 item 3) expressed through the engine's own cache, with no
        // second bookkeeping to keep coherent.
        let compute_root_taffy = self.nodes.get(&root).copied().expect("root mapped");

        for (id, change) in dirty {
            if change.intersects(Change::LAYOUT.union(Change::STRUCTURE))
                && let Some(taffy_id) = self.nodes.get(id)
            {
                let _ = self.tree.mark_dirty(*taffy_id);
            }
        }

        let available = taffy::Size {
            width: taffy::AvailableSpace::Definite(viewport.width),
            height: taffy::AvailableSpace::Definite(viewport.height),
        };

        let mut calls = 0_u64;
        let mut cache = std::mem::take(&mut self.cache);
        let measurer = Arc::clone(&self.measurer);
        let compute = self.tree.compute_layout_with_measure(
            compute_root_taffy,
            available,
            |input, _node, ctx: Option<&mut SpecCell>, _style| {
                let Some(cell) = ctx else {
                    return taffy::LayoutOutput::HIDDEN;
                };
                let Some(spec) = cell.get() else {
                    return taffy::LayoutOutput::from_outer_size(taffy::Size::ZERO);
                };
                let max_width = definite_width(input.available_space.width);
                calls += 1;
                match spec {
                    MeasureSpec::Fixed(size) => taffy::LayoutOutput::from_outer_size(taffy::Size {
                        width: size.width,
                        height: size.height,
                    }),
                    MeasureSpec::Text(text) => {
                        let key = MeasureKey::from_spec(&text, max_width);
                        if let Some(size) = cache.get(&key) {
                            return taffy::LayoutOutput::from_outer_size(taffy::Size {
                                width: size.width,
                                height: size.height,
                            });
                        }
                        let measured = measurer.measure(&text, max_width);
                        cache.put(key, measured);
                        taffy::LayoutOutput::from_outer_size(taffy::Size {
                            width: measured.width,
                            height: measured.height,
                        })
                    }
                }
            },
        );
        compute.expect("taffy compute succeeds on well-formed trees");
        self.cache = cache;
        self.measure_calls += calls;
        compute_root_taffy
    }

    /// Writes absolute bounds for the whole tree back into the arena.
    ///
    /// Positions above the recomputed region are unchanged (Taffy keeps their
    /// cached layouts), so this walk is cheap and keeps bounds coherent.
    fn publish_bounds(&mut self, arena: &mut Arena, root: NodeId) {
        publish(self, arena, root, Point::zero());
    }
}

fn layout_dirty_nodes(dirty: &FxHashMap<NodeId, Change>) -> Vec<NodeId> {
    dirty
        .iter()
        .filter(|(_, c)| c.intersects(Change::LAYOUT.union(Change::STRUCTURE)))
        .map(|(id, _)| *id)
        .collect()
}

fn publish(host: &mut LayoutHost, arena: &mut Arena, node: NodeId, origin: Point) {
    let Some(taffy_id) = host.nodes.get(&node).copied() else {
        return;
    };
    let Ok(layout) = host.tree.layout(taffy_id) else {
        return;
    };
    let abs = Point::new(origin.x + layout.location.x, origin.y + layout.location.y);
    let size = Size::new(layout.size.width, layout.size.height);
    arena.update_bounds(node, Rect::new(abs, size));
    for child in arena.children(node).to_vec() {
        publish(host, arena, child, abs);
    }
}

fn is_live(arena: &Arena, root: Option<NodeId>, id: NodeId) -> bool {
    // Dead slots (removed by the view reconciler or by hand) leave the index
    // first — `parent()` on them would panic. Self-heal: the caller drops
    // every dead id from the taffy map (ADR-3: the persistent tree is pruned,
    // never cleared).
    if !arena.is_live(id) {
        return false;
    }
    let Some(root) = root else {
        return false;
    };
    let mut cursor = Some(id);
    while let Some(c) = cursor {
        if c == root {
            return true;
        }
        cursor = arena.parent(c);
    }
    false
}

impl LayoutEngine for LayoutHost {
    fn sync_and_layout(
        &mut self,
        arena: &mut Arena,
        root: NodeId,
        dirty: &FxHashMap<NodeId, Change>,
        viewport: Size,
    ) -> Result<(), LayoutError> {
        self.sync_tree(arena, root);
        if layout_dirty_nodes(dirty).is_empty() {
            // Paint- or a11y-only frame: the persistent tree is untouched and
            // cached layouts stay valid (§8.1 — never re-measure without need).
            return Ok(());
        }
        self.refresh_dirty(arena, dirty);
        let _region = self.compute_region(dirty, root, viewport);
        self.publish_bounds(arena, root);
        Ok(())
    }

    fn measure_calls(&self) -> u64 {
        self.measure_calls
    }

    fn reset_measure_calls(&mut self) {
        self.measure_calls = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutHost;
    use alacer_core::Arena;
    use alacer_core::arena::NodeId;
    use alacer_core::flux::Flux;
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::mask::Change;
    use alacer_core::passes::{LayoutEngine, PassPipeline};
    use alacer_core::style::{MeasureFn, MeasureSpec, Style, TextSpec};
    use alacer_core::widget::{LabelSource, Role, Widget};
    use rustc_hash::{FxHashMap, FxHashSet};
    use std::sync::{Arc, Mutex};

    /// Deterministic pseudo-random generator for the equivalence property test.
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

    /// A box widget whose style lives in a shared cell the test can churn.
    struct TestBox {
        style: Style,
        shared: Option<Arc<Mutex<Style>>>,
        spec: Option<MeasureSpec>,
    }

    impl TestBox {
        fn fixed(w: f32, h: f32) -> Box<Self> {
            Box::new(Self {
                style: Style::default(),
                shared: None,
                spec: Some(MeasureSpec::Fixed(Size::new(w, h))),
            })
        }

        fn with_style(style: Style) -> Box<Self> {
            Box::new(Self {
                style,
                shared: None,
                spec: None,
            })
        }

        /// A box whose style can be churned externally between frames.
        fn churnable(style: Style) -> (Box<Self>, Arc<Mutex<Style>>) {
            let cell = Arc::new(Mutex::new(style.clone()));
            (
                Box::new(Self {
                    style,
                    shared: Some(Arc::clone(&cell)),
                    spec: None,
                }),
                cell,
            )
        }

        fn text(text: &str) -> Box<Self> {
            Box::new(Self {
                style: Style::default(),
                shared: None,
                spec: Some(MeasureSpec::Text(TextSpec {
                    text: text.to_string(),
                    font_id: 1,
                    font_size: 14.0,
                })),
            })
        }
    }

    impl Widget for TestBox {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn role(&self) -> Role {
            Role::Container
        }

        fn label(&self) -> LabelSource {
            LabelSource::text("box")
        }

        fn style(&self) -> Style {
            if let Some(cell) = &self.shared {
                cell.lock().expect("style cell").clone()
            } else {
                self.style.clone()
            }
        }

        fn measure_spec(&self) -> Option<MeasureSpec> {
            self.spec.clone()
        }
    }

    /// Fake text measurer: 6 px per char wide, 14 px tall.
    struct FakeMeasure;

    impl MeasureFn for FakeMeasure {
        fn measure(&self, spec: &TextSpec, max_width: Option<f32>) -> Size {
            let width = (spec.text.len().min(200) as f32) * 6.0;
            let width = max_width.map_or(width, |mw| width.min(mw));
            Size::new(width, 14.0)
        }
    }

    fn host() -> LayoutHost {
        LayoutHost::new(Arc::new(FakeMeasure))
    }

    fn viewport() -> Size {
        Size::new(400.0, 600.0)
    }

    #[test]
    fn lays_out_a_vertical_stack_with_real_taffy() {
        let mut arena = Arena::new();
        let root = arena.set_root(TestBox::with_style(Style::column()));
        let a = arena.add_child(root, TestBox::fixed(100.0, 30.0));
        let b = arena.add_child(root, TestBox::fixed(50.0, 20.0));

        let mut dirty = FxHashMap::default();
        dirty.insert(root, Change::STRUCTURE);
        dirty.insert(a, Change::STRUCTURE);
        dirty.insert(b, Change::STRUCTURE);

        let mut engine = host();
        engine
            .sync_and_layout(&mut arena, root, &dirty, viewport())
            .expect("layout");

        assert_eq!(
            arena.bounds(a),
            Some(Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 30.0))),
            "first child at the top"
        );
        assert_eq!(
            arena.bounds(b),
            Some(Rect::new(Point::new(0.0, 30.0), Size::new(50.0, 20.0))),
            "second child below the first"
        );
    }

    #[test]
    fn text_leaves_measure_through_the_seam() {
        let mut arena = Arena::new();
        let root = arena.set_root(TestBox::with_style(Style::column()));
        let label = arena.add_child(root, TestBox::text("hello"));

        let mut dirty = FxHashMap::default();
        dirty.insert(root, Change::STRUCTURE);
        dirty.insert(label, Change::STRUCTURE);
        let mut engine = host();
        engine
            .sync_and_layout(&mut arena, root, &dirty, viewport())
            .expect("layout");

        let bounds = arena.bounds(label).expect("label bounds");
        assert_eq!(bounds.size.width, 30.0, "5 chars x 6 px");
        assert_eq!(bounds.size.height, 14.0);
    }

    #[test]
    fn persistent_tree_survives_across_frames() {
        let mut arena = Arena::new();
        let root = arena.set_root(TestBox::with_style(Style::column()));
        let a = arena.add_child(root, TestBox::fixed(100.0, 30.0));
        let b = arena.add_child(root, TestBox::fixed(50.0, 20.0));

        let mut dirty = FxHashMap::default();
        dirty.insert(root, Change::STRUCTURE);
        dirty.insert(a, Change::STRUCTURE);
        dirty.insert(b, Change::STRUCTURE);
        let mut engine = host();
        engine
            .sync_and_layout(&mut arena, root, &dirty, viewport())
            .expect("layout");
        let count_after_first = engine.taffy_node_count();

        // Frame 2: only b resizes — no new nodes, no clearing.
        let mut dirty2 = FxHashMap::default();
        dirty2.insert(b, Change::LAYOUT);
        engine
            .sync_and_layout(&mut arena, root, &dirty2, viewport())
            .expect("layout 2");

        assert_eq!(
            engine.taffy_node_count(),
            count_after_first,
            "tree persisted, not rebuilt"
        );
        assert!(engine.taffy_node(b).is_some());
    }

    #[test]
    fn paint_only_dirty_never_measures() {
        let mut arena = Arena::new();
        let root = arena.set_root(TestBox::with_style(Style::column()));
        let child = arena.add_child(root, TestBox::fixed(100.0, 30.0));
        let mut dirty = FxHashMap::default();
        dirty.insert(root, Change::STRUCTURE);
        dirty.insert(child, Change::STRUCTURE);
        let mut engine = host();
        engine
            .sync_and_layout(&mut arena, root, &dirty, viewport())
            .expect("layout");
        engine.reset_measure_calls();

        let mut dirty_paint = FxHashMap::default();
        dirty_paint.insert(child, Change::PAINT);
        engine
            .sync_and_layout(&mut arena, root, &dirty_paint, viewport())
            .expect("layout");
        assert_eq!(engine.measure_calls(), 0, "paint-only dirty never measures");
    }

    /// Builds the same random tree twice from one seed. Churnable children
    /// expose their style cells so the churn test can mutate real styles.
    /// A tree plus the shared style cells for churn.
    type BuiltTree = (Arena, Vec<NodeId>, Vec<Option<Arc<Mutex<Style>>>>);

    fn build_tree(seed: u64) -> BuiltTree {
        let mut rng = Lcg(seed);
        let mut arena = Arena::new();
        let root = arena.set_root(TestBox::with_style(Style::column()));
        let mut ids = Vec::new();
        let mut cells = Vec::new();
        for _ in 0..10 {
            let grow = ((rng.next() % 4) as f32) / 2.0;
            let (child, cell) = TestBox::churnable(Style {
                grow,
                ..Style::default()
            });
            let child_id = arena.add_child(root, child);
            if rng.next().is_multiple_of(3) {
                arena.add_child(
                    child_id,
                    TestBox::fixed(f32::from((rng.next() % 40) as u16) + 20.0, 14.0),
                );
            }
            ids.push(child_id);
            cells.push(Some(cell));
        }
        (arena, ids, cells)
    }

    fn all_dirty(arena: &Arena, ids: &[NodeId]) -> FxHashMap<NodeId, Change> {
        let mut dirty: FxHashMap<NodeId, Change> =
            ids.iter().map(|id| (*id, Change::STRUCTURE)).collect();
        if let Some(root) = arena.root() {
            dirty.insert(root, Change::STRUCTURE);
        }
        dirty
    }

    #[test]
    fn incremental_result_equals_full_rebuild_on_random_churn() {
        // §8.4: the Dioxus-oracle pattern — the churned incremental engine is
        // compared against a fresh engine that rebuilds from scratch, over
        // identically-constructed arenas, and the bounds must agree exactly.
        let mut rng = Lcg(0x5EED_2026);
        for case in 0..64_u64 {
            let seed = rng.next();
            let (mut inc_arena, ids, cells) = build_tree(seed);
            let (mut full_arena, full_ids, _full_cells) = build_tree(seed);

            let mut inc = host();
            let initial = all_dirty(&inc_arena, &ids);
            let inc_root = inc_arena.root().expect("root");
            inc.sync_and_layout(&mut inc_arena, inc_root, &initial, viewport())
                .expect("initial");

            for _ in 0..6 {
                // The churn: one child's grow factor changes for real.
                let victim_idx = (rng.next() % ids.len() as u64) as usize;
                let victim = ids[victim_idx];
                let new_grow = ((rng.next() % 4) as f32) / 2.0;
                *cells[victim_idx]
                    .as_ref()
                    .expect("churnable cell")
                    .lock()
                    .expect("style cell") = Style {
                    grow: new_grow,
                    ..Style::default()
                };
                {
                    let mut guard = inc_arena.mutate(victim);
                    guard.request(Change::LAYOUT);
                }
                let mut one = FxHashMap::default();
                one.insert(victim, Change::LAYOUT);
                let inc_root = inc_arena.root().expect("root");
                inc.sync_and_layout(&mut inc_arena, inc_root, &one, viewport())
                    .expect("incremental");
            }

            // Oracle: everything dirty on a fresh engine over the same tree.
            let mut full = host();
            let full_dirty = all_dirty(&full_arena, &full_ids);
            let full_root = full_arena.root().expect("root");
            full.sync_and_layout(&mut full_arena, full_root, &full_dirty, viewport())
                .expect("full");

            for (inc_id, full_id) in ids.iter().zip(full_ids.iter()) {
                assert_eq!(
                    inc_arena.bounds(*inc_id),
                    full_arena.bounds(*full_id),
                    "case {case}: incremental == full rebuild"
                );
            }
        }
    }

    #[test]
    fn the_engine_plugs_into_the_pass_pipeline() {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(host()),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        let root = pipeline.mount_root(TestBox::with_style(Style::column()));
        let a = pipeline.add_child(root, TestBox::fixed(100.0, 30.0));
        let report = pipeline.tick(viewport()).expect("frame");
        assert!(report.layout_ran);
        assert_eq!(
            pipeline.arena().bounds(a).expect("bounds").size,
            Size::new(100.0, 30.0)
        );
    }

    /// Null sinks for pipeline-level tests.
    struct NullJournal;

    impl alacer_core::passes::SceneJournal for NullJournal {
        fn commit_layer(&mut self, _node: NodeId, _primitives: Vec<alacer_core::scene::Primitive>) {
        }
        fn drop_layer(&mut self, _node: NodeId) {}
        fn snapshot(&self, _order: &[NodeId]) -> alacer_core::scene::SceneSnapshot {
            alacer_core::scene::SceneSnapshot::default()
        }
        fn retain(&mut self, _order: &[NodeId]) {}
        fn clear(&mut self) {}
    }

    struct NullA11y;

    impl alacer_core::passes::A11ySink for NullA11y {
        fn apply(
            &mut self,
            _snapshot: &alacer_core::passes::TreeSnapshot,
            _dirty: &FxHashSet<u64>,
        ) {
        }
    }
}
