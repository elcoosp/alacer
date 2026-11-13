//! M9 exit-criteria integration tests (§11.1, ADR-0015): the windowed path
//! drains the pipeline's own sink. Dirty-only `TreeUpdate`s carry the full
//! M6 vocabulary — values, toggle state, grid slots — removals and focus
//! ride the same delta, the adapter-activation response is the sink's own
//! full-tree build, and the a11y ratio is a published budget.
//!
//! Every test runs the real pipeline (A4) through the same composition the
//! platform loop uses: reach the erased `A11ySink`, downcast to the
//! `AccessKitSink`, drain. No test double, no source-string pin for the
//! behavior under test — the mechanism itself runs here.

use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::Role;
use alacer_core::events::Key;
use alacer_core::passes::A11ySink;
use alacer_testing::TestHarness;
use std::path::PathBuf;
use std::sync::Arc;

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

/// The exact mechanism the platform loop calls at frame boundaries and on
/// adapter activation: reach the pipeline's erased a11y sink, downcast,
/// delegate. The platform's drain helper wraps this same triple.
fn with_sink<R>(harness: &mut TestHarness, f: impl FnOnce(&mut AccessKitSink) -> R) -> R {
    let sink = harness
        .pipeline_mut()
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<AccessKitSink>())
        .expect("the harness carries the AccessKit sink — the windowed composition");
    f(sink)
}

fn drain_updates(harness: &mut TestHarness) -> Vec<accesskit::TreeUpdate> {
    with_sink(harness, AccessKitSink::drain_updates)
}

fn widget_id(harness: &TestHarness, role: Role, name: &str) -> u64 {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name))
        .map_or_else(
            || panic!("no {role:?} named {name:?} in the resolved tree"),
            |node| node.id,
        )
}

// ---------------------------------------------------------------------------
// The frame-boundary protocol: init once, deltas after
// ---------------------------------------------------------------------------

#[test]
fn m9_the_first_frame_drains_the_init_tree_and_quiet_frames_drain_nothing() {
    let mut harness = TestHarness::new(240.0, 320.0);
    let root = harness.mount(Box::new(alacer_widgets::Column::new()));
    let _ = harness.add_child(root, Box::new(alacer_widgets::Label::new("A")));
    let _ = harness.add_child(root, Box::new(alacer_widgets::Label::new("B")));
    harness.settle();

    let drained = drain_updates(&mut harness);
    assert!(!drained.is_empty(), "the init frame produces an update");
    let init = drained.last().expect("at least one update");
    assert!(init.tree.is_some(), "the init update (re)builds the tree");
    let resolved = harness
        .pipeline()
        .a11y_snapshot()
        .expect("the tree resolved")
        .nodes
        .len();
    assert_eq!(
        init.nodes.len(),
        resolved,
        "the init update covers the whole forest"
    );
    for update in drained.iter().take(drained.len() - 1) {
        assert!(update.tree.is_none(), "only the last frame initializes");
    }

    // The steady state: a quiet frame produces no update at all — the
    // pre-M9 platform rebuilt the whole tree per a11y frame regardless.
    harness.settle();
    assert_eq!(
        drain_updates(&mut harness),
        Vec::new(),
        "no a11y work, no update — the §11.1 item 4 contract at the window"
    );
}

#[test]
fn m9_a_toggle_reaches_the_window_as_a_dirty_only_update_with_its_state() {
    let mut harness = TestHarness::new(240.0, 320.0);
    let root = harness.mount(Box::new(alacer_widgets::Column::new()));
    let _ = harness.add_child(
        root,
        Box::new(alacer_widgets::Checkbox::new("Ship in bulk")),
    );
    let _ = harness.add_child(
        root,
        Box::new(alacer_widgets::Checkbox::new("Ship in parts")),
    );
    let _ = harness.add_child(
        root,
        Box::new(alacer_widgets::Checkbox::new("Hold the boat")),
    );
    harness.settle();
    let _ = drain_updates(&mut harness);

    harness.press(Role::CheckBox, "Ship in parts");
    let drained = drain_updates(&mut harness);
    assert!(!drained.is_empty(), "the toggle's delta drains");
    let target = widget_id(&harness, Role::CheckBox, "Ship in parts");
    let siblings = [
        widget_id(&harness, Role::CheckBox, "Ship in bulk"),
        widget_id(&harness, Role::CheckBox, "Hold the boat"),
    ];
    let mut emitted = std::collections::BTreeSet::new();
    for update in &drained {
        assert!(
            update.tree.is_none(),
            "a toggle never rebuilds the tree — §11.1 item 4 at the window"
        );
        emitted.extend(update.nodes.iter().map(|(id, _)| id.0));
    }
    assert!(
        emitted.contains(&target),
        "the toggled checkbox re-emits with its state"
    );
    for sibling in siblings {
        assert!(
            !emitted.contains(&sibling),
            "clean siblings cost nothing — dirty nodes only"
        );
    }
    let toggled = drained
        .iter()
        .rev()
        .find_map(|update| find_in_update(update, target))
        .expect("the toggled node is in the drained batch");
    assert_eq!(
        toggled.toggled(),
        Some(accesskit::Toggled::True),
        "the OS adapter now sees the checked state — the pre-M9 window dropped it"
    );
}

