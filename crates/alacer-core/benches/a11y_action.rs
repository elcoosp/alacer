//! A11y-action routing benchmarks (§11.5, ADR-0016): one hundred mixed
//! assistive-technology actions routed through pass 10 on a 1,000-node
//! arena — the full `node_for_hash` walk plus the widget hook per action.
//! The published `a11y_action/route_100_actions_1k` regression gate keeps
//! the action tier honest as the vocabulary grows: routing an action must
//! stay the same order of cost as routing a pointer event, never a
//! re-materialization of the a11y tree.

use alacer_core::arena::NodeId;
use alacer_core::flux::Flux;
use alacer_core::geom::{Point, Size};
use alacer_core::mask::Change;
use alacer_core::passes::{A11yDelta, A11ySink, PassPipeline, SceneJournal, TreeSnapshot};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::style::MeasureSpec;
use alacer_core::widget::{A11yAction, LabelSource, Role, ScrollUnit, Widget};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;

const VIEWPORT: Size = Size::new(800.0, 600.0);
const ACTIONS: usize = 100;

/// A fixed-size leaf that implements no actions: the measured cost is the
/// route itself (the id-hash walk and the refused hook), not consumption.
struct FixedBox {
    size: Size,
}

impl Widget for FixedBox {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Decoration
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("box")
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(self.size))
    }
}

/// The injected layout engine: stacks children vertically (deterministic
/// geometry, zero Taffy — the routing walk is what's under test).
struct StackLayout;

impl alacer_core::passes::LayoutEngine for StackLayout {
    fn sync_and_layout(
        &mut self,
        arena: &mut alacer_core::Arena,
        root: NodeId,
        _dirty: &FxHashMap<NodeId, Change>,
        _viewport: Size,
    ) -> Result<(), alacer_core::passes::LayoutError> {
        fn layout_subtree(arena: &mut alacer_core::Arena, node: NodeId, y: f32) -> f32 {
            let size = match arena.widget(node).measure_spec() {
                Some(MeasureSpec::Fixed(size)) => size,
                _ => Size::zero(),
            };
            arena.update_bounds(node, alacer_core::geom::Rect::new(Point::new(0.0, y), size));
            let mut cursor = y + size.height;
            for child in arena.children(node).to_vec() {
                cursor = layout_subtree(arena, child, cursor);
            }
            cursor
        }
        layout_subtree(arena, root, 0.0);
        Ok(())
    }

    fn measure_calls(&self) -> u64 {
        0
    }

    fn reset_measure_calls(&mut self) {}
}

struct NullJournal;

impl SceneJournal for NullJournal {
    fn commit_layer(&mut self, _node: NodeId, _primitives: Vec<Primitive>) {}
    fn drop_layer(&mut self, _node: NodeId) {}
    fn layer(&self, _node: NodeId) -> Option<Vec<Primitive>> {
        None
    }
    fn snapshot(&self, _order: &[NodeId]) -> SceneSnapshot {
        SceneSnapshot::default()
    }
    fn retain(&mut self, _order: &[NodeId]) {}
    fn clear(&mut self) {}
    fn stats(&self) -> alacer_core::passes::JournalStats {
        alacer_core::passes::JournalStats::default()
    }
}

struct NullA11y;

impl A11ySink for NullA11y {
    fn apply(&mut self, _snapshot: std::sync::Arc<TreeSnapshot>, _delta: &A11yDelta) {}
}

/// The deepest-last node's a11y id: the worst-case target, forcing the
/// full pre-order walk per routed action. The id is the id-path hash of
/// `[root, …, leaf]` (§6.3) — built here from the node ids the mount
/// returned, exactly the path the pipeline's own `id_hash` rebuilds.
fn deepest_target(root: NodeId, last_child: NodeId) -> u64 {
    let mut path = alacer_core::identity::IdPath::default();
    path.push(root.raw());
    path.push(last_child.raw());
    path.hash()
}

fn action(i: usize) -> A11yAction {
    match i % 5 {
        0 => A11yAction::Increment,
        1 => A11yAction::Decrement,
        2 => A11yAction::ScrollDown(ScrollUnit::Item),
        3 => A11yAction::ScrollUp(ScrollUnit::Page),
        _ => A11yAction::DefaultPress,
    }
}

/// The value tier's payloads (M11, ADR-0017): 100 distinct `SetValue`
/// actions, built once and cloned per routed action — the clone *is* the
/// adapter's real mapping cost, because a `SetValue` `ActionRequest`
/// carries its payload and `map_action_request` allocates the core action
/// from it. Routing then walks to the worst-case target and the leaf
/// refuses: the measured cost is the route plus the faithful payload
/// transport, never an a11y re-materialization.
fn set_value_actions() -> Vec<A11yAction> {
    (0..ACTIONS)
        .map(|i| A11yAction::SetValue(format!("value-{i}")))
        .collect()
}

fn bench_a11y_action_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("a11y_action");
    group.throughput(Throughput::Elements(
        u64::try_from(ACTIONS).expect("non-negative"),
    ));

    group.bench_function("route_100_actions_1k", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(StackLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        let root = pipeline.mount_root(Box::new(FixedBox {
            size: Size::new(800.0, 0.0),
        }));
        let mut last_child = root;
        for _ in 0..1000 {
            last_child = pipeline.add_child(
                root,
                Box::new(FixedBox {
                    size: Size::new(20.0, 0.4),
                }),
            );
        }
        pipeline.tick(VIEWPORT).expect("bench layout");
        let target = deepest_target(root, last_child);

        b.iter(|| {
            let mut consumed = 0;
            for i in 0..ACTIONS {
                if pipeline.dispatch_a11y_action(target, black_box(action(i))) {
                    consumed += 1;
                }
            }
            black_box(consumed);
        });
    });

    group.bench_function("route_100_setvalue_1k", |b| {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(StackLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        );
        let root = pipeline.mount_root(Box::new(FixedBox {
            size: Size::new(800.0, 0.0),
        }));
        let mut last_child = root;
        for _ in 0..1000 {
            last_child = pipeline.add_child(
                root,
                Box::new(FixedBox {
                    size: Size::new(20.0, 0.4),
                }),
            );
        }
        pipeline.tick(VIEWPORT).expect("bench layout");
        let target = deepest_target(root, last_child);
        let actions = set_value_actions();

        b.iter(|| {
            let mut consumed = 0;
            for action in &actions {
                if pipeline.dispatch_a11y_action(target, black_box(action.clone())) {
                    consumed += 1;
                }
            }
            black_box(consumed);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_a11y_action_route);
criterion_main!(benches);
