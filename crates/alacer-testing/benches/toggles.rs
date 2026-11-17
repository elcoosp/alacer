//! The toggle route's tri-state honesty bench (M94, ADR-0098): the a11y
//! snapshot pass over a 1,000-node tri-state arena against the same
//! 1,000-node binary arena — the same roles, the same names, the same
//! trees, the same route through the real pipeline (A4). The published
//! `toggle_tri_state_overhead` ratio budget keeps the summary honest:
//! the mixed state rides the same frame the binary toggle rides, and the
//! dash the painter draws is the only difference the pipeline prices —
//! the tri-state shortens nothing, costs nothing the bit does not.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Checkbox, Column};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node toggle tree under a column root: every node a
/// checkbox carrying its name and its state — `tri_state` sprinkles the
/// mixed summary through every third control (the select-all canon's
/// shape), `binary` keeps the bool pair — laid out and settled before
/// the clock.
fn mount_tree(harness: &mut TestHarness, tri_state: bool) -> NodeId {
    let root = harness.mount(Column::new().gap(4.0).boxed());
    for index in 0..NODES {
        let mut checkbox = Checkbox::new(format!("toggle {index}"));
        checkbox = if tri_state && index % 3 == 0 {
            checkbox.state(alacer_core::widget::ToggleState::Mixed)
        } else {
            checkbox.checked(tri_state && index % 3 == 1)
        };
        let _ = harness.add_child(root, checkbox.boxed());
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

fn bench_toggle_tri_state_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("toggle");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_tri_state", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, true);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.bench_function("snapshot_1k_binary", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, false);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_toggle_tri_state_overhead);
criterion_main!(benches);
