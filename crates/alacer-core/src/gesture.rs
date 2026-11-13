//! The gesture trail: how the value tier's drag — the pointer's own
//! write (§13.3, ADR-0052) — reaches the causal log without any widget
//! holding a devtools dependency.
//!
//! ADR-0046 priced the value tier's AT writes and ADR-0051 priced the
//! keyboard's steps, but the drag stayed off the log: every pointer
//! move inside a drag would have flooded the trail (the second kill
//! rule's own words), so the value's motion went unexplained — a trail
//! reader could not answer "why did the slider's value move while the
//! hand dragged it". The price the kill rule (and ADR-0051's kill arm)
//! names is the per-*gesture* record: one [`crate::inspect::GestureRecord`]
//! when the press arms the drag, one when the release or the leave
//! resolves it — the phases the record's own vocabulary names, the
//! flood rule holding by construction (one hundred moves stream two
//! records).
//!
//! The trail is the value trail's shape (ADR-0046): the widget notes
//! the fact at the exact place the answer resolves — the drag's door
//! knows the phase and the value it holds, the trail knows the drain —
//! and the dispatching pipeline drains it at the tick's start into
//! [`crate::inspect::InspectEvent::Gesture`], dev-mode and bounded,
//! silent while nobody listens.
//!
//! Since M46 the record names its control too (ADR-0052, the M43/M45
//! stamp discipline at the pointer tier): the dispatch funnel's pointer
//! route knows the deepest hit that captured (ADR-0025), the widget
//! does not — so the stamp side lives here (`GESTURE_TARGET`, held by
//! the route for the length of one routed event) and [`note_gesture`]
//! stamps the facts queued beneath a routed pointer event with it,
//! never a node id guessed at the note site.
//!
//! ```text
//! // Widgets (core-only dependencies):
//! use alacer_core::gesture::note_gesture;
//! note_gesture(alacer_core::inspect::GestureRecord { /* the drag's fact */ });
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The gesture trail's cap: facts carried per thread before the oldest
/// evicts — the value trail's own bounded rule (§14.3) applied to its
/// sibling (a devtools log, not a frame-budget liability; the same
/// order the edit, announcement, clipboard, and value trails ride).
pub const GESTURE_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per gesture phase (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`GESTURE_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::GestureRecord>> =
        const { RefCell::new(VecDeque::new()) };
    /// The control the dispatch funnel's pointer route is currently
    /// resolving (M46, ADR-0052): `Some(id)` while a routed pointer
    /// event holds its guard, `None` otherwise — the facts queued
    /// beneath the route carry the deepest hit's id, the facts queued
    /// outside one ride their absence.
    static GESTURE_TARGET: Cell<Option<u64>> = const { Cell::new(None) };
}

/// Enables or disables the gesture trail (the pipeline's side of the
/// trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_gesture_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Whether the trail notes facts on this thread — the pointer route's
/// idle-contract check (M46, ADR-0052): the stamp's id-path walk runs
/// only under an observer, while nobody listens the stamp is one
/// `Cell` read answered here (the A7 idle contract, one branch per
/// routed pointer event).
pub(crate) fn gesture_trail_enabled() -> bool {
    TRAIL_ENABLED.with(Cell::get)
}

/// The funnel's stamp guard (M46, ADR-0052): the pointer route holds
/// one for the length of one routed event — the walks the route runs —
/// the facts queued beneath it carrying the id of the control the route
/// resolved (the deepest hit that captured, ADR-0025's own answer; the
/// widget never guesses its own identity, the M43 rule's words at the
/// pointer tier). Dropping clears the stamp — a panicking widget leaves
/// no stale identity on the thread.
pub(crate) struct GestureTargetGuard {
    previous: Option<u64>,
}

impl GestureTargetGuard {
    /// Stamps `target` as the control the pointer route is resolving,
    /// restoring the previous stamp on drop (the route nests never —
    /// one guard per routed event — but a nested route would still
    /// answer honestly).
    pub(crate) fn for_target(target: Option<u64>) -> Self {
        let previous = GESTURE_TARGET.with(|slot| {
            let previous = slot.get();
            slot.set(target);
            previous
        });
        Self { previous }
    }
}

impl Drop for GestureTargetGuard {
    fn drop(&mut self) {
        GESTURE_TARGET.with(|slot| slot.set(self.previous));
    }
}

/// Notes one gesture fact on this thread's trail (the drag's side of
/// the trail, ADR-0052): a no-op while the trail is disabled, an
/// oldest-evicting push while it rides. The drag's door calls this at
/// the exact place the phase resolves — the press that arms the drag,
/// the release or the leave that answers it — so a status line, an
/// inspector trail, or a test reads the drag's story without
/// re-implementing the observation at the gesture site, and every
/// per-move fact stays off the log (the flood rule: the moves note
/// nothing, the phases note once). Since M46 (ADR-0052) a record whose
/// `target` the widget left `None` is stamped with the id the pointer
/// route held — the deepest hit that captured, never guessed at the
/// note site.
pub fn note_gesture(record: crate::inspect::GestureRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        // The stamp side (M46, ADR-0052): the funnel's guard answers
        // the *who* the widget cannot know — the widget's own `None`
        // stays `None` when no route holds a guard (the honest
        // absence).
        let mut record = record;
        if record.target.is_none() {
            record.target = GESTURE_TARGET.with(Cell::get);
        }
        tracing::trace!(
            target: "alacer::gesture",
            phase = record.phase.name(),
            from = record.from,
            "gesture fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= GESTURE_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::Gesture`].
