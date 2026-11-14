//! M63 exit-criteria integration tests (§13.3, ADR-0069): the trail's
//! provided reader joins the seam. A consumer prices the *collector* —
//! every m-suite gate and every trail example hand-rolls the same
//! shape (`Rc<RefCell<VecDeque>>` behind a one-method observer, the
//! m-suite's "helper shape" repeated 33 files over), and the harness
//! could not install a reader without each test writing its own. The
//! seam grows the reader the way the clipboard seam grew
//! `alacer_core::clipboard::MemoryClipboard`: a provided implementation
//! beside the trait — `alacer_core::TrailLog`, the bounded in-process
//! ring the pipeline forwards into, with the drain as the reader's own
//! window and the eviction named (a dropped count the ring increments,
//! never a silent loss). The harness grows the one-call door:
//! `TestHarness::trail` installs the reader and hands back the reading
//! handle — one line where every suite wrote eleven.
//!
//! The A7 idle contract holds in both directions: the trail-off
//! pipeline streams nothing (the observer's branch is the only cost),
//! and a trail nobody drains is bounded — the ring evicts its oldest
//! and counts the eviction, never a frame-budget liability. The facts
//! the reader prices are the routes' own: the M58 key-edit record
//! (ADR-0064) with the M62 outcome beside it, the M18 announcement's
//! outcome (ADR-0024) — streamed oldest-first, the causal order the
//! trail examples assert by hand today.

use alacer_core::announce::{Politeness, announce};
use alacer_core::inspect::{
    AnnounceOutcome, AnnounceRecord, InputKind, InputRecord, InspectEvent, InspectObserver,
};
use alacer_core::{KeyEditAction, KeyEditDriver, KeyEditOutcome, NodeId, TrailLog};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

/// The pipeline's own event, built at a known target — the ring tests'
/// exact arithmetic input (one observation per call, no route noise).
fn a_pointer_down(node: NodeId) -> InspectEvent {
    InspectEvent::Input(InputRecord {
        kind: InputKind::PointerDown,
        target: Some(node),
    })
}

/// The typing door's mount: an input through the real pipeline, the
/// harness's trail door installed, focused and settled.
fn mount_input_with_trail() -> (TestHarness, TrailLog, NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    let trail = harness.trail();
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("field").boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    (harness, trail, node)
}

/// The streamed key-edit facts, oldest first.
fn key_edits(events: &[InspectEvent]) -> Vec<alacer_core::KeyEditRecord> {
    events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::KeyEdit(record) => Some(*record),
            _ => None,
        })
        .collect()
}

/// The streamed announcement facts, oldest first.
fn announcements(events: &[InspectEvent]) -> Vec<AnnounceRecord> {
    events
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Announce(record) => Some(*record),
            _ => None,
        })
        .collect()
}

#[test]
fn the_provided_reader_prices_the_typing_route() {
    let (mut harness, trail, _node) = mount_input_with_trail();

    // One batch: the flood rule folds the run — one record pricing the
    // run's shape (the M58 record, the M20 idle-frame word beside it).
    harness.type_text("ab");
    harness.settle();

    let edits = key_edits(&trail.drain());
    assert_eq!(
        edits.len(),
        1,
        "a coalesced typed run lands as one record (the flood rule)"
    );
    let run = &edits[0];
    assert!(matches!(run.action, KeyEditAction::Insert));
    assert!(matches!(run.driver, KeyEditDriver::Keyboard));
    assert_eq!(run.outcome, KeyEditOutcome::Landed, "the door landed it");
    assert_eq!(run.inserted, 2, "the shape prices the run's two units");
    assert!(run.coalesced, "the landed run rides the flood rule");

    // A second batch: the next run lands beside the first, oldest
    // first — the causal order the drain hands back.
    harness.type_text("c");
    harness.settle();
    let edits = key_edits(&trail.drain());
    assert_eq!(edits.len(), 1, "the second run is the window's own");
    assert_eq!(edits[0].inserted, 1, "the single character prices itself");
}

