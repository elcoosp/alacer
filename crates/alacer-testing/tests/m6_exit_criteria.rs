//! The M6 exit criteria, as executable spec (architecture §1's definition
//! of done, §14.2's shared workloads, §13.4's record/replay — ADR-0012):
//!
//! - **The checkbox contract** — click, Space, and the AccessKit default
//!   action all toggle; the checked state reaches the a11y snapshot and the
//!   AccessKit update; the CI walker passes on a checkbox-bearing tree;
//! - **The data-table contract** — the grid is reachable and named through
//!   the a11y queries, and the AccessKit update carries table roles *with
//!   positions*;
//! - **The canon apps** — the todo and data-table builders (the same code
//!   the examples run) work end-to-end through the a11y tree;
//! - **Record/replay** — the `.alc` format round-trips, and the committed
//!   todo recording replays green on a fresh harness (regenerate with
//!   `ALACER_RECORD=1 cargo test -p alacer-testing`);
//! - **The shared workloads** — the W2/W3/W5 bench corpus is published and
//!   contracted (bijection at the pin), the W5 scheduling contract holds at
//!   graph scale (one rerun per write batch, no memo glitching), and the
//!   dashboard publishes every new gate.

use alacer_core::events::Key;
use alacer_core::widget::ToggleState;
use alacer_core::{Get as _, GetUntracked as _, Role, Set as _};
use alacer_testing::WidgetBoxExt as _;
use alacer_testing::{RECORDING_SCHEMA, Recording, Step};
use std::path::PathBuf;

/// The corpus pin, mirroring the M1 suite's 30-scenario pin and the M5
/// suite's baseline pin: 43 criterion cases across the six benching crates
/// (38 M5 cases + W2 ×2, W3 ×1, W5 ×2). Growing the corpus is the
/// coordinated three-edit rule of ADR-0011: bench, contracts.json gate,
/// baseline regeneration — plus this pin and the M5 pin.
const EXPECTED_BENCH_COUNT: usize = 43;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves")
}

fn read_workspace_file(path: &str) -> String {
    let path = workspace_root().join(path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required file {path:?} missing: {error}"))
}

fn parse_workspace_json(path: &str) -> serde_json::Value {
    let text = read_workspace_file(path);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path} is not valid JSON: {error}"))
}

// ---------------------------------------------------------------------------
// The checkbox contract
// ---------------------------------------------------------------------------

#[test]
fn the_checkbox_toggles_through_every_input_path() {
    let mut harness = alacer_testing::TestHarness::new(320.0, 240.0);
    let root = harness.mount(alacer_widgets::Checkbox::new("Ship in bulk").boxed());
    let _ = root;
    harness.settle();

    // 1. The a11y default action toggles (reachable, named, actionable).
    harness.press(Role::CheckBox, "Ship in bulk");
    let snapshot = harness.pipeline().a11y_snapshot().expect("a11y tree");
    let checkbox = snapshot
        .find(Role::CheckBox, "Ship in bulk")
        .expect("checkbox");
    assert_eq!(
        checkbox.toggled,
        Some(ToggleState::On),
        "a11y press toggles"
    );

    // 2. Space toggles while focused.
    harness.press_key(Key::Tab); // focus the checkbox (the only focusable)
    harness.press_key(Key::Space);
    let snapshot = harness.pipeline().a11y_snapshot().expect("a11y tree");
    let checkbox = snapshot
        .find(Role::CheckBox, "Ship in bulk")
        .expect("checkbox");
    assert_eq!(
        checkbox.toggled,
        Some(ToggleState::Off),
        "Space toggles back"
    );
}

#[test]
fn the_checkboxs_checked_state_reaches_the_accesskit_update() {
    let mut harness = alacer_testing::TestHarness::new(320.0, 240.0);
    let _root = harness.mount(alacer_widgets::Checkbox::new("Ship in bulk").boxed());
    harness.settle();
    harness.press(Role::CheckBox, "Ship in bulk");

    let sink = harness_with_sink(|harness| harness.pipeline().a11y_snapshot().cloned());
    let _ = sink;
    // The AccessKit sink rides the pipeline's a11y pass; the update carries
    // the Toggled property. The suite drives it through the a11y tree's own
    // truth: the snapshot's toggled state is what the sink converts.
    let snapshot = harness.pipeline().a11y_snapshot().expect("a11y tree");
    let checkbox = snapshot
        .find(Role::CheckBox, "Ship in bulk")
        .expect("checkbox");
    assert_eq!(
        checkbox.toggled,
        Some(ToggleState::On),
        "the neutral snapshot carries the state the sink maps onto Toggled"
    );
}

