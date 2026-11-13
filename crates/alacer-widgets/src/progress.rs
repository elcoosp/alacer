//! The progress indicator: the value tier's read half (M21, ADR-0027).
//!
//! A bounded scalar the *app* reports and the AT reads. The write
//! vocabulary is the app's, and it has two doors that land in the same
//! `PAINT | A11Y` currency: a [`RenderBinding`] the flux bridge drives
//! (one write batch, the caller's masks — the reactive-text seam, at a
//! value), and `set_value` for the mutate seam (the app takes the arena
//! lease, writes in place, and requests the write's own masks on the
//! guard; the answer is the `bool` that says whether the value moved, so
//! a write that lands where the value already is queues nothing — the
//! idle-frame property the storm budget leans on, at the app seam).
//!
//! The AT tier is read-only here by construction: every action is refused
//! honestly — never munged into a step or a press (the M11 rule) — a
//! reporting widget has no write door, and the number the tree carries is
//! the M3 `value()` contract's answer. The widget never writes a11y
//! directly (A1: masks are the only currency).

use std::cmp::Ordering;

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, Widget};

/// A read-only bounded-scalar report with a mandatory name (ADR-5: an
/// unnamed progress bar reads as a bare number). The value rides the M3
/// `value()` contract — the a11y tree carries it, and the app writes it
/// through the bridge or the mutate seam, never the AT.
pub struct Progress {
    name: String,
    min: f32,
    max: f32,
    source: ProgressValue,
    fill: Color,
    text_color: Color,
    font_size: f32,
    style: Style,
}

/// Where the reported value comes from: the app's imperative write
/// (`set_value`) or the reactive binding the flux bridge drives.
enum ProgressValue {
    Fixed(f32),
    Binding(RenderBinding<f32>),
}

impl Progress {
    /// The bar's paint height.
    const BAR_HEIGHT: f32 = 10.0;
    /// The gap between the bar and the name.
    const GAP: f32 = 6.0;
    /// The bar's default width.
    const WIDTH: f32 = 180.0;

