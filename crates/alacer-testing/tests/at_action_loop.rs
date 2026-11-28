//! The AT's action loop, headless and end to end (ADR-0161's verification
//! note cashed): the word the tree says read the way a screen reader
//! reads it — off the `accesskit::TreeUpdate` the sink emits, node ids and
//! `supports_action` alone — and every ask the word invites sent the way a
//! reader sends it: the AccessKit `ActionRequest`, payload and scroll unit,
//! crossing the platform's boundary mapping onto the funnel's door,
//! dispatched at the node id the wire carried, the app's state moved and
//! the tree re-saying the moved fact.
//!
//! ADR-0161 recorded the action tier's one open verification: *"UNVERIFIED
//! at strict platform adapters (no AT was run for this cut): the
//! consumer's derivation is pinned by reading its source and by the sink's
//! `supports_action` tests, not by a live screen reader session."* A live
//! reader is the one instrument a headless CI cannot run; its *loop* is
//! not. The loop is exactly three doors in a row — the sink's
//! `advertised_actions` table (M156), the platform's `map_action_request`
//! (the exact inverse, M10), the funnel's `dispatch_a11y_action` (M10's
//! route) — and every one is drivable headlessly (A4). These tests run the
//! whole chain over the real pipeline, the canon surface the m156 suite
//! mounted, so the tier's invariant is proven across the boundary the way
//! a live session would exercise it: the word the AT reads on the wire
//! predicts, ask for ask, exactly which asks the door consumes — and no
//! ask the word withheld lands, no refusal queues a mask, and the
//! availability door moves the wire word and the boundary answer together.
//!
//! The m156 suite pinned the word and the door *inside* the core
//! vocabulary (`harness.act`, find-by-role); the platform's inline tests
//! pinned the mapping table in isolation. Nothing drove the three doors as
//! one flow — the seam this suite owns.

use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::color::Color;
use alacer_core::inspect::{ActionOutcome, ActionRefusal, ActionRider, InspectEvent};
use alacer_core::passes::TreeNode;
use alacer_core::trail::TrailLog;
use alacer_core::widget::{A11yAction, A11yActions, Role, TextSelection, ToggleState};
use alacer_core::{NodeId, RouteDropRecord, RouteDropRider, RouteDropSpecies};
use alacer_testing::TestHarness;
use alacer_widgets::{
    BoxedWidget as _, Button, Canvas, Checkbox, Column, Label, Link, List, Menu, MenuBar, MenuItem,
    Meter, Panel, Progress, Radio, Slider, SpinButton, Switch, Tab, TabList, TextInput,
};

/// The ten AccessKit actions the platform maps onto the vocabulary (the
/// sink's inverse table): the set an AT can ask for, in the order the
/// vocabulary spells them.
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

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> TreeNode {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name))
        .cloned()
        .unwrap_or_else(|| panic!("no {role} named {name:?} is reachable"))
}

/// The canon's arena handle the field beats reach through the pass-3
/// focus contract: selection implies focus (the m11 suite's rule), and
/// the focus door takes the arena id the mount returned, never the a11y
/// hash the wire carries.
struct Canon {
    field: NodeId,
}