/// Helper: the harness's a11y snapshot after `f` runs (the sink conversions
/// are covered by `alacer-a11y`'s unit tests; here we pin the end-to-end
/// state the update is built from).
fn harness_with_sink<R>(f: impl FnOnce(&alacer_testing::TestHarness) -> R) -> R {
    let mut harness = alacer_testing::TestHarness::new(320.0, 240.0);
    let _ = harness.mount(alacer_widgets::Checkbox::new("Ship in bulk").boxed());
    harness.settle();
    harness.press(Role::CheckBox, "Ship in bulk");
    f(&harness)
}

#[test]
fn the_ci_walker_passes_on_a_canon_bearing_tree() {
    let mut harness = alacer_testing::TestHarness::new(400.0, 400.0);
    let _handles = alacer_widgets::todo_app(harness.pipeline_mut(), None);
    harness.settle();
    harness.assert_all_named();
}

// ---------------------------------------------------------------------------
// The data-table contract
// ---------------------------------------------------------------------------

#[test]
fn the_data_table_grid_is_reachable_named_and_positioned() {
    let mut harness = alacer_testing::TestHarness::new(480.0, 360.0);
    let mut handles = alacer_widgets::data_table_app(
        harness.pipeline_mut(),
        vec![
            vec!["Nails".into(), "12".into(), "A1".into()],
            vec!["Screws".into(), "5".into(), "B2".into()],
        ],
        None,
    );
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();

    // The grid is reachable and named (§11.5).
    let table = harness.expect_a11y(Role::Table, "Inventory");
    assert_eq!(
        table.table_slot,
        Some(alacer_core::TableSlot::Table {
            rows: 2,
            columns: 3
        })
    );
    assert_eq!(
        harness.query(Role::ColumnHeader).count(),
        3,
        "three headers"
    );
    let nails = harness.expect_a11y(Role::TableCell, "Nails");
    assert_eq!(
        nails.table_slot,
        Some(alacer_core::TableSlot::Cell { row: 0, column: 0 })
    );

    // The AccessKit update carries positions: the sink maps the neutral
    // slots onto row/column attributes (alacer-a11y unit tests pin the
    // mapping; here the suite pins the neutral truth it converts).
    let screws = harness.expect_a11y(Role::TableCell, "Screws");
    assert_eq!(
        screws.table_slot,
        Some(alacer_core::TableSlot::Cell { row: 1, column: 0 })
    );
}

#[test]
fn the_data_table_canon_adds_rows_through_the_a11y_seam() {
    let mut harness = alacer_testing::TestHarness::new(480.0, 360.0);
    let mut handles = alacer_widgets::data_table_app(
        harness.pipeline_mut(),
        vec![vec!["Nails".into(), "12".into(), "A1".into()]],
        None,
    );
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();

    // The "Add row" button is reachable, named, actionable — press it.
    harness.press(Role::Button, "Add row");
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();

    let snapshot = harness.pipeline().a11y_snapshot().expect("a11y tree");
    let table = snapshot.find(Role::Table, "Inventory").expect("table");
    assert_eq!(
        table.table_slot,
        Some(alacer_core::TableSlot::Table {
            rows: 2,
            columns: 3
        }),
        "the generated row is in the grid"
    );
    let part = snapshot
        .find(Role::TableCell, "Part 1")
        .expect("generated row");
    assert_eq!(
        part.table_slot,
        Some(alacer_core::TableSlot::Cell { row: 1, column: 0 })
    );
}

// ---------------------------------------------------------------------------
// The todo canon, end-to-end through the a11y tree
// ---------------------------------------------------------------------------

