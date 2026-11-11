//! The `Widget` trait: accessibility is a type-level contract (ADR-5).
//!
//! `role` and `label` are **required** trait methods — a widget that cannot be
//! named cannot be compiled (Masonry's required-method design). The debug-mode
//! CI walker in `alacer-testing` fails builds on unnamed input controls; the
//! contract holds by construction, not by discipline.

use crate::events::{EventCtx, ImeEvent, KeyEvent, PointerEvent};
use crate::scene::PaintCtx;
use crate::style::Style;
use std::borrow::Cow;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Semantic role of a widget, mapped onto AccessKit roles by `alacer-a11y`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// No meaningful semantic; used for pure decorations.
    Decoration,
    /// A generic grouping container.
    Container,
    /// Static text.
    Label,
    /// An actionable button.
    Button,
    /// A single-line text input.
    TextInput,
    /// A scrollable region.
    ScrollArea,
    /// One row of a list.
    ListItem,
    /// A collection of list items.
    List,
    /// A binary toggle control (the todo canon's item switch, M6).
    CheckBox,
    /// A value control (the slider, M20): a bounded scalar the user sets
    /// by drag or arrow keys and the AT sets through the value tier
    /// (ADR-0026). The value itself rides the M3 `value()` contract.
    Slider,
    /// A read-only value report (the progress indicator, M21): a bounded
    /// scalar the *app* writes and the AT reads through the M3 `value()`
    /// contract — the value tier's read half (ADR-0027). The write
    /// vocabulary is the app's; every AT action is refused honestly.
    ProgressIndicator,
    /// A data table: headers, rows, cells (the data-table canon, M6).
    Table,
    /// One row of a data table.
    TableRow,
    /// One cell of a data table row.
    TableCell,
    /// A table's column header.
    ColumnHeader,
    /// The app's reserved announcement region (§11.6, ADR-0024): a live
    /// region the pipeline owns — its content is the last announcement,
    /// and assistive technologies speak its changes. Never a widget's
    /// role; the pipeline builds the node itself.
    LiveRegion,
}

impl Role {
    /// Roles whose accessible name is mandatory; the CI walker fails on
    /// unnamed widgets of these roles (§11.1). The checkbox joins the set in
    /// M6: an unnamed toggle is unusable with a screen reader. The slider
    /// joins in M20: an unnamed value control reads as a bare number. The
    /// progress indicator joins in M21: an unnamed report reads as a bare
    /// number too.
    #[must_use]
    pub const fn requires_name(self) -> bool {
        matches!(
            self,
            Self::Button
                | Self::TextInput
                | Self::CheckBox
                | Self::Slider
                | Self::ProgressIndicator
        )
    }

    /// Human-readable role name, used by the CI walker's diagnostics.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Decoration => "decoration",
            Self::Container => "container",
            Self::Label => "label",
            Self::Button => "button",
            Self::TextInput => "text_input",
            Self::ScrollArea => "scroll_area",
            Self::ListItem => "list_item",
            Self::List => "list",
            Self::CheckBox => "checkbox",
            Self::Slider => "slider",
            Self::ProgressIndicator => "progress_indicator",
            Self::Table => "table",
            Self::TableRow => "table_row",
            Self::TableCell => "table_cell",
            Self::ColumnHeader => "column_header",
            Self::LiveRegion => "live_region",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Where a widget's accessible name comes from.
#[derive(Clone)]
pub enum LabelSource {
    /// A fixed string owned by the widget.
    Text(Cow<'static, str>),
    /// A live string shared with reactive state (e.g. a bound label).
    Computed(Arc<Mutex<String>>),
    /// A render binding — the same slot the widget paints from.
    Binding(crate::flux::RenderBinding<String>),
    /// The name is the concatenation of descendant names (containers).
    FromChildren,
}

impl fmt::Debug for LabelSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(t) => f.debug_tuple("Text").field(&t).finish(),
            Self::Computed(s) => {
                if let Ok(guard) = s.lock() {
                    f.debug_tuple("Computed").field(&guard.as_str()).finish()
                } else {
                    f.debug_tuple("Computed").field(&"<poisoned>").finish()
                }
            }
            Self::Binding(b) => f
                .debug_tuple("Binding")
                .field(&b.with(String::len))
                .finish(),
            Self::FromChildren => f.write_str("FromChildren"),
        }
    }
}

