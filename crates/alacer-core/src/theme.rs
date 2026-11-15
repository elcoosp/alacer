//! The runtime theme seam (§7.2 pass 8, ADR-0079): the palette every
//! widget borrows for its un-overridden paint roles becomes an
//! installed implementation the paint pass resolves — the way the
//! clipboard is a seam ([`crate::clipboard::ClipboardSeam`], ADR-0020)
//! and text measurement is a seam ([`crate::style::MeasureFn`], ADR-9).
//! The palette was ten compile-time constants on
//! `alacer_widgets::theme` — "make it dark" meant rebuilding every
//! widget, and no door answered the ask at all.
//!
//! The seam's shape is the codebase's own: the app installs a
//! [`ThemeSeam`] through [`crate::passes::PassPipeline::set_theme`],
//! the door answers `false` on a swap that lands where the palette
//! already is (the idle-frame property, A1's rule at the theme tier —
//! no broadcast, no record), and lands the swap otherwise: the roles
//! resolve once per paint pass into [`crate::scene::PaintCtx`], the
//! widgets paint the roles their app did not override, and the
//! broadcast rides the one invalidation currency — `Change::PAINT` on
//! every node, the mask's own documented semantics ("a color change
//! repaints this node only"), never a private repaint channel.
//!
//! The swap answers on the causal log: one
//! [`crate::inspect::ThemeRecord`] per swap that landed — the
//! palette's tag, never its colors (the payload rule's shape: the log
//! names the palette, `PaintCtx` carries it) — and since M78 (ADR-0084,
//! ADR-0082's second pre-registered re-open rule reaching the theme
//! tier) the broadcast's node count rides beside the tag (the lengths
//! rule: the count names the swap's reach, the ids never riding, the
//! composed swap naming the same reach the flip's broadcast resolved —
//! one flip, one count) — the route-drop trail's
//! shape (ADR-0063), bounded, drained at the tick's start, silent
//! while nobody listens (the A7 idle contract).
//!
//! ```text
//! // The door (core-only dependency):
//! pipeline.set_theme(Box::new(MyPalette));
//! // The paint half (widgets, per node):
//! let fill = self.fill.unwrap_or(ctx.palette().accent);
//! ```

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;

use crate::color::Color;
use crate::inspect::ThemeRecord;

/// The theme trail's cap: facts carried per thread before the oldest
/// evicts — the journal's own bounded rule (§14.3) applied to the theme
/// tier (a devtools log, not a frame-budget liability; the same order
/// the edit, announcement, and route-drop trails ride).
pub const THEME_TRAIL_CAP: usize = 256;

/// The paint roles one palette owns — the vocabulary the widgets
/// borrow, the canon palette pins, and a retheme re-resolves. Plain
/// data (`Copy`): the paint pass resolves the palette once per pass
/// and hands copies through [`crate::scene::PaintCtx`]; a swap that
/// lands where this value already is queues nothing (the door's
/// `PartialEq` idle rule).
///
/// Each role names the look it paints, not the widget that reads it —
/// a role is borrowed by several widgets (the border hairline on the
/// table, the track, the unavailable rail), and the canon values pin
/// the constants the blessed snapshot corpus was blessed with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Window background (the `panel` element's default fill).
    pub surface: Color,
    /// Raised container background; an input's own background.
    pub panel: Color,
    /// The primary action's fill; a control's active accent.
    pub accent: Color,
    /// The accent while the control is pressed.
    pub accent_pressed: Color,
    /// Text-selection highlight behind the selected range (M11,
    /// ADR-0017).
    pub selection: Color,
    /// Body text.
    pub text: Color,
    /// Muted text; a control's unavailable label.
    pub text_muted: Color,
    /// Hairline separators; an unfocused rail; the unavailable fill.
    pub border: Color,
    /// Text sitting on the accent (the button's label default).
    pub on_accent: Color,
}

