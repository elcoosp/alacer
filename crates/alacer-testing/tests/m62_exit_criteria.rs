//! M62 exit-criteria integration tests (§13.3, ADR-0068): the typing
//! door's refusals join the causal log. ADR-0064's second kill rule
//! cashes — a consumer prices the *closed field's wall* on the log: the
//! routed edit key a disabled field ignored, the IME's commit a closed
//! field turned away — the M58 pin read the wall as pure silence ("no
//! edit lands, no fact queues"), and a typed run beside a wall was
//! indistinguishable from an honest gap. The record grows its outcome
//! beside the landed shape (the value trail's own refusal vocabulary,
//! ADR-0046) — [`alacer_core::KeyEditOutcome::Landed`] on every
//! delivered edit (the M58 shape, additive at version 1) and
//! [`alacer_core::KeyEditOutcome::Refused`]([`alacer_core::KeyEditRefusal::Unavailable`])
//! on the wall: the ask the open door would have answered named by its
//! action (the classifier's own match — never every routed keystroke),
//! the shape priced at nothing (the value untouched), never a munged
//! delivered edit. ADR-0066's third kill rule rides beside: the
//! chords' refusals stream behind their own actions (`paste`/`cut`
//! under the `clipboard` driver) — a chord the wall turned away names
//! itself, while the engine never ran (the clipboard trail answers
//! none: one fact per trail).
//!
//! The window the facts ride is the availability tier's own (M22,
//! ADR-0028): focus is evicted the frame the control becomes
//! unavailable — the eviction pass, not the door, so a keystroke
//! routed in the *same frame* still reaches the held field's door and
//! the wall answers it. The asks that arrive after the eviction are
//! the boundary's own silence (the key route's fallbacks, the IME
//! route's `no_focused_field` drop — M59's vocabulary), never a
//! munged typing-door fact. Every test routes the raw events the way
//! one frame's batch arrives — the key route and the IME route, the
//! funnel stamp riding — and settles once to drain.

use alacer_core::events::{ImeEvent, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver};
use alacer_core::widget::{Role, TextSelection};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The key-edit trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the wall's facts are the assertion source).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed key-edit facts, oldest first.
    fn key_edits(&self) -> Vec<alacer_core::KeyEditRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::KeyEdit(record) => Some(*record),
                _ => None,
            })
            .collect()
    }

    /// The streamed clipboard round trips, oldest first.
    fn clipboards(&self) -> Vec<alacer_core::ClipboardRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::Clipboard(record) => Some(*record),
                _ => None,
            })
            .collect()
    }
}

impl InspectObserver for TrailObserver {
    fn observe(&mut self, event: InspectEvent) {
        if self.events.len() >= 64 {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m-suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<TrailObserver>>);

impl Share {
    fn new() -> Self {
        Self(std::rc::Rc::new(std::cell::RefCell::new(
            TrailObserver::new(),
        )))
    }
}

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The streamed key-edit facts, oldest first.
fn key_edits(observer: &Share) -> Vec<alacer_core::KeyEditRecord> {
    observer.0.borrow().key_edits()
}

/// The streamed clipboard round trips, oldest first.
fn clipboards(observer: &Share) -> Vec<alacer_core::ClipboardRecord> {
    observer.0.borrow().clipboards()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the wall under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// An input mounted through the real pipeline, focused and settled —
/// the typing door's own mount. Returns the harness, the input's
/// AccessKit-stable id (the funnel's stamp vocabulary) and its arena
/// node.
fn mount_input(observer: &Share) -> (TestHarness, u64, alacer_core::NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("field").boxed());
    harness.settle();
    let id = harness.query(Role::TextInput).named("field").first().id;
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    (harness, id, node)
}

/// The mounted input held down by the availability tier (M22,
/// ADR-0028) — *without* a settle: the route still holds the field
/// (the eviction is the frame's pass, not the door), so the wall's
/// window is open and the keys routed this frame reach the door.
fn disable_without_settling(harness: &mut TestHarness, node: alacer_core::NodeId) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_enabled(false);
    }
    drop(guard);
}

