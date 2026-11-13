//! M50 exit-criteria integration tests (§13.3, ADR-0056): the scroll's
//! story joins the causal log. ADR-0055's second kill rule cashes: a
//! consumer needs the wall named on a tier whose funnel knows no
//! range — the list's scroll clamp — so the scroll tier grows its own
//! record beside its species, never the value record's (the wall is
//! the asker's story — ADR-0054's rule at the next tier). The four
//! scroll doors — the wheel, the AT's unit scrolls and value steps,
//! the app's programmatic scroll — answered in silence: a wheel past
//! the content's end clamped without a word, a wheel over a list whose
//! content fits the viewport moved nothing unexplained, and a
//! non-finite wheel delta corrupted the offset (the `NaN` the clamp
//! spread into the state). The record prices every answer: the landed
//! offset beside the ask ([`alacer_core::inspect::ScrollRecord`]), the
//! wall the clamp pinned ([`alacer_core::inspect::RangeBound`]), the
//! named refusals, and the route's stamp.
//!
//! Every test runs the real pipeline (A4) with a real list driven by
//! the real pointer route, the real AT action route, and the app's own
//! handle — the exact paths a status line, an inspector client, or a
//! test reads.

use alacer_core::arena::NodeId;
use alacer_core::geom::{Point, Vector};
use alacer_core::inspect::{InspectEvent, InspectObserver, ScrollAction, ScrollOutcome};
use alacer_core::widget::{A11yAction, EventResponse, Role, ScrollUnit};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The scroll-trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the scroll facts and the tier-fact counts are the
/// assertion source).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed scroll facts, oldest first.
    fn scrolls(&self) -> Vec<alacer_core::ScrollRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::Scroll(record) => Some(*record),
                _ => None,
            })
            .collect()
    }

    /// How many value-tier facts streamed — the scroll tier's proof
    /// that a scroll ask is a scroll fact, never a value fact.
    fn value_count(&self) -> usize {
        self.events
            .iter()
            .filter(|event| matches!(event, InspectEvent::Value(_)))
            .count()
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

/// The streamed scroll facts, oldest first.
fn scroll_records(observer: &Share) -> Vec<alacer_core::ScrollRecord> {
    observer.0.borrow().scrolls()
}

/// How many value facts streamed (the tier-boundary proof).
fn value_count(observer: &Share) -> usize {
    observer.0.borrow().value_count()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the ask under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// A list mounted through the real pipeline with a content height the
/// viewport can scroll: 500 px of content in a 100 px viewport — a
/// 400 px span, the walls at 0 and 400 (the m10 suite's shape). The
/// node rides back for the wheel beats: the scroll container's own
/// bounds translate with the offset (compose moves the viewport), so a
/// beat aims at the bounds' current center, wherever the scroll put
/// them.
fn mount_scrollable_list(observer: &Share) -> (TestHarness, alacer_widgets::ListHandle, NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let list = List::new(100.0);
    let handle = list.handle();
    let node = harness.add_child(root, list.boxed());
    handle.set_content_height(500.0);
    harness.settle();
    (harness, handle, node)
}

/// A list whose content fits its viewport: the span is zero, the
/// range has no wall to name — the honest refusal's stage.
fn mount_fitting_list(observer: &Share) -> (TestHarness, alacer_widgets::ListHandle, NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let list = List::new(100.0);
    let handle = list.handle();
    let node = harness.add_child(root, list.boxed());
    handle.set_content_height(80.0);
    harness.settle();
    (harness, handle, node)
}

/// One wheel routed through the real pointer route, aimed at the
/// list's *current* bounds (the scroll container's own bounds translate
/// with the offset — compose moves the viewport — so the beat reads
/// them per ask): the response the route gave.
fn wheel(harness: &mut TestHarness, node: NodeId, delta: f32) -> EventResponse {
    let bounds = harness.bounds(node);
    let position = Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    );
    harness.queue_pointer(alacer_core::events::PointerEvent::Wheel {
        delta: Vector::new(0.0, delta),
        position,
    });
    harness.settle();
    EventResponse::Handled
}

#[test]
fn m50_the_wheel_names_its_landing() {
    let observer = Share::new();
    let (mut harness, handle, node) = mount_scrollable_list(&observer);
    let id = harness.query(Role::ScrollArea).named("list").first().id;

    // The mid-range wheel: the record carries the landed offset beside
    // the ask (the M44 shape at the scroll tier), the wall absent — a
    // landing the clamp never touched pins nothing.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -60.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "one wheel, one fact");
    assert_eq!(records[0].action, ScrollAction::Wheel);
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 60.0, "the landing the wheel produced");
    assert_eq!(
        records[0].requested,
        Some(60.0),
        "the ask rides beside the landing (the M44 rule at the scroll tier)"
    );
    assert_eq!(records[0].pinned, None, "a mid-range landing pins no wall");
    assert_eq!(
        records[0].target,
        Some(id),
        "the wheel route stamps the hit it resolved (the M46 discipline)"
    );
    assert_eq!(handle.offset(), 60.0);

    // A second wheel in the same direction coalesces upstream (§10.2):
    // the accumulated run routes as one ask — the delivery the ask
    // produced, the trail reading one fact for the merged run.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -40.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the merged run routes as one ask");
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 100.0, "60 + 40: the run accumulated");
    assert_eq!(records[0].pinned, None, "100 is mid-range — no wall");
}

