//! The click chain's price (M103, ADR-0107): one press routed through
//! [`alacer_platform::ClickCounter::on_down`], the window priced by the
//! canon [`alacer_core::time::MonotonicClock`] against the seam's
//! dispatch floor.
//!
//! Two cases, one subject: `seamed_chain_1k` prices the shipped default
//! (the canon clock's real read per press — the vDSO syscall is the
//! chain's own cost), `manual_chain_1k` prices the same storm through
//! the deterministic [`alacer_core::time::ManualClock`] — no clock
//! syscall, the seam indirection's floor. The pair's ratio is the clock
//! read's own price, not the seam's, so no ratio budget rides it: both
//! cases publish regression gates, the per-case medians the perf
//! contracts hold (ADR-0011).

use alacer_core::geom::Point;
use alacer_core::time::{ManualClock, MonotonicClock};
use alacer_platform::ClickCounter;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

/// The storm's shape: 1,000 chained presses, each inside the slop (a
/// 1 px walk) — the chain counts to u8 saturation and keeps going, the
/// per-press price the subject.
const PRESSES: u32 = 1_000;

fn chain_positions() -> Vec<Point> {
    (0..PRESSES)
        .map(|i| {
            Point::new(
                10.0 + f32::from((i % 3) as u8),
                10.0 + f32::from((i % 2) as u8),
            )
        })
        .collect()
}

fn bench_click_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("click_chain");
    group.throughput(Throughput::Elements(u64::from(PRESSES)));

    let positions = chain_positions();
    group.bench_function("seamed_chain_1k", |b| {
        b.iter(|| {
            let mut clicks = ClickCounter::with_clock(Box::new(MonotonicClock::new()));
            for position in &positions {
                black_box(clicks.on_down(black_box(*position)));
            }
        });
    });

    group.bench_function("manual_chain_1k", |b| {
        b.iter(|| {
            let mut clicks = ClickCounter::with_clock(Box::new(ManualClock::new()));
            for position in &positions {
                black_box(clicks.on_down(black_box(*position)));
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_click_chain);
criterion_main!(benches);
