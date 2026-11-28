//! M156 exit-criteria integration tests (§11.5, ADR-0161): the action
//! tier — the asks a control's door answers, said by the tree. Through
//! M155 the snapshot said what a control *is*, its name, its value, its
//! bounds, whether it is available, read-only, required or selected,
//! what it opens, the chord it is invoked with — and the asks its door
//! *answers* had no tier: the platform mapped ten AccessKit actions onto
//! `on_a11y_action`'s vocabulary (M10, ADR-0016; M11, ADR-0017; M91,
//! ADR-0095) and every canon control answered or refused each ask
//! honestly, while the node the AT walked to advertised none of them —
//! `alacer-a11y` never called `add_action`, and the AccessKit consumer
//! derives clickability and the step support from that call alone.
//!
//! The plan recorded the deferral and its trigger. ADR-0016 §3: "No
//! per-action capability advertisement (AccessKit `supports_*` actions):
//! refusal is honest and observable today; advertising is a widget-canon
//! sweep once more actions exist." ADR-0017 §3 carries the same line;
//! ADR-0023 §3 names the vehicle — "No live-region, pointer-capture,
//! `supports_*`, or `Slider` vocabulary — each rides M18+ with its own
//! corpus cycle." The actions exist (ten, every one with a consuming
//! widget); the declared-but-unreported gap (the M94 pattern) is cashed
//! at the action tier: [`alacer_core::Widget::a11y_actions`] beside
//! [`alacer_core::Widget::on_a11y_action`], the sink mapping each
//! advertised ask onto AccessKit's action set in the same snapshot every
//! other property rides.
//!
//! The word is the door's own answer said before the ask: a control
//! advertises exactly the asks its door consumes in the state it is in —
//! an unavailable control advertises none (the M22 rule reaching the
//! word: a closed door is no capability), a read-only field the selection
//! ask alone (M105). The word says the door and opens nothing: an ask
//! still travels the route it always travelled and the door still
//! answers each ask on its own merits.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the sink's update is the platform's own read, and the
//! app's door rides the mutate seam whose `PAINT | A11Y` request
//! re-announces exactly the node whose word moved.

use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::color::Color;
use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, A11yActions, Role, ScrollUnit, TextSelection};
use alacer_testing::TestHarness;
use alacer_widgets::{
    BoxedWidget as _, Button, Canvas, Checkbox, Column, Label, Link, List, Menu, MenuBar, MenuItem,
    Meter, Panel, Progress, Radio, Slider, SpinButton, Switch, Tab, TabList, TextInput,
};

/// The suite's corpus pin: the coordinated rule's count at the M156 cut
/// — the action tier's sink case joining the corpus the M155 cuts
/// published (the conditional pair's gates the last to ride).
const EXPECTED_BENCH_COUNT: usize = 208;

/// The suite's own read door into the published corpus: the sink's
/// ceiling case — the 1,000-node snapshot with every node advertising
/// the whole vocabulary — against the published bare rebuild
/// (`a11y/full_rebuild_1k`, the M9 case the tier reuses; the honest
/// comparison is the same tree with and without the word).
const ACTIONS_CASE: &str = "a11y/full_rebuild_1k_actions";
const BARE_REBUILD_CASE: &str = "a11y/full_rebuild_1k";

/// The ten AccessKit actions the platform maps onto the vocabulary (the
/// inverse of the sink's mapping): the set an AT can ask for, in the
/// order the vocabulary spells them.
const PLATFORM_ASKS: [accesskit::Action; 10] = [
    accesskit::Action::Click,
    accesskit::Action::Increment,
    accesskit::Action::Decrement,
    accesskit::Action::ScrollUp,
    accesskit::Action::ScrollDown,
    accesskit::Action::ScrollLeft,
    accesskit::Action::ScrollRight,
    accesskit::Action::SetValue,
    accesskit::Action::ReplaceSelectedText,
    accesskit::Action::SetTextSelection,
];

