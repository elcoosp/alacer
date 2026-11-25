//! The layout seam's call ledger (ADR-0097): a recording engine beside
//! the production one — every [`LayoutEngine`] call rides the wrapped
//! engine untouched, and one [`LayoutCallRecord`] lands in the ledger
//! beside it: the root the call asked, the dirty set's size, the
//! viewport the clamp priced against, the outcome. The `measure_calls()`
//! counter the seam has priced itself with since M1 carries one number;
//! the ledger carries the story.
//!
//! The instrument is installed by the binary that wants it — the
//! test/bench/example binaries, a consumer debugging their own injected
//! engine (the seam is public on purpose) — never by the framework (the
//! A7 rule the alloc ledger shipped by, ADR-0088).
//! [`crate::LayoutHost`] stays the production stage;
//! `alacer_testing::TestHarness::with_layout` is the harness door that
//! drives the instrument through the real pipeline.

use alacer_core::arena::NodeId;
use alacer_core::geom::Size;
use alacer_core::mask::Change;
use alacer_core::passes::{LayoutEngine, LayoutError};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;

/// The ledger's window: records beyond the cap evict their oldest, the
/// eviction counted on the handle — the A7 window discipline the trail
/// ring answers by (a ledger nobody drains never leaks, it forgets and
/// says so).
pub const DEFAULT_LEDGER_CAP: usize = 4_096;

/// One layout call's story: the facts the seam's counter never carried.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutCallRecord {
    /// The root the call asked to lay out.
    pub root: NodeId,
    /// The dirty set's size — the nodes the frame named.
    pub dirty: usize,
    /// The viewport the clamp priced against.
    pub viewport: Size,
    /// The call's outcome: delivered, or the wrapped engine's own
    /// error — the error context the seam's `Result` contract ships,
    /// carried into the ledger (ADR-0023's honesty: a refusal reads as
    /// itself, never a bare flag). The typed error rides the ledger
    /// unchanged: `LayoutError` is `Clone + PartialEq`, so the record
    /// carries the failure the engine answered with — the failing node
    /// beside the prose — never a string-erased shadow of it.
    pub outcome: Result<(), LayoutError>,
}

/// The reading handle over the recording engine's ledger (the trail
/// door's own shape: the reader cloned beside the install, so the test
/// reads through it after the harness has taken the engine's ownership).
#[derive(Clone)]
pub struct LayoutLedger {
    ledger: Rc<RefCell<Window>>,
}

impl LayoutLedger {
    /// The records so far, oldest first — a snapshot: one clone, no
    /// borrow held, the engine free to keep recording while read.
    #[must_use]
    pub fn calls(&self) -> Vec<LayoutCallRecord> {
        self.ledger.borrow().records.clone()
    }

    /// Empties the ledger (the test's drain).
    pub fn clear(&self) {
        self.ledger.borrow_mut().records.clear();
    }

    /// How many records the window evicted — the honesty counter: a
    /// ledger that forgot says so (the A7 rule).
    #[must_use]
    pub fn evicted(&self) -> u64 {
        self.ledger.borrow().evicted
    }
}

/// The bounded window behind the handle: records, the eviction count,
/// the cap they answer to.
struct Window {
    records: Vec<LayoutCallRecord>,
    evicted: u64,
    cap: usize,
}

impl Window {
    fn push(&mut self, record: LayoutCallRecord) {
        if self.cap == 0 {
            self.evicted += 1;
            return;
        }
        if self.records.len() >= self.cap {
            let drop_count = self.records.len() - self.cap + 1;
            self.records.drain(0..drop_count);
            self.evicted += u64::try_from(drop_count).unwrap_or(u64::MAX);
        }
        self.records.push(record);
    }
}

/// The recording engine: a delegating [`LayoutEngine`] whose every call
/// lands one [`LayoutCallRecord`] in the ledger — the wrapped engine
/// untouched, the instrument beside it.
pub struct RecordingLayout {
    inner: Box<dyn LayoutEngine>,
    ledger: Rc<RefCell<Window>>,
}

impl RecordingLayout {
    /// Wraps an engine: the production host, or the consumer's own.
    #[must_use]
    pub fn new(inner: Box<dyn LayoutEngine>) -> Self {
        Self::with_cap(inner, DEFAULT_LEDGER_CAP)
    }

    /// Wraps an engine with a named window — the cap the records answer
    /// to (zero records everything: the honesty counter climbs instead).
    #[must_use]
    pub fn with_cap(inner: Box<dyn LayoutEngine>, cap: usize) -> Self {
        Self {
            inner,
            ledger: Rc::new(RefCell::new(Window {
                records: Vec::new(),
                evicted: 0,
                cap,
            })),
        }
    }

    /// The reading handle (the trail door's shape): cloned beside the
    /// install, read after the pipeline has taken the engine's
    /// ownership.
    #[must_use]
    pub fn handle(&self) -> LayoutLedger {
        LayoutLedger {
            ledger: Rc::clone(&self.ledger),
        }
    }

    /// The records so far, oldest first.
    #[must_use]
    pub fn calls(&self) -> Vec<LayoutCallRecord> {
        self.handle().calls()
    }

    /// Empties the ledger.
    pub fn clear(&mut self) {
        self.handle().clear();
    }

    /// How many records the window evicted so far.
    #[must_use]
    pub fn evicted(&self) -> u64 {
        self.handle().evicted()
    }
}

impl LayoutEngine for RecordingLayout {
    fn sync_and_layout(
        &mut self,
        arena: &mut alacer_core::Arena,
        root: NodeId,
        dirty: &FxHashMap<NodeId, Change>,
        viewport: Size,
    ) -> Result<(), LayoutError> {
        let outcome = self.inner.sync_and_layout(arena, root, dirty, viewport);
        let record = LayoutCallRecord {
            root,
            dirty: dirty.len(),
            viewport,
            outcome: match &outcome {
                Ok(()) => Ok(()),
                Err(error) => Err(error.clone()),
            },
        };
        self.ledger.borrow_mut().push(record);
        outcome
    }

