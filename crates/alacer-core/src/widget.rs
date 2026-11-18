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
    /// A toggle control (the todo canon's item switch, M6): the binary
    /// pair plus, since M94 (ADR-0098), the tri-state summary — a
    /// checkbox holding `ToggleState::Mixed` is the select-all parent,
    /// "some of the children are"; every user door resolves it.
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
    /// A free-form custom-paint surface (the canvas, M82): the app's paint
    /// closure records the primitives through the paint seam, and the AT
    /// gets the name — a named canvas is a chart, a diagram, a drawing
    /// surface; an unnamed one is an unusable blank (the ADR-5 rule the
    /// named roles carry).
    Canvas,
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
    /// number too. The canvas joins in M82: an unnamed drawing surface
    /// reads as a blank.
    #[must_use]
    pub const fn requires_name(self) -> bool {
        matches!(
            self,
            Self::Button
                | Self::TextInput
                | Self::CheckBox
                | Self::Slider
                | Self::ProgressIndicator
                | Self::Canvas
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
            Self::Canvas => "canvas",
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

/// How far an [`A11yAction::ScrollUp`], [`A11yAction::ScrollDown`],
/// [`A11yAction::ScrollLeft`], or [`A11yAction::ScrollRight`] should
/// travel, normalized across platforms (the AccessKit `ScrollUnit` shape,
/// mirrored in core so the DAG stays adapter-free — ADR-9). Since M91
/// (ADR-0095) the unit travels the horizontal pair too: an item is one
/// line, a page is the viewport — the viewport's *width* on the x axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollUnit {
    /// A single item of a list (one line, the platform's 40 px wheel notch).
    Item,
    /// The amount of content that fits in the viewport.
    Page,
}

impl ScrollUnit {
    /// The unit's name, the wire vocabulary's `snake_case` tag (M52,
    /// ADR-0058; the protocol's stable, documented field values — the
    /// asker's unit the scroll record carries beside the unit-scroll
    /// actions, the same naming rule the action species and the
    /// refusal species ride).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Item => "item",
            Self::Page => "page",
        }
    }
}

/// The single axis a scroll container declares its scrolls to ride
/// (M98, ADR-0102; the AccessKit `Orientation` shape, mirrored in core
/// so the DAG stays adapter-free — ADR-9).
///
/// The vocabulary is deliberately two-variant: it names a *single-axis*
/// declaration, the word ARIA's `aria-orientation` carries. A container
/// that declared both axes (the two-axis list, ADR-0093) has no
/// single-axis word to say — its honest answer is the trait door's
/// `None`, the absence ARIA spells `undefined`. The enum never guesses
/// an axis the constructor did not name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollOrientation {
    /// The declared scrolls ride the y axis — the vertical track.
    Vertical,
    /// The declared scrolls ride the x axis — the horizontal track.
    Horizontal,
}

impl ScrollOrientation {
    /// The orientation's name, the wire vocabulary's `snake_case` tag
    /// (the same naming rule [`ScrollUnit::name`] rides — the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

/// One scroll axis's position and extent (M101, ADR-0105): the offset
/// the clamp landed and the wall it clamps to — AccessKit's
/// `ScrollY`/`ScrollYMax` shape (the x halves beside it), mirrored in
/// core so the DAG stays adapter-free (ADR-9). The floor is the
/// funnel's own zero — the clamp arithmetic every scroll door answers
/// by (`clamp(0.0, span)`), never a guessed minimum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollAxisPosition {
    /// The resolved offset the doors landed (positive = scrolled down
    /// on the vertical axis, right on the horizontal one) — the same
    /// state [`Widget::scroll_offset`] names for compose.
    pub offset: f32,
    /// The wall the clamp lands on: the declared span (content minus
    /// viewport), the largest offset any door can land. A position at
    /// the wall is the clamp's own landing — the tree says where the
    /// container stopped.
    pub max: f32,
}

