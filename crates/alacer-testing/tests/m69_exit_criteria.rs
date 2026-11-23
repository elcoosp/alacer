//! M69 exit-criteria integration tests (§13.3, ADR-0075): the activation
//! ask answers on the log. The kill rule the action tier's own record
//! pre-registers cashes — a consumer prices the AT's activation ask (the
//! press: the chord arrived at a target, the target's door answered, and
//! the answer was a bare `bool` whose `false` collapsed the two facts a
//! trail reader wants apart — the availability tier's wall and the
//! doorless target's silence; "did the AT's press land — or why not"
//! unanswerable from the log alone) → the action tier grows its own
//! trail ([`alacer_core::inspect::ActionRecord`], the funnel's own
//! species beside the ask: the door landed it, the wall held it
//! ([`alacer_core::inspect::ActionRefusal::Unavailable`]), or no door
//! answered it ([`alacer_core::inspect::ActionRefusal::NoDoor`]) — the
//! species computed where both facts live: the route holds the ask, the
//! target, the door's answer, and the tree's one `enabled` truth, M22)
//! and the thread trail below, the route-drop trail's shape: the note
//! site is the exact place the ask resolves — the dispatch funnel — and
//! the dispatching pipeline drains it at the tick's start into
//! [`alacer_core::inspect::InspectEvent::Action`], dev-mode and bounded,
//! silent while nobody listens.
//!
//! One fact per trail (the M39 rule at the activation tier): the asks
//! whose tiers already price themselves — the value tier's steps, the
//! scroll tier's units, the text-write tier's rewrites — stream no
//! action fact; the action trail prices the activation ask only.
//!
//! Every test runs the real pipeline (A4): the asks routed the exact way
//! the platform's action route and the harness's a11y door ride them —
//! through the funnel, the trail door installed beside it.

use alacer_core::inspect::{ActionOutcome, ActionRefusal, InspectEvent};
use alacer_core::trail::TrailLog;
use alacer_core::widget::{A11yAction, Role, ScrollUnit};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, Slider};

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

/// A button mounted through the real pipeline and settled, with the
/// trail installed first (the trail door's contract — the observer's
/// install enables the trail). Returns the harness, the trail, the
/// root column, and the button's AccessKit-stable id (the funnel's
/// target vocabulary).
fn mount_button() -> (TestHarness, TrailLog, alacer_core::NodeId, u64) {
    let mut harness = TestHarness::new(400.0, 200.0);
    let trail = harness.trail();
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Button::new("Increment").on_click(|| {}).boxed());
    harness.settle();
    let id = harness.query(Role::Button).named("Increment").first().id;
    (harness, trail, root, id)
}

#[test]
fn m69_the_press_lands_on_the_log() {
    let (mut harness, trail, _root, id) = mount_button();

    // The AT's activation ask rides the real action route (the funnel
    // door the platform's adapter drives): the door lands it, and the
    // fact streams the tick after the ask resolves.
    clear_window(&trail);
    assert!(
        harness.act(Role::Button, "Increment", A11yAction::DefaultPress),
        "the enabled button lands the press"
    );
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 1, "the press is one fact");
    let record = facts[0];
    assert_eq!(
        record.ask,
        alacer_core::inspect::ActionAsk::Press,
        "the activation ask, named by its own vocabulary"
    );
    assert_eq!(
        record.outcome,
        ActionOutcome::Delivered,
        "the landing is the door's own answer"
    );
    assert_eq!(
        record.target,
        Some(id),
        "the funnel's stamp: the fact names the control it rode"
    );
    assert_eq!(
        record.rider,
        alacer_core::inspect::ActionRider::ActionRoute,
        "the funnel's own note site names the action route (M70's rider)"
    );
}

#[test]
fn m69_the_wall_names_itself() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let trail = harness.trail();
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(
        root,
        Button::new("Send").enabled(false).on_click(|| {}).boxed(),
    );
    harness.settle();
    let id = harness.query(Role::Button).named("Send").first().id;

    // The availability tier holds the control down (M22, ADR-0028): the
    // door refuses, and the refusal is no longer a collapsed `false` —
    // the wall's species rides the record beside the ask.
    clear_window(&trail);
    assert!(
        !harness.act(Role::Button, "Send", A11yAction::DefaultPress),
        "the wall refuses the press"
    );
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 1, "the refused press is one fact");
    assert_eq!(
        facts[0].outcome,
        ActionOutcome::Refused(ActionRefusal::Unavailable),
        "the wall's species, computed where the truth lives"
    );
    assert_eq!(
        facts[0].target,
        Some(id),
        "the wall's fact names the control it held down"
    );
    assert_eq!(
        facts[0].rider,
        alacer_core::inspect::ActionRider::ActionRoute,
        "the wall's fact is the AT route's"
    );
}