/// A Ctrl press (the clipboard chords' command set; ADR-0020) — the
/// exact event the platform delivers when the user pastes or cuts.
fn ctrl(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

#[test]
fn m62_the_wall_names_the_ask_it_turned_away() {
    let observer = Share::new();
    let (mut harness, id, node) = mount_input(&observer);
    disable_without_settling(&mut harness, node);
    // The asks the open door would have answered with facts, routed in
    // the wall's own frame: two typed characters and a backspace — the
    // wall names each by its action, never a per-keystroke flood of
    // keys that edit nothing.
    for ch in "ab".chars() {
        let _ = harness.pipeline_mut().dispatch_key(&KeyEvent::char(ch));
    }
    let _ = harness
        .pipeline_mut()
        .dispatch_key(&KeyEvent::pressed(Key::Backspace));
    harness.settle();
    let edits = key_edits(&observer);
    assert_eq!(
        edits.len(),
        3,
        "the wall streams one fact per edit ask, never one per keystroke routed"
    );
    let actions: Vec<_> = edits.iter().map(|edit| edit.action).collect();
    assert_eq!(
        actions,
        vec![
            alacer_core::KeyEditAction::Insert,
            alacer_core::KeyEditAction::Insert,
            alacer_core::KeyEditAction::Backspace,
        ],
        "the refusal names the ask the open door would have answered"
    );
    for edit in &edits {
        assert_eq!(
            edit.outcome,
            alacer_core::KeyEditOutcome::Refused(alacer_core::KeyEditRefusal::Unavailable),
            "the wall's own species, the value tier's vocabulary (ADR-0046)"
        );
        assert_eq!(
            edit.driver,
            alacer_core::KeyEditDriver::Keyboard,
            "the keyboard drove every refused ask"
        );
        assert_eq!(edit.replaced, 0, "the wall consumed no span");
        assert_eq!(edit.inserted, 0, "the wall carried no payload");
        assert!(
            !edit.coalesced,
            "a refusal never rides the flood rule — one fact per ask"
        );
        assert_eq!(
            edit.target,
            Some(id),
            "the key route's stamp: the fact names the control the wall held"
        );
    }
    // And the wall moved nothing: the field's value is the tree's own
    // vocabulary the AT reads.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        assert!(
            input.value().is_empty(),
            "the wall typed nothing into the field"
        );
    }
    drop(guard);
    for edit in &edits {
        assert_eq!(
            edit.selection,
            TextSelection::new(0, 0),
            "the record carries the field's untouched selection"
        );
    }
}

#[test]
fn m62_the_chords_refuse_behind_their_own_actions() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);
    disable_without_settling(&mut harness, node);
    harness.set_clipboard("seed");
    clear_trail(&observer);
    let _ = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('v')));
    let _ = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('x')));
    harness.settle();
    let edits = key_edits(&observer);
    assert_eq!(
        edits.len(),
        2,
        "each chord the wall turned away is one fact"
    );
    assert_eq!(
        edits[0].action,
        alacer_core::KeyEditAction::Paste,
        "the paste's refusal is never a munged Insert (ADR-0066's own word)"
    );
    assert_eq!(
        edits[1].action,
        alacer_core::KeyEditAction::Cut,
        "the cut's refusal is never a munged Delete"
    );
    for edit in &edits {
        assert_eq!(
            edit.driver,
            alacer_core::KeyEditDriver::Clipboard,
            "the chords' refusals name the chord's own driver (the M60 rule)"
        );
        assert_eq!(
            edit.outcome,
            alacer_core::KeyEditOutcome::Refused(alacer_core::KeyEditRefusal::Unavailable),
            "the wall's species, behind the chord's own action"
        );
        assert_eq!(edit.replaced, 0, "the wall consumed no span");
        assert_eq!(edit.inserted, 0, "the wall carried no payload");
    }
    // The engine never ran: the door refused before the read — the
    // clipboard trail answers none (one fact per trail, never a
    // double-priced refusal).
    assert!(
        clipboards(&observer).is_empty(),
        "a chord refused at the door never reached the clipboard engine"
    );
}

#[test]
fn m62_the_copy_edits_nothing_at_the_wall_too() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);
    disable_without_settling(&mut harness, node);
    let _ = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('c')));
    harness.settle();
    assert!(
        key_edits(&observer).is_empty(),
        "a copy edits nothing at an open door — the wall prices edits only"
    );
    assert!(
        clipboards(&observer).is_empty(),
        "the closed door never reached the engine either"
    );
}

#[test]
fn m62_navigation_rides_no_wall_fact() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);
    disable_without_settling(&mut harness, node);
    // The keys an open door answers without editing (the caret's walk):
    // the wall names none — the refusal prices edit asks, never every
    // routed keystroke.
    let _ = harness
        .pipeline_mut()
        .dispatch_key(&KeyEvent::pressed(Key::ArrowLeft));
    let _ = harness
        .pipeline_mut()
        .dispatch_key(&KeyEvent::pressed(Key::ArrowRight));
    let _ = harness
        .pipeline_mut()
        .dispatch_key(&KeyEvent::pressed(Key::Home));
    let _ = harness
        .pipeline_mut()
        .dispatch_key(&KeyEvent::pressed(Key::End));
    harness.settle();
    assert!(
        key_edits(&observer).is_empty(),
        "navigation keys edit nothing at an open door — no wall facts either"
    );
}

#[test]
fn m62_the_release_rides_no_fact() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);
    disable_without_settling(&mut harness, node);
    // The open door acts on presses only (§10.3); the wall mirrors it —
    // a release never names an ask.
    let _ = harness.pipeline_mut().dispatch_key(&KeyEvent {
        key: Key::Backspace,
        state: KeyState::Released,
        modifiers: Modifiers::default(),
        text: None,
    });
    harness.settle();
    assert!(
        key_edits(&observer).is_empty(),
        "the wall prices the press's ask, never a release"
    );
}

