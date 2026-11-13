//! M21 exit-criteria integration tests (§11.5, ADR-0027): the value tier
//! grows its read half. A progress indicator joins the shipped vocabulary —
//! a bounded scalar the *app* reports and the AT reads, carried in the same
//! `PAINT | A11Y` currency every other value rides: the bound bridge queues
//! the caller's masks per write batch, the imperative `set_value` answers
//! the mutate seam with the move it made, and a write that lands where the
//! value already is queues nothing. Every AT action is refused honestly —
//! a reporting widget has no write door (the M11 rule). The corpus grows
//! 55 → 56 by the coordinated rule — the value-write route's criterion case
//! joins at this measurement run, with its baseline and its regression gate
//! — and the living documents re-agree (the migration guides' 0.12.0
//! self-description among them).
//!
//! Every test runs the real pipeline (A4) with the real layout host — the
//! exact route an app's write batch takes from signal to `TreeUpdate`.

use alacer_core::mask::Change;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Progress};
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

/// The consumer's window: a fixed progress bar named "Batch" the app
/// drives through the mutate seam, plus the status line the m1 corpus's
/// s36 mirrors — the exact shape `examples/progress.rs` assembles.
fn mount_batch(harness: &mut TestHarness) {
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, Label::new("The progress (ADR-0027)").boxed());
    let _ = harness.add_child(root, Progress::new("Batch", 0.0, 100.0).boxed());
    harness.settle();
}

/// The progress bar's a11y node: the public read the AT owns (tests query
/// the tree, never widget internals).
fn batch_in_tree(harness: &TestHarness) -> alacer_core::passes::TreeNode {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::ProgressIndicator, "Batch"))
        .cloned()
        .expect("the progress bar is named in the a11y tree")
}

/// The progress bar's arena node (the mutate seam's target): the walk the
/// facade's own examples use at assembly time.
fn batch_node(harness: &TestHarness) -> alacer_core::NodeId {
    let arena = harness.pipeline().arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |id, slot| {
        if found.is_none()
            && slot.widget().role() == Role::ProgressIndicator
            && slot.widget().label().resolve() == "Batch"
        {
            found = Some(id);
        }
    });
    found.expect("the progress bar is mounted")
}

/// The mutate seam the example demonstrates: take the arena lease, write
/// the value in place, and request the write's own masks on the guard
/// (Masonry `WidgetMut` discipline — the masks merge on drop).
fn app_writes(harness: &mut TestHarness, value: f32) -> bool {
    let node = batch_node(harness);
    let mut moved = false;
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
        if let Some(progress) = guard.widget_mut().as_any_mut().downcast_mut::<Progress>() {
            moved = progress.set_value(value);
            if moved {
                guard.request(Change::PAINT.union(Change::A11Y));
            }
        }
    }
    moved
}

#[test]
fn m21_the_app_writes_and_the_tree_carries() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_batch(&mut harness);

    // The app's write rides the mutate seam: the value moves, the masks
    // are the write's own currency, and the tick re-announces exactly the
    // number the tree carries.
    assert!(app_writes(&mut harness, 42.0), "the write moved the value");
    let report = harness.tick();
    assert!(
        report.paint_ran && report.a11y_ran,
        "the write repaints the fill and re-announces the value"
    );
    assert!(
        !report.layout_ran,
        "a value write relayouts nothing — the geometry never moved"
    );
    assert_eq!(
        batch_in_tree(&harness).value.as_deref(),
        Some("42"),
        "the tree carries the number the app wrote"
    );

    // The clamp is a wall, whichever side of the range the write asks for.
    assert!(app_writes(&mut harness, 1000.0));
    harness.settle();
    assert_eq!(
        batch_in_tree(&harness).value.as_deref(),
        Some("100"),
        "the range is a wall"
    );
}

#[test]
fn m21_a_no_op_write_costs_an_idle_frame() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_batch(&mut harness);
    assert!(app_writes(&mut harness, 30.0));
    harness.settle();

    // A write that lands where the value already is: no move, no masks,
    // an idle frame — the storm budget's property, at the app seam.
    assert!(
        !app_writes(&mut harness, 30.0),
        "a same-value write is no move"
    );
    assert_eq!(harness.settle(), 0, "the idle frame queues no masks");
    assert_eq!(
        batch_in_tree(&harness).value.as_deref(),
        Some("30"),
        "the tree is unchanged"
    );
}

#[test]
fn m21_the_bound_bridge_carries_the_currency() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let signal = harness.pipeline_mut().flux_mut().signal(0.0_f32);
    let binding = alacer_core::flux::RenderBinding::new(0.0_f32);
    let progress = Progress::bound("Batch", 0.0, 100.0, binding.clone());
    let _ = harness.add_child(root, progress.boxed());
    harness.settle();

    let node = batch_node(&harness);

    // The bridge door: one write batch, the caller's masks — `PAINT |
    // A11Y` — land on the node, and the widget paints and reports from
    // the binding (A1: the effect never mutates the widget).
    harness.pipeline_mut().flux_mut().bind_value(
        signal,
        node,
        &binding,
        Change::PAINT.union(Change::A11Y),
        |value: &f32| *value,
    );
    alacer_core::flux::Flux::batch_of(|| {
        reactive_graph::traits::Set::set(&signal, 75.0);
    });
    let report = harness.tick();
    assert!(
        report.paint_ran && report.a11y_ran,
        "the bridge write repaints and re-announces"
    );
    assert_eq!(
        batch_in_tree(&harness).value.as_deref(),
        Some("75"),
        "the tree reads the binding the app wrote"
    );
}

