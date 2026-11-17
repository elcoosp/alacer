//! The layout seam's call ledger (M93, ADR-0097) driven from the
//! consumer side: [`RecordingLayout`] wrapping the production host —
//! every [`alacer_core::passes::LayoutEngine`] call rides the engine
//! untouched, and one record lands in the ledger beside it: the root
//! asked, the dirty set's size, the viewport, the outcome. The beats:
//! one frame's story read back through the handle, the translation-only
//! probe keeping its meaning under the instrument (the `measure_calls`
//! counter the scroll milestones priced themselves by), and a
//! consumer's own broken engine refusing — its message landing in the
//! ledger where the debug session reads it.
//!
//! Run with `cargo run -p alacer --example layout_probe`. The example
//! is headless — the real pass pipeline (A4) — so the same beats run
//! anywhere the tests do.

use alacer_core::geom::Size;
use alacer_core::passes::{LayoutEngine, LayoutError};
use alacer_layout::{LayoutHost, RecordingLayout};
use alacer_testing::TestHarness;
use alacer_text::ThreadLocalMeasurer;
use alacer_widgets::{BoxedWidget as _, Column, Label};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// The viewport the demo's beats settle into.
const VIEWPORT: Size = Size::new(640.0, 480.0);

/// A consumer's own engine that refuses: the shape the instrument
/// exists to debug — every fact of the refusal landing in the ledger
/// instead of the frame-buster the raw `Result` would demand.
struct BrokenEngine {
    message: &'static str,
}

impl LayoutEngine for BrokenEngine {
    fn sync_and_layout(
        &mut self,
        _arena: &mut alacer_core::Arena,
        _root: alacer_core::arena::NodeId,
        _dirty: &FxHashMap<alacer_core::arena::NodeId, alacer_core::Change>,
        _viewport: Size,
    ) -> Result<(), LayoutError> {
        Err(LayoutError {
            message: self.message.to_string(),
            node: None,
        })
    }

    fn measure_calls(&self) -> u64 {
        0
    }

    fn reset_measure_calls(&mut self) {}
}

/// Beat 1: one frame's story — the real host wrapped, the mount's
/// frames each landing a record the handle reads back after the
/// harness has taken the engine's ownership (the trail door's shape).
fn one_frames_story() {
    let engine = RecordingLayout::new(Box::new(LayoutHost::new(Arc::new(ThreadLocalMeasurer))));
    let ledger = engine.handle();
    let mut harness = TestHarness::with_layout(VIEWPORT.width, VIEWPORT.height, Box::new(engine));

    let root = harness.mount(Column::new().gap(4.0).boxed());
    let _ = harness.add_child(root, Label::new("the ledger's first frame").boxed());
    harness.settle();

    let records = ledger.calls();
    println!("beat 1: {} records landed — the first:", records.len());
    let first = &records[0];
    println!(
        "  root {}, dirty {}, viewport {}×{}, outcome {}",
        first.root,
        first.dirty,
        first.viewport.width,
        first.viewport.height,
        match &first.outcome {
            Ok(()) => "delivered".to_string(),
            Err(message) => format!("refused: {message}"),
        }
    );
    assert_eq!(first.root, root, "the asked root rides the record");
    assert_eq!(first.outcome, Ok(()), "the production host delivered");
}

/// Beat 2: the translation-only probe keeps its meaning — the
/// `measure_calls` counter answers through the wrapper, the reset
/// reaches the engine, and the next record prices a one-node layout.
fn the_probe_keeps_its_meaning() {
    let engine = RecordingLayout::new(Box::new(LayoutHost::new(Arc::new(ThreadLocalMeasurer))));
    let ledger = engine.handle();
    let mut harness = TestHarness::with_layout(VIEWPORT.width, VIEWPORT.height, Box::new(engine));

    let root = harness.mount(Column::new().gap(4.0).boxed());
    let label = harness.add_child(root, Label::new("the translation probe").boxed());
    harness.settle();
    harness.reset_measure_calls();
    assert_eq!(
        harness.measure_calls(),
        0,
        "the reset answered through the wrapper"
    );

    harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(label, alacer_core::Change::LAYOUT);
    harness.tick();
    let records = ledger.calls();
    let last = records.last().expect("the frame's record");
    assert_eq!(last.dirty, 1, "the one-node ask priced as one dirty node");
    println!(
        "beat 2: the counter answered through the wrapper; the one-node frame priced dirty {}",
        last.dirty
    );
}

/// Beat 3: a consumer's own broken engine — the refusal rides the
/// record with its own message, the error context the seam's `Result`
/// contract ships, never a bare flag.
fn the_refusal_rides_the_record() {
    let engine = RecordingLayout::new(Box::new(BrokenEngine {
        message: "the persistent tree refused the style",
    }));
    let ledger = engine.handle();
    let mut harness = TestHarness::with_layout(VIEWPORT.width, VIEWPORT.height, Box::new(engine));

    let _root = harness.mount(Column::new().gap(4.0).boxed());

    // The harness's tick panics on a layout failure; the refusal is
    // priced at the pipeline's door, where the seam's `Result` lives.
    let outcome = harness.pipeline_mut().tick(VIEWPORT);
    assert!(outcome.is_err(), "the broken engine refused");
    let records = ledger.calls();
    let last = records.last().expect("the refused call's record");
    let message = match &last.outcome {
        Err(error) => error.message.as_str(),
        Ok(()) => panic!("the refusal rides the record"),
    };
    println!("beat 3: the refusal landed — the record carries: {message}");
}

fn main() {
    println!("=== the layout seam's call ledger (M93, ADR-0097) ===");
    one_frames_story();
    the_probe_keeps_its_meaning();
    the_refusal_rides_the_record();
    println!("=== every beat answered — the seam's call story is data ===");
}
