//! The time seam (M103, ADR-0107) driven from the consumer side: the
//! app's clock as an injected seam — the label answers whatever the
//! seam reads, and a [`alacer_core::time::ManualClock`] makes the
//! answer exact. No `Instant::now` anywhere in the app's own code, no
//! sleep in any test that prices the label: advance the clock, write
//! the read in one batch, settle, and the tree says exactly what the
//! advance named.
//!
//! The beat the example exists to show: the widget never owns the
//! clock (§10.3, the axiom the click chain's own doc names), and app
//! code that needs elapsed time rides the same seam — hold one
//! [`alacer_core::time::TimeSeam`], read it, write the read as one
//! value batch. Swap the [`alacer_core::time::ManualClock`] for a
//! [`alacer_core::time::MonotonicClock`] and every line below stays
//! the app's — the label answers wall time instead of test time.
//!
//! Run with `cargo run -p alacer --example time_seam`. The example is
//! headless — the real pass pipeline with the software raster, no
//! window (A4) — and its assertions run on every CI test pass.

use std::rc::Rc;

use alacer::prelude::*;
use alacer_core::flux::RenderBinding;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label};
use reactive_graph::traits::Set as _;

/// The one formatter the label's map and the app's own read share —
/// the value, formatted once, never two vocabularies.
fn format_elapsed(nanos: u64) -> String {
    let seconds = nanos / 1_000_000_000;
    let millis = (nanos % 1_000_000_000) / 1_000_000;
    format!("{seconds}.{millis:03} s elapsed")
}

/// The app's only time read: one seam read minus the start, formatted —
/// the contract [`alacer_core::time::TimeSeam`] names (differences of
/// two monotonic reads, never a calendar fact). A `&dyn TimeSeam`
/// argument is the seam's own shape: the caller decides which clock
/// answers, the code never does.
fn elapsed_label(clock: &dyn TimeSeam, started_at: u64) -> String {
    format_elapsed(clock.now_nanos().saturating_sub(started_at))
}

fn main() {
    // The app's clock: the deterministic seam — the shape a test holds.
    // (`Rc<ManualClock>` because the app's handle and the writer's beat
    // share one reading; the consumer holds the seam through the same
    // handle, never a copy.)
    let clock = Rc::new(ManualClock::new());
    let started_at = clock.now_nanos();

    let mut harness = TestHarness::new(360.0, 120.0);
    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());

    let nanos = harness.pipeline_mut().flux().signal(0_u64);
    let binding = RenderBinding::new("0.000 s elapsed".to_string());
    let label = harness
        .pipeline_mut()
        .add_child(root, Label::bound(binding.clone()).boxed());
    harness.pipeline_mut().flux_mut().bind_value(
        nanos,
        label,
        &binding,
        Change::LAYOUT.union(Change::PAINT),
        |value: &u64| format_elapsed(*value),
    );
    harness.settle();
    assert!(
        harness.query(Role::Label).named("0.000 s elapsed").exists(),
        "the fresh label answers the seam's zero"
    );
    println!("fresh      : the tree says \"0.000 s elapsed\" — the seam's zero");

    // The beat: advance the clock exactly 1.5 s, read the seam, write
    // the read as one value batch — one frame, one re-announcement.
    clock.advance(std::time::Duration::from_millis(1500));
    let read = elapsed_label(clock.as_ref(), started_at);
    nanos.set(clock.now_nanos());
    harness.settle();
    assert!(
        harness.query(Role::Label).named("1.500 s elapsed").exists(),
        "the label answers exactly what the advance named — no sleeps, no hoping"
    );
    println!("app read   : \"{read}\" — the seam's own answer, the tree beside it");
    println!("advanced   : the tree says \"1.500 s elapsed\" — exact, every run");

    // The canon clock answers the same code: real time, never backwards.
    let wall = MonotonicClock::new();
    let first = TimeSeam::now_nanos(&wall);
    let second = TimeSeam::now_nanos(&wall);
    assert!(second >= first, "the canon clock never goes backwards");
    println!("canon      : MonotonicClock reads {second} ns — the shipped default");

    // The seam the click chain rides (the platform's own beat, §10.3):
    // the window is priced by the same contract this example reads.
    println!("the seam   : one trait, two clocks, every read named (ADR-0107)");
}