/// One well-formed ask per kind the vocabulary carries — the ask a
/// reader sends when it means exactly that kind, payload and all. Row
/// `i` names the kind the sink maps onto `PLATFORM_ASKS[i]`.
fn every_ask() -> [(A11yActions, A11yAction); 10] {
    [
        (A11yActions::DEFAULT_PRESS, A11yAction::DefaultPress),
        (A11yActions::INCREMENT, A11yAction::Increment),
        (A11yActions::DECREMENT, A11yAction::Decrement),
        (
            A11yActions::SCROLL_UP,
            A11yAction::ScrollUp(ScrollUnit::Item),
        ),
        (
            A11yActions::SCROLL_DOWN,
            A11yAction::ScrollDown(ScrollUnit::Item),
        ),
        (
            A11yActions::SCROLL_LEFT,
            A11yAction::ScrollLeft(ScrollUnit::Item),
        ),
        (
            A11yActions::SCROLL_RIGHT,
            A11yAction::ScrollRight(ScrollUnit::Item),
        ),
        (
            A11yActions::SET_VALUE,
            A11yAction::SetValue("42".to_owned()),
        ),
        (
            A11yActions::REPLACE_SELECTED_TEXT,
            A11yAction::ReplaceSelectedText("x".to_owned()),
        ),
        (
            A11yActions::SET_TEXT_SELECTION,
            A11yAction::SetTextSelection(TextSelection::new(0, 0)),
        ),
    ]
}

/// The value tier's three asks: the write and the two steps.
fn value_asks() -> A11yActions {
    A11yActions::SET_VALUE
        .union(A11yActions::INCREMENT)
        .union(A11yActions::DECREMENT)
}

/// The scroll tier's six asks: the four travel asks and the two page
/// steps.
fn scroll_asks() -> A11yActions {
    A11yActions::SCROLL_UP
        .union(A11yActions::SCROLL_DOWN)
        .union(A11yActions::SCROLL_LEFT)
        .union(A11yActions::SCROLL_RIGHT)
        .union(A11yActions::INCREMENT)
        .union(A11yActions::DECREMENT)
}

/// The text tier's three asks: the two writes and the selection.
fn text_asks() -> A11yActions {
    A11yActions::SET_VALUE
        .union(A11yActions::REPLACE_SELECTED_TEXT)
        .union(A11yActions::SET_TEXT_SELECTION)
}

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> TreeNode {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name))
        .cloned()
        .unwrap_or_else(|| panic!("no {role} named {name:?} is reachable"))
}

/// The canon under one root: every control whose door answers an ask,
/// beside the reports whose doors answer none. Each keeps the name its
/// constructor gave it.
fn mount_canon(harness: &mut TestHarness) {
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    let _ = harness.add_child(root, Checkbox::new("Bold").boxed());
    let _ = harness.add_child(root, Radio::new("Small").boxed());
    let _ = harness.add_child(root, Switch::new("Wi-Fi").boxed());
    let _ = harness.add_child(root, Link::new("Site").boxed());
    let _ = harness.add_child(root, Slider::new("Volume", 0.0, 100.0).boxed());
    let _ = harness.add_child(root, SpinButton::new("Count", 0.0, 10.0).step(1.0).boxed());
    let _ = harness.add_child(root, TextInput::new("Name").boxed());
    let list = harness.add_child(root, List::new(160.0).boxed());
    for _ in 0..4 {
        let _ = harness.add_child(list, Panel::new(28.0, Color::rgb(59, 130, 246)).boxed());
    }
    let strip = harness.add_child(root, TabList::new("Mailboxes").boxed());
    let _ = harness.add_child(strip, Tab::new("Inbox").boxed());
    let bar = harness.add_child(root, MenuBar::new("Main").boxed());
    let menu = harness.add_child(bar, Menu::new("File").boxed());
    let _ = harness.add_child(menu, MenuItem::new("Open").boxed());
    let _ = harness.add_child(root, Progress::new("Loading", 0.0, 100.0).boxed());
    let _ = harness.add_child(root, Meter::new("Disk", 0.0, 100.0).boxed());
    let _ = harness.add_child(root, Canvas::new("Chart").boxed());
    let _ = harness.add_child(root, Label::new("A plain label").boxed());
    harness.settle();
}

