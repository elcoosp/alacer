//! The auto-complete route's snapshot bench (M111, ADR-0115): the a11y
//! snapshot pass over a 1,000-node input arena — every node a field
//! whose constructor declared the suggestion contract (`auto_complete`
//! alternating the kind down the column, the contract the AT reads
//! beside the hint the placeholder tier carried) — against the same
//! 1,000-node input arena bare (`placeholder/snapshot_1k_bare`, the M96
//! bare case; the M95 pricing rule — the word is a field word, so the
//! honest comparison is fields against fields, the M106/M107
//! denominator). The published `auto_complete_snapshot_overhead` ratio
//! budget (≤ 1.25×) keeps the word honest: the word rides the snapshot
//! the field rides, the tier's price the one `Option` the node carries
//! beside it — the same-length allocation either way, the word the only
//! difference. The word's semantics (the doors answering unchanged
//! beside it) are pinned functionally by the m111 suite; nothing else
//! in the frame pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_core::widget::AutoCompleteKind;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// Mounts a 1,000-node input tree under a column root: every node a
/// named text input whose constructor declared the suggestion contract
/// — the word the tree says, the completion modes cycling the kind the
/// AT reads — laid out and settled before the clock.
fn mount_tree(harness: &mut TestHarness) -> NodeId {
    let root = harness.mount(Column::new().gap(4.0).boxed());
    let kinds = [
        AutoCompleteKind::Inline,
        AutoCompleteKind::List,
        AutoCompleteKind::Both,
    ];
    for index in 0..NODES {
        let input =
            TextInput::new(format!("field {index}")).auto_complete(kinds[index % kinds.len()]);
        let _ = harness.add_child(root, input.boxed());
    }
    harness.settle();
    root
}

/// One measured frame: the root marked for the a11y pass, the tick
/// rebuilding the snapshot over the whole arena — the pass the AT's
/// tree rides, the same route the bare arena travels.
fn snapshot_frame(harness: &mut TestHarness, root: NodeId) {
    harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(root, Change::A11Y);
    black_box(harness.tick());
}

fn bench_auto_complete_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("auto_complete");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_auto_complete", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_auto_complete_snapshot_overhead);
criterion_main!(benches);
