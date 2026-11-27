//! The heading tier (M126, ADR-0130): the section title the document
//! outline walks.
//!
//! "This names a section" was a vocabulary hole: an app wiring a
//! document outline reached for a plain label — and the AT walked the
//! page as one flat run of text, the rotor's quick-nav, the outline
//! every reader skims by, having nothing to land on. [`Role::Heading`]
//! carries the "names a section" fact AccessKit's own `Heading` role
//! spells — not a bold label wearing bigger glyphs — and the depth in
//! the outline rides the level word the M113 tier published
//! ([`Widget::level`], AccessKit's `Level` property, zero-based where
//! ARIA's `aria-level` is one-based — the mapping one branch the
//! sink's existing word never grew): the heading's natural carrier,
//! never a second depth vocabulary.
//!
//! The tier prices the word and nothing else: a heading names
//! structure, it walls nothing — the section's content reads as it
//! always did, the heading's read routes the label's own. A heading is
//! not a control: it holds no focus, answers no activation, and the
//! AT's actions land on the widgets the section contains. The paint is
//! the label's own — the word the tree carries, the glyphs the app
//! styles ([`Heading::font_size`]), the outline's depth a fact and
//! never a font size.
//!
//! The widget never writes a11y directly (A1: masks are the only
//! currency).

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// A section title with a mandatory name (ADR-5: an unnamed heading
/// reads as bare styled text — the outline the AT cannot walk) and the
/// depth in the document outline the AT's quick-nav rides.
pub struct Heading {
    name: String,
    /// The depth in the outline (M113, ADR-0117): zero-based, AccessKit's
    /// own base beside ARIA's one-based `aria-level` — the AT reads the
    /// depth beside the name; `None` is the honest absence, a title
    /// that declared no depth announces none.
    level: Option<usize>,
    /// The language tier's word (M123, ADR-0127): the language the
    /// heading's text is written in; `None` is the honest absence.
    language: Option<String>,
    /// The app's text-color override; `None` borrows the theme's text
    /// role at paint time (ADR-0079).
    color: Option<Color>,
    font_size: f32,
    style: Style,
}

impl Heading {
    /// A heading titled `name`. The depth rides [`Heading::level`] — a
    /// heading that declares none announces the honest absence, never
    /// a guessed level.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: None,
            language: None,
            color: None,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// Mounts the title's depth (M113, ADR-0117): the place in the
    /// document outline said by the tree from the first frame —
    /// AccessKit's `Level` property (zero-based, beside ARIA's
    /// one-based `aria-level`) names the depth to the AT beside the
    /// name the ADR-5 contract requires. The word mirrors the outline
    /// it never styles: the depth is a fact, never a font size.
    #[must_use]
    pub fn level(mut self, level: usize) -> Self {
        self.level = Some(level);
        self
    }

    /// The language word (M123, ADR-0127): the BCP 47 tag the AT
    /// voices the title with ("fr-FR", "ja-JP"), the tag the tree says
    /// from the first frame. The word names the language the app
    /// declared, it never translates the text.
    #[must_use]
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Text color. Without it the heading borrows the theme's text
    /// role at paint time (ADR-0079).
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Font size override. The outline's depth is a fact and never a
    /// font size — the glyphs are the app's styling, the same override
    /// the label tier answers.
    #[must_use]
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Layout styles.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Replaces the title in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
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

    /// The one depth write (view reconciliation, the M113 tier):
    /// `None` clears to the honest absence. Report whether the depth
    /// moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the depth already is queues
    /// nothing (the M21 idle-frame property, at the heading tier).
    pub fn set_level(&mut self, level: Option<usize>) -> bool {
        if self.level == level {
            return false;
        }
        self.level = level;
        true
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
}

impl Widget for Heading {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Heading
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The level tier (M113, ADR-0117): the depth in the outline the
    /// AT's quick-nav walks, the honest absence when the title declared
    /// none — a text word, so the sink's existing mapping serves it
    /// with no second branch. Zero-based, AccessKit's own base beside
    /// ARIA's one-based `aria-level` — one branch with no translation,
    /// the word the trait's documentation carries.
    fn level(&self) -> Option<usize> {
        self.level
    }

