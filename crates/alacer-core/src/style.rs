//! Layout styles: the typed, M0-sized subset the layout engine understands.
//!
//! No runtime CSS engine (architecture §15): these are typed Rust properties
//! with reason-flag recalc semantics. `alacer-layout` converts them to Taffy.

/// Layout display of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    /// Participates in flex layout.
    #[default]
    Flex,
    /// Skipped by layout entirely.
    None,
}

/// Main-axis direction of a flex container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    /// Children run left-to-right.
    #[default]
    Row,
    /// Children run top-to-bottom.
    Column,
}

/// Alignment along the container's main axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainAxisAlignment {
    /// Pack to the start edge.
    #[default]
    Start,
    /// Center on the main axis.
    Center,
    /// Pack to the end edge.
    End,
}

/// Alignment along the container's cross axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossAxisAlignment {
    /// Pack to the start edge.
    #[default]
    Start,
    /// Center on the cross axis.
    Center,
    /// Pack to the end edge.
    End,
    /// Stretch to fill the cross axis.
    Stretch,
}

/// Typed layout styles for one node.
///
/// Paint properties (colors, borders) live on widgets, not here: styles are
/// layout-only, per §7.2 pass 5.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    /// Display mode; [`Display::None`] removes the node from layout and paint.
    pub display: Display,
    /// Main-axis direction for children.
    pub flex_direction: FlexDirection,
    /// How much this node grows into free space along the parent main axis.
    pub grow: f32,
    /// How much this node shrinks when the parent overflows.
    pub shrink: f32,
    /// Fixed size, when set; `None` axes resolve from content.
    pub size: Size2,
    /// Lower bounds per axis.
    pub min_size: Size2,
    /// Upper bounds per axis.
    pub max_size: Size2,
    /// Uniform padding inset, applied to all four edges.
    pub padding: f32,
    /// Gap between children along the main axis.
    pub gap: f32,
    /// Main-axis alignment of children.
    pub justify: MainAxisAlignment,
    /// Cross-axis alignment of children.
    pub align: CrossAxisAlignment,
    /// Paint children clipped to this node's bounds (scroll viewports).
    pub clip_overflow: bool,
}

/// A per-axis pair of optional f32 values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size2 {
    /// Constraint on the x axis.
    pub width: Option<f32>,
    /// Constraint on the y axis.
    pub height: Option<f32>,
}

impl Size2 {
    /// Both axes unset.
    #[must_use]
    pub const fn auto() -> Self {
        Self {
            width: None,
            height: None,
        }
    }

    /// Both axes set to the same value.
    #[must_use]
    pub const fn uniform(v: f32) -> Self {
        Self {
            width: Some(v),
            height: Some(v),
        }
    }
}

impl Style {
    /// Styles with `flex_direction: Column` — the app-shell default.
    #[must_use]
    pub fn column() -> Self {
        Self {
            flex_direction: FlexDirection::Column,
            ..Self::default()
        }
    }

    /// Styles with `flex_direction: Row`.
    #[must_use]
    pub fn row() -> Self {
        Self {
            flex_direction: FlexDirection::Row,
            ..Self::default()
        }
    }
}

/// A leaf widget's intrinsic measure request, consumed by the layout engine.
#[derive(Debug, Clone, PartialEq)]
pub enum MeasureSpec {
    /// A fixed intrinsic size (spacer, image placeholder).
    Fixed(crate::geom::Size),
    /// Text whose size depends on font resolution and available width.
    Text(TextSpec),
}

/// The parameters a text measurement needs.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSpec {
    /// The text to measure.
    pub text: String,
    /// Interned font tag (see `alacer-text`).
    pub font_id: u64,
    /// Font size in pixels.
    pub font_size: f32,
}

/// The text-measurement seam: implemented by `alacer-text` (parley), consumed
/// by `alacer-layout`'s measure functions. Keeping the trait here means the
/// layout layer never depends on the text layer.
pub trait MeasureFn: Send + Sync + 'static {
    /// Intrinsic size of `spec` when wrapped at `max_width` (`None` = unbounded).
    fn measure(&self, spec: &TextSpec, max_width: Option<f32>) -> crate::geom::Size;
}

#[cfg(test)]
mod tests {
    use super::{FlexDirection, Style};

    #[test]
    fn column_builder_sets_direction_only() {
        let s = Style::column();
        assert_eq!(s.flex_direction, FlexDirection::Column);
        assert_eq!(s.grow, 0.0);
        assert_eq!(s.padding, 0.0);
        assert!(!s.clip_overflow);
    }

    #[test]
    fn row_builder_sets_direction_only() {
        assert_eq!(Style::row().flex_direction, FlexDirection::Row);
    }
}
