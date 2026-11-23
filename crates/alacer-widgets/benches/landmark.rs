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

/// The vocabulary case (M144, ADR-0148): the growth's price — the eight
/// constructor words cycled through the same moving-name write, the
/// pool built one region per constructor (round-robin) and renamed the
/// way `name_write_256` renames. The word is the constructor's own
/// declaration; the write's price is the word it lands under — the case
/// proves the vocabulary's growth prices the words and nothing else
/// (the M143 rule, at the growth tier). Rides outside the published
/// corpus with the pair above (the coordinated rule grows the baseline
/// at the next published cut).
fn bench_words_write(c: &mut Criterion) {
    const WORDS: usize = 8;
    let mut group = c.benchmark_group("landmark");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("words_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| match index % WORDS {
                        0 => Landmark::banner(format!("Region {index}")),
                        1 => Landmark::navigation(format!("Region {index}")),
                        2 => Landmark::main(format!("Region {index}")),
                        3 => Landmark::content_info(format!("Region {index}")),
                        4 => Landmark::complementary(format!("Region {index}")),
                        5 => Landmark::form(format!("Region {index}")),
                        6 => Landmark::region(format!("Region {index}")),
                        _ => Landmark::search(format!("Region {index}")),
                    })
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

criterion_group!(
    benches,
    bench_name_write,
    bench_same_name,
    bench_words_write
);
criterion_main!(benches);
