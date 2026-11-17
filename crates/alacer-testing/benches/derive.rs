//! The derive route's zero-overhead bench (M92, ADR-0096): the a11y
//! snapshot pass over a 1,000-node derived arena against the same
//! 1,000-node hand-written arena — the same roles, the same names, the
//! same trees, the same route through the real pipeline (A4). The
//! published `derive_zero_overhead` ratio budget keeps the derive
//! honest: the generated impl *is* the hand-written impl, and the
//! frame the pipeline produces must cost the same — the derive
//! shortens the hand, never the contract.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_macro::Widget;
use alacer_testing::TestHarness;
use alacer_widgets::Column;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// The derived box: the required pair generated from two attributes —
/// the measured surface is the pipeline's frame over the generated
/// impls.
#[derive(Widget)]
#[widget(role = "container", label = "the derived box")]
struct DerivedBox;

/// The hand-written box: the same pair, typed by hand — the baseline
/// the ratio budget prices the derive against.
struct HandwrittenBox;

impl Widget for HandwrittenBox {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("the hand-written box")
    }
}

/// Mounts a 1,000-node tree under a column root: every node the same
/// widget kind (the derived arena or the hand-written one), laid out
/// and settled before the clock.
fn mount_tree(harness: &mut TestHarness, derived: bool) -> NodeId {
    let root = harness.mount(Column::new().boxed());
    for _ in 0..NODES {
        let widget: Box<dyn Widget> = if derived {
            Box::new(DerivedBox)
        } else {
            Box::new(HandwrittenBox)
        };
        let _ = harness.add_child(root, widget);
    }
    harness.settle();
    root
}

/// One measured frame: the root marked for the a11y pass, the tick
/// rebuilding the snapshot over the whole arena — the pass the AT's
/// tree rides, the same route both arenas travel.
fn snapshot_frame(harness: &mut TestHarness, root: NodeId) {
    harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(root, Change::A11Y);
    black_box(harness.tick());
}

fn bench_derive_zero_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("derive");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_derived", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, true);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.bench_function("snapshot_1k_handwritten", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, false);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_derive_zero_overhead);
criterion_main!(benches);
