//! The menu tier's word-write benchmark (M137, ADR-0141): the widget
//! crate's own pricing door. The published corpus prices the a11y
//! snapshot machinery (the M95 rule) — and the menu tier grows no
//! snapshot route, the words being the containers' and the entry's own,
//! the shape the M124/M127/M129/M131/M134/M135/M136 named containers
//! rode — so the tier's cost is priced here, at the widget door the
//! word writes ride: the moving name (the write the strip's word takes
//! when the app renames it) against the unchanged name (the write that
//! lands where the word already is — the honest floor of a `String`'s
//! write at the widget seam).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m137 suite's pin documents the same shape).

use alacer_widgets::MenuBar;
use criterion::{Criterion, criterion_group, criterion_main};

/// The strip pool: the strips the route cycles through, fresh per
/// case.
const POOL: usize = 256;

/// The moving name: every write lands on a strip whose word is
/// different — the `String`'s write the rename takes.
fn bench_name_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("menu_bar");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("name_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| MenuBar::new(format!("Bar {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, strip) in pool.iter_mut().enumerate() {
                    strip.set_name(format!("Renamed {index}"));
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
    let mut group = c.benchmark_group("menu_bar");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| MenuBar::new(format!("Bar {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, strip) in pool.iter_mut().enumerate() {
                    strip.set_name(format!("Bar {index}"));
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