    fn measure_calls(&self) -> u64 {
        self.inner.measure_calls()
    }

    /// The effect's door forwards (ADR-0160): the recording wrapper is
    /// transparent to the text scale — the wrapped engine owns the
    /// measure keys, so the flip's ask rides to it untouched.
    fn set_text_scale(&mut self, scale: f32) -> bool {
        self.inner.set_text_scale(scale)
    }

    fn reset_measure_calls(&mut self) {
        self.inner.reset_measure_calls();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The delivering stub: syncs succeed; the measure count answers a
    /// cell the test drives (the counter delegation's probe).
    struct DeliveringEngine {
        measures: std::cell::Cell<u64>,
    }

    impl LayoutEngine for DeliveringEngine {
        fn sync_and_layout(
            &mut self,
            _arena: &mut alacer_core::Arena,
            _root: NodeId,
            _dirty: &FxHashMap<NodeId, Change>,
            _viewport: Size,
        ) -> Result<(), LayoutError> {
            Ok(())
        }

        fn measure_calls(&self) -> u64 {
            self.measures.get()
        }

        fn reset_measure_calls(&mut self) {
            self.measures.set(0);
        }
    }

    /// The refusing stub: the sync's message names itself — the record's
    /// error-context probe.
    struct RefusingEngine {
        message: &'static str,
    }

    impl LayoutEngine for RefusingEngine {
        fn sync_and_layout(
            &mut self,
            _arena: &mut alacer_core::Arena,
            _root: NodeId,
            _dirty: &FxHashMap<NodeId, Change>,
            _viewport: Size,
        ) -> Result<(), LayoutError> {
            Err(LayoutError {
                message: self.message.to_string(),
                node: None,
            })
        }

        fn measure_calls(&self) -> u64 {
            0
        }

        fn reset_measure_calls(&mut self) {}
    }

    fn dirty_map(entries: usize) -> FxHashMap<NodeId, Change> {
        (0..entries)
            .map(|_| (NodeId::allocate(), Change::LAYOUT))
            .collect()
    }

    #[test]
    fn one_call_one_record() {
        let mut engine = RecordingLayout::new(Box::new(DeliveringEngine {
            measures: std::cell::Cell::new(0),
        }));
        let root = NodeId::allocate();
        let outcome = engine.sync_and_layout(
            &mut alacer_core::Arena::new(),
            root,
            &dirty_map(3),
            Size::new(800.0, 600.0),
        );
        assert!(outcome.is_ok(), "the wrapped engine answered untouched");
        let records = engine.calls();
        assert_eq!(records.len(), 1, "one call, one record");
        assert_eq!(records[0].root, root, "the asked root rides");
        assert_eq!(records[0].dirty, 3, "the dirty set's size rides");
        assert_eq!(records[0].viewport, Size::new(800.0, 600.0));
        assert_eq!(records[0].outcome, Ok(()), "the delivery rides");
    }

    #[test]
    fn the_counters_delegate() {
        let mut engine = RecordingLayout::new(Box::new(DeliveringEngine {
            measures: std::cell::Cell::new(7),
        }));
        assert_eq!(engine.measure_calls(), 7, "the count answers through");
        engine.reset_measure_calls();
        assert_eq!(engine.measure_calls(), 0, "the reset answers through");
    }

    #[test]
    fn the_refusal_rides_the_record_with_its_message() {
        let mut engine = RecordingLayout::new(Box::new(RefusingEngine {
            message: "the persistent tree refused the style",
        }));
        let outcome = engine.sync_and_layout(
            &mut alacer_core::Arena::new(),
            NodeId::allocate(),
            &dirty_map(1),
            Size::new(320.0, 240.0),
        );
        assert!(outcome.is_err(), "the refusal is the answer");
        let records = engine.calls();
        assert_eq!(records.len(), 1);
        assert!(
            records[0]
                .outcome
                .as_ref()
                .is_err_and(|error| error.message.contains("refused the style")),
            "the error's own message rides the record: {:?}",
            records[0].outcome
        );
    }

    #[test]
    fn the_handle_reads_after_clear() {
        let mut engine = RecordingLayout::new(Box::new(DeliveringEngine {
            measures: std::cell::Cell::new(0),
        }));
        let ledger = engine.handle();
        let _ = engine.sync_and_layout(
            &mut alacer_core::Arena::new(),
            NodeId::allocate(),
            &dirty_map(1),
            Size::new(800.0, 600.0),
        );
        assert_eq!(ledger.calls().len(), 1, "the handle reads the window");
        ledger.clear();
        assert!(ledger.calls().is_empty(), "the drain empties the window");
    }

    #[test]
    fn the_window_evicts_its_oldest_and_counts() {
        let mut engine = RecordingLayout::with_cap(
            Box::new(DeliveringEngine {
                measures: std::cell::Cell::new(0),
            }),
            2,
        );
        let ledger = engine.handle();
        let roots: Vec<NodeId> = (0..5).map(|_| NodeId::allocate()).collect();
        for root in &roots {
            let _ = engine.sync_and_layout(
                &mut alacer_core::Arena::new(),
                *root,
                &dirty_map(1),
                Size::new(800.0, 600.0),
            );
        }
        let records = ledger.calls();
        assert_eq!(records.len(), 2, "the window holds its cap");
        assert_eq!(
            records[0].root, roots[3],
            "the oldest evicted — the window holds the newest"
        );
        assert_eq!(ledger.evicted(), 3, "the eviction is counted, never silent");
    }
}
