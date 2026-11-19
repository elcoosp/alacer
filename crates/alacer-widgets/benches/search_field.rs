//! The search-field tier's purpose-write benchmark (M132, ADR-0136):
//! the widget crate's own pricing door. The published corpus prices
//! the a11y snapshot machinery (the M95 rule) — and the search-field
//! tier grows no snapshot route, the word being the flag the sink's
//! role branch reads beside the words the field already carries — so
//! the tier's cost is priced here, at the widget door the purpose
//! writes ride: the moving flag (the write the field's purpose takes
//! when the app declares or clears it) against the unchanged flag (the
//! write that lands where the word already is — the honest floor of a
//! `bool`'s write at the widget seam).
//!
//! This bench rides *outside* the published corpus: the coordinated
//! rule grows the baseline at the next published cut, never at a
//! widget bench's (the m132 suite's pin documents the same shape).

use alacer_widgets::TextInput;
use criterion::{Criterion, criterion_group, criterion_main};

/// The field pool: the fields the route cycles through, fresh per case.
const POOL: usize = 256;

/// The moving flag: every write lands on a field that declared nothing
/// and declares the purpose now — the declaration the purpose word
/// takes.
fn bench_purpose_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_field");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("purpose_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| TextInput::new(format!("Field {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for field in &mut pool {
                    field.set_search_field(true);
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

/// The unchanged flag: every write lands where the word already is —
/// the write's honest floor (the same `bool`'s write, no move).
fn bench_idle_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_field");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| TextInput::new(format!("Field {index}")).search_field(true))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for field in &mut pool {
                    field.set_search_field(true);
                }
                pool.len()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_purpose_write, bench_idle_write);
criterion_main!(benches);
