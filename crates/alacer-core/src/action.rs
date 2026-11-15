//! The action trail: how the activation ask — the press the assistive
//! technology's action route aims at a control (§11.5, §13.3, ADR-0075)
//! — reaches the causal log without any widget holding a devtools
//! dependency.
//!
//! The kill rule the action tier's own contract pre-registered: a
//! consumer prices the activation ask → the action tier grows its own
//! per-ask record beside the tiers that already price themselves, never
//! a munge onto a door that answered. Every other a11y action already
//! streams its tier's own fact — the value tier's writes
//! ([`crate::inspect::ValueRecord`], ADR-0046), the scroll tier's asks
//! ([`crate::inspect::ScrollRecord`], ADR-0056), the text-write tier's
//! rewrites ([`crate::inspect::TextWriteRecord`], ADR-0061) — but the
//! most common ask of all, the AT's press, answered a bare `bool` whose
//! `false` collapsed the two facts a trail reader wants apart: the
//! availability tier's wall and the doorless target's silence. The
//! activation ask answers as data now ([`crate::inspect::ActionRecord`],
//! carried in [`crate::inspect`]) and the thread trail below, the
//! route-drop trail's shape (ADR-0063): the note site is the exact
//! place the ask resolves — the dispatch funnel, which holds the ask,
//! the aimed id, the door's answer, and the tree's one `enabled` truth
//! (M22, ADR-0028) — and the dispatching pipeline drains it at the
//! tick's start into [`crate::inspect::InspectEvent::Action`], dev-mode
//! and bounded, silent while nobody listens.
//!
//! One fact per trail (the M39 rule at the activation tier): the asks
//! whose tiers already price themselves stream no action fact — the
//! funnel notes the activation ask only — and a question outside the
//! activation tier is not an action fact (the ADR-0046 rule the scroll
//! tier rides).
//!
//! ```text
//! // The funnel (core-only dependency):
//! alacer_core::action::note_action(alacer_core::ActionRecord { /* the ask's fact */ });
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The action trail's cap: facts carried per thread before the oldest
/// evicts — the journal's own bounded rule (§14.3) applied to its echo
/// (a devtools log, not a frame-budget liability; the same order the
/// edit, announcement, value, and route-drop trails ride).
pub const ACTION_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per ask (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`ACTION_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::ActionRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the action trail (the pipeline's side of
/// the trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_action_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one action fact on this thread's trail (the note site's side
/// of the trail, ADR-0075): a no-op while the trail is disabled,
/// an oldest-evicting push while it rides. The dispatch funnel calls
/// this at the exact place the ask resolves — the route holds the ask,
/// the aimed id, the door's answer, and the tree's `enabled` truth, the
/// species computed where both facts live — so a status line, an
/// inspector trail, or a test reads the activation tier without
/// re-implementing the observation at the call site. The record's
/// target is the aim the funnel resolved; since M70 (ADR-0076) a
/// record whose `target` the keyboard's activation left `None` is
/// stamped with the id the key route's guard held (M45, ADR-0051 — the
/// value trail's stamp vocabulary, the same guard the value tier's
/// key-route rides stamp from) — the identity never guessed at the
/// note site.
pub fn note_action(record: crate::inspect::ActionRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        // The stamp side (M43/M45, ADR-0049/ADR-0051): the route's guard
        // answers the *who* the widget cannot know — the keyboard
        // activation's own `None` stays `None` when no route holds a
        // guard (the honest absence).
        let mut record = record;
        if record.target.is_none() {
            record.target = crate::value::action_target();
        }
        tracing::trace!(
            target: "alacer::action",
            ask = record.ask.name(),
            rider = record.rider.name(),
            outcome = record.outcome.name(),
            "activation ask queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= ACTION_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::Action`].
