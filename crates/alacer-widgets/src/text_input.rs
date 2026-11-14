//! The text input: the IME contract's reference widget (§10.3).
//!
//! One line of editable text. Editing is **UTF-16-native**: the caret is a
//! UTF-16 offset, IME preedit composes inline at the caret (with the
//! platform's cursor offset inside the preedit), and commits insert final
//! text — every conversion between the contract's units and Rust's UTF-8
//! strings funnels through `alacer_core::utf16`, so surrogate pairs and CJK
//! never slice (the fcitx5/ibus matrix exercises exactly this).
//!
//! Focus is a first-class: the widget paints its focus ring when
//! [`Widget::set_focused`] says it holds keyboard focus, and reports
//! [`ImePurpose::Text`] (or [`ImePurpose::Secure`]) so the platform attaches
//! the IME while it is focused.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, ImeEvent, KeyEvent, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::history::EditHistory;
use alacer_core::passes::ImePurpose;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style};
use alacer_core::utf16;
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, TextSelection, Widget};

/// The value-change callback of [`TextInput`].
type ChangeHandler = Box<dyn FnMut(&str)>;

/// The submit callback of [`TextInput`] (M6: the todo canon's Enter seam).
type SubmitHandler = Box<dyn FnMut(&str)>;

/// The glyph inset: the run starts 2 px inside the frame. Paint and the
/// hit-test share the constant so a click maps onto what is drawn (M12).
const TEXT_INSET: f32 = 2.0;

/// A single-line editable text input (role `TextInput`, name mandatory —
/// ADR-5; the accessible name is the constructor's, the *value* is the
/// content).
pub struct TextInput {
    /// The accessible name (a screen reader says "Search, edit text").
    name: String,
    /// The committed text (UTF-8; all offsets below are UTF-16).
    text: String,
    /// The caret as a UTF-16 offset into `text` (the contract's unit).
    caret_utf16: usize,
    /// The selection anchor as a UTF-16 offset (M11, ADR-0017): where the
    /// selection started and does not move while the caret extends it.
    /// Equal to `caret_utf16` whenever the selection is degenerate — the
    /// bare caret every edit and collapse keeps the invariant on.
    anchor_utf16: usize,
    /// Whether the pointer is pressed inside the field (M12, ADR-0018):
    /// the drag-select arm. `true` from a press until the release (or a
    /// focus loss — the drag ends when the field loses the pointer's
    /// gesture owner); moves extend the selection only while it holds.
    pressing: bool,
    /// The composing preedit: `(text, cursor within preedit in UTF-16 units)`.
    /// `Some` while an IME composition is in progress.
    preedit: Option<(String, Option<usize>)>,
    /// The editing journal (M15, ADR-0021): the inverse of every committed
    /// value edit, bounded and coalesced by core's one-definition rules.
    history: EditHistory,
    /// Whether this widget holds keyboard focus (paints the ring).
    focused: bool,
    /// The availability tier's flag (M22, ADR-0028).
    enabled: bool,
    placeholder: String,
    purpose: ImePurpose,
    font_size: f32,
    style: Style,
    on_changed: Option<ChangeHandler>,
    on_submit: Option<SubmitHandler>,
}

/// The shape one edit landed (the typing door's fact, M58, ADR-0064):
/// the UTF-16 spans the edit consumed and inserted, and whether the
/// journal folded it into its tip — the flood rule's own bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LandedEdit {
    /// The span the edit consumed, in UTF-16 units.
    replaced: usize,
    /// The payload's own length, in UTF-16 units.
    inserted: usize,
    /// Whether the journal folded the edit into its tip.
    coalesced: bool,
}

impl LandedEdit {
    /// No edit landed — a no-op door answers no fact (the honest
    /// absence: the journal recorded nothing, the trail queues none).
    const NONE: Self = Self {
        replaced: 0,
        inserted: 0,
        coalesced: false,
    };

    /// Whether any edit landed — a no-op door queues nothing.
    fn landed(self) -> bool {
        self.replaced != 0 || self.inserted != 0
    }
}

