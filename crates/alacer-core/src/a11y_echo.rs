//! The a11y-echo trail: how the adapter's full-tree echo — the
//! activation response the a11y tier answers when an assistive
//! technology connects or reconnects (§13.3, ADR-0073) — reaches the
//! causal log without the a11y sink holding a devtools dependency.
//!
//! The kill rule M59 (ADR-0065) pre-registered: the adapter activation's
//! timing and the a11y sink's full-tree fallbacks were traced and gone —
//! a trail reader could not answer "why did the app just push the whole
//! tree, and how big was it" from the log alone. The tier grows its own
//! record now ([`crate::inspect::A11yEchoRecord`], carried in
//! [`crate::inspect`]) and the thread trail below, the route-drop
//! trail's shape (ADR-0063): the note site is the exact place the echo
//! resolves — the sink's full-tree build, the same builder the deltas
//! use — and the dispatching pipeline drains it at the tick's start,
//! ahead of the frame's routes (a connect predates the asks the frame
//! carries), into [`crate::inspect::InspectEvent::A11yEcho`], dev-mode
//! and bounded, silent while nobody listens.
//!
//! The record carries no funnel stamp (the M43 rule does not apply): an
//! echo's identity *is* the tree the build answered — the node count
//! rides, the payload itself never does (the lengths rule at every
//! tier, the route-drop trail's own). And the flood rule prices the
//! tier's shape: a connect is rare — one fact per build, never a
//! frame-rate companion, the trail quiet between connects.
//!
//! ```text
//! // The sink (the a11y tier) queues where the build resolves:
//! use alacer_core::a11y_echo::note_a11y_echo;
//! note_a11y_echo(alacer_core::A11yEchoRecord { /* the echo's fact */ });
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The a11y-echo trail's cap: facts carried per thread before the
/// oldest evicts — the journal's own bounded rule (§14.3) applied to
/// its echo (a devtools log, not a frame-budget liability; the same
/// order the route-drop and key-edit trails ride). A burst of
/// reconnects never queues more than the cap: the oldest echo evicts,
/// the newest fact wins.
pub const A11Y_ECHO_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per echo (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`A11Y_ECHO_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::A11yEchoRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the a11y-echo trail (the pipeline's side of
/// the trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_a11y_echo_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one a11y-echo fact on this thread's trail (the a11y tier's
/// side of the trail, ADR-0073): a no-op while the trail is disabled,
/// an oldest-evicting push while it rides. The echo site — the sink's
/// full-tree build, the activation response the adapter forwards —
/// calls this at the exact place the echo resolves, so a status line,
/// an inspector trail, or a test reads the tier without re-implementing
/// the observation at the connect site. The record rides as built: the
/// node count is the tree the build answered, verbatim — the trail
/// never re-counts it against the snapshot (a re-counted size would be
/// the snapshot's answer, not the echo's).
pub fn note_a11y_echo(record: crate::inspect::A11yEchoRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        tracing::trace!(
            target: "alacer::a11y_echo",
            nodes = record.nodes,
            "a11y-echo fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= A11Y_ECHO_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::A11yEcho`].
pub(crate) fn take_a11y_echo_trail() -> Vec<crate::inspect::A11yEchoRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{
        A11Y_ECHO_TRAIL_CAP, note_a11y_echo, set_a11y_echo_trail_enabled, take_a11y_echo_trail,
    };
    use crate::inspect::A11yEchoRecord;

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the route-drop trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_a11y_echo(A11yEchoRecord { nodes: 12 });
        assert!(
            take_a11y_echo_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once. The record rides as
    /// built — the node count the note site named is the count the
    /// trail carries, never re-counted against the snapshot.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        set_a11y_echo_trail_enabled(true);
        for nodes in 0..(A11Y_ECHO_TRAIL_CAP + 8) {
            note_a11y_echo(A11yEchoRecord { nodes });
        }
        let drained = take_a11y_echo_trail();
        assert_eq!(
            drained.len(),
            A11Y_ECHO_TRAIL_CAP,
            "the cap bounds the trail"
        );
        assert_eq!(drained[0].nodes, 8, "the oldest evicted first");
        assert!(
            take_a11y_echo_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        set_a11y_echo_trail_enabled(false);
    }

    /// The record rides the count its note site declared: the trail
    /// never re-counts the tree against the snapshot — the echo's
    /// identity is the build's own answer (ADR-0073's rule).
    #[test]
    fn the_record_rides_the_count_its_note_site_declared() {
        set_a11y_echo_trail_enabled(true);
        note_a11y_echo(A11yEchoRecord { nodes: 7 });
        note_a11y_echo(A11yEchoRecord { nodes: 1_024 });
        let drained = take_a11y_echo_trail();
        assert_eq!(drained[0].nodes, 7, "the first echo's own size");
        assert_eq!(
            drained[1].nodes, 1_024,
            "the reconnect's echo names its own tree, verbatim"
        );
        set_a11y_echo_trail_enabled(false);
    }
}