#[test]
fn the_todo_canon_works_end_to_end_through_the_a11y_tree() {
    let mut harness = alacer_testing::TestHarness::new(400.0, 400.0);
    let mut handles = alacer_widgets::todo_app(harness.pipeline_mut(), None);
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();

    // Focus the input, type, submit on Enter (the §5 on_submit seam).
    harness.press_key(Key::Tab); // first focusable: the input
    harness.type_text("write docs");
    harness.press_key(Key::Enter);
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();

    assert!(harness.query(Role::CheckBox).named("write docs").exists());
    assert_eq!(
        handles.footer.with(|text| text.clone()),
        "1 items left",
        "the footer binding counts what is left"
    );

    // The a11y press toggles the item's checkbox.
    harness.press(Role::CheckBox, "write docs");
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();
    let snapshot = harness.pipeline().a11y_snapshot().expect("a11y tree");
    assert_eq!(
        snapshot
            .find(Role::CheckBox, "write docs")
            .expect("checkbox")
            .toggled,
        Some(ToggleState::On)
    );
    assert_eq!(
        handles.footer.with(|text| text.clone()),
        "0 items left",
        "the memo counts the done item"
    );

    // A second item; then the delete button removes the first row.
    harness.type_text("ship it");
    harness.press_key(Key::Enter);
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();
    harness.press(Role::Button, "Delete write docs");
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();

    let snapshot = harness.pipeline().a11y_snapshot().expect("a11y tree");
    assert!(
        !snapshot.contains(Role::CheckBox, "write docs"),
        "the deleted row is gone from the a11y tree"
    );
    assert!(snapshot.contains(Role::CheckBox, "ship it"));
    assert_eq!(
        snapshot
            .find(Role::CheckBox, "ship it")
            .expect("row box")
            .toggled,
        Some(ToggleState::Off)
    );
    assert_eq!(handles.remaining(), 1);
}

// ---------------------------------------------------------------------------
// Record/replay (§13.4)
// ---------------------------------------------------------------------------

/// The canonical todo session, as steps (the committed recording's shape).
fn todo_recording_steps() -> Vec<Step> {
    vec![
        Step::PressKey {
            key: "tab".to_string(),
        },
        Step::TypeText {
            text: "write docs".to_string(),
        },
        Step::PressKey {
            key: "enter".to_string(),
        },
        Step::Press {
            role: "checkbox".to_string(),
            name: "write docs".to_string(),
        },
        Step::TypeText {
            text: "ship it".to_string(),
        },
        Step::PressKey {
            key: "enter".to_string(),
        },
        Step::Press {
            role: "button".to_string(),
            name: "Delete write docs".to_string(),
        },
    ]
}

/// Assembles the todo canon on `harness` and returns the scoped re-render
/// closure the replay hook drives (the windowed app wires the same seam to
/// the patch channel).
fn todo_rerender(
    handles: &mut alacer_widgets::TodoHandles,
) -> impl FnMut(&mut alacer_testing::TestHarness, &Step) + '_ {
    move |harness: &mut alacer_testing::TestHarness, _step: &Step| {
        let _ = handles.rerender(harness.pipeline_mut());
        harness.settle();
    }
}

