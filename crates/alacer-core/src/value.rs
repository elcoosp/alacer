//! The value tier's trail: how the AT's value writes — the `Slider`'s
//! and the `Progress`'s answers (§13.3, ADR-0046) — reach the causal
//! log without any widget holding a devtools dependency.
//!
//! The value tier's AT actions answered the widget's own `bool`, whose
//! `false` collapsed four different facts — the control the
//! availability tier holds down, the payload that parsed to nothing,
//! the payload the funnel cannot carry, the control that never had a
//! write door — into one silence. The ride the M31/M33/M36/M39 rhythm
//! pre-registers cashes as the record vocabulary
//! ([`crate::inspect::ValueRecord`], carried in [`crate::inspect`])
//! and the thread trail below, the clipboard trail's shape (ADR-0045):
//! the widget notes the fact at the exact place the answer resolves —
//! the widget knows the action and the value it holds, the trail knows
//! the drain — and the dispatching pipeline drains it at the tick's
//! start into [`crate::inspect::InspectEvent::Value`], dev-mode and
//! bounded, silent while nobody listens.
//!
//! Since M43 the record names its control too (ADR-0049, ADR-0046's
//! first re-open rule): the dispatch funnel knows the id the action
//! arrived under, the widget does not — so the stamp side lives here
//! (`ACTION_TARGET`, held by the funnel's route for the length of
//! one action) and [`note_write`] stamps the facts queued beneath a
//! routed action with it, never a node id guessed at the note site.
//! Since M45 (ADR-0051, ADR-0049's first kill rule) the key route
//! holds the same stamp: the focused control's arrows ride the log
//! attributed, the AT tier's rides and the keyboard's tellable apart
//! by nothing but the route that routed them — the same record, the
//! same stamp discipline.
//!
//! Since M49 the record names the wall the funnel pinned the landing
//! at too (ADR-0055, ADR-0053's second kill rule): a write the range
//! turned back at its edge — the clamp, the snap, the wall that held a
//! no-op against a past ask — carries the bound beside the species,
//! the ask's own wall named where the knowledge lives (the widget's
//! own door computes it, the trail never reconstructs it). The ask
//! that wanted the wall exactly (the M47 jumps) rides the honest
//! absence: the species already names the bound it asked for.
//!
//! ```text
//! // Widgets (core-only dependencies):
//! use alacer_core::value::note_write;
//! note_write(alacer_core::inspect::ValueRecord { /* the write's fact */ });
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The value trail's cap: facts carried per thread before the oldest
/// evicts — the journal's own bounded rule (§14.3) applied to its echo
/// (a devtools log, not a frame-budget liability; the same order the
/// edit, announcement, and clipboard trails ride).
pub const VALUE_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per write (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`VALUE_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::ValueRecord>> =
        const { RefCell::new(VecDeque::new()) };
    /// The action the dispatch funnel is currently routing (M43,
    /// ADR-0049): `Some(id)` while the AT action route holds its guard,
    /// `None` otherwise — the facts queued beneath the action carry the
    /// id, the facts queued outside one ride their absence.
    static ACTION_TARGET: Cell<Option<u64>> = const { Cell::new(None) };
}

/// Enables or disables the value trail (the pipeline's side of the
/// trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_value_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Whether the trail notes facts on this thread — the key route's
/// idle-contract check (M45, ADR-0051): the stamp's id-path walk runs
/// only under an observer, while nobody listens the stamp is one
/// `Cell` read answered here (the A7 idle contract, one branch per
/// routed key).
pub(crate) fn value_trail_enabled() -> bool {
    TRAIL_ENABLED.with(Cell::get)
}

/// The funnel's stamp guard (M43, ADR-0049): the AT action route holds
/// one for the length of one widget call, the facts queued beneath the
/// action carrying the id the action arrived under — the widget never
/// guesses it (the rule's own words: the dispatch funnel knows the
/// target node, the widget does not). Dropping clears the stamp — a
/// panicking widget leaves no stale identity on the thread.
pub(crate) struct ActionTargetGuard {
    previous: Option<u64>,
}

impl ActionTargetGuard {
    /// Stamps `target` as the action the thread is routing, restoring
    /// the previous stamp on drop (the funnel nests never — one guard
    /// per routed action — but a nested route would still answer
    /// honestly).
    pub(crate) fn for_target(target: Option<u64>) -> Self {
        let previous = ACTION_TARGET.with(|slot| {
            let previous = slot.get();
            slot.set(target);
            previous
        });
        Self { previous }
    }
}

impl Drop for ActionTargetGuard {
    fn drop(&mut self) {
        ACTION_TARGET.with(|slot| slot.set(self.previous));
    }
}

/// Notes one value fact on this thread's trail (the value tier's side
/// of the trail, ADR-0046): a no-op while the trail is disabled, an
/// oldest-evicting push while it rides. The value-tier widgets call
/// this at the exact place the answer resolves — the widget knows the
/// action and the value it holds, the trail knows the drain — so a
/// status line, an inspector trail, or a test reads the value tier
/// without re-implementing the observation at the action site. Since
/// M43 (ADR-0049) a record whose `target` the widget left `None` is
/// stamped with the id the routing funnel held — the control's
/// identity riding beside the action's routing, never guessed at the
/// note site. Since M45 (ADR-0051) the key route holds the same stamp,
/// the keyboard's rides carrying the focused control's id. The ask is
/// never stamped: the widget's own door computed it (M44, ADR-0050),
/// the record the caller built is the record the trail carries — the
/// stamp fills only the identity the widget cannot know.
pub fn note_write(record: crate::inspect::ValueRecord) {
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
            record.target = ACTION_TARGET.with(Cell::get);
        }
        tracing::trace!(
            target: "alacer::value",
            action = record.action.name(),
            outcome = record.outcome.name(),
            value = record.value,
            "value fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= VALUE_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::Value`].
