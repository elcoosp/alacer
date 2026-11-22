//! M87 exit-criteria integration tests (§A7, ADR-0091): the input-frame
//! allocation budget — the budget at the tier the warm tick does not
//! cover. ADR-0088's own pre-registered re-open rule cashed: "a
//! consumer needs the budget at a tier the warm tick does not cover
//! (an input frame, a flip frame, a present path the harness cannot
//! drive) → the beat joins the suite as its own measured case, priced
//! by the same ledger — never a budget asserted without an
//! instrument." Through M86 the ledger's budget had one tier: the warm
//! tick of a static UI, pinned clean to the byte (the m84 suite), an
//! input-driven frame priced by nothing — the folklore the instrument
//! exists to answer. The instrument measured first: an input frame is
//! **not** the warm path — a pointer-move frame churns a small,
//! balanced burst (blocks allocated and freed inside the same tick,
//! the dispatch buffers' capacity paid once on the first move and
//! retained ever after) — so the budget the suite pins is the honest
//! measured shape: the steady-state input frame **retains nothing**
//! (`is_balanced`, net zero to the byte, twenty frames in a row), the
//! first move's capacity the one retained cost (named, one-time), and
//! the warm tick beside it staying clean (the m84 budget re-agreed in
//! the same window). The corpus grows 92 → 93 by the coordinated rule
//! (`alloc/input_frame_budget`, the measured beat priced by the same
//! ledger the suite rides).
//!
//! Every test runs the real pipeline (A4) under the adopted hook
//! ([`dhat::Alloc`], the binary's own allocator — the framework never
//! installs it, ADR-0088's rule), one ledger armed per test for
//! exactly the beats it prices.

#[global_allocator]
static LEDGER: alacer_testing::alloc::Alloc = alacer_testing::alloc::Alloc;

use alacer_core::geom::Point;
use alacer_testing::TestHarness;
use alacer_testing::alloc::AllocLedger;
use alacer_widgets::BoxedWidget as _;

/// The harness's pointer-move frame: one move dispatched, one tick —
/// the input frame the budget names, the smallest unit an input beat
/// rides.
fn move_frame(harness: &mut TestHarness, x: f32, y: f32) {
    harness.pointer_move(Point::new(x, y));
    let _ = harness.tick();
}

/// The steady-state input frame retains nothing: the first tracked
/// move frame pays the dispatch queue's one-time block (the +192 bytes
/// the instrument measured — named, once), and the twenty frames after
/// it churn their balanced burst with the net at zero to the byte. The
/// warm idle tick beside it stays clean: the m84 budget re-agreed in
/// the same window.
#[test]
fn m87_the_input_frame_retains_nothing_in_steady_state() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(400.0, 300.0);
    let root = harness.mount(alacer_widgets::Column::new().gap(8.0).padding(12.0).boxed());
    let _button = harness.add_child(
        root,
        alacer_widgets::Button::new("The budget's button").boxed(),
    );
    harness.settle();

    let ledger = AllocLedger::arm();
    // The first tracked move frame: the dispatch queue's one-time
    // block lands here (the capacity the steady state then holds) —
    // measured and named, never wished away.
    let ((), first) = ledger.measure(|| move_frame(&mut harness, 40.0, 40.0));
    assert!(
        first.net_bytes() > 0,
        "the first tracked move frame paid no one-time block — the measured truth moved: {first:?}"
    );

    let ((), twenty) = ledger.measure(|| {
        for i in 0_u8..20 {
            move_frame(&mut harness, 44.0 + f32::from(i), 40.0);
        }
    });
    assert!(
        twenty.allocations > 0,
        "the input frame churned nothing — the budget priced a myth"
    );
    assert!(
        twenty.is_balanced(),
        "the input frame retained memory: {twenty:?}"
    );
    assert_eq!(
        twenty.net_bytes(),
        0,
        "the input frame's net moved off zero: {twenty:?}"
    );

    // The warm idle tick beside it: the m84 budget, re-agreed in the
    // same window the input frame rides.
    let (_, idle) = ledger.measure(|| harness.tick());
    assert!(
        idle.is_clean(),
        "the warm tick allocated {idle:?} — the A7 budget is broken"
    );
}

/// The first move's retained cost is one-time: a fresh pipeline's
/// first tracked pointer-move frame pays the dispatch queue's block
/// (the net the instrument measured at +192 bytes) — and the second
/// move frame retains nothing, the capacity held, never re-paid. A
/// consumer reading the budget sees the one retained block named, not
/// a per-frame leak.
#[test]
fn m87_the_first_move_pays_the_dispatch_capacity_once() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(400.0, 300.0);
    let root = harness.mount(alacer_widgets::Column::new().gap(8.0).padding(12.0).boxed());
    let _button = harness.add_child(
        root,
        alacer_widgets::Button::new("The budget's button").boxed(),
    );
    harness.settle();

    let ledger = AllocLedger::arm();
    let ((), first) = ledger.measure(|| move_frame(&mut harness, 40.0, 40.0));
    assert!(
        first.net_bytes() > 0,
        "the first move paid no capacity — the measured truth moved: {first:?}"
    );
    let ((), second) = ledger.measure(|| move_frame(&mut harness, 44.0, 40.0));
    assert_eq!(
        second.net_bytes(),
        0,
        "the second move re-paid the capacity — a per-frame leak: {second:?}"
    );
}

/// The corpus carries the input frame's pricing: the coordinated rule
/// (ADR-0011) grew 92 → 93 with `alloc/input_frame_budget` — the
/// measured beat published in the baseline, gated in the contracts,
/// and driven by the bench the ledger's binary runs (the m84 suite's
/// corpus shape, one tier further).
#[test]
fn m87_the_corpus_pins_the_input_frame_budget() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let root = root.canonicalize().expect("workspace root resolves");
    let baseline = root.join("benchmarks/baselines/baselines.json");
    let baseline = std::fs::read_to_string(&baseline).expect("the published baseline exists");
    let baseline: serde_json::Value =
        serde_json::from_str(&baseline).expect("the published baseline parses");
    let entry = baseline
        .get("benches")
        .and_then(|benches| benches.get("alloc/input_frame_budget"))
        .unwrap_or_else(|| {
            panic!("alloc/input_frame_budget is not published — the corpus grew without the beat")
        });
    assert!(
        entry["median_ns"].as_f64().unwrap_or(0.0) > 0.0,
        "the input frame's beat published a non-positive median"
    );
    let bench = root.join("crates/alacer-testing/benches/alloc.rs");
    let bench = std::fs::read_to_string(&bench).expect("the ledger's bench exists");
    assert!(
        bench.contains("input_frame_budget"),
        "the ledger's bench does not drive the input frame's beat"
    );
    let suite_pin = "const EXPECTED_BENCH_COUNT: usize = 151;";
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite exists");
    assert!(
        m5.contains(suite_pin),
        "the m5 suite does not pin the 131-case corpus"
    );
}
