//! The meter: the gauge tier's read half (M133, ADR-0137).
//!
//! A scalar *measurement* on a scale the app declares, and the AT reads
//! as a gauge — AccessKit's `Meter` role, ARIA's `role="meter"`. Not a
//! progress bar wearing the wrong word: a progress bar's value is
//! *task progress*, how far some task has come toward done; a meter's
//! value is *how much of something there is* — a disk's usage, a
//! battery's charge, a budget's headroom. The word prices the word and
//! the reading, and nothing else: the value rides the value tier's own
//! vocabulary (the M3 `value()` spoken word, the M97 range word, the
//! M99 numeric word — never a second value language), and the write
//! vocabulary is the app's, with two doors that land in the same
//! `PAINT | A11Y` currency a [`Progress`](crate::Progress) report
//! takes: a [`RenderBinding`] the flux bridge drives, and `set_value`
//! for the mutate seam (the answer is the `bool` that says whether the
//! reading moved, so a write that lands where the reading already is
//! queues nothing — the M21 idle-frame property, at the meter tier).
//!
//! The AT tier is read-only here by construction: a measurement has no
//! write door, and every action is refused honestly — never munged
//! into a step or a press (the M11 rule; the M21 progress rule, at the
//! meter tier). The gauge holds no focus and answers no activation
//! (the M126 heading's own rule at the value tier's read half). The
//! widget never writes a11y directly (A1: masks are the only
//! currency).

use std::cmp::Ordering;

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, Widget};

/// A read-only gauge with a mandatory name (ADR-5: an unnamed meter
/// reads as a bare number). The reading rides the M3 `value()`
/// contract — the a11y tree carries it, and the app writes it through
/// the bridge or the mutate seam, never the AT.
pub struct Meter {
    name: String,
    min: f32,
    max: f32,
    source: MeterValue,
    /// The app's fill override; `None` borrows the theme's accent role
    /// at paint time (ADR-0079).
    fill: Option<Color>,
    /// The app's text-color override; `None` borrows the theme's text
    /// role at paint time (ADR-0079).
    text_color: Option<Color>,
    font_size: f32,
    style: Style,
}

/// Where the reported reading comes from: the app's imperative write
/// (`set_value`) or the reactive binding the flux bridge drives.
enum MeterValue {
    Fixed(f32),
    Binding(RenderBinding<f32>),
}

impl Meter {
    /// The gauge's paint height.
    const BAR_HEIGHT: f32 = 10.0;
    /// The gap between the gauge and the name.
    const GAP: f32 = 6.0;
    /// The gauge's default width.
    const WIDTH: f32 = 180.0;
    /// The reading tick's width — the hairline the gauge marks the
    /// measurement with (a meter names *where on the scale* the
    /// reading sits, the tick the scale's own mark).
    const TICK_WIDTH: f32 = 2.0;

