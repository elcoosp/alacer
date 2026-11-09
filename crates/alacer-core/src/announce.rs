//! The announcement seam (§11.6, ADR-0024): the app's voice in the a11y
//! tree.
//!
//! Screen readers speak the widget tree — but state changes that own no
//! widget ("Saved", "3 items left", "Connection lost") had no route. An
//! announcement is a **reserved live-region node** in pass 9's snapshot:
//! the same snapshot, the same delta, the same `Change::A11Y` currency as
//! every other a11y fact (axiom A1 — no second channel). The AT watches
//! the live region and speaks its content; the politeness is the node's
//! `aria-live` priority.
//!
//! The contract's rules:
//!
//! - **The region is always present.** The reserved node joins the
//!   snapshot from the first frame with an empty name, so announcing
//!   changes one node's delta and never the tree's shape — no
//!   `STRUCTURE` churn, no removals, no ghost ids.
//! - **Last write wins.** Rapid successive announcements replace the
//!   region's content — one batch, one spoken fact, never an unbounded
//!   queue inside a frame. An app that needs sequencing announces once
//!   per state change.
//! - **Refuse, don't munge** (the M10 discipline): an empty or
//!   whitespace announcement is refused (`announce` returns `false`),
//!   and an announcement before a root exists is refused too — there is
//!   no tree to attach the region to, so there is nothing honest to
//!   ship. A refusal queues no masks: an idle frame.
//!
//! Two doors, one destination. Assembly sites and tests hold the pipeline
//! and call [`PassPipeline::announce`] directly; event handlers cannot
//! (widgets never touch the pipeline, axiom A1), so the ambient
//! [`announce`] fn queues the speech act on the calling thread and the
//! dispatching pipeline drains it at the next tick's start — the input
//! queue's shape (ADR-0014): push now, the tick routes, one frame.
//!
//! [`PassPipeline::announce`]: crate::passes::PassPipeline::announce
//! [`announce`]: announce

use std::cell::RefCell;

/// The reserved live-region node's identity: a tagged constant, not an
/// id-path fold — arena paths fold through [`crate::identity`]'s `FxHasher`
/// over short segment chains, and this ASCII marker ("ANNOUNCE") sits
/// outside every fold a real element path can produce. One node, one id,
/// stable for the process lifetime.
pub const ANNOUNCEMENT_NODE_HASH: u64 = 0x414E_4E4F_554E_4345; // "ANNOUNCE"

/// How urgently an assistive technology should speak the announcement
/// (ARIA `aria-live`, mirrored in core so the DAG stays adapter-free —
/// the same shape [`crate::widget::ScrollUnit`] rides).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Politeness {
    /// Speak at the next break — "Saved", counters, status lines.
    Polite,
    /// Interrupt now — "Connection lost", destructive-action confirmations.
    Assertive,
}

impl Politeness {
    /// Human-readable politeness, used by tracing fields and diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Polite => "polite",
            Self::Assertive => "assertive",
        }
    }
}

/// An announcement awaiting its delta: the text the AT speaks and the
/// priority it carries. Stored on the pipeline, never on a widget — the
/// region is the app's, not a widget's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
    /// The text the assistive technology speaks.
    pub text: String,
    /// The `aria-live` priority.
    pub politeness: Politeness,
}

thread_local! {
    /// The speech acts queued since the last tick drain — the handler side
    /// of the seam. Event handlers cannot hold the pipeline (widgets never
    /// touch it, axiom A1), so the ambient route queues like the input
    /// queue does (ADR-0014's shape) and the dispatching pipeline drains
    /// the last act at the next tick's start: a handler's speech act rides
    /// the same one-frame currency as its signal writes.
    static PENDING: RefCell<Vec<Announcement>> = const { RefCell::new(Vec::new()) };
}

/// Announces `text` to assistive technologies from an event handler (§11.6,
/// ADR-0024) — the clipboard-seam shape (ADR-0020): handlers cannot reach
/// the pipeline, so the speech act queues into this thread's pending slot
/// and the dispatching pipeline drains it at the next tick's start, into
/// the same reserved region [`PassPipeline::announce`] writes directly.
///
/// An empty or all-whitespace `text` speaks nothing and is refused here
/// (`false`, nothing queued). The queue accepts when a pipeline is absent —
/// the drain, not the push, is where a missing tree refuses (the same
/// shape as an input queued for a node that is gone by the tick).
///
/// [`PassPipeline::announce`]: crate::passes::PassPipeline::announce
pub fn announce(text: impl Into<String>, politeness: Politeness) -> bool {
    let announcement = Announcement {
        text: text.into(),
        politeness,
    };
    if announcement.text.trim().is_empty() {
        tracing::trace!(
            target: "alacer::announce",
            "refused: an empty announcement speaks nothing"
        );
        return false;
    }
    PENDING.with(|slot| {
        tracing::trace!(
            target: "alacer::announce",
            politeness = announcement.politeness.name(),
            chars = announcement.text.len(),
            "announcement queued for the next tick"
        );
        slot.borrow_mut().push(announcement);
    });
    true
}

/// The dispatching pipeline's drain: the last speech act queued on this
/// thread since the last tick — the whole batch clears, last write wins
/// (one region, one spoken fact), never an unbounded queue inside a frame
/// and never a stale act leaking into a later tick.
pub(crate) fn take_pending() -> Option<Announcement> {
    PENDING.with(|slot| {
        let mut slot = slot.borrow_mut();
        let last = slot.pop();
        slot.clear();
        last
    })
}

#[cfg(test)]
mod tests {
    use super::{ANNOUNCEMENT_NODE_HASH, Announcement, Politeness, announce, take_pending};

    #[test]
    fn politeness_names_itself() {
        assert_eq!(Politeness::Polite.name(), "polite");
        assert_eq!(Politeness::Assertive.name(), "assertive");
    }

    #[test]
    fn the_reserved_hash_is_the_documented_marker() {
        assert_eq!(ANNOUNCEMENT_NODE_HASH, 0x414E_4E4F_554E_4345);
        let announcement = Announcement {
            text: "Saved".to_string(),
            politeness: Politeness::Polite,
        };
        assert_eq!(announcement.text, "Saved");
    }

    #[test]
    fn the_queued_route_refuses_the_empty_and_drains_last_write_wins() {
        // Empty text speaks nothing: refused here, nothing queued.
        assert!(!announce("", Politeness::Polite));
        assert!(!announce("  ", Politeness::Polite));
        assert!(take_pending().is_none(), "a refusal queues nothing");

        // Two queued acts, one drain, last write wins (the replace rule).
        assert!(announce("Saved", Politeness::Polite));
        assert!(announce("3 items left", Politeness::Assertive));
        let drained = take_pending().expect("the batch's last act");
        assert_eq!(drained.text, "3 items left");
        assert_eq!(drained.politeness, Politeness::Assertive);
        assert!(take_pending().is_none(), "the batch drains to one act");
    }
}
