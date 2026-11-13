//! The scroll trail: how the scroll tier's answers — the list's wheel,
//! the AT's unit scrolls and value steps, the app's own programmatic
//! scroll (§13.3, ADR-0056) — reach the causal log without any widget
//! holding a devtools dependency.
//!
//! The scroll doors answered the dispatch contract in silence: a wheel
//! past the content's end clamped without a word, a wheel over a list
//! whose content fits the viewport moved nothing unexplained, a
//! non-finite delta rode the clamp arithmetic and corrupted the state.
//! ADR-0055's second kill rule pre-registered the price: a consumer
//! needs the wall named on a tier whose funnel knows no range — the
//! list's scroll clamp — and "that tier's own record grows its own
//! fact beside its species, never the value record's". The ride the
//! rule cashes is the record vocabulary
//! ([`crate::inspect::ScrollRecord`], carried in [`crate::inspect`])
//! and the thread trail below, the value trail's shape (ADR-0046): the
//! widget notes the fact at the exact place the answer resolves — the
//! door knows the action and the offset it holds, the trail knows the
//! drain — and the dispatching pipeline drains it at the tick's start
//! into [`crate::inspect::InspectEvent::Scroll`], dev-mode and
//! bounded, silent while nobody listens.
//!
//! The record names its control too (the M43/M45/M46 stamp discipline
//! at the scroll tier): the dispatch funnel's wheel route knows the
//! hit it resolved, the action route knows the id the action arrived
//! under, the widget does not — so the stamp side lives here
//! (`SCROLL_TARGET`, held by the route for the length of one routed
//! ask) and [`note_scroll`] stamps the facts queued beneath a routed
//! ask with it, never a node id guessed at the note site. The app's
//! own door ([`alacer_widgets::ListHandle::scroll_to`]) rides no route
//! — its facts carry the honest absence.
//!
//! ```text
//! // Widgets (core-only dependencies):
//! use alacer_core::scroll::note_scroll;
//! note_scroll(alacer_core::inspect::ScrollRecord { /* the ask's fact */ });
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The scroll trail's cap: facts carried per thread before the oldest
/// evicts — the value trail's own bounded rule (§14.3) applied to its
/// sibling (a devtools log, not a frame-budget liability; the same
/// order the edit, announcement, clipboard, value, and gesture trails
/// ride).
pub const SCROLL_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per ask (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`SCROLL_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::ScrollRecord>> =
        const { RefCell::new(VecDeque::new()) };
    /// The control the dispatch funnel's scroll routes are currently
    /// resolving (M50, ADR-0056): `Some(id)` while a routed ask holds
    /// its guard, `None` otherwise — the facts queued beneath the ask
    /// carry the route's id, the facts queued outside one ride their
    /// absence.
    static SCROLL_TARGET: Cell<Option<u64>> = const { Cell::new(None) };
}

/// Enables or disables the scroll trail (the pipeline's side of the
/// trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_scroll_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Whether the trail notes facts on this thread — the routes'
/// idle-contract check (M50, ADR-0056): the stamp's id-path walk runs
/// only under an observer, while nobody listens the stamp is one
/// `Cell` read answered here (the A7 idle contract, one branch per
/// routed ask).
pub(crate) fn scroll_trail_enabled() -> bool {
    TRAIL_ENABLED.with(Cell::get)
}

/// The funnel's stamp guard (M50, ADR-0056): the wheel route holds one
/// for the length of one routed wheel (the hit it resolved — the
/// deepest accepting hit under the hover), the action route one for
/// the length of one routed AT action — the facts queued beneath the
/// ask carrying the id the ask rode, the widget never guessing its own
/// identity (the M43 rule's words at the scroll tier). Dropping clears
/// the stamp — a panicking widget leaves no stale identity on the
/// thread.
pub(crate) struct ScrollTargetGuard {
    previous: Option<u64>,
}

impl ScrollTargetGuard {
    /// Stamps `target` as the control the scroll route is resolving,
    /// restoring the previous stamp on drop (the routes nest never —
    /// one guard per routed ask — but a nested route would still
    /// answer honestly).
    pub(crate) fn for_target(target: Option<u64>) -> Self {
        let previous = SCROLL_TARGET.with(|slot| {
            let previous = slot.get();
            slot.set(target);
            previous
        });
        Self { previous }
    }
}

impl Drop for ScrollTargetGuard {
    fn drop(&mut self) {
        SCROLL_TARGET.with(|slot| slot.set(self.previous));
    }
}

/// Notes one scroll fact on this thread's trail (the scroll tier's
/// side of the trail, ADR-0056): a no-op while the trail is disabled,
/// an oldest-evicting push while it rides. The scroll doors call this
/// at the exact place the answer resolves — the wheel's clamp, the
/// AT's unit scroll, the app's own scroll — so a status line, an
/// inspector trail, or a test reads the scroll tier without
/// re-implementing the observation at the door site. Since M50
/// (ADR-0056) a record whose `target` the door left `None` is stamped
/// with the id the route held — the wheel's hit, the action's target —
/// never guessed at the note site. The ask is never stamped: the
/// widget's own door computed it (the M44 rule at the scroll tier),
/// the record the caller built is the record the trail carries — the
/// stamp fills only the identity the widget cannot know.
pub fn note_scroll(record: crate::inspect::ScrollRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        // The stamp side (M50, ADR-0056): the funnel's guard answers
        // the *who* the widget cannot know — the widget's own `None`
        // stays `None` when no route holds a guard (the app door's
        // honest absence).
        let mut record = record;
        if record.target.is_none() {
            record.target = SCROLL_TARGET.with(Cell::get);
        }
        tracing::trace!(
            target: "alacer::scroll",
            action = record.action.name(),
            outcome = record.outcome.name(),
            offset = record.offset,
            "scroll fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= SCROLL_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::Scroll`].
