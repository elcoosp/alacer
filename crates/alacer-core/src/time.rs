//! The time seam: how the framework's clocks read time without any
//! consumer depending on the host's clock shape (ADR-0107).
//!
//! Two contracts in the workspace read a clock. The platform's click
//! chain prices every press against the double-click window (§10.3,
//! ADR-0019) — and its own test block carried the pre-registered gap
//! since M13: "the time bound rides `Instant` and is not sleep-tested".
//! The pipeline's pass timings price every timed frame (§7.2, the M1
//! gate instrumentation) against the same wall clock. Both hardcoded
//! [`std::time::Instant::now`] — the clock was the one external effect
//! with no seam, so the code that needed deterministic time (a test
//! asserting the chain's window, a harness pricing a frame exactly)
//! had to sleep and hope.
//!
//! Rather than dragging a host clock dependency into every consumer's
//! dependency set, the reading site holds one [`TimeSeam`] implementation
//! — the same discipline the clipboard seam ([`crate::clipboard`]) and
//! the setting seam ([`crate::settings`]) impose, with the ownership
//! shape each site already uses: the click chain owns its clock box,
//! the pipeline installs its clock through the
//! [`crate::passes::PassPipeline::set_clock`] door.
//!
//! ```text
//! // The canon read — real time, the default build's shape:
//! use alacer_core::time::{MonotonicClock, TimeSeam};
//! let clock = MonotonicClock::new();
//! let then = clock.now_nanos();
//!
//! // Tests and harnesses — the deterministic seam:
//! use alacer_core::time::ManualClock;
//! let clock = ManualClock::new();
//! clock.advance(std::time::Duration::from_millis(200));
//! assert_eq!(clock.now_nanos(), 200_000_000);
//!
//! // A consumer's own shape — one method, one monotonic read:
//! struct StepClock { nanos: std::cell::Cell<u64>, step: u64 }
//! impl TimeSeam for StepClock {
//!     fn now_nanos(&self) -> u64 {
//!         let next = self.nanos.get() + self.step;
//!         self.nanos.set(next);
//!         next
//!     }
//! }
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// The time seam: one monotonic clock read. One implementation per
/// reading site — installed where the clock's consumer is assembled.
///
/// The contract is minimal on purpose: a clock answers *now*, in
/// monotonic nanoseconds since an arbitrary anchor, and never goes
/// backwards within one installed clock. Durations are the caller's
/// arithmetic — differences of two reads — the shape both contracts
/// already ride (the chain's window bound, the pass ledger's
/// `nanos − start`). There is deliberately no sleep, no deadline and
/// no wall-calendar read: a wall date is a platform fact the OS owns
/// (the setting seam's tier), and a waiting framework is not this
/// framework's shape — the winit loop parks, it never polls a clock.
pub trait TimeSeam: 'static {
    /// Now, in monotonic nanoseconds since this clock's own anchor.
    /// Successive reads within one clock never go backwards; the
    /// anchor is arbitrary (the canon clock's is its construction
    /// instant) — only differences between reads mean anything.
    fn now_nanos(&self) -> u64;
}

/// The canon clock (the provided-implementation rule beside every
/// seam — [`crate::clipboard::MemoryClipboard`]'s precedent): real
/// monotonic time, read off [`std::time::Instant`]. The default
/// build's shape — the platform's click chain and the pipeline's
/// un-seamed timings ride it, and every frame they price is the wall
/// clock's own.
#[derive(Debug)]
pub struct MonotonicClock {
    /// The anchor: the construction instant. Only differences mean
    /// anything, so a fresh clock per reader is the honest shape —
    /// the nanos it answers are process-lifetime-ish, never a
    /// calendar fact.
    anchor: std::time::Instant,
}

impl MonotonicClock {
    /// A canon clock anchored at its construction instant.
    #[must_use]
    pub fn new() -> Self {
        Self {
            anchor: std::time::Instant::now(),
        }
    }
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeSeam for MonotonicClock {
    fn now_nanos(&self) -> u64 {
        crate::num::u128_to_u64(self.anchor.elapsed().as_nanos())
    }
}

/// The deterministic clock: what tests, harnesses and demos install so
/// the code that reads time answers *exactly what the test advanced it
/// to* — no sleeps, no hoping the scheduler agrees (the M13 gap's
/// honest replacement). A clone shares the one reading — the test
/// advances through its handle while the consumer reads through
/// theirs, the same shared-handle shape the recorder's
/// [`crate::inspect::InspectObserver`] handles ride.
#[derive(Debug, Clone, Default)]
pub struct ManualClock {
    /// The one reading every clone shares.
    nanos: Arc<AtomicU64>,
}

impl ManualClock {
    /// A deterministic clock reading zero.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nanos: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Advances the shared reading by `by`. Reads after the advance
    /// answer the new value; a paused clock prices every duration
    /// exactly zero — the deterministic frame's shape.
    pub fn advance(&self, by: std::time::Duration) {
        self.advance_nanos(u64::try_from(by.as_nanos()).unwrap_or(u64::MAX));
    }

    /// Advances the shared reading by `by` nanoseconds, saturating at
    /// the seam's own width (u64 nanos is 584 years from the anchor —
    /// the saturation is arithmetic honesty, never a reachable state).
    pub fn advance_nanos(&self, by: u64) {
        self.nanos.fetch_add(by, Ordering::Relaxed);
    }

    /// The shared reading, without the seam's indirection — a test
    /// asserting the clock itself, not a consumer's price.
    #[must_use]
    pub fn nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }
}

impl TimeSeam for ManualClock {
    fn now_nanos(&self) -> u64 {
        self.nanos()
    }
}

#[cfg(test)]
mod tests {
    use super::{ManualClock, MonotonicClock, TimeSeam};

    #[test]
    fn the_manual_clock_reads_what_it_was_advanced_to() {
        let clock = ManualClock::new();
        assert_eq!(clock.now_nanos(), 0, "a fresh clock reads zero");
        clock.advance(std::time::Duration::from_millis(250));
        assert_eq!(
            clock.now_nanos(),
            250_000_000,
            "the read answers exactly what the advance wrote"
        );
    }

    #[test]
    fn clones_of_the_manual_clock_share_one_reading() {
        let clock = ManualClock::new();
        let handle = clock.clone();
        handle.advance(std::time::Duration::from_millis(100));
        assert_eq!(
            clock.now_nanos(),
            100_000_000,
            "the clone's advance moved the one shared reading"
        );
    }

    #[test]
    fn monotonic_readings_never_go_backwards() {
        let clock = MonotonicClock::new();
        let first = clock.now_nanos();
        let second = clock.now_nanos();
        assert!(
            second >= first,
            "the canon clock never answers an earlier nanos ({second} < {first})"
        );
    }

    #[test]
    fn a_paused_manual_clock_prices_every_duration_zero() {
        let clock = ManualClock::new();
        let start = clock.now_nanos();
        let end = clock.now_nanos();
        assert_eq!(
            end.saturating_sub(start),
            0,
            "a clock the test did not advance prices zero — the deterministic frame"
        );
    }
}
