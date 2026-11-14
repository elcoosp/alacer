//! M56 exit-criteria integration tests (§13.3, ADR-0062): the app's text
//! write joins the causal log. ADR-0061's second kill rule cashes — a
//! consumer needs the app's own `set_value` (and the view
//! reconciliation's writes, which ride the same door) attributed on the
//! log: today the AT's asks stream one [`alacer_core::TextWriteRecord`]
//! each, but the app's writes streamed only the journal's reset echo (a
//! fact about the depths, not the door's fact about the write). The
//! record grows the additive `source` (the M30 attribution rule at the
//! text tier): `at_door` for the AT's asks, `app_door` for the app's
//! writes — one door vocabulary, every write attributable, the payload
//! never riding the log (a field's value can carry a secret, the
//! clipboard trail's rule, ADR-0045).
//!
//! Every test runs the real pipeline (A4) with a real input written
//! through the real app door (the mutate seam — the exact path app code
//! and the view reconciliation's patches ride) or the real AT action
//! route.

use alacer_core::inspect::{InspectEvent, InspectObserver, TextWriteAction, TextWriteOutcome};
use alacer_core::widget::{A11yAction, Role, TextSelection};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The text-write trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the write's facts are the assertion source).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed text-write facts, oldest first.
    fn text_writes(&self) -> Vec<alacer_core::TextWriteRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::TextWrite(record) => Some(*record),
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

/// The streamed text-write facts, oldest first.
fn text_writes(observer: &Share) -> Vec<alacer_core::TextWriteRecord> {
    observer.0.borrow().text_writes()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the write under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// An input mounted through the real pipeline carrying a value the
/// writes can land on: 11 UTF-16 units ("hello world", the m11 suite's
/// own shape). Returns the harness, the input's AccessKit-stable id
/// (the AT route's address) and its arena node (the mutate seam's).
fn mount_input(observer: &Share) -> (TestHarness, u64, alacer_core::NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let mut input = TextInput::new("field");
    input.set_value("hello world");
    let node = harness.add_child(root, input.boxed());
    harness.settle();
    let id = harness.query(Role::TextInput).named("field").first().id;
    (harness, id, node)
}

/// One app-door write through the real mutate seam (the exact path the
/// consumer's imperative code — and the view reconciliation's patch —
/// rides): the write lands, the masks queue on the guard's drop.
fn app_write(harness: &mut TestHarness, node: alacer_core::NodeId, value: &str) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_value(value);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

#[test]
fn m56_the_app_write_names_its_fact() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(&observer);

    // The app's whole-value write through the mutate seam: the door's
    // own fact streams beside the journal's reset echo (the reset
    // names the depths, the door names the write) — the shape named by
    // lengths, the payload never riding the log, `source` naming the
    // app door (the kill rule's own question: did the app rewrite this
    // field, what shape, where did the caret land).
    clear_trail(&observer);
    app_write(&mut harness, node, "written by the app");
    harness.settle();
    let records = text_writes(&observer);
    assert_eq!(records.len(), 1, "the app write answered");
    assert_eq!(records[0].action, TextWriteAction::SetValue);
    assert_eq!(records[0].outcome, TextWriteOutcome::Delivered);
    assert_eq!(records[0].source, alacer_core::TextWriteSource::AppDoor);
    assert_eq!(
        records[0].replaced,
        Some(11),
        "the whole value was the span the write consumed"
    );
    assert_eq!(
        records[0].inserted,
        Some(18),
        "the write's own length, the payload staying off the log"
    );
    assert_eq!(
        records[0].requested, None,
        "the whole-value write carries no selection ask"
    );
    assert_eq!(
        (records[0].selection.anchor, records[0].selection.focus),
        (18, 18),
        "the caret landed at the write's end (the whole-value rule)"
    );
    assert_eq!(
        records[0].target, None,
        "the app door rode no funnel route — the stamp's honest absence"
    );

    // The journal's reset still streams beside the door's own fact —
    // the app-write rule (ADR-0021): undo never fights the app's write.
    // The record names the door; the journal names the depths.
    let edits = observer
        .0
        .borrow()
        .events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Edit(record) => Some(*record),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        edits
            .iter()
            .any(|record| record.action == alacer_core::history::EditAction::Reset),
        "the journal's reset streams beside the app door's own fact"
    );
}

