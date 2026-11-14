//! The text-write trail: how the AT's text writes — the input's write
//! door (§13.3, ADR-0061) — reach the causal log without any widget
//! holding a devtools dependency.
//!
//! The editing tier's AT actions (M11, ADR-0017) answered the widget's
//! own `bool`, whose `false` collapsed the closed door (M22) into the
//! same silence as an answered write — and the answered write itself
//! left no fact a trail reader could price. The ride the
//! M31/M33/M36/M39/M46 rhythm pre-registers cashes as the record
//! vocabulary ([`crate::inspect::TextWriteRecord`], carried in
//! [`crate::inspect`]) and the thread trail below, the value trail's
//! shape (ADR-0046): the widget notes the fact at the exact place the
//! answer resolves — the widget knows the action and the selection it
//! carries, the trail knows the drain — and the dispatching pipeline
//! drains it at the tick's start into
//! [`crate::inspect::InspectEvent::TextWrite`], dev-mode and bounded,
//! silent while nobody listens.
//!
//! The record names its control with the value trail's own stamp
//! (M43, ADR-0049): the dispatch funnel holds one
//! [`crate::value::ActionTargetGuard`] for the length of one action,
//! the AT tier's facts and the keyboard's stamping themselves with it
//! — and the text-write door's facts stamp from the same guard, the
//! routes shared, the identity never guessed at the note site.
//!
//! ```text
//! // Widgets (core-only dependencies):
//! use alacer_core::text_write::note_text_write;
//! note_text_write(alacer_core::TextWriteRecord { /* the write's fact */ });
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The text-write trail's cap: facts carried per thread before the
/// oldest evicts — the journal's own bounded rule (§14.3) applied to
/// its echo (a devtools log, not a frame-budget liability; the same
/// order the edit, announcement, and value trails ride).
pub const TEXT_WRITE_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per write (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`TEXT_WRITE_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::TextWriteRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the text-write trail (the pipeline's side of
/// the trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_text_write_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one text-write fact on this thread's trail (the editing
/// tier's side of the trail, ADR-0061): a no-op while the trail is
/// disabled, an oldest-evicting push while it rides. The input's AT
/// door calls this at the exact place the answer resolves — the widget
/// knows the action and the selection it carries, the trail knows the
/// drain — so a status line, an inspector trail, or a test reads the
/// editing tier without re-implementing the observation at the action
/// site. Since M43 (ADR-0049) a record whose `target` the widget left
/// `None` is stamped with the id the routing funnel held — the
/// control's identity riding beside the action's routing, never
/// guessed at the note site (the value trail's own stamp, the same
/// guard the AT tier's writes and the keyboard's steps stamp from).
/// The ask is never stamped: the widget's own door computed it (the
/// M44 rule), the record the caller built is the record the trail
/// carries — the stamp fills only the identity the widget cannot know.
pub fn note_text_write(record: crate::inspect::TextWriteRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        // The stamp side (M43, ADR-0049): the funnel's guard answers
        // the *who* the widget cannot know — the widget's own `None`
        // stays `None` when no route holds a guard (the widget doors'
        // honest absence).
        let mut record = record;
        if record.target.is_none() {
            record.target = crate::value::action_target();
        }
        tracing::trace!(
            target: "alacer::text_write",
            action = record.action.name(),
            outcome = record.outcome.name(),
            replaced = record.replaced,
            inserted = record.inserted,
            "text-write fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= TEXT_WRITE_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::TextWrite`].
