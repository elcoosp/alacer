//! M93 exit-criteria integration tests (§8, ADR-0097): the layout seam's
//! call ledger — the recording engine beside the production one. The
//! seam's own probe has carried one number since M1 (`measure_calls`,
//! the translation-only probe every scroll milestone priced itself by);
//! every other fact of the call — the root asked, the dirty set's size,
//! the viewport, the outcome — was invisible. The alloc ledger's own
//! precedent (A7, M84 — the instrument the test/bench/example binaries
//! install, never the framework) names the shape this cut ships:
//! [`alacer_layout::RecordingLayout`] wraps the engine the assembly
//! site chose, one [`alacer_layout::LayoutCallRecord`] lands per call,
//! and the reading handle rides beside the install the way the trail
//! door's reader does.
//!
//! Every test runs the real pipeline (A4) through the harness's
//! `with_layout` door — the caller's engine under the same passes the
//! production host drives.

use alacer_core::geom::Size;
use alacer_core::mask::Change;
use alacer_core::passes::{LayoutEngine, LayoutError};
use alacer_layout::{LayoutHost, RecordingLayout};
use alacer_testing::TestHarness;
use alacer_text::ThreadLocalMeasurer;
use alacer_widgets::{BoxedWidget as _, Column, Label};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// The suite's corpus pin: the coordinated rule's count at the M93 cut —
/// the recording route's two cases joining the derive route's pair (M92)
/// on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 157;

/// The suite's own read doors into the published corpus: the recording
/// wrapper's honesty cases (the coordinated rule's bench half,
/// ADR-0097's exit criteria).
const WRAPPER_CASE: &str = "recording/wrapper_1k_dirty_10pct";
const BARE_CASE: &str = "recording/bare_wrapper_1k_dirty_10pct";

/// The harness's viewport in this suite (the door's own arguments).
const VIEWPORT: Size = Size::new(800.0, 600.0);

/// The refusing stub: the sync's message names itself — the record's
/// error-context probe (the consumer's own broken engine, the shape the
/// instrument exists to debug).
struct RefusingEngine {
    message: &'static str,
}

