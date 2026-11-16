//! The view-patch trail: how the view reconciliation's patches — the
//! mount's build and every re-render's diff (§13.3, ADR-0078) — reach
//! the causal log without the reconciler holding a devtools
//! dependency.
//!
//! The kill rule the view tier's contract pre-registered: a consumer
//! prices "what did my re-render change — and how much?" → the view
//! tier grows its own per-patch record beside the tiers that already
//! price themselves, never a munge onto the door that answers. Every
//! other tier already streams its tier's own fact — the frame tier's
//! refusals ([`crate::inspect::FrameRefusalRecord`], ADR-0077), the
//! activation tier's asks ([`crate::inspect::ActionRecord`],
//! ADR-0075) — but the view's reconciliation, the tier every
//! re-render rides, answered a `PatchReport` the imperative caller
//! read and the log carried no fact at all. The patch answers as data
//! now ([`crate::inspect::ViewPatchRecord`], carried in
//! [`crate::inspect`]) and the thread trail below, the route-drop
//! trail's shape (ADR-0063): the note site is the exact place the
//! patch resolves — the view root's `mount` build and `update` diff (the
//! `alacer_view` tier's own doors), where the report the door
//! still answers is computed — and the dispatching pipeline drains it
//! at the tick's start into
//! [`crate::inspect::InspectEvent::ViewPatch`], dev-mode and bounded,
//! silent while nobody listens.
//!
//! One fact per patch that moved something, never per frame (the
//! flood rule at the view tier): a no-op re-render — the reconciler's
//! four counts all zero — streams no patch fact at all (the
//! idle-frame property, the m32 canon's own rule: a write that lands
//! where the value already is queues no masks, and no facts). The
//! trail itself is transport — it carries what the note site queues,
//! the site owning the flood rule. A question outside the view tier
//! is not a view-patch fact (the ADR-0046 rule the other tiers ride).
//!
//! ```text
//! // The reconciler's note sites (core-only dependency):
//! alacer_core::view_patch::note_view_patch(
//!     alacer_core::ViewPatchRecord { /* the patch's four counts */ },
//! );
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The view-patch trail's cap: facts carried per thread before the
/// oldest evicts — the journal's own bounded rule (§14.3) applied to
/// the view tier (a devtools log, not a frame-budget liability; the
/// same order the frame-refusal trail rides). A burst of re-renders
/// evicts the oldest — the newest patches are the ones a consumer can
/// still act on.
pub const VIEW_PATCH_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per patch (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`VIEW_PATCH_TRAIL_CAP`] with
    /// oldest-first eviction, so a fact queued where no pipeline will
    /// ever drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::ViewPatchRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the view-patch trail (the pipeline's side of
/// the trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_view_patch_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one view-patch fact on this thread's trail (the note site's
/// side of the trail, ADR-0078): a no-op while the trail is disabled,
/// an oldest-evicting push while it rides. The reconciler's build
/// calls this at the exact place the mount's tree resolves, and the
/// re-render calls it at the exact place the diff resolves — so a
/// status line, an inspector trail, or a test reads the view tier
/// without re-implementing the observation at the call site. The
/// record rides beside the door's own `PatchReport`, never instead of
/// it: `update` still answers the report it always answered.
pub fn note_view_patch(record: crate::inspect::ViewPatchRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        tracing::trace!(
            target: "alacer::view_patch",
            created = record.created,
            removed = record.removed,
            patched = record.patched,
            moved = record.moved,
            "view patch queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= VIEW_PATCH_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::ViewPatch`].
pub(crate) fn take_view_patch_trail() -> Vec<crate::inspect::ViewPatchRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{VIEW_PATCH_TRAIL_CAP, note_view_patch, take_view_patch_trail};
    use crate::inspect::ViewPatchRecord;

    /// The record one re-render's patch carries: the four counts,
    /// verbatim (the reconciler's own accounting).
    fn patch(created: usize) -> ViewPatchRecord {
        ViewPatchRecord {
            created,
            removed: 1,
            patched: 2,
            moved: 3,
        }
    }

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the frame-refusal trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_view_patch(patch(1));
        assert!(
            take_view_patch_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::view_patch::set_view_patch_trail_enabled(true);
        for created in 0..(VIEW_PATCH_TRAIL_CAP + 8) {
            note_view_patch(patch(created));
        }
        let drained = take_view_patch_trail();
        assert_eq!(
            drained.len(),
            VIEW_PATCH_TRAIL_CAP,
            "the cap bounds the trail"
        );
        assert_eq!(
            drained[0].created, 8,
            "the oldest evicted first (the ninth patch's count)"
        );
        assert!(
            take_view_patch_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::view_patch::set_view_patch_trail_enabled(false);
    }

    /// The counts ride verbatim — the reconciler's accounting is the
    /// fact. The trail is transport: it carries what a note site
    /// queues, zeros included, and the flood rule lives at the site
    /// (the reconciler's no-op re-render never notes — the m32 canon's
    /// idle-frame property; the suite pins the site's silence).
    #[test]
    fn the_counts_ride_verbatim() {
        crate::view_patch::set_view_patch_trail_enabled(true);
        note_view_patch(ViewPatchRecord {
            created: 0,
            removed: 0,
            patched: 0,
            moved: 0,
        });
        note_view_patch(patch(7));
        let drained = take_view_patch_trail();
        assert_eq!(drained.len(), 2, "the trail carries what is noted");
        assert_eq!(
            drained[0],
            ViewPatchRecord {
                created: 0,
                removed: 0,
                patched: 0,
                moved: 0,
            },
            "the counts ride verbatim — the site owns the flood rule"
        );
        assert_eq!(
            (
                drained[1].created,
                drained[1].removed,
                drained[1].patched,
                drained[1].moved
            ),
            (7, 1, 2, 3),
            "the counts ride verbatim"
        );
        crate::view_patch::set_view_patch_trail_enabled(false);
    }
}
