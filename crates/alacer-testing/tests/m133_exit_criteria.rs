//! M133 exit-criteria integration tests (§7.2 pass 9, ADR-0137): the
//! meter tier — the gauge whose reading is a measurement, said by the
//! tree. Through M132 the snapshot said the search-field tier
//! (ADR-0136), the named pane (ADR-0135), the instant on/off control
//! (ADR-0134) — and the scalar *measurement* had no word: an app
//! wiring a disk-usage gauge, a battery reading, a capacity headroom
//! reached for the progress bar wearing the wrong word (a progress
//! bar's value is *task progress* — how far some task has come toward
//! done; a meter's value is a *measurement* on a scale the app
//! declares — how much of something there is, ARIA's `role="meter"`
//! beside its `role="progressbar"`), or for a bare label carrying no
//! value words at all. AccessKit's `Meter` role is its own fact — the
//! word the AT's gauge conventions key off.
//!
//! The word rides the value tier's own vocabulary, never a second
//! value language: the M3 `value()` contract carries the spoken
//! reading, the M97 range word carries the scale the app declared,
//! and the M99 numeric word carries the number behind the spoken one.
//! The gauge joins the required-name set (an unnamed meter reads as a
//! bare number — the ADR-5 rule the named roles carry) and never the
//! focusable set (a reading is not a control: it holds no focus and
//! answers no activation — the M126 heading's own rule at the value
//! tier's read half). The AT tier is read-only by construction: a
//! measurement has no write door, every action refused honestly (the
//! M21 progress rule, at the meter tier).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the sink's role branch grows one beside the
//! progress indicator's.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Meter, Progress};

/// The suite's corpus pin: the coordinated rule's count at the M133
/// cut — unchanged from the M132 cut. The meter tier grows no snapshot
/// *route* (the gauge rides the role branch the sink maps and the
/// value words M3/M97/M99 already carry — no new property, no new
/// machinery), and its pricing door is the widget-level bench the
/// tier carries in the widgets crate (`meter/value_write_256` against
/// `meter/idle_write_256`), which rides *outside* the published
/// corpus: the published baseline's routes price the a11y snapshot
/// machinery (the M95 rule), and the coordinated rule grows it at the
/// next published cut, never at a widget bench's. The pin re-agrees
/// at 144.
const EXPECTED_BENCH_COUNT: usize = 144;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m133_the_gauge_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    // The gauge, mounted through the constructor word: the reading
    // beside the task progress and the button the canon keeps honest —
    // each control's word its own.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(
        root,
        Meter::new("Disk usage", 0.0, 100.0).value(62.0).boxed(),
    );
    let _ = harness.add_child(root, Progress::new("Import", 0.0, 100.0).boxed());
    let _ = harness.add_child(root, Button::new("Refresh").boxed());
    harness.settle();

    // The gauge carries the meter word and the scale it declared — the
    // value tier's own words (the M3 spoken reading, the M97 range,
    // the M99 number behind the spoken one), never a second language.
    let meter = node(&harness, Role::Meter, "Disk usage").expect("the gauge rides the snapshot");
    assert_eq!(
        meter.numeric_value,
        Some(62.0),
        "the number behind the reading rides the M99 word"
    );
    let range = meter
        .value_range
        .expect("the declared scale rides the M97 word");
    assert_eq!(range.min, 0.0);
    assert_eq!(range.max, 100.0);
    assert_eq!(
        meter.value.as_deref(),
        Some("62"),
        "the spoken reading rides the M3 word"
    );

    // The neighbours keep the words they published: the task progress
    // reads as the progress tier's word, the button keeps its own.
    node(&harness, Role::ProgressIndicator, "Import").expect("the progress bar keeps its word");
    node(&harness, Role::Button, "Refresh").expect("the button keeps its word");
    harness.assert_all_named();
}