impl Palette {
    /// The canon palette: the exact constants `alacer_widgets::theme`
    /// shipped through 0.16 — the values the blessed snapshot corpus
    /// was blessed with. A default build (no [`ThemeSeam`] installed)
    /// resolves these, so a retheme never silently re-paints an
    /// untouched app.
    pub const CANON: Self = Self {
        surface: Color::rgb(245, 246, 248),
        panel: Color::rgb(255, 255, 255),
        accent: Color::rgb(37, 99, 235),
        accent_pressed: Color::rgb(29, 78, 216),
        selection: Color {
            r: 37,
            g: 99,
            b: 235,
            a: 64,
        },
        text: Color::rgb(15, 23, 42),
        text_muted: Color::rgb(100, 116, 139),
        border: Color::rgb(203, 213, 225),
        on_accent: Color::WHITE,
    };
}

/// The theme seam: implemented by the app (or by any palette library
/// on top of it), installed through
/// [`crate::passes::PassPipeline::set_theme`], resolved by the paint
/// pass once per frame. `Send + Sync` like [`crate::style::MeasureFn`]
/// — the pipeline owns the implementation across the platform loop.
///
/// The trait's contract is total: every role answers, every frame. A
/// palette that cannot answer a role is a palette that cannot paint —
/// the [`Palette`] struct is the contract, the trait is the
/// installation.
pub trait ThemeSeam: Send + Sync + 'static {
    /// The roles this palette resolves, re-read once per paint pass —
    /// a palette may answer differently over its lifetime (a
    /// time-of-day tint, an OS setting it polls), the swap door's
    /// equality check prices the values it held at install time.
    fn palette(&self) -> Palette;

    /// The palette's tag — the stable name the causal log's
    /// [`crate::inspect::ThemeRecord`] carries (the wire vocabulary's
    /// naming rule): `""`-free, app-defined (`"canon"`, `"midnight"`,
    /// `"high_contrast"`).
    fn name(&self) -> &'static str;
}

/// The provided canon theme (§13.3, the provided-implementation rule
/// beside every seam — [`crate::trail::TrailLog`]'s precedent): the
/// palette the constants pinned, installable by name so a consumer
/// that swapped away can swap back through the same door it swapped
/// in. Installing it where nothing is installed is the no-op the door
/// answers `false` on.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonPalette;

impl ThemeSeam for CanonPalette {
    fn palette(&self) -> Palette {
        Palette::CANON
    }

    fn name(&self) -> &'static str {
        "canon"
    }
}

thread_local! {
    /// Whether the trail notes facts: enabled by the dispatching
    /// pipeline when a devtools observer installs
    /// ([`crate::passes`] `PassPipeline::set_observer`), off
    /// otherwise — the disabled trail costs one branch per swap (the
    /// A7 idle contract).
    static TRAIL_ENABLED: Cell<bool> = const { Cell::new(false) };
    /// The facts queued since the last tick drain — the handler side
    /// of the trail, bounded by [`THEME_TRAIL_CAP`] with oldest-first
    /// eviction, so a fact queued where no pipeline will ever drain
    /// still terminates.
    static TRAIL: RefCell<VecDeque<ThemeRecord>> = const { RefCell::new(VecDeque::new()) };
}

/// Enables or disables the theme trail (the pipeline's side of the
/// trail: `PassPipeline::set_observer` turns it on when a devtools
/// observer installs). Disabling never clears the queue — the drain
/// owns that.
pub(crate) fn set_theme_trail_enabled(enabled: bool) {
    TRAIL_ENABLED.with(|slot| slot.set(enabled));
}

