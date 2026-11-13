//! M20 exit-criteria integration tests (§11.5, ADR-0026): the value tier
//! becomes a widget. The slider joins the shipped vocabulary — a bounded
//! scalar whose four doors (drag, arrows, Home/End, the AT's
//! `SetValue`/`Increment`/`Decrement`) ride one funnel that clamps to the
//! range, snaps to the step, and answers in the same `PAINT | A11Y`
//! currency every other widget queues; a write that lands where the value
//! already is queues no masks. The corpus grows 54 → 55 by the
//! coordinated rule — the M19-promised capture case joins at this
//! measurement run, with its baseline and its regression gate — and the
//! living documents re-agree.
//!
//! Every test runs the real pipeline (A4) with the real layout host — the
//! exact route a window's pointer, key, and AT writes take.

use alacer_core::events::Key;
use alacer_core::geom::Point;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Slider};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves")
}

fn read_workspace_file(path: &str) -> String {
    let path = workspace_root().join(path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required file {} missing: {error}", path.display()))
}

fn parse_workspace_json(path: &str) -> serde_json::Value {
    let text = read_workspace_file(path);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path} is not valid JSON: {error}"))
}

/// The consumer's window: a slider named "Volume" over 0..=100 with the
/// status line the m1 corpus's s35 mirrors — the exact shape
/// `examples/slider.rs` assembles.
fn mount_volume(harness: &mut TestHarness) {
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, Label::new("The slider (ADR-0026)").boxed());
    let _ = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0).on_change(|_| {}).boxed(),
    );
    harness.settle();
}

/// The slider's a11y node: the public read the AT owns (tests query the
/// tree, never widget internals).
fn volume_in_tree(harness: &TestHarness) -> alacer_core::passes::TreeNode {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Slider, "Volume"))
        .cloned()
        .expect("the slider is named in the a11y tree")
}

#[test]
fn m20_the_drag_writes_and_the_keys_step() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_volume(&mut harness);

    // The drag: a press mid-track writes (and focuses — the click-to-focus
    // seam), a move writes again, the release ends the gesture.
    let bounds = harness.query(Role::Slider).named("Volume").bounds();
    let track_x = |ratio: f32| bounds.min_x() + bounds.width() * ratio;
    harness.pointer_press(Point::new(track_x(0.5), bounds.min_y() + 8.0));
    harness.pointer_move(Point::new(track_x(0.75), bounds.min_y() + 8.0));
    harness.pointer_release(Point::new(track_x(0.75), bounds.min_y() + 8.0));
    harness.settle();
    let dragged = volume_in_tree(&harness)
        .value
        .expect("the slider carries its value");
    let dragged: f32 = dragged.parse().expect("the value is the number it carries");
    assert!(
        (60.0..=80.0).contains(&dragged),
        "a three-quarter drag lands in the high half, got {dragged}"
    );

    // The keyboard: the press focused the slider. Home and End are the
    // exact ends (padding-independent), an arrow steps by exactly one.
    harness.press_key(Key::End);
    harness.settle();
    assert_eq!(
        volume_in_tree(&harness).value.as_deref(),
        Some("100"),
        "End is the max, exactly"
    );
    harness.press_key(Key::Home);
    harness.settle();
    assert_eq!(
        volume_in_tree(&harness).value.as_deref(),
        Some("0"),
        "Home is the min, exactly"
    );
    harness.press_key(Key::ArrowRight);
    harness.settle();
    assert_eq!(
        volume_in_tree(&harness).value.as_deref(),
        Some("1"),
        "one arrow = one step"
    );
}

#[test]
fn m20_a_key_step_is_paint_plus_a11y_and_nothing_else() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_volume(&mut harness);

    // The click-to-focus seam: the press lands on the slider and holds
    // focus, so the keyboard writes reach it. Home pins the value at the
    // exact minimum first — the arrow's step is then exactly one.
    let bounds = harness.query(Role::Slider).named("Volume").bounds();
    harness.pointer_press(Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + 8.0,
    ));
    harness.pointer_release(Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + 8.0,
    ));
    harness.settle();
    harness.press_key(Key::Home);
    harness.settle();

    // The arrow writes through the pipeline directly (the harness's
    // `press_key` settles, which would spend the masks before the tick
    // could show them): dispatch, then tick — the report is the frame the
    // write costs.
    harness
        .pipeline_mut()
        .dispatch_key(&alacer_core::events::KeyEvent::pressed(Key::ArrowRight));
    let report = harness.tick();
    assert!(
        report.paint_ran && report.a11y_ran,
        "the step repaints the thumb and re-announces the value"
    );
    assert!(
        !report.layout_ran,
        "a value write relayouts nothing — the geometry never moved"
    );
    assert_eq!(
        volume_in_tree(&harness).value.as_deref(),
        Some("1"),
        "the focused slider answered the arrow"
    );
}