/// The canon under one root: every control whose door answers an ask,
/// beside the reports whose doors answer none. Each keeps the name its
/// constructor gave it (the m156 suite's surface, mounted again here —
/// the loop's proof rides the same canon the tier's cut pinned). Returns
/// the two handles the suite's beats ride.
fn mount_canon(harness: &mut TestHarness) -> Canon {
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    let _ = harness.add_child(root, Checkbox::new("Bold").boxed());
    let _ = harness.add_child(root, Radio::new("Small").boxed());
    let _ = harness.add_child(root, Switch::new("Wi-Fi").boxed());
    let _ = harness.add_child(root, Link::new("Site").boxed());
    let _ = harness.add_child(root, Slider::new("Volume", 0.0, 100.0).boxed());
    let _ = harness.add_child(root, SpinButton::new("Count", 0.0, 10.0).step(1.0).boxed());
    let field = harness.add_child(root, TextInput::new("Name").boxed());
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
    Canon { field }
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
fn advertised(update: &accesskit::TreeUpdate, id: u64) -> Vec<accesskit::Action> {
    find_in_update(update, id).map_or_else(
        || panic!("the wire carries node {id} — the AT reads what was emitted"),
        |platform| {
            PLATFORM_ASKS
                .iter()
                .copied()
                .filter(|action| platform.supports_action(*action))
                .collect()
        },
    )
}

/// One AT ask, the way a reader sends it: the AccessKit action named, the
/// payload the ask carries, aimed at the node id the wire said. A
/// well-formed payload rides every ask that carries one, so a `None` from
/// the boundary is the *mapping's* refusal and never a malformed
/// payload's.
fn an_ask(
    action: accesskit::Action,
    data: Option<accesskit::ActionData>,
    target: u64,
) -> accesskit::ActionRequest {
    accesskit::ActionRequest {
        action,
        target_tree: accesskit::TreeId::ROOT,
        target_node: accesskit::NodeId(target),
        data,
    }
}

/// The well-formed payload per AccessKit action, beside the vocabulary
/// kind the mapped action carries — the pair a reader builds when it
/// means exactly that ask, aimed at `target`.
fn well_formed_ask(
    action: accesskit::Action,
    target: u64,
) -> (A11yActions, Option<accesskit::ActionData>) {
    use accesskit::Action as Ak;
    match action {
        Ak::Click => (A11yActions::DEFAULT_PRESS, None),
        Ak::Increment => (A11yActions::INCREMENT, None),
        Ak::Decrement => (A11yActions::DECREMENT, None),
        Ak::ScrollUp => (
            A11yActions::SCROLL_UP,
            Some(accesskit::ActionData::ScrollUnit(
                accesskit::ScrollUnit::Page,
            )),
        ),
        Ak::ScrollDown => (
            A11yActions::SCROLL_DOWN,
            Some(accesskit::ActionData::ScrollUnit(
                accesskit::ScrollUnit::Page,
            )),
        ),
        Ak::ScrollLeft => (
            A11yActions::SCROLL_LEFT,
            Some(accesskit::ActionData::ScrollUnit(
                accesskit::ScrollUnit::Item,
            )),
        ),
        Ak::ScrollRight => (
            A11yActions::SCROLL_RIGHT,
            Some(accesskit::ActionData::ScrollUnit(
                accesskit::ScrollUnit::Item,
            )),
        ),
        Ak::SetValue => (
            A11yActions::SET_VALUE,
            Some(accesskit::ActionData::Value("42".into())),
        ),
        Ak::ReplaceSelectedText => (
            A11yActions::REPLACE_SELECTED_TEXT,
            Some(accesskit::ActionData::Value("x".into())),
        ),
        Ak::SetTextSelection => (
            A11yActions::SET_TEXT_SELECTION,
            Some(accesskit::ActionData::SetTextSelection(
                accesskit::TextSelection {
                    anchor: accesskit::TextPosition {
                        node: accesskit::NodeId(target),
                        character_index: 0,
                    },
                    focus: accesskit::TextPosition {
                        node: accesskit::NodeId(target),
                        character_index: 3,
                    },
                },
            )),
        ),
        other => panic!("the probe sends the ten mapped asks, never {other:?}"),
    }
}

/// The app's availability door: the mutate seam the app's code rides —
/// the flag moved, `PAINT | A11Y` requested on the control when the write
/// moved it, the frames the re-announcement rides settled before the read
/// (the m156 suite's own helper).
fn set_enabled(harness: &mut TestHarness, node_id: NodeId, enabled: bool) -> bool {
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

/// The streamed action facts, oldest first (the m-suite's filter shape:
/// the assertion source is the trail's own records, never internals).
fn action_facts(trail: &TrailLog) -> Vec<alacer_core::inspect::ActionRecord> {
    trail
        .look()
        .into_iter()
        .filter_map(|event| match event {
            InspectEvent::Action(record) => Some(record),
            _ => None,
        })
        .collect()
}

/// Empties the window between beats: each assertion reads the facts the
/// ask under test streamed, nothing earlier.
fn clear_window(trail: &TrailLog) {
    let _ = trail.drain();
}

/// The route-drop facts the tick's start drain streams, collected the way
/// the exit suites collect them (the bounded-ring shape).
#[derive(Default)]
struct DropRing {
    drops: std::collections::VecDeque<RouteDropRecord>,
}

impl DropRing {
    fn drops(&self) -> Vec<RouteDropRecord> {
        self.drops.iter().copied().collect()
    }
}

impl alacer_core::inspect::InspectObserver for DropRing {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::RouteDrop(record) = event {
            self.drops.push_back(record);
        }
    }
}

/// The shared-ring wrapper the pipeline owns (`Rc<RefCell>`, the
/// m-suite's shape): the observer slot takes one boxed observer, the test
/// keeps the reading handle.
#[derive(Clone)]
struct Ring(std::rc::Rc<std::cell::RefCell<DropRing>>);

impl alacer_core::inspect::InspectObserver for Ring {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

#[test]
fn the_at_reads_the_wire_and_every_ask_the_word_invites_lands() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 640.0);
    let trail = harness.trail();
    let canon = mount_canon(&mut harness);

    // The AT's first read: the full tree the adapter answers activation
    // with — the same node builder every delta rides, the node the AT
    // walks, the word the ADR-0161 sweep pinned said per control.
    let full = with_sink(&mut harness, |sink| sink.full_tree_update())
        .expect("the sink resolved the tree");
    for (role, name) in [
        (Role::Button, "Save"),
        (Role::CheckBox, "Bold"),
        (Role::Slider, "Volume"),
        (Role::SpinButton, "Count"),
        (Role::TextInput, "Name"),
        (Role::ScrollArea, "list"),
    ] {
        let core = node(&harness, role, name);
        let word = advertised(&full, core.id);
        assert!(
            !word.is_empty(),
            "{role} {name:?}: the wire advertises the asks the door answers"
        );
        for action in word {
            let (kind, data) = well_formed_ask(action, core.id);
            let request = an_ask(action, data, core.id);
            let mapped = alacer_platform::map_action_request(&request).unwrap_or_else(|| {
                panic!("the boundary maps {action:?} — the platform's own inverse table")
            });
            assert_eq!(
                mapped.advertises(),
                kind,
                "{role} {name:?}: the mapping lands on the kind the ask invited"
            );
            // The ask rides the funnel the way the platform's window loop
            // drives it — the node id the wire carried, the door's own
            // answer.
            assert!(
                harness.act_at(core.id, mapped),
                "{role} {name:?}: the advertised {action:?} lands on the door"
            );
        }
    }

    // The state the asks moved, read off the tree the AT re-reads —
    // each fact the ask's own door moved: the click toggled, the
    // set-value wrote the slider's value.
    assert_eq!(
        node(&harness, Role::CheckBox, "Bold").toggled,
        Some(ToggleState::On),
        "the click the word invited toggled the checkbox"
    );
    assert_eq!(
        node(&harness, Role::Slider, "Volume").numeric_value,
        Some(42.0),
        "the set-value the word invited wrote the slider's value"
    );

    // The text tier's contract, asked one ask at a time the way a reader
    // drives a field: the whole-value write, the selection moved onto the
    // written text, the paste-shaped write over the selection — each
    // landed through the boundary at the node id the wire carried, the
    // tree re-saying the fact each door moved (M11, ADR-0017). Selection
    // implies focus, so the field is focused through the pass-3 contract
    // first — the route a reader's caret ask always rides.
    harness.pipeline_mut().request_focus(canon.field);
    harness.settle();
    let field_id = node(&harness, Role::TextInput, "Name").id;
    assert!(
        harness.act_at(field_id, A11yAction::SetValue("424".to_owned())),
        "the whole-value write lands"
    );
    assert_eq!(
        node(&harness, Role::TextInput, "Name").value.as_deref(),
        Some("424"),
        "the write the word invited wrote the field's value"
    );
    assert!(
        harness.act_at(
            field_id,
            A11yAction::SetTextSelection(TextSelection::new(0, 3))
        ),
        "the selection ask lands"
    );
    assert_eq!(
        harness.selection(Role::TextInput, "Name"),
        Some((0, 3)),
        "the selection ask the word invited moved the field's selection — three UTF-16 units, three said"
    );
    assert!(
        harness.act_at(field_id, A11yAction::ReplaceSelectedText("x".to_owned())),
        "the paste-shaped write lands"
    );
    assert_eq!(
        node(&harness, Role::TextInput, "Name").value.as_deref(),
        Some("x"),
        "the replace ask wrote over the selection — the selection-consuming edit"
    );

    // The activation ask answers on the trail as data (M69): the press
    // the word invited names its landing, its target, its rider.
    clear_window(&trail);
    let save_id = node(&harness, Role::Button, "Save").id;
    assert!(
        harness.act_at(save_id, A11yAction::DefaultPress),
        "the press the word invited lands"
    );
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 1, "the press is one fact");
    assert_eq!(facts[0].outcome, ActionOutcome::Delivered);
    assert_eq!(facts[0].target, Some(save_id));
    assert_eq!(facts[0].rider, ActionRider::ActionRoute);
}

