//! The heading tier's level-write benchmark (M126, ADR-0130): the
//! widget crate's own pricing door. The published corpus prices the
//! a11y snapshot machinery (the M95 rule) — and the heading tier grows
//! no snapshot route, the role being the fact and the depth riding the
//! level word the M113 tier published — so the tier's cost is priced
//! here, at the widget door the level write rides: the moving write
//! (the compare, the state write, the moved answer) against the idle
//! write (the M21 idle path — the write that lands where the depth
//! already is, answering `false` and queueing nothing).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m126 suite's pin documents the same shape).

use alacer_widgets::Heading;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

/// The write pool: the headings the route cycles through, fresh per
/// case so every write is a moved one.
const POOL: usize = 256;

/// The moving write: every write lands on a heading whose depth is not
/// the one written — the compare, the state write, the `true` answer.
fn bench_level_write_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("heading");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("level_write_route_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Heading::new(format!("Section {index}")).level(index))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, heading) in pool.iter_mut().enumerate() {
                    black_box(heading.set_level(Some(index + 1)));
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// The idle write: every write lands where the depth already is — the
/// `false` answer, the M21 idle path priced (the level write route's
/// honest floor).
fn bench_idle_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("heading");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Heading::new(format!("Section {index}")).level(index))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, heading) in pool.iter_mut().enumerate() {
                    black_box(heading.set_level(Some(index)));
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_level_write_route, bench_idle_write);
criterion_main!(benches);
