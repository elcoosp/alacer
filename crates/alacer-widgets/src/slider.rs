//! The slider: the value tier's widget (M20, ADR-0026).
//!
//! Four ways to write one bounded scalar, one funnel: a drag sets the value
//! from the pointer's x inside the track, the arrow keys step it, Home/End
//! jump to the range's ends, and the AccessKit value tier answers
//! `SetValue` (parsed, then snapped like every other write), `Increment`,
//! and `Decrement`. Every write clamps to the range, snaps to the step,
//! and requests the same masks — `PAINT | A11Y` — so the thumb repaints
//! and the a11y tree re-announces the value. The widget never writes a11y
//! directly (A1: masks are the only currency), and a write that lands where
//! the value already is queues no masks — the idle-frame property the
//! storm budget leans on.

use std::cmp::Ordering;

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, KeyEvent, PointerEvent};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, Widget};

/// A bounded scalar control with a mandatory name (ADR-5: an unnamed
/// slider reads as a bare number). The value rides the M3 `value()`
/// contract — the a11y tree carries it, and the AT writes it back through
/// the value tier.
pub struct Slider {
    name: String,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    on_change: Option<Box<dyn FnMut(f32)>>,
    focused: bool,
    dragging: bool,
    accent: Color,
    text_color: Color,
    font_size: f32,
    style: Style,
}

impl Slider {
    /// The track's paint height.
    const TRACK_HEIGHT: f32 = 8.0;
    /// The thumb's side (the value's hit target inside the track).
    const THUMB: f32 = 16.0;
    /// The gap between the track and the name.
    const GAP: f32 = 6.0;
    /// The track's default width.
    const WIDTH: f32 = 180.0;

