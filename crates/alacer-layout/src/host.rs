//! The layout engine: a persistent Taffy tree that is **never cleared between
//! frames**, with ancestor-candidate relayout (ADR-3, §8.1–8.2).

use crate::measure_cache::{LruMeasureCache, MeasureKey};
use crate::style_map::{definite_width, to_taffy};
use alacer_core::arena::{Arena, NodeId};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::mask::Change;
use alacer_core::passes::{LayoutEngine, LayoutError};
use alacer_core::style::{MeasureFn, MeasureSpec};
use rustc_hash::{FxHashMap, FxHashSet};
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
    ///
    /// # Panics
    ///
    /// If the cell's lock is poisoned — a panicked writer means the frame is
    /// corrupt either way.
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
    /// Each container's last-synced taffy children — `set_children` fires
    /// only on real order changes (an unconditional re-set would dirty the
    /// whole persistent tree every frame and defeat taffy's caches).
    children_cache: FxHashMap<NodeId, Vec<taffy::NodeId>>,
    measurer: Arc<dyn MeasureFn>,
    cache: LruMeasureCache,
    measure_calls: u64,
    /// The synthetic viewport-fill root the arena root hangs from: taffy
    /// shrink-wraps an auto-sized compute root, so filling the window needs a
    /// sized ancestor (the classic `size: 100%` shell node).
    fill_root: Option<taffy::NodeId>,
    /// The arena root currently attached under [`Self::fill_root`].
    fill_attached: Option<NodeId>,
    /// The region the last compute produced, when it was candidate-bounded:
    /// `(arena candidate, absolute origin of the candidate's box)`. Consumed
    /// by the bounded publish; `None` means publish the whole tree.
    pending_region: Option<(NodeId, Point)>,
    /// The last computed viewport — a resize forces a full recomputation
    /// even when no mask says so (the fill root's percent sizes resolve
    /// against it).
    last_viewport: Option<Size>,
    /// Each node's last-published parent-relative location. Taffy zeroes a
    /// compute root's location, so the engine keeps its own record for
    /// candidate seeding (ADR-0008 item 1).
    loc_cache: FxHashMap<NodeId, Point>,
}