impl LabelSource {
    /// Resolves to the current accessible name. Never panics: a poisoned
    /// name cell resolves to the empty string — a name is a label, never
    /// a frame-buster.
    #[must_use]
    pub fn resolve(&self) -> String {
        match self {
            Self::Text(t) => t.to_string(),
            Self::Computed(cell) => cell.lock().map(|guard| guard.clone()).unwrap_or_default(),
            Self::Binding(binding) => binding.with(Clone::clone),
            Self::FromChildren => String::new(),
        }
    }

    /// A fixed-text source from any string-like value.
    #[must_use]
    pub fn text(t: impl Into<Cow<'static, str>>) -> Self {
        Self::Text(t.into())
    }
}

/// How far an [`A11yAction::ScrollUp`] or [`A11yAction::ScrollDown`] should
/// travel, normalized across platforms (the AccessKit `ScrollUnit` shape,
/// mirrored in core so the DAG stays adapter-free — ADR-9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollUnit {
    /// A single item of a list (one line, the platform's 40 px wheel notch).
    Item,
    /// The amount of content that fits in the viewport.
    Page,
}

/// A range of text, expressed in the contract's unit — **UTF-16 code
/// units** into the widget's value (§10.3; the same unit as the caret).
///
/// `anchor` is where the selection started and does not move as the
/// selection grows; `focus` is the active end (the caret while extending).
/// `anchor == focus` is the degenerate selection — a bare caret. Every
/// conversion to Rust's UTF-8 strings funnels through
/// [`crate::utf16`], so surrogate pairs never slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSelection {
    /// Where the selection began, as a UTF-16 offset into the value.
    pub anchor: usize,
    /// The active end (the caret while extending), as a UTF-16 offset.
    pub focus: usize,
}

impl TextSelection {
    /// A selection from `anchor` to `focus` (both UTF-16 offsets).
    #[must_use]
    pub const fn new(anchor: usize, focus: usize) -> Self {
        Self { anchor, focus }
    }

    /// The selection's edges in ascending order: `(start, end)`.
    #[must_use]
    pub const fn normalized(&self) -> (usize, usize) {
        if self.anchor <= self.focus {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }
}

/// An accessibility action requested by an assistive technology.
///
/// The vocabulary is the routing contract (ADR-0016): the platform maps each
/// AccessKit action onto exactly one variant — an action a widget does not
/// implement is *refused* by that widget, never munged into a different
/// action. Growing the enum is how the AT tier widens; every variant gains
/// a consuming widget in the same milestone. M11 (ADR-0017) adds the value
/// tier: `SetValue`, `ReplaceSelectedText`, and `SetTextSelection` carry
/// their payload, so the enum is `Clone` but no longer `Copy` — the price
/// of a faithful mapping, and the reason refusal (no allocation, no move)
/// stays observable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum A11yAction {
    /// The default action — "press" for buttons, "toggle" for checkboxes.
    DefaultPress,
    /// Increment a value by one step (a scroll area steps by one page).
    Increment,
    /// Decrement a value by one step (a scroll area steps by one page).
    Decrement,
    /// Scroll up by `unit` (the AT's scroll command on a scroll area).
    ScrollUp(ScrollUnit),
    /// Scroll down by `unit` (the AT's scroll command on a scroll area).
    ScrollDown(ScrollUnit),
    /// Replace the widget's whole value and reset the selection (M11,
    /// ADR-0017) — the AccessKit `SetValue` mapping, faithful to the AT's
    /// request.
    SetValue(String),
    /// Delete any selected text and insert `value` in its place (M11,
    /// ADR-0017) — the AT typing on the user's behalf, like a paste.
    ReplaceSelectedText(String),
    /// Move the selection directly to `anchor`/`focus` (M11, ADR-0017) —
    /// the AT repositioning the caret or the range, UTF-16 units.
    SetTextSelection(TextSelection),
}

