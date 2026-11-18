//! M70 exit-criteria integration tests (§13.3, ADR-0076): the
//! activation names its rider. M69's own pre-registered kill rule
//! cashes — a consumer cannot tell the AT's press from the keyboard's
//! Space/Enter activation on the log the action trail streams (the
//! record answered "did the press land — or why not", never "who
//! pressed") → the record grows the rider beside the ask
//! ([`alacer_core::inspect::ActionRider`], the M30 attribution rule at
//! the activation tier): the AT's ask riding the action route
//! ([`alacer_core::inspect::ActionRider::ActionRoute`], the dispatch
//! funnel's own note site) and the keyboard's own activation riding
//! [`alacer_core::inspect::ActionRider::Keyboard`] — the button's
//! GTK release rule and the checkbox's toggle-on-press, the M10 parity
//! the widgets answer like a click, named where the route is known.
//!
//! The keyboard's ride carries the key route's stamp (M45, ADR-0051):
//! the focused control's AccessKit-stable id, the guard the key visit
//! holds, never guessed at the note site. The wire grows the additive
//! `rider` field at version 1 (ADR-0010); one fact per activation —
//! the button's pressed phase arms nothing on the log, the release is
//! where the door answered (the flood rule at the activation tier).
//!
//! Every test runs the real pipeline (A4): the AT's ask through the
//! real action route, the keyboard's activation through the real key
//! route — through the harness's trail door.

use alacer_core::events::{Key, KeyEvent, KeyState};
use alacer_core::inspect::{ActionOutcome, ActionRider, InspectEvent};
use alacer_core::trail::TrailLog;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column};

/// The streamed action facts, oldest first (the m-suite's filter
/// shape: the assertion source is the trail's own records, never
/// internals).
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

/// Empties the window between beats: each assertion reads the facts
/// the ask under test streamed, nothing earlier.
fn clear_window(trail: &TrailLog) {
    let _ = trail.drain();
}

/// A button and a checkbox mounted through the real pipeline and
/// settled, with the trail installed first (the trail door's
/// contract). Returns the harness, the trail, the two controls' arena
/// nodes (the focus doors' vocabulary) and their AccessKit-stable ids
/// (the routes' stamp vocabulary).
struct Controls {
    harness: TestHarness,
    trail: TrailLog,
    button_node: alacer_core::NodeId,
    checkbox_node: alacer_core::NodeId,
    button_id: u64,
    checkbox_id: u64,
}

fn mount_controls() -> Controls {
    let mut harness = TestHarness::new(400.0, 240.0);
    let trail = harness.trail();
    let root = harness.mount(Column::new().boxed());
    let button_node = harness.add_child(root, Button::new("Increment").on_click(|| {}).boxed());
    let checkbox_node = harness.add_child(root, Checkbox::new("Ship it").boxed());
    harness.settle();
    let button_id = harness.query(Role::Button).named("Increment").first().id;
    let checkbox_id = harness.query(Role::CheckBox).named("Ship it").first().id;
    Controls {
        harness,
        trail,
        button_node,
        checkbox_node,
        button_id,
        checkbox_id,
    }
}

/// A Space press (the activation chord's own shape) — the exact event
/// the platform delivers when the user activates the focused control.
fn space(state: KeyState) -> KeyEvent {
    KeyEvent {
        key: Key::Space,
        state,
        modifiers: alacer_core::events::Modifiers::default(),
        text: None,
    }
}

#[test]
fn m70_the_at_activation_names_its_route() {
    let Controls {
        mut harness,
        trail,
        button_id,
        ..
    } = mount_controls();

    // The AT's activation ask rides the real action route (the funnel
    // door the platform's adapter drives): the record names the route
    // that routed it.
    clear_window(&trail);
    assert!(
        harness.act(
            Role::Button,
            "Increment",
            alacer_core::widget::A11yAction::DefaultPress
        ),
        "the enabled button lands the AT's press"
    );
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 1, "the AT's press is one fact");
    assert_eq!(
        facts[0].rider,
        ActionRider::ActionRoute,
        "the funnel's own note site names the action route"
    );
    assert_eq!(
        facts[0].target,
        Some(button_id),
        "the AT route's stamp: the id the funnel resolved"
    );
}

