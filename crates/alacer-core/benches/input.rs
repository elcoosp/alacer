//! Input-batching benchmarks (§10.2, ADR-0014): a 50-event pointer storm
//! through the pipeline queue (coalesced, one routed event) against the
//! pre-M8 shape (50 direct dispatches). The ratio is the published
//! `input_storm_batching` budget — the §10.2 contract as a number.

use alacer_core::arena::NodeId;
use alacer_core::events::{MouseButton, PointerEvent};
use alacer_core::flux::Flux;
use alacer_core::geom::{Point, Size};
use alacer_core::mask::Change;
use alacer_core::passes::{A11yDelta, A11ySink, FocusMove, PassPipeline, SceneJournal, TreeSnapshot};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::style::MeasureSpec;
use alacer_core::widget::{LabelSource, Role, Widget};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;

const VIEWPORT: Size = Size::new(800.0, 600.0);
const STORM: usize = 50;
const CAPTURED_MOVES: usize = 100;
const FOCUS_MOVES: usize = 100;

/// A fixed-size leaf: geometry without text, so the routing walk is the
/// cost under test.
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

/// A focusable leaf: `Role::Button` joins the default `focusable()`
/// derivation, so the walk's candidate list is the 1k children — the
/// focus-move routes are the walk's cost per move, nothing else.
struct FocusBox {
    size: Size,
}

impl Widget for FocusBox {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Button
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("stop")
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

/// A pipeline over a 1,000-node arena — every node accepts pointer events,
/// so each routed `Move` walks the full chain (the per-event cost the
/// batching contract removes).
fn storm_pipeline() -> PassPipeline {
    PassPipeline::new(
        Flux::new(),
        Box::new(StackLayout),
        Box::new(NullJournal),
        Box::new(NullA11y),
    )
}

fn mount_1k(pipeline: &mut PassPipeline) {
    let root = pipeline.mount_root(Box::new(FixedBox {
        size: Size::new(800.0, 0.0),
    }));
    for _ in 0..1000 {
        let _ = pipeline.add_child(
            root,
            Box::new(FixedBox {
                size: Size::new(20.0, 0.4),
            }),
        );
    }
    pipeline.tick(VIEWPORT).expect("bench layout");
}

/// A pipeline whose 1,000 children are all focusable — the walk's
/// candidate list, stacked vertically so every child lies fully below the
/// one before it (the directional route's geometry is exact: child *i + 1*
/// starts where child *i* ends).
fn mount_focusable_1k(pipeline: &mut PassPipeline) {
    let root = pipeline.mount_root(Box::new(FixedBox {
        size: Size::new(800.0, 0.0),
    }));
    for _ in 0..1000 {
        let _ = pipeline.add_child(
            root,
            Box::new(FocusBox {
                size: Size::new(20.0, 0.4),
            }),
        );
    }
    pipeline.tick(VIEWPORT).expect("bench layout");
}

fn move_at(i: usize) -> PointerEvent {
    PointerEvent::Move {
        position: Point::new(
            alacer_core::num::usize_to_f32(i % 20),
            alacer_core::num::usize_to_f32(i % 400),
        ),
    }
}

fn bench_input_storm(c: &mut Criterion) {
    let mut group = c.benchmark_group("input");
    group.throughput(Throughput::Elements(
        u64::try_from(STORM).expect("non-negative"),
    ));

    group.bench_function("storm_50_moves_queued", |b| {
        let mut pipeline = storm_pipeline();
        mount_1k(&mut pipeline);
        b.iter(|| {
            for i in 0..STORM {
                pipeline.queue_pointer(black_box(move_at(i)));
            }
            let report = pipeline.tick(VIEWPORT).expect("bench layout");
            black_box(report.passes.len());
        });
    });

    group.bench_function("storm_50_moves_direct", |b| {
        let mut pipeline = storm_pipeline();
        mount_1k(&mut pipeline);
        b.iter(|| {
            for i in 0..STORM {
                pipeline.dispatch_pointer(&black_box(move_at(i)));
            }
            let report = pipeline.tick(VIEWPORT).expect("bench layout");
            black_box(report.passes.len());
        });
    });

    // The M19 captured route (ADR-0025): a press holds its deepest
    // accepting hit, then 100 moves ride the captured chain — no hit
    // test, the ancestor walk per event. The same 1k arena the storm
    // walks: the capture's cost per move, measured.
    group.throughput(Throughput::Elements(
        u64::try_from(CAPTURED_MOVES).expect("non-negative"),
    ));
    group.bench_function("route_100_captured_moves_1k", |b| {
        let mut pipeline = storm_pipeline();
        mount_1k(&mut pipeline);
        pipeline.dispatch_pointer(&PointerEvent::Down {
            position: Point::new(10.0, 10.0),
            button: MouseButton::Left,
            click_count: 1,
        });
        assert!(pipeline.captured().is_some(), "the press holds its hit");
        b.iter(|| {
            for i in 0..CAPTURED_MOVES {
                pipeline.dispatch_pointer(&black_box(move_at(i)));
            }
            black_box(pipeline.captured().is_some());
            let report = pipeline.tick(VIEWPORT).expect("bench layout");
            black_box(report.passes.len());
        });
    });

    // The focus routes (ADR-0030/ADR-0031/ADR-0032), one measured cost per
    // move against the same 1k arena. `route_100_focus_moves_1k` is the
    // tree-order funnel the keyboard's Tab rides; `route_100_scoped_moves_1k`
    // adds the ancestor filter the scoped door's candidate list pays;
    // `route_100_directional_moves_1k` is the M26 geometry selection — the
    // bounds reads the directional walk answers with. The three joined the
    // published corpus at the M26 cut's measurement run (the coordinated
    // rule: measured, published, and gated in the same cut).
    group.throughput(Throughput::Elements(
        u64::try_from(FOCUS_MOVES).expect("non-negative"),
    ));
    group.bench_function("route_100_focus_moves_1k", |b| {
        let mut pipeline = storm_pipeline();
        mount_focusable_1k(&mut pipeline);
        b.iter(|| {
            for _ in 0..FOCUS_MOVES {
                black_box(pipeline.focus_move(black_box(FocusMove::Next)));
            }
            let report = pipeline.tick(VIEWPORT).expect("bench layout");
            black_box(report.passes.len());
        });
    });

    group.bench_function("route_100_scoped_moves_1k", |b| {
        let mut pipeline = storm_pipeline();
        mount_focusable_1k(&mut pipeline);
        let root = pipeline.arena().root().expect("the bench mounts a root");
        b.iter(|| {
            for _ in 0..FOCUS_MOVES {
                black_box(pipeline.focus_move_in(black_box(root), black_box(FocusMove::Next)));
            }
            let report = pipeline.tick(VIEWPORT).expect("bench layout");
            black_box(report.passes.len());
        });
    });

    group.bench_function("route_100_directional_moves_1k", |b| {
        let mut pipeline = storm_pipeline();
        mount_focusable_1k(&mut pipeline);
        b.iter(|| {
            // Fifty steps down the stack, fifty back up — every move
            // lands, no refusals, one route shape.
            for _ in 0..FOCUS_MOVES / 2 {
                black_box(pipeline.focus_move(black_box(FocusMove::Down)));
            }
            for _ in 0..FOCUS_MOVES / 2 {
                black_box(pipeline.focus_move(black_box(FocusMove::Up)));
            }
            let report = pipeline.tick(VIEWPORT).expect("bench layout");
            black_box(report.passes.len());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_input_storm);
criterion_main!(benches);