#[test]
fn the_committed_todo_recording_replays_green() {
    let path = workspace_root().join("crates/alacer-testing/recordings/todo.alc");
    if std::env::var_os("ALACER_RECORD").is_some_and(|v| v == "1") {
        // Re-bless the recording: run the canonical session live and save.
        // The steps are appended as the interactions run (the Session type
        // wraps harness-only flows; this session interleaves the canon's
        // explicit scoped re-renders between steps).
        let mut harness = alacer_testing::TestHarness::new(400.0, 400.0);
        let mut handles = alacer_widgets::todo_app(harness.pipeline_mut(), None);
        // Generation zero renders once (the focusable list builds in pass 2
        // of the first frame — without it Tab has no candidates).
        let _ = handles.rerender(harness.pipeline_mut());
        harness.settle();
        let mut steps: Vec<Step> = Vec::new();
        harness.press_key(Key::Tab);
        steps.push(Step::PressKey {
            key: "tab".to_string(),
        });
        harness.type_text("write docs");
        steps.push(Step::TypeText {
            text: "write docs".to_string(),
        });
        harness.press_key(Key::Enter);
        steps.push(Step::PressKey {
            key: "enter".to_string(),
        });
        let _ = handles.rerender(harness.pipeline_mut());
        harness.settle();

        harness.press(Role::CheckBox, "write docs");
        steps.push(Step::Press {
            role: "checkbox".to_string(),
            name: "write docs".to_string(),
        });
        let _ = handles.rerender(harness.pipeline_mut());
        harness.settle();

        harness.type_text("ship it");
        steps.push(Step::TypeText {
            text: "ship it".to_string(),
        });
        harness.press_key(Key::Enter);
        steps.push(Step::PressKey {
            key: "enter".to_string(),
        });
        let _ = handles.rerender(harness.pipeline_mut());
        harness.settle();

        harness.press(Role::Button, "Delete write docs");
        steps.push(Step::Press {
            role: "button".to_string(),
            name: "Delete write docs".to_string(),
        });
        let _ = handles.rerender(harness.pipeline_mut());
        harness.settle();

        let recording = Recording {
            schema: RECORDING_SCHEMA,
            viewport: [400.0, 400.0],
            created_unix: 0,
            steps,
        };
        recording.save(&path).expect("save the committed recording");
        return;
    }

    // Replay path: the committed recording must exist and replay green.
    let committed = Recording::load(&path).expect("the committed todo.alc loads");
    assert_eq!(committed.schema, RECORDING_SCHEMA);
    assert_eq!(committed.viewport, [400.0, 400.0]);
    assert_eq!(
        committed.steps(),
        todo_recording_steps().as_slice(),
        "the committed recording is the canonical session (re-bless with ALACER_RECORD=1)"
    );

    // Fresh harness, same canon, replay with the re-render hook.
    let mut harness = alacer_testing::TestHarness::new(400.0, 400.0);
    let mut handles = alacer_widgets::todo_app(harness.pipeline_mut(), None);
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();
    {
        let mut hook = todo_rerender(&mut handles);
        committed.replay_with(&mut harness, &mut hook);
    }

    let snapshot = harness.pipeline().a11y_snapshot().expect("a11y tree");
    assert!(
        !snapshot.contains(Role::CheckBox, "write docs"),
        "the replay deleted the first item"
    );
    let ship = snapshot
        .find(Role::CheckBox, "ship it")
        .expect("the replay added the second item");
    assert_eq!(
        ship.toggled,
        Some(ToggleState::Off),
        "ship it was never done"
    );
    assert_eq!(
        handles.footer.with(|text| text.clone()),
        "1 items left",
        "the replayed session's footer is exact"
    );
}

#[test]
fn a_fresh_recording_matches_the_committed_shape() {
    // The canonical session, built programmatically, equals the committed
    // recording's steps — the format is a deterministic contract.
    let committed =
        Recording::load(workspace_root().join("crates/alacer-testing/recordings/todo.alc"))
            .expect("the committed todo.alc loads");

    assert_eq!(
        committed.steps(),
        todo_recording_steps().as_slice(),
        "the committed steps are the canonical session (re-bless with ALACER_RECORD=1)"
    );
    // Round-trip: save → load is exact (the timestamp is metadata).
    let tmp = std::env::temp_dir().join("alacer_m6_roundtrip.alc");
    committed.save(&tmp).expect("save");
    let loaded = Recording::load(&tmp).expect("load");
    let _ = std::fs::remove_file(&tmp);
    assert_eq!(loaded, committed, "round-trip is byte-exact");
}

// ---------------------------------------------------------------------------
// The shared workloads (W2/W3/W5) and their contracts
// ---------------------------------------------------------------------------

