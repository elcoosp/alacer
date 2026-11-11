//! The editing history: the undo/redo vocabulary of the editing contract
//! (§10.3, ADR-0021).
//!
//! The clipboard tier (M14, ADR-0020) moves a selection out of the field;
//! the history tier moves its edits back. Like every editing rule that
//! must not grow widget dialects — `paste_text`, `word_hop` — the journal
//! and its rules live in core, one definition, pure text arithmetic over
//! the contract's UTF-16 units ([`crate::utf16`]), and the editor only
//! consumes them.
//!
//! ```text
//! // The widget journals every committed value edit through one funnel:
//! self.history.record(at_utf16, removed, inserted, before, after);
//!
//! // The chords drive the two directions; the journal does the surgery:
//! let Some((anchor, caret)) = self.history.undo(&mut self.text) else { … };
//! ```
//!
//! Three rules fix the vocabulary, each test-pinned here:
//!
//! 1. **The pure-append coalescing rule.** Consecutive insertions that
//!    continue exactly at the previous insertion's end — typing a word a
//!    character at a time — coalesce into one undo step; a deletion, a
//!    discontinuity, or a selection boundary opens a fresh one. Every
//!    other edit (backspace, a word deletion, a cut, a paste, an IME
//!    commit) is its own step.
//! 2. **The reset rule.** A whole-value replacement — the view's
//!    reconciliation, the AT's `SetValue` — resets the journal: the app
//!    wrote the value, not the user, and undo must never fight the app's
//!    own write (the same reset a programmatic set has in every platform
//!    editor).
//! 3. **The bounded-journal rule.** The journal carries a named capacity
//!    and evicts its *oldest* entry when the bound is crossed — a cache
//!    with an eviction policy and tests that prove it (§14.3), never an
//!    unbounded vector riding a long editing session.
//!
//! **The echo (§13.3, ADR-0037).** The journal speaks for itself: every
//! [`EditHistory`] action streams one [`EditRecord`] — the action, the
//! depths it left, the refusal it named — into this thread's edit trail,
//! the queue the dispatching pipeline drains at the tick's start into
//! [`crate::inspect::InspectEvent::Edit`]. The seam is dev-mode and
//! opt-in: the journal notes a fact only while the trail is enabled (one
//! branch per action otherwise — the A7 idle contract), the pipeline
//! enables it when an observer installs, and the queue itself is bounded
//! by [`EDIT_TRAIL_CAP`] with the journal's own oldest-first eviction
//! (rule 3's discipline, applied to the echo). The clipboard-seam shape
//! (ADR-0020): handlers cannot reach the pipeline (axiom A1), so the
//! journal queues on the calling thread and the dispatching pipeline
//! drains — the input queue's shape (ADR-0014): push now, the tick
//! routes, one frame. An action queued with no pipeline anywhere still
//! terminates: the bound evicts, never the app's memory.

use crate::inspect::EditRecord;
use crate::utf16;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

/// One committed value edit: the inverse pair the journal replays.
///
/// The replaced span is `[at_utf16, at_utf16 + removed units)` *before*
/// the edit and `[at_utf16, at_utf16 + inserted units)` *after* it — undo
/// replaces the inserted span with `removed`, redo the removed span with
/// `inserted`. Both selections are `(anchor, caret)` in UTF-16 units: the
/// selection the edit consumed (`before`) and the one it left (`after`),
/// so an undo restores not just the text but the range the user was
/// extending when they made the edit (the M11 vocabulary, ADR-0017).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditEntry {
    /// The replaced span's start, in UTF-16 units of the field's value.
    pub at_utf16: usize,
    /// The text the edit removed from the span (empty for a pure insert).
    pub removed: String,
    /// The text the edit inserted into the span (empty for a pure delete).
    pub inserted: String,
    /// The `(anchor, caret)` selection before the edit, UTF-16 units.
    pub selection_before: (usize, usize),
    /// The `(anchor, caret)` selection after the edit, UTF-16 units.
    pub selection_after: (usize, usize),
}