#[test]
fn m56_the_at_door_names_its_rider() {
    let observer = Share::new();
    let (mut harness, id, _node) = mount_input(&observer);

    // The AT's whole-value write: the same action vocabulary, the other
    // door — the record names the asker (`at_door`) beside the funnel's
    // stamp (M43). One ask, one fact: the write's raw side rides
    // beneath the AT door's note, never a second record (the app door's
    // fact is the app door's alone).
    clear_trail(&observer);
    assert!(
        harness
            .pipeline_mut()
            .dispatch_a11y_action(id, A11yAction::SetValue("written by the reader".into()))
    );
    harness.settle();
    let records = text_writes(&observer);
    assert_eq!(records.len(), 1, "one ask, one fact — never a double note");
    assert_eq!(records[0].action, TextWriteAction::SetValue);
    assert_eq!(records[0].outcome, TextWriteOutcome::Delivered);
    assert_eq!(records[0].source, alacer_core::TextWriteSource::AtDoor);
    assert_eq!(records[0].replaced, Some(11));
    assert_eq!(records[0].inserted, Some(21));
    assert_eq!(
        records[0].target,
        Some(id),
        "the funnel stamps the control the action arrived under (M43)"
    );
}

#[test]
fn m56_the_selection_ask_rides_its_door_too() {
    let observer = Share::new();
    let (mut harness, id, _node) = mount_input(&observer);

    // Every AT text write names its source — the selection ask (M55's
    // own record) grows the same attribution, the ask beside the
    // landing unchanged (the M44 rule at the text tier).
    clear_trail(&observer);
    assert!(
        harness
            .pipeline_mut()
            .dispatch_a11y_action(id, A11yAction::SetTextSelection(TextSelection::new(2, 100)))
    );
    harness.settle();
    let records = text_writes(&observer);
    assert_eq!(records.len(), 1, "the ask answered");
    assert_eq!(records[0].action, TextWriteAction::SetSelection);
    assert_eq!(records[0].source, alacer_core::TextWriteSource::AtDoor);
    assert_eq!(
        records[0].requested,
        Some(TextSelection::new(2, 100)),
        "the ask beside the landing — the clamp visible as data"
    );
}

#[test]
fn m56_the_surface_rides_the_same_cut() {
    // The seam ships where the vocabulary lives: the app door's note
    // beside the AT door's, the attribution enum, the wire field, the
    // client's echo, the decision record itself.
    let input = read_workspace_file("crates/alacer-widgets/src/text_input.rs");
    for fragment in [
        "TextWriteSource::AppDoor",
        "TextWriteSource::AtDoor",
        "fn apply_value",
    ] {
        assert!(
            input.contains(fragment),
            "the doors name their rider: {fragment}"
        );
    }
    let inspect = read_workspace_file("crates/alacer-core/src/inspect.rs");
    assert!(
        inspect.contains("pub enum TextWriteSource"),
        "the attribution vocabulary lives beside the record"
    );
    let wire = read_workspace_file("crates/alacer-inspect/src/wire.rs");
    assert!(
        wire.contains("pub fn wire_text_write") && wire.contains("source:"),
        "the wire carries the write's source"
    );
    let client = read_workspace_file("crates/alacer-inspect/src/inspector.html");
    assert!(
        client.contains("event.source"),
        "the embedded client renders the write's source"
    );

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0062-m56-scope.md");
    assert!(
        adr.contains("M56 scope") && adr.contains("app door"),
        "ADR-0062 documents the app-door scope"
    );
}

#[test]
fn m56_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // trail sentence carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M64",
        "seventy-one decision records",
        "Since M56 the app's own text writes name their door too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `app_text_writes` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`app_text_writes`](../crates/alacer/examples/app_text_writes.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0062",
        "71 decision records",
        "the app's text write joins the causal log",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`source`") && protocol.contains("app_door"),
        "the protocol document carries the write's source"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M56 — the app's text write joins the causal log (ADR-0062)**"),
        "the CHANGELOG carries the M56 entry"
    );

    // Every pin the M-count move touched moved together — the suites'
    // document pins ride the same cut, the chain cannot drift.
    for suite in [
        "m7", "m19", "m24", "m35", "m43", "m49", "m50", "m51", "m52", "m53", "m54", "m55",
    ] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M64"),
            "the {suite} suite's README pin moved with the documents"
        );
    }
    for suite in [
        "m19", "m24", "m35", "m43", "m49", "m50", "m51", "m52", "m53", "m54", "m55",
    ] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("71 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
    for suite in [
        "m24", "m35", "m43", "m49", "m50", "m51", "m52", "m53", "m54", "m55",
    ] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("seventy-one decision records"),
            "the {suite} suite's README status pin moved with the documents"
        );
    }
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
