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

use crate::utf16;

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
    /// seam (tests pin the eviction at small bounds; the inspector may
    /// someday surface the depth).
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
        match self.undo.last_mut() {
            Some(tip) if Self::coalesces(tip, at_utf16, &removed, &inserted, selection_before) => {
                tip.inserted.push_str(&inserted);
                tip.selection_after = selection_after;
            }
            _ => self.undo.push(EditEntry {
                at_utf16,
                removed,
                inserted,
                selection_before,
                selection_after,
            }),
        }
        if self.undo.len() > self.capacity {
            let excess = self.undo.len() - self.capacity;
            self.undo.drain(0..excess);
        }
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
    /// chord with no masks (the refuse-don't-munge rule).
    pub fn undo(&mut self, text: &mut String) -> Option<(usize, usize)> {
        let entry = self.undo.pop()?;
        let range = utf16::byte_range(
            text,
            entry.at_utf16,
            entry.at_utf16 + entry.inserted_units(),
        );
        text.replace_range(range, &entry.removed);
        self.redo.push(entry.clone());
        Some(entry.selection_before)
    }

    /// Redoes the newest undone step: replaces its removed span with the
    /// text the edit inserted, restores the selection the edit left, and
    /// returns it as `(anchor, caret)` in UTF-16 units. `None` is the
    /// honest answer for an empty redo branch.
    pub fn redo(&mut self, text: &mut String) -> Option<(usize, usize)> {
        let entry = self.redo.pop()?;
        let range = utf16::byte_range(text, entry.at_utf16, entry.at_utf16 + entry.removed_units());
        text.replace_range(range, &entry.inserted);
        self.undo.push(entry.clone());
        Some(entry.selection_after)
    }

    /// Resets the journal — the whole-value replacement's rule (the view
    /// reconciliation, the AT's `SetValue`): both stacks clear, and the
    /// next undo refuses.
    pub fn reset(&mut self) {
        self.undo.clear();
        self.redo.clear();
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

    /// The journal's depth: the undo branch's length — the inspector's
    /// future vocabulary, and the tests' direct read.
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
    use super::EditHistory;

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
}