#[test]
fn m20_the_at_value_tier_answers_the_slider() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_volume(&mut harness);

    // The write: an in-range numeric SetValue rides the same funnel the
    // drag rides — the tree reads back what the AT wrote.
    assert!(
        harness.act(
            Role::Slider,
            "Volume",
            A11yAction::SetValue("42".to_string())
        ),
        "the slider answers the AT's write"
    );
    harness.settle();
    assert_eq!(
        volume_in_tree(&harness).value.as_deref(),
        Some("42"),
        "the tree carries the written value"
    );

    // The refusals are honest: a non-numeric write is refused, never
    // munged into a step; a press action is not a value control's contract.
    assert!(
        !harness.act(
            Role::Slider,
            "Volume",
            A11yAction::SetValue("loud".to_string())
        ),
        "a non-numeric SetValue is refused"
    );
    assert!(!harness.act(Role::Slider, "Volume", A11yAction::DefaultPress));
    assert_eq!(
        volume_in_tree(&harness).value.as_deref(),
        Some("42"),
        "the refusals wrote nothing"
    );

    // The step tier: increment and decrement by the step, the clamp is a
    // wall, and every answered write lands in the tree.
    assert!(harness.act(Role::Slider, "Volume", A11yAction::Increment));
    assert!(harness.act(
        Role::Slider,
        "Volume",
        A11yAction::SetValue("1000".to_string())
    ));
    harness.settle();
    assert_eq!(
        volume_in_tree(&harness).value.as_deref(),
        Some("100"),
        "the range is a wall, whichever door wrote"
    );
    assert!(harness.act(Role::Slider, "Volume", A11yAction::Decrement));
    harness.settle();
    assert_eq!(
        volume_in_tree(&harness).value.as_deref(),
        Some("99"),
        "one decrement from the wall"
    );
}

#[test]
fn m20_the_uncaptured_hover_writes_nothing() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_volume(&mut harness);

    // Fifty moves across the track with no press in flight: the value is
    // the hover's passenger, never its cargo — no write, no masks, an
    // idle frame (the M19 idle property, at the widget tier).
    let bounds = harness.query(Role::Slider).named("Volume").bounds();
    for i in 0..50 {
        let x = bounds.min_x() + bounds.width() * alacer_core::num::usize_to_f32(i) / 50.0;
        harness.pointer_move(Point::new(x, bounds.min_y() + 8.0));
    }
    assert_eq!(harness.settle(), 0, "a pressless wave queues no masks");
    assert_eq!(
        volume_in_tree(&harness).value.as_deref(),
        Some("0"),
        "the hover wrote nothing"
    );
}

#[test]
fn m20_the_capture_case_joins_the_corpus_by_the_coordinated_rule() {
    // The M19-promised criterion case (ADR-0025's own exit row) is
    // published at this measurement run: the baseline entry and its
    // regression gate land together with the bench (ADR-0011;
    // CONTRIBUTING's publishing rule).
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let id = "input/route_100_captured_moves_1k";

    assert_eq!(
        baselines["bench_count"],
        serde_json::json!(63),
        "the corpus's pins moved to 57 at M22's measurement run (the one-cycle drift the coordinated rule prices)"
    );
    let benches = baselines["benches"].as_object().expect("benches object");
    assert!(benches.contains_key(id), "{id} is published");
    let entry = &benches[id];
    assert!(
        entry["median_ns"].as_f64().expect("median") > 0.0
            && entry["samples"].as_u64() == Some(100),
        "{id} carries a real measurement (100 samples)"
    );
    let gates = contracts["regressions"].as_array().expect("regressions");
    assert_eq!(
        gates.len(),
        63,
        "every case carries exactly one regression gate"
    );
    assert!(
        gates
            .iter()
            .any(|gate| gate["baseline"] == serde_json::json!(id)
                && gate["scope"] == serde_json::json!("same-host")),
        "{id} is gated same-host"
    );

    // Every M-suite pin moved to 57 together — the chain cannot drift.
    for suite in [
        "m5_exit_criteria.rs",
        "m6_exit_criteria.rs",
        "m7_exit_criteria.rs",
        "m8_exit_criteria.rs",
        "m9_exit_criteria.rs",
        "m10_exit_criteria.rs",
        "m11_exit_criteria.rs",
        "m12_exit_criteria.rs",
        "m13_exit_criteria.rs",
        "m14_exit_criteria.rs",
        "m15_exit_criteria.rs",
    ] {
        let source = read_workspace_file(&format!("crates/alacer-testing/tests/{suite}"));
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 63;"),
            "{suite} must pin the 60-case corpus"
        );
    }
    let m19 = read_workspace_file("crates/alacer-testing/tests/m19_exit_criteria.rs");
    assert!(
        m19.contains("All 63 criterion cases") && m19.contains("serde_json::json!(63)"),
        "the m19 suite's pins moved with the corpus"
    );
}

