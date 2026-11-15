//! The frame-refusal trail: how the frame's own refusals — the tick's
//! layout stage, the backends' present ladder (§13.3, ADR-0077) —
//! reach the causal log without any backend holding a devtools
//! dependency.
//!
//! The kill rule the frame tier's contract pre-registered: a consumer
//! prices "why did nothing paint?" → the frame tier grows its own
//! per-refusal record beside the tiers that already price themselves,
//! never a munge onto a door that refused. Every other tier already
//! streams its tier's own fact — the activation tier's asks
//! ([`crate::inspect::ActionRecord`], ADR-0075), the scroll tier's
//! asks ([`crate::inspect::ScrollRecord`], ADR-0056) — but the frame
//! itself, the tier every other tier's work rides, answered a log line
//! and a skipped frame: the platform's present ladder logged and moved
//! on, and the log carried no fact at all. The refusal answers as data
//! now ([`crate::inspect::FrameRefusalRecord`], carried in
//! [`crate::inspect`]) and the thread trail below, the route-drop
//! trail's shape (ADR-0063): the note site is the exact place the
//! refusal resolves — the tick's layout block for the engine seam's
//! own [`crate::passes::LayoutError`], the backends' present ladder
//! for the render/present stages — and the dispatching pipeline drains
//! it at the tick's start into
//! [`crate::inspect::InspectEvent::FrameRefusal`], dev-mode and
//! bounded, silent while nobody listens.
//!
//! One fact per trail (the M39 rule at the frame tier): the frame's
//! happy paths stream no fact — the frame summary the pipeline already
//! forwards (ADR-0037) prices the frames that painted, the refusal
//! trail prices the frames that did not — and a question outside the
//! frame tier is not a frame-refusal fact (the ADR-0046 rule the other
//! tiers ride).
//!
//! ```text
//! // The present ladder (core-only dependency):
//! alacer_core::frame_refusal::note_frame_refusal(
//!     alacer_core::FrameRefusalRecord { /* the ladder stage's fact */ },
//! );
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The frame-refusal trail's cap: facts carried per thread before the
/// oldest evicts — the journal's own bounded rule (§14.3) applied to
/// the frame tier (a devtools log, not a frame-budget liability; the
/// same order the edit, announcement, value, and route-drop trails
/// ride). A run of failed frames evicts the oldest — the newest
/// refusals are the ones a consumer can still act on.
pub const FRAME_REFUSAL_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per refusal (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`FRAME_REFUSAL_TRAIL_CAP`] with
    /// oldest-first eviction, so a fact queued where no pipeline will
    /// ever drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::FrameRefusalRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the frame-refusal trail (the pipeline's side of
/// the trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_frame_refusal_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one frame-refusal fact on this thread's trail (the note
/// site's side of the trail, ADR-0077): a no-op while the trail is
/// disabled, an oldest-evicting push while it rides. The tick's layout
/// block calls this at the exact place the engine seam's error
/// resolves, and the present ladder calls it at the exact place its
/// stage refused — so a status line, an inspector trail, or a test
/// reads the frame tier without re-implementing the observation at the
/// call site. The record rides beside the refusal's own door, never
/// instead of it: the tick still answers `Err(LayoutError)`, the
/// ladder still skips the frame it cannot present.
pub fn note_frame_refusal(record: crate::inspect::FrameRefusalRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        tracing::trace!(
            target: "alacer::frame_refusal",
            species = record.species.name(),
            node = record.node,
            "frame refusal queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= FRAME_REFUSAL_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::FrameRefusal`].
pub(crate) fn take_frame_refusal_trail() -> Vec<crate::inspect::FrameRefusalRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{FRAME_REFUSAL_TRAIL_CAP, note_frame_refusal, take_frame_refusal_trail};
    use crate::inspect::FrameRefusalSpecies;

    /// The fact one layout refusal carries: the stage's species beside
    /// the failing node's id (ADR-0023's vocabulary, verbatim).
    fn layout_refused_at(node: Option<u64>) -> crate::inspect::FrameRefusalRecord {
        crate::inspect::FrameRefusalRecord {
            species: FrameRefusalSpecies::LayoutRefused,
            node,
        }
    }

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the action trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_frame_refusal(layout_refused_at(Some(7)));
        assert!(
            take_frame_refusal_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::frame_refusal::set_frame_refusal_trail_enabled(true);
        for node in 0..(FRAME_REFUSAL_TRAIL_CAP + 8) {
            note_frame_refusal(layout_refused_at(Some(
                u64::try_from(node).expect("non-negative"),
            )));
        }
        let drained = take_frame_refusal_trail();
        assert_eq!(
            drained.len(),
            FRAME_REFUSAL_TRAIL_CAP,
            "the cap bounds the trail"
        );
        assert_eq!(
            drained[0].node,
            Some(8),
            "the oldest evicted first (the ninth refusal's node)"
        );
        assert!(
            take_frame_refusal_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::frame_refusal::set_frame_refusal_trail_enabled(false);
    }

    /// The vocabulary carries its stable tags — the wire vocabulary's
    /// `snake_case` rule, the names the protocol document pins.
    #[test]
    fn the_species_carry_their_stable_tags() {
        assert_eq!(FrameRefusalSpecies::LayoutRefused.name(), "layout_refused");
        assert_eq!(FrameRefusalSpecies::RenderFailed.name(), "render_failed");
        assert_eq!(
            FrameRefusalSpecies::PartialRepaintFailed.name(),
            "partial_repaint_failed"
        );
        assert_eq!(
            FrameRefusalSpecies::FramebufferLost.name(),
            "framebuffer_lost"
        );
        assert_eq!(FrameRefusalSpecies::PresentFailed.name(), "present_failed");
    }

    /// The present ladder's stages name their species — the record the
    /// platform's note sites queue (ADR-0077): each species is a
    /// distinct fact, never a collapsed `false`.
    #[test]
    fn the_present_ladder_stages_name_their_species() {
        crate::frame_refusal::set_frame_refusal_trail_enabled(true);

        let ladder = [
            FrameRefusalSpecies::RenderFailed,
            FrameRefusalSpecies::PartialRepaintFailed,
            FrameRefusalSpecies::FramebufferLost,
            FrameRefusalSpecies::PresentFailed,
        ];
        for species in ladder {
            note_frame_refusal(crate::inspect::FrameRefusalRecord {
                species,
                node: None,
            });
        }
        let drained = take_frame_refusal_trail();
        assert_eq!(drained.len(), 4, "one fact per refused stage");
        for (fact, species) in drained.iter().zip(ladder.iter()) {
            assert_eq!(fact.species, *species, "the stage's own species");
            assert_eq!(
                fact.node, None,
                "a present-tier failure is the window's fact, not a widget's"
            );
        }
        crate::frame_refusal::set_frame_refusal_trail_enabled(false);
    }
}