/// The exact mechanism the platform loop calls at frame boundaries:
/// reach the pipeline's erased a11y sink, downcast, delegate (the m9
/// suite's own helper, the windowed composition).
fn with_sink<R>(harness: &mut TestHarness, f: impl FnOnce(&mut AccessKitSink) -> R) -> R {
    let sink = harness
        .pipeline_mut()
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<AccessKitSink>())
        .expect("the harness carries the AccessKit sink — the windowed composition");
    f(sink)
}

/// The AccessKit actions a platform node advertises, among the ten the
/// platform maps — read the way the consumer reads them.
fn advertised(node: &accesskit::Node) -> Vec<accesskit::Action> {
    PLATFORM_ASKS
        .iter()
        .copied()
        .filter(|action| node.supports_action(*action))
        .collect()
}

/// The platform's own node for a control in the latest update that
/// carries it — the node the adapter hands an AT after the frame.
fn platform_node(update: &accesskit::TreeUpdate, id: u64) -> Option<&accesskit::Node> {
    find_in_update(update, id)
}

/// The equivalence the tier exists for, asked ask by ask: the tree's
/// word and the door's answer agree on every kind the vocabulary
/// carries. A well-formed ask rides each probe, so a refusal is the
/// door's own and never a malformed payload's.
fn assert_word_is_the_answer(harness: &mut TestHarness, role: Role, name: &str) {
    for (_, ask) in every_ask() {
        let kind = ask.advertises();
        let said = node(harness, role, name).actions.contains(kind);
        let answered = harness.act(role, name, ask);
        assert_eq!(
            said,
            answered,
            "{role} {name:?}: the tree {} the ask, the door {} it — the word is the door's answer",
            if said { "advertises" } else { "withholds" },
            if answered { "answers" } else { "refuses" },
        );
    }
}

#[test]
fn m156_the_tree_says_the_asks_each_control_answers() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 640.0);
    // The canon sweep ADR-0016 §3 named: every control whose door
    // answers an ask says so from the first frame — the press on the
    // actionable vocabulary, the value tier's three asks on the two
    // value controls, the text tier's three on the field, the scroll
    // tier's six on the list — and the reports whose doors answer
    // nothing say nothing, their values the app's to write.
    mount_canon(&mut harness);
    let press = A11yActions::DEFAULT_PRESS;
    for (role, name) in [
        (Role::Button, "Save"),
        (Role::CheckBox, "Bold"),
        (Role::Radio, "Small"),
        (Role::Switch, "Wi-Fi"),
        (Role::Link, "Site"),
        (Role::Tab, "Inbox"),
        (Role::MenuItem, "Open"),
    ] {
        assert_eq!(
            node(&harness, role, name).actions,
            press,
            "{role} {name:?} advertises the press and nothing else"
        );
    }
    assert_eq!(
        node(&harness, Role::Slider, "Volume").actions,
        value_asks(),
        "the slider advertises the value tier's three asks"
    );
    assert_eq!(
        node(&harness, Role::SpinButton, "Count").actions,
        value_asks(),
        "the spin button advertises the value tier's three asks"
    );
    assert_eq!(
        node(&harness, Role::TextInput, "Name").actions,
        text_asks(),
        "the field advertises the two writes and the selection"
    );
    assert_eq!(
        node(&harness, Role::ScrollArea, "list").actions,
        scroll_asks(),
        "the list advertises the scroll tier's six asks"
    );
    for (role, name) in [
        (Role::ProgressIndicator, "Loading"),
        (Role::Meter, "Disk"),
        (Role::Label, "A plain label"),
    ] {
        assert_eq!(
            node(&harness, role, name).actions,
            A11yActions::empty(),
            "{role} {name:?} answers nothing and advertises nothing — the honest absence"
        );
    }
    assert!(
        node(&harness, Role::Canvas, "Chart").actions.is_empty(),
        "the canvas answers nothing and advertises nothing"
    );
    harness.assert_all_named();
}