#[test]
fn m20_the_living_documents_re_agree() {
    // The README: the criterion count quotes the 60-case baseline, the
    // input bullet carries the value tier, the vocabulary names its
    // newest control, and the status block is the 0.14.0 family — the
    // status line and the register count moved with the M23 cut.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "All 63 criterion cases",
        "Since M20 the value tier is a widget",
        "version-0.17.0",
        "M0 through M45",
        "fifty-two decision records",
        "`Slider`",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `slider` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract, and the
    // snapshot corpus paragraph agrees with the blessed directory.
    let catalog = read_workspace_file("docs/examples.md");
    for fragment in [
        "[`slider`](../crates/alacer/examples/slider.rs)",
        "37 styled scenarios",
        "39 blessed PNGs",
    ] {
        assert!(
            catalog.contains(fragment),
            "docs/examples.md names {fragment}"
        );
    }

    // The architecture carries the decision, the register row, and the
    // roadmap row; the count lines agree with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0026",
        "52 decision records",
        "the slider joins the widget vocabulary",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The decision record itself documents the scope and its coordinated
    // remainders.
    let adr = read_workspace_file("docs/adr/ADR-0026-m20-scope.md");
    assert!(
        adr.contains("M20 scope") && adr.contains("value tier"),
        "ADR-0026 documents the slider's value-tier scope"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M20 — the valued widget (ADR-0026)**"),
        "the CHANGELOG carries the M20 entry"
    );

    // The migration guides' snapshot counts ride the same corpus.
    for guide in ["docs/migration/from-iced.md", "docs/migration/from-egui.md"] {
        let guide = read_workspace_file(guide);
        assert!(
            guide.contains("37 styled scenarios"),
            "{guide} quotes the grown corpus"
        );
    }
}

#[test]
fn m20_the_surface_rides_the_same_cut() {
    // The widget is the facade's vocabulary: exported, documented, and
    // demonstrated — the example names the decision it demonstrates.
    let lib = read_workspace_file("crates/alacer-widgets/src/lib.rs");
    assert!(
        lib.contains("pub use slider::Slider"),
        "the widgets crate exports the slider"
    );
    let example = read_workspace_file("crates/alacer/examples/slider.rs");
    assert!(
        example.contains("ADR-0026") && example.contains("Slider::new"),
        "examples/slider.rs demonstrates the value tier"
    );

    // The a11y sink maps the role faithfully — the AT sees a slider, not
    // a generic container.
    let sink = read_workspace_file("crates/alacer-a11y/src/lib.rs");
    assert!(
        sink.contains("Role::Slider => AccessKitRole::Slider"),
        "the role mapping carries the slider"
    );

    // The corpus's own count test moved with the scenarios (the m1 suite
    // pins 36 since M21, the directory carries the reported value's PNG).
    let m1 = read_workspace_file("crates/alacer-testing/tests/m1_exit_criteria.rs");
    assert!(
        m1.contains("fn m1_corpus_is_thirty_seven_scenarios()"),
        "the m1 corpus pin moved to 37"
    );
    let blessed = workspace_root().join("crates/alacer-testing/blessed");
    for png in ["m1_33_drag_capture.png", "m1_34_slider.png"] {
        assert!(
            blessed.join(png).is_file(),
            "{png} is blessed with the corpus"
        );
    }
}
