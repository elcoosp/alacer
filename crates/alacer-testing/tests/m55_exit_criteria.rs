//! M55 exit-criteria integration tests (§13.3, ADR-0061): the AT's
//! text write joins the causal log. ADR-0046's rhythm cashes at the
//! editing tier: a screen reader's `SetValue`, `ReplaceSelectedText`,
//! and `SetTextSelection` consumed the input's `bool`, whose `false`
//! collapsed the closed door (the availability tier, M22) into the
//! same silence as the answered write — and the answered write itself
//! left no fact a trail reader could price. The input's AT door
//! streams one [`alacer_core::TextWriteRecord`] per ask now — the
//! write's shape named by lengths (never the payload: a field's value
//! can carry a secret, the clipboard trail's rule), the ask beside the
//! landing on the selection door (the M44 rule at the text tier), the
//! refusal streaming too.
//!
//! Every test runs the real pipeline (A4) with a real input driven by
//! the real AT action route — the exact path a screen reader's text
//! write rides.

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
/// facts the ask under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// An input mounted through the real pipeline carrying a value the
/// writes can land on: 11 UTF-16 units, the walls of the selection
/// asks ("hello world", the m11 suite's own shape).
fn mount_input(observer: &Share) -> (TestHarness, u64) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let mut input = TextInput::new("field");
    input.set_value("hello world");
    let _ = harness.add_child(root, input.boxed());
    harness.settle();
    let id = harness.query(Role::TextInput).named("field").first().id;
    (harness, id)
}

/// The same input, mounted *unavailable*: the availability tier (M22,
/// ADR-0028) holds the write door closed before any ask arrives.
fn mount_disabled_input(observer: &Share) -> (TestHarness, u64) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let mut input = TextInput::new("field");
    input.set_value("hello world");
    input.set_enabled(false);
    let _ = harness.add_child(root, input.boxed());
    harness.settle();
    let id = harness.query(Role::TextInput).named("field").first().id;
    (harness, id)
}

/// One AT ask routed through the real action route (the exact path a
/// screen reader's text write rides): the consumed verdict.
fn at_ask(harness: &mut TestHarness, id: u64, action: A11yAction) -> bool {
    harness.pipeline_mut().dispatch_a11y_action(id, action)
}

#[test]
fn m55_the_at_text_write_names_its_fact() {
    let observer = Share::new();
    let (mut harness, id) = mount_input(&observer);

    // The whole-value write: the write's shape named by lengths — the
    // value the field carried is the span the write consumed (0 units:
    // "hello world" was mounted, the AT's value replaces it whole —
    // 11 units), the payload's own length beside it, the payload never
    // riding the log (a field's value can carry a secret — the
    // clipboard trail's rule, ADR-0045).
    clear_trail(&observer);
    assert!(at_ask(
        &mut harness,
        id,
        A11yAction::SetValue("written by the reader".into())
    ));
    harness.settle();
    let records = text_writes(&observer);
    assert_eq!(records.len(), 1, "the write answered");
    assert_eq!(records[0].action, TextWriteAction::SetValue);
    assert_eq!(records[0].outcome, TextWriteOutcome::Delivered);
    assert_eq!(
        records[0].replaced,
        Some(11),
        "the whole value was the consumed span"
    );
    assert_eq!(
        records[0].inserted,
        Some(21),
        "the write's own length, the payload staying off the log"
    );
    assert_eq!(
        records[0].requested, None,
        "the whole-value write carries no selection ask"
    );
    assert_eq!(
        (records[0].selection.anchor, records[0].selection.focus),
        (21, 21),
        "the caret landed at the write's end (the whole-value rule)"
    );
    assert_eq!(
        records[0].target,
        Some(id),
        "the funnel stamps the control the action arrived under (M43)"
    );

    // The journal's reset rides beneath the AT's whole-value write —
    // the app-write rule (ADR-0021): undo never fights the AT's write.
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
        "the journal's reset streams beside the door's own fact"
    );
}

#[test]
fn m55_the_replace_rides_the_span_it_consumed() {
    let observer = Share::new();
    let (mut harness, id) = mount_input(&observer);

    // The AT moves the selection first (the M11 vocabulary), then
    // types over it like a paste: the span the write consumed rides
    // beside the payload's length, the caret landing at the write's
    // end with the anchor collapsed onto it.
    assert!(at_ask(
        &mut harness,
        id,
        A11yAction::SetTextSelection(TextSelection::new(0, 5))
    ));
    harness.settle();
    clear_trail(&observer);
    assert!(at_ask(
        &mut harness,
        id,
        A11yAction::ReplaceSelectedText("HOWDY".into())
    ));
    harness.settle();
    let records = text_writes(&observer);
    assert_eq!(records.len(), 1, "the write answered");
    assert_eq!(records[0].action, TextWriteAction::ReplaceSelection);
    assert_eq!(records[0].outcome, TextWriteOutcome::Delivered);
    assert_eq!(
        records[0].replaced,
        Some(5),
        "the selection was the span the write consumed"
    );
    assert_eq!(records[0].inserted, Some(5), "the payload's own length");
    assert_eq!(
        (records[0].selection.anchor, records[0].selection.focus),
        (5, 5),
        "the caret parks at the insertion's end"
    );
}

