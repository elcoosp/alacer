//! The preedit trail: how the field's composing state — the preedit
//! the IME shows while it owns the canvas (§13.3, ADR-0067) — reaches
//! the causal log without any widget holding a devtools dependency.
//!
//! The kill rule ADR-0064 pre-registered: a consumer prices the
//! *field's composing state* — the preedit's own trail, the
//! composition's inline shape — on the log → the IME tier grows its
//! own record beside the commit's. The commit's landing already
//! streams ([`crate::inspect::KeyEditRecord`], the IME driver named,
//! ADR-0064); the composing state itself — what the field showed while
//! the IME composed, how large it grew, where the platform's cursor
//! sat inside it — was traced and gone. The preedit is not an edit: it
//! lands nowhere, the journal never hears of it, and the vocabulary
//! ([`crate::inspect::PreeditRecord`], carried in [`crate::inspect`])
//! never munges a composition onto the edit tier's actions. The thread
//! trail below is the key-edit trail's shape (ADR-0064 — the
//! text-write trail's rhythm): the widget notes the fact at the exact
//! place the composing state resolves, and the dispatching pipeline
//! drains it at the tick's start into
//! [`crate::inspect::InspectEvent::Preedit`], dev-mode and bounded,
//! silent while nobody listens.
//!
//! The flood rule is the session's own supersede: a live composition
//! is one record — N composition frames supersede the session's
//! record in place (the same control, the composing phase), the shape
//! growing to the composition's — never a default-on echo of every
//! composition frame. A cleared composing state (the empty preedit the
//! platform delivers when the session dissolves with no landing)
//! answers its own [`crate::inspect::PreeditAction::Cleared`] fact; a
//! commit's landing streams on the key-edit trail beside it — one fact
//! per trail, the trails tellable apart on one log.
//!
//! The record names its control with the value trail's own stamp
//! (M43, ADR-0049): the IME route has held one
//! `crate::value::ActionTargetGuard` per event since M58, the
//! composing door's facts stamping themselves with it.
//!
//! ```text
//! // Widgets (core-only dependencies):
//! use alacer_core::preedit::note_preedit;
//! note_preedit(alacer_core::PreeditRecord { /* the composing fact */ });
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The preedit trail's cap: facts carried per thread before the oldest
/// evicts — the journal's own bounded rule (§14.3) applied to its echo
/// (a devtools log, not a frame-budget liability; the same order the
/// key-edit, edit, announcement, value, and text-write trails ride).
pub const PREEDIT_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per preedit event (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`PREEDIT_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::PreeditRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the preedit trail (the pipeline's side of the
/// trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_preedit_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one preedit fact on this thread's trail (the composing door's
/// side of the trail, ADR-0067): a no-op while the trail is disabled,
/// an oldest-evicting push while it rides. The field's composing door
/// calls this at the exact place the composing state resolves — the
/// widget knows the shape and the cursor it shows, the trail knows the
/// drain — so a status line, an inspector trail, or a test reads the
/// composition without re-implementing the observation at the IME
/// site. Since M43 (ADR-0049) a record whose `target` the widget left
/// `None` is stamped with the id the routing funnel held — the
/// control's identity riding beside the event's routing, never guessed
/// at the note site.
///
/// The flood rule (ADR-0067, the session's own supersede): a composing
/// arrival superseding a live session's record — the same control, the
/// composing phase — grows that record in place: N composition frames
/// while nobody drained land as one record whose shape is the
/// composition's, never a default-on echo of every composition frame.
/// A cleared arrival pushes its own fact; a `Composing` fact beside a
/// `Cleared` one is a new session's own record.
pub fn note_preedit(record: crate::inspect::PreeditRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        // The stamp side (M43, ADR-0049): the funnel's guard answers
        // the *who* the widget cannot know — the widget's own `None`
        // stays `None` when no route holds a guard.
        let mut record = record;
        if record.target.is_none() {
            record.target = crate::value::action_target();
        }
        tracing::trace!(
            target: "alacer::preedit",
            action = record.action.name(),
            shape = record.shape,
            cursor = ?record.cursor,
            "preedit fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            // The flood rule's supersede: the session is live and this
            // arrival composes into it — the last record is the same
            // control's live session, and the composition's shape
            // grows it in place. One live session, one record.
            if record.action == crate::inspect::PreeditAction::Composing
                && let Some(last) = queue.back_mut()
                && last.action == crate::inspect::PreeditAction::Composing
                && last.target == record.target
            {
                last.shape = record.shape;
                last.cursor = record.cursor;
                return;
            }
            if queue.len() >= PREEDIT_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::Preedit`].
