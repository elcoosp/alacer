//! The allocation ledger (§A7, M84, ADR-0088): the heap budget's checker
//! — the instrument architecture A7's consequence has always named
//! ("the warm paint path for a static UI performs **zero heap
//! allocations** [budget, CI-enforced via dhat]"), adopted, not written
//! (A2 — steal proven cores; own only the composition).
//!
//! The hook is [`dhat`]'s [`Alloc`] global allocator; the composition the
//! ledger owns is the budget's vocabulary: an armed [`AllocLedger`]
//! window, [`AllocStats`] snapshots, and [`AllocDelta`] answers the exit
//! suites assert against. The framework never installs the allocator —
//! a test binary, a bench, or an example does, at its own top level:
//!
//! ```text
//! #[global_allocator]
//! static LEDGER: alacer_testing::alloc::Alloc = alacer_testing::alloc::Alloc;
//!
//! let ledger = AllocLedger::arm();
//! let before = ledger.stats();
//! let (report, delta) = ledger.measure(|| harness.tick());
//! assert!(delta.is_clean(), "the warm tick allocated: {delta:?}");
//! ```
//!
//! The shape is the trail tiers' own (§13.3): a bounded, honest window
//! that costs nothing while nobody arms it — the [`Alloc`] hook forwards
//! to the system allocator until a [`Profiler`](dhat::Profiler) lives,
//! and the exit suites arm one for exactly the beats they price. One
//! ledger at a time (dhat's own rule): arm, measure, drop, re-arm.

pub use dhat::Alloc;

/// One heap snapshot: the counters the armed ledger holds at an instant.
///
/// The totals are cumulative since the ledger armed; the live counters
/// are what is still allocated now; the peak counters are what the heap
/// held at its widest. [`AllocLedger::measure`] differences two of these
/// into an [`AllocDelta`]; the exit suites read them for the ledger's
/// own lines (`examples/alloc_budget.rs` prints them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocStats {
    /// Blocks allocated since the ledger armed — the cumulative count.
    pub total_allocs: u64,
    /// Bytes allocated since the ledger armed — the cumulative total.
    pub total_bytes: u64,
    /// Blocks still allocated at this instant.
    pub live_allocs: u64,
    /// Bytes still allocated at this instant.
    pub live_bytes: u64,
    /// Blocks the heap held at its widest point since the ledger armed.
    pub peak_allocs: u64,
    /// Bytes the heap held at its widest point since the ledger armed.
    pub peak_bytes: u64,
}

impl AllocStats {
    /// Reads the counters the armed ledger holds now.
    ///
    /// # Panics
    ///
    /// Panics when no ledger is armed — dhat's counters exist only while
    /// a [`dhat::Profiler`] runs, and a snapshot outside the window is a
    /// bug, never a stale number. The exit suites read through
    /// [`AllocLedger::stats`] and [`AllocLedger::measure`], the window's
    /// own doors.
    #[must_use]
    pub fn get() -> Self {
        let stats = dhat::HeapStats::get();
        Self {
            total_allocs: stats.total_blocks,
            total_bytes: stats.total_bytes,
            live_allocs: u64::try_from(stats.curr_blocks).unwrap_or(u64::MAX),
            live_bytes: u64::try_from(stats.curr_bytes).unwrap_or(u64::MAX),
            peak_allocs: u64::try_from(stats.max_blocks).unwrap_or(u64::MAX),
            peak_bytes: u64::try_from(stats.max_bytes).unwrap_or(u64::MAX),
        }
    }
}

/// The heap's movement between two [`AllocStats`] snapshots — one
/// measured beat, priced.
///
/// The counts name the movement honestly: `allocations` and
/// `deallocations` are the blocks that moved in and out of the window
/// (a block both made and freed inside the beat counts on both sides —
/// the churn the budget prices), `allocated_bytes` and `freed_bytes`
/// the same in bytes. [`AllocDelta::is_clean`] is the A7 budget's
/// answer; [`AllocDelta::is_balanced`] prices whether the beat left the
/// heap where it found it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AllocDelta {
    /// Blocks allocated during the measured beat.
    pub allocations: u64,
    /// Blocks freed during the measured beat.
    pub deallocations: u64,
    /// Bytes allocated during the measured beat.
    pub allocated_bytes: u64,
    /// Bytes freed during the measured beat.
    pub freed_bytes: u64,
}

impl AllocDelta {
    /// The A7 budget's answer: nothing was allocated and nothing freed —
    /// the beat moved the heap not at all. This is what a static UI's
    /// warm tick answers (the m84 suite and the `alloc` bench pin it).
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.allocations == 0 && self.deallocations == 0
    }

    /// Whether the beat left the heap where it found it: every block it
    /// allocated was freed inside the beat. A balanced, non-clean beat
    /// is transient churn — the bounded trails' drain vectors, the
    /// raster the harness materializes — never a growing ledger.
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.allocations == self.deallocations
    }

    /// The beat's net effect on the heap, in bytes: positive grew, zero
    /// balanced, negative shrank.
    #[must_use]
    pub fn net_bytes(&self) -> i64 {
        if self.allocated_bytes >= self.freed_bytes {
            i64::try_from(self.allocated_bytes - self.freed_bytes).unwrap_or(i64::MAX)
        } else {
            -i64::try_from(self.freed_bytes - self.allocated_bytes).unwrap_or(i64::MAX)
        }
    }
}