#[test]
fn m21_the_at_tier_is_read_only() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_batch(&mut harness);
    assert!(app_writes(&mut harness, 42.0));
    harness.settle();

    // A reporting widget has no write door: every AT action is refused —
    // never munged into a step or a press, the M11 rule — and the tree
    // keeps the value the app wrote.
    for action in [
        A11yAction::SetValue("99".to_string()),
        A11yAction::Increment,
        A11yAction::Decrement,
        A11yAction::DefaultPress,
    ] {
        assert!(
            !harness.act(Role::ProgressIndicator, "Batch", action),
            "the read-only value refuses the AT's write"
        );
    }
    assert_eq!(
        batch_in_tree(&harness).value.as_deref(),
        Some("42"),
        "the refusals wrote nothing"
    );

    // The funnel's own refusals: a non-finite request is not a value the
    // widget can carry. The lease scopes before the tree is read again —
    // the guard's masks merge on drop.
    {
        let node = batch_node(&harness);
        let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
        if let Some(progress) = guard.widget_mut().as_any_mut().downcast_mut::<Progress>() {
            assert!(!progress.set_value(f32::NAN), "NaN is not a value");
        }
    }
    assert_eq!(
        batch_in_tree(&harness).value.as_deref(),
        Some("42"),
        "the refusal wrote nothing"
    );
}

#[test]
fn m21_the_value_write_route_joins_the_corpus_by_the_coordinated_rule() {
    // The route's criterion case (ADR-0027's own exit row) is published at
    // this measurement run: the baseline entry and its regression gate
    // land together with the bench (ADR-0011; CONTRIBUTING's publishing
    // rule).
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let id = "progress/route_100_value_writes_1k";

    assert_eq!(
        baselines["bench_count"],
        serde_json::json!(63),
        "the corpus grows 56 → 57 at M22's measurement run"
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
    let m20 = read_workspace_file("crates/alacer-testing/tests/m20_exit_criteria.rs");
    assert!(
        m20.contains("serde_json::json!(63)") && m20.contains("usize = 63;"),
        "the m20 suite's pins moved with the corpus (the one-cycle drift the pattern prices)"
    );
}

#[test]
fn m21_the_living_documents_re_agree() {
    // The README: the criterion count quotes the 60-case baseline, the
    // a11y bullet carries the reported value, the vocabulary names its
    // newest control, and the status block is the 0.14.0 family at M23.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "All 63 criterion cases",
        "Since M21 the value is reportable too",
        "version-0.17.0",
        "M0 through M43",
        "fifty decision records",
        "`Progress`",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `progress` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract, and the
    // snapshot corpus paragraph agrees with the blessed directory.
    let catalog = read_workspace_file("docs/examples.md");
    for fragment in [
        "[`progress`](../crates/alacer/examples/progress.rs)",
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
        "ADR-0027",
        "50 decision records",
        "the progress indicator joins the value tier",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The decision record itself documents the scope and its coordinated
    // remainders.
    let adr = read_workspace_file("docs/adr/ADR-0027-m21-scope.md");
    assert!(
        adr.contains("M21 scope") && adr.contains("reported value"),
        "ADR-0027 documents the progress indicator's scope"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M21 — the reported value (ADR-0027)**"),
        "the CHANGELOG carries the M21 entry"
    );

    // The migration guides re-agree: the 0.12.0 self-description the M16
    // pattern's remainder left behind names the shipped family again, and
    // the snapshot counts ride the grown corpus.
    for guide in ["docs/migration/from-iced.md", "docs/migration/from-egui.md"] {
        let guide = read_workspace_file(guide);
        assert!(
            guide.contains("onto Alacer **0.16.0**"),
            "{guide} names the shipped family"
        );
        assert!(
            guide.contains("37 styled scenarios"),
            "{guide} quotes the grown corpus"
        );
    }
}

#[test]
fn m21_the_surface_rides_the_same_cut() {
    // The widget is the facade's vocabulary: exported, documented, and
    // demonstrated — the example names the decision it demonstrates.
    let lib = read_workspace_file("crates/alacer-widgets/src/lib.rs");
    assert!(
        lib.contains("pub use progress::Progress"),
        "the widgets crate exports the progress indicator"
    );
    let example = read_workspace_file("crates/alacer/examples/progress.rs");
    assert!(
        example.contains("ADR-0027") && example.contains("Progress::"),
        "examples/progress.rs demonstrates the reported value"
    );

    // The a11y sink maps the role faithfully — the AT sees a progress
    // indicator, not a generic container.
    let sink = read_workspace_file("crates/alacer-a11y/src/lib.rs");
    assert!(
        sink.contains("Role::ProgressIndicator => AccessKitRole::ProgressIndicator"),
        "the role mapping carries the progress indicator"
    );

    // The micro-debt the cut prices is gone: the dead introspection
    // accessors (the M18 species) no longer ship.
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    assert!(
        !passes.contains("fn has_observer"),
        "PassPipeline::has_observer is gone"
    );
    let backend = read_workspace_file("crates/alacer-platform/src/wgpu_backend.rs");
    assert!(
        !backend.contains("fn atlas_clears") && !backend.contains("fn glyphs_rendered"),
        "the wgpu backend's dead stats surface is gone"
    );

    // The corpus's own count test moved with the scenarios (the m1 suite
    // pins 36, the directory carries the new PNG).
    let m1 = read_workspace_file("crates/alacer-testing/tests/m1_exit_criteria.rs");
    assert!(
        m1.contains("fn m1_corpus_is_thirty_seven_scenarios()"),
        "the m1 corpus pin moved to 37"
    );
    let blessed = workspace_root().join("crates/alacer-testing/blessed");
    assert!(
        blessed.join("m1_35_progress.png").is_file(),
        "m1_35_progress.png is blessed with the corpus"
    );
}
