//! The pointer cursor seam: the hover shape a widget declares, named in
//! core and applied by the platform without any widget depending on the
//! window system (§10.1, ADR-0156).
//!
//! A link reads as "opens something", a text field as "type here" — and
//! the pointer is the vocabulary that makes the difference visible before
//! the first click. Through M151 the workspace named mouse positions
//! ([`crate::events::PointerEvent`]), IME carets, and an inspector's
//! replay cursor, but the window's cursor *shape* had no door: a widget
//! author could not ask for the hand over a link or the I-beam over a
//! field, and the platform never touched the window's cursor. The seam
//! gives the widget tier one declared word and the platform one applied
//! fact, the layering the clipboard seam imposes (ADR-0020) held exactly:
//! core carries the vocabulary, the platform owns the window call.
//!
//! ```text
//! // Widget tier (core-only dependencies) — declare the hover shape:
//! impl Widget for Link {
//!     fn cursor_style(&self) -> CursorStyle { CursorStyle::Pointer }
//! }
//!
//! // Pipeline tier — resolve the hover (the deepest accepting hit):
//! let style = pipeline.cursor_at(pointer_position);
//!
//! // Platform tier — apply the delta to the window, only when it moved:
//! window.set_cursor(cursor_icon_of(style));
//! ```
//!
//! The vocabulary is the honest CSS-parity subset of the window system's
//! own icon set (winit's `CursorIcon`, the names the web's `cursor`
//! property taught): every style maps one-to-one, none is invented. The
//! single-edge resizes (`e_resize`, `n_resize`, …) are deliberately
//! absent — they are the window chrome's own words (winit maps them from
//! `ResizeDirection`), never a widget's declaration; the two-axis and
//! column/row resizes ride beside them.
//!
//! The application is the platform's delta discipline (the IME request's
//! own rule, §10.3): resolve the hover per frame, call the window only
//! when the shape moved — platform calls are not free, and a still
//! pointer over a still tree owes the OS nothing.

/// The hover shape a widget declares for the pointer over its face.
///
/// The default is the platform's arrow: a widget that overrides nothing
/// declares nothing, and the honest answer is the absence — never a
/// guessed affordance (the ADR-5 discipline, at the cursor tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorStyle {
    /// The platform's arrow — the default every widget inherits.
    Default,
    /// The open affordance: the hand over a hyperlink or a web-style
    /// target.
    Pointer,
    /// The text caret: the I-beam over selectable or editable text.
    Text,
    /// The precision crosshair (a canvas awaiting a pick).
    Crosshair,
    /// The open hand: this face is grabbable.
    Grab,
    /// The closed hand: a grab is in flight.
    Grabbing,
    /// The forbidden sign: this ask cannot land here.
    NotAllowed,
    /// The busy watch: the app is working and input will wait.
    Wait,
    /// The busy-with-progress arrow: work runs behind the pointer.
    Progress,
    /// The question arrow: help rides this face.
    Help,
    /// The menu arrow: a context menu lives here.
    ContextMenu,
    /// The four-way move cross.
    Move,
    /// The copy arrow with a plus: a drag here duplicates.
    Copy,
    /// The shortcut arrow: this face aliases the drag's meaning.
    Alias,
    /// The no-entry sign: the drag's payload cannot drop here.
    NoDrop,
    /// The spreadsheet cell cross.
    Cell,
    /// The vertical text I-beam.
    VerticalText,
    /// The omnidirectional scroll cross.
    AllScroll,
    /// The magnifier with a plus: zoom in.
    ZoomIn,
    /// The magnifier with a minus: zoom out.
    ZoomOut,
    /// The horizontal double arrow: resize the east/west span.
    EwResize,
    /// The vertical double arrow: resize the north/south span.
    NsResize,
    /// The descending diagonal double arrow (north-east ↔ south-west).
    NeswResize,
    /// The ascending diagonal double arrow (north-west ↔ south-east).
    NwseResize,
    /// The column splitter: resize a vertical divider.
    ColResize,
    /// The row splitter: resize a horizontal divider.
    RowResize,
}