#[test]
fn m50_the_past_ask_names_its_wall() {
    let observer = Share::new();
    let (mut harness, _handle, node) = mount_scrollable_list(&observer);

    // The ask past the high wall: one wheel of 10 000 px from the top
    // asks 10 000, the clamp lands 400 — the record names the bound
    // the clamp pinned the landing at, beside the ask it turned back.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -10_000.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "one ask, one fact");
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 400.0, "the high wall held");
    assert_eq!(
        records[0].requested,
        Some(10_000.0),
        "the past ask rides beside the wall it was turned back by"
    );
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Max),
        "the clamp pinned the landing at the max — the wall is a named fact"
    );

    // The ask past the low wall: the other edge, the other species.
    // The direction flip starts a fresh run (§10.2: the previous run
    // drained with the last frame), so one ask routes — the up-ask
    // landing at the low wall, the ask beside the landing.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, 10_000.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the flipped direction routes one ask");
    assert_eq!(records[0].offset, 0.0, "the low wall held");
    assert_eq!(
        records[0].requested,
        Some(-9_600.0),
        "the past ask rides beside the wall it was turned back by"
    );
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Min),
        "the clamp pinned the landing at the min"
    );

    // The wall that held a no-op against a past ask (the M49 sentence
    // at the scroll tier): already at the top, one more wheel up asks
    // past the min and lands where the offset already is — delivered,
    // pinned, the trail naming the wall that held it.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, 40.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the held ask answers");
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 0.0, "the offset never moved");
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Min),
        "the wall held a no-op against a past ask — the pin names it"
    );
}

#[test]
fn m50_the_ask_that_wanted_the_wall_exactly_pins_nothing() {
    let observer = Share::new();
    let (mut harness, handle, _node) = mount_scrollable_list(&observer);

    // The programmatic door: the app's own scroll, no route behind it
    // — the stamp rides its honest absence (the M43 rule: the stamp
    // fills only the identity the widget cannot know).
    clear_trail(&observer);
    handle.scroll_to(400.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the app's scroll streams one fact");
    assert_eq!(records[0].action, ScrollAction::ScrollTo);
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 400.0, "the ask wanted the wall exactly");
    assert_eq!(
        records[0].pinned, None,
        "an ask the wall never moved pins nothing — the landing *is* the ask"
    );
    assert_eq!(
        records[0].target, None,
        "the app's door rides no route — the absence honest"
    );

    // The ask past it: the clamp moves it — the pin rides.
    clear_trail(&observer);
    handle.scroll_to(450.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records[0].offset, 400.0, "the wall held");
    assert_eq!(
        records[0].pinned,
        Some(alacer_core::RangeBound::Max),
        "the clamp pinned the past ask at the max"
    );
}

#[test]
fn m50_the_at_scroll_rides_attributed() {
    let observer = Share::new();
    let (mut harness, handle, _node) = mount_scrollable_list(&observer);
    let id = harness.query(Role::ScrollArea).named("list").first().id;

    // The AT's scroll command through the real action route: the unit
    // scrolls travel by their unit (the M10 vocabulary), the record
    // carries the landed offset beside the ask and the action route's
    // stamp — the ride attributed like every other fact on the log.
    clear_trail(&observer);
    assert!(harness.act(
        Role::ScrollArea,
        "list",
        A11yAction::ScrollDown(ScrollUnit::Item)
    ));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "one AT ask, one fact");
    assert_eq!(records[0].action, ScrollAction::ScrollDown);
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 40.0, "one item = one line = 40 px");
    assert_eq!(
        records[0].requested,
        Some(40.0),
        "the ask beside the landing"
    );
    assert_eq!(
        records[0].target,
        Some(id),
        "the action route stamps the id the action arrived under"
    );

    // The value steps the scroll area answers (M10): the scrollbar's
    // value *is* the offset — the step is a scroll fact on the scroll
    // log, never a value fact on the value log (the tier boundary the
    // ADR-0046 rule draws: a question outside the tier is not a tier
    // fact).
    clear_trail(&observer);
    assert!(harness.act(Role::ScrollArea, "list", A11yAction::Increment));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the step is a scroll fact");
    assert_eq!(records[0].action, ScrollAction::Increment);
    assert_eq!(records[0].offset, 140.0, "one step = one page = 100 px");
    assert_eq!(records[0].target, Some(id), "the stamp rides");
    assert_eq!(
        value_count(&observer),
        0,
        "a scroll area answers in the scroll tier's vocabulary — the value log stays silent"
    );
    assert_eq!(handle.offset(), 140.0);
}

