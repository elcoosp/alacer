//! The key-edit trail: how the typing door's edits — the keyboard's
//! keys and the IME's commits (§13.3, ADR-0064) — reach the causal
//! log without any widget holding a devtools dependency.
//!
//! The kill rule ADR-0062 pre-registered: a consumer prices the
//! keyboard's and IME's edits → the edit stream grows its own
//! per-change record beside the journal's, never a per-keystroke
//! flood. The journal's [`crate::inspect::EditRecord`] streams the
//! depths an edit left; the change's *shape* — the spans the edit
//! consumed and inserted, the landing selection, the driver who wrote
//! it — was traced and gone. The typing door grows its own record
//! vocabulary ([`crate::inspect::KeyEditRecord`], carried in
//! [`crate::inspect`]) and the thread trail below, the text-write
//! trail's shape (ADR-0061): the widget notes the fact at the exact
//! place the edit resolves — the widget knows the action and the
//! selection it lands, the trail knows the drain — and the
//! dispatching pipeline drains it at the tick's start into
//! [`crate::inspect::InspectEvent::KeyEdit`], dev-mode and bounded,
//! silent while nobody listens.
//!
//! The flood rule is the journal's own fold: a coalesced typed run
//! supersedes the run's record in place — N keystrokes in one
//! pure-append run land as one record whose shape is the run's, the
//! `coalesced` bit naming the fold — and every edit the journal kept
//! as its own step lands as its own record. Since M60 (ADR-0066) the
//! clipboard's chords name their field changes here too — the paste's
//! shape and the cut's span behind their own actions and the
//! clipboard's driver — while the round trip keeps the clipboard
//! trail's fact (ADR-0045): one fact per trail, the chord never
//! superseded into a typed run's record and never a double-priced
//! edit; the typing door's notes name the typing door only.
//!
//! The record names its control with the value trail's own stamp
//! (M43, ADR-0049): the key route and the IME route hold one
//! `crate::value::ActionTargetGuard` for the length of one event, the
//! typing door's facts stamping themselves with it — and since M58
//! the IME route holds the stamp too, the routes shared, the identity
//! never guessed at the note site.
//!
//! ```text
//! // Widgets (core-only dependencies):
//! use alacer_core::key_edit::note_key_edit;
//! note_key_edit(alacer_core::KeyEditRecord { /* the edit's fact */ });
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The key-edit trail's cap: facts carried per thread before the
/// oldest evicts — the journal's own bounded rule (§14.3) applied to
/// its echo (a devtools log, not a frame-budget liability; the same
/// order the edit, announcement, value, and text-write trails ride).
pub const KEY_EDIT_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per edit (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the trail, bounded by [`KEY_EDIT_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever
    /// drain still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::KeyEditRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the key-edit trail (the pipeline's side of
/// the trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_key_edit_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one key-edit fact on this thread's trail (the typing door's
/// side of the trail, ADR-0064): a no-op while the trail is disabled,
/// an oldest-evicting push while it rides. The field's keyboard and
/// IME doors call this at the exact place the edit resolves — the
/// widget knows the action and the selection it lands, the trail
/// knows the drain — so a status line, an inspector trail, or a test
/// reads the typing door without re-implementing the observation at
/// the key site. Since M43 (ADR-0049) a record whose `target` the
/// widget left `None` is stamped with the id the routing funnel held —
/// the control's identity riding beside the event's routing, never
/// guessed at the note site (the value trail's own stamp, the same
/// guard the AT tier's writes and the keyboard's steps stamp from).
///
/// The flood rule (ADR-0064, the journal's own fold): a record the
/// journal coalesced into its tip supersedes the run's record — the
/// same control, the same driver, a pure insertion continuing exactly
/// at the landing the record carries — so a typed run of N keystrokes
/// streams one record whose shape is the run's. An edit the journal
/// kept as its own step (the run's first keystroke, a deletion, an
/// IME commit beside a typed run) pushes its own record; the trail
/// never merges what the journal kept apart.
pub fn note_key_edit(record: crate::inspect::KeyEditRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        // The stamp side (M43, ADR-0049): the funnel's guard answers
        // the *who* the widget cannot know — the widget's own `None`
        // stays `None` when no route holds a guard (the direct mutate
        // seam's honest absence).
        let mut record = record;
        if record.target.is_none() {
            record.target = crate::value::action_target();
        }
        tracing::trace!(
            target: "alacer::key_edit",
            action = record.action.name(),
            driver = record.driver.name(),
            replaced = record.replaced,
            inserted = record.inserted,
            coalesced = record.coalesced,
            "key-edit fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            // The flood rule's supersede: the journal folded this edit
            // into its tip, and the trail's last record is the run that
            // tip continues — the same control, driver, and pure
            // insertion, landing exactly where the run's record left
            // off. The run's record grows; no per-keystroke flood.
            if record.coalesced
                && let Some(last) = queue.back_mut()
                && last.action == crate::inspect::KeyEditAction::Insert
                && last.driver == record.driver
                && last.target == record.target
                && record.selection.focus.checked_sub(record.inserted) == Some(last.selection.focus)
            {
                last.inserted += record.inserted;
                last.selection = record.selection;
                last.coalesced = true;
                return;
            }
            if queue.len() >= KEY_EDIT_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::KeyEdit`].
pub(crate) fn take_key_edit_trail() -> Vec<crate::inspect::KeyEditRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{KEY_EDIT_TRAIL_CAP, note_key_edit, take_key_edit_trail};
    use crate::inspect::{KeyEditAction, KeyEditDriver, KeyEditRecord};
    use crate::widget::TextSelection;

    /// The fact one edit carries: a pure insertion of `inserted` units
    /// starting at `from` — the landing caret the edit leaves at its
    /// end, anchor collapsed onto it (the M11 edit rule).
    fn insert_at(from: usize, inserted: usize, coalesced: bool) -> KeyEditRecord {
        let caret = from + inserted;
        KeyEditRecord {
            action: KeyEditAction::Insert,
            driver: KeyEditDriver::Keyboard,
            replaced: 0,
            inserted,
            selection: TextSelection::new(caret, caret),
            coalesced,
            target: None,
        }
    }

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the value trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_key_edit(insert_at(3, 1, false));
        assert!(
            take_key_edit_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        crate::key_edit::set_key_edit_trail_enabled(true);
        for caret in 0..(KEY_EDIT_TRAIL_CAP + 8) {
            note_key_edit(insert_at(caret, 1, false));
        }
        let drained = take_key_edit_trail();
        assert_eq!(
            drained.len(),
            KEY_EDIT_TRAIL_CAP,
            "the cap bounds the trail"
        );
        assert_eq!(
            drained[0].selection.focus, 9,
            "the oldest evicted first (the ninth insert's landing)"
        );
        assert!(
            take_key_edit_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        crate::key_edit::set_key_edit_trail_enabled(false);
    }

    /// The stamp side (M43, ADR-0049): a record the widget queued with
    /// a `None` target is stamped with the id the funnel's guard held —
    /// the control's identity riding beside the event's routing, never
    /// guessed at the note site. A fact queued outside a guard rides
    /// its honest absence (the direct mutate seam's), and a dropped
    /// guard leaves no stale identity.
    #[test]
    fn the_funnel_stamp_names_the_control_and_the_absence_holds() {
        crate::key_edit::set_key_edit_trail_enabled(true);

        // Beneath a guard: the widget's `None` becomes the routed id.
        let guard = crate::value::ActionTargetGuard::for_target(Some(0xAB_CDEF));
        note_key_edit(insert_at(1, 1, false));
        drop(guard);
        let drained = take_key_edit_trail();
        assert_eq!(
            drained[0].target,
            Some(0xAB_CDEF),
            "the fact queued beneath the route names the control the key rode"
        );

        // Outside a guard: the absence honest — the direct seam's ride.
        note_key_edit(insert_at(2, 1, false));
        let drained = take_key_edit_trail();
        assert_eq!(
            drained[0].target, None,
            "a ride no funnel routed carries no target"
        );

        // A dropped guard leaves no stale identity: the next fact is
        // unstamped.
        let guard = crate::value::ActionTargetGuard::for_target(Some(1));
        drop(guard);
        note_key_edit(insert_at(3, 1, false));
        let drained = take_key_edit_trail();
        assert_eq!(
            drained[0].target, None,
            "the guard's drop cleared the stamp"
        );
        crate::key_edit::set_key_edit_trail_enabled(false);
    }

    /// The flood rule (ADR-0064): a coalesced typed run lands as one
    /// record — the run's shape, the fold named. N keystrokes in one
    /// pure-append run = exactly one fact, never a per-keystroke flood.
    #[test]
    fn the_coalesced_run_lands_as_one_record() {
        crate::key_edit::set_key_edit_trail_enabled(true);

        // The run's first keystroke pushes its own record; the journal
        // kept no tip to fold into yet.
        note_key_edit(insert_at(0, 1, false));
        // Every continuation folds into the journal's tip — the trail
        // supersedes the run's record, the shape growing to the run's.
        note_key_edit(insert_at(1, 1, true));
        note_key_edit(insert_at(2, 2, true));
        let drained = take_key_edit_trail();
        assert_eq!(drained.len(), 1, "a typed run is one fact, not four");
        assert_eq!(drained[0].inserted, 4, "the record's shape is the run's");
        assert_eq!(
            drained[0].selection.focus, 4,
            "the landing is the run's own caret"
        );
        assert!(drained[0].coalesced, "the fold is named on the record");
        crate::key_edit::set_key_edit_trail_enabled(false);
    }

    /// The supersede never crosses runs the journal kept apart: a
    /// non-coalesced edit (a new run's first keystroke) pushes beside
    /// the old run's record, and a different control or driver is never
    /// merged into another's run — the identity the trail never guesses.
    #[test]
    fn the_supersede_stays_within_the_run() {
        crate::key_edit::set_key_edit_trail_enabled(true);

        // A run lands, then a fresh keystroke the journal did not fold
        // (the caret moved — a new run): two records.
        note_key_edit(insert_at(0, 1, false));
        note_key_edit(insert_at(1, 1, true));
        note_key_edit(insert_at(8, 1, false));
        let drained = take_key_edit_trail();
        assert_eq!(
            drained.len(),
            2,
            "the journal's step boundary is the trail's"
        );
        assert_eq!(drained[1].selection.focus, 9, "the new run lands beside");

        // Another control's coalesced run never merges into the first
        // field's record: the stamp's identity holds the runs apart.
        note_key_edit(KeyEditRecord {
            action: KeyEditAction::Insert,
            driver: KeyEditDriver::Keyboard,
            replaced: 0,
            inserted: 1,
            selection: TextSelection::new(11, 11),
            coalesced: true,
            target: Some(0xBEEF),
        });
        note_key_edit(KeyEditRecord {
            action: KeyEditAction::Insert,
            driver: KeyEditDriver::Keyboard,
            replaced: 0,
            inserted: 1,
            selection: TextSelection::new(12, 12),
            coalesced: true,
            target: Some(0xFEED),
        });
        let drained = take_key_edit_trail();
        assert_eq!(drained.len(), 2, "different controls never merge");
        assert_eq!(drained[0].target, Some(0xBEEF), "the first field's run");
        assert_eq!(drained[1].target, Some(0xFEED), "the second field's run");

        // The IME's committed composition beside a typed run rides its
        // own record — the journal may fold the shape, the trail keeps
        // the drivers apart (the attribution rule at the typing tier).
        note_key_edit(insert_at(0, 1, false));
        note_key_edit(KeyEditRecord {
            action: KeyEditAction::Insert,
            driver: KeyEditDriver::Ime,
            replaced: 0,
            inserted: 3,
            selection: TextSelection::new(4, 4),
            coalesced: true,
            target: None,
        });
        let drained = take_key_edit_trail();
        assert_eq!(drained.len(), 2, "the drivers stay apart");
        assert_eq!(
            drained[0].driver,
            KeyEditDriver::Keyboard,
            "the typed run's driver"
        );
        assert_eq!(drained[1].driver, KeyEditDriver::Ime, "the commit's driver");
        crate::key_edit::set_key_edit_trail_enabled(false);
    }

    /// The chord's record is its own fact (M60, ADR-0066): a paste the
    /// journal folded into the typed run's tip lands beside the run's
    /// record — the drivers' records never merge even when the
    /// journal's tip folded the shape — and two chords never supersede
    /// each other: the flood rule stays the typed run's, the chord a
    /// discrete act, one fact per chord.
    #[test]
    fn the_chord_records_stay_their_own_facts() {
        crate::key_edit::set_key_edit_trail_enabled(true);

        // The typed run's record, then the paste the journal folded
        // (coalesced) but the trail kept apart: the driver guard holds
        // the run and the chord apart.
        note_key_edit(insert_at(0, 2, false));
        note_key_edit(KeyEditRecord {
            action: KeyEditAction::Paste,
            driver: KeyEditDriver::Clipboard,
            replaced: 0,
            inserted: 3,
            selection: TextSelection::new(5, 5),
            coalesced: true,
            target: None,
        });
        let drained = take_key_edit_trail();
        assert_eq!(
            drained.len(),
            2,
            "the paste never merges into the typed run's record"
        );
        assert_eq!(drained[1].action, KeyEditAction::Paste);

        // Two chords the journal folded tip-to-tip stay two facts: the
        // supersede is the pure-insertion run's rule, never a chord's.
        note_key_edit(KeyEditRecord {
            action: KeyEditAction::Paste,
            driver: KeyEditDriver::Clipboard,
            replaced: 0,
            inserted: 2,
            selection: TextSelection::new(5, 5),
            coalesced: true,
            target: None,
        });
        note_key_edit(KeyEditRecord {
            action: KeyEditAction::Paste,
            driver: KeyEditDriver::Clipboard,
            replaced: 0,
            inserted: 2,
            selection: TextSelection::new(7, 7),
            coalesced: true,
            target: None,
        });
        let drained = take_key_edit_trail();
        assert_eq!(
            drained.len(),
            2,
            "a chord never supersedes a chord — one fact per chord"
        );
        assert_eq!(drained[0].inserted, 2, "the first chord's own shape");
        assert_eq!(drained[1].inserted, 2, "the second chord's own shape");
        crate::key_edit::set_key_edit_trail_enabled(false);
    }

    /// The vocabulary carries its stable tags — the wire vocabulary's
    /// `snake_case` rule, the names the protocol document pins.
    #[test]
    fn the_actions_and_drivers_carry_their_stable_tags() {
        assert_eq!(KeyEditAction::Insert.name(), "insert");
        assert_eq!(KeyEditAction::Backspace.name(), "backspace");
        assert_eq!(KeyEditAction::Delete.name(), "delete");
        assert_eq!(KeyEditAction::WordDeleteBack.name(), "word_delete_back");
        assert_eq!(
            KeyEditAction::WordDeleteForward.name(),
            "word_delete_forward"
        );
        assert_eq!(KeyEditAction::Paste.name(), "paste");
        assert_eq!(KeyEditAction::Cut.name(), "cut");
        assert_eq!(KeyEditDriver::Keyboard.name(), "keyboard");
        assert_eq!(KeyEditDriver::Ime.name(), "ime");
        assert_eq!(KeyEditDriver::Clipboard.name(), "clipboard");
    }
}
