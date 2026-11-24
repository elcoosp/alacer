//! M131 exit-criteria integration tests (§10.3, ADR-0135): the
//! tab-panel tier — the named pane the selected page reveals, said by
//! the tree. Through M130 the snapshot said the instant on/off control
//! (ADR-0134), the tool strip (ADR-0133), the bounded number field
//! (ADR-0132), the page selector and its strip (ADR-0131) — and the
//! strip's *content* still had no word: an app wiring a tab strip
//! mounted the pages (the member, M127) inside the named strip (the
//! container, M127) and showed the selection's content in a bare box —
//! a container wearing no name (the set the AT cannot name is a set it
//! cannot walk, the ADR-5 rule the named roles carry) or for a second
//! tab list wearing the wrong word (a tab list *names the pages' strip*;
//! a tab panel *names the page's content*, and the AT's vocabulary
//! names the difference: AccessKit's `TabPanel` role is its own fact).
//! [`alacer_core::Role::TabPanel`] carries the "holds the selected
//! page's content" fact, the member-and-container shape (ADR-0128)
//! grown to its third word: the member ([`Role::Tab`]), the strip
//! ([`Role::TabList`]), and now the pane ([`Role::TabPanel`]).
//!
//! The tier prices the word and nothing else: a mounted pane's children
//! keep the roles and labels they already carry — the button inside a
//! named pane presses, the label reads — and the pane itself is not a
//! control: it holds no focus and answers no activation (the M126/M129
//! container rule), the selection's exclusivity staying the app's
//! wiring exactly as the tab tier published it.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the pane's word rides the role mapping the sink
//! grows one branch beside the tab pair's.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, Tab, TabList, TabPanel};

/// The suite's corpus pin: the coordinated rule's count at the M131
/// cut — unchanged from the M130 cut. The tab-panel tier grows no
/// snapshot route (the word rides the role mapping the sink grows one
/// branch beside the tab pair's — no property, no second channel), and
/// its pricing door is the widget-level bench the tier carries in the
/// widgets crate (`tab_panel/name_write_256` against
/// `tab_panel/idle_write_256`), which rides *outside* the published
/// corpus: the published baseline's routes price the a11y snapshot
/// machinery (the M95 rule), and the coordinated rule grows it at the
/// next published cut, never at a widget bench's. The pin re-agrees at
/// 155.
const EXPECTED_BENCH_COUNT: usize = 163;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m131_the_pane_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The tab tier's third word, mounted through the constructor words:
    // the named strip, the pages it names, and the named pane the
    // selection's content lives in — every word present from the first
    // frame, none borrowed from a neighbour.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let strip_id = harness.add_child(root, TabList::new("Mailboxes").boxed());
    let _ = harness.add_child(strip_id, Tab::new("Inbox").selected(true).boxed());
    let _ = harness.add_child(strip_id, Tab::new("Sent").boxed());
    let _ = harness.add_child(root, TabPanel::new("Inbox content").boxed());
    let _ = harness.add_child(root, Label::new("Read it or leave it").boxed());
    harness.settle();

    // The pane: the "holds the selected page's content" fact AccessKit's
    // own TabPanel role spells — named, the way the member and the strip
    // are named.
    let pane =
        node(&harness, Role::TabPanel, "Inbox content").expect("the named pane rides the snapshot");
    assert_eq!(pane.role, Role::TabPanel);
    assert_eq!(pane.label, "Inbox content");

    // The tier's shape re-agrees beside the pane: the member and the
    // strip carry the words M127 published, untouched.
    let inbox = node(&harness, Role::Tab, "Inbox").expect("the selected page rides");
    assert_eq!(
        inbox.selected,
        Some(true),
        "the selection rides the M108 word — never a second one"
    );
    let sent = node(&harness, Role::Tab, "Sent").expect("the unselected page rides");
    assert_eq!(sent.selected, Some(false));
    node(&harness, Role::TabList, "Mailboxes").expect("the strip keeps its word");

    // The honest neighbour: the plain label beside the pane reads as it
    // always did — the pane's word took nothing from it.
    node(&harness, Role::Label, "Read it or leave it").expect("the plain label keeps its word");
    harness.assert_all_named();
}

#[test]
fn m131_the_pane_is_a_container_and_its_children_keep_their_words() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    let pane_id = harness.add_child(root, TabPanel::new("Inbox content").boxed());
    let _ = harness.add_child(pane_id, Button::new("Open message").boxed());
    let _ = harness.add_child(pane_id, Label::new("From Ada").boxed());
    harness.settle();

    // The pane is the content's layout, never a second channel to it:
    // the children ride the arena exactly as a `Column`'s do, each
    // keeping the role and label it already carries — the button
    // presses, the label reads, the pane naming what they read as.
    node(&harness, Role::Button, "Open message").expect("the pane's child keeps its role");
    node(&harness, Role::Label, "From Ada").expect("the pane's child keeps its role");

    // The pane is not a control: it holds no focus and answers no
    // activation — the M126/M129 container rule at the pane tier. The
    // focusable set carries the button alone.
    let focusables = harness.pipeline().focusables();
    assert!(
        !focusables.contains(&pane_id),
        "a named pane is the content's layout — the content holds the focus"
    );
    harness.assert_all_named();
}

#[test]
fn m131_the_pane_word_writes_move_the_word() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    let pane_id = harness.add_child(root, TabPanel::new("Inbox content").boxed());
    harness.settle();

    // The programmatic write (view reconciliation): the pane's word is
    // the app's declaration, renamed in place — the tree re-announces
    // the new word the next frame, the way the strip's rename reads.
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(pane_id);
        guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<TabPanel>()
            .expect("the pane mounts as itself")
            .set_name("Sent content");
        guard.request(alacer_core::Change::A11Y);
    }
    harness.settle();
    let pane = node(&harness, Role::TabPanel, "Sent content")
        .expect("the rename landed — the tree carries the new word");
    assert_eq!(pane.label, "Sent content");
    assert!(
        node(&harness, Role::TabPanel, "Inbox content").is_none(),
        "the old word is gone — one word per pane"
    );
    harness.assert_all_named();
}

#[test]
fn m131_the_corpus_pin_re_agrees_at_the_m131_cut() {
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

    // The published corpus pin re-agrees: the tab-panel tier grew no
    // snapshot route (the word rides the role mapping, one branch beside
    // the tab pair's), and its pricing door is the widgets crate's own
    // bench — outside the published corpus, the coordinated rule growing
    // it at the next published cut.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M131 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M131 cut"
    );
}