#[test]
fn m9_text_values_reach_the_windowed_update() {
    let mut harness = TestHarness::new(240.0, 320.0);
    let _root = harness.mount(Box::new(alacer_widgets::TextInput::new("Notes")));
    harness.settle();
    let _ = drain_updates(&mut harness);

    harness.press_key(Key::Tab);
    harness.type_text("hello");
    let drained = drain_updates(&mut harness);
    assert!(!drained.is_empty(), "the typing delta drains");
    let target = widget_id(&harness, Role::TextInput, "Notes");
    let node = drained
        .iter()
        .rev()
        .find_map(|update| find_in_update(update, target))
        .expect("the edited input re-emits");
    assert_eq!(
        node.value(),
        Some("hello"),
        "inputs carry their content to the OS (§11.1: the M3 contract, now at the window)"
    );
}

#[test]
fn m9_removals_ride_the_delta_and_the_resolved_tree_shrinks() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let mut handles = alacer_widgets::todo_app(harness.pipeline_mut(), None);
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();
    let _ = drain_updates(&mut harness);

    harness.press_key(Key::Tab); // first focusable: the input
    harness.type_text("write docs");
    harness.press_key(Key::Enter);
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();
    let _ = drain_updates(&mut harness);
    assert!(
        harness.query(Role::CheckBox).named("write docs").exists(),
        "the item exists before the removal"
    );

    harness.press(Role::Button, "Delete write docs");
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();
    let drained = drain_updates(&mut harness);
    assert!(!drained.is_empty(), "the removal's delta drains");
    assert!(
        !harness
            .pipeline()
            .a11y_snapshot()
            .is_some_and(|tree| tree.contains(Role::CheckBox, "write docs")),
        "the resolved tree no longer carries the dead item"
    );
    with_sink(&mut harness, |sink| {
        assert!(
            !sink.last_removed_ids().is_empty(),
            "the dead widget's id surfaces through the sink's removal record"
        );
    });
}

// ---------------------------------------------------------------------------
// The adapter-activation protocol: one full tree from the same builder
// ---------------------------------------------------------------------------

#[test]
fn m9_the_activation_response_is_the_sinks_own_full_tree() {
    let mut harness = TestHarness::new(240.0, 320.0);
    let root = harness.mount(Box::new(alacer_widgets::Column::new()));
    let _ = harness.add_child(root, Box::new(alacer_widgets::Label::new("A")));
    harness.settle();

    // The AT connects: the platform answers from the sink's resolved tree —
    // one builder, full vocabulary, no platform-side rebuild.
    let live = with_sink(&mut harness, |sink| {
        sink.full_tree_update().expect("the tree is resolved")
    });

    // Parity: what a fresh sink builds from the same snapshot on its init
    // frame must equal the activation response byte for byte.
    let snapshot = Arc::new(
        harness
            .pipeline()
            .a11y_snapshot()
            .cloned()
            .expect("resolved tree"),
    );
    let mut fresh = AccessKitSink::new();
    fresh.apply(
        Arc::clone(&snapshot),
        &alacer_core::passes::A11yDelta {
            initializes: true,
            ..alacer_core::passes::A11yDelta::empty()
        },
    );
    let init = fresh.last_update().cloned().expect("init update");
    assert_eq!(
        live, init,
        "one code path materializes the tree — the platform builds nothing"
    );
}

// ---------------------------------------------------------------------------
// The published budget (the M5 coordinated corpus rule)
// ---------------------------------------------------------------------------

/// The corpus pin, mirroring the coordinated rule: 48 criterion cases —
/// the 47 pre-M10 cases plus the `a11y_action/route_100_actions_1k` route
/// bench (ADR-0016). Growing the corpus moves the bench file,
/// contracts.json, the baseline, and the M5/M6/M7/M8/M9 pins together.
const EXPECTED_BENCH_COUNT: usize = 64;

#[test]
fn m9_the_a11y_ratio_is_a_published_budget_and_the_corpus_moves_together() {
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");

    let benches = baselines["benches"].as_object().expect("benches object");
    let regressions = contracts["regressions"].as_array().expect("regressions");
    assert_eq!(
        benches.len(),
        EXPECTED_BENCH_COUNT,
        "the corpus grew by the pair"
    );
    assert_eq!(regressions.len(), EXPECTED_BENCH_COUNT);
    for id in ["a11y/full_rebuild_1k", "a11y/incremental_1k_dirty_10pct"] {
        assert!(benches.contains_key(id), "{id} published");
        assert!(
            regressions
                .iter()
                .any(|gate| gate["baseline"] == serde_json::json!(id)),
            "{id} contracted"
        );
    }

    let budgets = contracts["budgets"].as_array().expect("budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"] == serde_json::json!("a11y_incremental_speedup"))
        .expect("the a11y ratio is a published budget");
    assert_eq!(
        budget["numerator"],
        serde_json::json!("a11y/full_rebuild_1k"),
        "the pre-M9 shape is the numerator"
    );
    assert_eq!(
        budget["denominator"],
        serde_json::json!("a11y/incremental_1k_dirty_10pct"),
        "the M9 shape is the denominator"
    );
    let floor = budget["budget"].as_f64().expect("budget is a number");
    assert!(floor >= 2.0, "the ratio floor is the §11.1 gate (≥ 2×)");

    // The earlier suites pin the same corpus — the chain cannot drift.
    for suite in [
        "m5_exit_criteria.rs",
        "m6_exit_criteria.rs",
        "m7_exit_criteria.rs",
        "m8_exit_criteria.rs",
    ] {
        let source = read_workspace_file(&format!("crates/alacer-testing/tests/{suite}"));
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 64;"),
            "{suite} must pin the 60-case corpus"
        );
    }
}
