//! Flex containers: [`Column`] and [`Row`], plus [`Spacer`].

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::style::{CrossAxisAlignment, MainAxisAlignment, MeasureSpec, Style};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// A vertically-stacking container. Children are arena-managed; the container
/// only supplies layout styles (§7.2 pass 5).
pub struct Column {
    style: Style,
}

impl Column {
    /// An empty column.
    #[must_use]
    pub fn new() -> Self {
        Self {
            style: Style::column(),
        }
    }

    /// Gap between children.
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = gap;
        self
    }

    /// Uniform padding.
    #[must_use]
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding = padding;
        self
    }

    /// Grow factor.
    #[must_use]
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.grow = grow;
        self
    }

    /// Fixed height.
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.style.size.height = Some(height);
        self
    }

    /// Fixed width.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.style.size.width = Some(width);
        self
    }

    /// Main-axis alignment.
    #[must_use]
    pub fn justify(mut self, justify: MainAxisAlignment) -> Self {
        self.style.justify = justify;
        self
    }

    /// Cross-axis alignment.
    #[must_use]
    pub fn align(mut self, align: CrossAxisAlignment) -> Self {
        self.style.align = align;
        self
    }

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Column {
    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::FromChildren
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

/// A horizontally-stacking container.
pub struct Row {
    style: Style,
}

impl Row {
    /// An empty row.
    #[must_use]
    pub fn new() -> Self {
        Self {
            style: Style::row(),
        }
    }

    /// Gap between children.
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = gap;
        self
    }

    /// Uniform padding.
    #[must_use]
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding = padding;
        self
    }

    /// Grow factor.
    #[must_use]
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.grow = grow;
        self
    }

    /// Fixed height.
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.style.size.height = Some(height);
        self
    }

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Row {
    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::FromChildren
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

/// A fixed-size empty box — layout filler.
pub struct Spacer {
    size: f32,
}

impl Spacer {
    /// A spacer of `size` pixels along the parent's main axis.
    #[must_use]
    pub fn new(size: f32) -> Self {
        Self { size }
    }

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for Spacer {
    fn role(&self) -> Role {
        Role::Decoration
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("spacer")
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(
            self.size, self.size,
        )))
    }

    fn accepts_pointer(&self) -> bool {
        false
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, Row, Spacer};
    use alacer_core::style::{FlexDirection, MeasureSpec};
    use alacer_core::widget::Widget as _;

    #[test]
    fn column_and_row_set_their_directions() {
        let col = Column::new().gap(8.0).padding(4.0);
        assert_eq!(col.style().flex_direction, FlexDirection::Column);
        assert_eq!(col.style().gap, 8.0);
        assert_eq!(col.style().padding, 4.0);

        let row = Row::new();
        assert_eq!(row.style().flex_direction, FlexDirection::Row);
    }

    #[test]
    fn spacer_measures_fixed() {
        let spacer = Spacer::new(24.0);
        match spacer.measure_spec() {
            Some(MeasureSpec::Fixed(size)) => {
                assert_eq!((size.width, size.height), (24.0, 24.0));
            }
            other => panic!("expected fixed spec, got {other:?}"),
        }
        assert!(!spacer.accepts_pointer());
    }

    #[test]
    fn builders_chain_without_suprise() {
        let col = Column::new().height(300.0).width(200.0).grow(1.0);
        let s = col.style();
        assert_eq!(s.size.height, Some(300.0));
        assert_eq!(s.size.width, Some(200.0));
        assert_eq!(s.grow, 1.0);
    }
}
