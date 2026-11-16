//! The allocation ledger benchmark (M84, ADR-0088): the heap budget's
//! own corpus — the warm tick's zero-allocation answer driven through
//! the real pipeline on the ledger's static UI, the observed tick's
//! priced record beside it, and the ledger's own costs (the hook's
//! alloc/dealloc wrapper, the snapshot read) priced beside the frame.
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness
//! layer drives every surface), and because the budget's checker is
//! harness vocabulary ([`alacer_testing::alloc`]): the binary installs
//! the adopted hook ([`Alloc`] — dhat's own allocator, A2) and arms one
//! process-wide ledger for the run, so every case's time is the honest
//! tracking-included number. The published regression gate keeps the
//! budget honest as the capability grows: the warm tick's time rides
//! the ledger's own overhead, and the cases' medians are the corpus's
//! record of what the checker costs (ADR-0011's publish-or-fail rule).

use alacer_testing::TestHarness;
use alacer_testing::alloc::{Alloc, AllocLedger};
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, TextInput};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// The adopted hook (A2): this binary's allocator is the tracking one —
// without it the ledger's counters never move and the assertions below
// would pass vacuously.
#[global_allocator]
static LEDGER: Alloc = Alloc;

/// The ledger's static UI — the m84 suite's own subject, mounted once:
/// a column, a button, a field, six listed labels. Nothing dirty after
/// the settle; the warm ticks that follow are the budget's subject.
fn mount_static_ui(harness: &mut TestHarness) {
    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());
    let _title = harness.add_child(root, Label::new("The allocation ledger").boxed());
    let _action = harness.add_child(root, Button::new("The ledger's button").boxed());
    let _field = harness.add_child(root, TextInput::new("The ledger's field").boxed());
    for row in 0..6 {
        let _ = harness.add_child(
            root,
            Label::new(format!("Ledger row {row}, mounted once")).boxed(),
        );
    }
}

fn bench_alloc_budget(c: &mut Criterion) {
    // One process-wide ledger: the hook tracks every case, the numbers
    // are the tracking-included truth, and one profiler at a time is
    // dhat's own rule. The leaked guard is the window — the bench
    // process's whole life is the measured run.
    let ledger: &'static AllocLedger = Box::leak(Box::new(AllocLedger::arm()));
    let mut group = c.benchmark_group("alloc");

    group.bench_function("warm_frame_zero_alloc", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        mount_static_ui(&mut harness);
        harness.settle();
        for _ in 0..3 {
            let _ = harness.tick();
        }

        // Honesty before the clock: the beat the budget names is clean
        // before it is timed — a warm tick that allocated anything is a
        // broken budget, never a slow-but-legal frame.
        let (_, delta) = ledger.measure(|| harness.tick());
        assert!(
            delta.is_clean(),
            "the warm tick allocated {delta:?} — the A7 budget is broken"
        );

        b.iter(|| black_box(harness.tick()));
    });

    group.bench_function("warm_frame_observed", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        mount_static_ui(&mut harness);
        harness.settle();
        for _ in 0..3 {
            let _ = harness.tick();
        }
        let trail = harness.trail_with_cap(8);
        for _ in 0..8 {
            let _ = harness.tick();
        }

        // Honesty before the clock: the observed tick's ring sits at its
        // cap, so the record it allocates evicts the record it replaces
        // — the priced, balanced churn the trail's bounded rule owns.
        let (_, delta) = ledger.measure(|| harness.tick());
        assert!(
            delta.is_balanced() && delta.allocations <= 2 && delta.net_bytes() == 0,
            "the observed warm tick's record is priced differently now: {delta:?}"
        );

        b.iter(|| black_box(harness.tick()));
        let _ = trail.dropped();
    });

    group.bench_function("ledger_overhead", |b| {
        // The hook's own wrapper cost: one 64-byte block made and freed
        // per iteration through the tracking allocator — the price every
        // armed beat pays per allocation, priced alone.
        b.iter(|| black_box(Box::new([0_u8; 64])));
    });

    group.bench_function("snapshot_get", |b| {
        // The snapshot read: the counters the beats' deltas bracket, one
        // lock-and-read per call — the cost a budget probe pays per
        // snapshot, priced alone.
        b.iter(|| black_box(ledger.stats()));
    });

    group.finish();
}

criterion_group!(benches, bench_alloc_budget);
criterion_main!(benches);
