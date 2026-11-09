//! The progress benchmark (§11.5, ADR-0027): the app-driven value-write
//! route driven through the pipeline on a 1,000-node arena — the bound
//! bridge's write batch lands `PAINT | A11Y` on the bar's node and the
//! tick repaints the fill and re-announces exactly that node, on the
//! shipped composition (harness + AccessKit sink), not a probe stand-in.
//!
//! The published regression gate keeps the report honest as the contract
//! grows: a value write must stay in the cost class of a bound write —
//! one signal write, one binding read, one pass run, one `TreeUpdate` —
//! never a re-layout of the arena or a re-materialization of its a11y
//! tree (the bar is a leaf; a value write moves no geometry).
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness
//! layer drives every surface).

use alacer_core::mask::Change;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Panel, Progress};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const ROUTES: usize = 100;
const FILLER: usize = 1000;
const PANEL_HEIGHT: f32 = 0.4;

fn bench_progress_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("progress");
    group.throughput(Throughput::Elements(
        u64::try_from(ROUTES).expect("non-negative"),
    ));

    group.bench_function("route_100_value_writes_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().boxed());
        let _ = harness.add_child(root, Label::new("The reported value").boxed());
        for i in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
            let _ = i;
        }
        let binding = alacer_core::flux::RenderBinding::new(0.0_f32);
        let _ = harness.add_child(
            root,
            Progress::bound("Batch", 0.0, 100.0, binding.clone()).boxed(),
        );
        harness.settle();

        let node = {
            let arena = harness.pipeline().arena();
            let mut found = None;
            arena.for_each_pre_order(&mut |id, slot| {
                if found.is_none() && slot.widget().role() == alacer_core::Role::ProgressIndicator {
                    found = Some(id);
                }
            });
            found.expect("the progress bar is mounted")
        };
        let signal = harness.pipeline_mut().flux_mut().signal(0.0_f32);
        harness.pipeline_mut().flux_mut().bind_value(
            signal,
            node,
            &binding,
            Change::PAINT.union(Change::A11Y),
            |value: &f32| *value,
        );
        harness.settle();

        // One hundred value writes, bouncing across the range — each
        // route is one write batch + one tick: the binding's masks, the
        // paint pass's fill repaint, and the a11y pass's node-only delta.
        let mut value = 0.0_f32;
        b.iter(|| {
            value = if value >= 100.0 { 0.0 } else { value + 1.0 };
            alacer_core::flux::Flux::batch_of(|| {
                reactive_graph::traits::Set::set(&signal, value);
            });
            black_box(harness.tick());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_progress_route);
criterion_main!(benches);