impl LayoutHost {
    /// Creates the engine around an injected text measurer.
    #[must_use]
    pub fn new(measurer: Arc<dyn MeasureFn>) -> Self {
        Self {
            tree: taffy::TaffyTree::new(),
            nodes: FxHashMap::default(),
            children_cache: FxHashMap::default(),
            measurer,
            cache: LruMeasureCache::new(1024),
            measure_calls: 0,
            fill_root: None,
            fill_attached: None,
            pending_region: None,
            last_viewport: None,
            loc_cache: FxHashMap::default(),
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
    /// creates missing nodes, re-syncs child order **only where it changed**,
    /// and removes nodes whose arena slot died. Structural work only — the
    /// tree itself is never cleared (§8.1), and an unchanged frame pays
    /// nothing: an unconditional `set_children` would dirty every container
    /// and silently defeat taffy's caches (ADR-0008 item 3).
    fn sync_tree(&mut self, arena: &Arena, root: NodeId) {
        // One pre-order walk defines liveness for this sync (a per-node
        // ancestor walk would be O(n·depth)).
        let mut live = FxHashSet::default();
        arena.for_each_pre_order(&mut |id, _| {
            live.insert(id);
        });

        // Create missing nodes.
        arena.for_each_pre_order(&mut |id, slot| {
            if self.nodes.contains_key(&id) {
                return;
            }
            let style = mapped_style(&slot.widget().style(), id == root);
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
            self.children_cache.entry(id).or_default();
        });

        // Sync child order where it actually changed.
        arena.for_each_pre_order(&mut |id, slot| {
            let Some(parent_taffy) = self.nodes.get(&id).copied() else {
                return;
            };
            let children: Vec<taffy::NodeId> = slot
                .children
                .iter()
                .filter_map(|c| self.nodes.get(c).copied())
                .collect();
            let unchanged = self
                .children_cache
                .get(&id)
                .is_some_and(|cached| cached.as_slice() == children.as_slice());
            if unchanged {
                return;
            }
            self.tree
                .set_children(parent_taffy, &children)
                .expect("parent is live");
            self.children_cache.insert(id, children);
        });

        // Prune dead slots.
        let removed: Vec<NodeId> = self
            .nodes
            .keys()
            .copied()
            .filter(|id| !live.contains(id))
            .collect();
        for id in removed {
            if let Some(taffy_id) = self.nodes.remove(&id) {
                let _ = self.tree.remove(taffy_id);
            }
            self.children_cache.remove(&id);
        }
        if !live.contains(&root) {
            self.fill_attached = None;
        }
    }

    /// Refreshes style + measure context for dirty nodes.
    fn refresh_dirty(&mut self, arena: &Arena, root: NodeId, dirty: &FxHashMap<NodeId, Change>) {
        for (id, change) in dirty {
            let Some(taffy_id) = self.nodes.get(id).copied() else {
                continue;
            };
            if change.contains(Change::LAYOUT) {
                let style = mapped_style(&arena.widget(*id).style(), *id == root);
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

    /// Marks Taffy dirty and computes the frame's layout region.
    ///
    /// Structural frames and viewport changes compute from the fill root.
    /// Otherwise this is torin's root candidate (§8.2 item 3): the **closest
    /// common ancestor** of the dirty set is recomputed against its cached
    /// size, and if the candidate's own box changes, the candidate walks up
    /// one level and recomputes — bounded by tree depth (ADR-0008 item 1).
    /// Returns the Taffy node the computation was rooted at.
    #[must_use]
    fn compute_region(
        &mut self,
        arena: &Arena,
        dirty: &FxHashMap<NodeId, Change>,
        root: NodeId,
        viewport: Size,
    ) -> taffy::NodeId {
        let root_taffy = self.nodes.get(&root).copied().expect("root mapped");
        let structural = dirty.values().any(|m| m.contains(Change::STRUCTURE));
        let resized = self.last_viewport != Some(viewport);

        if structural || resized {
            let fill_root = self.ensure_fill_root(root_taffy, root);
            self.mark_layout_dirty(dirty);
            self.compute_from(fill_root, definite_available(viewport));
            self.pending_region = None;
            self.last_viewport = Some(viewport);
            return fill_root;
        }

        // Candidate path (torin's root candidate, §8.2 item 3): the compute
        // root is the LCA's **arranger** — its parent container (or the fill
        // root for the arena root) — because a standalone compute loses the
        // flex context (stretch, grow, position) that only a parent supplies.
        // If the compute root's own box changes, it walks up one level and
        // recomputes, bounded by tree depth (ADR-0008 item 1).
        let dirty_nodes: Vec<NodeId> = dirty
            .iter()
            .filter(|(_, m)| m.intersects(Change::LAYOUT))
            .map(|(id, _)| *id)
            .collect();
        self.mark_layout_dirty(dirty);

        let lca = Self::closest_common_ancestor(arena, &dirty_nodes);
        // The compute root is the LCA's **arranger** — its parent container —
        // because a standalone compute loses the flex context (stretch, grow,
        // position) that only a parent supplies. One exception: a candidate
        // whose style fixes BOTH axes has a parent-independent box, so it can
        // compute standalone — that keeps a fixed-size card's churn entirely
        // inside the card (the W1 workload's active section). The arena root
        // still arranges under the fill root.
        let lca_style = arena.widget(lca).style();
        let lca_definite = lca_style.size.width.is_some() && lca_style.size.height.is_some();
        let mut compute_root: Option<NodeId> = if lca == root {
            None // the fill root arranges the arena root
        } else if lca_definite {
            Some(lca)
        } else {
            arena.parent(lca)
        };
        let region = loop {
            match compute_root {
                None => {
                    let fill_root = self.ensure_fill_root(root_taffy, root);
                    self.compute_from(fill_root, definite_available(viewport));
                    break (root, Point::zero());
                }
                Some(node) => {
                    let node_taffy = self.nodes.get(&node).copied().expect("region root mapped");
                    let cached_size = {
                        let layout = self.tree.layout(node_taffy).expect("region root layout");
                        Size::new(layout.size.width, layout.size.height)
                    };
                    // Parent-relative location from the publish cache — taffy
                    // zeroes a compute root's location, so its own layout
                    // store is not authoritative for compute roots.
                    let cached_loc = self.loc_cache.get(&node).copied().unwrap_or(Point::zero());
                    self.compute_from(node_taffy, definite_available(cached_size));
                    let new_size = {
                        let layout = self.tree.layout(node_taffy).expect("layout");
                        Size::new(layout.size.width, layout.size.height)
                    };
                    if new_size == cached_size {
                        let parent_abs = arena
                            .bounds(arena.parent(node).expect("non-root has a parent"))
                            .map_or(Point::zero(), |r| r.origin);
                        break (
                            node,
                            Point::new(parent_abs.x + cached_loc.x, parent_abs.y + cached_loc.y),
                        );
                    }
                    // The compute root's box changed: its own arrangement is
                    // stale — walk up (torin's dependants walk).
                    compute_root = arena.parent(node);
                }
            }
        };

        self.pending_region = Some(region);
        self.last_viewport = Some(viewport);
        let (region_root, _) = region;
        self.nodes
            .get(&region_root)
            .copied()
            .expect("region root mapped")
    }

    /// The synthetic viewport-fill root, attached under the arena root.
    fn ensure_fill_root(&mut self, root_taffy: taffy::NodeId, root: NodeId) -> taffy::NodeId {
        let fill_root = *self.fill_root.get_or_insert_with(|| {
            self.tree
                .new_leaf(taffy::Style {
                    // Fill the viewport both ways: column direction puts the
                    // arena root on the main axis (`flex_grow: 1` fills the
                    // height), stretch covers the width.
                    flex_direction: taffy::FlexDirection::Column,
                    align_items: taffy::AlignItems::STRETCH,
                    size: taffy::Size {
                        width: taffy::Dimension::percent(1.0),
                        height: taffy::Dimension::percent(1.0),
                    },
                    ..taffy::Style::default()
                })
                .expect("fill root")
        });
        if self.fill_attached != Some(root) {
            self.tree
                .set_children(fill_root, &[root_taffy])
                .expect("fill root is live");
            self.fill_attached = Some(root);
        }
        fill_root
    }

    /// Marks every layout-relevant dirty node in the taffy tree.
    fn mark_layout_dirty(&mut self, dirty: &FxHashMap<NodeId, Change>) {
        for (id, change) in dirty {
            if change.intersects(Change::LAYOUT.union(Change::STRUCTURE))
                && let Some(taffy_id) = self.nodes.get(id)
            {
                let _ = self.tree.mark_dirty(*taffy_id);
            }
        }
    }

    /// One taffy compute over `compute_root` with the shared measure closure.
    fn compute_from(
        &mut self,
        compute_root: taffy::NodeId,
        available: taffy::Size<taffy::AvailableSpace>,
    ) {
        let mut calls = 0_u64;
        let mut cache = std::mem::take(&mut self.cache);
        let measurer = Arc::clone(&self.measurer);
        let compute = self.tree.compute_layout_with_measure(
            compute_root,
            available,
            |input, _node, ctx: Option<&mut SpecCell>, _style| {
                let Some(cell) = ctx else {
                    return taffy::LayoutOutput::HIDDEN;
                };
                let known = input.known_dimensions;
                let Some(spec) = cell.get() else {
                    // No measure request: the node is content-less, but a
                    // stretched or fixed known dimension still applies.
                    return taffy::LayoutOutput::from_outer_size(taffy::Size {
                        width: known.width.unwrap_or(0.0),
                        height: known.height.unwrap_or(0.0),
                    });
                };
                // A definite known width is the wrap width for text. The
                // wrap decision is **integral** (§8.3, egui's re-wrap
                // feedback lesson): sub-pixel container jitter must never
                // flip a line break, so the width floors to whole device
                // pixels before it reaches the measurer or the cache key.
                let max_width = known
                    .width
                    .or_else(|| definite_width(input.available_space.width))
                    .map(f32::floor);
                calls += 1;
                match spec {
                    MeasureSpec::Fixed(size) => taffy::LayoutOutput::from_outer_size(taffy::Size {
                        width: known.width.unwrap_or(size.width),
                        height: known.height.unwrap_or(size.height),
                    }),
                    MeasureSpec::Text(text) => {
                        let key = MeasureKey::from_spec(&text, max_width);
                        if let Some(size) = cache.get(&key) {
                            return taffy::LayoutOutput::from_outer_size(taffy::Size {
                                width: known.width.unwrap_or(size.width),
                                height: known.height.unwrap_or(size.height),
                            });
                        }
                        let size = measurer.measure(&text, max_width);
                        cache.put(key, size);
                        taffy::LayoutOutput::from_outer_size(taffy::Size {
                            width: known.width.unwrap_or(size.width),
                            height: known.height.unwrap_or(size.height),
                        })
                    }
                }
            },
        );
        compute.expect("taffy compute succeeds on well-formed trees");
        self.cache = cache;
        self.measure_calls += calls;
    }

    /// The closest common ancestor of `nodes` in the arena (root included).
    /// Every node is live and mapped by the time layout runs.
    fn closest_common_ancestor(arena: &Arena, nodes: &[NodeId]) -> NodeId {
        debug_assert!(!nodes.is_empty(), "the candidate path needs dirty nodes");
        let paths: Vec<Vec<NodeId>> = nodes.iter().map(|n| Self::root_path(arena, *n)).collect();
        let shortest = paths
            .iter()
            .min_by_key(|p| p.len())
            .expect("non-empty dirty set");
        for depth in (0..shortest.len()).rev() {
            let candidate = shortest[depth];
            if paths
                .iter()
                .all(|path| path.len() > depth && path[depth] == candidate)
            {
                return candidate;
            }
        }
        shortest[0]
    }

    /// The arena path `root → node`, parents via the arena's parent links.
    fn root_path(arena: &Arena, node: NodeId) -> Vec<NodeId> {
        let mut path = vec![node];
        let mut cursor = node;
        while let Some(parent) = arena.parent(cursor) {
            path.push(parent);
            cursor = parent;
        }
        path.reverse();
        path
    }

    /// Writes absolute bounds back into the arena — the recomputed region
    /// only when the frame was candidate-bounded, the whole tree otherwise
    /// (ADR-0008 item 2). Bounds outside a candidate region stay valid by
    /// construction: nothing above or beside it was recomputed.
    fn publish_bounds(&mut self, arena: &mut Arena, root: NodeId) {
        match self.pending_region.take() {
            Some((candidate, origin)) => publish(self, arena, candidate, origin),
            None => publish(self, arena, root, Point::zero()),
        }
    }
}

fn layout_dirty_nodes(dirty: &FxHashMap<NodeId, Change>) -> Vec<NodeId> {
    dirty
        .iter()
        .filter(|(_, c)| c.intersects(Change::LAYOUT.union(Change::STRUCTURE)))
        .map(|(id, _)| *id)
        .collect()
}

/// Definite available space for one size — a candidate is computed against
/// its cached box.
fn definite_available(size: Size) -> taffy::Size<taffy::AvailableSpace> {
    taffy::Size {
        width: taffy::AvailableSpace::Definite(size.width),
        height: taffy::AvailableSpace::Definite(size.height),
    }
}

/// Maps a widget style to Taffy; the arena root additionally grows into the
/// synthetic fill root's main axis so the app fills the window both ways
/// (cross-axis stretch covers width; `flex_grow` covers height).
fn mapped_style(style: &alacer_core::style::Style, is_root: bool) -> taffy::Style {
    let mut mapped = to_taffy(style);
    if is_root {
        mapped.flex_grow = 1.0;
    }
    mapped
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
        let Some(child_taffy) = host.nodes.get(&child).copied() else {
            continue;
        };
        let Ok(child_layout) = host.tree.layout(child_taffy) else {
            continue;
        };
        // The parent-relative location cache: taffy zeroes a compute root's
        // location, so the engine keeps the publish-derived record.
        host.loc_cache.insert(
            child,
            Point::new(child_layout.location.x, child_layout.location.y),
        );
        publish(host, arena, child, abs);
    }
}

impl LayoutEngine for LayoutHost {
    fn sync_and_layout(
        &mut self,
        arena: &mut Arena,
        root: NodeId,
        dirty: &FxHashMap<NodeId, Change>,
        viewport: Size,
    ) -> Result<(), LayoutError> {
        // Structural frames sync the persistent tree with the arena's shape;
        // style-churn frames cannot create, remove, or reorder nodes, so the
        // whole sync (three O(n) walks) is skippable (ADR-0008 item 3).
        let structural =
            dirty.values().any(|m| m.contains(Change::STRUCTURE)) || self.nodes.is_empty();
        if structural {
            self.sync_tree(arena, root);
        }
        if layout_dirty_nodes(dirty).is_empty() {
            // Paint- or a11y-only frame: the persistent tree is untouched and
            // cached layouts stay valid (§8.1 — never re-measure without need).
            return Ok(());
        }
        self.refresh_dirty(arena, root, dirty);
        let _region = self.compute_region(arena, dirty, root, viewport);
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
    use rustc_hash::FxHashMap;
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
        fn fixed(w: f32, h: f32) -> Self {
            Self {
                style: Style::default(),
                shared: None,
                spec: Some(MeasureSpec::Fixed(Size::new(w, h))),
            }
        }

        fn with_style(style: Style) -> Self {
            Self {
                style,
                shared: None,
                spec: None,
            }
        }

        /// A box whose style can be churned externally between frames.
        fn churnable(style: Style) -> (Self, Arc<Mutex<Style>>) {
            let cell = Arc::new(Mutex::new(style.clone()));
            (
                Self {
                    style,
                    shared: Some(Arc::clone(&cell)),
                    spec: None,
                },
                cell,
            )
        }

        fn text(text: &str) -> Self {
            Self {
                style: Style::default(),
                shared: None,
                spec: Some(MeasureSpec::Text(TextSpec {
                    text: text.to_string(),
                    font_id: 1,
                    font_size: 14.0,
                })),
            }
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
            let width = alacer_core::num::usize_to_f32(spec.text.len().min(200)) * 6.0;
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
        let root = arena.set_root(Box::new(TestBox::with_style(Style::column())));
        let a = arena.add_child(root, Box::new(TestBox::fixed(100.0, 30.0)));
        let b = arena.add_child(root, Box::new(TestBox::fixed(50.0, 20.0)));

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
            Some(Rect::new(Point::new(0.0, 0.0), Size::new(400.0, 30.0))),
            "first child at the top, stretched to the container width"
        );
        assert_eq!(
            arena.bounds(b),
            Some(Rect::new(Point::new(0.0, 30.0), Size::new(400.0, 20.0))),
            "second child below the first"
        );
    }

    #[test]
    fn text_leaves_measure_through_the_seam() {
        let mut arena = Arena::new();
        // column → row → text leaf: the column fills the viewport, the row
        // shrinks to its content, and the leaf's main axis (width) stays
        // measured while its cross axis stretches to the row's height.
        let root = arena.set_root(Box::new(TestBox::with_style(Style::column())));
        let row = arena.add_child(root, Box::new(TestBox::with_style(Style::row())));
        let label = arena.add_child(row, Box::new(TestBox::text("hello")));

        let mut dirty = FxHashMap::default();
        dirty.insert(root, Change::STRUCTURE);
        dirty.insert(row, Change::STRUCTURE);
        dirty.insert(label, Change::STRUCTURE);
        let mut engine = host();
        engine
            .sync_and_layout(&mut arena, root, &dirty, viewport())
            .expect("layout");

        let bounds = arena.bounds(label).expect("label bounds");
        assert_eq!(bounds.size.width, 30.0, "5 chars x 6 px, measured");
        assert_eq!(bounds.size.height, 14.0);
    }

    #[test]
    fn persistent_tree_survives_across_frames() {
        let mut arena = Arena::new();
        let root = arena.set_root(Box::new(TestBox::with_style(Style::column())));
        let a = arena.add_child(root, Box::new(TestBox::fixed(100.0, 30.0)));
        let b = arena.add_child(root, Box::new(TestBox::fixed(50.0, 20.0)));

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
        let root = arena.set_root(Box::new(TestBox::with_style(Style::column())));
        let child = arena.add_child(root, Box::new(TestBox::fixed(100.0, 30.0)));
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
        let root = arena.set_root(Box::new(TestBox::with_style(Style::column())));
        let mut ids = Vec::new();
        let mut cells = Vec::new();
        for _ in 0..10 {
            let grow = alacer_core::num::u64_to_f32(rng.next() % 4) / 2.0;
            let (child, cell) = TestBox::churnable(Style {
                grow,
                ..Style::default()
            });
            let child_id = arena.add_child(root, Box::new(child));
            if rng.next().is_multiple_of(3) {
                arena.add_child(
                    child_id,
                    Box::new(TestBox::fixed(
                        f32::from((rng.next() % 40) as u16) + 20.0,
                        14.0,
                    )),
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
        // compared against a fresh engine that rebuilds from scratch. The
        // oracle replays the *identical* churn onto the pristine tree first,
        // so both engines lay out the same final tree and the bounds must
        // agree exactly.
        let mut rng = Lcg(0x5EED_2026);
        for case in 0..64_u64 {
            let seed = rng.next();
            let (mut inc_arena, ids, cells) = build_tree(seed);
            let (mut full_arena, full_ids, full_cells) = build_tree(seed);

            let mut inc = host();
            let initial = all_dirty(&inc_arena, &ids);
            let inc_root = inc_arena.root().expect("root");
            inc.sync_and_layout(&mut inc_arena, inc_root, &initial, viewport())
                .expect("initial");

            // The churn: one child's grow factor changes per step; the same
            // ops are recorded for the oracle.
            let mut churn: Vec<(usize, Style)> = Vec::new();
            for _ in 0..6 {
                let victim_idx =
                    usize::try_from(rng.next() % u64::try_from(ids.len()).expect("countable"))
                        .expect("in range");
                let victim = ids[victim_idx];
                let new_grow = alacer_core::num::u64_to_f32(rng.next() % 4) / 2.0;
                let style = Style {
                    grow: new_grow,
                    ..Style::default()
                };
                *cells[victim_idx]
                    .as_ref()
                    .expect("churnable cell")
                    .lock()
                    .expect("style cell") = style.clone();
                churn.push((victim_idx, style));
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

            // Oracle: replay the churn on the pristine tree, then lay
            // everything out from scratch on a fresh engine.
            for (victim_idx, style) in &churn {
                *full_cells[*victim_idx]
                    .as_ref()
                    .expect("churnable cell")
                    .lock()
                    .expect("style cell") = style.clone();
            }
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
        let root = pipeline.mount_root(Box::new(TestBox::with_style(Style::column())));
        let row = pipeline.add_child(root, Box::new(TestBox::with_style(Style::row())));
        let a = pipeline.add_child(row, Box::new(TestBox::fixed(100.0, 30.0)));
        let report = pipeline.tick(viewport()).expect("frame");
        assert!(report.layout_ran);
        assert_eq!(
            pipeline.arena().bounds(a).expect("bounds").size,
            Size::new(100.0, 30.0),
            "row main axis leaves the measured width alone"
        );
    }

    /// Null sinks for pipeline-level tests.
    struct NullJournal;

    impl alacer_core::passes::SceneJournal for NullJournal {
        fn commit_layer(&mut self, _node: NodeId, _primitives: Vec<alacer_core::scene::Primitive>) {
        }
        fn drop_layer(&mut self, _node: NodeId) {}
        fn layer(&self, _node: NodeId) -> Option<Vec<alacer_core::scene::Primitive>> {
            None
        }
        fn snapshot(&self, _order: &[NodeId]) -> alacer_core::scene::SceneSnapshot {
            alacer_core::scene::SceneSnapshot::default()
        }
        fn retain(&mut self, _order: &[NodeId]) {}
        fn clear(&mut self) {}
        fn stats(&self) -> alacer_core::passes::JournalStats {
            alacer_core::passes::JournalStats::default()
        }
    }

    struct NullA11y;

    impl alacer_core::passes::A11ySink for NullA11y {
        fn apply(
            &mut self,
            _snapshot: std::sync::Arc<alacer_core::passes::TreeSnapshot>,
            _delta: &alacer_core::passes::A11yDelta,
        ) {
        }
    }
}