/// A toggle control's checked state, carried neutrally to the a11y sink
/// (M6): `alacer-a11y` maps it onto AccessKit's `Toggled` property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToggleState {
    /// Unchecked.
    Off,
    /// Checked.
    On,
    /// Partially checked (tri-state parents).
    Mixed,
}

impl From<bool> for ToggleState {
    fn from(checked: bool) -> Self {
        if checked { Self::On } else { Self::Off }
    }
}

/// A node's position inside a data table, carried neutrally to the a11y
/// sink (M6). `alacer-a11y` maps the variants onto AccessKit's table
/// attributes — a grid a screen reader can navigate needs positions, not
/// just roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableSlot {
    /// The table itself: its live geometry (rows × columns).
    Table {
        /// Number of data rows currently mounted.
        rows: usize,
        /// Number of columns (header count).
        columns: usize,
    },
    /// A column header for zero-based `column`.
    Header {
        /// Zero-based column index.
        column: usize,
    },
    /// A data cell at zero-based `(row, column)`.
    Cell {
        /// Zero-based row index.
        row: usize,
        /// Zero-based column index.
        column: usize,
    },
}

/// What happened to a routed event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventResponse {
    /// The widget did not consume the event; it bubbles on.
    #[default]
    Ignored,
    /// The widget consumed the event; bubbling stops.
    Handled,
}

