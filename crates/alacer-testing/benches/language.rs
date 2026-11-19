//! The language route's honesty bench (M123, ADR-0127): the a11y
//! snapshot pass over a 1,000-node label arena carrying its language
//! word against the same 1,000-node arena bare — the same roles, the
//! same names, the same trees, the same route through the real pipeline
//! (A4). The tier's own honest pair (the M119 shape; the M95 pricing
//! rule — the word is a label word, so the honest comparison is labels
//! against labels). The published `language_snapshot_overhead` ratio
//! budget (≤ 1.25×) keeps the word honest: the tag rides the snapshot
//! the name rides, the tier's price the second bounded copy the node
//! builds beside the label's own — nothing else in the frame pays for
//! the tree's truth. The word's semantics (the doors answering unchanged
//! beside it) are pinned functionally by the m123 suite.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// The tags the worded arena cycles — the BCP 47 spellings the ATs
/// voice, one per flavor the contract speaks, so the snapshot's carried
/// word is every shape the app names, not one.
const TAGS: [&str; 4] = ["fr-FR", "ja-JP", "de-DE", "pt-BR"];

/// Mounts a 1,000-node label tree under a column root: every node a
/// label; `worded` gives each its language tag — the word the AT voices
/// the text with — laid out and settled before the clock.
fn mount_tree(harness: &mut TestHarness, worded: bool) -> NodeId {
    let root = harness.mount(Column::new().gap(2.0).boxed());
    for index in 0..NODES {
        let mut label = Label::new(format!("Item {index}"));
        if worded {
            label = label.language(TAGS[index % TAGS.len()]);
        }
        let _ = harness.add_child(root, label.boxed());
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

fn bench_language_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("language");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_labels_language", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, true);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.bench_function("snapshot_1k_labels_bare", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, false);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_language_snapshot_overhead);
criterion_main!(benches);
