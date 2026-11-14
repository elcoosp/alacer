//! The trail's provided reader (§13.3, ADR-0069): the bounded,
//! in-process [`InspectObserver`] the seam provides beside the trait —
//! the way the clipboard seam provides [`crate::clipboard::MemoryClipboard`].
//! Every m-suite gate and every trail example hand-rolled the same
//! collector (`Rc<RefCell<VecDeque>>` behind a one-method wrapper, the
//! m-suite's "helper shape" repeated 33 files over) to answer one
//! question a consumer now asks the seam directly: *what did the log
//! say*. [`TrailLog`] is that answer: the pipeline forwards its
//! [`InspectEvent`]s into a ring, the drain hands the window back
//! oldest-first, and the eviction is named — a dropped count the ring
//! increments, never a silent loss (the named-refusal rule at the
//! reader's own tier).
//!
//! The window disciplines (A7) hold in both directions. The trail-off
//! pipeline streams nothing — the observer's branch is the only cost
//! the idle window pays. A trail nobody drains is bounded: the ring
//! holds its cap, evicts its oldest, counts the eviction — a devtools
//! log, never a frame-budget liability. The facts ride the routes'
//! own records (the payload itself never on the log, ADR-0011's rule
//! at every tier), so a reader that mirrors a field's value reads the
//! lengths, not the secret.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::inspect::{InspectEvent, InspectObserver};

/// The reader's default window: the events a [`TrailLog`] holds before
/// the ring evicts its oldest (the [`InspectObserver`] contract's own
/// bound — a devtools log, not a frame-budget liability). A reader
/// needing a different window prices it at construction
/// ([`TrailLog::with_cap`]).
pub const TRAIL_RING_CAP: usize = 1024;

/// The trail's provided reader (§13.3, ADR-0069): a bounded ring the
/// pipeline forwards into, clonable so one installation serves many
/// readers. The observing half is the trait's own one method; the
/// reading half is the window's — [`TrailLog::drain`] hands the held
/// events over and empties the window, [`TrailLog::look`] reads it
/// without disturbing it, [`TrailLog::dropped`] names what the ring
/// evicted.
///
/// # Panics
///
/// [`TrailLog::with_cap`] refuses a `0` cap (a reader that remembers
/// nothing would silently drop every fact).
#[derive(Debug, Clone)]
pub struct TrailLog {
    inner: Arc<Mutex<TrailRing>>,
}

/// The ring behind the handle: the held window, the eviction count,
/// the cap the window is priced at.
#[derive(Debug)]
struct TrailRing {
    ring: VecDeque<InspectEvent>,
    dropped: u64,
    cap: usize,
}

impl TrailLog {
    /// A reader with the default window ([`TRAIL_RING_CAP`]).
    #[must_use]
    pub fn new() -> Self {
        Self::with_cap(TRAIL_RING_CAP)
    }

    /// A reader with the window priced at `cap` events.
    ///
    /// # Panics
    ///
    /// Panics when `cap` is `0` — a window that holds nothing would
    /// silently drop every fact, and a silent drop is the one shape
    /// the trail never streams.
    #[must_use]
    pub fn with_cap(cap: usize) -> Self {
        assert!(
            cap > 0,
            "refused: a trail that remembers nothing — help: the ring's window is priced in events; \
             `0` would silently drop every fact the routes stream (the named-refusal rule), \
             install `1` or higher"
        );
        Self {
            inner: Arc::new(Mutex::new(TrailRing {
                ring: VecDeque::with_capacity(cap),
                dropped: 0,
                cap,
            })),
        }
    }

    /// The window's cap: the events the ring holds before evicting.
    ///
    /// # Panics
    ///
    /// Panics if the ring's lock is poisoned (a reader panicked while
    /// holding it).
    #[must_use]
    pub fn cap(&self) -> usize {
        self.inner.lock().expect("trail lock").cap
    }

