//! The dialog tier's name-write benchmark (M134, ADR-0138): the
//! widget crate's own pricing door. The published corpus prices the
//! a11y snapshot machinery (the M95 rule) — and the dialog tier grows
//! no snapshot route, the word being the container's own (the shape
//! the M124/M127/M129/M131 named containers rode) — so the tier's cost
//! is priced here, at the widget door the window word's writes ride:
//! the moving rename (the write the window's word takes when the app
//! re-titles the conversation) against the unchanged name (the write
//! that lands where the word already is — the honest floor of a
//! `String`'s write at the widget seam, the M21 idle-frame property
//! priced).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m134 suite's pin documents the same shape).

use alacer_widgets::Dialog;
use criterion::{Criterion, criterion_group, criterion_main};

/// The window pool: the windows the route cycles through, fresh per case.
const POOL: usize = 256;

/// The moving rename: every write lands on a window named one thing
/// and names it another — the move the window word takes.
fn bench_name_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("dialog");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("name_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Dialog::new(format!("Confirm {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, window) in pool.iter_mut().enumerate() {
                    window.set_name(format!("Confirmed {index}"));
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
    let mut group = c.benchmark_group("dialog");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Dialog::new(format!("Confirm {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, window) in pool.iter_mut().enumerate() {
                    window.set_name(format!("Confirm {index}"));
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
