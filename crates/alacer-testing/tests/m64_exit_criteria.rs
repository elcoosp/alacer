//! M64 exit-criteria integration tests (§13.3, ADR-0070): the observer's
//! tee joins the seam. A consumer prices the *second door* — the
//! pipeline's observer slot is single (`PassPipeline::set_observer`),
//! and two devtools at once (the inspector's recorder on the wire
//! beside the provided reader in-process, ADR-0069's) needed either a
//! hand-rolled wrapper (the benches' own `Share` shape, again) or one
//! door silently losing the other. The seam grows the combinator the
//! slot's contract implies: [`alacer_core::Tee`], two observers behind
//! one, each event forwarded first to `first` then to `second` — the
//! arrival order the causal reads rely on — and a third reader nests
//! (`Tee::new(a, Tee::new(b, c))`), the combinator's own composition.
//!
//! The tee prices nothing a consumer does not ask for: two doors cost
//! two observations (the bench prices the fan-out beside the reader's
//! own, ADR-0069), and the A7 idle contract still holds — the slot's
//! single `None` branch unchanged. Every test routes the real pipeline
//! the way one frame's batch arrives — the announce route, the same
//! fact the M18 record named — and settles once to drain.

use alacer_core::announce::{Politeness, announce};
use alacer_core::inspect::{AnnounceOutcome, InspectEvent, InspectObserver, Tee};
use alacer_core::trail::TrailLog;
use alacer_inspect::CausalRecorder;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label};
use std::cell::RefCell;
use std::rc::Rc;

/// The announcement the tests stream — the ambient route's own fact.
fn speak() {
    assert!(announce("Saved", Politeness::Polite));
}

/// The arrival ledger the order test shares: one tag per observation,
/// the tee's own sequence made visible.
#[derive(Clone, Default)]
struct ArrivalLedger(Rc<RefCell<Vec<&'static str>>>);

struct ArrivalWatcher {
    tag: &'static str,
    ledger: ArrivalLedger,
}

impl InspectObserver for ArrivalWatcher {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Announce(_) = event else {
            return;
        };
        self.ledger.0.borrow_mut().push(self.tag);
    }
}

#[test]
fn the_tee_feeds_both_doors() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let first = TrailLog::new();
    let second = TrailLog::new();
    harness.pipeline_mut().set_observer(Box::new(Tee::new(
        Box::new(first.clone()),
        Box::new(second.clone()),
    )));
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Label::new("The tee's own column").boxed());
    harness.settle();

    speak();
    harness.settle();

    for (name, log) in [("first", &first), ("second", &second)] {
        let events = log.drain();
        let announced: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::Announce(record) => Some(*record),
                _ => None,
            })
            .collect();
        assert_eq!(
            announced.len(),
            1,
            "{name} priced the speech act — the tee fed both doors"
        );
        assert_eq!(
            announced[0].outcome,
            AnnounceOutcome::Delivered,
            "{name} read the same fact the single slot streamed"
        );
    }
}

#[test]
fn the_tee_forwards_first_then_second() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let ledger = ArrivalLedger::default();
    harness.pipeline_mut().set_observer(Box::new(Tee::new(
        Box::new(ArrivalWatcher {
            tag: "first",
            ledger: ledger.clone(),
        }),
        Box::new(ArrivalWatcher {
            tag: "second",
            ledger: ledger.clone(),
        }),
    )));
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Label::new("The order's own column").boxed());
    harness.settle();

    speak();
    harness.settle();
    speak();
    harness.settle();

    let order = ledger.0.borrow().clone();
    assert_eq!(
        order,
        vec!["first", "second", "first", "second"],
        "each event forwards first to `first` then to `second` — the arrival order the causal reads rely on"
    );
}

#[test]
fn the_recorder_and_the_reader_share_the_slot() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let recorder = CausalRecorder::new("m64-tee");
    let handle = recorder.clone_handle();
    let reader = TrailLog::new();
    harness.pipeline_mut().set_observer(Box::new(Tee::new(
        Box::new(handle.clone()),
        Box::new(reader.clone()),
    )));
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Label::new("The shared slot's column").boxed());
    harness.settle();

    speak();
    harness.settle();

    let wire = recorder.announces();
    assert_eq!(
        wire.len(),
        1,
        "the recorder still rides the wire — the tee starved no door"
    );
    let in_process = reader.drain();
    let counted = in_process
        .iter()
        .filter(|event| matches!(event, InspectEvent::Announce(_)))
        .count();
    assert_eq!(
        counted, 1,
        "the provided reader priced the same act in-process (ADR-0069's door)"
    );
}

#[test]
fn the_nested_tee_serves_three() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let a = TrailLog::new();
    let b = TrailLog::new();
    let c = TrailLog::new();
    harness.pipeline_mut().set_observer(Box::new(Tee::new(
        Box::new(a.clone()),
        Box::new(Tee::new(Box::new(b.clone()), Box::new(c.clone()))),
    )));
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Label::new("The nest's own column").boxed());
    harness.settle();

    speak();
    harness.settle();

    for (name, log) in [("a", &a), ("b", &b), ("c", &c)] {
        let events = log.drain();
        let announced = events
            .iter()
            .filter(|event| matches!(event, InspectEvent::Announce(_)))
            .count();
        assert_eq!(
            announced, 1,
            "{name} priced the speech act — the nested tee composed"
        );
    }
}
