//! M127 exit-criteria integration tests (§10.3, ADR-0131): the tab
//! tier — the page selector and the named strip it reads as, said by
//! the tree. Through M126 the snapshot said the section title the
//! document outline walks (ADR-0130), the hyperlink and its open verb
//! (ADR-0129), the one-of-N control and the group it reads as
//! (ADR-0128), the set words, the form facts, the collection facts,
//! the declared live region, the chord, the capacity, the language —
//! and the tab strip had no tier: an app wiring one reached for a row
//! of buttons (the wrong shape — a button's activation is a *press*
//! with no after-state, a tab's activation *selects the page*) or for
//! bare labels wearing no selection fact at all. The tier ships two
//! roles beside one machinery: [`alacer_core::Role::Tab`] carrying the
//! "selects a page" fact AccessKit's own `Tab` role spells, the
//! selection riding the selected word the tree already carries (the
//! M108 contract, ADR-0112), and [`alacer_core::Role::TabList`] the
//! named strip the set reads as — the radio tier's own
//! member-and-container shape (ADR-0128) at the page tier.
//!
//! The tier prices the pages and nothing else: a mounted strip's
//! neighbours answer exactly as they always did — the button beside it
//! presses, the heading beside it names, the tab's doors are the
//! select's. An unavailable tab refuses every door honestly (the M22
//! wall, the M11 discipline); a redundant ask is the M21 idle frame;
//! the exclusivity is the app's wiring — the framework never rewrites
//! widgets it does not own (A1).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's doors ride the mutate seam and the
//! dispatch funnel whose `PAINT | A11Y` request re-announces exactly
//! the tab's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Heading, Tab, TabList};

/// The suite's corpus pin: the coordinated rule's count at the M127
/// cut — unchanged from the M126 cut. The tab tier grows no snapshot
/// route (its word is the selected tier's, published since M108), and
/// its pricing door is the widget-level bench the tier carries in the
/// widgets crate (`tab/select_route_256` against
/// `tab/redundant_ask_256`), which rides *outside* the published
/// corpus: the published baseline's routes price the a11y snapshot
/// machinery (the M95 rule), and the coordinated rule grows it at the
/// next published cut, never at a widget bench's. The pin re-agrees at
/// 155.
const EXPECTED_BENCH_COUNT: usize = 204;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m127_the_tab_words_land_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The strip, mounted through the constructor words: the named
    // container the AT announces before it walks the pages, the
    // selected page beside the unselected one — the selection fact the
    // tree carries — and the plain widgets the tier keeps honest. No
    // tab word leaks onto a button or a heading.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Heading::new("Mail").level(1).boxed());
    let strip = TabList::new("Mailboxes");
    let strip_id = harness.add_child(root, strip.boxed());
    let _ = harness.add_child(strip_id, Tab::new("Inbox").selected(true).boxed());
    let _ = harness.add_child(strip_id, Tab::new("Archive").boxed());
    let _ = harness.add_child(root, Button::new("Compose").boxed());
    harness.settle();

    // The strip: the "reads as a set of pages" fact the container role
    // spells, the name the ADR-5 contract requires.
    let mailboxes = node(&harness, Role::TabList, "Mailboxes").expect("the strip rides");
    assert_eq!(
        mailboxes.role,
        Role::TabList,
        "the strip keeps its own word"
    );
    // The pages: the "selects a page" fact the member role spells, the
    // selection the selected tier's word carries — picked and unpicked
    // exactly as mounted.
    let inbox = node(&harness, Role::Tab, "Inbox").expect("the selected page rides");
    assert_eq!(inbox.selected, Some(true), "the mounted selection said");
    let archive = node(&harness, Role::Tab, "Archive").expect("the unselected page rides");
    assert_eq!(
        archive.selected,
        Some(false),
        "an unselected page announces the fact, never the absence"
    );

    // The honest absences beside the tier: the neighbours' roles are
    // exactly what they were before the tier existed.
    let mail = node(&harness, Role::Heading, "Mail").expect("the heading rides");
    assert_eq!(mail.role, Role::Heading, "the heading keeps its own word");
    let compose = node(&harness, Role::Button, "Compose").expect("the button rides");
    assert_eq!(compose.role, Role::Button, "the button keeps its own press");
    harness.assert_all_named();
}

#[test]
fn m127_the_select_door_selects_and_the_wall_refuses() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let strip = TabList::new("Mailboxes");
    let strip_id = harness.add_child(root, strip.boxed());
    let _ = harness.add_child(strip_id, Tab::new("Inbox").selected(true).boxed());
    let _ = harness.add_child(strip_id, Tab::new("Archive").boxed());
    let _ = harness.add_child(strip_id, Tab::new("Junk").enabled(false).boxed());
    harness.settle();

    // The AT's ask selects: the tab's door consumes the activation and
    // the page becomes the selection — the tree re-announces exactly
    // the tab's node.
    assert!(
        harness.act(Role::Tab, "Archive", A11yAction::DefaultPress),
        "the tab's door answers the select ask"
    );
    harness.settle();
    let archive = node(&harness, Role::Tab, "Archive").expect("the moved page rides");
    assert_eq!(archive.selected, Some(true), "the ask selected the page");

    // The exclusivity is the app's wiring: the framework never rewrites
    // widgets it does not own (A1) — the previously selected page stays
    // selected until the strip's writer clears it through the view's
    // programmatic door, the one deselect door a tab has.
    let inbox = node(&harness, Role::Tab, "Inbox").expect("the previous page rides");
    assert_eq!(
        inbox.selected,
        Some(true),
        "the sibling keeps its word — the app's wiring is the exclusivity"
    );

    // The M22 wall, at the tab tier: the unavailable tab's ask is
    // refused before it is parsed.
    assert!(
        !harness.act(Role::Tab, "Junk", A11yAction::DefaultPress),
        "an unavailable tab has no write door"
    );
    harness.settle();
    harness.assert_all_named();
}

#[test]
fn m127_the_corpus_pin_re_agrees_at_the_m127_cut() {
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

    // The published corpus pin re-agrees: the tab tier grew no snapshot
    // route, and its pricing door is the widgets crate's own bench —
    // outside the corpus at its own cut, joined by the corpus catch-up's published cut — the coordinated rule's next published cut cashed.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M127 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M127 cut"
    );
}