#[test]
fn the_announce_rides_the_provided_reader() {
    let (mut harness, trail, _node) = mount_input_with_trail();

    let accepted = announce("Saved", Politeness::Polite);
    assert!(accepted, "the ambient route accepted the speech act");
    harness.settle();

    let announced = announcements(&trail.drain());
    assert_eq!(announced.len(), 1, "the speech act streams one fact");
    assert_eq!(
        announced[0].outcome,
        AnnounceOutcome::Delivered,
        "the region carries the announcement"
    );
}

#[test]
fn the_ring_is_bounded_and_the_eviction_is_named() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let node = harness.mount(Column::new().boxed());
    let mut trail = TrailLog::with_cap(4);
    harness.pipeline_mut().set_observer(Box::new(trail.clone()));
    harness.settle();

    // The mount frame's own facts rode the provided reader — and the
    // ring held them at the cap (the route's window, bounded).
    assert!(
        trail.len() <= trail.cap(),
        "the route's facts bounded at the cap"
    );
    let _ = trail.drain();

    // The ring's exact arithmetic, on an emptied window: seven
    // observations against a cap of four — three evicted, named.
    let baseline = trail.dropped();
    for _ in 0..7 {
        trail.observe(a_pointer_down(node));
    }
    assert_eq!(trail.len(), 4, "the ring holds its cap, never more");
    assert_eq!(
        trail.dropped(),
        baseline + 3,
        "the evictions are counted, named"
    );
    let window = trail.drain();
    assert_eq!(window.len(), 4, "the drain hands over the held window");
    assert_eq!(
        trail.dropped(),
        baseline + 3,
        "the eviction count outlives a drain"
    );
    for (i, event) in window.iter().enumerate() {
        let InspectEvent::Input(record) = event else {
            panic!("the route's own fact, not noise: index {i}");
        };
        assert!(
            matches!(record.kind, InputKind::PointerDown),
            "the fourth through seventh observations survived the ring"
        );
    }
}

#[test]
fn the_drain_is_the_readers_window() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let node = harness.mount(Column::new().boxed());
    let mut trail = TrailLog::new();
    harness.pipeline_mut().set_observer(Box::new(trail.clone()));
    harness.settle();
    let _ = trail.drain();

    trail.observe(a_pointer_down(node));
    trail.observe(a_pointer_down(node));

    assert_eq!(trail.len(), 2, "the window holds what nobody drained");
    assert!(!trail.is_empty(), "len and is_empty agree");
    let looked = trail.look();
    assert_eq!(looked.len(), 2, "the look leaves the window intact");
    assert_eq!(trail.len(), 2, "a look is not a drain");
    let tip = trail.last().expect("the window's tip");
    let InspectEvent::Input(record) = tip else {
        panic!("the tip is the route's own fact");
    };
    assert!(
        matches!(record.kind, InputKind::PointerDown),
        "last names the newest observation"
    );

    let drained = trail.drain();
    assert_eq!(drained.len(), 2, "the drain hands over the window");
    assert!(trail.is_empty(), "the window emptied");
    assert!(trail.last().is_none(), "nothing rides behind the drain");
}

#[test]
fn the_trail_is_bounded_when_nobody_drains() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let trail = harness.trail_with_cap(8);
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("field").boxed());
    harness.settle();
    harness.pipeline_mut().request_focus(node);
    harness.settle();

    for _ in 0..64 {
        harness.type_text("x");
        harness.settle();
        assert!(
            trail.len() <= trail.cap(),
            "the ring holds its cap — the trail is a bounded window, never a leak"
        );
    }
    assert_eq!(trail.len(), trail.cap(), "the undrained trail saturated");
    assert!(
        trail.dropped() > 0,
        "the undrained trail evicted, and the eviction is named"
    );
}
