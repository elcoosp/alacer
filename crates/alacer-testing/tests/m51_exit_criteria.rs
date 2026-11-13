//! M51 exit-criteria integration tests (§13.3, ADR-0057): the
//! content-side clamp joins the scroll log. ADR-0056's third kill rule
//! cashes: a consumer needs the content-side clamp named —
//! `set_content_height` silently re-clamping the offset when the
//! content shrank under it — so the content's door grows its own fact
//! ("the clamp is the content's story, not the ask's; today it rides
//! outside the log"). The door answered in silence: a resize that
//! moved the offset left no word on any log, and a non-finite height
//! rode the clamp arithmetic and spread `NaN` into the offset *and*
//! the stored span alike — the corruption class the M50 trail exposed
//! at the wheel door, still open at the content's own door (the
//! micro-debt the record's design priced, cashed with it).
//!
//! Every test runs the real pipeline (A4) with a real list driven by
//! the app's own handle — the exact door a consumer's resize rides —
//! and reads the facts back off the causal observer.

use alacer_core::RangeBound;
use alacer_core::inspect::{InspectEvent, InspectObserver, ScrollAction, ScrollOutcome};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, List};

/// The scroll-trail collector the tests install as the pipeline's
/// observer: every event streams into a bounded ring (the m-suite's
/// helper shape — the content-clamp facts are the assertion source).
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

/// Empties the shared ring between beats: each assertion reads the
/// facts the resize under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// A list mounted through the real pipeline with a content height the
/// viewport can scroll: 500 px of content in a 100 px viewport — a
/// 400 px span, the walls at 0 and 400 (the m50 suite's shape). The
/// initial resize queues nothing: the offset the fresh state holds is
/// zero, and growth never moves an offset that is already at the top.
fn mount_scrollable_list(observer: &Share) -> (TestHarness, alacer_widgets::ListHandle) {
    let mut harness = TestHarness::new(400.0, 200.0);
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().boxed());
    let list = List::new(100.0);
    let handle = list.handle();
    let _ = harness.add_child(root, list.boxed());
    handle.set_content_height(500.0);
    harness.settle();
    (harness, handle)
}

#[test]
fn m51_the_content_clamp_names_its_wall() {
    let observer = Share::new();
    let (mut harness, handle) = mount_scrollable_list(&observer);

    // The clamp's stage: the offset rides to the high wall — the app's
    // own ask, the ask-side fact the M50 trail already prices.
    handle.scroll_to(400.0);
    harness.settle();
    assert_eq!(handle.offset(), 400.0);
    clear_trail(&observer);

    // The content shrinks under the offset: 300 px of content in a
    // 100 px viewport — the span falls to 200, the clamp lands the
    // offset at the new high wall. The door streams its own fact (the
    // kill rule's words): the content's story, no ask beside it (none
    // existed), the wall the clamp pinned named — and the stamp's
    // honest absence, the app door riding no route.
    handle.set_content_height(300.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "one resize, one fact");
    assert_eq!(records[0].action, ScrollAction::ContentClamp);
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(
        records[0].offset, 200.0,
        "the landing the content clamp produced"
    );
    assert_eq!(
        records[0].requested, None,
        "the content's story has no ask — the honest absence"
    );
    assert_eq!(
        records[0].pinned,
        Some(RangeBound::Max),
        "the wall the content clamp pinned is a named fact"
    );
    assert_eq!(
        records[0].target, None,
        "the app's handle rides no route — the stamp's honest absence"
    );
    assert_eq!(handle.offset(), 200.0, "the state carries the landing");
}

#[test]
fn m51_the_resize_the_offset_never_felt_queues_nothing() {
    let observer = Share::new();
    let (mut harness, handle) = mount_scrollable_list(&observer);
    handle.scroll_to(100.0);
    harness.settle();
    assert_eq!(handle.offset(), 100.0);
    clear_trail(&observer);

    // The content shrinks, but the span still holds the offset: 300 px
    // of content in a 100 px viewport — a 200 px span above the 100 px
    // offset. The clamp moved nothing, so the door queues nothing —
    // the M20 idle-frame property at the content's own door.
    handle.set_content_height(300.0);
    harness.settle();
    assert_eq!(handle.offset(), 100.0, "the offset stays where it was");
    assert!(
        scroll_records(&observer).is_empty(),
        "a resize the offset never felt is not a fact — nothing queued"
    );

    // The content grows: the span widens, the offset stays — again
    // nothing queues (growth never moves an offset).
    handle.set_content_height(600.0);
    harness.settle();
    assert_eq!(handle.offset(), 100.0);
    assert!(
        scroll_records(&observer).is_empty(),
        "growth never moves the offset — nothing queued"
    );
}

