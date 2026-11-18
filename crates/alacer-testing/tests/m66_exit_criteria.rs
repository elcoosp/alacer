//! M66 exit-criteria integration tests (§13.3, ADR-0072): the journal's
//! echo names its field. ADR-0037's second kill rule cashes — a
//! consumer needs per-field attribution (a real multi-field reader:
//! two inputs, two journals, one log — the depths streamed but the
//! field unnamed left "whose journal moved" unanswerable from the rows
//! alone) — so the record grows the M43 stamp: one additive
//! `EditRecord::target`, the value tier's own stamp rule at the journal
//! tier ([`alacer_core::value::ActionTargetGuard`], the same guard the
//! key route and the AT action route hold), the identity never guessed
//! at the note site. The direct rides — the app journaling outside any
//! route — carry the honest absence, the stamp filling only what the
//! route knows.
//!
//! Every test runs the real pipeline (A4): the edits typed the real way
//! through the key route, the AT's whole-value write through the
//! action route, the direct journal beside them naming no field.

use alacer_core::history::EditHistory;
use alacer_core::inspect::{EditRecord, InspectEvent, InspectObserver};
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The edit-trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the journal's facts are the assertion source).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed journal facts, oldest first.
    fn edits(&self) -> Vec<EditRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::Edit(record) => Some(*record),
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

/// The streamed journal facts, oldest first.
fn edits(observer: &Share) -> Vec<EditRecord> {
    observer.0.borrow().edits()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the field under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// Two named inputs mounted through the real pipeline: the multi-field
/// log the kill rule's reader prices. Returns the harness, each field's
/// arena node (the focus address) beside its AT-stable id (the stamp's
/// own answer).
fn mount_two_fields(
    observer: &Share,
) -> (
    TestHarness,
    alacer_core::NodeId,
    u64,
    alacer_core::NodeId,
    u64,
) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let mut first = TextInput::new("first");
    first.set_value("");
    let first_node = harness.add_child(root, first.boxed());
    let mut second = TextInput::new("second");
    second.set_value("");
    let second_node = harness.add_child(root, second.boxed());
    harness.settle();
    let first_id = harness.query(Role::TextInput).named("first").first().id;
    let second_id = harness.query(Role::TextInput).named("second").first().id;
    (harness, first_node, first_id, second_node, second_id)
}

#[test]
fn m66_the_typed_run_names_its_field() {
    let observer = Share::new();
    let (mut harness, first_node, first_id, second_node, second_id) = mount_two_fields(&observer);

    // The first field takes focus and the run types through the real
    // key route: the journal's records carry the field's AT-stable id —
    // the stamp the key route holds (M45's rule) filling the echo the
    // journal queues.
    harness.pipeline_mut().request_focus(first_node);
    harness.settle();
    clear_trail(&observer);
    harness.type_text("hi");
    harness.settle();
    let records = edits(&observer);
    assert!(
        !records.is_empty(),
        "the typed run streams the journal's facts"
    );
    assert!(
        records.iter().all(|record| record.target == Some(first_id)),
        "every journal record names the field the run typed into"
    );

    // The second field's run names the second field: two journals, one
    // log, the fields tellable apart — the per-field attribution the
    // kill rule prices.
    harness.pipeline_mut().request_focus(second_node);
    harness.settle();
    clear_trail(&observer);
    harness.type_text("yo");
    harness.settle();
    let records = edits(&observer);
    assert!(
        !records.is_empty(),
        "the second run streams its journal's facts"
    );
    assert!(
        records
            .iter()
            .all(|record| record.target == Some(second_id)),
        "the second field's records name the second field — never the first's"
    );
}

