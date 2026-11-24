//! Pointer-cursor benchmarks (§10.1, ADR-0156; the trail since M153,
//! ADR-0157): the hover resolution's walk is the frame-tier cost a
//! cursor-carrying app pays every frame — the platform resolves the
//! pointer's style per frame and applies the delta. The disabled half
//! costs nothing (the door is a pure read — no pass, no trail, no
//! observer); the walk over a 200-node arena is the price under test:
//! the pointer route's own pre-order bounds sweep
//! (`cursor_at_deepest_hit_200`, the styled leaf naming the answer)
//! against the void's answer (`cursor_at_void_200`, nothing accepting —
//! the default the empty chain returns). The geometry is staged zero
//! Taffy (the `StackLayout` stand-in the input bench rides, ADR-0014's
//! own shape): absolute bounds written back, so the walk is the cost
//! under test, never the layout. Since M153 the trail's two prices
//! join the corpus (ADR-0157): `trail_note_on_change_200` — the note's
//! cost under the observer it serves, every resolution a real change
//! the tick's start drain streams — and `trail_note_no_change_200` —
//! the compare the idle-frame property owes a still hover (a style
//! that did not move is not a fact, the flood rule at the cursor
//! tier).

use alacer_core::arena::NodeId;
use alacer_core::cursor::CursorStyle;
use alacer_core::flux::Flux;
use alacer_core::geom::{Point, Size};
use alacer_core::mask::Change;
use alacer_core::passes::{A11yDelta, A11ySink, PassPipeline, SceneJournal, TreeSnapshot};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::style::MeasureSpec;
use alacer_core::widget::{LabelSource, Role, Widget};
use criterion::{Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;
use std::hint::black_box;

const VIEWPORT: Size = Size::new(800.0, 600.0);
const ROWS: usize = 200;
const ROW_HEIGHT: f32 = 0.4;

/// A fixed-size leaf: geometry without text, so the hover walk is the
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

/// The styled leaf: the last row names the pointer — the deepest hit the
/// hover walk answers with (the link's word, staged on a geometry leaf).
struct StyledBox {
    size: Size,
}

impl Widget for StyledBox {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Decoration
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("styled")
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(self.size))
    }

    fn cursor_style(&self) -> CursorStyle {
        CursorStyle::Pointer
    }
}

/// Zero-Taffy geometry: a vertical stack of the declared sizes — the
/// absolute bounds the hover walk reads (the input bench's own
/// stand-in, ADR-0014's staged shape).
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

/// A pipeline over a 200-row arena — the root plus the rows stacked
/// vertically, the last row styled (the deepest hit a hover over the
/// column's face resolves).
fn cursor_pipeline() -> PassPipeline {
    PassPipeline::new(
        Flux::new(),
        Box::new(StackLayout),
        Box::new(NullJournal),
        Box::new(NullA11y),
    )
}

fn mount_rows(pipeline: &mut PassPipeline) {
    let root = pipeline.mount_root(Box::new(FixedBox {
        size: Size::new(800.0, 0.0),
    }));
    let rows = ROWS.saturating_sub(1);
    for i in 0..rows {
        let row: Box<dyn Widget> = if i == rows - 1 {
            Box::new(StyledBox {
                size: Size::new(20.0, ROW_HEIGHT),
            })
        } else {
            Box::new(FixedBox {
                size: Size::new(20.0, ROW_HEIGHT),
            })
        };
        let _ = pipeline.add_child(root, row);
    }
    pipeline.tick(VIEWPORT).expect("bench layout");
}

fn bench_cursor(c: &mut Criterion) {
    let mut group = c.benchmark_group("cursor");

    // The hover's answer over the styled leaf: the full pre-order walk
    // (every row probed, the styled deepest hit named) beside the style
    // read — the per-frame price a cursor-carrying app pays while the
    // pointer rests on the styled face.
    group.bench_function("cursor_at_deepest_hit_200", |b| {
        let mut pipeline = cursor_pipeline();
        mount_rows(&mut pipeline);
        let point = Point::new(
            10.0,
            alacer_core::num::usize_to_f32(ROWS) * ROW_HEIGHT - 0.1,
        );
        b.iter(|| {
            black_box(pipeline.cursor_at(black_box(point)));
        });
    });

    // The hover's answer over the void: the same walk, nothing
    // accepting — the default the empty chain returns (the walk is the
    // whole cost; the answer is the absence).
    group.bench_function("cursor_at_void_200", |b| {
        let mut pipeline = cursor_pipeline();
        mount_rows(&mut pipeline);
        let point = Point::new(400.0, 400.0);
        b.iter(|| {
            black_box(pipeline.cursor_at(black_box(point)));
        });
    });

    // The trail's live note, read end to end (ADR-0157): the observer
    // installed, every resolution a real change (the hover alternating
    // between the styled leaf and the void), the tick's start drain
    // streaming the fact into the trail reader — the note's cost under
    // the observer it serves, the settings door's own bench shape.
    group.bench_function("trail_note_on_change_200", |b| {
        let mut pipeline = cursor_pipeline();
        mount_rows(&mut pipeline);
        let reader = alacer_core::trail::TrailLog::new();
        pipeline.set_observer(Box::new(reader.clone()));
        let over = Point::new(
            10.0,
            alacer_core::num::usize_to_f32(ROWS) * ROW_HEIGHT - 0.1,
        );
        let void = Point::new(400.0, 400.0);
        let mut on_leaf = false;
        b.iter(|| {
            on_leaf = !on_leaf;
            let point = if on_leaf { over } else { void };
            black_box(pipeline.cursor_at(black_box(point)));
            pipeline
                .tick(VIEWPORT)
                .expect("the trail bench's frames lay out");
        });
    });

    // The trail's still price (ADR-0157): the observer installed, the
    // hover resting on the styled leaf — every resolution compares
    // against the last noted answer and queues nothing (a style that
    // did not move is not a fact), the compare the whole cost the
    // idle-frame property owes the tier.
    group.bench_function("trail_note_no_change_200", |b| {
        let mut pipeline = cursor_pipeline();
        mount_rows(&mut pipeline);
        let reader = alacer_core::trail::TrailLog::new();
        pipeline.set_observer(Box::new(reader.clone()));
        let over = Point::new(
            10.0,
            alacer_core::num::usize_to_f32(ROWS) * ROW_HEIGHT - 0.1,
        );
        let _ = pipeline.cursor_at(over);
        b.iter(|| {
            black_box(pipeline.cursor_at(black_box(over)));
            pipeline
                .tick(VIEWPORT)
                .expect("the trail bench's frames lay out");
        });
    });
}

criterion_group!(benches, bench_cursor);
criterion_main!(benches);