/// The armed measurement window: one [`dhat::Profiler`] in testing mode,
/// alive exactly as long as the beats it prices.
///
/// Arming is the ledger's whole cost model: the [`Alloc`] hook forwards
/// to the system allocator until a ledger lives, and while one lives it
/// counts every block on the thread. One ledger at a time (dhat's own
/// rule — a second `Profiler` panics): arm, measure the beats, drop,
/// re-arm. Testing mode never writes dhat's profile file — the window
/// answers only to the code that reads it.
pub struct AllocLedger {
    _profiler: dhat::Profiler,
}

impl AllocLedger {
    /// Arms the ledger: heap tracking starts here and stops when the
    /// ledger drops.
    ///
    /// # Panics
    ///
    /// Panics when another ledger (any `dhat::Profiler`) is already
    /// running — one window per process, the suites' own rule.
    #[must_use]
    pub fn arm() -> Self {
        Self {
            _profiler: dhat::Profiler::builder().testing().build(),
        }
    }

    /// The counters now.
    #[must_use]
    pub fn stats(&self) -> AllocStats {
        AllocStats::get()
    }

    /// The heap's movement since `earlier` — the beat the two snapshots
    /// bracket.
    ///
    /// The algebra reads the counters, never guesses: `allocations` is
    /// the cumulative totals' difference; `deallocations` is what the
    /// live counters say left (every block allocated inside the beat
    /// that is not still alive now, plus the earlier-live blocks the
    /// beat freed) — a block both made and freed inside the beat counts
    /// on both sides, the churn the budget prices.
    #[must_use]
    pub fn delta_since(&self, earlier: AllocStats) -> AllocDelta {
        let now = self.stats();
        let allocations = now.total_allocs.saturating_sub(earlier.total_allocs);
        let allocated_bytes = now.total_bytes.saturating_sub(earlier.total_bytes);
        let deallocations = earlier
            .live_allocs
            .saturating_add(allocations)
            .saturating_sub(now.live_allocs);
        let freed_bytes = earlier
            .live_bytes
            .saturating_add(allocated_bytes)
            .saturating_sub(now.live_bytes);
        AllocDelta {
            allocations,
            deallocations,
            allocated_bytes,
            freed_bytes,
        }
    }

    /// Prices one beat: snapshots, runs `body`, snapshots again, answers
    /// the movement beside the body's own answer.
    ///
    /// ```text
    /// let (report, delta) = ledger.measure(|| harness.tick());
    /// assert!(delta.is_clean(), "the warm tick allocated: {delta:?}");
    /// ```
    pub fn measure<T>(&self, body: impl FnOnce() -> T) -> (T, AllocDelta) {
        let before = self.stats();
        let answer = body();
        let delta = self.delta_since(before);
        (answer, delta)
    }
}

impl std::fmt::Display for AllocDelta {
    /// The delta's own ledger line: the budget's answer when the beat
    /// moved nothing, the counted movement when it did.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_clean() {
            write!(f, "nothing moved")
        } else {
            write!(
                f,
                "{} blocks / {} bytes allocated, {} freed (net {} bytes)",
                self.allocations,
                self.allocated_bytes,
                self.freed_bytes,
                self.net_bytes()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AllocDelta;

    /// The clean answer is the budget's own: no allocations, no
    /// deallocations, nothing moved.
    #[test]
    fn a_clean_delta_answers_the_budget() {
        let delta = AllocDelta::default();
        assert!(delta.is_clean(), "the empty beat moved nothing");
        assert!(delta.is_balanced(), "nothing allocated, nothing leaked");
        assert_eq!(delta.net_bytes(), 0, "the heap is where it was");
    }

    /// A balanced beat is transient churn: every block made inside the
    /// window was freed inside it — never a growing ledger.
    #[test]
    fn a_balanced_beat_is_churn_not_growth() {
        let delta = AllocDelta {
            allocations: 3,
            deallocations: 3,
            allocated_bytes: 512,
            freed_bytes: 512,
        };
        assert!(!delta.is_clean(), "churn is priced, not free");
        assert!(delta.is_balanced(), "every block was freed inside the beat");
        assert_eq!(delta.net_bytes(), 0, "the heap is where it was");
    }

    /// An unbalanced beat grew the heap: the ledger names the growth the
    /// budget refuses.
    #[test]
    fn an_unbalanced_beat_names_its_growth() {
        let delta = AllocDelta {
            allocations: 2,
            deallocations: 1,
            allocated_bytes: 288,
            freed_bytes: 96,
        };
        assert!(!delta.is_clean());
        assert!(!delta.is_balanced());
        assert_eq!(delta.net_bytes(), 192, "192 bytes the beat kept");
    }

    /// The ledger line names the answer plainly: the budget's own words
    /// when the beat is clean, the counted movement when it is not.
    #[test]
    fn the_display_line_names_the_beat() {
        assert_eq!(
            AllocDelta::default().to_string(),
            "nothing moved",
            "a clean beat prints the budget's answer"
        );
        let churn = AllocDelta {
            allocations: 1,
            deallocations: 1,
            allocated_bytes: 128,
            freed_bytes: 128,
        };
        assert_eq!(
            churn.to_string(),
            "1 blocks / 128 bytes allocated, 128 freed (net 0 bytes)",
            "a churned beat prints both sides of its movement"
        );
    }

    /// A freshly armed ledger starts at zero: its counters count what
    /// the window sees, and the window has seen nothing yet — the
    /// snapshot the beats' deltas bracket is honest from the first beat.
    #[test]
    fn a_fresh_ledgers_counters_start_at_zero() {
        let ledger = super::AllocLedger::arm();
        let stats = ledger.stats();
        assert_eq!(
            (stats.total_allocs, stats.total_bytes, stats.live_allocs),
            (0, 0, 0),
            "a fresh ledger already counts something"
        );
    }
}