#[test]
fn the_wire_word_is_the_boundary_answer_ask_for_ask() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 640.0);
    let _canon = mount_canon(&mut harness);
    let full = with_sink(&mut harness, |sink| sink.full_tree_update())
        .expect("the sink resolved the tree");

    // The tier's invariant, asked across the boundary: for every canon
    // control and every AccessKit ask the platform maps, the word the AT
    // reads on the wire (`supports_action`) predicts exactly what the
    // door consumes — the word is the door's answer, said before the ask,
    // now proven over the real `ActionRequest`s a reader sends.
    for (role, name) in [
        (Role::Button, "Save"),
        (Role::CheckBox, "Bold"),
        (Role::Radio, "Small"),
        (Role::Switch, "Wi-Fi"),
        (Role::Link, "Site"),
        (Role::Slider, "Volume"),
        (Role::SpinButton, "Count"),
        (Role::TextInput, "Name"),
        (Role::ScrollArea, "list"),
        (Role::Tab, "Inbox"),
        (Role::MenuItem, "Open"),
        (Role::ProgressIndicator, "Loading"),
        (Role::Meter, "Disk"),
        (Role::Canvas, "Chart"),
        (Role::Label, "A plain label"),
    ] {
        let core = node(&harness, role, name);
        let word = advertised(&full, core.id);
        for action in PLATFORM_ASKS {
            let (kind, data) = well_formed_ask(action, core.id);
            let mapped = alacer_platform::map_action_request(&an_ask(action, data, core.id))
                .unwrap_or_else(|| {
                    panic!("the boundary maps {action:?} — the mapped set is total")
                });
            let answered = harness.act_at(core.id, mapped);
            assert_eq!(
                word.contains(&action),
                answered,
                "{role} {name:?}: the wire {} {action:?}, the door {} it",
                if word.contains(&action) {
                    "advertises"
                } else {
                    "withholds"
                },
                if answered { "answered" } else { "refused" },
            );
            assert_eq!(
                core.actions.contains(kind),
                word.contains(&action),
                "{role} {name:?}: the snapshot's word and the wire's word agree on {action:?}"
            );
        }
    }
    // The loop's tail is the honest absences (every ask refused): a
    // refused ask queues nothing — the frame the next settle rides is
    // the first idle one (the M21 property at the boundary).
    assert_eq!(harness.settle(), 0, "the refused asks queued no masks");
}

