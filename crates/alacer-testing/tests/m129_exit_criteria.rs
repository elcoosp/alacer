//! M129 exit-criteria integration tests (§10.3, ADR-0133): the toolbar
//! tier — the named strip a set of tools reads as, said by the tree.
//! Through M128 the snapshot said the bounded number field the AT
//! walks by arrows (ADR-0132), the page selector and its strip
//! (ADR-0131), the section title (ADR-0130), the hyperlink
//! (ADR-0129), the one-of-N control and its group (ADR-0128) — and
//! the tool strip had no word: an app wiring one reached for a bare
//! row (a container wearing no name — the set the AT cannot name is a
//! set it cannot walk, the ADR-5 rule the named roles carry) or for a
//! tab list wearing the wrong word (a tab strip *selects pages*; a
//! toolbar *offers tools*). [`alacer_core::Role::Toolbar`] carries the
//! strip word AccessKit's own `Toolbar` role spells — the
//! tab-list/radio-group shape (the named container, ADR-0128's
//! member-and-container discipline) at the tool tier.
//!
//! The strip is the tools' layout, never a second channel to them: the
//! tools keep the roles and labels they already carry, the button
//! beside the input pressing and editing exactly as they always did.
//! A toolbar is not a control: it holds no focus and answers no
//! activation — the word names the set the tools read as, it walls
//! nothing.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the strip's word costs the word and nothing else.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, TextInput, Toolbar};

/// The suite's corpus pin: the coordinated rule's count at the M129
/// cut — unchanged from the M128 cut. The toolbar tier grows no
/// snapshot route (the word is the container's own, the shape the
/// M124/M127 named containers rode), and its pricing door is the
/// widget-level bench the tier carries in the widgets crate
/// (`toolbar/name_write_256` against `toolbar/idle_write_256`), which
/// rides *outside* the published corpus: the published baseline's
/// routes price the a11y snapshot machinery (the M95 rule), and the
/// coordinated rule grows it at the next published cut, never at a
/// widget bench's. The pin re-agrees at 155.
const EXPECTED_BENCH_COUNT: usize = 207;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m129_the_toolbar_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The strip, mounted through the constructor word: the named
    // container the AT announces before it walks the tools, the tools
    // the members' own controls — and the plain widgets the tier keeps
    // honest. No toolbar word leaks onto a button or an input.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let strip = Toolbar::new("Formatting");
    let strip_id = harness.add_child(root, strip.boxed());
    let _ = harness.add_child(strip_id, Button::new("Bold").boxed());
    let _ = harness.add_child(strip_id, Button::new("Italic").boxed());
    let _ = harness.add_child(root, TextInput::new("Body").boxed());
    harness.settle();

    // The strip: the "reads as a set of tools" fact the container role
    // spells, the name the ADR-5 contract requires.
    let formatting = node(&harness, Role::Toolbar, "Formatting").expect("the strip rides");
    assert_eq!(
        formatting.role,
        Role::Toolbar,
        "the strip keeps its own word"
    );

    // The tools: the members keep the words they always carried — the
    // strip is their layout, never a second channel to them.
    let bold = node(&harness, Role::Button, "Bold").expect("the tool rides");
    assert_eq!(bold.role, Role::Button, "the button keeps its own press");
    let italic = node(&harness, Role::Button, "Italic").expect("the second tool rides");
    assert_eq!(italic.role, Role::Button, "the button keeps its own press");

    // The honest absences beside the tier: the neighbour's role is
    // exactly what it was before the tier existed.
    let body = node(&harness, Role::TextInput, "Body").expect("the input rides");
    assert_eq!(body.role, Role::TextInput, "the input keeps its own edit");
    harness.assert_all_named();
}

#[test]
fn m129_the_strip_walls_nothing() {
    // A toolbar is not a control: it holds no focus and answers no
    // activation — the tools' doors are the tools' own. The word names
    // the set the tools read as, it walls nothing (the M126 heading's
    // own rule, at the container tier).
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let strip = Toolbar::new("Formatting");
    let strip_id = harness.add_child(root, strip.boxed());
    let bold_id = harness.add_child(strip_id, Button::new("Bold").boxed());
    harness.settle();

    // The strip keeps its word in the snapshot, the tool its own.
    let formatting = node(&harness, Role::Toolbar, "Formatting").expect("the strip rides");
    assert_eq!(formatting.role, Role::Toolbar);
    let bold = node(&harness, Role::Button, "Bold").expect("the tool rides");
    assert_eq!(bold.role, Role::Button);

    // The focusable set the pipeline derives: the tool rides it, the
    // strip does not — the container is no control, the tier never
    // joining the focusable set.
    let focusables = harness.pipeline().focusables();
    assert!(
        !focusables.contains(&strip_id),
        "the container holds no focus — the tools hold the focus, never the strip"
    );
    assert!(
        focusables.contains(&bold_id),
        "the tool keeps its focus door — the word walls nothing"
    );
    assert!(bold.enabled, "the tool keeps its availability word");
    harness.assert_all_named();
}

#[test]
fn m129_the_corpus_pin_re_agrees_at_the_m129_cut() {
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

    // The published corpus pin re-agrees: the toolbar tier grew no
    // snapshot route, and its pricing door is the widgets crate's own
    // bench — outside the corpus at its own cut, joined by the corpus catch-up's published cut — the coordinated rule's next published cut cashed.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M129 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M129 cut"
    );
}
