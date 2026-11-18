//! The frame's clock (M104, ADR-0108) driven from the consumer side:
//! the pass ledger's wall pricing as data the seam decides — a stepping
//! clock prices every pass exactly, a frozen clock prices the idle frame
//! zero, and with no clock installed the fallback anchor still answers
//! real time. The same [`alacer_core::time::TimeSeam`] the platform's
//! click chain rides (M103), installed at the pipeline's own door
//! ([`alacer_core::passes::PassPipeline::set_clock`]).
//!
//! The beat the example exists to show: a timed frame is not "whatever
//! the wall clock felt like" once a seam clock is installed — the ledger
//! the M1 gate instrumentation reports, the frame the inspector's trail
//! carries, becomes reproducible: exact nanos, every run, no sleeps.
//! Tests assert the ledger; CI pins it; a demo prints it.
//!
//! Run with `cargo run -p alacer --example frame_clock`. The example is
//! headless — the real pass pipeline with the software raster, no
//! window (A4) — and its assertions run on every CI test pass.

use std::cell::Cell;

use alacer::prelude::*;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label};

/// The demo's seam: every read one step past the last — a clock the
/// demo controls without sleeping, the deterministic frame's own shape.
struct StepClock {
    nanos: Cell<u64>,
    step: u64,
}

impl TimeSeam for StepClock {
    fn now_nanos(&self) -> u64 {
        let next = self.nanos.get().saturating_add(self.step);
        self.nanos.set(next);
        next
    }
}

fn main() {
    let mut harness = TestHarness::new(360.0, 120.0);
    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());
    let _title = harness.add_child(
        root,
        Label::new("The frame's clock: the ledger the seam prices (ADR-0108)").boxed(),
    );
    harness.settle();

    // The shipped shape first: no clock installed, timings on — the
    // fallback anchor's real reads. Run it twice and the totals differ:
    // that is the wall clock's own honesty, and exactly what a test
    // cannot assert.
    harness.pipeline_mut().set_timings(true);
    let first = harness.tick();
    let second = harness.tick();
    println!(
        "fallback  : total {} ns then {} ns — real time, never twice the same",
        first.total_nanos, second.total_nanos
    );

    // The seam: install the stepping clock and the same idle frame is a
    // table of exact values — every priced pass a whole step, two runs
    // printing byte-identical ledgers.
    harness.pipeline_mut().set_clock(Box::new(StepClock {
        nanos: Cell::new(0),
        step: 1_000_000,
    }));
    let report = harness.tick();
    for run in report.passes.iter() {
        println!(
            "seam      : {:<24} {:>6} ns  ({})",
            run.kind.name(),
            run.nanos,
            if run.ran { "ran" } else { "skipped" }
        );
    }
    assert!(
        report.passes.iter().all(|run| run.nanos % 1_000_000 == 0),
        "every priced pass answers whole steps of the seam's own read"
    );
    println!(
        "seam      : total {} ns — exact, every run, no sleeps",
        report.total_nanos
    );

    // The frozen clock: the deterministic zero frame — the shape the
    // m-suite's timed assertions ride when the ledger's *shape* is the
    // question and the wall noise is not.
    harness
        .pipeline_mut()
        .set_clock(Box::new(ManualClock::new()));
    let report = harness.tick();
    assert!(
        report.passes.iter().all(|run| run.nanos == 0) && report.total_nanos == 0,
        "a clock the demo never advanced prices zero"
    );
    println!("frozen    : every price exactly zero — the test's shape");
}