/// The scroll container's resolved position and extent (M101,
/// ADR-0105): the fact the doors resolve and the AT reads as "scrolled
/// 200 of 0..600" — AccessKit's `ScrollY` / `ScrollYMin` / `ScrollYMax`
/// properties (the `ScrollX` halves beside them on a two-axis
/// container), mapped by `alacer-a11y` in the same snapshot every other
/// property rides.
///
/// Each axis rides its own honesty: an axis whose span is empty (the
/// content fits the viewport, or the width story was never declared)
/// carries no position — the same wall every scroll door answers
/// `no_scrollable_content` with, never a guessed zero. A container with
/// no live axis on either side answers `None` outright.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollPosition {
    /// The vertical axis's fact — the story every scroll container
    /// declares first. `None` when the y span is empty.
    pub vertical: Option<ScrollAxisPosition>,
    /// The horizontal axis's fact (M89, ADR-0093's other axis) —
    /// `None` when the x span is empty or undeclared.
    pub horizontal: Option<ScrollAxisPosition>,
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
    /// Scroll left by `unit` (M91, ADR-0095): the AT's horizontal
    /// vocabulary beside the vertical pair — the same unit contract
    /// (an item is one line, a page is the viewport's width), consumed
    /// by the scroll area's own clamp arithmetic on the declared width
    /// span.
    ScrollLeft(ScrollUnit),
    /// Scroll right by `unit` (M91, ADR-0095): the AT's horizontal
    /// vocabulary beside the vertical pair — the same unit contract
    /// (an item is one line, a page is the viewport's width), consumed
    /// by the scroll area's own clamp arithmetic on the declared width
    /// span.
    ScrollRight(ScrollUnit),
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

impl A11yAction {
    /// The action's name, the wire vocabulary's `snake_case` tag (M57,
    /// ADR-0063): the stable, documented field value the route-drop
    /// trail's records carry beside their species — the dropped
    /// action's own name, never a munge onto the action it did not
    /// become (the platform boundary's mapping uses the same tags for
    /// the actions the two vocabularies share).
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::DefaultPress => "default_press",
            Self::Increment => "increment",
            Self::Decrement => "decrement",
            Self::ScrollUp(_) => "scroll_up",
            Self::ScrollDown(_) => "scroll_down",
            Self::ScrollLeft(_) => "scroll_left",
            Self::ScrollRight(_) => "scroll_right",
            Self::SetValue(_) => "set_value",
            Self::ReplaceSelectedText(_) => "replace_selected_text",
            Self::SetTextSelection(_) => "set_text_selection",
        }
    }
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

/// The bounded scalar's declared span (M97, ADR-0101): the range every
/// value-tier write funnels through, carried neutrally to the a11y sink.
/// `alacer-a11y` maps `min`/`max` onto AccessKit's `MinNumericValue` /
/// `MaxNumericValue` properties — ARIA's `aria-valuemin` /
/// `aria-valuemax` — and `step`, when the widget declares one, onto
/// `NumericValueStep`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueRange {
    /// The range's lower wall: the smallest value every door lands on.
    pub min: f64,
    /// The range's upper wall: the largest value every door lands on.
    pub max: f64,
    /// The amount one step moves (`None` when the widget declares no
    /// step — the honest absence, never a guessed default).
    pub step: Option<f64>,
}

impl ValueRange {
    /// A span with no step: the report half's shape (the progress
    /// declares where its value lives, not how far one act moves it).
    #[must_use]
    pub const fn new(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            step: None,
        }
    }

    /// Attaches the step one act moves (the slider's snap arithmetic).
    #[must_use]
    pub const fn with_step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }
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