    /// A slider labeled `name`, spanning `min..=max`, starting at `min`.
    ///
    /// # Panics
    ///
    /// Panics when `min` or `max` is not finite, when `min >= max` (a
    /// degenerate range has no value to carry), or when `step` is not
    /// finite and positive — the invariants every write relies on.
    #[must_use]
    pub fn new(name: impl Into<String>, min: f32, max: f32) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min < max,
            "the slider's range must be finite with min < max, got {min}..={max}"
        );
        Self {
            name: name.into(),
            min,
            max,
            step: 1.0,
            value: min,
            on_change: None,
            focused: false,
            dragging: false,
            accent: crate::theme::ACCENT,
            text_color: crate::theme::TEXT,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// The step every write snaps to (drag, arrows, AT acts alike).
    ///
    /// # Panics
    ///
    /// Panics when `step` is not finite and positive — the invariant the
    /// snap arithmetic relies on.
    #[must_use]
    pub fn step(mut self, step: f32) -> Self {
        assert!(
            step.is_finite() && step > 0.0,
            "the slider's step must be finite and positive, got {step}"
        );
        self.step = step;
        self
    }

    /// Initial value; clamped into the range and snapped to the step.
    #[must_use]
    pub fn value(mut self, value: f32) -> Self {
        self.value = Self::clamp_snap(value, self.min, self.max, self.step);
        self
    }

    /// Installs the change handler; invoked with the *new* value after
    /// every change (drag, arrows, a11y act) — never for a write that
    /// landed where the value already was.
    #[must_use]
    pub fn on_change(mut self, handler: impl FnMut(f32) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Accent color of the filled track and the thumb.
    #[must_use]
    pub fn accent(mut self, accent: Color) -> Self {
        self.accent = accent;
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

    /// Current value (test introspection).
    #[must_use]
    pub const fn current_value(&self) -> f32 {
        self.value
    }

    /// Sets the value in place (view reconciliation): clamped and snapped
    /// like every write, fires no handler — programmatic state, not a
    /// user act. A non-finite request is ignored.
    pub fn set_value(&mut self, value: f32) {
        if value.is_finite() {
            self.value = Self::clamp_snap(value, self.min, self.max, self.step);
        }
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Replaces the change handler in place (view reconciliation). `None`
    /// detaches it.
    pub fn set_on_change(&mut self, handler: Option<Box<dyn FnMut(f32)>>) {
        self.on_change = handler;
    }

    /// The one float equality the widget owns: `total_cmp` ordering —
    /// `f32::eq` is a lint trap the whole workspace refuses.
    fn same_value(a: f32, b: f32) -> bool {
        a.total_cmp(&b) == Ordering::Equal
    }

    /// The one write funnel: clamp into the range, snap to the step,
    /// clamp again (the snap can land a hair past `max`). Every door —
    /// drag, keys, the AT — reaches the value through here.
    fn clamp_snap(value: f32, min: f32, max: f32, step: f32) -> f32 {
        let clamped = value.clamp(min, max);
        let steps = ((clamped - min) / step).round();
        (min + steps * step).clamp(min, max)
    }

    /// The value's position in `0.0..=1.0` along the track.
    fn ratio(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }

    /// The value under a pointer x, given the widget-local track geometry:
    /// the thumb's center travels the track minus one thumb width.
    fn value_at(local_x: f32, width: f32, min: f32, max: f32) -> f32 {
        let travel = width - Self::THUMB;
        if travel <= 0.0 {
            return min;
        }
        let ratio = ((local_x - Self::THUMB * 0.5) / travel).clamp(0.0, 1.0);
        min + ratio * (max - min)
    }

    /// The one commit path: the funnel, then — when the value moved — the
    /// masks and the handler. The answer is `true` whenever the request
    /// was answerable (the write landed where the value already is is an
    /// answered request, not a refusal); `false` refuses a request the
    /// widget cannot carry (a non-finite value) — never munged.
    fn commit(&mut self, requested: f32, ctx: Option<&mut EventCtx>) -> bool {
        if !requested.is_finite() {
            return false;
        }
        let value = Self::clamp_snap(requested, self.min, self.max, self.step);
        if !Self::same_value(value, self.value) {
            self.value = value;
            if let Some(ctx) = ctx {
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
            }
            if let Some(handler) = &mut self.on_change {
                handler(value);
            }
        }
        true
    }

    /// The value under a window-space pointer x, resolved against the
    /// node's frame (the router hands it to every event visit).
    fn commit_at(&mut self, x: f32, bounds: Rect, ctx: &mut EventCtx) -> bool {
        self.commit(
            Self::value_at(x - bounds.min_x(), bounds.width(), self.min, self.max),
            Some(ctx),
        )
    }

    /// Steps by `direction` (the arrows' and the AT step tier's shared
    /// route).
    fn step_by(&mut self, direction: f32, ctx: &mut EventCtx) -> bool {
        self.commit(self.value + direction * self.step, Some(ctx))
    }

    /// Intrinsic size: the thumb's row (the track rides inside it), the
    /// gap, and the name.
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
        Size::new(Self::WIDTH, Self::THUMB + Self::GAP + text.height)
    }
}

impl Widget for Slider {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Slider
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

    /// The M3 value contract: the AT reads the bounded scalar, not just
    /// the name.
    fn value(&self) -> Option<String> {
        Some(format!("{}", self.value))
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let width = ctx.size().width;
        let ratio = self.ratio();
        let track_y = (Self::THUMB - Self::TRACK_HEIGHT) * 0.5;

        // The track: a rounded rail the full width.
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(
                Point::new(0.0, track_y),
                Size::new(width, Self::TRACK_HEIGHT),
            ),
            color: crate::theme::BORDER,
            corner_radius: Self::TRACK_HEIGHT * 0.5,
            border_color: crate::theme::BORDER,
            border_width: 0.0,
            clip: None,
        });

        // The fill: from the track's start to the thumb's center.
        let fill_width = (width - Self::THUMB).max(0.0) * ratio + Self::THUMB * 0.5;
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(
                Point::new(0.0, track_y),
                Size::new(fill_width, Self::TRACK_HEIGHT),
            ),
            color: self.accent,
            corner_radius: Self::TRACK_HEIGHT * 0.5,
            border_color: self.accent,
            border_width: 0.0,
            clip: None,
        });

        // The thumb: a centered knob riding the travel.
        let travel = (width - Self::THUMB).max(0.0);
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(
                Point::new(ratio * travel, 0.0),
                Size::new(Self::THUMB, Self::THUMB),
            ),
            color: self.accent,
            corner_radius: Self::THUMB * 0.5,
            border_color: if self.focused {
                crate::theme::ACCENT_PRESSED
            } else {
                crate::theme::BORDER
            },
            border_width: if self.focused { 2.0 } else { 1.5 },
            clip: None,
        });

        // The name, under the track.
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.name, self.font_size, None);
            let text_origin = Point::new(0.0, Self::THUMB + Self::GAP);
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

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// The value tier (M20, ADR-0026): `SetValue` parses then rides the
    /// one funnel, `Increment`/`Decrement` step by the step — the same
    /// masks as a drag. `DefaultPress` is not a value control's contract —
    /// refused, never munged into a write (the M11 rule).
    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        match action {
            A11yAction::SetValue(value) => match value.trim().parse::<f32>() {
                Ok(parsed) => self.commit(parsed, None),
                Err(_) => false,
            },
            A11yAction::Increment | A11yAction::Decrement => {
                let direction = if action == A11yAction::Increment {
                    1.0
                } else {
                    -1.0
                };
                self.commit(self.value + direction * self.step, None)
            }
            A11yAction::DefaultPress
            | A11yAction::ScrollUp(_)
            | A11yAction::ScrollDown(_)
            | A11yAction::ReplaceSelectedText(_)
            | A11yAction::SetTextSelection(_) => false,
        }
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        match event {
            PointerEvent::Down { position, .. } => {
                self.dragging = true;
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::FOCUS));
                if let Some(bounds) = ctx.bounds() {
                    let _ = self.commit_at(position.x, bounds, ctx);
                }
                EventResponse::Handled
            }
            PointerEvent::Move { position } => {
                if !self.dragging {
                    return EventResponse::Ignored;
                }
                match ctx.bounds() {
                    Some(bounds) => {
                        let _ = self.commit_at(position.x, bounds, ctx);
                        EventResponse::Handled
                    }
                    None => EventResponse::Ignored,
                }
            }
            PointerEvent::Up { .. } => {
                self.dragging = false;
                EventResponse::Handled
            }
            // The leave ends the drag honestly (the capture released); the
            // wheel is not a value gesture.
            PointerEvent::Leave => {
                self.dragging = false;
                EventResponse::Ignored
            }
            PointerEvent::Wheel { .. } => EventResponse::Ignored,
        }
    }

    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        if event.state != alacer_core::events::KeyState::Pressed {
            return EventResponse::Ignored;
        }
        let response = match event.key {
            alacer_core::events::Key::ArrowLeft | alacer_core::events::Key::ArrowDown => {
                self.step_by(-1.0, ctx)
            }
            alacer_core::events::Key::ArrowRight | alacer_core::events::Key::ArrowUp => {
                self.step_by(1.0, ctx)
            }
            alacer_core::events::Key::Home => self.commit(self.min, Some(ctx)),
            alacer_core::events::Key::End => self.commit(self.max, Some(ctx)),
            _ => return EventResponse::Ignored,
        };
        if response {
            EventResponse::Handled
        } else {
            EventResponse::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Slider;
    use alacer_core::events::{
        EventCtx, Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent,
    };
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::mask::Change;
    use alacer_core::widget::{A11yAction, EventResponse, Role, Widget};

    /// The float equality the tests own (the widget's own rule).
    fn same(a: f32, b: f32) -> bool {
        a.total_cmp(&b) == std::cmp::Ordering::Equal
    }

    fn arrow(key: Key) -> KeyEvent {
        KeyEvent {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    /// The event context the pass-1 router builds: phase plus the node's
    /// resolved frame — the widget-local math needs the frame.
    fn ctx_at(bounds: Rect) -> EventCtx {
        EventCtx::with_phase_and_bounds(alacer_core::events::EventPhase::Bubble, Some(bounds))
    }

    fn move_to(x: f32) -> PointerEvent {
        PointerEvent::Move {
            position: Point::new(x, 8.0),
        }
    }

    #[test]
    fn role_name_and_value_satisfy_the_contract() {
        let slider = Slider::new("Volume", 0.0, 100.0);
        assert_eq!(slider.role(), Role::Slider);
        assert_eq!(slider.label().resolve(), "Volume");
        assert!(slider.focusable(), "sliders hold focus");
        assert!(slider.role().requires_name());
        assert_eq!(Widget::value(&slider), Some("0".to_string()));
        assert_eq!(slider.table_slot(), None);
    }

    #[test]
    fn drag_sets_the_value_from_the_pointer_x() {
        let mut slider = Slider::new("Volume", 0.0, 100.0);
        // The widget's frame at (100, 100), 180 wide: the track travel is
        // 164 px of thumb math inside it.
        let bounds = Rect::new(Point::new(100.0, 100.0), Size::new(180.0, 30.0));
        let down = PointerEvent::Down {
            position: Point::new(190.0, 108.0),
            button: MouseButton::Left,
            click_count: 1,
        };
        let mut ctx = ctx_at(bounds);
        assert_eq!(
            slider.on_pointer_event(&mut ctx, &down),
            EventResponse::Handled
        );
        // The press lands mid-track: 90 local x, minus half a thumb, over
        // 164 travel = exactly 0.5 → the step snaps nothing away from 50.
        assert!(
            same(slider.current_value(), 50.0),
            "mid-track press = 50, got {}",
            slider.current_value()
        );
        let masks = ctx.take();
        assert!(
            masks.contains(Change::PAINT) && masks.contains(Change::A11Y),
            "the drag repaints and re-announces, got {masks:?}"
        );

        // The drag past the right edge clamps at the max.
        let mut ctx = ctx_at(bounds);
        let move_far = move_to(400.0);
        assert_eq!(
            slider.on_pointer_event(&mut ctx, &move_far),
            EventResponse::Handled
        );
        assert!(same(slider.current_value(), 100.0), "the range is a wall");

        // The release ends the gesture; the next move is a hover.
        let up = PointerEvent::Up {
            position: Point::new(400.0, 108.0),
            button: MouseButton::Left,
        };
        let mut ctx = ctx_at(bounds);
        assert_eq!(
            slider.on_pointer_event(&mut ctx, &up),
            EventResponse::Handled
        );
        let mut ctx = ctx_at(bounds);
        let hover = move_to(120.0);
        assert_eq!(
            slider.on_pointer_event(&mut ctx, &hover),
            EventResponse::Ignored,
            "a move without a press is the hover, never a write"
        );
        assert!(same(slider.current_value(), 100.0));
    }

    #[test]
    fn a_no_op_drag_move_queues_no_masks() {
        let mut slider = Slider::new("Volume", 0.0, 100.0).value(50.0);
        let bounds = Rect::new(Point::new(100.0, 100.0), Size::new(180.0, 30.0));
        let mut ctx = ctx_at(bounds);
        // The exact x the current value already rides: half the travel
        // plus half a thumb.
        let x = 100.0 + 164.0 * 0.5 + 8.0;
        assert!(slider.commit_at(x, bounds, &mut ctx));
        assert_eq!(
            ctx.take(),
            Change::empty(),
            "a write that lands where the value is queues no masks"
        );
    }

    #[test]
    fn arrows_step_and_home_end_jump() {
        let mut slider = Slider::new("Volume", 0.0, 100.0).value(50.0);
        let mut ctx = EventCtx::new();
        assert_eq!(
            slider.on_key_event(&mut ctx, &arrow(Key::ArrowRight)),
            EventResponse::Handled
        );
        assert!(same(slider.current_value(), 51.0));
        assert!(ctx.take().contains(Change::A11Y));

        let mut ctx = EventCtx::new();
        let _ = slider.on_key_event(&mut ctx, &arrow(Key::ArrowLeft));
        let _ = slider.on_key_event(&mut ctx, &arrow(Key::ArrowDown));
        assert!(same(slider.current_value(), 49.0), "down mirrors left");

        let mut ctx = EventCtx::new();
        let _ = slider.on_key_event(&mut ctx, &arrow(Key::End));
        assert!(same(slider.current_value(), 100.0));
        let mut ctx = EventCtx::new();
        let _ = slider.on_key_event(&mut ctx, &arrow(Key::Home));
        assert!(same(slider.current_value(), 0.0));

        let mut ctx = EventCtx::new();
        assert_eq!(
            slider.on_key_event(&mut ctx, &arrow(Key::Enter)),
            EventResponse::Ignored,
            "a key with no value meaning is the hover's answer"
        );
    }

    #[test]
    fn the_at_value_tier_writes_and_refuses_honestly() {
        let seen = std::rc::Rc::new(std::cell::Cell::new(0.0_f32));
        let sink = std::rc::Rc::clone(&seen);
        let mut slider = Slider::new("Volume", 0.0, 100.0).on_change(move |v| sink.set(v));

        assert!(
            slider.on_a11y_action(A11yAction::SetValue("42".to_string())),
            "an in-range numeric SetValue is answered"
        );
        assert!(same(slider.current_value(), 42.0));
        assert!(same(seen.get(), 42.0), "the handler observed the write");
        assert_eq!(Widget::value(&slider), Some("42".to_string()));

        assert!(
            !slider.on_a11y_action(A11yAction::SetValue("loud".to_string())),
            "a non-numeric SetValue is refused, never munged into a step"
        );
        assert!(same(slider.current_value(), 42.0));
        assert!(
            !slider.on_a11y_action(A11yAction::SetValue("inf".to_string())),
            "a non-finite value is not a value the widget can carry"
        );

        assert!(slider.on_a11y_action(A11yAction::Increment));
        assert!(same(slider.current_value(), 43.0));
        assert!(slider.on_a11y_action(A11yAction::Decrement));
        assert!(same(slider.current_value(), 42.0));

        assert!(
            !slider.on_a11y_action(A11yAction::DefaultPress),
            "a value control has no press action — the M11 refusal rule"
        );
    }

    #[test]
    fn every_write_clamps_and_snaps_to_the_step() {
        let mut slider = Slider::new("Volume", 0.0, 10.0).step(2.5);
        assert!(slider.on_a11y_action(A11yAction::SetValue("3.1".to_string())));
        assert!(
            same(slider.current_value(), 2.5),
            "the AT's write snaps like a drag, got {}",
            slider.current_value()
        );
        assert!(slider.on_a11y_action(A11yAction::SetValue("99".to_string())));
        assert!(same(slider.current_value(), 10.0), "the range is a wall");
    }

    #[test]
    fn programmatic_set_value_fires_no_handler() {
        let mut slider = Slider::new("Volume", 0.0, 100.0).on_change(|_| panic!("no user act"));
        slider.set_value(30.0);
        assert!(same(slider.current_value(), 30.0));
        slider.set_value(f32::NAN);
        assert!(same(slider.current_value(), 30.0), "NaN is ignored");
    }

    #[test]
    fn a_degenerate_range_or_step_is_a_panic() {
        let result = std::panic::catch_unwind(|| Slider::new("Broken", 10.0, 0.0));
        assert!(result.is_err(), "min >= max has no value to carry");
        let result = std::panic::catch_unwind(|| Slider::new("Broken", 0.0, 10.0).step(0.0));
        assert!(result.is_err(), "a zero step divides by zero");
    }
}