#[test]
fn m156_the_word_is_the_doors_answer_ask_for_ask() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 640.0);
    mount_canon(&mut harness);
    for (role, name) in [
        (Role::Button, "Save"),
        (Role::CheckBox, "Bold"),
        (Role::Radio, "Small"),
        (Role::Switch, "Wi-Fi"),
        (Role::Link, "Site"),
        (Role::Tab, "Inbox"),
        (Role::MenuItem, "Open"),
        (Role::Slider, "Volume"),
        (Role::SpinButton, "Count"),
        (Role::TextInput, "Name"),
        (Role::ScrollArea, "list"),
        (Role::ProgressIndicator, "Loading"),
        (Role::Meter, "Disk"),
        (Role::Label, "A plain label"),
    ] {
        assert_word_is_the_answer(&mut harness, role, name);
    }
}

#[test]
fn m156_an_unavailable_control_advertises_none_and_a_read_only_field_its_selection() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 240.0);
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Button::new("Save").enabled(false).boxed());
    let _ = harness.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0).enabled(false).boxed(),
    );
    let _ = harness.add_child(root, TextInput::new("Name").enabled(false).boxed());
    let _ = harness.add_child(root, TextInput::new("Notes").read_only().boxed());
    harness.settle();

    // The M22 rule reaching the word: a closed door is no capability —
    // the unavailable controls advertise none of their asks, and the
    // tree's `disabled` flag beside the empty word says why.
    for (role, name) in [
        (Role::Button, "Save"),
        (Role::Slider, "Volume"),
        (Role::TextInput, "Name"),
    ] {
        let withdrawn = node(&harness, role, name);
        assert!(
            withdrawn.actions.is_empty(),
            "{role} {name:?}: an unavailable control advertises none"
        );
        assert!(
            !withdrawn.enabled,
            "{role} {name:?}: the tree carries the flag that closed the door"
        );
    }
    // The M105 rule reaching the word: a read-only field reads — the
    // selection ask answers, the two writes are refused on the declared
    // word, and the tree advertises exactly that.
    assert_eq!(
        node(&harness, Role::TextInput, "Notes").actions,
        A11yActions::SET_TEXT_SELECTION,
        "a read-only field advertises the selection ask alone"
    );
    for (role, name) in [
        (Role::Button, "Save"),
        (Role::Slider, "Volume"),
        (Role::TextInput, "Name"),
        (Role::TextInput, "Notes"),
    ] {
        assert_word_is_the_answer(&mut harness, role, name);
    }
}

#[test]
fn m156_the_availability_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let save = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();
    assert_eq!(
        node(&harness, Role::Button, "Save").actions,
        A11yActions::DEFAULT_PRESS,
        "the available control advertises the press"
    );

    // The availability door: the flag moved through the mutate seam, the
    // `PAINT | A11Y` request paying the re-announcement — the platform's
    // node re-emitted with the word withdrawn, then restored.
    assert!(
        set_enabled(&mut harness, save, false),
        "the availability write moved the flag"
    );
    assert!(
        node(&harness, Role::Button, "Save").actions.is_empty(),
        "the closed door withdraws the word the tree says"
    );
    let save_id = node(&harness, Role::Button, "Save").id;
    let narrowed = with_sink(&mut harness, AccessKitSink::drain_updates);
    let re_emitted = narrowed
        .iter()
        .rev()
        .find_map(|update| platform_node(update, save_id))
        .expect("the narrowed control is re-emitted");
    assert!(
        !advertised(re_emitted).contains(&accesskit::Action::Click),
        "the platform's node advertises no click on the closed door"
    );
    assert!(
        set_enabled(&mut harness, save, true),
        "the availability write moved the flag back"
    );
    assert_eq!(
        node(&harness, Role::Button, "Save").actions,
        A11yActions::DEFAULT_PRESS,
        "the reopened door restores the word"
    );
    // The write that lands where the flag already is queues nothing: the
    // door answers `false`, the caller requests no re-announcement.
    assert!(
        !set_enabled(&mut harness, save, true),
        "the write that lands where the flag already is queues nothing — the M21 idle-frame property"
    );
    assert_eq!(
        node(&harness, Role::Button, "Save").actions,
        A11yActions::DEFAULT_PRESS,
        "the word survives the no-op write"
    );
    harness.assert_all_named();
}