#[test]
fn m50_the_fitting_list_refuses_honestly() {
    let observer = Share::new();
    let (mut harness, handle, node) = mount_fitting_list(&observer);
    let id = harness.query(Role::ScrollArea).named("list").first().id;

    // A wheel over a list whose content fits the viewport: the range
    // has no span, no scroll exists to deliver — the record names the
    // refusal, the offset never moves, and the door's routing contract
    // is untouched (the wheel stays consumed — the M10 re-announce
    // rule is the dispatch contract's business, the record is the
    // tier's answer).
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -60.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the refused ask streams its fact");
    assert_eq!(records[0].action, ScrollAction::Wheel);
    assert_eq!(
        records[0].outcome,
        ScrollOutcome::Refused(alacer_core::ScrollRefusal::NoScrollableContent),
        "the range has no span — the refusal is the tier's honest answer"
    );
    assert_eq!(records[0].offset, 0.0, "the offset never moved");
    assert_eq!(
        records[0].requested,
        Some(60.0),
        "the ask rode — the refusal is about the range, not the ask"
    );
    assert_eq!(
        records[0].pinned, None,
        "a write that moved nothing pinned nothing"
    );
    assert_eq!(records[0].target, Some(id), "the wheel route's stamp rides");
    assert_eq!(handle.offset(), 0.0);

    // The AT's ask refuses the same way.
    clear_trail(&observer);
    assert!(harness.act(
        Role::ScrollArea,
        "list",
        A11yAction::ScrollDown(ScrollUnit::Page)
    ));
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the AT ask refused on the log");
    assert_eq!(
        records[0].outcome,
        ScrollOutcome::Refused(alacer_core::ScrollRefusal::NoScrollableContent)
    );
    assert_eq!(handle.offset(), 0.0, "the refusal moved nothing");
}

#[test]
fn m50_the_non_finite_ask_refuses_before_it_corrupts() {
    let observer = Share::new();
    let (mut harness, handle, node) = mount_scrollable_list(&observer);

    // The debt the trail exposed: a non-finite wheel delta rode the
    // clamp arithmetic and spread `NaN` into the scroll state — the
    // list scrolled to nothing, permanently. The door now refuses the
    // ask before the arithmetic: the refusal is named, the state keeps
    // the offset the app had, and the next good wheel still works.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, f32::NAN);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the non-finite ask streams its refusal");
    assert_eq!(
        records[0].outcome,
        ScrollOutcome::Refused(alacer_core::ScrollRefusal::NonFiniteAsk),
        "the funnel carries only finite asks — the wall is its own fact"
    );
    assert_eq!(
        records[0].requested, None,
        "the wire cannot carry an infinite ask honestly (the value tier's rule)"
    );
    assert_eq!(
        records[0].offset, 0.0,
        "the state keeps the offset the ask found"
    );
    assert_eq!(handle.offset(), 0.0, "the corruption is gone");

    // The next good ask answers like nothing happened.
    clear_trail(&observer);
    let _ = wheel(&mut harness, node, -60.0);
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 60.0, "the list still scrolls");
    assert_eq!(handle.offset(), 60.0);
}

#[test]
fn m50_the_outside_tier_is_not_a_scroll_fact() {
    let observer = Share::new();
    let (mut harness, handle, _node) = mount_scrollable_list(&observer);

    // A press is not a scroll ask (the M11 rule's own words at the
    // scroll tier): the widget refuses it and the log records no
    // scroll fact — the question outside the tier is not a tier fact
    // (the ADR-0046 rule the scroll tier rides).
    clear_trail(&observer);
    assert!(
        !harness.act(Role::ScrollArea, "list", A11yAction::DefaultPress),
        "the scroll area refuses the press"
    );
    harness.settle();
    assert!(
        scroll_records(&observer).is_empty(),
        "a refused press is a silence on the scroll log, not a refusal fact"
    );
    assert_eq!(handle.offset(), 0.0, "the press never scrolled");
}