impl EditEntry {
    /// The entry's inserted length, in UTF-16 units.
    #[must_use]
    pub fn inserted_units(&self) -> usize {
        utf16::len_utf16(&self.inserted)
    }

    /// The entry's removed length, in UTF-16 units.
    #[must_use]
    pub fn removed_units(&self) -> usize {
        utf16::len_utf16(&self.removed)
    }
}

/// The bounded two-stack journal behind the editing contract's undo/redo
/// tier (§10.3, ADR-0021). One per field — the history *is* the field's
/// state, not a platform fact, so unlike the clipboard seam there is no
/// indirection: the widget owns one and core fixes the rules.
#[derive(Debug)]
pub struct EditHistory {
    undo: Vec<EditEntry>,
    redo: Vec<EditEntry>,
    capacity: usize,
}

/// What the journal just answered (ADR-0037): the edit trail's action
/// vocabulary, one [`EditRecord`] per resolved action. The names are the
/// wire's stable tags ([`EditAction::name`], the protocol's
/// `snake_case` rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditAction {
    /// A committed value edit was journaled — the widget's edit funnel
    /// call. The record's `coalesced` names whether it folded into the
    /// journal's tip (the pure-append rule) or opened a fresh step.
    Record,
    /// An undo chord was answered — applied, or refused on an exhausted
    /// branch (the record's `refused`).
    Undo,
    /// A redo chord was answered — applied, or refused on an empty
    /// branch.
    Redo,
    /// The whole-value replacement's reset — the app's write cleared
    /// both branches (never refused; there is nothing to refuse).
    Reset,
}

impl EditAction {
    /// The action's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values —
    /// [`crate::passes::FocusMove::name`] for movements, this for
    /// journal actions).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Record => "record",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::Reset => "reset",
        }
    }
}

