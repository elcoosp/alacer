//! The W5 workload (§14.2, ADR-0012): signal-graph churn — 10k signals, 1k
//! memos, batched writes.
//!
//! The gate is the scheduling contract at graph scale (ADR-6): one write
//! batch ⇒ at most one rerun per batch scope, memos never glitch, and the
//! mask drain stays linear in dirty nodes. The one-rerun-per-batch and
//! no-glitch properties are test-enforced (`alacer-testing/tests/
//! m6_exit_criteria.rs`); this bench measures the batch drain's cost.

use alacer_core::arena::NodeId;
use alacer_core::flux::Flux;
use alacer_core::mask::Change;
use alacer_core::{Get, GetUntracked as _, Set as _};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const SIGNALS: usize = 10_000;
const MEMOS: usize = 1_000;
/// Writes per frame — 1% of the graph churns per batch.
const WRITES_PER_FRAME: usize = 100;

/// The graph: 10k signals; memo `i` folds signals `i*10..i*10+10` (each
/// memo tracks 10 sources — 1k memos over the same graph, no glitches).
struct Graph {
    flux: Flux,
    signals: Vec<alacer_core::FluxRwSignal<i64>>,
    /// The memos, read by the churn loop to pin no-glitch consistency.
    memos: Vec<reactive_graph::computed::Memo<i64>>,
}

fn build_graph() -> Graph {
    let flux = Flux::new();
    let signals: Vec<alacer_core::FluxRwSignal<i64>> = (0..SIGNALS)
        .map(|i| flux.signal(i64::from(u32::try_from(i).unwrap_or(u32::MAX))))
        .collect();
    let memos = (0..MEMOS)
        .map(|i| {
            let a = signals[(i * 10) % SIGNALS];
            let b = signals[(i * 10 + 3) % SIGNALS];
            let c = signals[(i * 10 + 7) % SIGNALS];
            flux.memo(move |_| a.get() + b.get() + c.get())
        })
        .collect();
    Graph {
        flux,
        signals,
        memos,
    }
}

fn bench_w5(c: &mut Criterion) {
    let mut group = c.benchmark_group("w5");
    group.throughput(Throughput::Elements(
        u64::try_from(WRITES_PER_FRAME).expect("fits u64"),
    ));

    // One frame = one batch of 100 signal writes across the graph, then the
    // mask drain. Memos recompute inside the batch scope (coalesced).
    group.bench_function("batch_churn_100_of_10k", |b| {
        let mut graph = build_graph();
        // Bind the drains to nodes so the frame's masks have somewhere to
        // land (the pipeline's pass 4 consumes them in production).
        let mut flux = std::mem::take(&mut graph.flux);
        for signal in graph.signals.iter().take(WRITES_PER_FRAME) {
            flux.bind(*signal, NodeId::INVALID, |_: &i64| Change::PAINT);
        }
        let mut frame = 0_usize;
        b.iter(|| {
            frame += 1;
            let base = i64::try_from(frame).unwrap_or(i64::MAX);
            flux.batch(|| {
                for (index, signal) in graph.signals.iter().enumerate().take(WRITES_PER_FRAME) {
                    signal.set(base + i64::try_from(index).unwrap_or(i64::MAX));
                }
                // The memos read consistently inside the batch: no glitching
                // (the mid-batch value is either all-old or all-new).
                let sum = graph.memos[0].get() + graph.memos[1].get();
                black_box(sum);
            });
            let drained = flux.take_pending_masks();
            black_box(drained.len());
        });
        graph.flux = flux;
    });

    // The memo's first-read cost: a cold memo over 10 sources.
    group.bench_function("memo_cold_10_sources", |b| {
        b.iter_batched(
            || {
                let flux = Flux::new();
                let signals: Vec<alacer_core::FluxRwSignal<i64>> =
                    (0..10).map(|i| flux.signal(i64::from(i))).collect();
                let cloned: Vec<alacer_core::FluxRwSignal<i64>> = signals.clone();
                let memo =
                    flux.memo(move |_| cloned.iter().map(|signal| signal.get()).sum::<i64>());
                (flux, memo)
            },
            |(_flux, memo)| black_box(memo.get_untracked()),
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_w5);
criterion_main!(benches);