impl TextInput {
    /// A text input with the accessible name `name`.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            text: String::new(),
            caret_utf16: 0,
            anchor_utf16: 0,
            pressing: false,
            preedit: None,
            history: EditHistory::new(),
            focused: false,
            enabled: true,
            placeholder: String::new(),
            purpose: ImePurpose::Text,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
            on_changed: None,
            on_submit: None,
        }
    }

    /// Secret entry: the platform suppresses preedit echo and suggestions.
    #[must_use]
    pub fn secure(mut self) -> Self {
        self.purpose = ImePurpose::Secure;
        self
    }

    /// Initial availability (M22, ADR-0028). An unavailable input paints
    /// muted, holds no focus, and refuses every editing door honestly —
    /// the value the tree carries is the app's, and the AT reads the
    /// refusal before it happens.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Placeholder text shown while the value is empty.
    #[must_use]
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Installs the change handler — invoked after every committed change to
    /// the value (key edits and IME commits alike).
    #[must_use]
    pub fn on_changed(mut self, handler: impl FnMut(&str) + 'static) -> Self {
        self.on_changed = Some(Box::new(handler));
        self
    }

    /// Installs the submit handler (M6): invoked on Enter with the current
    /// value, then the field clears — the todo canon's `input { on_submit:
    /// … }` seam. Without a handler Enter stays ignored (Tab cycling).
    #[must_use]
    pub fn on_submit(mut self, handler: impl FnMut(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(handler));
        self
    }

    /// The committed value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.text
    }

    /// The caret as a UTF-16 offset (the contract's unit; test introspection).
    #[must_use]
    pub const fn caret_utf16(&self) -> usize {
        self.caret_utf16
    }

    /// The composing preedit, if any (`(text, cursor_utf16)`).
    #[must_use]
    pub const fn preedit(&self) -> Option<&(String, Option<usize>)> {
        self.preedit.as_ref()
    }

    /// Whether the journal carries a step to undo (test introspection;
    /// the inspector's future vocabulary, ADR-0021).
    #[must_use]
    pub const fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Whether the journal carries a step to redo (test introspection;
    /// the inspector's future vocabulary, ADR-0021).
    #[must_use]
    pub const fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Replaces the whole value and puts the caret at the end (§6.1
    /// reconciliation; fires `on_changed`). The selection collapses with
    /// the caret — a new value has no remembered range — and the journal
    /// resets: the app wrote the value, not the user, and undo must never
    /// fight the app's own write (ADR-0021's reset rule).
    ///
    /// Since M56 (ADR-0062) the app door streams its own fact beside the
    /// journal's reset echo: one [`alacer_core::TextWriteRecord`] with
    /// `source: AppDoor` — the write's shape named by lengths (the value
    /// the field carried, the value the app handed over; the payload
    /// itself never riding the log — a field's value can carry a secret,
    /// the clipboard trail's rule, ADR-0045) and the caret the write
    /// left. The view reconciliation's patches ride this same door, so
    /// their writes are attributed with it — the kill rule ADR-0061
    /// pre-registered. The AT's asks never ride here: the action door
    /// writes through the raw `apply_value` and notes its own record
    /// (one ask, one fact).
    pub fn set_value(&mut self, value: impl Into<String>) {
        let replaced = utf16::len_utf16(&self.text);
        self.apply_value(value);
        alacer_core::text_write::note_text_write(alacer_core::TextWriteRecord {
            action: alacer_core::TextWriteAction::SetValue,
            outcome: alacer_core::TextWriteOutcome::Delivered,
            replaced: Some(replaced),
            inserted: Some(utf16::len_utf16(&self.text)),
            requested: None,
            selection: TextSelection::new(self.anchor_utf16, self.caret_utf16),
            target: None,
            source: alacer_core::TextWriteSource::AppDoor,
        });
    }

    /// The whole-value write, raw — no door fact of its own (M56,
    /// ADR-0062). The app door (`set_value`) notes its record around it;
    /// the AT action door rides it bare, so one ask streams one fact.
    fn apply_value(&mut self, value: impl Into<String>) {
        self.text = value.into();
        self.caret_utf16 = utf16::len_utf16(&self.text);
        self.anchor_utf16 = self.caret_utf16;
        self.history.reset();
        self.notify_changed();
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// The availability the widget reports — the same flag the M22
    /// `enabled()` contract carries into the a11y tree (test
    /// introspection).
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The one availability write: report whether the flag moved (the
    /// M21 idle-frame property, at the widget seam).
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        if self.enabled == enabled {
            return false;
        }
        self.enabled = enabled;
        true
    }

    /// Replaces the placeholder in place (view reconciliation).
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Replaces the change handler in place (view reconciliation). `None`
    /// detaches it.
    pub fn set_on_changed(&mut self, handler: Option<ChangeHandler>) {
        self.on_changed = handler;
    }

    /// Replaces the submit handler in place (view reconciliation). `None`
    /// detaches it.
    pub fn set_on_submit(&mut self, handler: Option<SubmitHandler>) {
        self.on_submit = handler;
    }

    fn notify_changed(&mut self) {
        if let Some(handler) = &mut self.on_changed {
            handler(&self.text.clone());
        }
    }

    /// Notes the typing door's fact (M58, ADR-0064): the shape the edit
    /// landed, named by lengths — never the payload (a field's value can
    /// carry a secret, the clipboard trail's rule, ADR-0045) — the
    /// landing selection beside it. The funnel stamps the control when a
    /// route holds it (the M43 rule); a no-op edit answers no fact.
    fn note_key_edit(
        &self,
        action: alacer_core::KeyEditAction,
        driver: alacer_core::KeyEditDriver,
        landed: LandedEdit,
    ) {
        if !landed.landed() {
            return;
        }
        alacer_core::key_edit::note_key_edit(alacer_core::KeyEditRecord {
            action,
            driver,
            replaced: landed.replaced,
            inserted: landed.inserted,
            selection: TextSelection::new(self.anchor_utf16, self.caret_utf16),
            coalesced: landed.coalesced,
            target: None,
        });
    }

    /// Inserts `insertion` at the caret (UTF-16 units of it — commits may
    /// carry any text, including surrogate pairs). An active selection is
    /// consumed first (M11): the insertion replaces it, like every editor.
    /// Answers the shape the edit landed (the typing door's fact).
    fn insert(&mut self, insertion: &str) -> LandedEdit {
        let (from, to) = self.selection_span();
        self.edit(from, to, insertion)
    }

    /// Deletes the text before the caret (backspace). An active selection
    /// is consumed whole (M11): the range goes, not one scalar value.
    /// Answers the shape the edit landed — `NONE` when nothing went (the
    /// caret at the value's start, an empty selection).
    fn backspace(&mut self) -> LandedEdit {
        if self.has_selection() {
            let (from, to) = self.selection_range();
            return self.edit(from, to, "");
        }
        if self.caret_utf16 == 0 {
            return LandedEdit::NONE;
        }
        // Floor to the scalar boundary at or below the unit before the
        // caret: a caret mid-pair deletes the whole pair, never half.
        let from = utf16::floor_boundary(&self.text, self.caret_utf16 - 1);
        self.edit(from, self.caret_utf16, "")
    }

    /// Deletes the scalar value after the caret (forward delete). An active
    /// selection is consumed whole (M11), like under backspace. Answers
    /// the shape the edit landed — `NONE` at the value's end.
    fn delete(&mut self) -> LandedEdit {
        if self.has_selection() {
            let (from, to) = self.selection_range();
            return self.edit(from, to, "");
        }
        let len = utf16::len_utf16(&self.text);
        if self.caret_utf16 >= len {
            return LandedEdit::NONE;
        }
        let mut to = self.caret_utf16 + 1;
        while to < len && !utf16::is_boundary(&self.text, to) {
            to += 1;
        }
        self.edit(self.caret_utf16, to, "")
    }

    /// Whether a non-degenerate selection is active.
    fn has_selection(&self) -> bool {
        self.anchor_utf16 != self.caret_utf16
    }

    /// The selection's edges in ascending order: `(start, end)`.
    fn selection_range(&self) -> (usize, usize) {
        (
            self.anchor_utf16.min(self.caret_utf16),
            self.anchor_utf16.max(self.caret_utf16),
        )
    }

    /// The UTF-16 span the next edit replaces: an active selection's
    /// range, or the degenerate caret span — the M11 edit rule's one
    /// decision point ("replace the range" vs "act at the caret"),
    /// resolved once for the funnel.
    fn selection_span(&self) -> (usize, usize) {
        if self.has_selection() {
            self.selection_range()
        } else {
            (self.caret_utf16, self.caret_utf16)
        }
    }

    /// The one edit funnel every committed value change rides: replaces
    /// the UTF-16 span `[from, to)` with `insertion`, journals the inverse
    /// (ADR-0021), parks the caret at the insertion's end with the anchor
    /// collapsed onto it, and notifies. One definition — no edit path
    /// grows its own caret arithmetic or its own dialect of the history,
    /// and every journaled step carries the selection it consumed.
    /// Answers the shape the edit landed (the typing door's fact, M58,
    /// ADR-0064): the spans it consumed and inserted, and the journal's
    /// own fold — the caller notes the fact beside the door it rode.
    fn edit(&mut self, from_utf16: usize, to_utf16: usize, insertion: &str) -> LandedEdit {
        let selection_before = (self.anchor_utf16, self.caret_utf16);
        let from = utf16::byte_offset(&self.text, from_utf16);
        let to = utf16::byte_offset(&self.text, to_utf16);
        let removed = self.text[from..to].to_string();
        self.text.replace_range(from..to, insertion);
        self.caret_utf16 = from_utf16 + utf16::len_utf16(insertion);
        self.anchor_utf16 = self.caret_utf16;
        let coalesced = self.history.record(
            from_utf16,
            removed,
            insertion.to_string(),
            selection_before,
            (self.caret_utf16, self.caret_utf16),
        );
        self.notify_changed();
        LandedEdit {
            replaced: to_utf16 - from_utf16,
            inserted: utf16::len_utf16(insertion),
            coalesced,
        }
    }

    /// Undoes the newest journaled edit (Ctrl/Cmd+Z, ADR-0021): the text
    /// and the selection the edit consumed return together. `false` is
    /// the honest refusal of an exhausted journal — the chord still
    /// consumes, but queues no masks (the clipboard tier's discipline).
    fn undo(&mut self) -> bool {
        let Some((anchor, caret)) = self.history.undo(&mut self.text) else {
            return false;
        };
        self.anchor_utf16 = anchor;
        self.caret_utf16 = caret;
        self.notify_changed();
        true
    }

    /// Redoes the newest undone edit (Shift+Ctrl/Cmd+Z, Ctrl/Cmd+Y,
    /// ADR-0021), restoring the selection the edit left. `false` is the
    /// honest refusal of an empty redo branch.
    fn redo(&mut self) -> bool {
        let Some((anchor, caret)) = self.history.redo(&mut self.text) else {
            return false;
        };
        self.anchor_utf16 = anchor;
        self.caret_utf16 = caret;
        self.notify_changed();
        true
    }

    /// Collapses both ends of the selection at `at` — the invariant that a
    /// caret move without Shift forgets the range.
    fn collapse_at(&mut self, at: usize) {
        self.caret_utf16 = at;
        self.anchor_utf16 = at;
    }

    /// Selects the whole value (Ctrl+A; the AT's select-all chord).
    fn select_all(&mut self) {
        self.anchor_utf16 = 0;
        self.caret_utf16 = utf16::len_utf16(&self.text);
    }

    /// Moves the caret one scalar value left, keeping the anchor (a
    /// Shift+Left extension; never into a surrogate pair).
    fn extend_left(&mut self) {
        if self.caret_utf16 == 0 {
            return;
        }
        self.caret_utf16 = utf16::floor_boundary(&self.text, self.caret_utf16 - 1);
    }

    /// Moves the caret one scalar value left without Shift. A first press
    /// over a live selection collapses to its start (GTK semantics) and
    /// does not move; only a second press steps.
    fn caret_left(&mut self) {
        if self.has_selection() {
            let (min, _) = self.selection_range();
            self.collapse_at(min);
            return;
        }
        self.extend_left();
        self.anchor_utf16 = self.caret_utf16;
    }

    /// Moves the caret one scalar value right, keeping the anchor (a
    /// Shift+Right extension; never into a surrogate pair).
    fn extend_right(&mut self) {
        let len = utf16::len_utf16(&self.text);
        if self.caret_utf16 >= len {
            return;
        }
        let mut next = self.caret_utf16 + 1;
        while next < len && !utf16::is_boundary(&self.text, next) {
            next += 1;
        }
        self.caret_utf16 = next;
    }

    /// Moves the caret one scalar value right without Shift — the mirror
    /// of [`Self::caret_left`]: collapse to the selection's end first.
    fn caret_right(&mut self) {
        if self.has_selection() {
            let (_, max) = self.selection_range();
            self.collapse_at(max);
            return;
        }
        self.extend_right();
        self.anchor_utf16 = self.caret_utf16;
    }

    /// The committed value's word segmentation, asked of the text seam —
    /// the widget never carries the Unicode tables (§10.3, ADR-0019).
    fn word_segments(&self) -> Vec<alacer_core::text_seam::WordSegment> {
        alacer_core::text_seam::with_seam(|seam| seam.word_segments(&self.text))
    }

    /// Hops one word in `direction`: the contract's hop rule
    /// ([`alacer_core::text_seam::word_hop`]) over the seam's
    /// segmentation, at the caret.
    fn word_hop(&self, direction: alacer_core::text_seam::WordDirection) -> usize {
        let segments = self.word_segments();
        alacer_core::text_seam::word_hop(&segments, self.caret_utf16, direction)
    }

    /// Moves the caret one word left without Shift. A first press over a
    /// live selection collapses to its start (the GTK rule the arrows
    /// follow); only a second press hops.
    fn caret_word_left(&mut self) {
        if self.has_selection() {
            let (min, _) = self.selection_range();
            self.collapse_at(min);
            return;
        }
        self.extend_word_left();
        self.anchor_utf16 = self.caret_utf16;
    }

    /// Moves the caret one word left, keeping the anchor (a Shift+Alt+Left
    /// extension; a hop never slices a surrogate pair — the seam's edges
    /// are scalar boundaries).
    fn extend_word_left(&mut self) {
        self.caret_utf16 = self.word_hop(alacer_core::text_seam::WordDirection::Previous);
    }

    /// Moves the caret one word right without Shift — the mirror of
    /// [`Self::caret_word_left`]: collapse to the selection's end first.
    fn caret_word_right(&mut self) {
        if self.has_selection() {
            let (_, max) = self.selection_range();
            self.collapse_at(max);
            return;
        }
        self.extend_word_right();
        self.anchor_utf16 = self.caret_utf16;
    }

    /// Moves the caret one word right, keeping the anchor (a Shift+Alt+
    /// Right extension).
    fn extend_word_right(&mut self) {
        self.caret_utf16 = self.word_hop(alacer_core::text_seam::WordDirection::Next);
    }

    /// Deletes the hop's span behind the caret (Ctrl/Alt+Backspace, ADR-
    /// 0019): a word chord deletes exactly the span the caret would hop —
    /// the word, with the separator following in the next chord. An
    /// active selection is consumed whole first (the M11 edit rule).
    /// Answers the shape the edit landed — `NONE` when no word went.
    fn delete_word_back(&mut self) -> LandedEdit {
        if self.has_selection() {
            let (from, to) = self.selection_range();
            return self.edit(from, to, "");
        }
        let to = self.word_hop(alacer_core::text_seam::WordDirection::Previous);
        if to == self.caret_utf16 {
            return LandedEdit::NONE;
        }
        self.edit(to, self.caret_utf16, "")
    }

    /// Deletes the hop's span ahead of the caret (Ctrl/Alt+Delete), the
    /// mirror of [`Self::delete_word_back`] — the caret stays put.
    /// Answers the shape the edit landed, like its mirror.
    fn delete_word_forward(&mut self) -> LandedEdit {
        if self.has_selection() {
            let (from, to) = self.selection_range();
            return self.edit(from, to, "");
        }
        let to = self.word_hop(alacer_core::text_seam::WordDirection::Next);
        if to == self.caret_utf16 {
            return LandedEdit::NONE;
        }
        self.edit(self.caret_utf16, to, "")
    }

    /// Writes the active selection's span to the clipboard seam (ADR-0020):
    /// the exact UTF-16 range the edits consume, resolved through the same
    /// byte range the highlight paints. A degenerate selection copies
    /// nothing and returns `false` — a copy never invents content.
    ///
    /// Since M39 the round trip streams a
    /// [`alacer_core::ClipboardRecord`] (ADR-0045): the delivered write
    /// beside the payload's count, the no-selection and refused-write
    /// species named — the facts a trail reader routes on, queued where
    /// the round trip resolves.
    fn copy_selection(&self) -> bool {
        if !self.has_selection() {
            alacer_core::clipboard::note_round_trip(alacer_core::ClipboardRecord {
                action: alacer_core::ClipboardAction::Copy,
                outcome: alacer_core::ClipboardOutcome::Refused(
                    alacer_core::ClipboardRefusal::NoSelection,
                ),
                engine: None,
                read: None,
                chars: 0,
            });
            return false;
        }
        let (min, max) = self.selection_range();
        let range = utf16::byte_range(&self.text, min, max);
        let copied = &self.text[range];
        // The engine's reported write (M41, ADR-0047): the platform-side
        // fact rides the record beside the round trip's verdict — the
        // log answers "which clipboard failed it", not only "it failed".
        let fact = alacer_core::clipboard::with_clipboard(|seam| seam.write_reported(copied));
        let accepted = matches!(fact, alacer_core::clipboard::ClipboardWriteFact::Accepted);
        alacer_core::clipboard::note_round_trip(alacer_core::ClipboardRecord {
            action: alacer_core::ClipboardAction::Copy,
            outcome: if accepted {
                alacer_core::ClipboardOutcome::Delivered
            } else {
                alacer_core::ClipboardOutcome::Refused(alacer_core::ClipboardRefusal::WriteRefused)
            },
            engine: match fact {
                alacer_core::clipboard::ClipboardWriteFact::Accepted => None,
                alacer_core::clipboard::ClipboardWriteFact::Refused(refusal) => Some(refusal),
            },
            read: None,
            chars: if accepted { copied.chars().count() } else { 0 },
        });
        accepted
    }

    /// Moves the active selection to the clipboard seam, then deletes it
    /// (Ctrl/Cmd+X). The deletion only follows an accepted write: a
    /// clipboard that refused the write never deletes the selection — the
    /// refuse-don't-munge rule applied to a destructive edit. The
    /// deletion rides the funnel like every edit: its undo restores the
    /// moved-out span (ADR-0021).
    ///
    /// Since M39 the round trip streams its own fact (ADR-0045) before
    /// the deletion it drives lands — the trail reads clipboard → edit,
    /// the causal order the cut ran. The cut never rides the copy funnel:
    /// a cut that streamed a copy fact and a cut fact would double the
    /// round trip in the log.
    fn cut(&mut self) -> bool {
        if !self.has_selection() {
            alacer_core::clipboard::note_round_trip(alacer_core::ClipboardRecord {
                action: alacer_core::ClipboardAction::Cut,
                outcome: alacer_core::ClipboardOutcome::Refused(
                    alacer_core::ClipboardRefusal::NoSelection,
                ),
                engine: None,
                read: None,
                chars: 0,
            });
            return false;
        }
        let (min, max) = self.selection_range();
        let range = utf16::byte_range(&self.text, min, max);
        let moved = &self.text[range];
        // The engine's reported write (M41, ADR-0047): the platform-side
        // fact rides the record beside the round trip's verdict — the
        // cut's refusal names which clipboard failed it, the deletion
        // still following only an accepted write (ADR-0020's rule).
        let fact = alacer_core::clipboard::with_clipboard(|seam| seam.write_reported(moved));
        let accepted = matches!(fact, alacer_core::clipboard::ClipboardWriteFact::Accepted);
        if !accepted {
            alacer_core::clipboard::note_round_trip(alacer_core::ClipboardRecord {
                action: alacer_core::ClipboardAction::Cut,
                outcome: alacer_core::ClipboardOutcome::Refused(
                    alacer_core::ClipboardRefusal::WriteRefused,
                ),
                engine: match fact {
                    alacer_core::clipboard::ClipboardWriteFact::Refused(refusal) => Some(refusal),
                    alacer_core::clipboard::ClipboardWriteFact::Accepted => None,
                },
                read: None,
                chars: 0,
            });
            return false;
        }
        let chars = moved.chars().count();
        alacer_core::clipboard::note_round_trip(alacer_core::ClipboardRecord {
            action: alacer_core::ClipboardAction::Cut,
            outcome: alacer_core::ClipboardOutcome::Delivered,
            engine: None,
            read: None,
            chars,
        });
        let (from, to) = self.selection_span();
        self.edit(from, to, "");
        true
    }

    /// Inserts the clipboard's text at the caret (Ctrl/Cmd+V), sanitized
    /// through core's single-line paste rule — line breaks collapse to
    /// spaces ([`alacer_core::clipboard::paste_text`]) — and consuming an
    /// active selection first, like every editor (the M11 edit rule). An
    /// empty or refused read inserts nothing and returns `false`.
    ///
    /// Since M39 the round trip streams its own fact (ADR-0045) beside
    /// the insert it drives — the trail reads clipboard → edit, the
    /// payload's count naming what landed. Since M42 the paste rides
    /// the engine's reported read (ADR-0048): the read's own species
    /// rides the record beside the verdict — a paste that refused for
    /// the *read* names the species, a paste that refused for the
    /// *field* (a payload that sanitized to nothing) carries the read's
    /// absence — the M39 collapse split, one verdict and two facts.
    fn paste(&mut self) -> bool {
        // The engine's reported read (M42, ADR-0048): the read-side fact
        // rides the record beside the round trip's verdict — the log
        // answers "did the paste refuse for the read, or for the field".
        let fact = alacer_core::clipboard::with_clipboard(
            alacer_core::clipboard::ClipboardSeam::read_reported,
        );
        let (raw, read) = match fact {
            alacer_core::clipboard::ClipboardReadFact::Delivered(text) => (Some(text), None),
            alacer_core::clipboard::ClipboardReadFact::Refused(species) => (None, Some(species)),
        };
        let Some(raw) = raw else {
            alacer_core::clipboard::note_round_trip(alacer_core::ClipboardRecord {
                action: alacer_core::ClipboardAction::Paste,
                outcome: alacer_core::ClipboardOutcome::Refused(
                    alacer_core::ClipboardRefusal::EmptyClipboard,
                ),
                engine: None,
                read,
                chars: 0,
            });
            return false;
        };
        let sanitized = alacer_core::clipboard::paste_text(&raw);
        if sanitized.is_empty() {
            alacer_core::clipboard::note_round_trip(alacer_core::ClipboardRecord {
                action: alacer_core::ClipboardAction::Paste,
                outcome: alacer_core::ClipboardOutcome::Refused(
                    alacer_core::ClipboardRefusal::EmptyClipboard,
                ),
                engine: None,
                read: None,
                chars: 0,
            });
            return false;
        }
        let chars = sanitized.chars().count();
        alacer_core::clipboard::note_round_trip(alacer_core::ClipboardRecord {
            action: alacer_core::ClipboardAction::Paste,
            outcome: alacer_core::ClipboardOutcome::Delivered,
            engine: None,
            read: None,
            chars,
        });
        self.insert(&sanitized);
        true
    }
    /// The key's editing outcome: `(changed, consumed)` — whether value or
    /// caret moved (the masks ride), and whether the key is the field's at
    /// all.
    ///
    /// An unconsumed key stays Ignored (Tab cycling, ancestor
    /// accelerators). A consumed key that changed nothing — a refused
    /// clipboard read or write, a copy without a selection — is Handled
    /// with no masks: the refusal costs an idle frame, the honesty the
    /// action tier already pins (ADR-0020).
    ///
    /// Since M58 (ADR-0064) every editing arm notes the typing door's
    /// fact beside the mutation it landed — the match is the keyboard
    /// door's own narrative, each arm the key's whole story.
    #[expect(
        clippy::too_many_lines,
        reason = "the keyboard door's match is one readable key-by-key narrative; splitting it would hide the note sites the M58 facts ride"
    )]
    fn edit_key(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> (bool, bool) {
        let shift = event.modifiers.shift;
        let command = event.modifiers.control || event.modifiers.meta;
        // The word chords (ADR-0019): Alt on every platform (GTK's double
        // binding, Option on macOS) or the Ctrl/Meta command set.
        let word_chord = event.modifiers.alt || command;
        match &event.key {
            alacer_core::events::Key::Char(ch) => {
                if command {
                    self.command_chord(*ch, shift)
                } else {
                    let landed = self.insert(&ch.to_string());
                    self.note_key_edit(
                        alacer_core::KeyEditAction::Insert,
                        alacer_core::KeyEditDriver::Keyboard,
                        landed,
                    );
                    (true, true)
                }
            }
            alacer_core::events::Key::Backspace => {
                let landed = if word_chord {
                    self.delete_word_back()
                } else {
                    self.backspace()
                };
                let action = if word_chord {
                    alacer_core::KeyEditAction::WordDeleteBack
                } else {
                    alacer_core::KeyEditAction::Backspace
                };
                self.note_key_edit(action, alacer_core::KeyEditDriver::Keyboard, landed);
                (true, true)
            }
            alacer_core::events::Key::Delete => {
                let landed = if word_chord {
                    self.delete_word_forward()
                } else {
                    self.delete()
                };
                let action = if word_chord {
                    alacer_core::KeyEditAction::WordDeleteForward
                } else {
                    alacer_core::KeyEditAction::Delete
                };
                self.note_key_edit(action, alacer_core::KeyEditDriver::Keyboard, landed);
                (true, true)
            }
            // Arrows and Home/End: Shift extends from the anchor (M11), a
            // plain press over a live selection collapses to its near edge
            // first (GTK semantics), and only a second press steps. The
            // word chords (Alt; Ctrl/Meta) hop whole word runs (ADR-0019),
            // the same anchor and collapse rules riding along.
            alacer_core::events::Key::ArrowLeft => {
                if word_chord {
                    if shift {
                        self.extend_word_left();
                    } else {
                        self.caret_word_left();
                    }
                } else if shift {
                    self.extend_left();
                } else {
                    self.caret_left();
                }
                (true, true)
            }
            alacer_core::events::Key::ArrowRight => {
                if word_chord {
                    if shift {
                        self.extend_word_right();
                    } else {
                        self.caret_word_right();
                    }
                } else if shift {
                    self.extend_right();
                } else {
                    self.caret_right();
                }
                (true, true)
            }
            alacer_core::events::Key::Home => {
                if shift {
                    self.caret_utf16 = 0;
                } else {
                    self.collapse_at(0);
                }
                (true, true)
            }
            alacer_core::events::Key::End => {
                let len = utf16::len_utf16(&self.text);
                if shift {
                    self.caret_utf16 = len;
                } else {
                    self.collapse_at(len);
                }
                (true, true)
            }
            alacer_core::events::Key::Enter => {
                // The submit seam (M6): fire with the current value, then
                // clear. The widget mutates only itself (rule 1); the
                // handler's writes ride the batch the caller chose.
                if let Some(handler) = &mut self.on_submit {
                    let value = self.text.clone();
                    handler(&value);
                    self.text.clear();
                    self.caret_utf16 = 0;
                    ctx.request(
                        alacer_core::Change::PAINT
                            .union(alacer_core::Change::LAYOUT)
                            .union(alacer_core::Change::A11Y),
                    );
                    return (true, true);
                }
                (false, false)
            }
            // Everything else — Tab (focus cycling), Escape, the vertical
            // arrows, unknown keys — is not the field's.
            _ => (false, false),
        }
    }

    /// The command chord set (Ctrl/Cmd). Ctrl/Cmd+A selects the whole
    /// value; the clipboard tier (ADR-0020) copies, cuts, and pastes
    /// through the clipboard seam; the history tier (ADR-0021) undoes on
    /// Ctrl/Cmd+Z and redoes on its Shift and on the Ctrl/Cmd+Y alternate
    /// (the Win32/GTK binding); anything else is not the field's — the
    /// pre-M11 path *inserted* the character, which turned Ctrl+A into
    /// typing an 'a' (the bug this method's predecessor deletes).
    fn command_chord(&mut self, ch: char, shift: bool) -> (bool, bool) {
        match ch {
            'z' | 'Z' if shift => (self.redo(), true),
            'z' | 'Z' => (self.undo(), true),
            'y' | 'Y' => (self.redo(), true),
            'a' | 'A' => {
                self.select_all();
                (true, true)
            }
            'c' | 'C' => (self.copy_selection(), true),
            'x' | 'X' => (self.cut(), true),
            'v' | 'V' => (self.paste(), true),
            _ => (false, false),
        }
    }

    /// Selects the word under `at` (a double-click, ADR-0019): anchor at
    /// the word's start, caret at its end — tree vocabulary in the
    /// click's own reading order. A hit on a separator collapses at the
    /// hit point: a double-click never wanders to the nearest word.
    fn select_word_at(&mut self, at: usize) {
        let segments = self.word_segments();
        let word = segments
            .iter()
            .find(|segment| segment.word_like && segment.start <= at && at < segment.end);
        match word {
            Some(word) => {
                self.anchor_utf16 = word.start;
                self.caret_utf16 = word.end;
            }
            None => self.collapse_at(at),
        }
    }

    /// The painted run: committed text with the preedit spliced in at the
    /// caret — what the user sees and what the IME anchors to.
    /// The field's text color: the muted constant when the field is
    /// unavailable (M22, ADR-0028) or showing the placeholder, the text
    /// color once real content or a composition rides.
    fn field_text_color(&self, has_preedit: bool) -> Color {
        if !self.enabled || (self.text.is_empty() && !has_preedit) {
            crate::theme::TEXT_MUTED
        } else {
            crate::theme::TEXT
        }
    }

    fn display_parts(&self) -> (String, Option<String>, String) {
        match &self.preedit {
            Some((preedit, _)) => {
                let at = utf16::byte_offset(&self.text, self.caret_utf16);
                (
                    self.text[..at].to_string(),
                    Some(preedit.clone()),
                    self.text[at..].to_string(),
                )
            }
            None => (self.text.clone(), None, String::new()),
        }
    }

    /// Maps a pointer x (window coordinates) to a UTF-16 offset of the
    /// committed text: the resolved frame the router carried in converts
    /// it to field-local, the glyph inset moves it into the run's space,
    /// and the text seam's hit-test does the bounds-aware offset mapping
    /// (M12, ADR-0018). During a composition the committed text is what
    /// offsets map to — the preedit is overlay spliced at the caret, not
    /// a click target (the caret's placement decides where it re-splices).
    fn caret_offset_at(&self, ctx: &EventCtx, window_x: f32) -> usize {
        let local_x = ctx
            .bounds()
            .map_or(window_x, |frame| window_x - frame.origin.x);
        let run_x = local_x - TEXT_INSET;
        alacer_core::text_seam::with_seam(|seam| seam.hit_test(&self.text, self.font_size, run_x))
    }
}

impl Widget for TextInput {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::TextInput
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The M3 a11y contract: editable widgets expose their content (the
    /// committed value — preedit is composing, not committed).
    fn value(&self) -> Option<String> {
        Some(self.text.clone())
    }

    /// The availability tier (M22, ADR-0028): the flag the a11y tree
    /// carries — an unavailable input reports itself before the AT tries
    /// the edit it will refuse.
    fn enabled(&self) -> bool {
        self.enabled
    }

    /// The M11 a11y contract (ADR-0017): the focused input exposes its
    /// selection — anchor and caret, UTF-16 units — and nothing while a
    /// composition is in flight (the IME owns the canvas; a stale range
    /// would lie to the AT). Losing focus drops the report with it: the
    /// selection is focus's editing half.
    fn selection(&self) -> Option<TextSelection> {
        if self.focused && self.preedit.is_none() {
            Some(TextSelection::new(self.anchor_utf16, self.caret_utf16))
        } else {
            None
        }
    }

    fn style(&self) -> Style {
        Style {
            padding: 8.0,
            ..self.style.clone()
        }
    }

    fn ime_purpose(&self) -> ImePurpose {
        self.purpose
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            // Losing focus cancels any in-flight composition (the platform
            // detaches the IME; a stale preedit would be a ghost) and ends
            // any drag in flight — the pointer's gesture owner is the
            // focused field, and a selection extended past a blur would
            // keep moving under someone else's pointer.
            self.preedit = None;
            self.pressing = false;
        }
    }

    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        // The value tier (M11, ADR-0017): a faithful AT mapping, same masks
        // as a key edit. `DefaultPress` and the scroll/step tier are not an
        // input's contract — refused, never munged into an edit.
        // The availability tier (M22, ADR-0028): an unavailable input has
        // no write door — the AT's edit is refused honestly, never munged.
        // Since M55 (ADR-0061) every text write the AT asks streams one
        // [`alacer_core::TextWriteRecord`] — the answer's own species
        // where the `bool` collapsed the closed door and the answered
        // write into one silence, the refusal streaming too (the value
        // trail's shape, ADR-0046), the write's shape named by lengths
        // and never the payload (a field's value can carry a secret —
        // the clipboard trail's rule, ADR-0045).
        let tier = match action {
            A11yAction::SetValue(_) => alacer_core::TextWriteAction::SetValue,
            A11yAction::ReplaceSelectedText(_) => alacer_core::TextWriteAction::ReplaceSelection,
            A11yAction::SetTextSelection(_) => alacer_core::TextWriteAction::SetSelection,
            A11yAction::DefaultPress
            | A11yAction::Increment
            | A11yAction::Decrement
            | A11yAction::ScrollUp(_)
            | A11yAction::ScrollDown(_) => return false,
        };
        if !self.enabled {
            // The closed door's fact: the availability tier refused the
            // write before arithmetic (M22, ADR-0028) — the shape rides
            // the honest absence, the field's own selection untouched
            // beside it, the funnel's stamp naming the control the ask
            // arrived under (M43, ADR-0049).
            alacer_core::text_write::note_text_write(alacer_core::TextWriteRecord {
                action: tier,
                outcome: alacer_core::TextWriteOutcome::Refused(
                    alacer_core::TextWriteRefusal::Unavailable,
                ),
                replaced: None,
                inserted: None,
                requested: None,
                selection: TextSelection::new(self.anchor_utf16, self.caret_utf16),
                target: None,
                source: alacer_core::TextWriteSource::AtDoor,
            });
            return false;
        }
        match action {
            A11yAction::SetValue(value) => {
                // The whole-value write: the value the field carried is
                // the span the write consumed — the lengths name the
                // shape, the payload never rides the log — and the
                // journal resets beneath it (ADR-0021's reset rule: the
                // AT's write is an app write, undo never fights it).
                let replaced = utf16::len_utf16(&self.text);
                self.apply_value(value);
                alacer_core::text_write::note_text_write(alacer_core::TextWriteRecord {
                    action: tier,
                    outcome: alacer_core::TextWriteOutcome::Delivered,
                    replaced: Some(replaced),
                    inserted: Some(utf16::len_utf16(&self.text)),
                    requested: None,
                    selection: TextSelection::new(self.anchor_utf16, self.caret_utf16),
                    target: None,
                    source: alacer_core::TextWriteSource::AtDoor,
                });
                true
            }
            A11yAction::ReplaceSelectedText(value) => {
                // The paste-shaped write: the selection's span is the
                // span the write consumed, the payload's own length
                // beside it — the caret parks at the insertion's end
                // with the anchor collapsed onto it (the edit funnel's
                // own rule, ADR-0021).
                let (from, to) = self.selection_span();
                let replaced = to - from;
                self.insert(&value);
                alacer_core::text_write::note_text_write(alacer_core::TextWriteRecord {
                    action: tier,
                    outcome: alacer_core::TextWriteOutcome::Delivered,
                    replaced: Some(replaced),
                    inserted: Some(utf16::len_utf16(&value)),
                    requested: None,
                    selection: TextSelection::new(self.anchor_utf16, self.caret_utf16),
                    target: None,
                    source: alacer_core::TextWriteSource::AtDoor,
                });
                true
            }
            A11yAction::SetTextSelection(selection) => {
                // Forgiving read side: offsets floor to scalar boundaries
                // and clamp to the value's end, exactly like the IME's.
                // The ask rides beside the landing (the M44 rule,
                // ADR-0050, at the text tier): a floor or clamp the
                // trail could not show is one record's story now, the
                // write's shape riding the honest absence (a selection
                // consumes no span, inserts nothing).
                let len = utf16::len_utf16(&self.text);
                self.anchor_utf16 = utf16::floor_boundary(&self.text, selection.anchor.min(len));
                self.caret_utf16 = utf16::floor_boundary(&self.text, selection.focus.min(len));
                alacer_core::text_write::note_text_write(alacer_core::TextWriteRecord {
                    action: tier,
                    outcome: alacer_core::TextWriteOutcome::Delivered,
                    replaced: None,
                    inserted: None,
                    requested: Some(selection),
                    selection: TextSelection::new(self.anchor_utf16, self.caret_utf16),
                    target: None,
                    source: alacer_core::TextWriteSource::AtDoor,
                });
                true
            }
            // The tier match above routed the rest; unreachable here.
            _ => false,
        }
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(
            180.0,
            (self.font_size * crate::theme::LINE_HEIGHT).max(24.0) + 16.0,
        )))
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let (width, height) = (ctx.size().width, ctx.size().height);
        let border_color = if self.focused {
            crate::theme::ACCENT
        } else {
            crate::theme::BORDER
        };
        crate::paint_background(
            ctx,
            (width, height),
            crate::theme::PANEL,
            6.0,
            (border_color, if self.focused { 2.0 } else { 1.0 }),
        );

        let (before, preedit, after) = self.display_parts();

        alacer_core::text_seam::with_seam(|seam| {
            let text_color = self.field_text_color(preedit.is_some());

            // The selection highlight (M11, ADR-0017): one quad behind the
            // selected committed range, from the prefix widths the seam
            // already measures for the caret. Only while focused and not
            // composing — the IME owns the canvas during composition, and
            // its preedit splices committed-text offsets anyway.
            if self.focused && self.preedit.is_none() && self.has_selection() {
                let (min, max) = self.selection_range();
                let from = utf16::byte_offset(&self.text, min);
                let to = utf16::byte_offset(&self.text, max);
                let left = seam
                    .layout_glyphs(&self.text[..from], self.font_size, None)
                    .width;
                let right = seam
                    .layout_glyphs(&self.text[..to], self.font_size, None)
                    .width;
                let top = (height - self.font_size) * 0.5;
                ctx.push_quad(alacer_core::scene::Quad::filled(
                    alacer_core::geom::Rect::new(
                        Point::new(TEXT_INSET + left, top),
                        alacer_core::geom::Size::new(right - left, self.font_size),
                    ),
                    crate::theme::SELECTION,
                ));
            }

            // The visible text: committed prefix + composing preedit +
            // committed suffix (the preedit composes *inline* at the caret).
            let visible = format!("{}{}{}", before, preedit.clone().unwrap_or_default(), after);
            let empty = visible.is_empty();
            let source_text = if empty {
                self.placeholder.clone()
            } else {
                visible
            };
            if !source_text.is_empty() {
                let layout = seam.layout_glyphs(&source_text, self.font_size, Some(width - 4.0));
                for glyph in &layout.glyphs {
                    ctx.push_glyph(alacer_core::scene::Glyph {
                        position: Point::new(TEXT_INSET + glyph.x, glyph.y),
                        size: glyph.size,
                        color: text_color,
                        font_id: glyph.font_id,
                        glyph_id: glyph.glyph_id,
                        clip: None,
                    });
                }
            }

            // The caret: a 2px accent bar at the insertion point, drawn only
            // while focused (§10.3: the caret is focus's visible half).
            if self.focused {
                let caret_x = if empty {
                    TEXT_INSET
                } else {
                    let prefix = format!(
                        "{before}{}",
                        preedit.as_ref().map_or(String::new(), |p| {
                            // During composition the caret sits at the IME's own
                            // cursor within the preedit (UTF-16 units of it).
                            let cursor = self.preedit.as_ref().and_then(|(_, c)| *c).unwrap_or(0);
                            let at = utf16::byte_offset(p, cursor);
                            p[..at].to_string()
                        })
                    );
                    seam.layout_glyphs(&prefix, self.font_size, None).width
                };
                let caret_top = (height - self.font_size) * 0.5;
                ctx.push_quad(alacer_core::scene::Quad::filled(
                    alacer_core::geom::Rect::new(
                        Point::new(TEXT_INSET + caret_x, caret_top),
                        alacer_core::geom::Size::new(2.0, self.font_size),
                    ),
                    crate::theme::ACCENT,
                ));
                // The preedit underline: composing text is visually distinct.
                if preedit.is_some() {
                    let preedit_width = seam
                        .layout_glyphs(preedit.as_deref().unwrap_or(""), self.font_size, None)
                        .width;
                    ctx.push_quad(alacer_core::scene::Quad::filled(
                        alacer_core::geom::Rect::new(
                            Point::new(
                                TEXT_INSET
                                    + seam.layout_glyphs(&before, self.font_size, None).width,
                                caret_top + self.font_size + 1.0,
                            ),
                            alacer_core::geom::Size::new(preedit_width, 1.5),
                        ),
                        crate::theme::ACCENT,
                    ));
                }
            }
        });
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        // The wall (M22, ADR-0028): a disabled input consumes the press —
        // the caret gesture never starts, no focus, no masks.
        if !self.enabled && matches!(event, PointerEvent::Down { .. } | PointerEvent::Up { .. }) {
            return EventResponse::Handled;
        }
        match event {
            PointerEvent::Down {
                position,
                click_count,
                ..
            } => {
                // The M12 contract (ADR-0018): the caret answers the
                // pointer. The click's window x maps through the text
                // seam's hit-test into a UTF-16 offset of the committed
                // text — the same prefix widths the caret and the highlight
                // already measure, inverted — and the fresh click is a
                // caret gesture: anchor and caret collapse onto the hit
                // point (the pre-M12 honest fallback is deleted; the
                // recorded hit-test seam is now real). The focus request
                // is the click-to-focus seam (§10.3), unchanged.
                let offset = self.caret_offset_at(ctx, position.x);
                if *click_count > 1 {
                    // The M13 word contract (ADR-0019): the second press
                    // of a double-click selects the word under the hit —
                    // anchor at its start, caret at its end, the same
                    // anchor+caret vocabulary the AT reads. A separator
                    // hit collapses (a double-click never wanders).
                    self.select_word_at(offset);
                } else {
                    self.caret_utf16 = offset;
                    self.anchor_utf16 = offset;
                }
                self.pressing = true;
                ctx.request(
                    alacer_core::Change::PAINT
                        .union(alacer_core::Change::A11Y)
                        .union(alacer_core::Change::FOCUS),
                );
                EventResponse::Handled
            }
            PointerEvent::Move { position } => {
                // A drag (press held) extends the selection from the
                // anchor — the M11 anchor state is the pointer's drag
                // anchor too. An unpressed move is hover, never selection.
                // The routed Move may land outside the field (the drag left
                // the frame): the routing seam's contains-test is the
                // delivery contract, so out-of-frame moves simply do not
                // arrive — no state to guard here.
                if !self.pressing {
                    return EventResponse::Ignored;
                }
                self.caret_utf16 = self.caret_offset_at(ctx, position.x);
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
                EventResponse::Handled
            }
            PointerEvent::Up { .. } => {
                // The release ends the gesture; the selection it built is
                // a state, not a gesture, and survives until an edit or a
                // fresh click consumes it.
                self.pressing = false;
                EventResponse::Handled
            }
            PointerEvent::Leave | PointerEvent::Wheel { .. } => EventResponse::Ignored,
        }
    }

    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        // The availability tier (M22, ADR-0028): the keyboard never
        // reaches an unavailable input — a routed edit key is ignored so
        // the framework keeps it (Tab cycling).
        if !self.enabled {
            return EventResponse::Ignored;
        }
        // Editing acts on presses only (§10.3); preedit state makes keys
        // inert — the IME owns the composition until commit/cancel.
        if event.state != alacer_core::events::KeyState::Pressed || self.preedit.is_some() {
            return EventResponse::Ignored;
        }
        let (changed, consumed) = self.edit_key(ctx, event);
        if changed {
            // Value and/or caret moved: repaint the field; value changes are
            // a11y-visible too.
            ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        }
        if consumed {
            EventResponse::Handled
        } else {
            EventResponse::Ignored
        }
    }

    fn on_ime_event(&mut self, ctx: &mut EventCtx, event: &ImeEvent) -> EventResponse {
        // The availability tier (M22, ADR-0028): an unavailable input
        // answers no composition — the IME event is ignored (nothing else
        // consumes it; the framework drops it).
        if !self.enabled {
            return EventResponse::Ignored;
        }
        match event {
            // Composition in progress: show the marked text inline. An empty
            // preedit cancels the composition (winit's synthetic clear).
            // A11Y rides the request (M11): the field's composing state is
            // part of its accessible state — the selection report drops
            // while the IME owns the canvas, and re-arms after commit.
            ImeEvent::Preedit { text, cursor_utf16 } => {
                if text.is_empty() {
                    self.preedit = None;
                } else {
                    self.preedit = Some((text.clone(), *cursor_utf16));
                }
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
                EventResponse::Handled
            }
            // Final text: insert at the caret, clear the preedit. Commits may
            // carry surrogate pairs — `insert` counts UTF-16 units. Since
            // M58 (ADR-0064) the commit's fact rides the key-edit trail —
            // the composition's shape beside the landing, the IME driver
            // named (the attribution rule at the typing tier).
            ImeEvent::Commit { text } => {
                self.preedit = None;
                if !text.is_empty() {
                    let landed = self.insert(text);
                    self.note_key_edit(
                        alacer_core::KeyEditAction::Insert,
                        alacer_core::KeyEditDriver::Ime,
                        landed,
                    );
                }
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
                EventResponse::Handled
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TextInput;
    use alacer_core::events::{
        EventCtx, ImeEvent, Key, KeyEvent, KeyState, Modifiers, PointerEvent,
    };
    use alacer_core::geom::Point;
    use alacer_core::passes::ImePurpose;
    use alacer_core::utf16;
    use alacer_core::widget::{EventResponse, Role, Widget};

    fn key(key: Key) -> KeyEvent {
        KeyEvent {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    fn type_char(input: &mut TextInput, ch: char) {
        let mut ctx = EventCtx::new();
        let event = KeyEvent::char(ch);
        let _ = input.on_key_event(&mut ctx, &event);
    }

    #[test]
    fn role_name_and_value_satisfy_the_contract() {
        let input = TextInput::new("Search");
        assert_eq!(input.role(), Role::TextInput);
        assert_eq!(input.label().resolve(), "Search");
        assert_eq!(input.value(), "");
        assert!(input.focusable());
        assert_eq!(input.ime_purpose(), ImePurpose::Text);
    }

    #[test]
    fn enter_submits_then_clears_the_field() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
        let sink = std::rc::Rc::clone(&seen);
        let mut input = TextInput::new("Add a todo")
            .on_submit(move |value| sink.borrow_mut().push(value.to_string()));
        for ch in "write docs".chars() {
            let mut ctx = EventCtx::new();
            let _ = input.on_key_event(&mut ctx, &KeyEvent::char(ch));
        }
        let mut ctx = EventCtx::new();
        let enter = KeyEvent {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert_eq!(input.on_key_event(&mut ctx, &enter), EventResponse::Handled);
        assert_eq!(seen.borrow().as_slice(), ["write docs"]);
        assert_eq!(input.value(), "", "the field clears after submit");
        assert_eq!(input.caret_utf16(), 0);
        let masks = ctx.take();
        assert!(
            masks.contains(alacer_core::Change::A11Y) && masks.contains(alacer_core::Change::PAINT),
            "submit repaints the cleared field and re-announces, got {masks:?}"
        );
    }

    #[test]
    fn enter_without_a_handler_stays_ignored() {
        let mut input = TextInput::new("Search");
        let mut ctx = EventCtx::new();
        let enter = KeyEvent {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert_eq!(input.on_key_event(&mut ctx, &enter), EventResponse::Ignored);
    }

    #[test]
    fn typing_builds_ascii_and_cjk_values() {
        let mut input = TextInput::new("Search");
        for ch in "héllo".chars() {
            type_char(&mut input, ch);
        }
        assert_eq!(input.value(), "héllo");
        assert_eq!(input.caret_utf16(), 5);

        let mut cjk = TextInput::new("Input");
        for ch in "にほんご".chars() {
            type_char(&mut cjk, ch);
        }
        assert_eq!(cjk.value(), "にほんご");
        // CJK BMP runes: one UTF-16 unit each.
        assert_eq!(cjk.caret_utf16(), 4);
    }

    #[test]
    fn backspace_never_slices_a_surrogate_pair() {
        let mut input = TextInput::new("Emoji");
        type_char(&mut input, '\u{1F600}'); // 😀 = 2 UTF-16 units
        type_char(&mut input, 'x');
        assert_eq!(input.caret_utf16(), 3);

        // Backspace from the end removes 'x' (floor 3-1=2 is the pair end).
        let mut ctx = EventCtx::new();
        let _ = input.on_key_event(&mut ctx, &key(Key::Backspace));
        assert_eq!(input.value(), "\u{1F600}");

        // Backspace again removes the whole pair — never half of it.
        let mut ctx = EventCtx::new();
        let _ = input.on_key_event(&mut ctx, &key(Key::Backspace));
        assert_eq!(input.value(), "");
        assert_eq!(input.caret_utf16(), 0);
    }

    #[test]
    fn arrows_walk_scalar_boundaries() {
        let mut input = TextInput::new("Input");
        for ch in "a\u{1F600}b".chars() {
            type_char(&mut input, ch);
        }
        assert_eq!(input.caret_utf16(), 4);

        let mut ctx = EventCtx::new();
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowLeft));
        assert_eq!(input.caret_utf16(), 3, "'b' start");
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowLeft));
        assert_eq!(input.caret_utf16(), 1, "floors to the pair's start");
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowLeft));
        assert_eq!(input.caret_utf16(), 0);
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowLeft));
        assert_eq!(input.caret_utf16(), 0, "clamped at home");

        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowRight));
        assert_eq!(input.caret_utf16(), 1);
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowRight));
        assert_eq!(input.caret_utf16(), 3, "skips the mid-pair position");
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowRight));
        assert_eq!(input.caret_utf16(), 4);
        let _ = input.on_key_event(&mut ctx, &key(Key::ArrowRight));
        assert_eq!(input.caret_utf16(), 4, "clamped at end");

        let _ = input.on_key_event(&mut ctx, &key(Key::Home));
        assert_eq!(input.caret_utf16(), 0);
        let _ = input.on_key_event(&mut ctx, &key(Key::End));
        assert_eq!(input.caret_utf16(), 4);
    }

    #[test]
    fn ime_preedit_composes_and_cancels() {
        let mut input = TextInput::new("Input");
        let mut ctx = EventCtx::new();
        let preedit = ImeEvent::Preedit {
            text: "にほ".to_string(),
            cursor_utf16: Some(2),
        };
        assert_eq!(
            input.on_ime_event(&mut ctx, &preedit),
            EventResponse::Handled
        );
        assert_eq!(
            input.preedit().map(|(t, c)| (t.as_str(), *c)),
            Some(("にほ", Some(2)))
        );
        assert_eq!(input.value(), "", "preedit is composing, not committed");

        // Empty preedit cancels.
        let cancel = ImeEvent::Preedit {
            text: String::new(),
            cursor_utf16: None,
        };
        let _ = input.on_ime_event(&mut ctx, &cancel);
        assert_eq!(input.preedit(), None);
        assert_eq!(input.value(), "");
    }

    #[test]
    fn ime_commit_inserts_at_the_caret_and_clears_preedit() {
        let mut input = TextInput::new("Input");
        type_char(&mut input, 'a');
        let mut ctx = EventCtx::new();
        let _ = input.on_ime_event(
            &mut ctx,
            &ImeEvent::Preedit {
                text: "ん".to_string(),
                cursor_utf16: Some(1),
            },
        );
        let commit = ImeEvent::Commit {
            text: "日本".to_string(),
        };
        let _ = input.on_ime_event(&mut ctx, &commit);
        assert_eq!(input.value(), "a日本");
        assert_eq!(input.preedit(), None);
        // a=1 unit + 日本=2 units.
        assert_eq!(input.caret_utf16(), 3);
    }

    #[test]
    fn commit_with_surrogate_pairs_counts_utf16() {
        let mut input = TextInput::new("Input");
        let mut ctx = EventCtx::new();
        let _ = input.on_ime_event(
            &mut ctx,
            &ImeEvent::Commit {
                text: "\u{1F600}".to_string(), // 😀 = 4 UTF-8 bytes, 2 UTF-16 units
            },
        );
        assert_eq!(input.value(), "\u{1F600}");
        assert_eq!(input.caret_utf16(), 2);
        assert_eq!(utf16::len_utf16(input.value()), 2);
    }

    #[test]
    fn keys_are_inert_while_composing() {
        let mut input = TextInput::new("Input");
        let mut ctx = EventCtx::new();
        let _ = input.on_ime_event(
            &mut ctx,
            &ImeEvent::Preedit {
                text: "ん".to_string(),
                cursor_utf16: Some(1),
            },
        );
        // A character press during composition is ignored — the IME owns it.
        assert_eq!(
            input.on_key_event(&mut ctx, &KeyEvent::char('x')),
            EventResponse::Ignored
        );
        assert_eq!(input.value(), "");
    }

    #[test]
    fn losing_focus_cancels_the_composition() {
        let mut input = TextInput::new("Input");
        let mut ctx = EventCtx::new();
        let _ = input.on_ime_event(
            &mut ctx,
            &ImeEvent::Preedit {
                text: "ん".to_string(),
                cursor_utf16: Some(1),
            },
        );
        input.set_focused(true);
        input.set_focused(false);
        assert_eq!(input.preedit(), None, "no ghost preedit after blur");
    }

    #[test]
    fn secure_inputs_request_the_secure_ime() {
        let input = TextInput::new("Password").secure();
        assert_eq!(input.ime_purpose(), ImePurpose::Secure);
    }

    #[test]
    fn set_value_moves_the_caret_to_the_end() {
        let mut input = TextInput::new("Search");
        input.set_value("にほんご");
        assert_eq!(input.value(), "にほんご");
        assert_eq!(input.caret_utf16(), 4);
    }

    /// A deterministic seam: 10 px per character, no glyphs — the exact
    /// prefix widths the hit-test inverts, so caret assertions are exact.
    /// Its segmentation is the alphanumeric-run rule, exact for the ASCII
    /// values these tests edit (`hello world`); the real tables ride the
    /// engine (alacer-text).
    struct FixedWidthStub;

    impl alacer_core::text_seam::TextSeam for FixedWidthStub {
        fn measure(
            &self,
            _spec: &alacer_core::style::TextSpec,
            _max_width: Option<f32>,
        ) -> alacer_core::geom::Size {
            alacer_core::geom::Size::new(10.0, 10.0)
        }

        fn layout_glyphs(
            &self,
            text: &str,
            _font_size: f32,
            _max_width: Option<f32>,
        ) -> alacer_core::text_seam::GlyphRun {
            alacer_core::text_seam::GlyphRun {
                width: f32::from(u16::try_from(text.chars().count()).unwrap_or(u16::MAX)) * 10.0,
                height: 10.0,
                glyphs: Vec::new(),
            }
        }

        fn word_segments(&self, text: &str) -> Vec<alacer_core::text_seam::WordSegment> {
            let mut segments = Vec::new();
            let (mut start, mut word_like, mut units) = (0_usize, false, 0_usize);
            for ch in text.chars() {
                let like = ch.is_alphanumeric();
                if units > 0 && like != word_like {
                    segments.push(alacer_core::text_seam::WordSegment {
                        start,
                        end: units,
                        word_like,
                    });
                    start = units;
                }
                word_like = like;
                units += ch.len_utf16();
            }
            if units > start {
                segments.push(alacer_core::text_seam::WordSegment {
                    start,
                    end: units,
                    word_like,
                });
            }
            segments
        }
    }

    /// The resolved frame every routed pointer event carries (M12's core
    /// growth): the field sits at window (100, 50), 40 px wide.
    fn field_frame() -> alacer_core::geom::Rect {
        alacer_core::geom::Rect::new(
            Point::new(100.0, 50.0),
            alacer_core::geom::Size::new(40.0, 24.0),
        )
    }

    fn down(x: f32) -> PointerEvent {
        PointerEvent::Down {
            position: Point::new(x, 62.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 1,
        }
    }

    fn mv(x: f32) -> PointerEvent {
        PointerEvent::Move {
            position: Point::new(x, 62.0),
        }
    }

    fn up(x: f32) -> PointerEvent {
        PointerEvent::Up {
            position: Point::new(x, 62.0),
            button: alacer_core::events::MouseButton::Left,
        }
    }

    /// A focused input in a frame at window (100, 50), with the stub seam
    /// installed on this test's thread.
    fn stubbed_focused_input(content: &str) -> TextInput {
        alacer_core::text_seam::install(std::rc::Rc::new(FixedWidthStub));
        let mut input = TextInput::new("Search");
        input.set_value(content);
        input.set_focused(true);
        input
    }

    #[test]
    fn a_click_moves_the_caret_to_the_nearest_boundary() {
        let mut input = stubbed_focused_input("abcd");
        let mut ctx = EventCtx::with_phase_and_bounds(
            alacer_core::events::EventPhase::Bubble,
            Some(field_frame()),
        );

        // Window x = 125 → local 25 → run x 23: boundaries at 0/10/20/30/40,
        // nearest is 2 (width 20) — the caret parks there, anchor collapses.
        let response = input.on_pointer_event(&mut ctx, &down(125.0));
        assert_eq!(response, EventResponse::Handled);
        assert_eq!(input.caret_utf16(), 2, "the click hit maps to offset 2");
        assert_eq!(
            input.selection().map(|s| s.normalized()),
            Some((2, 2)),
            "a fresh click is a degenerate selection at the hit point"
        );
        let masks = ctx.take();
        assert!(
            masks.contains(alacer_core::Change::FOCUS),
            "the click still asks for focus (click-to-focus), got {masks:?}"
        );
        assert!(
            masks.contains(alacer_core::Change::PAINT) && masks.contains(alacer_core::Change::A11Y),
            "the caret move repaints and re-announces, got {masks:?}"
        );
    }

    #[test]
    fn a_drag_extends_the_selection_from_the_anchor() {
        let mut input = stubbed_focused_input("abcd");

        let mut ctx = EventCtx::with_phase_and_bounds(
            alacer_core::events::EventPhase::Bubble,
            Some(field_frame()),
        );
        let _ = input.on_pointer_event(&mut ctx, &down(125.0));
        assert_eq!(input.caret_utf16(), 2);

        // Move while the press is held: the anchor stays at 2, the caret
        // rides to the drag's hit — window 145 → local 45 → run 43 → end.
        let _ = input.on_pointer_event(&mut ctx, &mv(145.0));
        assert_eq!(
            input.selection().map(|s| s.normalized()),
            Some((2, 4)),
            "the drag selects from the anchor to the drag's hit"
        );
        let masks = ctx.take();
        assert!(
            masks.contains(alacer_core::Change::PAINT) && masks.contains(alacer_core::Change::A11Y),
            "the selection is paint and a11y vocabulary, got {masks:?}"
        );

        // The release ends the gesture but the range survives; a later
        // unpressed move must not extend it.
        let _ = input.on_pointer_event(&mut ctx, &up(145.0));
        let _ = input.on_pointer_event(&mut ctx, &mv(105.0));
        assert_eq!(
            input.selection().map(|s| s.normalized()),
            Some((2, 4)),
            "the release froze the range; hover never extends it"
        );
    }

    #[test]
    fn a_leftward_drag_selects_backwards_with_the_anchor_at_the_press() {
        let mut input = stubbed_focused_input("abcd");
        let mut ctx = EventCtx::with_phase_and_bounds(
            alacer_core::events::EventPhase::Bubble,
            Some(field_frame()),
        );
        let _ = input.on_pointer_event(&mut ctx, &down(145.0));
        let _ = input.on_pointer_event(&mut ctx, &mv(105.0));
        let selection = input.selection().expect("focused and not composing");
        assert_eq!(selection.anchor, 4, "the anchor is where the press landed");
        assert_eq!(selection.focus, 0, "the caret is where the drag ended");
        assert_eq!(selection.normalized(), (0, 4), "normalized ascending");
    }

    #[test]
    fn an_empty_field_clicks_to_the_start() {
        let mut input = stubbed_focused_input("");
        let mut ctx = EventCtx::with_phase_and_bounds(
            alacer_core::events::EventPhase::Bubble,
            Some(field_frame()),
        );
        let _ = input.on_pointer_event(&mut ctx, &down(125.0));
        assert_eq!(input.caret_utf16(), 0, "an empty value has one boundary");
    }

    #[test]
    fn a_click_without_a_resolved_frame_treats_x_as_local() {
        let mut input = stubbed_focused_input("abcd");
        // Direct dispatch without routing (harness probes, replay): no
        // frame arrived, so the x is already field-local — 23 → offset 2.
        let mut ctx = EventCtx::new();
        let _ = input.on_pointer_event(&mut ctx, &down(23.0));
        assert_eq!(input.caret_utf16(), 2);
    }

    // ---- The word contract (§10.3, ADR-0019) ---------------------------

    /// An Alt press (word movement; Option on macOS).
    fn alt(key: Key) -> KeyEvent {
        KeyEvent {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers {
                alt: true,
                ..Modifiers::default()
            },
            text: None,
        }
    }

    /// A Ctrl press (word movement and word deletion on Linux/Windows).
    fn ctrl(key: Key) -> KeyEvent {
        KeyEvent {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers {
                control: true,
                ..Modifiers::default()
            },
            text: None,
        }
    }

    /// Shift+Alt (word-wise extension).
    fn shift_alt(key: Key) -> KeyEvent {
        KeyEvent {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers {
                shift: true,
                alt: true,
                ..Modifiers::default()
            },
            text: None,
        }
    }

    fn edit(input: &mut TextInput, event: &KeyEvent) -> EventResponse {
        let mut ctx = EventCtx::new();
        input.on_key_event(&mut ctx, event)
    }

    #[test]
    fn alt_arrows_hop_word_runs() {
        // "hello world" — hop to word ends, then word starts; separators
        // are never landings (the stub's alphanumeric segmentation is
        // exact for this ASCII value).
        let mut input = stubbed_focused_input("hello world");
        assert_eq!(input.caret_utf16(), 11);

        assert_eq!(
            edit(&mut input, &alt(Key::ArrowLeft)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 6, "end -> world's start");
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowLeft)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 0, "world's start -> hello's start");
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowLeft)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 0, "the run's start clamps");

        assert_eq!(
            edit(&mut input, &alt(Key::ArrowRight)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 5, "start -> hello's end");
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowRight)),
            EventResponse::Handled
        );
        assert_eq!(
            input.caret_utf16(),
            6,
            "hello's end -> world's start, one hop"
        );
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowRight)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 11, "world's end");
    }

    #[test]
    fn alt_arrows_mid_word_hop_to_word_edges() {
        let mut input = stubbed_focused_input("hello world");
        input.collapse_at(2); // hel|lo world
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowRight)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 5, "mid-word -> the word's end");
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowRight)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 6);
        input.collapse_at(8); // hello w|orld
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowLeft)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 6, "mid-word -> the word's start");
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowLeft)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 0);
    }

    #[test]
    fn a_plain_word_arrow_collapses_the_selection_first() {
        // The GTK rule the arrows already follow: a plain press over a
        // live selection collapses to the near edge and does not move;
        // the second press hops.
        let mut input = stubbed_focused_input("hello world");
        input.anchor_utf16 = 0;
        input.caret_utf16 = 5;
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowRight)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 5, "collapsed to the selection's end");
        assert_eq!(input.anchor_utf16, 5);
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowRight)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 6, "the second press hops");

        let mut input = stubbed_focused_input("hello world");
        input.anchor_utf16 = 0;
        input.caret_utf16 = 5;
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowLeft)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 0, "collapsed to the selection's start");
        assert_eq!(
            edit(&mut input, &alt(Key::ArrowLeft)),
            EventResponse::Handled
        );
        assert_eq!(input.caret_utf16(), 0, "clamped at home");
    }

    #[test]
    fn shift_alt_arrows_extend_from_the_anchor() {
        let mut input = stubbed_focused_input("hello world");
        input.collapse_at(6);
        assert_eq!(
            edit(&mut input, &shift_alt(Key::ArrowRight)),
            EventResponse::Handled
        );
        assert_eq!(
            input.selection().map(|s| s.normalized()),
            Some((6, 11)),
            "the anchor stays, the caret rides the word hop"
        );
        assert_eq!(
            edit(&mut input, &shift_alt(Key::ArrowLeft)),
            EventResponse::Handled
        );
        assert_eq!(
            input.selection().map(|s| s.normalized()),
            Some((6, 6)),
            "back to the anchor collapses the range"
        );
        assert_eq!(
            edit(&mut input, &shift_alt(Key::ArrowLeft)),
            EventResponse::Handled
        );
        assert_eq!(
            input.selection().map(|s| s.normalized()),
            Some((0, 6)),
            "left past the anchor extends backwards"
        );
    }

    #[test]
    fn word_chords_delete_the_hops_span() {
        // The uniform rule (ADR-0019): a word chord deletes exactly the
        // span the caret would hop. From the value's end the hop lands on
        // `world`'s start — the word goes, the separator stays for the
        // next chord, exactly like the caret's own two hops.
        let mut input = stubbed_focused_input("hello world");
        assert_eq!(
            edit(&mut input, &ctrl(Key::Backspace)),
            EventResponse::Handled
        );
        assert_eq!(input.value(), "hello ", "the hop's span went: [6, 11)");
        assert_eq!(input.caret_utf16(), 6);
        assert_eq!(
            edit(&mut input, &ctrl(Key::Backspace)),
            EventResponse::Handled
        );
        assert_eq!(input.value(), "", "hello went in the second chord");
        assert_eq!(input.caret_utf16(), 0);

        // Alt+Backspace is the same chord set (GTK binds both).
        let mut input = stubbed_focused_input("hello world");
        assert_eq!(
            edit(&mut input, &alt(Key::Backspace)),
            EventResponse::Handled
        );
        assert_eq!(input.value(), "hello ");

        // Ctrl+Delete deletes forward, the caret stays. From the value's
        // start the hop lands on `hello`'s end; the separator follows in
        // the next chord, mirroring the backward direction.
        let mut input = stubbed_focused_input("hello world");
        input.collapse_at(0);
        assert_eq!(edit(&mut input, &ctrl(Key::Delete)), EventResponse::Handled);
        assert_eq!(input.value(), " world", "hello went; the caret stayed at 0");
        assert_eq!(input.caret_utf16(), 0);
        assert_eq!(edit(&mut input, &ctrl(Key::Delete)), EventResponse::Handled);
        assert_eq!(
            input.value(),
            "world",
            "the separator went in the second chord"
        );
        assert_eq!(edit(&mut input, &ctrl(Key::Delete)), EventResponse::Handled);
        assert_eq!(input.value(), "", "world went in the third");
    }

    #[test]
    fn word_chords_consume_a_selection_whole() {
        // The M11 edit rule stands: every edit consumes the selection
        // whole — a word chord over a range deletes the range, not a hop.
        let mut input = stubbed_focused_input("hello world");
        input.anchor_utf16 = 0;
        input.caret_utf16 = 5;
        assert_eq!(
            edit(&mut input, &ctrl(Key::Backspace)),
            EventResponse::Handled
        );
        assert_eq!(
            input.value(),
            " world",
            "the selection went, not a word hop"
        );
        assert_eq!(input.caret_utf16(), 0);
    }

    #[test]
    fn word_chords_are_inert_while_composing() {
        // The IME owns the canvas (§10.3) — word chords are keys like any
        // other and stay inert while a preedit composes.
        let mut input = stubbed_focused_input("hello");
        input.preedit = Some(("世界".to_string(), Some(2)));
        let _ = edit(&mut input, &alt(Key::ArrowRight));
        assert_eq!(input.caret_utf16(), 5, "the composition absorbed the chord");
        let _ = edit(&mut input, &ctrl(Key::Backspace));
        assert_eq!(input.value(), "hello", "no edit under a composition");
    }

    #[test]
    fn a_double_click_selects_the_word_under_it() {
        let mut input = stubbed_focused_input("hello world");
        let mut ctx = EventCtx::with_phase_and_bounds(
            alacer_core::events::EventPhase::Bubble,
            Some(field_frame()),
        );

        // A double-click on "world" (window x 172 → local 72 → run 70 →
        // hit offset 7) selects the word: anchor at its start, caret at
        // its end — tree vocabulary in the click's own reading order.
        let click = PointerEvent::Down {
            position: Point::new(172.0, 62.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 2,
        };
        assert_eq!(
            input.on_pointer_event(&mut ctx, &click),
            EventResponse::Handled
        );
        assert_eq!(
            input.selection().map(|s| s.normalized()),
            Some((6, 11)),
            "the word under the click selects"
        );
        let selection = input.selection().expect("focused and not composing");
        assert_eq!(selection.anchor, 6, "the anchor is the word's start");
        assert_eq!(selection.focus, 11, "the caret is the word's end");
        let masks = ctx.take();
        assert!(
            masks.contains(alacer_core::Change::FOCUS)
                && masks.contains(alacer_core::Change::PAINT)
                && masks.contains(alacer_core::Change::A11Y),
            "the double-click still asks for focus and re-announces, got {masks:?}"
        );

        // A double-click on a separator (" ") collapses at the hit — there
        // is no word to select (the honest refuse: never the nearest word).
        let mut input = stubbed_focused_input("hello world");
        let mut ctx = EventCtx::with_phase_and_bounds(
            alacer_core::events::EventPhase::Bubble,
            Some(field_frame()),
        );
        let on_space = PointerEvent::Down {
            position: Point::new(152.0, 62.0), // local 52 → run 50 → offset 5, the space's start
            button: alacer_core::events::MouseButton::Left,
            click_count: 2,
        };
        let _ = input.on_pointer_event(&mut ctx, &on_space);
        assert_eq!(
            input.selection().map(|s| s.normalized()),
            Some((5, 5)),
            "a separator double-click collapses at the hit point"
        );
    }

    #[test]
    fn a_double_click_over_a_selection_replaces_it() {
        // The double-click is a fresh gesture: whatever selection the
        // anchor state carried, the word under the click replaces it.
        let mut input = stubbed_focused_input("hello world");
        input.anchor_utf16 = 0;
        input.caret_utf16 = 11;
        let mut ctx = EventCtx::with_phase_and_bounds(
            alacer_core::events::EventPhase::Bubble,
            Some(field_frame()),
        );
        let click = PointerEvent::Down {
            position: Point::new(172.0, 62.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 2,
        };
        let _ = input.on_pointer_event(&mut ctx, &click);
        assert_eq!(
            input.selection().map(|s| s.normalized()),
            Some((6, 11)),
            "the word selection replaced the stale range"
        );
    }

    // ---- The clipboard tier (§10.3, ADR-0020) --------------------------

    /// Installs the test's clipboard seam and returns the shared handle —
    /// the honest in-memory store, never the operator's OS clipboard.
    fn clipboard_seam() -> std::rc::Rc<alacer_core::clipboard::MemoryClipboard> {
        let clipboard = std::rc::Rc::new(alacer_core::clipboard::MemoryClipboard::default());
        alacer_core::clipboard::install(std::rc::Rc::clone(&clipboard)
            as std::rc::Rc<dyn alacer_core::clipboard::ClipboardSeam>);
        clipboard
    }

    /// A seam that refuses every write and reads empty — the
    /// permission/escape story made testable (a clipboard this process
    /// cannot open).
    struct RefusingClipboard;

    impl alacer_core::clipboard::ClipboardSeam for RefusingClipboard {
        fn read_text(&self) -> Option<String> {
            None
        }

        fn write_text(&self, _text: &str) -> bool {
            false
        }
    }

    #[test]
    fn copy_writes_the_selection_and_edits_nothing() {
        let clipboard = clipboard_seam();
        let mut input = stubbed_focused_input("hello world");
        input.anchor_utf16 = 0;
        input.caret_utf16 = 5;

        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl(Key::Char('c'))),
            EventResponse::Handled
        );
        assert_eq!(
            clipboard.text().as_deref(),
            Some("hello"),
            "the copy writes the selection's exact span"
        );
        assert_eq!(input.value(), "hello world", "a copy edits nothing");
        assert_eq!(input.selection().map(|s| s.normalized()), Some((0, 5)));
        let masks = ctx.take();
        assert!(
            masks.contains(alacer_core::Change::PAINT),
            "a successful copy is a consumed chord, got {masks:?}"
        );
    }

    #[test]
    fn cut_moves_the_selection_to_the_seam() {
        let clipboard = clipboard_seam();
        let mut input = stubbed_focused_input("hello world");
        input.anchor_utf16 = 6;
        input.caret_utf16 = 11;

        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl(Key::Char('x'))),
            EventResponse::Handled
        );
        assert_eq!(
            clipboard.text().as_deref(),
            Some("world"),
            "the cut writes the span before deleting it"
        );
        assert_eq!(input.value(), "hello ", "the range went with the cut");
        assert_eq!(input.caret_utf16(), 6, "the caret collapsed at its start");
        let masks = ctx.take();
        assert!(
            masks.contains(alacer_core::Change::A11Y) && masks.contains(alacer_core::Change::PAINT),
            "a cut is an edit: repaint and re-announce, got {masks:?}"
        );
    }

    #[test]
    fn paste_inserts_sanitized_text_and_consumes_the_selection() {
        let clipboard = clipboard_seam();
        let mut input = stubbed_focused_input("hello world");
        input.anchor_utf16 = 0;
        input.caret_utf16 = 5;
        clipboard.set("HEL\r\nLO");

        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl(Key::Char('v'))),
            EventResponse::Handled
        );
        assert_eq!(
            input.value(),
            "HEL LO world",
            "the paste replaced the selection; the CRLF pair is one space"
        );
        assert_eq!(input.caret_utf16(), 6, "the caret parks after the span");
        let masks = ctx.take();
        assert!(
            masks.contains(alacer_core::Change::A11Y) && masks.contains(alacer_core::Change::PAINT),
            "a paste is an edit: repaint and re-announce, got {masks:?}"
        );
    }

    #[test]
    fn a_refused_clipboard_chord_costs_an_idle_frame() {
        // The refuse-don't-munge rule at the clipboard tier: a copy
        // without a selection and a paste from an empty clipboard are
        // consumed chords that queue no masks — the next frame is idle.
        let clipboard = clipboard_seam();
        let mut input = stubbed_focused_input("hello world");
        clipboard.set("the operator's payload");

        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl(Key::Char('c'))),
            EventResponse::Handled,
            "the field consumes its own command chord"
        );
        assert!(ctx.take() == alacer_core::Change::empty(), "no masks");
        assert_eq!(
            clipboard.text().as_deref(),
            Some("the operator's payload"),
            "a refused copy never touches the seam"
        );

        clipboard.clear();
        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl(Key::Char('v'))),
            EventResponse::Handled,
            "the field consumes its own command chord"
        );
        assert!(ctx.take() == alacer_core::Change::empty(), "no masks");
        assert_eq!(input.value(), "hello world", "nothing to paste, no edit");
    }

    #[test]
    fn a_refused_clipboard_write_never_deletes_the_selection() {
        // The cut's destructive half only follows an accepted write: a
        // clipboard that refused it leaves the selection exactly as it
        // was — never munged into a deletion.
        alacer_core::clipboard::install(std::rc::Rc::new(RefusingClipboard));
        let mut input = stubbed_focused_input("hello world");
        input.anchor_utf16 = 0;
        input.caret_utf16 = 5;

        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl(Key::Char('x'))),
            EventResponse::Handled,
            "the field consumes its own command chord"
        );
        assert_eq!(input.value(), "hello world", "the selection survived");
        assert_eq!(input.selection().map(|s| s.normalized()), Some((0, 5)));
        assert!(ctx.take() == alacer_core::Change::empty(), "no masks");
    }

    #[test]
    fn clipboard_chords_are_inert_while_composing() {
        // Chords are keys like any other: the IME owns the canvas until
        // commit/cancel (the M13 rule the word chords already follow).
        let clipboard = clipboard_seam();
        let mut input = stubbed_focused_input("hello");
        input.preedit = Some(("世界".to_string(), Some(2)));
        clipboard.set("payload");

        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl(Key::Char('c'))),
            EventResponse::Ignored,
            "a composing field ignores the clipboard chords"
        );
        assert_eq!(
            clipboard.text().as_deref(),
            Some("payload"),
            "the composing field never wrote the seam"
        );
    }

    fn ctrl_shift(key: Key) -> KeyEvent {
        KeyEvent {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers {
                control: true,
                shift: true,
                ..Modifiers::default()
            },
            text: None,
        }
    }

    #[test]
    fn the_history_chords_undo_and_redo_through_the_funnel() {
        // The M15 chords ride the same dispatch as every edit: typing
        // coalesced one step, undo restores text and caret, redo replays
        // it, and the exhausted redo refuses honestly with no masks.
        let _clipboard = clipboard_seam();
        let mut input = stubbed_focused_input("");
        for ch in "held".chars() {
            type_char(&mut input, ch);
        }
        assert!(input.can_undo(), "the typed fill journaled one step");
        assert!(!input.can_redo());

        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl(Key::Char('z'))),
            EventResponse::Handled
        );
        assert_eq!(input.value(), "", "the typed step undoes");
        assert_eq!(input.caret_utf16(), 0);
        let masks = ctx.take();
        assert!(
            masks.contains(alacer_core::Change::A11Y) && masks.contains(alacer_core::Change::PAINT),
            "an undo is an edit: repaint and re-announce, got {masks:?}"
        );

        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl_shift(Key::Char('z'))),
            EventResponse::Handled
        );
        assert_eq!(input.value(), "held", "the redo replays the step");
        assert_eq!(input.caret_utf16(), 4, "the caret parks at the edit's end");

        // The exhausted redo: consumed, no masks, nothing moved.
        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl_shift(Key::Char('z'))),
            EventResponse::Handled,
            "the field consumes its own command chord"
        );
        assert!(ctx.take() == alacer_core::Change::empty(), "no masks");
        assert_eq!(input.value(), "held");
    }

    #[test]
    fn an_ime_commit_journals_one_undo_step() {
        let _clipboard = clipboard_seam();
        let mut input = stubbed_focused_input("にほ");

        let mut ctx = EventCtx::new();
        let _ = input.on_ime_event(
            &mut ctx,
            &ImeEvent::Commit {
                text: "んご".to_string(),
            },
        );
        assert_eq!(input.value(), "にほんご");
        assert!(input.can_undo());

        let mut ctx = EventCtx::new();
        let _ = input.on_key_event(&mut ctx, &ctrl(Key::Char('z')));
        assert_eq!(input.value(), "にほ", "the whole commit undoes as one step");
        assert_eq!(input.caret_utf16(), 2);
    }

    #[test]
    fn a_whole_value_replacement_resets_the_field_journal() {
        let _clipboard = clipboard_seam();
        let mut input = stubbed_focused_input("");
        for ch in "typed".chars() {
            type_char(&mut input, ch);
        }
        assert!(input.can_undo());

        // The app wrote the value (reconciliation, the AT's SetValue):
        // the journal resets — undo never fights the app's own write.
        input.set_value("replaced by the app");
        assert!(!input.can_undo(), "the reset cleared the journal");
        let mut ctx = EventCtx::new();
        assert_eq!(
            input.on_key_event(&mut ctx, &ctrl(Key::Char('z'))),
            EventResponse::Handled
        );
        assert!(ctx.take() == alacer_core::Change::empty(), "no masks");
        assert_eq!(input.value(), "replaced by the app");
    }

    #[test]
    fn undo_never_slices_a_surrogate_pair() {
        let _clipboard = clipboard_seam();
        let mut input = stubbed_focused_input("");
        type_char(&mut input, '\u{1F600}');
        type_char(&mut input, 'x');
        assert_eq!(input.caret_utf16(), 3);

        // Two typed runes coalesce into one step; the undo restores both
        // and the redo replays them — the pair rides whole throughout.
        let mut ctx = EventCtx::new();
        let _ = input.on_key_event(&mut ctx, &ctrl(Key::Char('z')));
        assert_eq!(input.value(), "");
        let mut ctx = EventCtx::new();
        let _ = input.on_key_event(&mut ctx, &ctrl(Key::Char('y')));
        assert_eq!(input.value(), "\u{1F600}x");
        assert_eq!(input.caret_utf16(), 3);
    }

    #[test]
    fn a_disabled_input_refuses_every_editing_door() {
        // M22, ADR-0028: the keys, the pointer, the IME, and the AT's
        // value tier all refuse honestly — the value the tree carries is
        // the app's, and the refusal queues nothing.
        let mut input = TextInput::new("Email").enabled(false);
        assert!(!input.focusable(), "an unavailable control holds no focus");
        let mut ctx = EventCtx::new();

        assert_eq!(
            input.on_key_event(&mut ctx, &key(Key::Char('x'))),
            EventResponse::Ignored,
            "the keyboard never reaches an unavailable input"
        );
        let down = PointerEvent::Down {
            position: Point::new(20.0, 10.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 1,
        };
        assert_eq!(
            input.on_pointer_event(&mut ctx, &down),
            EventResponse::Handled,
            "the wall consumes the caret gesture"
        );
        assert!(
            ctx.take().is_empty(),
            "a refused gesture queues no masks — an idle frame"
        );
        assert_eq!(
            input.on_ime_event(
                &mut ctx,
                &ImeEvent::Preedit {
                    text: "é".to_string(),
                    cursor_utf16: None,
                }
            ),
            EventResponse::Ignored,
            "an unavailable input answers no composition"
        );
        assert!(
            !input.on_a11y_action(alacer_core::widget::A11yAction::SetValue(
                "typed@at.io".to_string()
            )),
            "the AT's write is refused, never munged"
        );
        assert_eq!(input.value(), "", "the refusals wrote nothing");

        // The write answers the move (the M21 idle-frame property).
        assert!(!input.set_enabled(false), "already unavailable");
        assert!(input.set_enabled(true), "the write moved");
        assert!(input.is_enabled());
    }
}
