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
    /// eviction, so a fact queued where no pipeline will ever drain
    /// still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::ValueRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the value trail (the pipeline's side of the
/// trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_value_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one value fact on this thread's trail (the value tier's side
/// of the trail, ADR-0046): a no-op while the trail is disabled, an
/// oldest-evicting push while it rides. The value-tier widgets call
/// this at the exact place the answer resolves — the widget knows the
/// action and the value it holds, the trail knows the drain — so a
/// status line, an inspector trail, or a test reads the value tier
/// without re-implementing the observation at the action site.
pub fn note_write(record: crate::inspect::ValueRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
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
}
