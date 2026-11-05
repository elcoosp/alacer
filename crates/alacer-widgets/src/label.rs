//! The static text widget: paints its bound or literal string.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// Static text. The accessible name *is* the text.
pub struct Label {
    text: LabelText,
    color: Color,
    font_size: f32,
    style: Style,
}

enum LabelText {
    Fixed(String),
    Binding(RenderBinding<String>),
}

impl Label {
    /// A label with fixed text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: LabelText::Fixed(text.into()),
            color: crate::theme::TEXT,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// A label whose text tracks a [`RenderBinding`] — re-rendered when the
    /// bound signal changes, never rebuilt (§6.1).
    #[must_use]
    pub fn bound(binding: RenderBinding<String>) -> Self {
        Self {
            text: LabelText::Binding(binding),
            ..Self::new(String::new())
        }
    }

    /// Text color.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Font size override.
    #[must_use]
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Layout styles override.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Replaces the fixed text in place (view reconciliation, §6.1). A no-op
    /// on a binding-backed label: its text belongs to the reactive binding.
    pub fn set_text(&mut self, text: impl Into<String>) {
        if let LabelText::Fixed(slot) = &mut self.text {
            *slot = text.into();
        }
    }

    /// Sets the text color in place (view reconciliation).
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Sets the font size in place (view reconciliation).
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    fn current_text(&self) -> String {
        match &self.text {
            LabelText::Fixed(text) => text.clone(),
            LabelText::Binding(binding) => binding.with(|v| v.clone()),
        }
    }

    fn binding_handle(&self) -> Option<RenderBinding<String>> {
        match &self.text {
            LabelText::Fixed(_) => None,
            LabelText::Binding(binding) => Some(binding.clone()),
        }
    }
}

impl Widget for Label {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Label
    }

    fn label(&self) -> LabelSource {
        match self.binding_handle() {
            Some(binding) => LabelSource::Binding(binding),
            None => LabelSource::text(self.current_text()),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let text = self.current_text();
        if text.is_empty() {
            return;
        }
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&text, self.font_size, None);
            for glyph in &layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(glyph.x, glyph.y),
                    size: glyph.size,
                    color: self.color,
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    clip: None,
                });
            }
        });
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Text(TextSpec {
            text: self.current_text(),
            font_id: 0,
            font_size: self.font_size,
        }))
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}