/// The retained widget contract.
///
/// Widgets are never re-created on state change (§6.1): they are created once,
/// mounted into the [`crate::arena::Arena`], and mutated in place. Reactive
/// updates land through `alacer-core::flux::RenderBinding`s, not through this
/// trait.
pub trait Widget: 'static {
    /// The widget's semantic role. Required: the a11y contract starts here.
    fn role(&self) -> Role;

    /// The widget's accessible name source. Required: see ADR-5.
    fn label(&self) -> LabelSource;

    /// Layout styles. Paint properties stay on the widget (§7.2 pass 5).
    fn style(&self) -> Style {
        Style::default()
    }

    /// Emits this widget's own primitives (background, text). Children are
    /// painted by the pass system walking the tree; containers override only
    /// for decorations.
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let _ = ctx;
    }

    /// Scroll offset this widget applies to its children during compose
    /// (translation-only updates; §8.2). Non-scrolling widgets return zero.
    fn scroll_offset(&self) -> crate::geom::Vector {
        crate::geom::Vector::zero()
    }

    /// Whether this widget's children translate with [`Self::scroll_offset`].
    ///
    /// The arena indexes scroll containers so the compose pass visits only
    /// real candidates instead of walking every node each frame.
    fn is_scroll_container(&self) -> bool {
        false
    }

    /// The widget's intrinsic measure request, consumed by the layout engine's
    /// measure functions.
    fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
        None
    }

    /// Handles a routed pointer event targeting this widget.
    ///
    /// Mutate your own state here and request masks through `ctx` — never
    /// touch other widgets (the pass system owns the tree).
    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Ignored
    }

    /// Handles a key event routed to this widget because it holds focus
    /// (§10.3 — keyboard input is focus-routed, never hit-tested). Editing
    /// widgets act on [`crate::events::KeyState::Pressed`] only; returning [`EventResponse::
    /// Ignored`] lets the framework take the key (Tab's focus cycling).
    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Ignored
    }

    /// Handles an IME event routed to this widget because it holds focus
    /// (§10.3, the IME contract's event half). Preedit is *composing* input:
    /// display it inline (the platform cursor is a UTF-16 offset within the
    /// preedit); an empty preedit cancels; [`ImeEvent::Commit`] inserts final
    /// text at the caret and clears any preedit.
    fn on_ime_event(&mut self, ctx: &mut EventCtx, event: &ImeEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Ignored
    }

    /// Notifies the widget that keyboard focus moved to it (`true`) or away
    /// (`false`) — the focus-ring hook (§7.2 pass 3). Widgets store the flag
    /// and repaint on their next look; the framework already merges the
    /// PAINT mask on both endpoints of the move.
    fn set_focused(&mut self, focused: bool) {
        let _ = focused;
    }

    /// The IME flavor this widget wants while focused (§10.3). [`ImePurpose::
    /// Off`] (the default) tells the platform no IME is needed; the pipeline
    /// derives the frame's [`crate::passes::ImeRequest`] from the focused
    /// widget's answer.
    fn ime_purpose(&self) -> crate::passes::ImePurpose {
        crate::passes::ImePurpose::Off
    }

    /// The widget's exposed value for the a11y tree (M3 contract): editable
    /// widgets report their content so screen readers read the text, not just
    /// the name. Valueless roles return `None`.
    fn value(&self) -> Option<String> {
        None
    }

    /// Whether the control accepts interaction (M22, ADR-0028): the
    /// availability tier's single source of truth. An unavailable control
    /// reports itself to the a11y tree (the sink maps the flag onto
    /// AccessKit's `disabled` property), refuses every door honestly —
    /// the pointer is a wall (the gesture never clicks through), the
    /// keyboard never reaches it, and every AT action is refused, never
    /// munged (the M11 rule) — holds no focus (the default [`Self::
    /// focusable`] derivation consults this), and is never a Tab stop.
    /// Reporting widgets and decorations are always available: only
    /// actionable controls override this.
    fn enabled(&self) -> bool {
        true
    }

    /// The widget's toggle state for the a11y tree (M6): checkboxes report
    /// their checked state so screen readers announce "checked"/"not
    /// checked". Non-toggle roles return `None`.
    fn toggled(&self) -> Option<ToggleState> {
        None
    }

    /// The widget's position inside a data table (M6): the table, its
    /// headers, and its cells report their geometry so the AccessKit update
    /// carries navigable grid attributes. Non-table roles return `None`.
    fn table_slot(&self) -> Option<TableSlot> {
        None
    }

    /// The widget's text selection for the a11y tree (M11, ADR-0017):
    /// editable widgets report the focused selection — anchor and caret, in
    /// UTF-16 units — so a screen reader reads the range, not just the
    /// value. `None` when the widget exposes none (not focused, composing,
    /// or a valueless role); a focused caret reports as the degenerate
    /// selection.
    fn selection(&self) -> Option<TextSelection> {
        None
    }

    /// Handles an accessibility action targeting this widget. Returns `true`
    /// when consumed. Making widgets *actionable* completes the contract in
    /// §11.5: reachable, named, and actionable.
    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        let _ = action;
        false
    }

    /// Whether this widget accepts direct pointer hits (scroll containers do;
    /// decorations do not).
    fn accepts_pointer(&self) -> bool {
        true
    }

    /// Whether this widget can hold keyboard focus (§7.2 pass 2 recomputes
    /// the candidates from this predicate after structure changes).
    ///
    /// The default derives from the semantic role — actionable inputs are
    /// focusable — *and* from the availability tier (M22, ADR-0028): an
    /// unavailable control is never a focus candidate, so a widget that
    /// overrides [`Self::enabled`] needs no `focusable` override of its
    /// own. Override for custom controls.
    fn focusable(&self) -> bool {
        matches!(
            self.role(),
            Role::Button | Role::TextInput | Role::CheckBox | Role::Slider
        ) && self.enabled()
    }

    /// The `Any` escape hatch for in-place patching (§6.1: view diffs mutate
    /// retained widgets in place; concrete patchers downcast through this,
    /// the same discipline Masonry's `WidgetMut` machinery uses). Implement
    /// as `{ self }`.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
