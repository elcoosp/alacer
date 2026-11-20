//! The meter tier's value-write benchmark (M133, ADR-0137): the widget
//! crate's own pricing door. The published corpus prices the a11y
//! snapshot machinery (the M95 rule) — and the meter tier grows no
//! snapshot route, the gauge riding the role branch the sink maps and
//! the value words M3/M97/M99 already carry — so the tier's cost is
//! priced here, at the widget door the reading writes ride: the
//! moving write (the write the gauge's reading takes when the app
//! moves it) against the unchanged write (the write that lands where
//! the reading already is — the honest floor of a reported value's
//! write at the widget seam, the M21 idle-frame property priced).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m133 suite's pin documents the same shape).

use alacer_widgets::Meter;
use criterion::{Criterion, criterion_group, criterion_main};

/// The gauge pool: the gauges the route cycles through, fresh per case.
const POOL: usize = 256;

/// The moving write: every write lands on a gauge reporting nothing
/// and reports the mid-scale reading now — the move the reading takes.
fn bench_value_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("meter");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("value_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Meter::new(format!("Gauge {index}"), 0.0, 100.0))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for gauge in &mut pool {
                    gauge.set_value(62.0);
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// The unchanged write: every write lands where the reading already
/// is — the write's honest floor (the M21 idle path priced, the same
/// reported value's write, no move).
fn bench_idle_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("meter");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Meter::new(format!("Gauge {index}"), 0.0, 100.0).value(62.0))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for gauge in &mut pool {
                    gauge.set_value(62.0);
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_value_write, bench_idle_write);
criterion_main!(benches);