/// The app's availability door: the mutate seam the app's code rides —
/// the flag moved, `PAINT | A11Y` requested on the control when the
/// write moved it, the frames the re-announcement rides settled before
/// the read.
fn set_enabled(harness: &mut TestHarness, node_id: alacer_core::NodeId, enabled: bool) -> bool {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    let moved = if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_enabled(enabled)
    } else {
        false
    };
    if moved {
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
    moved
}

#[test]
fn m156_the_sink_advertises_what_the_tree_says() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 640.0);
    mount_canon(&mut harness);
    // The platform's own read: the full tree the sink answers an
    // adapter's activation with — the same node builder every delta
    // rides, the node the AT walks.
    let full = with_sink(&mut harness, |sink| sink.full_tree_update())
        .expect("the sink resolved the tree");

    for (role, name) in [
        (Role::Button, "Save"),
        (Role::CheckBox, "Bold"),
        (Role::Radio, "Small"),
        (Role::Switch, "Wi-Fi"),
        (Role::Link, "Site"),
        (Role::Tab, "Inbox"),
        (Role::MenuItem, "Open"),
        (Role::Slider, "Volume"),
        (Role::SpinButton, "Count"),
        (Role::TextInput, "Name"),
        (Role::ScrollArea, "list"),
        (Role::ProgressIndicator, "Loading"),
        (Role::Meter, "Disk"),
        (Role::Label, "A plain label"),
    ] {
        let said = node(&harness, role, name);
        let platform = platform_node(&full, said.id)
            .unwrap_or_else(|| panic!("the platform tree carries {role} {name:?}"));
        let expected: Vec<accesskit::Action> = every_ask()
            .into_iter()
            .zip(PLATFORM_ASKS)
            .filter(|((word, _), _)| said.actions.contains(*word))
            .map(|(_, action)| action)
            .collect();
        assert_eq!(
            advertised(platform),
            expected,
            "{role} {name:?}: the platform node advertises exactly the tree's word"
        );
    }
    let chart = node(&harness, Role::Canvas, "Chart");
    let platform = platform_node(&full, chart.id).expect("the canvas rides the tree");
    assert!(
        advertised(platform).is_empty(),
        "a control with no door advertises nothing on the platform either"
    );
}

#[test]
fn m156_the_corpus_grew_by_the_coordinated_rule() {
    // A file-reading test: the published corpus is read as the CI reads
    // it, so it takes no flux lock and can poison none for its neighbours.
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

    // The published corpus carries the action tier's case: the 1,000-node
    // snapshot with every node advertising the whole vocabulary — the
    // word's ceiling price, measured against the same tree bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(ACTIONS_CASE) && benches.contains_key(BARE_REBUILD_CASE),
        "the action tier's case is published beside its bare denominator ({ACTIONS_CASE}, {BARE_REBUILD_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M156 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [ACTIONS_CASE, BARE_REBUILD_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("a11y_actions_overhead"))
        .expect("the action budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(ACTIONS_CASE),
        "the budget prices the advertising sink"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BARE_REBUILD_CASE),
        "the budget prices it against the same tree, bare"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the word must not outprice the node it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M156 cut"
    );
}
