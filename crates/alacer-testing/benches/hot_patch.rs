//! The hot-reload benchmark (§13.2): the patch answer's price on the
//! shipped composition (harness + AccessKit sink) — one delivered patch
//! (the generation swap + exactly one scoped re-render) at the prop-only
//! patch shape the CI hot-patch suite pins: nothing created, nothing
//! removed, exactly the swapped element patched in place.
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness layer
//! drives every surface), and because `HotView` is the view layer's own
//! contract — the runtime `alacer::hot` adds over it is the subsecond
//! detour, not the re-render this bench prices.

use alacer_macro::rsx;
use alacer_testing::TestHarness;
use alacer_view::{HotView, ViewNode};
use alacer_widgets::default_registry;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

/// The generation's row count: a realistic view the patch re-renders
/// while only the one marker element changes.
const ROWS: usize = 50;

/// One generation of the patched view: the marker label (the text
/// constant a binary patch edits) over a keyed `for` list — the shape
/// `examples/hot_reload.rs` demonstrates, with the prop-only diff the
/// CI suite pins (`created: 0, removed: 0, patched: 1`).
fn generation_view(marker: &str, rows: u64) -> ViewNode {
    rsx! {
        column { gap: 8.0, padding: 12.0,
            label { key: 0_u64, text: marker }
            for index in 0..rows {
                label { key: index + 1, text: "row {index}" }
            }
        }
    }
}

fn bench_hot_patch(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_patch");
    group.throughput(Throughput::Elements(1));

    group.bench_function("regenerate_50_rows", |b| {
        let mut harness = TestHarness::new(360.0, 900.0);
        let mut hot = HotView::mount(harness.pipeline_mut(), default_registry(), || {
            generation_view("before", u64::try_from(ROWS).expect("non-negative"))
        });
        harness.settle();
        let mut flipped = false;
        b.iter(|| {
            // The patch arrives: the swapped generation stands in for the
            // new code a binary patch installs, one scoped re-render
            // answers it — the marker's text constant is the only diff,
            // the rows reconcile in place.
            flipped = !flipped;
            let marker = if flipped { "after" } else { "before" };
            hot.set_generation(move || {
                generation_view(marker, u64::try_from(ROWS).expect("non-negative"))
            });
            let report = hot.regenerate(harness.pipeline_mut());
            harness.settle();
            black_box(report);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_hot_patch);
criterion_main!(benches);
