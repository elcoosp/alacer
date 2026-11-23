//! M68 exit-criteria integration tests (§13.3, ADR-0074): the payloadless
//! chord names its read. M62's pre-registered kill rule cashes — a
//! consumer prices the payloadless chord (the copy or cut whose
//! selection is empty: the chord arrived, the field's door answered,
//! and nothing was there to move) → the wall grows the read's species
//! beside the door's
//! ([`alacer_core::KeyEditRefusal::EmptySelection`], the `Unavailable`
//! wall's neighbor at the typing tier), never a double-priced round
//! trip — the round trip keeps its one clipboard fact (the
//! `NoSelection` species the M39 trail already streams), the field's
//! own answer riding the key-edit trail as the chord's second fact, one
//! per trail, the delivered cut's exact shape mirrored at the wall.
//!
//! Every test runs the real pipeline (A4) with the real clipboard chords
//! routed the way the platform's keys arrive — the key route, the
//! funnel stamp riding.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver};
use alacer_core::widget::{Role, TextSelection};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The key-edit trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the chords' facts are the assertion source).
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
/// facts the chord under test streamed, nothing earlier.
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
fn m68_the_payloadless_copy_names_its_read() {
    let observer = Share::new();
    let (mut harness, id, _node) = mount_input(&observer);

    // The empty selection: the chord arrives, the door answers, and
    // nothing was there to move — the read's species, named.
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('c')));
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the payloadless copy is one fact");
    let edit = edits[0];
    assert_eq!(
        edit.action,
        alacer_core::KeyEditAction::Copy,
        "the chord's own action, never a munge"
    );
    assert_eq!(edit.driver, alacer_core::KeyEditDriver::Clipboard);
    assert_eq!(
        edit.outcome,
        alacer_core::KeyEditOutcome::Refused(alacer_core::KeyEditRefusal::EmptySelection),
        "the wall grows the read's species beside the door's"
    );
    assert_eq!(edit.replaced, 0, "the shape priced at nothing");
    assert_eq!(edit.inserted, 0, "nothing moved out, nothing moved in");
    assert_eq!(
        edit.selection,
        TextSelection::new(0, 0),
        "the untouched selection, the caret the field kept"
    );
    assert_eq!(
        edit.target,
        Some(id),
        "the key route's stamp: the fact names the control it rode"
    );

    // The round trip keeps its own fact: one per trail, never a
    // double-priced ask — the clipboard's `NoSelection` beside the
    // field's refusal.
    let trips = clipboards(&observer);
    assert_eq!(trips.len(), 1, "the round trip is the clipboard trail's");
    assert!(
        matches!(
            trips[0].outcome,
            alacer_core::ClipboardOutcome::Refused(alacer_core::ClipboardRefusal::NoSelection)
        ),
        "the clipboard trail names its own species, unchanged"
    );
}

#[test]
fn m68_the_payloadless_cut_refuses_the_same_read() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);

    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('x')));
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the payloadless cut is one fact");
    let edit = edits[0];
    assert_eq!(edit.action, alacer_core::KeyEditAction::Cut);
    assert_eq!(
        edit.outcome,
        alacer_core::KeyEditOutcome::Refused(alacer_core::KeyEditRefusal::EmptySelection),
        "the cut's read names the same species — the chord's own door"
    );
    assert_eq!(
        edit.selection,
        TextSelection::new(0, 0),
        "a refused cut deletes nothing"
    );
}

#[test]
fn m68_the_delivered_chord_stays_one_fact_per_trail() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.set_clipboard("seed");
    harness.type_text("cut me");

    // The delivered cut: the field's change rides the key-edit trail as
    // a Landed fact (the M60 shape), the round trip beside it — the
    // refusal never rides a delivered chord, never a munged outcome.
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('a')));
    harness.key(&ctrl(Key::Char('x')));
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the delivered cut is one key-edit fact");
    assert_eq!(
        edits[0].outcome,
        alacer_core::KeyEditOutcome::Landed,
        "the delivered cut lands, the wall's species beside it never"
    );
    assert_eq!(
        clipboards(&observer).len(),
        1,
        "the round trip is the clipboard trail's own fact, unchanged"
    );
}

#[test]
fn m68_the_delivered_copy_answers_no_typing_door_fact() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);
    harness.type_text("copy me");

    // The delivered copy edits the field nowhere: no key-edit fact (the
    // m60 pin stands — the wall prices refusals, a delivered copy is
    // none).
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('a')));
    harness.key(&ctrl(Key::Char('c')));
    assert!(
        key_edits(&observer).is_empty(),
        "a delivered copy edits nothing — no typing-door fact"
    );
    assert_eq!(
        clipboards(&observer).len(),
        1,
        "the round trip is the clipboard trail's own fact"
    );
}

#[test]
fn m68_the_refused_paste_keeps_the_honest_absence() {
    let observer = Share::new();
    let (mut harness, _id, _node) = mount_input(&observer);

    // The store was never seeded: the read refuses in the clipboard
    // seam, the engine never ran — the m60 pin stands (the wall prices
    // the field's own read, a paste's read is the clipboard's).
    clear_trail(&observer);
    harness.key(&ctrl(Key::Char('v')));
    assert!(
        key_edits(&observer).is_empty(),
        "a refused paste answers no typing-door fact, unchanged"
    );
    assert_eq!(
        clipboards(&observer).len(),
        1,
        "the refusal is the clipboard trail's fact"
    );
}

#[test]
fn m68_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // trail sentence carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M145",
        "a hundred and fifty decision records",
        "Since M68 the payloadless chord names its read",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0074",
        "a hundred and fifty decision records",
        "the wall grows the read's species beside the door's",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary: the refusal
    // species beside the outcome.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`empty_selection`"),
        "the protocol document carries the read's species"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M68 — the payloadless chord names its read (ADR-0074)**"),
        "the CHANGELOG carries the M68 entry"
    );

    // The catalog: the demonstration walks the read-refusal beat.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("the payloadless copy"),
        "docs/examples.md carries the read-refusal beat"
    );

    // The sibling suites' pins ride the same cut (one sweep, no drift).
    let m67 = read_workspace_file("crates/alacer-testing/tests/m67_exit_criteria.rs");
    assert!(
        m67.contains("M0 through M145"),
        "the m67 suite's pins moved with the documents"
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
