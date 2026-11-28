//! M104 exit-criteria integration tests (§7.2 pass 1, ADR-0108): the
//! pipeline's frame timing rides the time seam.
//!
//! The row's contract, pinned through the real pipeline (A4 — the
//! harness's assembled host, journal, and sink): `PassPipeline::set_clock`
//! beside `set_timings`, the newest install wins; the fallback kept (no
//! clock installed: the anchor's elapsed answers — the frame still
//! measures); the timed frame exact data (a stepping clock prices every
//! pass a whole step, two idle ticks of the same tree price identically,
//! a frozen clock prices zero). No patch-tier growth and no trail tier —
//! the ledger is the frame report's own vocabulary. The corpus cases
//! (`pass_overhead/timed_idle_10k` beside
//! `pass_overhead/seamed_timed_idle_10k`, the ratio budget) ride the
//! published corpus; this suite pins the data asserts the row names.

use alacer_core::passes::{FrameReport, PassKind};
use alacer_core::prelude::{ManualClock, TimeSeam};
use alacer_testing::TestHarness;

/// A stepping clock: every read advances the reading one whole step —
/// the deterministic frame's price probe (the core tests' own shape,
/// pinned here against the assembled real pipeline).
struct StepClock {
    nanos: std::cell::Cell<u64>,
    step: u64,
}

impl StepClock {
    fn new(step: u64) -> Self {
        Self {
            nanos: std::cell::Cell::new(0),
            step,
        }
    }
}

impl TimeSeam for StepClock {
    fn now_nanos(&self) -> u64 {
        let next = self.nanos.get().saturating_add(self.step);
        self.nanos.set(next);
        next
    }
}

/// The pass ledger's prices, plainly — the determinism the seam claims,
/// as data.
fn ledger_prices(report: &FrameReport) -> Vec<u64> {
    report.passes.iter().map(|run| run.nanos).collect()
}

/// Ticks until no pass has work (the m-suite's settle shape, timings
/// kept on so every settled frame prices its passes).
fn settle(harness: &mut TestHarness) -> FrameReport {
    for _ in 0..16 {
        let report = harness.tick();
        if report.is_idle() {
            return report;
        }
    }
    panic!("the suite's frames never settled");
}

/// A stepping clock prices every pass a whole step: the structural
/// frame runs every pass, every read prices the seam's own step, and
/// two idle ticks of the same tree price identically — the determinism,
/// as data, through the assembled real stack.
#[test]
fn a_stepping_clock_prices_every_pass_a_whole_step_and_idle_ticks_identically() {
    let mut harness = TestHarness::new(400.0, 600.0);
    let _column = harness.mount(alacer_widgets::Column::new().boxed());
    harness.pipeline_mut().set_timings(true);
    harness
        .pipeline_mut()
        .set_clock(Box::new(StepClock::new(1_000_000)));

    let first = harness.tick();
    let second = harness.tick();
    let third = harness.tick();

    let first_prices = ledger_prices(&first);
    assert!(
        !first_prices.is_empty() && first_prices.iter().all(|nanos| *nanos > 0),
        "the structural frame runs every pass: every read priced"
    );
    for nanos in first_prices.iter().chain(ledger_prices(&second).iter()) {
        assert_eq!(
            nanos % 1_000_000,
            0,
            "every pass prices whole steps of the seam's own read ({nanos})"
        );
    }
    assert_eq!(
        first.total_nanos % 1_000_000,
        0,
        "the frame's total prices whole steps too"
    );
    assert_eq!(
        ledger_prices(&second),
        ledger_prices(&third),
        "two idle ticks of the same tree price identically — the seam's determinism, as data"
    );
}

/// A frozen clock prices zero: the deterministic frame the tests install
/// a `ManualClock` for — no read moves, no pass prices anything.
#[test]
fn a_frozen_manual_clock_prices_the_tick_zero() {
    let mut harness = TestHarness::new(400.0, 600.0);
    let _column = harness.mount(alacer_widgets::Column::new().boxed());
    harness.pipeline_mut().set_timings(true);
    harness
        .pipeline_mut()
        .set_clock(Box::new(ManualClock::new()));

    let report = harness.tick();
    for run in report.passes.iter() {
        assert_eq!(
            run.nanos, 0,
            "a clock the test never advanced prices zero — the deterministic frame"
        );
    }
    assert_eq!(
        report.total_nanos, 0,
        "the frame's total answers the same frozen clock"
    );
}

/// The newest install wins, one door: the frozen clock's frames price
/// zero until the stepping clock installs — the next frame prices the
/// new clock's steps, the old install never answering again.
#[test]
fn the_newest_installed_clock_wins() {
    let mut harness = TestHarness::new(400.0, 600.0);
    let _column = harness.mount(alacer_widgets::Column::new().boxed());
    harness.pipeline_mut().set_timings(true);
    harness
        .pipeline_mut()
        .set_clock(Box::new(ManualClock::new()));
    let frozen = harness.tick();
    assert!(
        frozen.passes.iter().all(|run| run.nanos == 0),
        "the frozen clock prices zero"
    );

    harness
        .pipeline_mut()
        .set_clock(Box::new(StepClock::new(500_000)));
    let stepped = harness.tick();
    assert!(
        stepped.passes.iter().all(|run| run.nanos % 500_000 == 0)
            && stepped.passes.iter().any(|run| run.nanos > 0),
        "the newest installed clock answers — one door, the last install wins"
    );

    // The layout pass's ledger entry rides the same pricing: the pass
    // that ran carries the step, the skipped ones still recorded (the
    // contract order's own rule).
    let layout = stepped
        .passes
        .iter()
        .find(|run| run.kind == PassKind::Layout)
        .unwrap_or_else(|| panic!("the ledger records the layout pass"));
    assert!(
        layout.ran || layout.nanos == 0,
        "a skipped pass records zero, a ran pass its step — the ledger's honesty"
    );
}

/// The fallback kept: no clock installed, the tick still measures wall
/// time — the anchor's elapsed answers, the pre-seam tick's shape.
#[test]
fn without_a_clock_the_tick_still_measures_wall_time() {
    let mut harness = TestHarness::new(400.0, 600.0);
    let _column = harness.mount(alacer_widgets::Column::new().boxed());
    harness.pipeline_mut().set_timings(true);

    let report = harness.tick();
    assert!(
        report.total_nanos > 0,
        "the fallback measures the frame — no clock installed is not no timing"
    );
    assert!(
        ledger_prices(&report).iter().any(|nanos| *nanos > 0),
        "the passes the structural frame ran carry wall prices"
    );
    settle(&mut harness);
}

/// The corpus pin re-agrees: the frame timing's pricing doors
/// (`pass_overhead/timed_idle_10k` beside
/// `pass_overhead/seamed_timed_idle_10k`) are already in the published
/// corpus — the suite grows no case, the pin asserting the count this
/// cut publishes.
const EXPECTED_BENCH_COUNT: usize = 208;

#[test]
fn m104_the_corpus_pin_re_agrees_at_the_cut() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let baselines = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/baselines/baselines.json"))
            .expect("the published baseline is readable"),
    )
    .expect("the published baseline is valid JSON");
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the cut"
    );
}