pub(crate) fn take_action_trail() -> Vec<crate::inspect::ActionRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{ACTION_TRAIL_CAP, note_action, take_action_trail};
    use crate::inspect::{ActionAsk, ActionOutcome, ActionRefusal};

    /// The fact one landed press carries: the activation ask, aimed at
    /// the control the funnel resolved — the route's own rider beside
    /// it (the funnel's notes are the AT route's, the keyboard's
    /// activation naming itself at the widget's door, M70).
    fn delivered_at(target: Option<u64>) -> crate::inspect::ActionRecord {
        crate::inspect::ActionRecord {
            ask: ActionAsk::Press,
            target,
            outcome: ActionOutcome::Delivered,
            rider: crate::inspect::ActionRider::ActionRoute,
        }
    }

    /// The fact one refusal carries: the ask the door turned away, the
    /// species the route computed beside it (M22's wall, or the
    /// doorless target's silence).
    fn refused_at(refusal: ActionRefusal, target: Option<u64>) -> crate::inspect::ActionRecord {
        crate::inspect::ActionRecord {
            ask: ActionAsk::Press,
            target,
            outcome: ActionOutcome::Refused(refusal),
            rider: crate::inspect::ActionRider::ActionRoute,
        }
    }

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the value trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_action(delivered_at(Some(7)));
        assert!(
            take_action_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::action::set_action_trail_enabled(true);
        for target in 0..(ACTION_TRAIL_CAP + 8) {
            note_action(delivered_at(Some(
                u64::try_from(target).expect("non-negative"),
            )));
        }
        let drained = take_action_trail();
        assert_eq!(drained.len(), ACTION_TRAIL_CAP, "the cap bounds the trail");
        assert_eq!(
            drained[0].target,
            Some(8),
            "the oldest evicted first (the ninth press's aim)"
        );
        assert!(
            take_action_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::action::set_action_trail_enabled(false);
    }

    /// The vocabulary carries its stable tags — the wire vocabulary's
    /// `snake_case` rule, the names the protocol document pins.
    #[test]
    fn the_asks_and_outcomes_carry_their_stable_tags() {
        assert_eq!(ActionAsk::Press.name(), "press");
        assert_eq!(ActionOutcome::Delivered.name(), "delivered");
        assert_eq!(
            ActionOutcome::Refused(ActionRefusal::NoDoor).name(),
            "refused"
        );
        assert_eq!(ActionRefusal::NoDoor.name(), "no_door");
        assert_eq!(ActionRefusal::Unavailable.name(), "unavailable");
        assert_eq!(
            crate::inspect::ActionRider::ActionRoute.name(),
            "action_route"
        );
        assert_eq!(crate::inspect::ActionRider::Keyboard.name(), "keyboard");
    }

    /// The keyboard's activation names its own rider (M70, ADR-0076):
    /// the widget's door notes the ride with `Keyboard` beside it — the
    /// attribution declared where the route is known, the funnel's
    /// notes staying the AT route's.
    #[test]
    fn the_keyboard_activation_names_its_rider() {
        crate::action::set_action_trail_enabled(true);

        note_action(crate::inspect::ActionRecord {
            ask: ActionAsk::Press,
            target: None,
            outcome: ActionOutcome::Delivered,
            rider: crate::inspect::ActionRider::Keyboard,
        });
        let drained = take_action_trail();
        assert_eq!(drained.len(), 1, "the activation is one fact");
        assert_eq!(
            drained[0].rider,
            crate::inspect::ActionRider::Keyboard,
            "the keyboard's rider, beside the ask"
        );
        crate::action::set_action_trail_enabled(false);
    }

    /// The refusal rides its species and its aim verbatim: the wall's
    /// fact and the doorless silence are the ask's two named answers,
    /// the aimed id never re-resolved against the tree.
    #[test]
    fn the_refusal_names_its_species_and_its_aim() {
        crate::action::set_action_trail_enabled(true);

        note_action(refused_at(ActionRefusal::Unavailable, Some(0xAB_CDEF)));
        note_action(refused_at(ActionRefusal::NoDoor, Some(1)));
        note_action(refused_at(ActionRefusal::NoDoor, None));
        let drained = take_action_trail();
        assert_eq!(drained.len(), 3, "one fact per ask");
        assert_eq!(
            drained[0].outcome,
            ActionOutcome::Refused(ActionRefusal::Unavailable),
            "the wall's species, beside the aim"
        );
        assert_eq!(
            drained[1].outcome,
            ActionOutcome::Refused(ActionRefusal::NoDoor),
            "the doorless silence, beside the aim"
        );
        assert_eq!(
            drained[2].target, None,
            "an ask no route resolved rides the honest absence"
        );
        crate::action::set_action_trail_enabled(false);
    }
}