pub(crate) fn take_text_write_trail() -> Vec<crate::inspect::TextWriteRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{TEXT_WRITE_TRAIL_CAP, note_text_write, take_text_write_trail};
    use crate::inspect::{TextWriteAction, TextWriteOutcome, TextWriteRecord, TextWriteRefusal};
    use crate::widget::TextSelection;

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the value trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_text_write(TextWriteRecord {
            action: TextWriteAction::SetValue,
            outcome: TextWriteOutcome::Delivered,
            replaced: Some(0),
            inserted: Some(3),
            requested: None,
            selection: TextSelection::new(3, 3),
            target: None,
        });
        assert!(
            take_text_write_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::text_write::set_text_write_trail_enabled(true);
        for index in 0..(TEXT_WRITE_TRAIL_CAP + 8) {
            let caret = index % 256;
            note_text_write(TextWriteRecord {
                action: TextWriteAction::SetSelection,
                outcome: TextWriteOutcome::Delivered,
                replaced: None,
                inserted: None,
                requested: Some(TextSelection::new(caret, caret)),
                selection: TextSelection::new(caret, caret),
                target: None,
            });
        }
        let drained = take_text_write_trail();
        assert_eq!(
            drained.len(),
            TEXT_WRITE_TRAIL_CAP,
            "the cap bounds the trail"
        );
        assert_eq!(drained[0].selection.focus, 8, "the oldest evicted first");
        assert!(
            take_text_write_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::text_write::set_text_write_trail_enabled(false);
    }

    /// The stamp side (M43, ADR-0049): a record the widget queued with
    /// a `None` target is stamped with the id the funnel's guard held —
    /// the control's identity riding beside the action's routing, never
    /// guessed at the note site. A record the caller named itself keeps
    /// its own answer, and a fact queued outside a guard rides its
    /// honest absence.
    #[test]
    fn the_funnel_stamp_names_the_control_and_the_absence_holds() {
        crate::text_write::set_text_write_trail_enabled(true);

        // Beneath a guard: the widget's `None` becomes the routed id,
        // the ask the caller carried untouched (the M44 rule — the
        // stamp fills only the identity the widget cannot know).
        let guard = crate::value::ActionTargetGuard::for_target(Some(0xAB_CDEF));
        note_text_write(TextWriteRecord {
            action: TextWriteAction::SetValue,
            outcome: TextWriteOutcome::Delivered,
            replaced: Some(2),
            inserted: Some(9),
            requested: None,
            selection: TextSelection::new(9, 9),
            target: None,
        });
        drop(guard);
        let drained = take_text_write_trail();
        assert_eq!(
            drained[0].target,
            Some(0xAB_CDEF),
            "the fact queued beneath the route names the control the action arrived under"
        );
        assert_eq!(
            drained[0].replaced,
            Some(2),
            "the caller's own shape rides untouched — the stamp never fills it"
        );

        // Outside a guard: the absence honest — the widget doors' rides.
        note_text_write(TextWriteRecord {
            action: TextWriteAction::SetSelection,
            outcome: TextWriteOutcome::Delivered,
            replaced: None,
            inserted: None,
            requested: Some(TextSelection::new(0, 4)),
            selection: TextSelection::new(0, 4),
            target: None,
        });
        // The caller's own answer is never overwritten.
        note_text_write(TextWriteRecord {
            action: TextWriteAction::ReplaceSelection,
            outcome: TextWriteOutcome::Refused(TextWriteRefusal::Unavailable),
            replaced: None,
            inserted: None,
            requested: None,
            selection: TextSelection::new(0, 0),
            target: Some(1),
        });
        let drained = take_text_write_trail();
        assert_eq!(
            drained[0].target, None,
            "a ride no funnel routed carries no target"
        );
        assert_eq!(
            drained[1].target,
            Some(1),
            "a record the caller named keeps its own answer"
        );

        // A dropped guard leaves no stale identity: the next fact is
        // unstamped.
        note_text_write(TextWriteRecord {
            action: TextWriteAction::SetValue,
            outcome: TextWriteOutcome::Delivered,
            replaced: Some(0),
            inserted: Some(1),
            requested: None,
            selection: TextSelection::new(1, 1),
            target: None,
        });
        let drained = take_text_write_trail();
        assert_eq!(
            drained[0].target, None,
            "the guard's drop cleared the stamp"
        );
        crate::text_write::set_text_write_trail_enabled(false);
    }
}