    /// The events the window holds — what nobody has drained yet.
    ///
    /// # Panics
    ///
    /// Panics if the ring's lock is poisoned (a reader panicked while
    /// holding it).
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.lock().expect("trail lock").ring.len()
    }

    /// Whether the window holds nothing — the drained (or never-fed)
    /// state.
    ///
    /// # Panics
    ///
    /// Panics if the ring's lock is poisoned (a reader panicked while
    /// holding it).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.lock().expect("trail lock").ring.is_empty()
    }

    /// The events the ring has evicted — the named eviction (the
    /// oldest left so the newest fits), cumulative since the reader's
    /// construction and never reset by a drain: a reader between
    /// drains diffs the count to know whether the window overflowed
    /// while it was away.
    ///
    /// # Panics
    ///
    /// Panics if the ring's lock is poisoned (a reader panicked while
    /// holding it).
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.inner.lock().expect("trail lock").dropped
    }

    /// The window's contents, oldest first — without disturbing it.
    /// A reader that looks keeps the window: the next drain hands the
    /// same facts over.
    ///
    /// # Panics
    ///
    /// Panics if the ring's lock is poisoned (a reader panicked while
    /// holding it).
    #[must_use]
    pub fn look(&self) -> Vec<InspectEvent> {
        self.inner.lock().expect("trail lock").ring.clone().into()
    }

    /// The window's tip — the newest fact — without disturbing the
    /// window. `None` when the window is empty.
    ///
    /// # Panics
    ///
    /// Panics if the ring's lock is poisoned (a reader panicked while
    /// holding it).
    #[must_use]
    pub fn last(&self) -> Option<InspectEvent> {
        self.inner.lock().expect("trail lock").ring.back().cloned()
    }

    /// The window, drained: the held events handed back oldest first,
    /// the window emptied behind them. The eviction count is the
    /// reader's own — a drain never resets it.
    ///
    /// # Panics
    ///
    /// Panics if the ring's lock is poisoned (a reader panicked while
    /// holding it).
    #[must_use]
    pub fn drain(&self) -> Vec<InspectEvent> {
        std::mem::take(&mut self.inner.lock().expect("trail lock").ring).into()
    }
}

impl Default for TrailLog {
    fn default() -> Self {
        Self::new()
    }
}

impl InspectObserver for TrailLog {
    fn observe(&mut self, event: InspectEvent) {
        let mut ring = self.inner.lock().expect("trail lock");
        if ring.ring.len() >= ring.cap {
            ring.ring.pop_front();
            ring.dropped += 1;
        }
        ring.ring.push_back(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::{InputKind, InputRecord};

    /// One observation at a known target — the ring tests' exact
    /// arithmetic input (one event per call, no route noise).
    fn a_pointer_down() -> InspectEvent {
        InspectEvent::Input(InputRecord {
            kind: InputKind::PointerDown,
            target: None,
        })
    }

    #[test]
    fn the_default_window_is_the_seams_cap() {
        let trail = TrailLog::new();
        assert_eq!(trail.cap(), TRAIL_RING_CAP);
        assert!(trail.is_empty());
        assert_eq!(trail.dropped(), 0);
    }

    #[test]
    fn the_ring_holds_its_cap_and_names_the_eviction() {
        let mut trail = TrailLog::with_cap(4);
        for _ in 0..7 {
            trail.observe(a_pointer_down());
        }
        assert_eq!(trail.len(), 4);
        assert_eq!(trail.dropped(), 3);
        let window = trail.drain();
        assert_eq!(window.len(), 4);
        assert_eq!(trail.dropped(), 3, "the eviction count outlives a drain");
        assert!(trail.is_empty());
    }

    #[test]
    fn the_drain_hands_back_oldest_first() {
        let mut trail = TrailLog::with_cap(8);
        for _ in 0..3 {
            trail.observe(a_pointer_down());
        }
        let window = trail.drain();
        assert_eq!(window.len(), 3);
        assert!(trail.last().is_none());
        assert_eq!(trail.dropped(), 0);
    }

    #[test]
    fn the_look_leaves_the_window_intact() {
        let mut trail = TrailLog::with_cap(8);
        trail.observe(a_pointer_down());
        let looked = trail.look();
        assert_eq!(looked.len(), 1);
        assert_eq!(trail.len(), 1, "a look is not a drain");
        assert!(trail.last().is_some());
    }

    #[test]
    #[should_panic(expected = "refused: a trail that remembers nothing")]
    fn a_zero_cap_is_refused() {
        let _ = TrailLog::with_cap(0);
    }

    #[test]
    fn clone_handles_read_one_ring() {
        let mut trail = TrailLog::with_cap(8);
        let reader = trail.clone();
        trail.observe(a_pointer_down());
        assert_eq!(reader.len(), 1, "the handle reads the same ring");
        assert_eq!(reader.drain().len(), 1);
        assert!(trail.is_empty(), "one ring behind every handle");
    }
}
