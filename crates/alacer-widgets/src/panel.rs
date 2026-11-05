//! A fixed-size, single-color panel — list items, cards, separators.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// A solid rectangle with a fixed height (width fills the parent).
pub struct Panel {
    color: Color,
    height: f32,
}

impl Panel {
    /// A panel of `height` pixels in `color`.
    #[must_use]
    pub fn new(height: f32, color: Color) -> Self {
        Self { color, height }
    }

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for Panel {
    fn role(&self) -> Role {
        Role::Decoration
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("panel")
    }

    fn style(&self) -> Style {
        Style {
            size: alacer_core::style::Size2 {
                width: None,
                height: Some(self.height),
            },
            ..Style::default()
        }
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        crate::paint_background(
            ctx,
            (ctx.size().width, ctx.size().height),
            self.color,
            4.0,
            (Color::TRANSPARENT, 0.0),
        );
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(
            10.0,
            self.height,
        )))
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::Panel;
    use alacer_core::color::Color;
    use alacer_core::style::MeasureSpec;
    use alacer_core::widget::{Role, Widget as _};

    #[test]
    fn panel_measures_its_height_and_is_a_decoration() {
        let panel = Panel::new(32.0, Color::BLACK);
        assert_eq!(panel.role(), Role::Decoration);
        match panel.measure_spec() {
            Some(MeasureSpec::Fixed(size)) => assert_eq!(size.height, 32.0),
            other => panic!("unexpected spec {other:?}"),
        }
    }
}
