//! M65 exit-criteria integration tests (§13.3, ADR-0071): the route
//! drop names its rider and splits its silence. Two pre-registered
//! prices cash — ADR-0063's third kill rule (the drops need their
//! rider: a reader asking "which route refused my ask" reads the
//! platform boundary's mapping, the dispatch funnel's target
//! resolution, and the IME route's silence apart) and ADR-0065's first
//! kill rule (the dead focused node's drop prices apart from the
//! never-focused one: the vocabulary splits `no_focused_field`, the
//! composition with nowhere to land naming *why* it had nowhere).
//!
//! Every test runs the real pipeline (A4): the ask routed the real way
//! — the record's rider declared where the drop resolves, the species
//! naming the silence, the aim riding verbatim.

use alacer_core::events::ImeEvent;
use alacer_core::inspect::{InspectEvent, InspectObserver, RouteDropRider};
use alacer_core::widget::A11yAction;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The route-drop trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the drops' facts are the assertion source).
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

/// A harness with the observer riding and no field focused: the
/// route's silence is the assertion source. Returns the harness and
/// the field's arena node (the dead-focus beat's address).
fn mount_observer(observer: &Share) -> (TestHarness, alacer_core::NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let field = harness.add_child(root, TextInput::new("field").boxed());
    harness.settle();
    (harness, field)
}

#[test]
fn m65_the_unfocused_drop_names_its_route() {
    let observer = Share::new();
    let (mut harness, _field) = mount_observer(&observer);
    clear_trail(&observer);
    // No field holds focus: the committed composition drops before any
    // widget's door — and the record names the route that dropped it
    // (the IME route's own silence, the M57 rider rule at the route
    // tier), the aim riding its honest absence.
    let _ = harness.pipeline_mut().dispatch_ime(&ImeEvent::Commit {
        text: "よ!".into()
    });
    harness.settle();
    let drops = route_drops(&observer);
    assert_eq!(drops.len(), 1, "the drop is a fact the log carries");
    assert_eq!(drops[0].action, "ime_commit", "the event's own tag");
    assert_eq!(
        drops[0].species,
        alacer_core::RouteDropSpecies::NoFocusedField,
        "the never-focused silence keeps its species"
    );
    assert_eq!(
        drops[0].rider,
        RouteDropRider::ImeRoute,
        "the drop names the route that refused the ask"
    );
}

#[test]
fn m65_the_dead_focus_prices_apart_from_the_never_focused() {
    let observer = Share::new();
    let (mut harness, field) = mount_observer(&observer);

    // The field takes focus, then its node dies: the focused node has
    // gone — the IME event no live field held for a *different* reason
    // than never-focused. The vocabulary splits the species (ADR-0065's
    // first kill rule): the composition names the dead focus, never the
    // never-focused one the old collapse munged it into.
    harness.pipeline_mut().request_focus(field);
    harness.settle();
    harness.pipeline_mut().arena_mut().remove_subtree(field);
    clear_trail(&observer);
    let _ = harness
        .pipeline_mut()
        .dispatch_ime(&ImeEvent::Commit { text: "よ".into() });
    harness.settle();
    let drops = route_drops(&observer);
    assert_eq!(drops.len(), 1, "the drop is a fact the log carries");
    assert_eq!(
        drops[0].species,
        alacer_core::RouteDropSpecies::DeadFocus,
        "the focused node has gone — its own species, never the collapse"
    );
    assert_eq!(
        drops[0].rider,
        RouteDropRider::ImeRoute,
        "the same route, the split silence beside it"
    );

    // The never-focused one keeps its own species: the split is a
    // split — the two silences tellable apart on one log.
    let observer_two = Share::new();
    let (mut fresh, _field_two) = mount_observer(&observer_two);
    clear_trail(&observer_two);
    let _ = fresh
        .pipeline_mut()
        .dispatch_ime(&ImeEvent::Commit { text: "よ".into() });
    fresh.settle();
    let drops = route_drops(&observer_two);
    assert_eq!(drops.len(), 1);
    assert_eq!(
        drops[0].species,
        alacer_core::RouteDropSpecies::NoFocusedField,
        "the never-focused silence stays its own"
    );
}