#[test]
fn m70_the_keyboard_activation_names_its_rider() {
    let Controls {
        mut harness,
        trail,
        button_node,
        button_id,
        ..
    } = mount_controls();

    // The user presses Space on the focused button: the GTK rule
    // answers the activation at the release — one fact per activation,
    // the pressed phase arming nothing on the log.
    clear_window(&trail);
    harness.pipeline_mut().request_focus(button_node);
    harness.settle();
    harness.key(&space(KeyState::Pressed));
    assert!(
        action_facts(&trail).is_empty(),
        "the pressed phase arms nothing on the log (the flood rule)"
    );
    harness.key(&space(KeyState::Released));
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 1, "the activation is one fact");
    assert_eq!(
        facts[0].rider,
        ActionRider::Keyboard,
        "the keyboard's rider, beside the ask"
    );
    assert_eq!(
        facts[0].outcome,
        ActionOutcome::Delivered,
        "the keyboard's activation landed — the M10 parity"
    );
    assert_eq!(
        facts[0].target,
        Some(button_id),
        "the key route's stamp rides the keyboard's activation too (M45)"
    );
}

#[test]
fn m70_the_checkbox_toggle_names_the_keyboard() {
    let Controls {
        mut harness,
        trail,
        checkbox_node,
        checkbox_id,
        ..
    } = mount_controls();

    // The checkbox's toggle-on-press (HTML semantics): one Space, one
    // fact — the rider beside the toggle the door answered.
    clear_window(&trail);
    harness.pipeline_mut().request_focus(checkbox_node);
    harness.settle();
    harness.key(&space(KeyState::Pressed));
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 1, "the toggle is one fact");
    assert_eq!(facts[0].rider, ActionRider::Keyboard);
    assert_eq!(facts[0].target, Some(checkbox_id));
}

#[test]
fn m70_the_two_riders_tellable_apart_on_one_log() {
    let Controls {
        mut harness,
        trail,
        button_node,
        button_id,
        ..
    } = mount_controls();

    // Both riders, one log: the AT's ask, then the user's Space — the
    // records telling apart "did the screen reader press it, or did
    // the user hit Space" from the rider alone.
    clear_window(&trail);
    harness.act(
        Role::Button,
        "Increment",
        alacer_core::widget::A11yAction::DefaultPress,
    );
    harness.pipeline_mut().request_focus(button_node);
    harness.settle();
    harness.key(&space(KeyState::Pressed));
    harness.key(&space(KeyState::Released));
    let facts = action_facts(&trail);
    assert_eq!(facts.len(), 2, "two rides, two facts");
    assert_eq!(facts[0].rider, ActionRider::ActionRoute, "the AT's ask");
    assert_eq!(facts[0].target, Some(button_id));
    assert_eq!(facts[1].rider, ActionRider::Keyboard, "the user's Space");
    assert_eq!(facts[1].target, Some(button_id), "the same control");
}

#[test]
fn m70_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // trail sentence carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M113",
        "a hundred and eighteen decision records",
        "Since M70 the activation names its rider",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0076",
        "a hundred and eighteen decision records",
        "the activation names its rider",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary: the additive
    // rider field beside the ask.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`action_route`"),
        "the protocol document carries the rider's tag"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M70 — the activation names its rider (ADR-0076)**"),
        "the CHANGELOG carries the M70 entry"
    );

    // The catalog: the demonstration walks the rider beat.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("the keyboard's Space"),
        "docs/examples.md carries the rider beat"
    );

    // The sibling suites' pins ride the same cut (one sweep, no drift).
    let m69 = read_workspace_file("crates/alacer-testing/tests/m69_exit_criteria.rs");
    assert!(
        m69.contains("M0 through M113"),
        "the m69 suite's pins moved with the documents"
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