impl CursorStyle {
    /// The style's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values — the CSS `cursor`
    /// words, underscored).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Pointer => "pointer",
            Self::Text => "text",
            Self::Crosshair => "crosshair",
            Self::Grab => "grab",
            Self::Grabbing => "grabbing",
            Self::NotAllowed => "not_allowed",
            Self::Wait => "wait",
            Self::Progress => "progress",
            Self::Help => "help",
            Self::ContextMenu => "context_menu",
            Self::Move => "move",
            Self::Copy => "copy",
            Self::Alias => "alias",
            Self::NoDrop => "no_drop",
            Self::Cell => "cell",
            Self::VerticalText => "vertical_text",
            Self::AllScroll => "all_scroll",
            Self::ZoomIn => "zoom_in",
            Self::ZoomOut => "zoom_out",
            Self::EwResize => "ew_resize",
            Self::NsResize => "ns_resize",
            Self::NeswResize => "nesw_resize",
            Self::NwseResize => "nwse_resize",
            Self::ColResize => "col_resize",
            Self::RowResize => "row_resize",
        }
    }
}

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

/// The cursor trail's cap: facts carried per thread before the oldest
/// evicts — the journal's own bounded rule (§14.3) applied to its echo
/// (a devtools log, not a frame-budget liability; the same order the
/// clipboard trail rides, ADR-0045).
pub const CURSOR_TRAIL_CAP: usize = 256;

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs ([`crate::passes`]
    /// `PassPipeline::set_observer`), off otherwise — the disabled
    /// trail costs one branch per resolution (the A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The last noted answer: the flood rule's coalescing state — a
    /// style that did not move is not a fact, so a hover resting on one
    /// face resolves at one branch (the compare) and queues nothing.
    static LAST_NOTED: Cell<Option<CursorStyle>> = const { Cell::new(None) };
    /// The facts queued since the last tick drain — the handler side of
    /// the seam, bounded by [`CURSOR_TRAIL_CAP`] with oldest-first
    /// eviction, so a change queued where no pipeline will ever drain
    /// still terminates.
    static TRAIL: RefCell<VecDeque<crate::inspect::CursorRecord>> =
        const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the cursor trail (the pipeline's side of the
/// seam: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that — and never resets the coalescing state: the first
/// re-enabled change is the change the facts resume at.
pub(crate) fn set_cursor_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// The resolution door's note (M153, ADR-0157) — the caller is
/// [`crate::passes::PassPipeline::cursor_at`], where the hover resolves:
/// a no-op while the trail is disabled (one branch per resolution — the
/// A7 idle contract), a coalesced push while it rides. One record per
/// resolved *change*, never per frame (the flood rule at the cursor
/// tier): a style that did not move is not a fact, the compare against
/// the last noted answer the whole cost of a still hover.
pub(crate) fn note_resolution(style: CursorStyle, source: crate::inspect::CursorSource) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        if LAST_NOTED.with(Cell::get) == Some(style) {
            return;
        }
        LAST_NOTED.with(|slot| slot.set(Some(style)));
        tracing::trace!(
            target: "alacer::cursor",
            style = style.name(),
            source = source.name(),
            "cursor fact queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= CURSOR_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(crate::inspect::CursorRecord { style, source });
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::Cursor`].
pub(crate) fn take_cursor_trail() -> Vec<crate::inspect::CursorRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::CursorStyle;

    /// The styles' wire names: the protocol's stable, documented
    /// `snake_case` tags — the CSS `cursor` words, underscored.
    #[test]
    fn the_styles_carry_their_wire_names() {
        let styles = [
            (CursorStyle::Default, "default"),
            (CursorStyle::Pointer, "pointer"),
            (CursorStyle::Text, "text"),
            (CursorStyle::Crosshair, "crosshair"),
            (CursorStyle::Grab, "grab"),
            (CursorStyle::Grabbing, "grabbing"),
            (CursorStyle::NotAllowed, "not_allowed"),
            (CursorStyle::Wait, "wait"),
            (CursorStyle::Progress, "progress"),
            (CursorStyle::Help, "help"),
            (CursorStyle::ContextMenu, "context_menu"),
            (CursorStyle::Move, "move"),
            (CursorStyle::Copy, "copy"),
            (CursorStyle::Alias, "alias"),
            (CursorStyle::NoDrop, "no_drop"),
            (CursorStyle::Cell, "cell"),
            (CursorStyle::VerticalText, "vertical_text"),
            (CursorStyle::AllScroll, "all_scroll"),
            (CursorStyle::ZoomIn, "zoom_in"),
            (CursorStyle::ZoomOut, "zoom_out"),
            (CursorStyle::EwResize, "ew_resize"),
            (CursorStyle::NsResize, "ns_resize"),
            (CursorStyle::NeswResize, "nesw_resize"),
            (CursorStyle::NwseResize, "nwse_resize"),
            (CursorStyle::ColResize, "col_resize"),
            (CursorStyle::RowResize, "row_resize"),
        ];
        for (style, name) in styles {
            assert_eq!(style.name(), name, "{name} is the wire tag");
        }
    }
}
