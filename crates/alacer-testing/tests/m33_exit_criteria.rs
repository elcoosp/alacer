//! M33 exit-criteria integration tests (§13.3, ADR-0039): the
//! announcement speaks its trail. The causal log answers "why did this
//! render", "why did focus change — or why didn't it", and "what did
//! this field's journal do" — but the app's voice had no route: a
//! speech act landed on the reserved live region and left no trace a
//! trail reader could route on. M33 cashes it: every announcement act
//! streams one [`AnnounceRecord`] (the politeness it asked for, the
//! outcome it got) the moment the pipeline answers it — the direct
//! door's acceptance and both refusals at the door, the push-side
//! empty refusal on the thread trail, and the tick's start drain
//! answering the queued batch with the last-write-wins rule visible as
//! [`AnnounceOutcome::Superseded`] records beside the delivered act.
//! The walk behind the region is untouched: the M18 contract answers
//! exactly as before, and with no observer installed the trail stays
//! silent — one branch per act, the A7 idle contract unchanged.
//!
//! Every test runs the real pipeline (A4) with real speech acts — the
//! exact route a status line, an inspector client, or a test reads.

use accesskit::Live;
use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::announce::{ANNOUNCEMENT_NODE_HASH, Politeness};
use alacer_core::inspect::{AnnounceOutcome, AnnounceRecord, InspectEvent, InspectObserver};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label};

/// The announcement-trail collector the tests install as the pipeline's
/// observer: the speech acts stream into a bounded ring (the m31
/// helper's shape — here the announce record is the assertion source).
struct AnnounceObserver {
    records: std::collections::VecDeque<AnnounceRecord>,
}

impl AnnounceObserver {
    fn new() -> Self {
        Self {
            records: std::collections::VecDeque::with_capacity(64),
        }
    }
}

impl InspectObserver for AnnounceObserver {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::Announce(record) = event {
            if self.records.len() >= 64 {
                self.records.pop_front();
            }
            self.records.push_back(record);
        }
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m30/m31 suites' shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<AnnounceObserver>>);

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The ring's read half: every streamed fact, oldest first.
fn records(observer: &Share) -> Vec<AnnounceRecord> {
    observer.0.borrow().records.iter().copied().collect()
}

/// Empties the shared ring between beats: each assertion reads the facts
/// the beat under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().records.clear();
}

/// A mounted app whose announcement routes drive the pipeline — the
/// example's window, assembled the consumer's way, with the observer
/// riding from the `set_observer` call on.
fn mount_app(harness: &mut TestHarness, observer: &Share) {
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, Label::new("The announcements").boxed());
    harness.settle();
    clear_trail(observer);
}

/// The live-region label the sink's last `TreeUpdate` carries — the AT's
/// reading of what the region holds right now.
fn region_label(harness: &mut TestHarness) -> (Live, Option<String>) {
    let sink = harness
        .pipeline_mut()
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<AccessKitSink>())
        .expect("the harness ships the AccessKit sink");
    let update = sink
        .last_update()
        .expect("the a11y pass produced an update");
    let region = find_in_update(update, ANNOUNCEMENT_NODE_HASH)
        .expect("the reserved region rides the update");
    (
        region.live().expect("the region carries its priority"),
        region.label().map(String::from),
    )
}

#[test]
fn m33_the_delivered_act_streams_one_record_with_its_politeness() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        AnnounceObserver::new(),
    )));
    mount_app(&mut harness, &observer);

    // The direct door accepts: one record, the politeness it asked for
    // named beside the delivery.
    let accepted = harness.pipeline_mut().announce("Saved", Politeness::Polite);
    assert!(accepted, "a rooted, non-empty announcement is accepted");
    harness.settle();

    let trail = records(&observer);
    assert_eq!(trail.len(), 1, "the accepted act streamed exactly once");
    assert_eq!(trail[0].politeness, Politeness::Polite);
    assert_eq!(trail[0].outcome, AnnounceOutcome::Delivered);

    // The region carries what the log says it delivered — the log is
    // the log, the tree answers the same fact.
    let (live, label) = region_label(&mut harness);
    assert_eq!(live, Live::Polite, "the polite priority rides the update");
    assert_eq!(label.as_deref(), Some("Saved"), "the AT reads the text");

    // A fact streams once: the next idle tick streams nothing.
    clear_trail(&observer);
    assert_eq!(harness.settle(), 0, "an idle act queues no masks");
    assert!(
        records(&observer).is_empty(),
        "an idle announcement streams nothing"
    );
}

