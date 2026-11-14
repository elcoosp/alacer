//! M57 exit-criteria integration tests (§13.3, ADR-0063): the boundary's
//! drops join the causal log. ADR-0061's third kill rule cashes — a
//! consumer needs the platform's defensive drops on the log: the
//! `SetValue` without a payload, the `SetTextSelection` aimed at another
//! node, the actions no widget contract answers, and the asks routed to
//! an id no tree node carries — every one dropped *before* a widget's
//! door, traced today, streamed as one [`alacer_core::RouteDropRecord`]
//! now. A drop is never a munged widget-door fact: the record is the
//! route's own vocabulary, the action named by its stable tag, the
//! species naming why the boundary refused, the aimed id riding
//! verbatim.
//!
//! Every test runs the real pipeline (A4) with a real ask routed the
//! real way — a consumed ask streams no drop (the honest absence), a
//! dropped ask streams its fact.

use alacer_core::inspect::{InspectEvent, InspectObserver};
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The route-drop trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed route-drop facts, oldest first.
    fn route_drops(&self) -> Vec<alacer_core::RouteDropRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::RouteDrop(record) => Some(*record),
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

/// The streamed route-drop facts, oldest first.
fn route_drops(observer: &Share) -> Vec<alacer_core::RouteDropRecord> {
    observer.0.borrow().route_drops()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the ask under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// An input mounted through the real pipeline (a live address the AT
/// route can also reach — the honest-absence beat needs it).
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

#[test]
fn m57_the_unknown_target_refuses_and_names_its_fact() {
    let observer = Share::new();
    let (mut harness, _id) = mount_input(&observer);

    // The ask aimed at an id no tree node carries: the funnel refuses
    // before any widget's door (the consumed verdict `false`, byte for
    // byte) — and the drop streams now, the aimed id riding verbatim,
    // the action named by its stable tag. A munged widget-door fact
    // would be the lie the kill rule refuses: no door ever answered.
    clear_trail(&observer);
    assert!(
        !harness
            .pipeline_mut()
            .dispatch_a11y_action(0xDEAD_BEEF, A11yAction::SetValue("typed".into())),
        "an unknown target consumes nothing"
    );
    harness.settle();
    let drops = route_drops(&observer);
    assert_eq!(drops.len(), 1, "the drop is a fact the log carries");
    assert_eq!(drops[0].action, "set_value");
    assert_eq!(
        drops[0].species,
        alacer_core::RouteDropSpecies::UnknownTarget
    );
    assert_eq!(
        drops[0].target,
        Some(0xDEAD_BEEF),
        "the aim the boundary refused, verbatim"
    );
}

#[test]
fn m57_the_routed_ask_streams_no_drop() {
    let observer = Share::new();
    let (mut harness, id) = mount_input(&observer);

    // A delivered ask is not a drop: the route streams nothing — the
    // honest absence beside the widget-door facts the ask does stream
    // (the M55 record at the editing tier). One funnel, two verdicts,
    // each fact on its own tier.
    clear_trail(&observer);
    assert!(
        harness
            .pipeline_mut()
            .dispatch_a11y_action(id, A11yAction::SetValue("written by the reader".into()))
    );
    harness.settle();
    assert!(
        route_drops(&observer).is_empty(),
        "a consumed ask names no drop"
    );
}

#[test]
fn m57_the_surface_rides_the_same_cut() {
    // The seam ships where the vocabulary lives: the route trail, the
    // funnel's drop, the boundary's note sites, the wire event, the
    // client's echo, the decision record itself.
    let trail = read_workspace_file("crates/alacer-core/src/route_drop.rs");
    assert!(
        trail.contains("note_route_drop") && trail.contains("ADR-0063"),
        "the route trail module names its decision record"
    );
    let inspect = read_workspace_file("crates/alacer-core/src/inspect.rs");
    assert!(
        inspect.contains("pub struct RouteDropRecord")
            && inspect.contains("pub enum RouteDropSpecies"),
        "the drop vocabulary lives beside the records"
    );
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    assert!(
        passes.contains("RouteDropSpecies::UnknownTarget"),
        "the funnel's own drop names its species"
    );
    let platform = read_workspace_file("crates/alacer-platform/src/lib.rs");
    assert!(
        platform.contains("note_route_drop") && platform.contains("RouteDropSpecies::NoPayload"),
        "the boundary's drop sites stream their facts"
    );
    let wire = read_workspace_file("crates/alacer-inspect/src/wire.rs");
    assert!(
        wire.contains("pub fn wire_route_drop"),
        "the wire speaks the route-drop event"
    );
    let client = read_workspace_file("crates/alacer-inspect/src/inspector.html");
    assert!(
        client.contains("case \"route_drop\""),
        "the embedded client renders the route-drop echo"
    );

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0063-m57-scope.md");
    assert!(
        adr.contains("M57 scope") && adr.contains("drop"),
        "ADR-0063 documents the boundary-drop scope"
    );
}

#[test]
fn m57_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // trail sentence carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M64",
        "seventy-one decision records",
        "Since M57 the boundary's drops stream their fact too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `route_drops` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`route_drops`](../crates/alacer/examples/route_drops.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0063",
        "71 decision records",
        "the boundary's drops join the causal log",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`route_drop`") && protocol.contains("unknown_target"),
        "the protocol document carries the route-drop event"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M57 — the boundary's drops join the causal log (ADR-0063)**"),
        "the CHANGELOG carries the M57 entry"
    );

    // The sibling suites' pins ride the same cut (the m56 suite pins
    // the chain too — one sweep, no drift).
    let m56 = read_workspace_file("crates/alacer-testing/tests/m56_exit_criteria.rs");
    assert!(
        m56.contains("M0 through M64"),
        "the m56 suite's pins moved with the documents"
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
