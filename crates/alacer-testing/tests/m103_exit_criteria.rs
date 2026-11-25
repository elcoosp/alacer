//! M103 exit-criteria integration tests (§7.2 pass 1, ADR-0107): the
//! time seam — the one external effect with no seam, cashed.
//!
//! The row's contract, pinned at the cross-crate level: `TimeSeam`
//! answering `now_nanos` — monotonic nanos since the clock's own
//! anchor, durations the caller's arithmetic — with `MonotonicClock`
//! the canon beside `ManualClock` the deterministic seam whose clones
//! share one reading; the seam public vocabulary in core's prelude (the
//! app's own code never calls `Instant::now`); no trail tier (a clock
//! read prices what the frame already did — the causal log carries no
//! clock event). The platform's click-chain half and the corpus cases
//! (`click_chain/*`) ride their own homes (the platform crate's
//! deterministic tests, the published corpus); the frame-timing half is
//! the M104 suite's subject.

use alacer_core::prelude::{ManualClock, MonotonicClock, TimeSeam};
use alacer_testing::TestHarness;
use alacer_widgets::BoxedWidget as _;

/// The canon clock: monotonic — successive reads never go backwards —
/// and its differences are the durations the caller's arithmetic
/// prices, the anchor arbitrary.
#[test]
fn the_canon_clock_is_monotonic_and_its_diffs_are_the_durations() {
    let clock = MonotonicClock::new();
    let first = clock.now_nanos();
    let second = clock.now_nanos();
    let third = clock.now_nanos();
    assert!(
        first <= second && second <= third,
        "the canon clock never goes backwards ({first} ≤ {second} ≤ {third})"
    );
    // The durations are the caller's arithmetic — differences of reads.
    let _elapsed: u64 = third.saturating_sub(first);
}

/// The deterministic seam: clones share the one reading — the test
/// advances through its handle while the consumer reads through theirs,
/// and every read answers exactly what the advance asked for.
#[test]
fn the_manual_clocks_clones_share_one_reading() {
    let clock = ManualClock::new();
    let reader = clock.clone();
    assert_eq!(reader.now_nanos(), 0, "a fresh clock reads zero");

    clock.advance(std::time::Duration::from_millis(200));
    assert_eq!(
        reader.now_nanos(),
        200_000_000,
        "the advance is visible through the clone's read"
    );
    clock.advance_nanos(1);
    assert_eq!(
        reader.now_nanos(),
        200_000_001,
        "the raw-nanos advance rides the same shared reading"
    );
    assert_eq!(
        reader.now_nanos(),
        200_000_001,
        "reads never move the reading — only the advance does"
    );
}

/// The seam is prelude vocabulary: the app's own code takes the trait
/// and the two provided clocks without reaching into a module path —
/// the never-call-`Instant::now` rule's import surface.
#[test]
fn the_seam_is_prelude_vocabulary() {
    fn prices_read(clock: &dyn TimeSeam) -> u64 {
        clock.now_nanos()
    }
    fn owns_a_clock(clock: Box<dyn TimeSeam>) -> Box<dyn TimeSeam> {
        assert!(
            clock.now_nanos() > 0,
            "the canon clock reads from its anchor"
        );
        clock
    }

    let manual = ManualClock::new();
    manual.advance(std::time::Duration::from_millis(7));
    assert_eq!(prices_read(&manual), 7_000_000);
    let owned = owns_a_clock(Box::new(MonotonicClock::new()));
    assert!(owned.now_nanos() > 0, "the owned clock keeps reading");
}

/// No trail tier: a frame of clock reads streams nothing — the causal
/// log carries no clock event, a read pricing what the frame already
/// did (the row's own rule), the trail draining empty.
#[test]
fn a_clock_read_streams_no_fact() {
    let mut harness = TestHarness::new(240.0, 160.0);
    let _label = harness.mount(alacer_widgets::Label::new("timed").boxed());
    let _report = harness.tick();
    let trail = harness.trail();
    assert!(
        trail.look().is_empty(),
        "no tier streamed — the clock read prices the frame, it never speaks"
    );
}

/// The corpus pin re-agrees: the time seam's pricing doors
/// (`click_chain/*`, `pass_overhead/*`) are already in the published
/// corpus — the suite grows no case, the pin asserting the count this
/// cut publishes.
const EXPECTED_BENCH_COUNT: usize = 205;

#[test]
fn m103_the_corpus_pin_re_agrees_at_the_cut() {
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