/// The edit trail's cap: facts carried per thread before the oldest
/// evicts — the journal's own bounded rule (§14.3) applied to its echo
/// (a devtools log, not a frame-budget liability; the same order the
/// recorder's input and focus rings ride).
pub const EDIT_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// journal costs one branch per action (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side of
    /// the seam, bounded by [`EDIT_TRAIL_CAP`] with oldest-first
    /// eviction, so an action queued where no pipeline will ever drain
    /// still terminates.
    static TRAIL: RefCell<VecDeque<EditRecord>> = const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the edit trail (the pipeline's side of the seam:
/// `PassPipeline::set_observer` turns it on when a devtools observer
/// installs). Disabling never clears the queue — the drain owns that.
pub(crate) fn set_edit_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one journal fact on this thread's trail (the journal's side of
/// the seam): a no-op while the trail is disabled, an oldest-evicting
/// push while it rides.
fn note(record: EditRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        tracing::trace!(
            target: "alacer::history",
            action = record.action.name(),
            coalesced = record.coalesced,
            refused = record.refused,
            undo_depth = record.undo_depth,
            redo_depth = record.redo_depth,
            "journal fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= EDIT_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// `InspectEvent::Edit`.
pub(crate) fn take_edit_trail() -> Vec<EditRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

impl EditHistory {
    /// The default bound: 100 steps, the depth every platform editor's
    /// undo carries or exceeds — deep enough that a user never notices,
    /// shallow enough that a long session cannot grow the field without
    /// bound.
    pub const DEFAULT_CAPACITY: usize = 100;

    /// An empty journal with the default capacity.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// An empty journal with an explicit capacity — the eviction policy's
    /// seam (tests pin the eviction at small bounds; the trail's records
    /// carry the depth live, ADR-0037).
    #[must_use]
    pub const fn with_capacity(capacity: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            capacity,
        }
    }

    /// Journals one committed value edit (the widget's single edit funnel
    /// calls this exactly once per edit).
    ///
    /// The new entry clears the redo stack — any edit invalidates the
    /// redo branch, the rule every editor shares — and coalesces into the
    /// journal's tip under the pure-append rule: both entries pure
    /// insertions, the new one continuing exactly at the tip's end, and
    /// the tip's `selection_after` the new one's `selection_before`
    /// (the user kept typing at the moving caret). Crossing the capacity
    /// evicts the oldest entry.
    pub fn record(
        &mut self,
        at_utf16: usize,
        removed: String,
        inserted: String,
        selection_before: (usize, usize),
        selection_after: (usize, usize),
    ) {
        self.redo.clear();
        let coalesced = match self.undo.last_mut() {
            Some(tip) if Self::coalesces(tip, at_utf16, &removed, &inserted, selection_before) => {
                tip.inserted.push_str(&inserted);
                tip.selection_after = selection_after;
                true
            }
            _ => {
                self.undo.push(EditEntry {
                    at_utf16,
                    removed,
                    inserted,
                    selection_before,
                    selection_after,
                });
                false
            }
        };
        if self.undo.len() > self.capacity {
            let excess = self.undo.len() - self.capacity;
            self.undo.drain(0..excess);
        }
        note(EditRecord {
            action: EditAction::Record,
            coalesced,
            refused: false,
            undo_depth: self.undo.len(),
            redo_depth: 0,
        });
    }

    /// The pure-append coalescing rule: a fresh pure insertion continues
    /// the tip's pure insertion exactly at its end, with no selection
    /// boundary crossed in between.
    fn coalesces(
        tip: &EditEntry,
        at_utf16: usize,
        removed: &str,
        inserted: &str,
        selection_before: (usize, usize),
    ) -> bool {
        removed.is_empty()
            && !inserted.is_empty()
            && tip.removed.is_empty()
            && !tip.inserted.is_empty()
            && at_utf16 == tip.at_utf16 + tip.inserted_units()
            && tip.selection_after == selection_before
    }

    /// Undoes the newest step: replaces its inserted span with the text
    /// the edit removed, restores the selection the edit consumed, and
    /// returns it as `(anchor, caret)` in UTF-16 units. `None` is the
    /// honest answer for an exhausted journal — the caller refuses the
    /// chord with no masks (the refuse-don't-munge rule), and the trail
    /// carries the refusal beside the depths it did not move (ADR-0037).
    pub fn undo(&mut self, text: &mut String) -> Option<(usize, usize)> {
        let Some(entry) = self.undo.pop() else {
            note(EditRecord {
                action: EditAction::Undo,
                coalesced: false,
                refused: true,
                undo_depth: 0,
                redo_depth: self.redo.len(),
            });
            return None;
        };
        let range = utf16::byte_range(
            text,
            entry.at_utf16,
            entry.at_utf16 + entry.inserted_units(),
        );
        text.replace_range(range, &entry.removed);
        self.redo.push(entry.clone());
        note(EditRecord {
            action: EditAction::Undo,
            coalesced: false,
            refused: false,
            undo_depth: self.undo.len(),
            redo_depth: self.redo.len(),
        });
        Some(entry.selection_before)
    }

    /// Redoes the newest undone step: replaces its removed span with the
    /// text the edit inserted, restores the selection the edit left, and
    /// returns it as `(anchor, caret)` in UTF-16 units. `None` is the
    /// honest answer for an empty redo branch, and the trail carries the
    /// refusal beside the depths it did not move (ADR-0037).
    pub fn redo(&mut self, text: &mut String) -> Option<(usize, usize)> {
        let Some(entry) = self.redo.pop() else {
            note(EditRecord {
                action: EditAction::Redo,
                coalesced: false,
                refused: true,
                undo_depth: self.undo.len(),
                redo_depth: 0,
            });
            return None;
        };
        let range = utf16::byte_range(text, entry.at_utf16, entry.at_utf16 + entry.removed_units());
        text.replace_range(range, &entry.inserted);
        self.undo.push(entry.clone());
        note(EditRecord {
            action: EditAction::Redo,
            coalesced: false,
            refused: false,
            undo_depth: self.undo.len(),
            redo_depth: self.redo.len(),
        });
        Some(entry.selection_after)
    }

    /// Resets the journal — the whole-value replacement's rule (the view
    /// reconciliation, the AT's `SetValue`): both stacks clear, the next
    /// undo refuses, and the trail names the emptied journal (ADR-0037).
    pub fn reset(&mut self) {
        self.undo.clear();
        self.redo.clear();
        note(EditRecord {
            action: EditAction::Reset,
            coalesced: false,
            refused: false,
            undo_depth: 0,
            redo_depth: 0,
        });
    }

    /// Whether a step is available to undo.
    #[must_use]
    pub const fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether a step is available to redo.
    #[must_use]
    pub const fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Whether the journal carries no steps at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.undo.is_empty() && self.redo.is_empty()
    }

    /// The journal's depth: the undo branch's length — the trail's
    /// records carry it live (ADR-0037), and the tests' direct read.
    #[must_use]
    pub const fn undo_depth(&self) -> usize {
        self.undo.len()
    }
}

impl Default for EditHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{EditAction, EditHistory};
    use crate::inspect::EditRecord;

    /// Drives the journal the way the widget's edit funnel does: one
    /// helper that records an edit and applies it to the text, so every
    /// test exercises the same record/apply pairing production uses. A
    /// caret edit (`at == to`) records the caret as the selection it
    /// consumed; a span edit records the span — the M11 vocabulary.
    fn edit(history: &mut EditHistory, text: &mut String, at: usize, to: usize, inserted: &str) {
        let selection_before = if at == to {
            (at, at)
        } else {
            (at.min(to), at.max(to))
        };
        let from = crate::utf16::byte_offset(text, at);
        let to_byte = crate::utf16::byte_offset(text, to);
        let removed = text[from..to_byte].to_string();
        text.replace_range(from..to_byte, inserted);
        let caret = at + crate::utf16::len_utf16(inserted);
        history.record(
            at,
            removed,
            inserted.to_string(),
            selection_before,
            (caret, caret),
        );
    }

    #[test]
    fn typing_coalesces_and_any_discontinuity_opens_a_step() {
        let mut history = EditHistory::new();
        let mut text = String::new();
        edit(&mut history, &mut text, 0, 0, "h");
        edit(&mut history, &mut text, 1, 1, "e");
        edit(&mut history, &mut text, 2, 2, "y");
        assert_eq!(text, "hey");
        assert_eq!(history.undo_depth(), 1, "a typed run is one step");

        // A caret jump breaks the chain: the next insertion is its own
        // step even though it still appends.
        edit(&mut history, &mut text, 0, 0, "— ");
        assert_eq!(text, "— hey");
        assert_eq!(history.undo_depth(), 2);
        let (anchor, caret) = history.undo(&mut text).expect("a step to undo");
        assert_eq!(text, "hey", "the jumped insertion undoes alone");
        assert_eq!((anchor, caret), (0, 0), "the selection before it returns");
    }

    #[test]
    fn deletions_are_their_own_steps_and_undo_restores_the_span() {
        let mut history = EditHistory::new();
        let mut text = String::from("hello world");
        edit(&mut history, &mut text, 5, 11, "");
        assert_eq!(text, "hello");
        edit(&mut history, &mut text, 4, 5, "");
        assert_eq!(text, "hell");
        assert_eq!(history.undo_depth(), 2, "deletions never coalesce");

        let (anchor, caret) = history.undo(&mut text).expect("a step to undo");
        assert_eq!(text, "hello");
        assert_eq!((anchor, caret), (4, 5), "the removed scalar is re-selected");

        let (anchor, caret) = history.undo(&mut text).expect("a step to undo");
        assert_eq!(text, "hello world");
        assert_eq!((anchor, caret), (5, 11), "the deleted range is re-selected");
        assert!(history.can_redo(), "the undone steps form the redo branch");
    }

    #[test]
    fn redo_replays_the_step_and_any_edit_invalidates_the_branch() {
        let mut history = EditHistory::new();
        let mut text = String::new();
        edit(&mut history, &mut text, 0, 0, "held");
        let _ = history.undo(&mut text);
        assert_eq!(text, "");

        let (anchor, caret) = history.redo(&mut text).expect("a step to redo");
        assert_eq!(text, "held");
        assert_eq!(
            (anchor, caret),
            (4, 4),
            "redo restores the edit's own caret"
        );

        // A fresh edit clears the redo branch — the rule every editor
        // shares: history forks at the newest edit.
        edit(&mut history, &mut text, 4, 4, "!");
        assert_eq!(text, "held!");
        assert!(!history.can_redo(), "the fork dropped the redo branch");
    }

    #[test]
    fn an_ime_sized_commit_is_one_step() {
        let mut history = EditHistory::new();
        let mut text = String::from("にほ");
        // A commit rides the same funnel: one insertion, one step, even
        // when the text carries multi-byte runes.
        edit(&mut history, &mut text, 2, 2, "んご");
        assert_eq!(text, "にほんご");
        let (anchor, caret) = history.undo(&mut text).expect("a step to undo");
        assert_eq!(text, "にほ", "the whole commit undoes as one step");
        assert_eq!((anchor, caret), (2, 2));
    }

    #[test]
    fn the_journal_never_slices_a_surrogate_pair() {
        let mut history = EditHistory::new();
        let mut text = String::from("a\u{1F600}b"); // a, pair, b — 4 UTF-16 units
        edit(&mut history, &mut text, 1, 3, "");
        assert_eq!(text, "ab", "the pair left as one scalar value");
        let _ = history.undo(&mut text);
        assert_eq!(text, "a\u{1F600}b", "the pair returns whole");

        edit(&mut history, &mut text, 4, 4, "!");
        let _ = history.undo(&mut text);
        assert_eq!(text, "a\u{1F600}b", "appending after the pair undoes clean");
    }

    #[test]
    fn a_whole_value_replacement_resets_the_journal() {
        let mut history = EditHistory::new();
        let mut text = String::from("typed");
        edit(&mut history, &mut text, 5, 5, " more");
        assert!(history.can_undo());

        // The app wrote the value: both stacks clear, undo refuses.
        text.clear();
        text.push_str("replaced by the app");
        history.reset();
        assert!(history.is_empty(), "the reset clears both branches");
        assert!(history.undo(&mut text).is_none(), "undo refuses");
    }

    #[test]
    fn the_journal_is_bounded_and_evicts_the_oldest_step() {
        let mut history = EditHistory::with_capacity(3);
        let mut text = String::new();
        // Five separate steps into a journal of three: every insertion
        // lands at 0 (a caret jump each time), so the pure-append rule
        // never coalesces them and the two oldest evict in order.
        for word in ["a", "b", "c", "d", "e"] {
            edit(&mut history, &mut text, 0, 0, word);
        }
        assert_eq!(text, "edcba");
        assert_eq!(history.undo_depth(), 3, "the bound holds");

        // The survivors are the three newest: undoing replays "e", then
        // "d", then "c" — the "a" and "b" steps are gone for good, the
        // FIFO policy the bound promises.
        let _ = history.undo(&mut text);
        assert_eq!(text, "dcba");
        let _ = history.undo(&mut text);
        assert_eq!(text, "cba");
        let _ = history.undo(&mut text);
        assert_eq!(text, "ba");
        assert!(
            history.undo(&mut text).is_none(),
            "the oldest steps evicted"
        );
    }

    #[test]
    fn the_journal_notes_its_facts_when_the_trail_rides() {
        crate::history::set_edit_trail_enabled(true);
        let mut history = EditHistory::new();
        let mut text = String::new();
        edit(&mut history, &mut text, 0, 0, "he");
        edit(&mut history, &mut text, 2, 2, "y");
        assert_eq!(text, "hey");
        assert_eq!(history.undo_depth(), 1, "a typed run is one step");

        let (anchor, caret) = history.undo(&mut text).expect("a step to undo");
        assert_eq!((anchor, caret), (0, 0));
        let (anchor2, caret2) = history.redo(&mut text).expect("a step to redo");
        assert_eq!((anchor2, caret2), (3, 3));

        let trail = crate::history::take_edit_trail();
        crate::history::set_edit_trail_enabled(false);
        assert_eq!(trail.len(), 4, "every journal action streamed");
        assert_eq!(
            trail[0],
            EditRecord {
                action: EditAction::Record,
                coalesced: false,
                refused: false,
                undo_depth: 1,
                redo_depth: 0,
            },
            "the first keystroke opens the step"
        );
        assert!(
            trail[1].coalesced,
            "the continued keystroke folds into the tip"
        );
        assert_eq!(trail[1].undo_depth, 1, "coalescing never deepens");
        assert_eq!(
            trail[2],
            EditRecord {
                action: EditAction::Undo,
                coalesced: false,
                refused: false,
                undo_depth: 0,
                redo_depth: 1,
            },
            "the undo names the depth it left"
        );
        assert_eq!(trail[3].action, EditAction::Redo);
        assert_eq!(trail[3].undo_depth, 1, "redo re-pushes the step");
    }

    #[test]
    fn a_refused_chord_streams_too() {
        crate::history::set_edit_trail_enabled(true);
        let mut history = EditHistory::new();
        let mut text = String::new();
        assert!(
            history.undo(&mut text).is_none(),
            "an exhausted journal refuses"
        );
        assert!(history.redo(&mut text).is_none(), "an empty branch refuses");
        let trail = crate::history::take_edit_trail();
        crate::history::set_edit_trail_enabled(false);
        assert_eq!(trail.len(), 2, "the refusals are facts the log carries");
        assert!(trail[0].refused, "the undo refused — the fact is named");
        assert_eq!(trail[0].action, EditAction::Undo);
        assert_eq!(trail[0].undo_depth, 0, "the refusal moved nothing");
        assert!(trail[1].refused);
        assert_eq!(trail[1].action, EditAction::Redo);
    }

    #[test]
    fn the_app_write_reset_streams_and_clears_the_redo_branch() {
        crate::history::set_edit_trail_enabled(true);
        let mut history = EditHistory::new();
        let mut text = String::new();
        edit(&mut history, &mut text, 0, 0, "typed");
        let _ = history.undo(&mut text);
        history.reset();
        let trail = crate::history::take_edit_trail();
        crate::history::set_edit_trail_enabled(false);
        assert_eq!(trail.len(), 3);
        assert_eq!(
            trail[2],
            EditRecord {
                action: EditAction::Reset,
                coalesced: false,
                refused: false,
                undo_depth: 0,
                redo_depth: 0,
            },
            "the reset names the emptied journal"
        );
    }

    #[test]
    fn the_trail_is_bounded_and_evicts_the_oldest_fact() {
        crate::history::set_edit_trail_enabled(true);
        let mut history = EditHistory::new();
        let mut text = String::new();
        for word in 0..(super::EDIT_TRAIL_CAP + 10) {
            edit(&mut history, &mut text, 0, 0, "x");
            // A caret jump each time: every insertion opens its own step,
            // so the journal's own bound never coalesces the facts away.
            edit(&mut history, &mut text, 0, 0, "\u{0}");
            let _ = history.undo(&mut text);
            let _ = word;
        }
        let trail = crate::history::take_edit_trail();
        crate::history::set_edit_trail_enabled(false);
        assert_eq!(trail.len(), super::EDIT_TRAIL_CAP, "the bound holds");
        assert!(
            trail
                .last()
                .is_some_and(|last| last.action == EditAction::Undo),
            "the newest fact is the last chord the loop drove"
        );
    }

    #[test]
    fn the_trail_stays_silent_when_no_one_listens() {
        crate::history::set_edit_trail_enabled(false);
        let mut history = EditHistory::new();
        let mut text = String::new();
        edit(&mut history, &mut text, 0, 0, "unheard");
        let _ = history.undo(&mut text);
        assert!(
            crate::history::take_edit_trail().is_empty(),
            "the disabled trail queues nothing — the edit path's one branch"
        );
    }

    #[test]
    fn the_drain_empties_the_queue() {
        crate::history::set_edit_trail_enabled(true);
        let mut history = EditHistory::new();
        let mut text = String::new();
        edit(&mut history, &mut text, 0, 0, "once");
        let first = crate::history::take_edit_trail();
        assert_eq!(first.len(), 1);
        assert!(
            crate::history::take_edit_trail().is_empty(),
            "the drain clears — a fact streams once"
        );
        crate::history::set_edit_trail_enabled(false);
    }
}