    /// A progress bar labeled `name`, spanning `min..=max`, reporting
    /// `min` until the app writes.
    ///
    /// # Panics
    ///
    /// Panics when `min` or `max` is not finite, or when `min >= max` (a
    /// degenerate range has no value to report) — the invariants every
    /// read relies on.
    #[must_use]
    pub fn new(name: impl Into<String>, min: f32, max: f32) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min < max,
            "the progress bar's range must be finite with min < max, got {min}..={max}"
        );
        Self {
            name: name.into(),
            min,
            max,
            source: ProgressValue::Fixed(min),
            fill: crate::theme::ACCENT,
            text_color: crate::theme::TEXT,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// A progress bar whose value tracks a [`RenderBinding`] — the app
    /// writes a signal, the flux bridge queues the caller's masks, and
    /// the widget paints and reports from the binding (§6.1; the
    /// reactive-text seam, at a value). The report is the binding's
    /// number, read live — the `Label::bound` rule: the app's voice, not
    /// the widget's rewrite — while the paint clamps into the range.
    ///
    /// # Panics
    ///
    /// Panics under the same range invariants as [`Progress::new`].
    #[must_use]
    pub fn bound(name: impl Into<String>, min: f32, max: f32, binding: RenderBinding<f32>) -> Self {
        let mut progress = Self::new(name, min, max);
        progress.source = ProgressValue::Binding(binding);
        progress
    }

    /// Initial reported value; clamped into the range. A non-finite
    /// request is ignored — not a value the widget can carry.
    #[must_use]
    pub fn value(mut self, value: f32) -> Self {
        if value.is_finite()
            && let ProgressValue::Fixed(reported) = &mut self.source
        {
            *reported = Self::clamp(value, self.min, self.max);
        }
        self
    }

    /// The fill's color (the reported span of the bar).
    #[must_use]
    pub fn fill(mut self, fill: Color) -> Self {
        self.fill = fill;
        self
    }

    /// Text color of the label.
    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Layout styles.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// The reported value (test introspection — the same number the M3
    /// `value()` contract carries into the a11y tree).
    #[must_use]
    pub fn current_value(&self) -> f32 {
        self.reported()
    }

    /// The one write funnel: clamp into the range, report whether the
    /// value moved. The caller requests the write's own masks — `PAINT |
    /// A11Y` — only when this answers `true`, so a write that lands where
    /// the value already is queues nothing. A non-finite request is
    /// refused honestly (`false`, never munged). A no-op on a
    /// binding-backed bar: its value belongs to the reactive binding (the
    /// `Label::set_text` rule).
    pub fn set_value(&mut self, value: f32) -> bool {
        let ProgressValue::Fixed(reported) = &mut self.source else {
            return false;
        };
        if !value.is_finite() {
            return false;
        }
        let value = Self::clamp(value, self.min, self.max);
        if Self::same_value(value, *reported) {
            return false;
        }
        *reported = value;
        true
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// The one float equality the widget owns: `total_cmp` ordering —
    /// `f32::eq` is a lint trap the whole workspace refuses.
    fn same_value(a: f32, b: f32) -> bool {
        a.total_cmp(&b) == Ordering::Equal
    }

    /// The clamp the funnel and the builder share (the range is a wall).
    fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.clamp(min, max)
    }

    /// The value the widget reports, from wherever the app writes it.
    fn reported(&self) -> f32 {
        match &self.source {
            ProgressValue::Fixed(value) => *value,
            ProgressValue::Binding(binding) => binding.with(|value| *value),
        }
    }

    /// The reported value's position in `0.0..=1.0` along the bar. The
    /// paint is bounded (the bar is a bounded control's body) even when a
    /// binding carries a number past the range — the report is the app's
    /// voice, the paint is the widget's.
    fn ratio(&self) -> f32 {
        let value = self.reported();
        ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// Intrinsic size: the bar, the gap, and the name.
    fn intrinsic(name: &str, font_size: f32) -> Size {
        let text = alacer_core::text_seam::with_seam(|seam| {
            seam.measure(
                &TextSpec {
                    text: name.to_string(),
                    font_id: 0,
                    font_size,
                },
                None,
            )
        });
        Size::new(Self::WIDTH, Self::BAR_HEIGHT + Self::GAP + text.height)
    }
}

impl Widget for Progress {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::ProgressIndicator
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    fn style(&self) -> Style {
        Style {
            padding: 4.0,
            ..self.style.clone()
        }
    }

    /// The M3 value contract: the AT reads the reported scalar, not just
    /// the name.
    fn value(&self) -> Option<String> {
        Some(format!("{}", self.reported()))
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let width = ctx.size().width;
        let ratio = self.ratio();

        // The track: a rounded rail the full width.
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(width, Self::BAR_HEIGHT)),
            color: crate::theme::BORDER,
            corner_radius: Self::BAR_HEIGHT * 0.5,
            border_color: crate::theme::BORDER,
            border_width: 0.0,
            clip: None,
        });

        // The fill: the reported span of the track.
        let fill_width = (width * ratio).max(0.0);
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(
                Point::new(0.0, 0.0),
                Size::new(fill_width, Self::BAR_HEIGHT),
            ),
            color: self.fill,
            corner_radius: Self::BAR_HEIGHT * 0.5,
            border_color: self.fill,
            border_width: 0.0,
            clip: None,
        });

        // The name, under the bar.
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.name, self.font_size, None);
            let text_origin = Point::new(0.0, Self::BAR_HEIGHT + Self::GAP);
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
        Some(MeasureSpec::Fixed(Self::intrinsic(
            &self.name,
            self.font_size,
        )))
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        // A report is not a control: the pointer is the hover's passenger,
        // never a write.
        EventResponse::Ignored
    }

    /// The AT tier is read-only (M21, ADR-0027): a reporting widget has no
    /// write door — every action is refused honestly, never munged into a
    /// step or a press (the M11 rule). The tree keeps the value the app
    /// wrote. Since M40 (ADR-0046) the value-tier refusals stream as
    /// [`alacer_core::inspect::ValueRecord`]s — the `no_write_door`
    /// species the `bool` collapsed into one silence, the read-only
    /// tier's honesty visible on the log (the clipboard trail's shape,
    /// ADR-0045); a question outside the value tier (a press, a scroll,
    /// a text edit) is not a value fact and stays silent. Since M44
    /// (ADR-0050) the refusal carries the ask's honest absence: no ask
    /// ever entered a door the widget does not have — the field's
    /// `None` is the fact (the door-less control never computed one).
    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        let tier = match action {
            A11yAction::SetValue(_) => alacer_core::ValueAction::SetValue,
            A11yAction::Increment => alacer_core::ValueAction::Increment,
            A11yAction::Decrement => alacer_core::ValueAction::Decrement,
            A11yAction::DefaultPress
            | A11yAction::ScrollUp(_)
            | A11yAction::ScrollDown(_)
            | A11yAction::ReplaceSelectedText(_)
            | A11yAction::SetTextSelection(_) => return false,
        };
        alacer_core::value::note_write(alacer_core::ValueRecord {
            action: tier,
            outcome: alacer_core::ValueOutcome::Refused(alacer_core::ValueRefusal::NoWriteDoor),
            value: self.reported(),
            requested: None,
            target: None,
            // A write that moved nothing pinned nothing (M49, ADR-0055):
            // the door-less control's refusal carries the honest
            // absence, the wall a delivered write pins being the
            // slider's story.
            pinned: None,
        });
        false
    }
}