#[test]
fn m133_the_word_prices_the_reading_and_nothing_else() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let meter_id = harness.add_child(root, Meter::new("Disk usage", 0.0, 100.0).boxed());
    harness.settle();

    // The one write funnel (the app's door): clamp into the range,
    // report whether the reading moved. The caller requests the write's
    // own masks — `PAINT | A11Y` — only when this answers `true`.
    let moved = {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(meter_id);
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Meter>()
            .is_some_and(|meter| meter.set_value(48.0));
        guard.request(alacer_core::Change::PAINT | alacer_core::Change::A11Y);
        drop(guard);
        moved
    };
    harness.settle();
    let meter = node(&harness, Role::Meter, "Disk usage").expect("the gauge survives");
    assert!(moved, "the app's write reports the move");
    assert_eq!(
        meter.numeric_value,
        Some(48.0),
        "the write landed — the reading rides"
    );

    // The wall: a write past the declared scale clamps into it — the
    // caller requests the write's own masks, the tree re-announces the
    // reading the next settle.
    let clamped = {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(meter_id);
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Meter>()
            .is_some_and(|meter| meter.set_value(1000.0));
        guard.request(alacer_core::Change::PAINT | alacer_core::Change::A11Y);
        drop(guard);
        moved
    };
    assert!(clamped, "the wall answers a move");
    harness.settle();
    let meter = node(&harness, Role::Meter, "Disk usage").expect("the gauge survives");
    assert_eq!(
        meter.numeric_value,
        Some(100.0),
        "the range is a wall — the reading lands at the max"
    );

    // The idle write: the write that lands where the reading already is
    // answers *not moved* and queues no masks — the tick it would have
    // paid reports idle (the M21 idle-frame property, at the meter
    // tier).
    let idle = {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(meter_id);
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Meter>()
            .is_some_and(|meter| meter.set_value(1000.0));
        drop(guard);
        moved
    };
    assert!(
        !idle,
        "the write landed where the reading already is — not moved, no masks"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "the idle write queues nothing — the frame after it is idle"
    );

    // The AT tier is read-only (a measurement has no write door): the
    // AT's value write is refused honestly, never munged (the M21
    // progress rule, at the meter tier), and the reading is untouched.
    let answered = harness.act(
        Role::Meter,
        "Disk usage",
        alacer_core::widget::A11yAction::SetValue("42".to_string()),
    );
    assert!(
        !answered,
        "a reporting gauge has no write door — the refusal is honest"
    );
    let meter = node(&harness, Role::Meter, "Disk usage").expect("the gauge survives");
    assert_eq!(
        meter.numeric_value,
        Some(100.0),
        "the refusal wrote nothing — the reading is the app's voice"
    );
    harness.assert_all_named();
}

#[test]
fn m133_the_dsl_meter_word_declares_it() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    // The `meter` word the gauge element parses — the tree's truth
    // rides the word, at mount, with no new prop class to pay for (the
    // M32 value tier's own shape: the binding wins where one is
    // offered, the static value stands where one is declared).
    let view = alacer_macro::rsx! {
        column {
            meter { name: "Battery", min: 0.0_f32, max: 100.0_f32, value: 87.0_f32 }
        }
    };
    let root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    let declared =
        node(&harness, Role::Meter, "Battery").expect("the view's gauge rides the snapshot");
    assert_eq!(
        declared.numeric_value,
        Some(87.0),
        "the DSL's meter word declares the reading through the same door"
    );
    let _ = root;
}

#[test]
fn m133_the_corpus_pin_re_agrees_at_the_m133_cut() {
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

    // The published corpus pin re-agrees: the meter tier grows no
    // snapshot route (the gauge rides the sink's role branch and the
    // value words M3/M97/M99 already carry), and its pricing door is
    // the widgets crate's own bench — outside the published corpus,
    // the coordinated rule growing it at the next published cut.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M133 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M133 cut"
    );
}

#[test]
fn m133_the_living_documents_re_agree() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).expect("the file is readable");

    // The README status reflects the living cut, and the register grew
    // the meter tier's record.
    let readme = read("README.md");
    assert!(
        readme.contains("M0 through M135"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("a hundred and forty decision records"),
        "the register grew the M133 row"
    );
    assert!(
        read("docs/architecture.md").contains("a hundred and forty decision records"),
        "the architecture carries the meter fact's decision"
    );

    // The examples catalog grew the meter row, and the changelog names
    // the tier.
    assert!(
        read("docs/examples.md").contains("examples/meter.rs"),
        "the catalog carries the meter demonstration"
    );
    assert!(
        read("CHANGELOG.md").contains("M133 — the meter tier"),
        "the changelog names the tier"
    );

    // The m81 suite's documents pin moved to M133's cut (the latest
    // suite carrying the living-documents pin — the m132 cut's suites
    // pin the corpus, and their pins re-agreed with it).
    let m81 = read("crates/alacer-testing/tests/m81_exit_criteria.rs");
    assert!(
        m81.contains("M0 through M135"),
        "the m81 suite's pins moved with the documents"
    );
}