#[test]
fn the_availability_door_moves_the_wire_word_and_the_boundary_answer() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let trail = harness.trail();
    let root = harness.mount(Column::new().boxed());
    let save = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();
    let full = with_sink(&mut harness, |sink| sink.full_tree_update())
        .expect("the sink resolved the tree");
    assert!(
        advertised(&full, node(&harness, Role::Button, "Save").id)
            .contains(&accesskit::Action::Click),
        "the available control advertises the press on the wire"
    );
    assert!(
        harness.act_at(
            node(&harness, Role::Button, "Save").id,
            A11yAction::DefaultPress
        ),
        "the open door lands the press"
    );

    // The availability door moves the wire word: the app's mutate seam
    // writes the flag, the `PAINT | A11Y` request re-announces, the
    // platform's node re-emitted with the word withdrawn.
    assert!(
        set_enabled(&mut harness, save, false),
        "the write moved the flag"
    );
    let drained = with_sink(&mut harness, AccessKitSink::drain_updates);
    let save_id = node(&harness, Role::Button, "Save").id;
    let re_emitted = drained
        .iter()
        .rev()
        .find_map(|update| find_in_update(update, save_id))
        .expect("the narrowed control is re-emitted");
    assert!(
        !re_emitted.supports_action(accesskit::Action::Click),
        "the closed door withdraws the word the wire says"
    );

    // The ask still travels the route it always travelled: the boundary
    // maps it (the mapping is availability-blind — the door is where the
    // availability wall stands) and the door refuses it, the trail
    // naming the wall (M22 reaching the boundary answer).
    clear_window(&trail);
    assert!(
        !harness.act_at(save_id, A11yAction::DefaultPress),
        "the closed door refuses the press the withdrawn word no longer invites"
    );
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 1, "the refusal is one fact");
    assert_eq!(
        facts[0].outcome,
        ActionOutcome::Refused(ActionRefusal::Unavailable),
        "the trail names the availability wall"
    );
    assert_eq!(facts[0].target, Some(save_id));

    // The reopened door restores the word and the landing together.
    assert!(
        set_enabled(&mut harness, save, true),
        "the write moved the flag back"
    );
    let restored = with_sink(&mut harness, AccessKitSink::drain_updates);
    let re_emitted = restored
        .iter()
        .rev()
        .find_map(|update| find_in_update(update, save_id))
        .expect("the reopened control is re-emitted");
    assert!(
        re_emitted.supports_action(accesskit::Action::Click),
        "the reopened door restores the word the wire says"
    );
    clear_window(&trail);
    assert!(
        harness.act_at(save_id, A11yAction::DefaultPress),
        "the reopened door lands the press again"
    );
    assert_eq!(
        action_facts(&trail)[0].outcome,
        ActionOutcome::Delivered,
        "the landing rides the restored word"
    );
}