/// The bring-into-view ask's two halves (M89, ADR-0093): the
/// content-space target offsets the pipeline priced per axis — the
/// vertical half exactly the M88 ask, the horizontal half beside it
/// priced by the same edge-pin convention from the bounds ledger the
/// door already owns. One ask, both axes: the container's door
/// answers each by its own declared span (the clamp arithmetic, the
/// named walls), never the pipeline guessing at a widget-private
/// range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BringIntoViewAsk {
    /// The vertical half: the content-space target offset the minimal
    /// vertical scroll prices (the M88 ask — above the window scrolls
    /// to the top edge, below it to the bottom edge, a rect taller
    /// than the viewport pins its top edge).
    pub vertical: f32,
    /// The horizontal half beside it (M89, ADR-0093): the content-space
    /// target offset the minimal horizontal scroll prices — left of
    /// the window scrolls to the left edge, right of it to the right
    /// edge, a rect wider than the viewport pins its left edge.
    pub horizontal: f32,
}

/// One axis's answer to a bring-into-view ask (M89, ADR-0093): the
/// clamp's arithmetic carried as data for that axis — the M27 rule at
/// the scroll tier, the shape [`ScrollLanding`] rode before the door
/// grew the axis. `offset` is the landed offset the container holds
/// on the axis (the clamp's result on a delivery, the untouched
/// offset on a refusal); `pinned` names the wall the clamp turned the
/// ask back at (the ADR-0055 rule the ask doors answer by — `None`
/// when the landing is the ask, on every landing mid-range, and on
/// every refusal).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollAxisLanding {
    /// What the container's clamp answered on this axis — delivered,
    /// or the named refusal species ([`crate::inspect::ScrollRefusal`]).
    pub outcome: crate::inspect::ScrollOutcome,
    /// The offset the container holds on this axis as the answer
    /// resolves.
    pub offset: f32,
    /// The wall the clamp pinned a past ask at (`min`/`max`), `None`
    /// when the landing is the ask or nothing moved.
    pub pinned: Option<crate::inspect::RangeBound>,
}