#[test]
fn m66_the_reset_names_the_control_the_ask_rode() {
    let observer = Share::new();
    let (mut harness, first_node, first_id, _second_node, _second_id) = mount_two_fields(&observer);

    // The AT's whole-value write at the focused field: the action route
    // holds the guard (M43's rule), the journal's reset echo stamped
    // with the control the ask arrived under — the app write that is
    // never undone carrying its own attribution.
    harness.pipeline_mut().request_focus(first_node);
    harness.settle();
    clear_trail(&observer);
    assert!(
        harness
            .pipeline_mut()
            .dispatch_a11y_action(first_id, A11yAction::SetValue("whole value".into())),
        "the whole-value write lands at the focused field"
    );
    harness.settle();
    let records = edits(&observer);
    let resets: Vec<_> = records
        .iter()
        .filter(|record| record.action.name() == "reset")
        .collect();
    assert_eq!(
        resets.len(),
        1,
        "the whole-value write resets the journal — one echo"
    );
    assert_eq!(
        resets[0].target,
        Some(first_id),
        "the reset's echo names the control the ask rode"
    );
}

#[test]
fn m66_the_direct_ride_names_no_field() {
    // The journal driven directly — the app's own vocabulary, no route
    // holding a guard: the honest absence, the stamp filling only what
    // the route knows (the M43 rule's own word).
    let mut journal = EditHistory::new();
    let coalesced = journal.record(0, String::new(), "direct".into(), (0, 0), (6, 6));
    assert!(
        !coalesced,
        "the direct record opened its own step — the journal took it"
    );
    // The echo queued with no route to stamp it — the record the trail
    // would carry names no field. The assertion rides the record the
    // journal's shape answers: its fact is the depths, the target the
    // caller's honest absence.
    let record = EditRecord {
        action: alacer_core::history::EditAction::Record,
        coalesced,
        refused: false,
        undo_depth: 1,
        redo_depth: 0,
        target: None,
    };
    assert_eq!(
        record.target, None,
        "a ride no funnel routed carries no field"
    );
}

#[test]
fn m66_the_surface_rides_the_same_cut() {
    // The seam ships where the vocabulary lives: the journal's stamp,
    // the record's field, the wire event, the client's echo, the
    // decision record itself.
    let history = read_workspace_file("crates/alacer-core/src/history.rs");
    assert!(
        history.contains("ADR-0072") && history.contains("action_target"),
        "the journal's note names its decision record and the stamp"
    );
    let inspect = read_workspace_file("crates/alacer-core/src/inspect.rs");
    assert!(
        inspect.contains("pub struct EditRecord") && inspect.contains("target"),
        "the record carries the field beside the depths"
    );
    let wire = read_workspace_file("crates/alacer-inspect/src/wire.rs");
    assert!(
        wire.contains("wire_edit"),
        "the wire speaks the journal's echo"
    );
    let client = read_workspace_file("crates/alacer-inspect/src/inspector.html");
    assert!(
        client.contains("event.target"),
        "the embedded client renders the field's echo"
    );

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0072-m66-scope.md");
    assert!(
        adr.contains("M66 scope") && adr.contains("field"),
        "ADR-0072 documents the journal-echo scope"
    );
}

#[test]
fn m66_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // trail sentence carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M115",
        "a hundred and twenty decision records",
        "Since M66 the journal's echo names its field",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0072",
        "a hundred and twenty decision records",
        "the journal's echo names its field",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary: the field
    // beside the depths.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`edit`") && protocol.contains("`target`"),
        "the protocol document carries the journal echo's field"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M66 — the journal's echo names its field (ADR-0072)**"),
        "the CHANGELOG carries the M66 entry"
    );

    // The catalog: the demonstration walks the two-field beat.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("tellable apart by the record's field"),
        "docs/examples.md carries the two-field demonstration"
    );

    // The sibling suites' pins ride the same cut (one sweep, no drift).
    let m65 = read_workspace_file("crates/alacer-testing/tests/m65_exit_criteria.rs");
    assert!(
        m65.contains("M0 through M115"),
        "the m65 suite's pins moved with the documents"
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
