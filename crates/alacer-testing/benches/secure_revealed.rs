//! The reveal route's snapshot bench (M102, ADR-0106): the a11y
//! snapshot pass over a 1,000-node revealed field arena — every node a
//! secured field whose revealed word holds, the tree resolving the
//! ordinary editable answer and the value's raw shape — against the
//! same 1,000-node secured arena masked (`secure/snapshot_1k_secured`,
//! the M100 case). The published `secure_revealed_snapshot_overhead`
//! ratio budget (≤ 1.25×) keeps the word honest: the flag rides the
//! snapshot the secure word rides, the tier's price the branch the
//! role mapping and the mask rule answer beside the M100 copy's own —
//! the same-length allocation either way. The reveal's tracking
//! behavior (the mask yielding to the word, the role yielding with it)
//! is pinned functionally by the m102 suite and the sink's unit test;
//! nothing else in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node revealed field arena under a column root: every
/// node a named field whose constructor declared secret entry and
/// mounted revealed — the word the tree says, the value the tree
/// resolves raw — laid out and settled before the clock.
fn mount_tree(harness: &mut TestHarness) -> NodeId {
    let root = harness.mount(Column::new().gap(4.0).boxed());
    for index in 0..NODES {
        let input = TextInput::new(format!("field {index}")).secure().revealed();
        let _ = harness.add_child(root, input.boxed());
    }
    harness.settle();
    root
}

/// One measured frame: the root marked for the a11y pass, the tick
/// rebuilding the snapshot over the whole arena — the pass the AT's
/// tree rides, the same route the masked arena travels.
fn snapshot_frame(harness: &mut TestHarness, root: NodeId) {
    harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(root, Change::A11Y);
    black_box(harness.tick());
}

fn bench_secure_revealed_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("secure_revealed");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_revealed", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_secure_revealed_snapshot_overhead);
criterion_main!(benches);
