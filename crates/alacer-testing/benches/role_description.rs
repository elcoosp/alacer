//! The role-description route's snapshot bench (M116, ADR-0120): the
//! a11y snapshot pass over a 1,000-node button arena — every node a
//! button whose constructor declared the read-as word
//! (`role_description` cycling the custom-control specializations down
//! the toolbar, the role the AT reads the control as) — against the
//! same 1,000-node button arena bare
//! (`current/snapshot_1k_buttons_bare`, the M114 case the M111
//! precedent reuses; the M95 pricing rule — the word is a button word,
//! so the honest comparison is buttons against buttons, the M108/M110
//! denominator shape). The published
//! `role_description_snapshot_overhead` ratio budget (≤ 1.25×) keeps
//! the word honest: the word rides the snapshot the button rides, the
//! tier's price the one `Option<String>` the node carries beside it —
//! the same-shaped node either way, the word the only difference. The
//! word's semantics (the doors answering unchanged beside it) are
//! pinned functionally by the m116 suite; nothing else in the frame
//! pays for the tree's truth.

use alacer_core::Change;
use alacer_core::arena::NodeId;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const NODES: usize = 1_000;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

/// The words the worded arena cycles — the custom-control
/// specializations the tier exists to say ("this slider is a star
/// rating", "that button is a wizard page"), so the snapshot's carried
/// word is every shape the contract speaks, not one.
const WORDS: [&str; 5] = [
    "star rating",
    "wizard page",
    "checklist step",
    "traffic light",
    "card field",
];

/// Mounts a 1,000-node button tree under a column root: every node a
/// button whose constructor declared the read-as word — the word the
/// tree says, the specialization the AT reads beside the generic role —
/// laid out and settled before the clock. `worded` mounts the bare
/// arena the pair prices the word against (the same names the M114 bare
/// arena carries, the word the only difference).
fn mount_tree(harness: &mut TestHarness, worded: bool) -> NodeId {
    let root = harness.mount(Column::new().gap(2.0).boxed());
    for index in 0..NODES {
        let mut button = Button::new(format!("Item {index}"));
        if worded {
            button = button.role_description(WORDS[index % WORDS.len()]);
        }
        let _ = harness.add_child(root, button.boxed());
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

fn bench_role_description_snapshot_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("role_description");
    group.throughput(Throughput::Elements(
        u64::try_from(NODES).expect("non-negative"),
    ));

    group.bench_function("snapshot_1k_role_descriptions", |b| {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let root = mount_tree(&mut harness, true);
        b.iter(|| snapshot_frame(&mut harness, root));
    });

    group.finish();
}

criterion_group!(benches, bench_role_description_snapshot_overhead);
criterion_main!(benches);