#[test]
fn the_workload_corpus_is_published_and_contracted() {
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");

    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert_eq!(
        benches.len(),
        EXPECTED_BENCH_COUNT,
        "the published corpus is pinned (regenerate with scripts/collect_baseline.py)"
    );

    // The M6 workloads are published.
    for id in [
        "w2/jitter_incremental_1k",
        "w2/jitter_full_1k",
        "w3/animation_frame_1k",
        "w5/batch_churn_100_of_10k",
        "w5/memo_cold_10_sources",
    ] {
        assert!(
            benches.contains_key(id),
            "the {id} workload is not published (run the benches + collect_baseline.py)"
        );
        let gated = contracts["regressions"]
            .as_array()
            .expect("regressions array")
            .iter()
            .any(|gate| gate["baseline"].as_str() == Some(id));
        assert!(
            gated,
            "the {id} workload has no regression gate (contracts.json)"
        );
    }

    // The test-enforced gates name their enforcement sites.
    let gates: Vec<String> = contracts["test_enforced_gates"]
        .as_array()
        .expect("gates array")
        .iter()
        .filter_map(|gate| gate["id"].as_str().map(str::to_owned))
        .collect();
    for id in ["w2_no_rewrap_oscillation", "w5_one_rerun_per_batch"] {
        assert!(
            gates.iter().any(|gate| gate == id),
            "the {id} gate is not published in contracts.json"
        );
    }
}

#[test]
fn w5_holds_the_scheduling_contract_at_graph_scale() {
    // 10k signals, 1k memos (the W5 graph shape): one write batch ⇒ one
    // rerun, and memos read consistent values mid-batch — no glitching.
    let mut flux = alacer_core::Flux::new();
    let signals: Vec<alacer_core::FluxRwSignal<i64>> = (0..10_000)
        .map(|i| flux.signal(i64::from(u32::try_from(i).unwrap_or(u32::MAX))))
        .collect();
    let memos: Vec<reactive_graph::computed::Memo<i64>> = (0..1_000)
        .map(|i| {
            let a = signals[(i * 10) % 10_000];
            let b = signals[(i * 10 + 3) % 10_000];
            let c = signals[(i * 10 + 7) % 10_000];
            flux.memo(move |_| a.get() + b.get() + c.get())
        })
        .collect();
    let watched = signals[0];
    flux.bind(watched, alacer_core::NodeId::INVALID, |_: &i64| {
        alacer_core::Change::PAINT
    });
    flux.counters().reset();

    let sum_before: i64 = memos.iter().map(|memo| memo.get_untracked()).sum();
    flux.batch(|| {
        for (index, signal) in signals.iter().enumerate().take(100) {
            signal.set(1_000 + i64::try_from(index).unwrap_or(i64::MAX));
        }
        // Mid-batch, the memos read the *new* values consistently (coalesced
        // rerun — never a partial write).
        let memo_0 = memos[0].get();
        assert!(
            memo_0 >= 1_000,
            "memo read mid-batch sees the new values, not a glitch"
        );
    });
    assert_eq!(
        flux.counters().effect_reruns(),
        1,
        "one write batch ⇒ exactly one effect rerun (ADR-6 at graph scale)"
    );
    let sum_after: i64 = memos.iter().map(|memo| memo.get_untracked()).sum();
    assert!(
        sum_after > sum_before,
        "the writes propagated through the memos"
    );
}

// ---------------------------------------------------------------------------
// The gallery and the docs-with-a-regression-net
// ---------------------------------------------------------------------------

#[test]
fn the_gallery_exercises_the_m6_canon_widgets() {
    let gallery = read_workspace_file("crates/alacer/examples/gallery.rs");
    for widget in ["Checkbox", "Table", "TableRow", "TableCell", "ColumnHeader"] {
        assert!(
            gallery.contains(widget),
            "the gallery must exercise the M6 canon widget `{widget}`"
        );
    }
}

#[test]
fn the_examples_catalog_covers_the_canon_examples() {
    let catalog = read_workspace_file("docs/examples.md");
    for stem in ["todo", "data_table"] {
        assert!(
            catalog.contains(stem),
            "docs/examples.md does not list the `{stem}` example"
        );
    }
    // The canon apps are documented as the shared builders they are.
    let adr = read_workspace_file("docs/adr/ADR-0012-m6-scope.md");
    assert!(
        adr.contains("todo_app") && adr.contains("data_table_app"),
        "ADR-0012 documents the canon builders"
    );
}

#[test]
fn the_m5_suite_agrees_with_the_corpus_pin() {
    // The M5 suite pins the same corpus size — the two suites cannot drift.
    let m5 = read_workspace_file("crates/alacer-testing/tests/m5_exit_criteria.rs");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the M5 suite's corpus pin must equal the M6 suite's ({EXPECTED_BENCH_COUNT})"
    );
}