#[cfg(test)]
mod tests {
    use super::Progress;
    use alacer_core::widget::{A11yAction, Role, Widget};

    /// The float equality the tests own (the widget's own rule).
    fn same(a: f32, b: f32) -> bool {
        a.total_cmp(&b) == std::cmp::Ordering::Equal
    }

    #[test]
    fn role_name_and_value_satisfy_the_contract() {
        let progress = Progress::new("Batch", 0.0, 100.0);
        assert_eq!(progress.role(), Role::ProgressIndicator);
        assert_eq!(progress.role().name(), "progress_indicator");
        assert_eq!(progress.label().resolve(), "Batch");
        assert!(progress.role().requires_name());
        assert!(!progress.focusable(), "a report holds no focus");
        assert_eq!(Widget::value(&progress), Some("0".to_string()));
        assert_eq!(progress.toggled(), None);
        assert_eq!(progress.table_slot(), None);
        assert_eq!(progress.selection(), None);
    }

    #[test]
    fn set_value_clamps_and_reports_the_move() {
        let mut progress = Progress::new("Batch", 0.0, 100.0);
        assert!(progress.set_value(42.0), "the write moved the value");
        assert!(same(progress.current_value(), 42.0));
        assert_eq!(Widget::value(&progress), Some("42".to_string()));

        assert!(progress.set_value(1000.0), "the wall answers a move");
        assert!(same(progress.current_value(), 100.0), "the range is a wall");
        assert!(progress.set_value(-5.0), "the wall answers a move");
        assert!(same(progress.current_value(), 0.0), "the range is a wall");
    }

    #[test]
    fn a_no_op_or_non_finite_write_is_no_move() {
        let mut progress = Progress::new("Batch", 0.0, 100.0).value(30.0);
        assert!(
            !progress.set_value(30.0),
            "a write that lands where the value is queues nothing"
        );
        assert!(!progress.set_value(f32::NAN), "NaN is not a value");
        assert!(
            !progress.set_value(f32::INFINITY),
            "the range is a wall, not a door"
        );
        assert!(
            same(progress.current_value(), 30.0),
            "the refusals wrote nothing"
        );
    }

    #[test]
    fn the_bound_progress_reads_the_binding() {
        let binding = alacer_core::flux::RenderBinding::new(10.0_f32);
        let mut progress = Progress::bound("Batch", 0.0, 100.0, binding.clone());
        assert!(
            same(progress.current_value(), 10.0),
            "the binding is the source"
        );
        assert_eq!(Widget::value(&progress), Some("10".to_string()));

        binding.set(75.0);
        assert!(same(progress.current_value(), 75.0), "the read is live");

        // The imperative door is a no-op on a bound bar: the value belongs
        // to the reactive binding (the `Label::set_text` rule).
        assert!(!progress.set_value(42.0), "set_value is the fixed door");
        assert!(same(progress.current_value(), 75.0));
    }

    #[test]
    fn the_at_tier_is_refused_honestly() {
        let mut progress = Progress::new("Batch", 0.0, 100.0).value(42.0);
        for action in [
            A11yAction::SetValue("99".to_string()),
            A11yAction::Increment,
            A11yAction::Decrement,
            A11yAction::DefaultPress,
            A11yAction::ScrollUp(alacer_core::ScrollUnit::Item),
            A11yAction::ScrollDown(alacer_core::ScrollUnit::Page),
            A11yAction::ReplaceSelectedText("x".to_string()),
            A11yAction::SetTextSelection(alacer_core::TextSelection::new(0, 1)),
        ] {
            assert!(
                !progress.on_a11y_action(action),
                "a reporting widget has no write door — the M11 rule"
            );
        }
        assert!(
            same(progress.current_value(), 42.0),
            "the refusals wrote nothing"
        );
    }

    #[test]
    fn a_degenerate_range_is_a_panic() {
        let result = std::panic::catch_unwind(|| Progress::new("Broken", 10.0, 0.0));
        assert!(result.is_err(), "min >= max has no value to report");
        let result = std::panic::catch_unwind(|| Progress::new("Broken", f32::NAN, 1.0));
        assert!(result.is_err(), "a non-finite range has no value to report");
    }
}