#[test]
fn m62_the_ime_commit_refuses_at_the_wall() {
    let observer = Share::new();
    let (mut harness, id, node) = mount_input(&observer);
    disable_without_settling(&mut harness, node);
    // The composition's own tier answers none on a closed door (the M61
    // silence rides the same wall): a preedit shown nowhere streams no
    // composing fact — and no key-edit fact beside it.
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Preedit {
        text: "composing".to_string(),
        cursor_utf16: Some(3),
    });
    assert!(
        key_edits(&observer).is_empty(),
        "a composition the closed field never showed edits nothing"
    );
    // The commit the IME delivers into the closed field, in the same
    // frame: the typing door's own ask, refused with the wall's
    // species — the driver named (the attribution rule at the typing
    // tier). The empty commit beside it asks nothing (the open door's
    // honest absence): one fact for the whole batch.
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Commit {
        text: "composed".to_string(),
    });
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Commit {
        text: String::new(),
    });
    harness.settle();
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the commit's refusal is one fact");
    assert_eq!(edits[0].action, alacer_core::KeyEditAction::Insert);
    assert_eq!(
        edits[0].driver,
        alacer_core::KeyEditDriver::Ime,
        "the IME drove the refused commit"
    );
    assert_eq!(
        edits[0].outcome,
        alacer_core::KeyEditOutcome::Refused(alacer_core::KeyEditRefusal::Unavailable),
        "the wall's species, the IME commit included"
    );
    assert_eq!(edits[0].replaced, 0, "the wall consumed no span");
    assert_eq!(edits[0].inserted, 0, "the composition landed nowhere");
    assert_eq!(
        edits[0].target,
        Some(id),
        "the IME route's stamp names the control (the M58 rule's own ride)"
    );
}

#[test]
fn m62_the_evicted_frame_answers_no_wall_fact() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);
    // The frame's pass evicts the held focus (M22's sentinel
    // discipline): the keystrokes routed *after* it are the boundary's
    // own silence — the key route's fallbacks, never a munged
    // typing-door fact (the door heard no ask).
    disable_without_settling(&mut harness, node);
    harness.settle();
    clear_trail(&observer);
    for ch in "ab".chars() {
        let _ = harness.pipeline_mut().dispatch_key(&KeyEvent::char(ch));
    }
    let _ = harness
        .pipeline_mut()
        .dispatch_key(&KeyEvent::pressed(Key::Backspace));
    harness.settle();
    assert!(
        key_edits(&observer).is_empty(),
        "an ask the boundary carried to no field is no typing-door fact — the route's own silence"
    );
}

#[test]
fn m62_the_open_door_still_streams_landed() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);
    harness.type_text("hi");
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the typed run is one fact (the flood rule)");
    assert_eq!(
        edits[0].outcome,
        alacer_core::KeyEditOutcome::Landed,
        "the M58 shape, named additively — the default the wall refuses"
    );
    assert_eq!(edits[0].inserted, 2, "the run's own shape");
    // And the re-focused door lands again: the wall is the tier's, not
    // the field's identity.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_enabled(true);
    }
    drop(guard);
    harness.settle();
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    clear_trail(&observer);
    harness.type_text("!");
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the reopened door answers");
    assert_eq!(
        edits[0].outcome,
        alacer_core::KeyEditOutcome::Landed,
        "the reopened door's edit landed"
    );
    assert_eq!(edits[0].inserted, 1, "the landing's own shape");
}

#[test]
fn m62_the_wall_never_supersedes_the_run() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);
    // The typed run lands while the door is open — its record streams
    // with the frame...
    harness.type_text("a");
    // ...then the tier holds the field down and the next keystroke
    // arrives in the wall's own frame: the refusal pushes its own fact
    // beside the run's record — the supersede is the delivered run's
    // fold, never a wall's (the munged merge the kill rule refuses).
    disable_without_settling(&mut harness, node);
    let _ = harness.pipeline_mut().dispatch_key(&KeyEvent::char('b'));
    harness.settle();
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 2, "the wall's fact lands beside the run's");
    assert_eq!(
        edits[0].outcome,
        alacer_core::KeyEditOutcome::Landed,
        "the delivered run keeps its own record"
    );
    assert_eq!(
        edits[1].outcome,
        alacer_core::KeyEditOutcome::Refused(alacer_core::KeyEditRefusal::Unavailable),
        "the refusal is never merged into the run it follows"
    );
    assert_eq!(edits[0].inserted, 1, "the run's shape stayed the run's");
    assert_eq!(
        edits[1].action,
        alacer_core::KeyEditAction::Insert,
        "the wall priced the ask, the insert named"
    );
}
