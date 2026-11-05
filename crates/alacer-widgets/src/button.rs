//! The button: clickable, pressable, named.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent, PointerEvent as PE};
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// An actionable button (role `Button`, name mandatory — ADR-5).
pub struct Button {
    label: String,
    pressed: bool,
    fill: Color,
    fill_pressed: Color,
    text_color: Color,
    font_size: f32,
    corner_radius: f32,
    style: Style,
    on_click: Option<Box<dyn FnMut()>>,
}

impl Button {
    /// A button labeled `label`.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            pressed: false,
            fill: crate::theme::ACCENT,
            fill_pressed: crate::theme::ACCENT_PRESSED,
            text_color: Color::WHITE,
            font_size: crate::theme::FONT_SIZE,
            corner_radius: 6.0,
            style: Style::default(),
            on_click: None,
        }
    }

    /// Installs the click handler; invoked on release inside the button.
    pub fn on_click(mut self, handler: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Fill colors `(rest, pressed)`.
    #[must_use]
    pub fn colors(mut self, rest: Color, pressed: Color) -> Self {
        self.fill = rest;
        self.fill_pressed = pressed;
        self
    }

    /// Text color.
    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Corner radius.
    #[must_use]
    pub fn radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Layout styles.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Whether the button is currently pressed (test introspection).
    #[must_use]
    pub const fn is_pressed(&self) -> bool {
        self.pressed
    }
}

impl Widget for Button {
    fn role(&self) -> Role {
        Role::Button
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.label.clone())
    }

    fn style(&self) -> Style {
        Style {
            padding: 10.0,
            ..self.style.clone()
        }
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The laid-out size comes from the pass system; no widget-side caching.
        let size = (ctx.size().width, ctx.size().height);
        let fill = if self.pressed {
            self.fill_pressed
        } else {
            self.fill
        };
        crate::paint_background(
            ctx,
            size,
            fill,
            self.corner_radius,
            (Color::TRANSPARENT, 0.0),
        );

        // Centered text.
        alacer_text::with_text(|text_ctx| {
            let layout = text_ctx.layout_text(&self.label, self.font_size, None);
            let text_origin = Point::new(
                (size.0 - layout.width).max(0.0) * 0.5,
                (size.1 - layout.height).max(0.0) * 0.5,
            );
            for glyph in &layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(text_origin.x + glyph.x, text_origin.y + glyph.y),
                    size: glyph.size,
                    color: self.text_color,
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    clip: None,
                });
            }
        });
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        let text_size = alacer_text::with_text(|text_ctx| {
            text_ctx.measure(
                &TextSpec {
                    text: self.label.clone(),
                    font_id: 0,
                    font_size: self.font_size,
                },
                None,
            )
        });
        Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(
            text_size.width + 28.0,
            text_size.height.max(24.0) + 16.0,
        )))
    }

    fn on_a11y_action(&mut self, action: alacer_core::widget::A11yAction) -> bool {
        if action == alacer_core::widget::A11yAction::DefaultPress {
            if let Some(handler) = &mut self.on_click {
                handler();
            }
            return true;
        }
        false
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PE) -> EventResponse {
        match event {
            PointerEvent::Down {
                button: _button, ..
            } => {
                self.pressed = true;
                ctx.request(alacer_core::Change::PAINT);
                EventResponse::Handled
            }
            PointerEvent::Up { .. } => {
                let was_pressed = self.pressed;
                self.pressed = false;
                ctx.request(alacer_core::Change::PAINT);
                if was_pressed && let Some(handler) = &mut self.on_click {
                    handler();
                }
                EventResponse::Handled
            }
            PointerEvent::Leave => {
                self.pressed = false;
                ctx.request(alacer_core::Change::PAINT);
                EventResponse::Handled
            }
            PointerEvent::Move { .. } | PointerEvent::Wheel { .. } => EventResponse::Ignored,
        }
    }
}
