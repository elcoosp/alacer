//! M121 exit-criteria integration tests (§10.3, ADR-0125): the shortcut
//! tier — the chord the keymap rides, said by the tree. Through M120 the
//! snapshot said the declared live region (ADR-0124), the collection's
//! place and span (ADR-0123), what a control is read as (ADR-0120), the
//! subtree's updating fact (ADR-0121), what a control opens (ADR-0119),
//! the form facts, the collection facts — and the word every
//! menu-and-toolbar vocabulary speaks about *how a control is invoked
//! from the keyboard* ("Save — Control+S") had no tier: the keymap names
//! its bindings to the causal trail (M37, ADR-0043's consult) and the
//! platform menus carry their accelerators — while the control the AT
//! walks to announced its name and never its chord — the
//! declared-but-unreported gap (the M94 pattern), cashed at the shortcut
//! tier: the word grows beside the keymap seam it mirrors —
//! [`alacer_core::Widget::keyboard_shortcut`] beside
//! [`alacer_widgets::Button::keyboard_shortcut`], AccessKit's
//! `KeyboardShortcut` property (ARIA's `aria-keyshortcuts`) the sink maps
//! in the same snapshot every other property rides.
//!
//! The tier prices the word and nothing else: a worded control's doors
//! answer exactly as they always did — the word names the chord, it
//! walls nothing, the control's press and focus routes the widgets' own,
//! and the keymap's dispatch is the dispatch it always was (the word
//! never binds a key — the keymap seam owns the chords, the word says
//! the one the app named). `None` is the honest absence (the ADR-0087
//! presence rule at the tier): a control that declared no chord
//! announces none, never a guessed binding.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label};

/// The suite's corpus pin: the coordinated rule's count at the M121 cut —
/// the shortcut route's case joining the current route's pair (M114),
/// the live route's case (M120), the busy route's pair (M117), the
/// role-description route's case (M116), the has-popup route's case
/// (M115), the set route's pair (M119) and every tier's before them on
/// the published corpus.
const EXPECTED_BENCH_COUNT: usize = 151;

/// The suite's own read door into the published corpus: the shortcut
/// route's case — the 1,000-node button arena carrying the constructor's
/// chord (the word cycling the platform spellings down the stream, the
/// way a toolbar's buttons alternate) — against the current route's
/// published bare arena (`current/snapshot_1k_buttons_bare`, the M114
/// case the tier reuses by the M111 precedent; the M95 pricing rule —
/// the word is a button word, so the honest comparison is buttons
/// against buttons, the M108/M120 denominator shape).
const SHORTCUT_CASE: &str = "shortcut/snapshot_1k_buttons_shortcut";
const BUTTONS_BARE_CASE: &str = "current/snapshot_1k_buttons_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's shortcut door: the mutate seam the app's code rides — the
/// chord moved, `PAINT | A11Y` requested on the control, the frames the
/// re-announcement rides settled before the read.
fn set_button_shortcut(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    shortcut: Option<&str>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_keyboard_shortcut(shortcut.map(str::to_owned));
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m121_the_shortcut_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The worded controls, mounted through the constructor word: the
    // fact the tree says from the first frame — the save chord the AT
    // announces beside the name, the print chord beside its own —
    // beside the plain widgets, the honest absence the tier keeps
    // honest.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(
        root,
        Button::new("Save").keyboard_shortcut("Control+S").boxed(),
    );
    let _ = harness.add_child(
        root,
        Button::new("Print")
            .keyboard_shortcut("Control+Shift+P")
            .boxed(),
    );
    // The plain widgets beside them: no declared chord to say.
    let _ = harness.add_child(root, Button::new("Close").boxed());
    let _ = harness.add_child(root, Label::new("A plain label").boxed());
    harness.settle();

    let save = node(&harness, Role::Button, "Save").expect("the worded save rides");
    assert_eq!(
        save.keyboard_shortcut.as_deref(),
        Some("Control+S"),
        "the constructor's word is the tree's truth — the chord said beside the name"
    );
    let print = node(&harness, Role::Button, "Print").expect("the worded print rides");
    assert_eq!(
        print.keyboard_shortcut.as_deref(),
        Some("Control+Shift+P"),
        "the second producer's chord lands in the same snapshot"
    );
    let close = node(&harness, Role::Button, "Close").expect("the plain button rides");
    assert_eq!(
        close.keyboard_shortcut, None,
        "a control that declared no chord announces none — the honest absence"
    );
    // The word prices nothing but the word: the worded control's
    // neighbours answer exactly as they always did — the chord names a
    // fact, it walls nothing.
    assert!(
        harness.act(Role::Button, "Save", A11yAction::DefaultPress),
        "the press route answers unchanged beside the worded control's chord"
    );
    assert!(
        harness.act(Role::Button, "Close", A11yAction::DefaultPress),
        "the unworded control's press route answers as it always did"
    );
    harness.assert_all_named();
}

#[test]
fn m121_the_shortcut_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let export = Button::new("Export");
    let export_id = harness.add_child(root, export.boxed());
    harness.settle();

    // The shortcut door: the app's chord moved, the re-announcement the
    // mutate seam's request paid for — and the honest absence restored
    // on the clear (the chord withdrawn, the control the widget it
    // always was).
    set_button_shortcut(&mut harness, export_id, Some("Control+E"));
    let named = node(&harness, Role::Button, "Export").expect("the button survives");
    assert_eq!(
        named.keyboard_shortcut.as_deref(),
        Some("Control+E"),
        "the app's door moves the word the tree says"
    );
    set_button_shortcut(&mut harness, export_id, None);
    let withdrawn = node(&harness, Role::Button, "Export").expect("the button survives");
    assert_eq!(
        withdrawn.keyboard_shortcut, None,
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(export_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Button>()
        .expect("the button survives")
        .set_keyboard_shortcut(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m121_the_corpus_grew_by_the_coordinated_rule() {
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
    let contracts = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/contracts.json"))
            .expect("the contract set is readable"),
    )
    .expect("the contract set is valid JSON");

    // The published corpus carries the shortcut route's case: the
    // 1,000-node button arena carrying the constructor's chord — the
    // word's cost the measurement, priced against the current route's
    // published bare arena (the same buttons, the chords absent).
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(SHORTCUT_CASE) && benches.contains_key(BUTTONS_BARE_CASE),
        "the shortcut route's case is published beside its reused denominator ({SHORTCUT_CASE}, {BUTTONS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M121 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [SHORTCUT_CASE, BUTTONS_BARE_CASE] {
        let gate = gates
            .iter()
            .find(|gate| gate["baseline"].as_str() == Some(case))
            .unwrap_or_else(|| panic!("the case carries a regression gate ({case})"));
        assert_eq!(
            gate["scope"].as_str(),
            Some("same-host"),
            "the gate is same-host scoped like the tier's own"
        );
    }
    let budgets = contracts["budgets"]
        .as_array()
        .expect("the contract set carries budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"].as_str() == Some("shortcut_snapshot_overhead"))
        .expect("the shortcut budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(SHORTCUT_CASE),
        "the budget prices the shortcut route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BUTTONS_BARE_CASE),
        "the budget prices it against the same button arena, bare"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the word must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M121 cut"
    );
}