pub(crate) fn take_scroll_trail() -> Vec<crate::inspect::ScrollRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{SCROLL_TRAIL_CAP, note_scroll, take_scroll_trail};
    use crate::inspect::{RangeBound, ScrollAction, ScrollOutcome, ScrollRecord, ScrollRefusal};

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the value trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_scroll(ScrollRecord {
            action: ScrollAction::Wheel,
            outcome: ScrollOutcome::Delivered,
            offset: 42.0,
            requested: Some(42.0),
            target: None,
            pinned: None,
        });
        assert!(
            take_scroll_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::scroll::set_scroll_trail_enabled(true);
        for index in 0..(SCROLL_TRAIL_CAP + 8) {
            let offset = f32::from(u16::try_from(index % 256).expect("in range"));
            note_scroll(ScrollRecord {
                action: ScrollAction::ScrollDown,
                outcome: ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent),
                offset,
                requested: None,
                target: None,
                pinned: None,
            });
        }
        let drained = take_scroll_trail();
        assert_eq!(drained.len(), SCROLL_TRAIL_CAP, "the cap bounds the trail");
        assert_eq!(drained[0].offset, 8.0, "the oldest evicted first");
        assert!(
            take_scroll_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::scroll::set_scroll_trail_enabled(false);
    }

    /// The stamp side (M50, ADR-0056): a record the door queued with a
    /// `None` target is stamped with the id the route's guard held —
    /// the wheel's hit, the action's target, never guessed at the note
    /// site. A record the caller named itself keeps its own answer,
    /// and a fact queued outside a guard rides its honest absence.
    #[test]
    fn the_route_stamp_names_the_control_and_the_absence_holds() {
        crate::scroll::set_scroll_trail_enabled(true);

        // Beneath a guard: the door's `None` becomes the routed id,
        // the ask the caller carried untouched (the stamp fills only
        // the identity the widget cannot know).
        let guard = crate::scroll::ScrollTargetGuard::for_target(Some(0xAB_CDEF));
        note_scroll(ScrollRecord {
            action: ScrollAction::Wheel,
            outcome: ScrollOutcome::Delivered,
            offset: 40.0,
            requested: Some(100.0),
            target: None,
            pinned: Some(RangeBound::Max),
        });
        drop(guard);
        let drained = take_scroll_trail();
        assert_eq!(
            drained[0].target,
            Some(0xAB_CDEF),
            "the fact queued beneath the route names the control the ask rode"
        );
        assert_eq!(
            drained[0].requested,
            Some(100.0),
            "the caller's own ask rides untouched — the stamp never fills it"
        );
        assert_eq!(
            drained[0].pinned,
            Some(RangeBound::Max),
            "the wall the caller computed rides untouched — the stamp never fills it"
        );

        // Outside a guard: the absence honest — the app door's rides.
        note_scroll(ScrollRecord {
            action: ScrollAction::ScrollTo,
            outcome: ScrollOutcome::Delivered,
            offset: 30.0,
            requested: Some(30.0),
            target: None,
            pinned: None,
        });
        // The caller's own answer is never overwritten.
        note_scroll(ScrollRecord {
            action: ScrollAction::Increment,
            outcome: ScrollOutcome::Delivered,
            offset: 130.0,
            requested: None,
            target: Some(1),
            pinned: None,
        });
        let drained = take_scroll_trail();
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
        note_scroll(ScrollRecord {
            action: ScrollAction::ScrollUp,
            outcome: ScrollOutcome::Delivered,
            offset: 90.0,
            requested: Some(130.0),
            target: None,
            pinned: None,
        });
        let drained = take_scroll_trail();
        assert_eq!(
            drained[0].target, None,
            "the guard's drop cleared the stamp"
        );
        crate::scroll::set_scroll_trail_enabled(false);
    }
}