/// Notes one theme fact on this thread's trail (the note site's side of
/// the trail, ADR-0079): a no-op while the trail is disabled, an
/// oldest-evicting push while it rides. [`crate::passes`]
/// `PassPipeline::set_theme` calls this at the exact place the swap
/// lands — so a status line, an inspector trail, or a test reads the
/// theme tier without re-implementing the observation at the call
/// site. Public like [`crate::frame_refusal::note_frame_refusal`]: the
/// note site's own door, priced by the bench corpus
/// (`theme/note_disabled`, `theme/note_enabled_bounded`) and usable by
/// a composition that drives its own theme door outside the pipeline.
/// One fact per swap that landed: a swap that lands where the palette
/// already is queues nothing (the door's idle rule), and the record
/// rides *beside* the door's `bool` answer, never instead of it.
pub fn note_theme(record: ThemeRecord) {
    TRAIL_ENABLED.with(|enabled| {
        if !enabled.get() {
            return;
        }
        tracing::trace!(
            target: "alacer::theme",
            palette = %record.palette,
            "theme swap queued for the next tick"
        );
        TRAIL.with(|slot| {
            let mut queue = slot.borrow_mut();
            if queue.len() >= THEME_TRAIL_CAP {
                queue.pop_front();
            }
            queue.push_back(record);
        });
    });
}

/// The dispatching pipeline's drain: every fact queued on this thread
/// since the last tick, oldest first — the whole queue clears (a fact
/// streams once), the records then riding the observer as
/// [`crate::inspect::InspectEvent::Theme`].
pub(crate) fn take_theme_trail() -> Vec<ThemeRecord> {
    TRAIL.with(|slot| slot.borrow_mut().drain(..).collect())
}

#[cfg(test)]
mod tests {
    use super::{CanonPalette, Palette, THEME_TRAIL_CAP, ThemeSeam, note_theme, take_theme_trail};
    use crate::color::Color;
    use crate::inspect::ThemeRecord;

    /// The canon palette pins the constants the blessed snapshot corpus
    /// was blessed with — byte for byte, the corpus's green depends on
    /// it.
    #[test]
    fn the_canon_palette_pins_the_blessed_constants() {
        let canon = Palette::CANON;
        assert_eq!(canon.surface, Color::rgb(245, 246, 248));
        assert_eq!(canon.panel, Color::rgb(255, 255, 255));
        assert_eq!(canon.accent, Color::rgb(37, 99, 235));
        assert_eq!(canon.accent_pressed, Color::rgb(29, 78, 216));
        assert_eq!(canon.selection, Color::new(37, 99, 235, 64));
        assert_eq!(canon.text, Color::rgb(15, 23, 42));
        assert_eq!(canon.text_muted, Color::rgb(100, 116, 139));
        assert_eq!(canon.border, Color::rgb(203, 213, 225));
        assert_eq!(canon.on_accent, Color::WHITE);
        assert_eq!(CanonPalette.palette(), canon);
        assert_eq!(CanonPalette.name(), "canon");
    }

    /// The disabled trail is a no-op: nothing queues, the drain answers
    /// empty (the A7 idle contract, the frame-refusal trail's own test).
    #[test]
    fn the_disabled_trail_notes_nothing() {
        note_theme(ThemeRecord {
            palette: "midnight".to_string(),
            repainted: 0,
        });
        assert!(
            take_theme_trail().is_empty(),
            "no observer, no trail — nothing queued"
        );
    }

    /// The enabled trail queues bounded, oldest-first eviction, and the
    /// drain clears whole: a fact streams once.
    #[test]
    fn the_enabled_trail_queues_bounded_and_drains_once() {
        super::set_theme_trail_enabled(true);
        for i in 0..(THEME_TRAIL_CAP + 8) {
            note_theme(ThemeRecord {
                palette: format!("theme-{i}"),
                repainted: i,
            });
        }
        let drained = take_theme_trail();
        assert_eq!(drained.len(), THEME_TRAIL_CAP, "the cap bounds the trail");
        assert_eq!(
            drained[0].palette, "theme-8",
            "the oldest evicted first (the ninth swap's tag)"
        );
        assert_eq!(
            drained[0].repainted, 8,
            "the count rides beside the tag — the swap's own reach"
        );
        assert!(
            take_theme_trail().is_empty(),
            "the drain clears whole — a fact streams once"
        );
        super::set_theme_trail_enabled(false);
    }
}
