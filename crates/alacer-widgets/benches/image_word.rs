//! The image tier's word-write benchmark (M138, ADR-0142): the widget
//! crate's own pricing door. The published corpus prices the a11y
//! snapshot machinery (the M95 rule) — and the image tier grows no
//! snapshot route, the word being the picture's own, the shape the
//! M124/M127/M129/M131/M134/M135/M136/M137 named containers rode — so
//! the tier's cost is priced here, at the widget door the word writes
//! ride: the moving name (the write the picture's word takes when the
//! app renames it) against the unchanged name (the write that lands
//! where the word already is — the honest floor of a `String`'s write
//! at the widget seam).
//!
//! The bench joins the published corpus at the M141 cut (ADR-0011's
//! rule: every workspace benchmark is a published baseline and gated),
//! under the `image_word` group its file stem names — the render
//! sibling's raster bench keeps the `image` group, the two targets
//! never sharing a stem (the CI loop's `--bench <stem>` contract one
//! target each).

use alacer_widgets::Image;
use criterion::{Criterion, criterion_group, criterion_main};

/// The picture pool: the pictures the route cycles through, fresh per
/// case.
const POOL: usize = 256;

/// The moving name: every write lands on a picture whose word is
/// different — the `String`'s write the rename takes.
fn bench_name_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_word");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("name_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Image::new(format!("Chart {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, picture) in pool.iter_mut().enumerate() {
                    picture.set_name(format!("Renamed {index}"));
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
    let mut group = c.benchmark_group("image_word");
    group.throughput(criterion::Throughput::Elements(POOL as u64));
    group.bench_function("idle_write_256", |b| {
        b.iter_batched(
            || {
                (0..POOL)
                    .map(|index| Image::new(format!("Chart {index}")))
                    .collect::<Vec<_>>()
            },
            |mut pool| {
                for (index, picture) in pool.iter_mut().enumerate() {
                    picture.set_name(format!("Chart {index}"));
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
