//! The tab-panel tier's word-write benchmark (M131, ADR-0135): the
//! widget crate's own pricing door. The published corpus prices the
//! a11y snapshot machinery (the M95 rule) — and the tab-panel tier
//! grows no snapshot route, the word being the container's own, the
//! shape the M124/M127/M129 named containers rode — so the tier's cost
//! is priced here, at the widget door the word writes ride: the moving
//! name (the write the pane's word takes when the app renames it) —
//! against the unchanged name (the write that lands where the word
//! already is — the honest floor of a `String`'s write at the widget
//! seam).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m131 suite's pin documents the same shape).

use alacer_widgets::TabPanel;
use criterion::{Criterion, criterion_group, criterion_main};

/// The pane pool: the panes the route cycles through, fresh per case.
const POOL: usize = 256;

/// The moving name: every write lands on a pane whose word is
/// different — the `String`'s write the rename takes.
fn bench_name_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("tab_panel");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("name_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| TabPanel::new(format!("Pane {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, pane) in pool.iter_mut().enumerate() {
                    pane.set_name(format!("Renamed {index}"));
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
    let mut group = c.benchmark_group("tab_panel");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| TabPanel::new(format!("Pane {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, pane) in pool.iter_mut().enumerate() {
                    pane.set_name(format!("Pane {index}"));
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
