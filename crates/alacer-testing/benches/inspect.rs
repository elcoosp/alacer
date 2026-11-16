//! The inspector's observed-tick benchmark (M83): the causal route driven
//! through the pipeline with the recorder attached on a 100-label tree —
//! every tick streams a frame summary plus the trails' facts into the
//! bounded rings and the wire-shaped broadcast queue, on the shipped
//! composition (harness + AccessKit sink), not a probe stand-in.
//!
//! The published regression gate keeps the observer honest as the causal
//! vocabulary grows: an observed tick stays in the tick's own cost class —
//! the rings push, the broadcast serializes on the observing thread, and
//! nothing blocks the pass (§13.3's opt-in overhead, priced here so it
//! cannot quietly grow a class).

use alacer_inspect::CausalRecorder;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const LABELS: usize = 100;

fn bench_inspect_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("inspect");
    group.throughput(Throughput::Elements(
        u64::try_from(LABELS).expect("non-negative"),
    ));

    group.bench_function("tick_with_recorder_attached", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().gap(4.0).boxed());
        for _ in 0..LABELS {
            let _ = harness.add_child(root, Label::new("The observed label").boxed());
        }

        // The observed shape: the recorder's handle in the observer slot —
        // the trails enable with it (the slot's own contract), the rings
        // and the broadcast queue carry the facts.
        let recorder = CausalRecorder::new("the inspect bench");
        let observer = recorder.clone_handle();
        harness.pipeline_mut().set_observer(Box::new(observer));
        harness.settle();

        // Honesty before the clock: the rings streamed the warm-up frames.
        assert!(
            recorder.frames_seen() > 0,
            "the recorder observed the warm-up frames"
        );

        b.iter(|| {
            black_box(harness.tick());
            // The rings are bounded (§14.3): the steady-state eviction is
            // part of the honest cost, never cleared to flatter the gate.
        });
    });

    group.finish();
}

criterion_group!(benches, bench_inspect_route);
criterion_main!(benches);