pub(crate) fn take_value_trail() -> Vec<crate::inspect::ValueRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{VALUE_TRAIL_CAP, note_write, take_value_trail};
    use crate::inspect::{ValueAction, ValueOutcome, ValueRecord, ValueRefusal};

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the clipboard trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_write(ValueRecord {
            action: ValueAction::SetValue,
            outcome: ValueOutcome::Delivered,
            value: 42.0,
            requested: None,
            target: None,
            pinned: None,
        });
        assert!(
            take_value_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::value::set_value_trail_enabled(true);
        for index in 0..(VALUE_TRAIL_CAP + 8) {
            let value = f32::from(u16::try_from(index % 256).expect("in range"));
            note_write(ValueRecord {
                action: ValueAction::Increment,
                outcome: ValueOutcome::Refused(ValueRefusal::NoWriteDoor),
                value,
                requested: None,
                target: None,
                pinned: None,
            });
        }
        let drained = take_value_trail();
        assert_eq!(drained.len(), VALUE_TRAIL_CAP, "the cap bounds the trail");
        assert_eq!(drained[0].value, 8.0, "the oldest evicted first");
        assert!(
            take_value_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::value::set_value_trail_enabled(false);
    }

    /// The stamp side (M43, ADR-0049): a record the widget queued with
    /// a `None` target is stamped with the id the funnel's guard held —
    /// the control's identity riding beside the action's routing, never
    /// guessed at the note site. A record the caller named itself keeps
    /// its own answer, and a fact queued outside a guard rides its
    /// honest absence.
    #[test]
    fn the_funnel_stamp_names_the_control_and_the_absence_holds() {
        crate::value::set_value_trail_enabled(true);

        // Beneath a guard: the widget's `None` becomes the routed id,
        // the ask the caller carried untouched (M44, ADR-0050 — the
        // stamp fills only the identity the widget cannot know).
        let guard = crate::value::ActionTargetGuard::for_target(Some(0xAB_CDEF));
        note_write(ValueRecord {
            action: ValueAction::SetValue,
            outcome: ValueOutcome::Delivered,
            value: 42.0,
            requested: Some(150.0),
            target: None,
            pinned: None,
        });
        drop(guard);
        let drained = take_value_trail();
        assert_eq!(
            drained[0].target,
            Some(0xAB_CDEF),
            "the fact queued beneath the route names the control the action arrived under"
        );
        assert_eq!(
            drained[0].requested,
            Some(150.0),
            "the caller's own ask rides untouched — the stamp never fills it"
        );

        // Outside a guard: the absence honest — the widget doors' rides.
        note_write(ValueRecord {
            action: ValueAction::Increment,
            outcome: ValueOutcome::Delivered,
            value: 43.0,
            requested: None,
            target: None,
            pinned: None,
        });
        // The caller's own answer is never overwritten.
        note_write(ValueRecord {
            action: ValueAction::Decrement,
            outcome: ValueOutcome::Delivered,
            value: 41.0,
            requested: Some(39.0),
            target: Some(1),
            pinned: None,
        });
        let drained = take_value_trail();
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
        note_write(ValueRecord {
            action: ValueAction::Increment,
            outcome: ValueOutcome::Refused(ValueRefusal::NoWriteDoor),
            value: 41.0,
            requested: None,
            target: None,
            pinned: None,
        });
        let drained = take_value_trail();
        assert_eq!(
            drained[0].target, None,
            "the guard's drop cleared the stamp"
        );
        crate::value::set_value_trail_enabled(false);
    }
}
