//! The route-drop trail: how the boundary's defensive drops — the asks
//! refused before any widget's door (§13.3, ADR-0063) — reach the
//! causal log without any widget holding a devtools dependency.
//!
//! The kill rule ADR-0061 pre-registered: the platform's defensive
//! drops (the `SetValue` without a payload, the `SetTextSelection`
//! aimed at another node, the actions no widget contract answers) were
//! traced and gone — a trail reader could not answer "the AT asked; why
//! did no door ever answer" from the log alone. The route grows its own
//! refusal vocabulary now ([`crate::inspect::RouteDropRecord`], carried
//! in [`crate::inspect`]) and the thread trail below, the value trail's
//! shape (ADR-0046): the note site is the exact place the drop resolves
//! — the platform boundary's mapping arms and the dispatch funnel's
//! unknown-target refusal — and the dispatching pipeline drains it at
//! the tick's start into [`crate::inspect::InspectEvent::RouteDrop`],
//! dev-mode and bounded, silent while nobody listens.
//!
//! The record carries no funnel stamp (the M43 rule does not apply): a
//! drop's identity *is* the aim the boundary refused — the aimed id
//! rides the record verbatim, never re-resolved against the tree (a
//! re-resolved aim would be the funnel's answer, not the AT's ask).
//!
//! ```text
//! // The boundary (platform) and the funnel (core) both queue:
//! use alacer_core::route_drop::note_route_drop;
//! note_route_drop(alacer_core::RouteDropRecord { /* the drop's fact */ });
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The route-drop trail's cap: facts carried per thread before the
/// oldest evicts — the journal's own bounded rule (§14.3) applied to
/// its echo (a devtools log, not a frame-budget liability; the same
/// order the edit, announcement, value, and text-write trails ride).
pub const ROUTE_DROP_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per drop (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`ROUTE_DROP_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::RouteDropRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the route-drop trail (the pipeline's side of
/// the trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_route_drop_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one route-drop fact on this thread's trail (the boundary's
/// side of the trail, ADR-0063): a no-op while the trail is disabled,
/// an oldest-evicting push while it rides. The drop sites — the
/// platform boundary's mapping arms, the dispatch funnel's
/// unknown-target refusal — call this at the exact place the drop
/// resolves, so a status line, an inspector trail, or a test reads the
/// boundary without re-implementing the observation at the refusal
/// site. The record rides as built: the aimed id is the aim the
/// boundary refused, verbatim — the trail never re-resolves it against
/// the tree (the record is the route's answer, not the tree's).
pub fn note_route_drop(record: crate::inspect::RouteDropRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        tracing::trace!(
            target: "alacer::route_drop",
            action = record.action,
            species = record.species.name(),
            ?record.target,
            "route-drop fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= ROUTE_DROP_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::RouteDrop`].
pub(crate) fn take_route_drop_trail() -> Vec<crate::inspect::RouteDropRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{ROUTE_DROP_TRAIL_CAP, note_route_drop, take_route_drop_trail};
    use crate::inspect::{RouteDropRecord, RouteDropSpecies};

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the value trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_route_drop(RouteDropRecord {
            action: "set_value",
            species: RouteDropSpecies::UnknownTarget,
            target: Some(0xDEAD_BEEF),
        });
        assert!(
            take_route_drop_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once. The record rides as
    /// built — the aim the caller named is the aim the trail carries.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::route_drop::set_route_drop_trail_enabled(true);
        for index in 0..(ROUTE_DROP_TRAIL_CAP + 8) {
            note_route_drop(RouteDropRecord {
                action: "set_value",
                species: RouteDropSpecies::UnknownTarget,
                target: Some(index as u64),
            });
        }
        let drained = take_route_drop_trail();
        assert_eq!(
            drained.len(),
            ROUTE_DROP_TRAIL_CAP,
            "the cap bounds the trail"
        );
        assert_eq!(drained[0].target, Some(8), "the oldest evicted first");
        assert!(
            take_route_drop_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::route_drop::set_route_drop_trail_enabled(false);
    }

    /// Every species carries its stable tag — the wire vocabulary's
    /// `snake_case` rule, the names the protocol document pins.
    #[test]
    fn the_species_carry_their_stable_tags() {
        assert_eq!(RouteDropSpecies::UnknownAction.name(), "unknown_action");
        assert_eq!(RouteDropSpecies::NoPayload.name(), "no_payload");
        assert_eq!(
            RouteDropSpecies::OffTargetSelection.name(),
            "off_target_selection"
        );
        assert_eq!(RouteDropSpecies::UnknownTarget.name(), "unknown_target");
        assert_eq!(
            RouteDropSpecies::NoFocusedField.name(),
            "no_focused_field",
            "the route's other silence joins the vocabulary (M59)"
        );
    }
}