#[test]
fn m50_the_doors_answer_the_same_without_devtools() {
    // The A7 idle contract, the trail's own rule: while no observer is
    // installed the doors note nowhere and answer the same — the
    // disabled trail costs one branch per ask (the trail mechanics
    // themselves are the scroll module's own tests; this is the
    // pipeline-level proof that the answers do not move).
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let list = List::new(100.0);
    let handle = list.handle();
    let node = harness.add_child(root, list.boxed());
    handle.set_content_height(500.0);
    harness.settle();

    let _ = wheel(&mut harness, node, -60.0);
    assert_eq!(handle.offset(), 60.0, "the wheel answers");
    assert!(
        harness.act(Role::ScrollArea, "list", A11yAction::Increment),
        "the AT ask answers"
    );
    assert_eq!(handle.offset(), 160.0, "one step = one page");
    handle.scroll_to(400.0);
    harness.settle();
    assert_eq!(handle.offset(), 400.0, "the app's door answers");
    handle.scroll_to(f32::NAN);
    assert_eq!(
        handle.offset(),
        400.0,
        "the non-finite ask refuses before the arithmetic, observer or not"
    );
    handle.scroll_to(0.0);
    harness.settle();
    assert_eq!(handle.offset(), 0.0);
}

#[test]
fn m50_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // features bullet carries the trail, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M52",
        "fifty-nine decision records",
        "Since M50 the scroll answers on the log too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `scroll_trail` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`scroll_trail`](../crates/alacer/examples/scroll_trail.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0056",
        "59 decision records",
        "the scroll's story joins the causal log",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0056-m50-scope.md");
    assert!(
        adr.contains("M50 scope") && adr.contains("scroll"),
        "ADR-0056 documents the scroll-trail scope"
    );

    // The protocol document carries the wire vocabulary.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`scroll`"),
        "the protocol document carries the scroll event"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M50 — the scroll's story joins the causal log (ADR-0056)**"),
        "the CHANGELOG carries the M50 entry"
    );

    // Every pin the M-count move touched moved together — the suites'
    // document pins ride the same cut, the chain cannot drift. The
    // suites phrase their pins differently (the m7 suite pins the
    // README count alone; the m19 suite pins the register count beside
    // it; the m24/m35/m43/m49 suites carry the full README pair), so
    // the sampled assertions ride each suite's own phrasing.
    for suite in ["m7", "m19", "m24", "m35", "m43", "m49"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M52"),
            "the {suite} suite's README pin moved with the documents"
        );
    }
    for suite in ["m19", "m24", "m35", "m43", "m49"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("59 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
    for suite in ["m24", "m35", "m43", "m49"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("fifty-nine decision records"),
            "the {suite} suite's README status pin moved with the documents"
        );
    }
}

#[test]
fn m50_the_surface_rides_the_same_cut() {
    // The seam ships where the trail lives: the record vocabulary, the
    // thread trail, the doors' notes, the funnel's drain and stamps.
    let scroll = read_workspace_file("crates/alacer-core/src/scroll.rs");
    for fragment in [
        "pub fn note_scroll(",
        "pub(crate) fn take_scroll_trail()",
        "SCROLL_TRAIL_CAP: usize = 256",
    ] {
        assert!(
            scroll.contains(fragment),
            "the scroll trail ships its own seam: {fragment}"
        );
    }

    // The pipeline drains the trail at the tick's start and holds the
    // wheel route's stamp — the M46 discipline at the scroll tier.
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    for fragment in [
        "fn notify_scroll(",
        "crate::scroll::take_scroll_trail()",
        "crate::scroll::ScrollTargetGuard::for_target",
    ] {
        assert!(
            passes.contains(fragment),
            "the pipeline carries the scroll trail: {fragment}"
        );
    }

    // The doors note where the answer resolves — the wheel, the AT
    // actions, the programmatic handle.
    let list = read_workspace_file("crates/alacer-widgets/src/list.rs");
    for fragment in ["note_scroll(", "fn pinned_offset_bound"] {
        assert!(
            list.contains(fragment),
            "the list's doors note the scroll facts: {fragment}"
        );
    }

    // The wire speaks the vocabulary (the additive rule at version 1).
    let wire = read_workspace_file("crates/alacer-inspect/src/wire.rs");
    assert!(
        wire.contains("Scroll {"),
        "the wire carries the scroll event"
    );

    // The consumer demonstration names the decision it demonstrates,
    // drives the seam it demonstrates, and rides the workspace's
    // per-seam tracing idiom.
    let example = read_workspace_file("crates/alacer/examples/scroll_trail.rs");
    assert!(
        example.contains("ADR-0056")
            && example.contains("ScrollRecord")
            && example.contains("alacer::scroll_trail"),
        "examples/scroll_trail.rs demonstrates the scroll trail"
    );
}

/// Reads a workspace-relative file (the m-suite's helper shape).
fn read_workspace_file(path: &str) -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let workspace = std::path::Path::new(manifest)
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| panic!("the harness sits two levels deep"));
    std::fs::read_to_string(workspace.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"))
}