#[test]
fn m65_the_funnel_drop_names_its_route() {
    let observer = Share::new();
    let (mut harness, _field) = mount_observer(&observer);
    clear_trail(&observer);
    // The ask aimed at an id no tree node carries: the funnel refuses
    // before any widget's door — and the record names the funnel (the
    // dispatch door's own refusal), the aimed id riding verbatim.
    assert!(
        !harness
            .pipeline_mut()
            .dispatch_a11y_action(0xDEAD_BEEF, A11yAction::SetValue("typed".into())),
        "an unknown target consumes nothing"
    );
    harness.settle();
    let drops = route_drops(&observer);
    assert_eq!(drops.len(), 1);
    assert_eq!(drops[0].action, "set_value");
    assert_eq!(
        drops[0].species,
        alacer_core::RouteDropSpecies::UnknownTarget
    );
    assert_eq!(
        drops[0].rider,
        RouteDropRider::Funnel,
        "the funnel's own drop names the funnel"
    );
    assert_eq!(
        drops[0].target,
        Some(0xDEAD_BEEF),
        "the aim the boundary refused, verbatim"
    );
}

#[test]
fn m65_the_surface_rides_the_same_cut() {
    // The seam ships where the vocabulary lives: the route trail, the
    // records' rider, the funnel's and the IME route's drops, the
    // boundary's note sites, the wire event, the client's echo, the
    // decision record itself.
    let trail = read_workspace_file("crates/alacer-core/src/route_drop.rs");
    assert!(
        trail.contains("ADR-0071") && trail.contains("rider"),
        "the route trail module names its decision record and the rider"
    );
    let inspect = read_workspace_file("crates/alacer-core/src/inspect.rs");
    assert!(
        inspect.contains("pub enum RouteDropRider")
            && inspect.contains("pub enum RouteDropSpecies"),
        "the rider lives beside the drop vocabulary"
    );
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    assert!(
        passes.contains("RouteDropRider::Funnel")
            && passes.contains("RouteDropRider::ImeRoute")
            && passes.contains("RouteDropSpecies::DeadFocus"),
        "the funnel and the IME route name themselves, the split riding"
    );
    let platform = read_workspace_file("crates/alacer-platform/src/lib.rs");
    assert!(
        platform.contains("RouteDropRider::Boundary"),
        "the boundary's drop sites stream their rider"
    );
    let wire = read_workspace_file("crates/alacer-inspect/src/wire.rs");
    assert!(
        wire.contains("rider"),
        "the wire speaks the rider beside the species"
    );
    let client = read_workspace_file("crates/alacer-inspect/src/inspector.html");
    assert!(
        client.contains("event.rider"),
        "the embedded client renders the rider echo"
    );

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0071-m65-scope.md");
    assert!(
        adr.contains("M65 scope") && adr.contains("rider"),
        "ADR-0071 documents the rider scope"
    );
}

#[test]
fn m65_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // trail sentence carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M120",
        "a hundred and twenty-five decision records",
        "Since M65 the drops name the route that dropped them",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0071",
        "a hundred and twenty-five decision records",
        "the route drop names its rider and splits its silence",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary: the rider
    // beside the species, the split silence's own tag.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`rider`")
            && protocol.contains("dead_focus")
            && protocol.contains("ime_route"),
        "the protocol document carries the rider and the split"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog
            .contains("**M65 — the route drop names its rider and splits its silence (ADR-0071)**"),
        "the CHANGELOG carries the M65 entry"
    );

    // The catalog: the demonstration walks the rider beat.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("names its rider"),
        "docs/examples.md carries the rider demonstration"
    );

    // The sibling suites' pins ride the same cut (one sweep, no drift).
    let m57 = read_workspace_file("crates/alacer-testing/tests/m57_exit_criteria.rs");
    assert!(
        m57.contains("M0 through M120"),
        "the m57 suite's pins moved with the documents"
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
