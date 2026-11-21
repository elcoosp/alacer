//! The static text widget: paints its bound or literal string.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Vector};
use alacer_core::scene::{Compose, PaintCtx};
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// Static text. The accessible name *is* the text.
pub struct Label {
    text: LabelText,
    /// The language tier's word (M123, ADR-0127): the language the
    /// label's text is written in; `None` is the honest absence.
    language: Option<String>,
    /// The app's text color override; `None` borrows the theme's text
    /// role at paint time (ADR-0079) — the role re-resolves on a
    /// retheme, the override never does.
    color: Option<Color>,
    font_size: f32,
    /// The compose tier's paint state (§14.2's W3): the fade and
    /// translation the label paints with — a paint fact, never a layout
    /// one; an animated frame moves it without re-measuring.
    compose: Compose,
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
            language: None,
            color: None,
            font_size: crate::theme::FONT_SIZE,
            compose: Compose::IDENTITY,
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

    /// Text color. Without it the label borrows the theme's text role
    /// at paint time (ADR-0079).
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
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

    /// The compose fade (§14.2's W3 mechanism): the opacity the label
    /// paints with — the compose tier's own fact, never a color rewrite.
    /// An animated frame moves it without touching layout, the palette,
    /// or the text: the fade rides the compose pass, the alpha folds into
    /// the glyphs at record time. Clamped to 0.0–1.0.
    #[must_use]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.compose.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// The compose translation (the M8 scrolling contract's tier): the
    /// offset the label paints at, in pixels — record-time coordinates
    /// riding the paint pass, never a layout move (no re-measure rides
    /// it).
    #[must_use]
    pub fn translate(mut self, translation: Vector) -> Self {
        self.compose.translation = translation;
        self
    }

    /// Replaces the compose state in place (view reconciliation) — the
    /// animated frame's door. Reports whether the state moved, so the
    /// caller requests `PAINT` only when the answer is `true` (the
    /// idle-frame rule at the compose tier: a write landing where the
    /// state already is queues nothing).
    pub fn set_compose(&mut self, compose: Compose) -> bool {
        if self.compose == compose {
            return false;
        }
        self.compose = compose;
        true
    }

    /// The language word (M123, ADR-0127): the language the label's
    /// text is written in — the BCP 47 tag the platform AT voices the
    /// text with ("fr-FR", "ja-JP"), the tag the tree says from the
    /// first frame (`aria-lang`, AccessKit's `Language` property). The
    /// word names the language the app declared, it never translates
    /// the text: the label's read routes the widget's own.
    #[must_use]
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// The one language write (M123, ADR-0127, view reconciliation):
    /// `None` clears to the honest absence. Report whether the word
    /// moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the language tier).
    pub fn set_language(&mut self, language: Option<String>) -> bool {
        if self.language == language {
            return false;
        }
        self.language = language;
        true
    }

    /// Replaces the fixed text in place (view reconciliation, §6.1). A no-op
    /// on a binding-backed label: its text belongs to the reactive binding.
    pub fn set_text(&mut self, text: impl Into<String>) {
        if let LabelText::Fixed(slot) = &mut self.text {
            *slot = text.into();
        }
    }

    /// Sets the text color in place (view reconciliation) — an explicit
    /// override, immune to rethemes.
    pub fn set_color(&mut self, color: Color) {
        self.color = Some(color);
    }

    /// Sets the font size in place (view reconciliation).
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    fn current_text(&self) -> String {
        match &self.text {
            LabelText::Fixed(text) => text.clone(),
            LabelText::Binding(binding) => binding.with(Clone::clone),
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

    /// The language tier (M123, ADR-0127): the language the label's
    /// text is written in, the honest absence when the label carries
    /// none — a text word, so every other role inherits the default.
    fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// The compose tier's ride: the label's built fade and translation
    /// join the walk — its own glyphs fold them in at record time, and a
    /// subtree under the label would inherit them the same way.
    fn compose(&self) -> Compose {
        self.compose
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let text = self.current_text();
        if text.is_empty() {
            return;
        }
        // The borrowed role resolves per paint pass (ADR-0079): a plain
        // label follows the installed palette, an overridden one keeps
        // the color its app set.
        let color = self.color.unwrap_or(ctx.palette().text);
        // Wrap at the laid-out width — the constraint the measure pass saw,
        // so the painted lines match the node's height (§5.2's wrapped label).
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&text, self.font_size, Some(ctx.size().width));
            for glyph in &layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(glyph.x, glyph.y),
                    size: glyph.size,
                    color,
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

#[cfg(test)]
mod tests {
    use super::Label;
    use alacer_core::Widget as _;
    use alacer_core::geom::Vector;
    use alacer_core::scene::Compose;

    #[test]
    fn the_builders_set_the_compose_state_the_pass_reads() {
        let label = Label::new("LIVE")
            .opacity(0.5)
            .translate(Vector::new(4.0, 0.0));
        let compose = label.compose();
        assert_eq!(compose.opacity, 0.5, "the fade the compose pass will fold");
        assert_eq!(
            compose.translation,
            Vector::new(4.0, 0.0),
            "the shift the compose pass will fold"
        );
    }

    #[test]
    fn opacity_clamps_to_the_unit_range() {
        assert_eq!(Label::new("LIVE").opacity(1.5).compose().opacity, 1.0);
        assert_eq!(Label::new("LIVE").opacity(-0.5).compose().opacity, 0.0);
    }

    #[test]
    fn set_compose_reports_whether_the_state_moved() {
        let mut label = Label::new("LIVE");
        assert!(
            !label.set_compose(Compose::IDENTITY),
            "the identity lands where the label already is — no mask, an idle frame"
        );
        assert!(
            label.set_compose(Compose::faded(0.4)),
            "a new fade moves the state — PAINT is worth requesting"
        );
        assert!(
            !label.set_compose(Compose::faded(0.4)),
            "the same fade lands where it already is — the idle-frame rule"
        );
    }
}