#[test]
fn the_boundary_refuses_what_it_cannot_say_before_any_door_answers() {
    let _guard = alacer_core::flux::batch_test_lock();
    let ring = Ring(std::rc::Rc::new(std::cell::RefCell::new(
        DropRing::default(),
    )));
    let mut harness = TestHarness::new(420.0, 320.0);
    harness.pipeline_mut().set_observer(Box::new(ring.clone()));
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    let _ = harness.add_child(root, TextInput::new("Name").boxed());
    harness.settle();
    let field_id = node(&harness, Role::TextInput, "Name").id;
    let button_id = node(&harness, Role::Button, "Save").id;

    // The boundary's refusals (M57, ADR-0063) ride the mapping arms: the
    // ask never became a core action, so no door ever answered — the
    // drop facts stream the tick after, the rider naming the boundary.
    // A `SetValue` with no payload: the write a reader cannot mean.
    assert!(
        alacer_platform::map_action_request(&an_ask(accesskit::Action::SetValue, None, field_id))
            .is_none(),
        "the payloadless write is refused at the boundary"
    );
    // A selection aimed at a node the ask did not target: the
    // single-node selection contract (ADR-0017) cannot honor it.
    let foreign = accesskit::TextSelection {
        anchor: accesskit::TextPosition {
            node: accesskit::NodeId(0xDEAD),
            character_index: 0,
        },
        focus: accesskit::TextPosition {
            node: accesskit::NodeId(0xDEAD),
            character_index: 3,
        },
    };
    assert!(
        alacer_platform::map_action_request(&an_ask(
            accesskit::Action::SetTextSelection,
            Some(accesskit::ActionData::SetTextSelection(foreign)),
            field_id,
        ))
        .is_none(),
        "the off-target selection is refused at the boundary"
    );
    // An action the vocabulary has no contract for: the `Focus` request
    // that clicked something would be a correctness bug, not a fallback.
    assert!(
        alacer_platform::map_action_request(&an_ask(accesskit::Action::Focus, None, button_id))
            .is_none(),
        "the unmapped action is refused at the boundary"
    );
    assert!(
        harness.act_at(button_id, A11yAction::DefaultPress),
        "the canon door still answers — the refusals touched nothing"
    );

    // The three drops streamed, one each, the species naming why the
    // boundary refused (the M57 discipline, observed through the
    // harness's own tick).
    let drops = ring.0.borrow().drops();
    assert_eq!(drops.len(), 3, "one fact per refusal");
    assert_eq!(drops[0].species, RouteDropSpecies::NoPayload);
    assert_eq!(drops[0].rider, RouteDropRider::Boundary);
    assert_eq!(drops[0].target, Some(field_id));
    assert_eq!(drops[1].species, RouteDropSpecies::OffTargetSelection);
    assert_eq!(drops[2].species, RouteDropSpecies::UnknownAction);

    // The funnel's own refusal (the rider that named the route since
    // M65): the ask aimed at an id no tree node carries — the wire id a
    // stale AT session still holds. The mapping maps (it is
    // availability- and tree-blind), the funnel drops, no door answers.
    assert!(
        !harness.act_at(0xBAD_C0DE, A11yAction::DefaultPress),
        "the stale id lands on no door"
    );
    let drops = ring.0.borrow().drops();
    let funnel_drop = drops
        .iter()
        .find(|drop| drop.rider == RouteDropRider::Funnel)
        .expect("the funnel's drop streams its own fact");
    assert_eq!(funnel_drop.species, RouteDropSpecies::UnknownTarget);
    assert_eq!(funnel_drop.target, Some(0xBAD_C0DE));
    // The word the surviving controls advertise is untouched: the drop
    // queue drained without re-announcing anything (a drop is not a
    // fact about the tree).
    let full = with_sink(&mut harness, |sink| sink.full_tree_update())
        .expect("the sink resolved the tree");
    assert_eq!(
        advertised(&full, button_id).len(),
        1,
        "the button's word stands — a drop never re-announces"
    );
    assert_eq!(
        advertised(&full, field_id).len(),
        3,
        "the field's word stands — the text tier's three asks"
    );
}