pub(crate) fn take_gesture_trail() -> Vec<crate::inspect::GestureRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{GESTURE_TRAIL_CAP, note_gesture, take_gesture_trail};
    use crate::inspect::{GesturePhase, GestureRecord};

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the value trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_gesture(GestureRecord {
            phase: GesturePhase::Started,
            from: 20.0,
            to: None,
            target: None,
        });
        assert!(
            take_gesture_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::gesture::set_gesture_trail_enabled(true);
        for index in 0..(GESTURE_TRAIL_CAP + 8) {
            let from = f32::from(u16::try_from(index % 256).expect("in range"));
            note_gesture(GestureRecord {
                phase: GesturePhase::Ended,
                from,
                to: Some(from + 1.0),
                target: None,
            });
        }
        let drained = take_gesture_trail();
        assert_eq!(drained.len(), GESTURE_TRAIL_CAP, "the cap bounds the trail");
        assert_eq!(drained[0].from, 8.0, "the oldest evicted first");
        assert!(
            take_gesture_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::gesture::set_gesture_trail_enabled(false);
    }

    /// The stamp side (M46, ADR-0052): a record the widget queued with
    /// a `None` target is stamped with the id the pointer route's guard
    /// held — the deepest hit that captured, never guessed at the note
    /// site. A record the caller named itself keeps its own answer, and
    /// a fact queued outside a guard rides its honest absence.
    #[test]
    fn the_pointer_route_stamp_names_the_control_and_the_absence_holds() {
        crate::gesture::set_gesture_trail_enabled(true);

        // Beneath a guard: the widget's `None` becomes the routed id —
        // the control the funnel resolved, not one the widget guessed.
        let guard = crate::gesture::GestureTargetGuard::for_target(Some(0xAB_CDEF));
        note_gesture(GestureRecord {
            phase: GesturePhase::Started,
            from: 20.0,
            to: None,
            target: None,
        });
        drop(guard);
        let drained = take_gesture_trail();
        assert_eq!(
            drained[0].target,
            Some(0xAB_CDEF),
            "the fact queued beneath the route names the control the gesture rides"
        );
        assert_eq!(
            drained[0].from, 20.0,
            "the caller's own fact rides untouched — the stamp fills only the identity"
        );

        // Outside a guard: the absence honest — the trail's own rule.
        note_gesture(GestureRecord {
            phase: GesturePhase::Ended,
            from: 20.0,
            to: Some(42.0),
            target: None,
        });
        // The caller's own answer is never overwritten.
        note_gesture(GestureRecord {
            phase: GesturePhase::Ended,
            from: 0.0,
            to: Some(1.0),
            target: Some(1),
        });
        let drained = take_gesture_trail();
        assert_eq!(
            drained[0].target, None,
            "a fact no funnel routed carries no target"
        );
        assert_eq!(
            drained[1].target,
            Some(1),
            "a record the caller named keeps its own answer"
        );

        // A dropped guard leaves no stale identity: the next fact is
        // unstamped.
        note_gesture(GestureRecord {
            phase: GesturePhase::Started,
            from: 5.0,
            to: None,
            target: None,
        });
        let drained = take_gesture_trail();
        assert_eq!(
            drained[0].target, None,
            "the guard's drop cleared the stamp"
        );
        crate::gesture::set_gesture_trail_enabled(false);
    }

    /// The phases name themselves: the wire's stable tags ride the
    /// record (the protocol's `snake_case` rule).
    #[test]
    fn the_phases_name_themselves() {
        assert_eq!(GesturePhase::Started.name(), "started");
        assert_eq!(GesturePhase::Ended.name(), "ended");
    }
}
