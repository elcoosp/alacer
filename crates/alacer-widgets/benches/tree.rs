//! The tree tier's name-write benchmark (M135, ADR-0139): the widget
//! crate's own pricing door. The published corpus prices the a11y
//! snapshot machinery (the M95 rule) — and the tree tier grows no
//! snapshot route, the words being the container's and member's own
//! (the shape the M124/M127/M129/M131/M134 named containers rode) — so
//! the tier's cost is priced here, at the widget door the member word's
//! writes ride: the moving rename (the write a tree item's word takes
//! when the app re-titles the member) against the unchanged name (the
//! write that lands where the word already is — the honest floor of a
//! `String`'s write at the widget seam, the M21 idle-frame property
//! priced).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m135 suite's pin documents the same shape).

use alacer_widgets::TreeItem;
use criterion::{Criterion, criterion_group, criterion_main};

/// The member pool: the members the route cycles through, fresh per case.
const POOL: usize = 256;

/// The moving rename: every write lands on a member named one thing
/// and names it another — the move the member word takes.
fn bench_name_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("name_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| TreeItem::new(format!("item-{index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, member) in pool.iter_mut().enumerate() {
                    member.set_name(format!("renamed-{index}"));
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
fn bench_idle_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| TreeItem::new(format!("item-{index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, member) in pool.iter_mut().enumerate() {
                    member.set_name(format!("item-{index}"));
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_name_write, bench_idle_write);
criterion_main!(benches);