#[test]
fn m51_the_non_finite_height_refuses_before_it_corrupts() {
    let observer = Share::new();
    let (mut harness, handle) = mount_scrollable_list(&observer);
    handle.scroll_to(100.0);
    harness.settle();
    assert_eq!(handle.offset(), 100.0);
    clear_trail(&observer);

    // The debt the record's design priced (the M50 ADR's own words at
    // the wheel door, still open here): a non-finite height rode the
    // clamp arithmetic and spread `NaN` into the offset *and* the
    // stored span alike — the list scrolled to nothing, permanently,
    // with no error anywhere. The door refuses before the arithmetic:
    // the state keeps the height and offset it held, the refusal a
    // named fact on the log.
    handle.set_content_height(f32::NAN);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the refused resize streams its fact");
    assert_eq!(records[0].action, ScrollAction::ContentClamp);
    assert_eq!(
        records[0].outcome,
        ScrollOutcome::Refused(alacer_core::ScrollRefusal::NonFiniteContent)
    );
    assert_eq!(
        records[0].offset, 100.0,
        "the state keeps the offset the resize found"
    );
    assert_eq!(
        records[0].requested, None,
        "a height is not an ask — the honest absence"
    );
    assert_eq!(
        records[0].pinned, None,
        "a write that moved nothing pinned nothing"
    );
    assert_eq!(handle.offset(), 100.0, "no NaN reached the offset");

    // The stored span kept its shape too: the next good ask answers by
    // the *old* arithmetic — the 500 px content's 400 px wall — like
    // nothing happened (the M50 rule at the content's door).
    handle.scroll_to(450.0);
    harness.settle();
    assert_eq!(
        handle.offset(),
        400.0,
        "the old content height held — the refusal never corrupted the span"
    );

    // Every non-finite shape refuses the same way: the infinities ride
    // the same wall.
    clear_trail(&observer);
    handle.set_content_height(f32::INFINITY);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1, "the infinite height refuses too");
    assert!(matches!(
        records[0].outcome,
        ScrollOutcome::Refused(alacer_core::ScrollRefusal::NonFiniteContent)
    ));
    assert_eq!(handle.offset(), 400.0, "the offset still holds");
}

#[test]
fn m51_the_ask_tier_and_the_content_tier_agree_on_the_walls() {
    let observer = Share::new();
    let (mut harness, handle) = mount_scrollable_list(&observer);

    // The same wall spoken identically on both stories: the ask tier's
    // wheel clamps at the 400 px max (the M50 fact), the content tier's
    // clamp pins the same wall when the content shrinks to it — one
    // `RangeBound` vocabulary, never a second species (ADR-0054's rule
    // the scroll tier rides).
    handle.scroll_to(200.0);
    harness.settle();
    handle.set_content_height(250.0);
    harness.settle();
    assert_eq!(handle.offset(), 150.0, "the span fell to 150");
    clear_trail(&observer);
    handle.set_content_height(140.0);
    harness.settle();
    let records = scroll_records(&observer);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].outcome, ScrollOutcome::Delivered);
    assert_eq!(records[0].offset, 40.0, "140 content - 100 viewport");
    assert_eq!(
        records[0].pinned,
        Some(RangeBound::Max),
        "the content clamp names the same wall the ask tier names"
    );
    assert!(
        records.iter().all(|record| record.target.is_none()),
        "the content's door rides no route, whatever it answers"
    );
}

#[test]
fn m51_the_surface_rides_the_same_cut() {
    // The seam ships where the trail lives: the record vocabulary in
    // core, the door's fact in the widget, the wire carrying the new
    // tags.
    let inspect = read_workspace_file("crates/alacer-core/src/inspect.rs");
    for fragment in [
        "ContentClamp",
        "content_clamp",
        "NonFiniteContent",
        "non_finite_content",
    ] {
        assert!(
            inspect.contains(fragment),
            "the record vocabulary ships the content clamp: {fragment}"
        );
    }
    let list = read_workspace_file("crates/alacer-widgets/src/list.rs");
    for fragment in [
        "ScrollAction::ContentClamp",
        "ScrollRefusal::NonFiniteContent",
    ] {
        assert!(
            list.contains(fragment),
            "the content door names its facts: {fragment}"
        );
    }
    let wire = read_workspace_file("crates/alacer-inspect/src/wire.rs");
    for fragment in ["content_clamp", "non_finite_content"] {
        assert!(
            wire.contains(fragment),
            "the wire carries the content clamp's tags: {fragment}"
        );
    }

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0057-m51-scope.md");
    assert!(
        adr.contains("M51 scope") && adr.contains("content"),
        "ADR-0057 documents the content-clamp scope"
    );
}

#[test]
fn m51_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // features bullet carries the fact, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M54",
        "sixty-one decision records",
        "Since M51 the content clamp answers on the log too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `content_clamp` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`content_clamp`](../crates/alacer/examples/content_clamp.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0057",
        "61 decision records",
        "the content-side clamp joins the causal log",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The protocol document carries the wire vocabulary.
    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`content_clamp`") && protocol.contains("`non_finite_content`"),
        "the protocol document carries the content clamp's tags"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M51 — the content-side clamp joins the causal log (ADR-0057)**"),
        "the CHANGELOG carries the M51 entry"
    );

    // Every pin the M-count move touched moved together — the suites'
    // document pins ride the same cut, the chain cannot drift.
    for suite in ["m7", "m19", "m24", "m35", "m43", "m49", "m50"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M54"),
            "the {suite} suite's README pin moved with the documents"
        );
    }
    for suite in ["m19", "m24", "m35", "m43", "m49", "m50"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("61 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
    for suite in ["m24", "m35", "m43", "m49", "m50"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("sixty-one decision records"),
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