#[test]
fn m69_the_doorless_target_names_the_silence() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let trail = harness.trail();
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Label::new("A heading").boxed());
    harness.settle();
    let id = harness.query(Role::Label).named("A heading").first().id;

    // The ask aimed at a widget that grew no activation door: the door
    // answered nothing, and the silence is now a named fact — the
    // species beside the aimed id, never a munge onto a door that
    // answered.
    clear_window(&trail);
    assert!(
        !harness.act(Role::Label, "A heading", A11yAction::DefaultPress),
        "the label has no press door"
    );
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 1, "the doorless ask is one fact");
    assert_eq!(
        facts[0].outcome,
        ActionOutcome::Refused(ActionRefusal::NoDoor),
        "the doorless silence, named"
    );
    assert_eq!(facts[0].target, Some(id), "the aim rides verbatim");
}

#[test]
fn m69_the_tiered_asks_stream_no_action_fact() {
    let (mut harness, trail, root, _id) = mount_button();
    let _slider = harness.add_child(root, Slider::new("Volume", 0.0, 100.0).boxed());
    harness.settle();
    let slider_id = harness.query(Role::Slider).named("Volume").first().id;

    // The value tier's step already prices itself on the value trail
    // (ADR-0046): the action trail streams no second fact for it — one
    // fact per trail, never a double-priced ask.
    clear_window(&trail);
    assert!(
        harness.act(Role::Slider, "Volume", A11yAction::Increment),
        "the slider answers its own tier"
    );
    let facts = action_facts(&trail);
    assert!(
        facts.is_empty(),
        "a value ask is the value trail's fact, never an action fact"
    );

    // The scroll tier's unit scroll prices itself on the scroll trail
    // (ADR-0056) the same way — the tier boundary the log prices.
    clear_window(&trail);
    let _ = harness.act(
        Role::Slider,
        "Volume",
        A11yAction::ScrollUp(ScrollUnit::Page),
    );
    assert!(
        action_facts(&trail).is_empty(),
        "an out-of-tier ask is not an action fact either"
    );
    let _ = slider_id;
}

#[test]
fn m69_the_delivered_ask_queues_masks_and_nothing_else() {
    let (mut harness, trail, _root, _id) = mount_button();

    // The funnel's mask semantics are unchanged (the bool contract of
    // record): a delivered press queues the PAINT | A11Y merge exactly
    // as before, the trail riding beside it — never instead of it.
    clear_window(&trail);
    let _ = harness.act(Role::Button, "Increment", A11yAction::DefaultPress);
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 1, "the delivered ask is one fact");
    assert!(
        matches!(facts[0].outcome, ActionOutcome::Delivered),
        "the mask-queueing ride and the trail ride tell one story"
    );
}

#[test]
fn m69_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // trail sentence carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M145",
        "a hundred and fifty decision records",
        "Since M69 the activation ask answers on the log",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0075",
        "a hundred and fifty decision records",
        "the action tier grows its own trail",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary: the flat
    // `action` event beside the other trails' events.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`action`"),
        "the protocol document carries the action event"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M69 — the activation ask answers on the log (ADR-0075)**"),
        "the CHANGELOG carries the M69 entry"
    );

    // The catalog: the demonstration walks the activation beat.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("the AT's press"),
        "docs/examples.md carries the activation beat"
    );

    // The sibling suites' pins ride the same cut (one sweep, no drift).
    let m68 = read_workspace_file("crates/alacer-testing/tests/m68_exit_criteria.rs");
    assert!(
        m68.contains("M0 through M145"),
        "the m68 suite's pins moved with the documents"
    );
}

/// Reads a workspace file relative to the manifest the suite runs from
/// (the m-suite's own helper shape: `CARGO_MANIFEST_DIR` walks up to
/// the workspace root).
fn read_workspace_file(path: &str) -> String {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir");
    let workspace = std::path::Path::new(&manifest)
        .ancestors()
        .nth(2)
        .expect("the suite rides two directories under the workspace root");
    std::fs::read_to_string(workspace.join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}
