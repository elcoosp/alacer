//! M120 exit-criteria integration tests (§10.3, ADR-0124): the live
//! tier — the declared live region, said by the tree. Through M119 the
//! snapshot said the member's place and the collection's size (the set
//! tier, ADR-0123), what a control is read as (the role-description
//! tier, ADR-0120), the subtree's updating fact (the busy tier,
//! ADR-0121) — and the word every streaming vocabulary speaks about
//! the region whose changes the AT speaks ("this region is live —
//! announce what lands here") had no app-facing tier: the AT spoke the
//! pipeline's own reserved region (§11.6, ADR-0024) and never a region
//! the app declared — the declared-but-unreported gap (the M94
//! pattern), cashed at the live tier: the word grows beside the
//! announcement machinery it extends — [`alacer_core::Widget::live`]
//! beside [`alacer_widgets::List::live`] and
//! [`alacer_widgets::Table::live`], AccessKit's `Live` property (ARIA's
//! `aria-live`) the sink maps in the same snapshot every other
//! property rides, the politeness the announcement seam's own
//! vocabulary ([`alacer_core::announce::Politeness`]) — the declared
//! half of the reserved region's imperative one.
//!
//! The tier prices the word and nothing else: a live region's doors
//! answer exactly as they always did — the word names the region the
//! AT watches, it walls nothing, the containers' routes the widgets'
//! own, and the imperative announcement channel is the channel it
//! always was (the word never silences the tree's routes — the AT
//! decides what it speaks, the M117 rule at the live tier). `None` is
//! the honest absence (the ADR-0087 presence rule at the tier): a
//! container that declared nothing announces no region, never a
//! guessed politeness.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::announce::Politeness;
use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, List, Table};

/// The suite's corpus pin: the coordinated rule's count at the M120 cut —
/// the live route's case joining the set route's pair (M119), the busy
/// route's pair (M117), the role-description route's case (M116), the
/// has-popup route's case (M115), the current route's pair (M114), the
/// level route's case (M113), the sort route's pair (M112), the
/// auto-complete route's case (M111), the expanded route's pair (M110),
/// the multiselectable route's pair (M109) and every tier's before them
/// on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 159;

/// The suite's own read door into the published corpus: the live
/// route's case — the 1,000-node list arena carrying the declared
/// region's word (the politeness cycling down the stream), priced
/// against the busy route's published bare arena (the M111 precedent's
/// reuse: the M117 cut measured the same 1,000-node list arena bare,
/// the tier's honest denominator — the word is a container word, so
/// the honest comparison is lists against lists, the M108/M109
/// denominator shape).
const LIVE_CASE: &str = "live/snapshot_1k_lists_live";
const LISTS_BARE_CASE: &str = "busy/snapshot_1k_lists_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's live door: the mutate seam the app's code rides — the word
/// moved, `PAINT | A11Y` requested on the container, the frames the
/// re-announcement rides settled before the read.
fn set_container_live(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    politeness: Option<Politeness>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(list) = guard.widget_mut().as_any_mut().downcast_mut::<List>() {
        list.set_live(politeness);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m120_the_live_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 320.0);
    // The declared regions, mounted through the constructor word: the
    // fact the tree says from the first frame — the feed the AT speaks
    // politely, the ledger the AT interrupts for — beside the settled
    // containers and the plain widgets, the honest absence the tier
    // keeps honest.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let feed = List::new(80.0).live(Politeness::Polite);
    let feed_id = harness.add_child(root, feed.boxed());
    let _ = harness.add_child(feed_id, Label::new("Row 1").boxed());
    let ledger = Table::new("Ledger", 2).live(Politeness::Assertive);
    let ledger_id = harness.add_child(root, ledger.boxed());
    let _ = harness.add_child(ledger_id, Label::new("Entry 1").boxed());
    let archive = Table::new("Archive", 2);
    let archive_id = harness.add_child(root, archive.boxed());
    let _ = harness.add_child(archive_id, Label::new("Entry 2").boxed());
    // The plain widgets beside them: no declared region to say.
    let _ = harness.add_child(root, Button::new("Save").boxed());
    let _ = harness.add_child(root, Label::new("A plain label").boxed());
    harness.settle();

    let feed_node = node(&harness, Role::ScrollArea, "list").expect("the live list rides");
    assert_eq!(
        feed_node.live,
        Some(Politeness::Polite),
        "the constructor's word is the tree's truth — the declared region said"
    );
    let ledger_node = node(&harness, Role::Table, "Ledger").expect("the live table rides");
    assert_eq!(
        ledger_node.live,
        Some(Politeness::Assertive),
        "the table's constructor word lands in the same snapshot — the tier's second producer"
    );
    let archive_node = node(&harness, Role::Table, "Archive").expect("the settled table rides");
    assert_eq!(
        archive_node.live, None,
        "a container that declared nothing announces no region — the honest absence"
    );
    let save = node(&harness, Role::Button, "Save").expect("the button rides");
    assert_eq!(
        save.live, None,
        "a non-container role inherits the honest absence"
    );
    // The word prices nothing but the word: the worded region's
    // neighbours answer exactly as they always did — the declared
    // region names a fact, it walls nothing.
    assert!(
        harness.act(Role::Button, "Save", A11yAction::DefaultPress),
        "the press route answers unchanged beside the container's word"
    );
    harness.assert_all_named();
}

#[test]
fn m120_the_live_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let feed = List::new(80.0);
    let feed_id = harness.add_child(root, feed.boxed());
    let _ = harness.add_child(feed_id, Label::new("Row 1").boxed());
    harness.settle();

    // The live door: the app's declared region moved, the
    // re-announcement the mutate seam's request paid for — and the
    // honest absence restored on the clear (the region withdrawn, the
    // container the widget it always was).
    set_container_live(&mut harness, feed_id, Some(Politeness::Assertive));
    let declared = node(&harness, Role::ScrollArea, "list").expect("the list survives");
    assert_eq!(
        declared.live,
        Some(Politeness::Assertive),
        "the app's door moves the word the tree says"
    );
    set_container_live(&mut harness, feed_id, None);
    let withdrawn = node(&harness, Role::ScrollArea, "list").expect("the list survives");
    assert_eq!(
        withdrawn.live, None,
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(feed_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<List>()
        .expect("the list survives")
        .set_live(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m120_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the live route's case: the
    // 1,000-node list arena carrying the declared region's word — the
    // word's cost the measurement, priced against the busy route's
    // published bare arena (the same lists, the words absent).
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(LIVE_CASE) && benches.contains_key(LISTS_BARE_CASE),
        "the live route's case is published beside its reused denominator ({LIVE_CASE}, {LISTS_BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M120 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [LIVE_CASE, LISTS_BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("live_snapshot_overhead"))
        .expect("the live budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(LIVE_CASE),
        "the budget prices the live route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(LISTS_BARE_CASE),
        "the budget prices it against the same list arena, bare"
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
        "the m5 corpus pin re-agrees at the M120 cut"
    );
}