    /// The language tier (M123, ADR-0127): the language the title's
    /// text is written in, the honest absence when the heading carries
    /// none.
    fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        if self.name.is_empty() {
            return;
        }
        // The borrowed role resolves per paint pass (ADR-0079): a plain
        // heading follows the installed palette, an overridden one
        // keeps the color its app set. The glyphs are the app's
        // styling — the outline's depth is the tree's fact, the paint
        // says nothing about it.
        let color = self.color.unwrap_or(ctx.palette().text);
        // Wrap at the laid-out width — the constraint the measure pass
        // saw, so the painted lines match the node's height (the
        // wrapped label's own rule).
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.name, self.font_size, Some(ctx.size().width));
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
            text: self.name.clone(),
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
    use super::Heading;
    use alacer_core::events::{EventCtx, PointerEvent};
    use alacer_core::geom::Point;
    use alacer_core::widget::{EventResponse, Role, Widget};

    #[test]
    fn role_name_and_depth_satisfy_the_contract() {
        let heading = Heading::new("Getting started");
        assert_eq!(heading.role(), Role::Heading);
        assert_eq!(heading.label().resolve(), "Getting started");
        assert!(heading.role().requires_name());
        assert!(
            !heading.focusable(),
            "a heading names structure — it is not a control, it holds no focus"
        );
        assert_eq!(
            Widget::level(&heading),
            None,
            "a heading that declared no depth announces none"
        );
        assert_eq!(
            Widget::language(&heading),
            None,
            "the language the honest absence by default"
        );
        assert_eq!(
            heading.toggled(),
            None,
            "a heading carries no selection fact"
        );
        assert_eq!(heading.table_slot(), None, "a heading carries no grid slot");
    }

    #[test]
    fn the_level_rides_the_trait_and_the_write_reports_the_move() {
        // M113, ADR-0117, at the heading tier: the zero-based depth the
        // outline walks, and the one write whose moved answer is the
        // only mask the tier costs.
        let mut heading = Heading::new("Installation").level(1);
        assert_eq!(
            Widget::level(&heading),
            Some(1),
            "the zero-based depth the outline walks — ARIA's aria-level reads 2"
        );
        assert!(
            !heading.set_level(Some(1)),
            "the idle write answers false — no masks"
        );
        assert!(heading.set_level(Some(3)), "the moved write answers true");
        assert_eq!(Widget::level(&heading), Some(3));
        assert!(heading.set_level(None), "clearing to the absence moved");
        assert_eq!(Widget::level(&heading), None);
        assert!(!heading.set_level(None), "the absence is already there");
    }

    #[test]
    fn the_language_write_reports_the_move_like_the_labels() {
        // M123, ADR-0127, at the heading tier: the word rides beside
        // the depth, the same idle-frame rule.
        let mut heading = Heading::new("Bienvenue").language("fr-FR");
        assert_eq!(Widget::language(&heading), Some("fr-FR"));
        assert!(!heading.set_language(Some("fr-FR".to_string())));
        assert!(heading.set_language(None), "the withdrawal moved");
        assert_eq!(Widget::language(&heading), None);
    }

    #[test]
    fn a_heading_answers_no_activation_and_holds_no_focus() {
        // The tier prices the word and nothing else: a heading is not a
        // control — the pointer moves through it ignored, the focus
        // walk never lands (the role's own answer to `focusable`), and
        // the AT's presses land on the widgets the section contains.
        let mut heading = Heading::new("Getting started");
        heading.set_focused(true);
        let mut ctx = EventCtx::new();
        assert_eq!(
            heading.on_pointer_event(
                &mut ctx,
                &PointerEvent::Down {
                    position: Point::new(4.0, 4.0),
                    button: alacer_core::events::MouseButton::Left,
                    click_count: 1,
                }
            ),
            EventResponse::Ignored,
            "the pointer's press reads, it never activates"
        );
        assert!(ctx.take().is_empty(), "the read queues no masks");
        assert!(
            !heading.on_a11y_action(alacer_core::widget::A11yAction::DefaultPress),
            "a heading has no write door — the ask is no one's to answer"
        );
    }

    #[test]
    fn the_title_write_replaces_the_name_in_place() {
        let mut heading = Heading::new("Getting started");
        heading.set_name("Getting started (updated)");
        assert_eq!(heading.label().resolve(), "Getting started (updated)");
    }
}