pub(crate) fn take_preedit_trail() -> Vec<crate::inspect::PreeditRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{PREEDIT_TRAIL_CAP, note_preedit, take_preedit_trail};
    use crate::inspect::{PreeditAction, PreeditRecord};

    /// A composing fact: the shape the composition shows, the
    /// platform's cursor inside it.
    fn composing(shape: usize, cursor: Option<usize>) -> PreeditRecord {
        PreeditRecord {
            action: PreeditAction::Composing,
            shape,
            cursor,
            target: None,
        }
    }

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the key-edit trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_preedit(composing(1, Some(1)));
        assert!(
            take_preedit_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once. The arrivals alternate
    /// phases — a composing fact supersedes its session in place, so a
    /// cleared fact between them is what makes each arrival its own
    /// record for the cap to bound.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::preedit::set_preedit_trail_enabled(true);
        for shape in 0..(PREEDIT_TRAIL_CAP + 8) {
            note_preedit(composing(shape, Some(shape)));
            note_preedit(PreeditRecord {
                action: PreeditAction::Cleared,
                shape: 0,
                cursor: None,
                target: None,
            });
        }
        let drained = take_preedit_trail();
        assert_eq!(drained.len(), PREEDIT_TRAIL_CAP, "the cap bounds the trail");
        // 264 pairs = 528 facts; 272 evicted oldest-first. The first
        // survivor is pair 136's composing fact (shape 136), its
        // cleared partner riding beside it.
        assert_eq!(drained[0].action, PreeditAction::Composing);
        assert_eq!(
            drained[0].shape, 136,
            "the oldest evicted first (the surviving session's shape)"
        );
        assert_eq!(drained[1].action, PreeditAction::Cleared);
        assert!(
            take_preedit_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::preedit::set_preedit_trail_enabled(false);
    }

    /// The flood rule (ADR-0067): a live composition is one record —
    /// N composition frames supersede the session's record in place,
    /// the shape and cursor growing to the composition's.
    #[test]
    fn the_live_session_is_one_record() {
        crate::preedit::set_preedit_trail_enabled(true);
        note_preedit(composing(1, Some(1)));
        note_preedit(composing(2, Some(2)));
        note_preedit(composing(3, Some(3)));
        let drained = take_preedit_trail();
        assert_eq!(
            drained.len(),
            1,
            "N composition frames are one record, never an echo per frame"
        );
        assert_eq!(drained[0].shape, 3, "the record's shape is the session's");
        assert_eq!(
            drained[0].cursor,
            Some(3),
            "the cursor rides as the last frame carried it"
        );
        crate::preedit::set_preedit_trail_enabled(false);
    }

    /// A cleared arrival pushes its own fact beside the session's: the
    /// session's end names itself, and a composing fact beside a
    /// cleared one is a new session's own record — the supersede never
    /// crosses the phase boundary.
    #[test]
    fn the_cleared_fact_pushes_and_the_new_session_opens() {
        crate::preedit::set_preedit_trail_enabled(true);
        note_preedit(composing(2, Some(2)));
        note_preedit(PreeditRecord {
            action: PreeditAction::Cleared,
            shape: 0,
            cursor: None,
            target: None,
        });
        note_preedit(composing(1, Some(1)));
        let drained = take_preedit_trail();
        assert_eq!(drained.len(), 3, "the phases stay their own facts");
        assert_eq!(drained[0].action, PreeditAction::Composing);
        assert_eq!(drained[1].action, PreeditAction::Cleared);
        assert_eq!(drained[2].action, PreeditAction::Composing);
        assert_eq!(drained[2].shape, 1, "the new session opens its own record");
        crate::preedit::set_preedit_trail_enabled(false);
    }

    /// The supersede stays within the control: another control's
    /// composing session is never merged into the first field's record
    /// — the stamp's identity holds the sessions apart.
    #[test]
    fn the_supersede_stays_within_the_control() {
        crate::preedit::set_preedit_trail_enabled(true);
        note_preedit(composing(1, Some(1)));
        note_preedit(PreeditRecord {
            action: PreeditAction::Composing,
            shape: 2,
            cursor: Some(2),
            target: Some(0xBEEF),
        });
        let drained = take_preedit_trail();
        assert_eq!(
            drained.len(),
            2,
            "different controls never merge — the stamp's identity holds"
        );
        assert_eq!(drained[0].target, None, "the unstamped session");
        assert_eq!(drained[1].target, Some(0xBEEF), "the routed session");
        crate::preedit::set_preedit_trail_enabled(false);
    }

    /// The stamp side (M43, ADR-0049): a record the widget queued with
    /// a `None` target is stamped with the id the funnel's guard held,
    /// and a dropped guard leaves no stale identity.
    #[test]
    fn the_funnel_stamp_names_the_control_and_the_absence_holds() {
        crate::preedit::set_preedit_trail_enabled(true);
        let guard = crate::value::ActionTargetGuard::for_target(Some(0xAB_CDEF));
        note_preedit(composing(1, Some(1)));
        drop(guard);
        let drained = take_preedit_trail();
        assert_eq!(
            drained[0].target,
            Some(0xAB_CDEF),
            "the fact queued beneath the route names the control it rode"
        );
        note_preedit(composing(1, Some(1)));
        let drained = take_preedit_trail();
        assert_eq!(
            drained[0].target, None,
            "a ride no funnel routed carries no target"
        );
        crate::preedit::set_preedit_trail_enabled(false);
    }

    /// The vocabulary carries its stable tags — the wire vocabulary's
    /// `snake_case` rule, the names the protocol document pins.
    #[test]
    fn the_actions_carry_their_stable_tags() {
        assert_eq!(PreeditAction::Composing.name(), "composing");
        assert_eq!(PreeditAction::Cleared.name(), "cleared");
    }
}
