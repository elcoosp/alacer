//! M126 exit-criteria integration tests (§10.3, ADR-0130): the heading
//! tier — the section title the document outline walks, said by the
//! tree. Through M125 the snapshot said the hyperlink and its open
//! verb (ADR-0129), the one-of-N control and the group it reads as
//! (ADR-0128), the set words, the form facts, the collection facts,
//! the declared live region, the chord, the capacity, the language —
//! and the section title had no role: an app wiring a document outline
//! reached for a plain label, and the AT walked the page as one flat
//! run of text — the rotor's quick-nav, the outline every reader
//! skims by, had nothing to land on. The tier ships the role:
//! [`alacer_core::Role::Heading`] carrying the "names a section" fact
//! AccessKit's own `Heading` role spells (not a bold label — the level
//! word M113 published is the heading's natural carrier, the depth in
//! the outline the AT's quick-nav rides), [`alacer_widgets::Heading`]
//! the producer whose text *is* the name, the zero-based depth the
//! outline walks (AccessKit's own base beside ARIA's one-based
//! `aria-level`, the count the M113 tier's documentation carries).
//!
//! The tier prices the word and nothing else: a mounted heading's
//! neighbours answer exactly as they always did — the label beside it
//! reads, the button beside it presses, the heading's read routes are
//! the label's own (a heading names structure, it walls nothing). The
//! level write is the M21 idle frame at the tier: a write that lands
//! where the depth already is queues nothing, the moved write is the
//! one mask the tier costs.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the heading's node re-announces through the same
//! snapshot every other role rides.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Heading, Label};
/// The suite's corpus pin: the coordinated rule's count at the M126
/// cut — unchanged from the M125 cut. The heading tier grows no
/// snapshot route (the role is the fact, the pass-9 fill needs no new
/// field), and its pricing door is the widget-level bench the tier
/// carries in the widgets crate (`heading/level_write_route_256`
/// against `heading/idle_write_256`), which rides *outside* the
/// published corpus: the published baseline's routes price the a11y
/// snapshot machinery (the M95 rule), and the coordinated rule grows
/// it at the next published cut, never at a widget bench's. The pin
/// re-agrees at 155.
const EXPECTED_BENCH_COUNT: usize = 163;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m126_the_heading_role_and_level_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The document outline, mounted through the constructor words: the
    // root section beside its nested subsection — the depth the AT's
    // quick-nav walks — beside the plain label and the plain button the
    // tier keeps honest. No heading word leaks onto a label or a
    // button.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Heading::new("Getting started").boxed());
    let _ = harness.add_child(root, Heading::new("Installation").level(1).boxed());
    let _ = harness.add_child(root, Label::new("Follow the steps below").boxed());
    let _ = harness.add_child(root, Button::new("Open the guide").boxed());
    harness.settle();

    // The headings: the "names a section" fact the role spells, the
    // name the ADR-5 contract requires, the depth the outline walks —
    // zero-based, AccessKit's own base (ARIA's `aria-level` reads the
    // one-based 2, the count the sink's existing mapping never guessed
    // at).
    let started = node(&harness, Role::Heading, "Getting started").expect("the heading rides");
    assert_eq!(
        started.level, None,
        "a heading that declared no depth announces none — the honest absence"
    );
    let install = node(&harness, Role::Heading, "Installation").expect("the sub-heading rides");
    assert_eq!(
        install.level,
        Some(1),
        "the depth the outline walks, said by the tree"
    );

    // The honest absences beside the tier: the neighbours' roles are
    // exactly what they were before the tier existed.
    let steps = node(&harness, Role::Label, "Follow the steps below").expect("the label rides");
    assert_eq!(steps.role, Role::Label, "the label keeps its own read");
    let guide = node(&harness, Role::Button, "Open the guide").expect("the button rides");
    assert_eq!(guide.role, Role::Button, "the button keeps its own press");
    harness.assert_all_named();
}

#[test]
fn m126_the_level_write_is_the_idle_frame_or_the_one_mask() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let heading_id = harness.add_child(root, Heading::new("Changelog").level(1).boxed());
    harness.settle();

    // The app's live door: the mutate seam the app's code rides — the
    // depth moved, `PAINT | A11Y` requested on the heading, the frames
    // the re-announcement rides settled before the read. The tree
    // carries the moved depth the way it carries the constructor's.
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(heading_id);
        if let Some(heading) = guard.widget_mut().as_any_mut().downcast_mut::<Heading>() {
            assert!(heading.set_level(Some(3)), "the moved write answers true");
            guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        }
    }
    harness.settle();
    let moved = node(&harness, Role::Heading, "Changelog").expect("the heading rides");
    assert_eq!(
        moved.level,
        Some(3),
        "the outline re-walks at the moved depth"
    );

    // The write that lands where the depth already is queues nothing:
    // the M21 idle-frame property, at the heading tier — the write
    // answers `false`, the caller requests `A11Y` only when this
    // answers `true`, and the frame the idle write rides requests no
    // mask at all.
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(heading_id);
        if let Some(heading) = guard.widget_mut().as_any_mut().downcast_mut::<Heading>() {
            assert!(
                !heading.set_level(Some(3)),
                "the idle write answers false — no masks, an idle frame"
            );
        }
    }
    harness.settle();
    harness.assert_all_named();
}

#[test]
fn m126_the_corpus_pin_re_agrees_at_the_m126_cut() {
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

    // The published corpus pin re-agrees: the heading tier grew no
    // snapshot route, and its pricing door is the widgets crate's own
    // bench — outside the published corpus, the coordinated rule
    // growing it at the next published cut.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M126 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M126 cut"
    );
}