    /// A gauge labeled `name`, spanning `min..=max`, reporting `min`
    /// until the app writes.
    ///
    /// # Panics
    ///
    /// Panics when `min` or `max` is not finite, or when `min >= max`
    /// (a degenerate scale has no reading to report) — the invariants
    /// every read relies on.
    #[must_use]
    pub fn new(name: impl Into<String>, min: f32, max: f32) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min < max,
            "the meter's scale must be finite with min < max, got {min}..={max}"
        );
        Self {
            name: name.into(),
            min,
            max,
            source: MeterValue::Fixed(min),
            fill: None,
            text_color: None,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// A gauge whose reading tracks a [`RenderBinding`] — the app
    /// writes a signal, the flux bridge queues the caller's masks, and
    /// the widget paints and reports from the binding (§6.1; the
    /// reactive-text seam, at a value). The report is the binding's
    /// number, read live — the `Label::bound` rule: the app's voice,
    /// not the widget's rewrite — while the paint clamps into the
    /// scale.
    ///
    /// # Panics
    ///
    /// Panics under the same scale invariants as [`Meter::new`].
    #[must_use]
    pub fn bound(name: impl Into<String>, min: f32, max: f32, binding: RenderBinding<f32>) -> Self {
        let mut meter = Self::new(name, min, max);
        meter.source = MeterValue::Binding(binding);
        meter
    }

    /// Initial reported reading; clamped into the scale. A non-finite
    /// request is ignored — not a value the widget can carry.
    #[must_use]
    pub fn value(mut self, value: f32) -> Self {
        if value.is_finite()
            && let MeterValue::Fixed(reported) = &mut self.source
        {
            *reported = Self::clamp(value, self.min, self.max);
        }
        self
    }

    /// The fill's color (the reported span of the scale). Without it
    /// the gauge borrows the theme's accent role at paint time
    /// (ADR-0079).
    #[must_use]
    pub fn fill(mut self, fill: Color) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Text color of the label. Without it the label borrows the
    /// theme's text role at paint time (ADR-0079).
    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Layout styles.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// The reported reading (test introspection — the same number the
    /// M3 `value()` contract carries into the a11y tree).
    #[must_use]
    pub fn current_value(&self) -> f32 {
        self.reported()
    }

    /// The one write funnel: clamp into the scale, report whether the
    /// reading moved. The caller requests the write's own masks —
    /// `PAINT | A11Y` — only when this answers `true`, so a write that
    /// lands where the reading already is queues nothing. A non-finite
    /// request is refused honestly (`false`, never munged). A no-op on
    /// a binding-backed gauge: its reading belongs to the reactive
    /// binding (the `Label::set_text` rule).
    pub fn set_value(&mut self, value: f32) -> bool {
        let MeterValue::Fixed(reported) = &mut self.source else {
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

    /// The clamp the funnel and the builder share (the scale is a
    /// wall).
    fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.clamp(min, max)
    }

    /// The value the widget reports, from wherever the app writes it.
    fn reported(&self) -> f32 {
        match &self.source {
            MeterValue::Fixed(value) => *value,
            MeterValue::Binding(binding) => binding.with(|value| *value),
        }
    }

    /// The reported reading's position in `0.0..=1.0` along the scale.
    /// The paint is bounded (the gauge is a bounded control's body)
    /// even when a binding carries a number past the scale — the
    /// report is the app's voice, the paint is the widget's.
    fn ratio(&self) -> f32 {
        let value = self.reported();
        ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// Intrinsic size: the gauge, the gap, and the name.
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

impl Widget for Meter {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Meter
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

    /// The M3 value contract: the AT reads the reported reading, not
    /// just the name.
    fn value(&self) -> Option<String> {
        Some(self.reported().to_string())
    }

    /// The range tier (M97, ADR-0101): the scale the app declared —
    /// the clamp every report rides — is the span the tree announces.
    /// The step is the honest absence: a measurement declares where
    /// its reading lives, not how far one act moves it (there is no
    /// stepping door on a read-only gauge), and the tier never guesses
    /// a default. The widening `f32` → `f64` is exact.
    fn value_range(&self) -> Option<alacer_core::ValueRange> {
        Some(alacer_core::ValueRange::new(
            f64::from(self.min),
            f64::from(self.max),
        ))
    }

    /// The numeric tier (M99, ADR-0103): the number behind the
    /// reading's spoken word — the read half of the value tier answers
    /// the `aria-valuenow` beside the `aria-valuetext` the M3 contract
    /// already carried. The same widening `f32` → `f64` is exact.
    fn numeric_value(&self) -> Option<f64> {
        Some(f64::from(self.reported()))
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let width = ctx.size().width;
        let ratio = self.ratio();
        // The borrowed roles resolve per paint pass (ADR-0079).
        let palette = ctx.palette();
        let fill = self.fill.unwrap_or(palette.accent);
        let text_color = self.text_color.unwrap_or(palette.text);

        // The scale: a rounded rail the full width.
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(width, Self::BAR_HEIGHT)),
            color: palette.border,
            corner_radius: Self::BAR_HEIGHT * 0.5,
            border_color: palette.border,
            border_width: 0.0,
            clip: None,
        });

        // The fill: the reported span of the scale.
        let fill_width = (width * ratio).max(0.0);
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(
                Point::new(0.0, 0.0),
                Size::new(fill_width, Self::BAR_HEIGHT),
            ),
            color: fill,
            corner_radius: Self::BAR_HEIGHT * 0.5,
            border_color: fill,
            border_width: 0.0,
            clip: None,
        });

        // The reading tick: the hairline the gauge marks the
        // measurement with, at the reading's position on the scale.
        let tick_x = (width * ratio - Self::TICK_WIDTH).max(0.0);
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(
                Point::new(tick_x, 0.0),
                Size::new(Self::TICK_WIDTH, Self::BAR_HEIGHT),
            ),
            color: text_color,
            corner_radius: 0.0,
            border_color: text_color,
            border_width: 0.0,
            clip: None,
        });

        // The name, under the gauge.
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.name, self.font_size, None);
            let text_origin = Point::new(0.0, Self::BAR_HEIGHT + Self::GAP);
            for glyph in &layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(text_origin.x + glyph.x, text_origin.y + glyph.y),
                    size: glyph.size,
                    color: text_color,
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
        // A reading is not a control: the pointer is the hover's
        // passenger, never a write.
        EventResponse::Ignored
    }

    /// The AT tier is read-only (M133, ADR-0137): a measurement has no
    /// write door — every action is refused honestly, never munged
    /// into a step or a press (the M11 rule; the M21 progress rule, at
    /// the meter tier). The tree keeps the reading the app wrote.
    /// Since M40 (ADR-0046) the value-tier refusals stream as
    /// [`alacer_core::inspect::ValueRecord`]s — the `no_write_door`
    /// species the `bool` collapsed into one silence, the read-only
    /// tier's honesty visible on the log; a question outside the value
    /// tier (a press, a scroll, a text edit) is not a value fact and
    /// stays silent. Since M44 (ADR-0050) the refusal carries the
    /// ask's honest absence: no ask ever entered a door the widget
    /// does not have.
    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        let tier = match action {
            A11yAction::SetValue(_) => alacer_core::ValueAction::SetValue,
            A11yAction::Increment => alacer_core::ValueAction::Increment,
            A11yAction::Decrement => alacer_core::ValueAction::Decrement,
            A11yAction::DefaultPress
            | A11yAction::ScrollUp(_)
            | A11yAction::ScrollDown(_)
            | A11yAction::ScrollLeft(_)
            | A11yAction::ScrollRight(_)
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
    use super::Meter;
    use alacer_core::widget::{A11yAction, Role, Widget};

    /// The float equality the tests own (the widget's own rule).
    fn same(a: f32, b: f32) -> bool {
        a.total_cmp(&b) == std::cmp::Ordering::Equal
    }

    #[test]
    fn role_name_and_reading_satisfy_the_contract() {
        let meter = Meter::new("Disk usage", 0.0, 100.0);
        assert_eq!(meter.role(), Role::Meter);
        assert_eq!(meter.role().name(), "meter");
        assert_eq!(meter.label().resolve(), "Disk usage");
        assert!(meter.role().requires_name());
        assert!(!meter.focusable(), "a reading holds no focus");
        assert_eq!(Widget::value(&meter), Some("0".to_string()));
        assert_eq!(meter.toggled(), None);
        assert_eq!(meter.table_slot(), None);
        assert_eq!(meter.selection(), None);
        assert!(!meter.multiline(), "a gauge carries no capacity word");
        assert_eq!(meter.language(), None);
    }

    #[test]
    fn set_value_clamps_and_reports_the_move() {
        let mut meter = Meter::new("Disk usage", 0.0, 100.0);
        assert!(meter.set_value(62.0), "the write moved the reading");
        assert!(same(meter.current_value(), 62.0));
        assert_eq!(Widget::value(&meter), Some("62".to_string()));

        assert!(meter.set_value(1000.0), "the wall answers a move");
        assert!(same(meter.current_value(), 100.0), "the scale is a wall");
        assert!(meter.set_value(-5.0), "the wall answers a move");
        assert!(same(meter.current_value(), 0.0), "the scale is a wall");
    }

    #[test]
    fn a_no_op_or_non_finite_write_is_no_move() {
        let mut meter = Meter::new("Disk usage", 0.0, 100.0).value(30.0);
        assert!(
            !meter.set_value(30.0),
            "a write that lands where the reading is queues nothing"
        );
        assert!(!meter.set_value(f32::NAN), "NaN is not a reading");
        assert!(
            !meter.set_value(f32::INFINITY),
            "the scale is a wall, not a door"
        );
        assert!(
            same(meter.current_value(), 30.0),
            "the refusals wrote nothing"
        );
    }

    #[test]
    fn the_bound_gauge_reads_the_binding() {
        let binding = alacer_core::flux::RenderBinding::new(10.0_f32);
        let mut meter = Meter::bound("Disk usage", 0.0, 100.0, binding.clone());
        assert!(
            same(meter.current_value(), 10.0),
            "the binding is the source"
        );
        assert_eq!(Widget::value(&meter), Some("10".to_string()));

        binding.set(75.0);
        assert!(same(meter.current_value(), 75.0), "the read is live");

        // The imperative door is a no-op on a bound gauge: the reading
        // belongs to the reactive binding (the `Label::set_text` rule).
        assert!(!meter.set_value(62.0), "set_value is the fixed door");
        assert!(same(meter.current_value(), 75.0));
    }

    #[test]
    fn the_at_tier_is_refused_honestly() {
        let mut meter = Meter::new("Disk usage", 0.0, 100.0).value(62.0);
        for action in [
            A11yAction::SetValue("42".to_string()),
            A11yAction::Increment,
            A11yAction::Decrement,
            A11yAction::DefaultPress,
            A11yAction::ScrollUp(alacer_core::ScrollUnit::Item),
            A11yAction::ScrollDown(alacer_core::ScrollUnit::Page),
            A11yAction::ReplaceSelectedText("x".to_string()),
            A11yAction::SetTextSelection(alacer_core::TextSelection::new(0, 1)),
        ] {
            assert!(
                !meter.on_a11y_action(action),
                "a measurement has no write door — the M11 rule"
            );
        }
        assert!(
            same(meter.current_value(), 62.0),
            "the refusals wrote nothing"
        );
    }

    #[test]
    fn a_degenerate_scale_is_a_panic() {
        let result = std::panic::catch_unwind(|| Meter::new("Broken", 10.0, 0.0));
        assert!(result.is_err(), "min >= max has no reading to report");
        let result = std::panic::catch_unwind(|| Meter::new("Broken", f32::NAN, 1.0));
        assert!(
            result.is_err(),
            "a non-finite scale has no reading to report"
        );
    }
}