impl LayoutEngine for RefusingEngine {
    fn sync_and_layout(
        &mut self,
        _arena: &mut alacer_core::Arena,
        _root: alacer_core::arena::NodeId,
        _dirty: &FxHashMap<alacer_core::arena::NodeId, Change>,
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

#[test]
fn m93_the_ledger_prices_the_call_story() {
    let _guard = alacer_core::flux::batch_test_lock();
    let engine = RecordingLayout::new(Box::new(LayoutHost::new(Arc::new(ThreadLocalMeasurer))));
    let ledger = engine.handle();
    let mut harness = TestHarness::with_layout(VIEWPORT.width, VIEWPORT.height, Box::new(engine));

    let root = harness.mount(Column::new().gap(4.0).boxed());
    let _ = harness.add_child(root, Label::new("the ledger's first frame").boxed());
    harness.settle();

    // One record per call, the call's facts carried: the asked root,
    // the dirty set's size, the viewport, the outcome.
    let records = ledger.calls();
    assert!(
        !records.is_empty(),
        "the settle's frames each landed a record"
    );
    let first = &records[0];
    assert_eq!(first.root, root, "the asked root rides the record");
    assert!(first.dirty >= 1, "the mount's dirty set is non-empty");
    assert_eq!(
        first.viewport, VIEWPORT,
        "the viewport the clamp priced against rides"
    );
    assert_eq!(
        first.outcome,
        Ok(()),
        "the delivery rides — the production host answered"
    );
    for record in &records {
        assert_eq!(record.outcome, Ok(()), "every call delivered");
    }

    // The drain empties the window; the next frame lands fresh.
    ledger.clear();
    assert!(ledger.calls().is_empty(), "the drain empties the ledger");
    harness
        .pipeline_mut()
        .arena_mut()
        .merge_mask(root, Change::LAYOUT);
    let report = harness.tick();
    assert!(
        !ledger.calls().is_empty() || report.is_idle(),
        "the drained window fills again with the next call (or the frame was already idle)"
    );
}

#[test]
fn m93_the_counters_delegate() {
    let _guard = alacer_core::flux::batch_test_lock();
    let engine = RecordingLayout::new(Box::new(LayoutHost::new(Arc::new(ThreadLocalMeasurer))));
    let mut harness = TestHarness::with_layout(VIEWPORT.width, VIEWPORT.height, Box::new(engine));

    let root = harness.mount(Column::new().gap(4.0).boxed());
    let _ = harness.add_child(root, Label::new("the translation probe").boxed());
    harness.settle();

    // The seam's own probe keeps its meaning under the instrument: the
    // count answers through the wrapper, the reset reaches the engine.
    let measured = harness.measure_calls();
    assert!(measured > 0, "the text label measured through the wrapper");
    harness.reset_measure_calls();
    assert_eq!(
        harness.measure_calls(),
        0,
        "the reset answers through the wrapper"
    );
}

#[test]
fn m93_the_refusals_ride_the_record() {
    let _guard = alacer_core::flux::batch_test_lock();
    let engine = RecordingLayout::new(Box::new(RefusingEngine {
        message: "the persistent tree refused the style",
    }));
    let ledger = engine.handle();
    let mut harness = TestHarness::with_layout(VIEWPORT.width, VIEWPORT.height, Box::new(engine));

    let _root = harness.mount(Column::new().gap(4.0).boxed());

    // The harness's tick panics on a layout failure (the suite's own
    // contract); the refusal is priced at the pipeline's door, where
    // the seam's `Result` lives — and the record carries the engine's
    // own message beside it.
    let outcome = harness.pipeline_mut().tick(VIEWPORT);
    assert!(outcome.is_err(), "the refusal is the answer");
    let records = ledger.calls();
    assert!(
        !records.is_empty(),
        "the refused call still landed a record"
    );
    let last = records.last().expect("the refused call's record");
    assert!(
        last.outcome
            .as_ref()
            .is_err_and(|error| error.message.contains("refused the style")),
        "the error's own message rides the record: {:?}",
        last.outcome
    );
}

#[test]
fn m93_the_corpus_grew_by_the_coordinated_rule() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let baselines = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/baselines/baselines.json"))
            .expect("the published baseline is readable"),
    )
    .expect("the published baseline is valid JSON");
    let contracts = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/contracts.json"))
            .expect("the contract set is readable"),
    )
    .expect("the contract set is valid JSON");

    // The published corpus carries the recording route's honesty pair:
    // the wrapper against the bare engine on the same incremental
    // route — the instrument's cost the measurement.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(WRAPPER_CASE) && benches.contains_key(BARE_CASE),
        "the recording route's pair is published ({WRAPPER_CASE}, {BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M93 cut"
    );

    // The contract set gates both cases and prices the honest-wrapper
    // budget — a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [WRAPPER_CASE, BARE_CASE] {
        let gate = gates
            .iter()
            .find(|gate| gate["baseline"].as_str() == Some(case))
            .unwrap_or_else(|| panic!("the case carries a regression gate ({case})"));
        assert_eq!(
            gate["scope"].as_str(),
            Some("same-host"),
            "the gate is same-host scoped like the route tier's own"
        );
    }
    let budgets = contracts["budgets"]
        .as_array()
        .expect("the contract set carries budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"].as_str() == Some("recording_wrapper_overhead"))
        .expect("the honest-wrapper budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(WRAPPER_CASE),
        "the budget prices the wrapped route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BARE_CASE),
        "the budget prices it against the bare engine"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the instrument must not outprice the engine it wraps"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 157;"),
        "the m5 corpus pin re-agrees at the M93 cut"
    );
}
