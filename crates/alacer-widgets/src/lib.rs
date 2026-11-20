//! Alacer's widget library: the M0 set the counter and the scrolling list
//! need, the M3 text input (the editing contract's editor — selection,
//! pointer, word, clipboard, and history tiers ride it), the M6 canon
//! (checkbox, data table) the todo and data-table apps need, the M20 value
//! tier's slider, the M21 value tier's read half (the progress indicator
//! the app reports through), the M82 canvas (the app's own paint closure
//! through the paint seam), and the panel/table/list vocabulary the M5
//! tour assembles.
//!
//! Widgets are retained: created once, mutated in place, painted on demand.
//! Reactive text lands through `RenderBinding`s; input lands through
//! `on_pointer_event` + `EventCtx` mask requests. Every widget carries a
//! required role and label (ADR-5).

mod button;
mod canon;
mod canvas;
mod checkbox;
mod column;
mod dialog;
mod heading;
mod image;
mod label;
mod link;
mod list;
mod menu;
mod meter;
mod panel;
mod progress;
mod radio;
mod registry;
mod slider;
mod spin_button;
mod switch;
mod tab;
mod tab_panel;
mod table;
mod text_input;
mod toolbar;
mod tooltip;
mod tree;

pub use button::Button;
pub use canon::{DataTableHandles, TodoHandles, TodoItem, data_table_app, todo_app};
pub use canvas::Canvas;
pub use checkbox::Checkbox;
pub use column::{Column, Row, Spacer};
pub use dialog::Dialog;
pub use heading::Heading;
pub use image::Image;
pub use label::Label;
pub use link::Link;
pub use list::{List, ListHandle};
pub use menu::{Menu, MenuBar, MenuItem};
pub use meter::Meter;
pub use panel::Panel;
pub use progress::Progress;
pub use radio::{Radio, RadioGroup};
pub use registry::default_registry;
pub use slider::Slider;
pub use spin_button::SpinButton;
pub use switch::Switch;
pub use tab::{Tab, TabList};
pub use tab_panel::TabPanel;
pub use table::{ColumnHeader, Table, TableCell, TableRow};
pub use text_input::TextInput;
pub use toolbar::Toolbar;
pub use tooltip::Tooltip;
pub use tree::{Tree, TreeItem};

use alacer_core::widget::Widget;

/// Boxes any widget into the arena's erased type.
pub trait BoxedWidget: Widget + Sized + 'static {
    /// Boxes the widget.
    fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl<W: Widget> BoxedWidget for W {}

use alacer_core::color::Color;
use alacer_core::scene::PaintCtx;

/// The shared color constants and type sizes every widget reads.
///
/// Since M73 (ADR-0079) these are the **canon pin**, not the paint
/// source: a widget's un-overridden paint roles resolve from the
/// installed [`alacer_core::theme::ThemeSeam`] at paint time
/// ([`alacer_core::theme::Palette::CANON`] pins these exact values, so
/// a default build paints what 0.16 painted), while an app-set color —
/// a builder prop or a `set_*` write — stays the override it was.
/// What remains here are the canon constants themselves and the font
/// metrics (the seam is paint-color-only; runtime font metrics are a
/// pre-registered re-open rule).
pub mod theme {
    use super::Color;

    /// Window background.
    pub const SURFACE: Color = Color::rgb(245, 246, 248);
    /// Container background.
    pub const PANEL: Color = Color::rgb(255, 255, 255);
    /// Primary button fill.
    pub const ACCENT: Color = Color::rgb(37, 99, 235);
    /// Primary button fill while pressed.
    pub const ACCENT_PRESSED: Color = Color::rgb(29, 78, 216);
    /// Text-selection highlight: the accent at a quarter opacity, behind
    /// the selected range (M11, ADR-0017).
    pub const SELECTION: Color = Color::rgb(37, 99, 235).with_alpha(64);
    /// Body text.
    pub const TEXT: Color = Color::rgb(15, 23, 42);
    /// Muted text.
    pub const TEXT_MUTED: Color = Color::rgb(100, 116, 139);
    /// Hairline separators.
    pub const BORDER: Color = Color::rgb(203, 213, 225);
    /// Base font size.
    pub const FONT_SIZE: f32 = 16.0;
    /// Line height multiplier baked into text layout.
    pub const LINE_HEIGHT: f32 = 1.25;
}

/// Paints a solid background quad for a widget's full bounds.
pub(crate) fn paint_background(
    ctx: &mut PaintCtx<'_>,
    size: (f32, f32),
    color: Color,
    radius: f32,
    border: (Color, f32),
) {
    let bounds = alacer_core::geom::Rect::new(
        alacer_core::geom::Point::zero(),
        alacer_core::geom::Size::new(size.0, size.1),
    );
    ctx.push_quad(alacer_core::scene::Quad {
        rect: bounds,
        color,
        corner_radius: radius,
        border_color: border.0,
        border_width: border.1,
        clip: None,
    });
}

#[cfg(test)]
mod tests {
    use super::theme;

    #[test]
    fn theme_colors_are_distinct_enough_for_snapshots() {
        assert_ne!(theme::SURFACE, theme::PANEL);
        assert_ne!(theme::ACCENT, theme::ACCENT_PRESSED);
        assert_ne!(theme::TEXT, theme::TEXT_MUTED);
        const {
            assert!(theme::FONT_SIZE > 0.0);
        }
    }

    #[test]
    fn the_canon_palette_pins_the_widget_constants() {
        // ADR-0079's pin: the seam's canon palette is byte-identical to
        // the constants this crate shipped through 0.16 — a default
        // build paints what the blessed corpus was blessed with.
        assert_eq!(alacer_core::theme::Palette::CANON.surface, theme::SURFACE);
        assert_eq!(alacer_core::theme::Palette::CANON.panel, theme::PANEL);
        assert_eq!(alacer_core::theme::Palette::CANON.accent, theme::ACCENT);
        assert_eq!(
            alacer_core::theme::Palette::CANON.accent_pressed,
            theme::ACCENT_PRESSED
        );
        assert_eq!(
            alacer_core::theme::Palette::CANON.selection,
            theme::SELECTION
        );
        assert_eq!(alacer_core::theme::Palette::CANON.text, theme::TEXT);
        assert_eq!(
            alacer_core::theme::Palette::CANON.text_muted,
            theme::TEXT_MUTED
        );
        assert_eq!(alacer_core::theme::Palette::CANON.border, theme::BORDER);
    }
}