#[test]
fn m33_the_refusals_stream_too_and_queue_no_masks() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        AnnounceObserver::new(),
    )));
    mount_app(&mut harness, &observer);

    // The empty speech act: refused at the door, the refusal is the
    // fact the log carries (the M28 honesty, the speech act's own).
    assert!(!harness.pipeline_mut().announce("", Politeness::Polite));
    assert!(
        !harness
            .pipeline_mut()
            .announce("   ", Politeness::Assertive)
    );
    harness.settle();
    let trail = records(&observer);
    assert_eq!(trail.len(), 2, "both empty refusals streamed");
    for record in &trail {
        assert_eq!(
            record.outcome,
            AnnounceOutcome::Refused(alacer_core::inspect::AnnounceRefusal::EmptyText),
            "an empty speech act names its refusal"
        );
    }
    assert_eq!(trail[0].politeness, Politeness::Polite);
    assert_eq!(trail[1].politeness, Politeness::Assertive);
    assert_eq!(
        harness.settle(),
        0,
        "a refused announcement queues no masks (the M18 rule)"
    );

    // The push-side route: the ambient `announce` refuses the empty act
    // on the calling thread — the fact queues on the thread's trail and
    // the tick's start drain streams it, the edit trail's shape.
    clear_trail(&observer);
    assert!(!alacer_core::announce::announce("  ", Politeness::Polite));
    assert_eq!(
        records(&observer).len(),
        0,
        "the push queues, never streams"
    );
    harness.settle();
    let trail = records(&observer);
    assert_eq!(trail.len(), 1, "the push-side refusal drained once");
    assert_eq!(
        trail[0].outcome,
        AnnounceOutcome::Refused(alacer_core::inspect::AnnounceRefusal::EmptyText)
    );
}

#[test]
fn m33_the_no_root_refusal_streams_at_the_door_and_the_drain() {
    // A fresh pipeline, nothing mounted: there is no tree to attach the
    // region to, so the direct door refuses honestly — and the refusal
    // is a fact the log carries.
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        AnnounceObserver::new(),
    )));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));

    assert!(!harness.pipeline_mut().announce("Saved", Politeness::Polite));
    harness.settle();
    let trail = records(&observer);
    assert_eq!(trail.len(), 1, "the door's refusal streamed");
    assert_eq!(
        trail[0].outcome,
        AnnounceOutcome::Refused(alacer_core::inspect::AnnounceRefusal::NoRoot)
    );

    // The ambient route queues for a tree that never mounts: the drain
    // drops the act and the drop streams — the same honest refusal, the
    // announcement drain's own answer.
    clear_trail(&observer);
    assert!(alacer_core::announce::announce(
        "spoken into the void",
        Politeness::Polite
    ));
    harness.settle();
    let trail = records(&observer);
    assert_eq!(trail.len(), 1, "the drain's drop streamed");
    assert_eq!(
        trail[0].outcome,
        AnnounceOutcome::Refused(alacer_core::inspect::AnnounceRefusal::NoRoot)
    );
    assert_eq!(
        harness.settle(),
        0,
        "the no-root refusal queues no masks (the M18 rule)"
    );
}

#[test]
fn m33_last_write_wins_is_visible_in_the_log() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let observer = Share(std::rc::Rc::new(std::cell::RefCell::new(
        AnnounceObserver::new(),
    )));
    mount_app(&mut harness, &observer);

    // Two speech acts in one batch: the drain answers both facts — the
    // replaced act streams Superseded, the survivor streams Delivered —
    // the replace rule visible as data instead of a silent drop.
    assert!(alacer_core::announce::announce("Saved", Politeness::Polite));
    assert!(alacer_core::announce::announce(
        "3 items left",
        Politeness::Assertive
    ));
    harness.settle();

    let trail = records(&observer);
    assert_eq!(trail.len(), 2, "the whole batch's facts streamed");
    assert_eq!(trail[0].politeness, Politeness::Polite);
    assert_eq!(trail[0].outcome, AnnounceOutcome::Superseded);
    assert_eq!(trail[1].politeness, Politeness::Assertive);
    assert_eq!(trail[1].outcome, AnnounceOutcome::Delivered);

    // The region carries the survivor — the log and the tree agree.
    let (live, label) = region_label(&mut harness);
    assert_eq!(live, Live::Assertive);
    assert_eq!(label.as_deref(), Some("3 items left"));

    // The direct door never sees the queue: two direct calls replace at
    // the door itself, each answering its own record.
    clear_trail(&observer);
    assert!(harness.pipeline_mut().announce("first", Politeness::Polite));
    assert!(
        harness
            .pipeline_mut()
            .announce("second", Politeness::Polite)
    );
    harness.settle();
    let trail = records(&observer);
    assert_eq!(trail.len(), 2, "each door call answered its own record");
    for record in &trail {
        assert_eq!(record.outcome, AnnounceOutcome::Delivered);
    }
    let (_, label) = region_label(&mut harness);
    assert_eq!(label.as_deref(), Some("second"), "last write wins");
}

#[test]
fn m33_no_observer_no_trail_the_idle_contract_holds() {
    // The default path: no observer, no enabled trail — the announcement
    // seam answers exactly as before and the frame costs what it always
    // cost (one branch per act, the A7 idle contract).
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, Label::new("The announcements").boxed());
    harness.settle();
    let accepted = harness.pipeline_mut().announce("Saved", Politeness::Polite);
    assert!(accepted, "the seam answers without a listener");
    let report = harness.tick();
    assert!(
        report.a11y_ran,
        "the region still rides the delta it always rode"
    );
    assert!(!alacer_core::announce::announce("", Politeness::Polite));
    assert_eq!(harness.settle(), 0, "the disabled trail queues no work");
}