#[test]
fn m55_the_selection_ask_rides_beside_the_landing() {
    let observer = Share::new();
    let (mut harness, id) = mount_input(&observer);

    // The selection ask rides beside the landing (the M44 rule at the
    // text tier): the forgiving read side floors to scalar boundaries
    // and clamps to the value's end — a clamp the trail could not show
    // is one record's story now. The write's shape rides the honest
    // absence: a selection is not a payload, nothing was replaced or
    // inserted.
    clear_trail(&observer);
    assert!(at_ask(
        &mut harness,
        id,
        A11yAction::SetTextSelection(TextSelection::new(2, 100))
    ));
    harness.settle();
    let records = text_writes(&observer);
    assert_eq!(records.len(), 1, "the ask answered");
    assert_eq!(records[0].action, TextWriteAction::SetSelection);
    assert_eq!(records[0].outcome, TextWriteOutcome::Delivered);
    assert_eq!(
        records[0].requested,
        Some(TextSelection::new(2, 100)),
        "the ask beside the landing — the clamp visible as data"
    );
    assert_eq!(
        (records[0].selection.anchor, records[0].selection.focus),
        (2, 11),
        "the landing clamped at the value's own wall"
    );
    assert_eq!(
        records[0].replaced, None,
        "a selection consumes no span — the honest absence"
    );
    assert_eq!(records[0].inserted, None, "a selection inserts nothing");
}

#[test]
fn m55_the_closed_door_refuses_honestly() {
    let observer = Share::new();
    let (mut harness, id) = mount_disabled_input(&observer);

    // The availability tier (M22, ADR-0028): an unavailable input has
    // no write door. The verdict consumes nothing and the refusal
    // streams — the fact the `bool` collapsed into one silence, named
    // where the value tier named its own four (ADR-0046's rhythm). The
    // write's shape rides the honest absence (nothing moved), the
    // selection the field carries riding untouched beside it.
    clear_trail(&observer);
    assert!(
        !at_ask(&mut harness, id, A11yAction::SetValue("typed".into())),
        "the closed door consumes nothing"
    );
    harness.settle();
    let records = text_writes(&observer);
    assert_eq!(records.len(), 1, "the refusal is a fact the log carries");
    assert_eq!(records[0].action, TextWriteAction::SetValue);
    assert_eq!(
        records[0].outcome,
        TextWriteOutcome::Refused(alacer_core::TextWriteRefusal::Unavailable)
    );
    assert_eq!(records[0].replaced, None, "nothing was replaced");
    assert_eq!(records[0].inserted, None, "nothing was inserted");
    assert_eq!(
        (records[0].selection.anchor, records[0].selection.focus),
        (11, 11),
        "the field's own selection rode untouched (the set_value caret rule)"
    );
    assert_eq!(
        records[0].target,
        Some(id),
        "the refusal names the control it refused at (the stamp's rule)"
    );
}

#[test]
fn m55_the_surface_rides_the_same_cut() {
    // The seam ships where the vocabulary lives: the door's note, the
    // trail module, the wire event, the client's echo, the decision
    // record itself.
    let input = read_workspace_file("crates/alacer-widgets/src/text_input.rs");
    for fragment in [
        "note_text_write",
        "TextWriteAction::SetValue",
        "TextWriteRefusal::Unavailable",
    ] {
        assert!(
            input.contains(fragment),
            "the AT text door streams its fact: {fragment}"
        );
    }
    let trail = read_workspace_file("crates/alacer-core/src/text_write.rs");
    assert!(
        trail.contains("ADR-0061"),
        "the trail module names its decision record"
    );
    let wire = read_workspace_file("crates/alacer-inspect/src/wire.rs");
    assert!(
        wire.contains("pub fn wire_text_write"),
        "the wire speaks the text-write event"
    );
    let client = read_workspace_file("crates/alacer-inspect/src/inspector.html");
    assert!(
        client.contains("case \"text_write\""),
        "the embedded client renders the text-write echo"
    );

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0061-m55-scope.md");
    assert!(
        adr.contains("M55 scope") && adr.contains("text write"),
        "ADR-0061 documents the text-write scope"
    );
}

#[test]
fn m55_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // features bullet carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M61",
        "sixty-eight decision records",
        "Since M55 the AT's text writes name their door too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `at_text_writes` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`at_text_writes`](../crates/alacer/examples/at_text_writes.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0061",
        "68 decision records",
        "the AT's text write joins the causal log",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`text_write`")
            && protocol.contains("the write's shape named by lengths"),
        "the protocol document carries the text-write event"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M55 — the AT's text write joins the causal log (ADR-0061)**"),
        "the CHANGELOG carries the M55 entry"
    );

    // Every pin the M-count move touched moved together — the suites'
    // document pins ride the same cut, the chain cannot drift.
    for suite in [
        "m7", "m19", "m24", "m35", "m43", "m49", "m50", "m51", "m52", "m53", "m54",
    ] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M61"),
            "the {suite} suite's README pin moved with the documents"
        );
    }
    for suite in [
        "m19", "m24", "m35", "m43", "m49", "m50", "m51", "m52", "m53", "m54",
    ] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("68 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
    for suite in [
        "m24", "m35", "m43", "m49", "m50", "m51", "m52", "m53", "m54",
    ] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("sixty-eight decision records"),
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
