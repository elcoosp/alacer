//! The landmark tier's word-write benchmark (M143, ADR-0147): the
//! widget crate's own pricing door. The published corpus prices the
//! a11y snapshot machinery (the M95 rule) — and the landmark tier grows
//! no snapshot route, the words being the containers' own, the shape
//! the M124/M127/M129/M131/M134/M135/M136 named containers rode — so
//! the tier's cost is priced here, at the widget door the word writes
//! ride: the moving name (the write the furniture's word takes when
//! the app renames it) against the unchanged name (the write that
//! lands where the word already is — the honest floor of a `String`'s
//! write at the widget seam).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m143 suite's pin documents the same shape).

use alacer_widgets::Landmark;
use criterion::{Criterion, criterion_group, criterion_main};

/// The region pool: the regions the route cycles through, fresh per
/// case.
const POOL: usize = 256;

/// The moving name: every write lands on a region whose word is
/// different — the `String`'s write the rename takes.
fn bench_name_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("landmark");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("name_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Landmark::main(format!("Region {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, region) in pool.iter_mut().enumerate() {
                    region.set_name(format!("Renamed {index}"));
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// The unchanged name: every write lands where the word already is —
/// the write's honest floor (the same `String`'s write, no move).
fn bench_same_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("landmark");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Landmark::main(format!("Region {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, region) in pool.iter_mut().enumerate() {
                    region.set_name(format!("Region {index}"));
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_name_write, bench_same_name);
criterion_main!(benches);