/// The container's answer to a bring-into-view ask (M88, ADR-0092):
/// the landing beside the outcome, the clamp's arithmetic carried as
/// data — the M27 rule at the scroll tier, the same shape the focus
/// walk's [`crate::passes::FocusLanding`] answers by. `offset` is the landed
/// offset the container holds (the clamp's result on a delivery, the
/// untouched offset on a refusal); `pinned` names the wall the clamp
/// turned the ask back at (the ADR-0055 rule the ask doors answer by —
/// `None` when the landing is the ask, on every landing mid-range, and
/// on every refusal). Since M89 (ADR-0093) the fields answer the
/// *vertical* axis and the horizontal axis's own answer rides beside
/// them ([`ScrollAxisLanding`], the same shape, the same rules — the
/// additive shape the scroll record's own halves ride).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollLanding {
    /// What the container's clamp answered — delivered, or the named
    /// refusal species ([`crate::inspect::ScrollRefusal`]).
    pub outcome: crate::inspect::ScrollOutcome,
    /// The offset the container holds as the answer resolves.
    pub offset: f32,
    /// The wall the clamp pinned a past ask at (`min`/`max`), `None`
    /// when the landing is the ask or nothing moved.
    pub pinned: Option<crate::inspect::RangeBound>,
    /// The horizontal axis's own answer (M89, ADR-0093), beside the
    /// vertical — priced by the same door, clamped by the same
    /// arithmetic, refused by its own declared span.
    pub horizontal: ScrollAxisLanding,
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

    /// The container's half of the bring-into-view door (M88, ADR-0092):
    /// the pipeline reads the bounds ledger it already owns, computes the
    /// minimal offset that lands the aimed node fully inside this
    /// container's viewport, and hands the ask here — the widget answers
    /// by its own clamp arithmetic (the span walls, the named refusals),
    /// the range living where the widget keeps it, never the pipeline
    /// guessing at a private span. Since M89 (ADR-0093) the ask carries
    /// both halves ([`BringIntoViewAsk`]): the vertical half exactly the
    /// M88 ask, the horizontal half beside it priced by the same
    /// edge-pin convention — and the container answers each axis by its
    /// own declared span, the landing reporting both ([`ScrollLanding`]'s
    /// vertical fields beside [`ScrollAxisLanding`]). The landing is data
    /// (the M27 rule at the scroll tier); the door notes the ask on the
    /// scroll log under [`crate::inspect::ScrollAction::BringIntoView`],
    /// the route stamps filling the container and the aimed node the
    /// widget cannot know (the M43 rule at the scroll tier). `None` when
    /// this widget claims the scroll slot ([`Self::is_scroll_container`])
    /// without shipping the door — the pipeline answers
    /// [`crate::passes::ScrollIntoViewRefusal::NoScrollDoor`] on that
    /// answer, the contract gap named where the consumer can read it.
    fn bring_into_view(&mut self, ask: BringIntoViewAsk) -> Option<ScrollLanding> {
        let _ = ask;
        None
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

    /// The widget's description for the a11y tree (M95, ADR-0099): the
    /// help text a screen reader reads when the name alone does not
    /// answer "what does this control do?" — AccessKit's `Description`
    /// property, ARIA's `aria-description`, mapped by `alacer-a11y` in
    /// the same snapshot every other property rides. The name is the
    /// control's identity; the description is its *instruction* — a
    /// format ("the 16 digits on the front"), a consequence ("saves the
    /// draft"), the sentence the label is too short to carry.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// description tier): a widget with nothing to explain announces
    /// nothing, never a guess — the tree M94 handed over. Actionable
    /// controls override this; containers and reports inherit the
    /// default.
    fn description(&self) -> Option<&str> {
        None
    }

    /// The widget's placeholder for the a11y tree (M96, ADR-0100): the
    /// hint text an empty editable announces before the user types —
    /// AccessKit's `Placeholder` property, ARIA's `aria-placeholder`,
    /// mapped by `alacer-a11y` in the same snapshot every other property
    /// rides. Through M95 the field painted the hint and the tree could
    /// not say it: the declared-but-unreported gap, the M94 pattern at
    /// the placeholder tier.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// placeholder tier): a field with no hint announces nothing, never
    /// a guess — an empty string is the absence, not an empty hint.
    fn placeholder(&self) -> Option<&str> {
        None
    }

    /// The widget's declared value range for the a11y tree (M97,
    /// ADR-0101): the span every value-tier write funnels through — the
    /// clamp-and-snap arithmetic the drag, the arrows, and the AT's own
    /// acts all ride — carried as AccessKit's `MinNumericValue` /
    /// `MaxNumericValue` properties beside the `NumericValueStep` the
    /// slider declares (ARIA's `aria-valuemin` / `aria-valuemax` / the
    /// step). Through M96 the tree said the value ("42") and not the
    /// span — whether that was the middle of 0..100 or the top of
    /// 0..50 — the declared-but-unreported gap, the M94 pattern at the
    /// range tier.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// range tier): a widget with no span announces nothing, never a
    /// guessed default. The value tier's controls override this; every
    /// other role inherits the default.
    fn value_range(&self) -> Option<ValueRange> {
        None
    }

    /// The widget's numeric now for the a11y tree (M99, ADR-0103): the
    /// number behind the value's spoken word — the scalar the value
    /// tier's funnel holds, carried as AccessKit's `NumericValue`
    /// property, ARIA's `aria-valuenow`, riding the text the tree
    /// already carried as the value's `aria-valuetext`. Through M98 the
    /// tree said the value as text ("40") and not the number (40): the
    /// automation and voice-control tier that reads the range widgets
    /// as numbers read prose — the gap ADR-0101's re-open rule
    /// pre-registered verbatim, cashed at the numeric tier.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// numeric tier): a widget whose value is prose announces no number,
    /// never a parsed guess — a text field's "16 digits" is not 16, and
    /// a valueless role announces nothing. The value tier's controls
    /// override this; every other role inherits the default.
    fn numeric_value(&self) -> Option<f64> {
        None
    }

    /// Whether the widget is a field of secret entry (M100, ADR-0104):
    /// the flag behind the constructor word [`ImePurpose::Secure`] the
    /// IME contract has carried since M4 — the field the platform's IME
    /// already treats as a password, suppresses echo and suggestions
    /// for. The tree says the word: AccessKit's `PasswordInput` role,
    /// the declaration every platform AT announces ("secure text
    /// entry"), and the value the tree resolves for the field is the
    /// mask pass 9 builds from it — one masking character per code
    /// point, never the secret itself (AccessKit's own contract: a
    /// `PasswordInput` node's value "must only contain masking
    /// characters").
    ///
    /// The default is the honest absence: a widget that declared
    /// nothing is no secure field, and a non-input role answers nothing.
    /// Editable controls override this; every other role inherits the
    /// default.
    fn secure_entry(&self) -> bool {
        false
    }

    /// Whether a declared-secure field's value rides revealed (M102,
    /// ADR-0106) — the word behind the "show password" toggle, the
    /// flag beside the constructor word [`Self::secure_entry`]
    /// answers. AccessKit's own contract names the mapping: "when the
    /// password is revealed, use `Role::TextInput` instead" — the
    /// sink's role branch reads the ordinary editable word, and the
    /// value the tree resolves is the field's ordinary value again
    /// (the mask yielding to the reveal; the pixels and the tree
    /// telling one truth).
    ///
    /// The word is a compound: meaningful only beside a declared
    /// secret entry — a field that answered `secure_entry()` `false`
    /// answers `false` here too, whatever its internal toggle holds
    /// (the plain field's value was always the tree's read, never a
    /// secret to reveal). Editable controls override this; every
    /// other role inherits the default.
    fn secure_revealed(&self) -> bool {
        false
    }

    /// The single axis a scroll container declares its scrolls to ride
    /// (M98, ADR-0102) — the word the AT's arrows read before they
    /// ride: ARIA's `aria-orientation`, AccessKit's `Orientation`
    /// property. Since M89 (ADR-0093) a scroll container declares its
    /// axes at construction and every scroll door answers per axis;
    /// since M98 the tree says which.
    ///
    /// `None` is the honest absence (the ADR-0087 presence rule at the
    /// orientation tier): a container that declared both axes has no
    /// single-axis word to say — ARIA's `undefined` is the spec's own
    /// word for "both axes", and the sink sets nothing rather than a
    /// wrong half. A widget that is no scroll container at all inherits
    /// the default. Scroll containers override this; every other role
    /// inherits it too.
    fn scroll_orientation(&self) -> Option<ScrollOrientation> {
        None
    }

    /// The scroll container's resolved position and extent (M101,
    /// ADR-0105): where the doors landed and the walls they clamp to —
    /// AccessKit's `ScrollY` / `ScrollYMin` / `ScrollYMax` properties
    /// (the `ScrollX` halves beside them), ARIA's scroll position
    /// arithmetic the platform ATs announce as "scrolled 200 of
    /// 0..600". Through M100 the tree said the axis word (M98,
    /// ADR-0102) and never where the container *was*: the offsets and
    /// spans the scroll doors resolve (M50–M54, ADR-0056; M89,
    /// ADR-0093) rode the actions and the trail, the
    /// declared-but-unreported gap, the M94 pattern at the scroll
    /// position tier.
    ///
    /// Each axis rides its own honesty (the ADR-0087 presence rule at
    /// the scroll position tier): an axis whose span is empty — the
    /// content fits the viewport, or the width story was never
    /// declared — carries no position, the same wall every scroll door
    /// answers `no_scrollable_content` with, never a guessed zero. A
    /// container with no live axis on either side answers `None`;
    /// a widget that is no scroll container at all inherits the
    /// default. Scroll containers override this; every other role
    /// inherits it too.
    ///
    /// The tree's currency rule is the wheel's own: the doors that pay
    /// the a11y currency (the wheel, the AT's asks, bring-into-view)
    /// merge `PAINT | A11Y` on the container, so the position
    /// re-announces the frame the offset moved; the app's own handle
    /// rides the next paid pass.
    fn scroll_position(&self) -> Option<ScrollPosition> {
        None
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
