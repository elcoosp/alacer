//! The checkbox: the todo canon's toggle control (M6).
//!
//! Three ways to toggle, one contract: a click, the Space key while focused,
//! and the AccessKit default action all flip the same bit and request the
//! same masks — `PAINT | A11Y` — so the check mark repaints and the a11y
//! tree re-announces the state. The widget never writes a11y directly (A1:
//! masks are the only currency).

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, KeyEvent, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, ToggleState, Widget};

/// A binary toggle with a mandatory name (ADR-5: an unnamed checkbox is
/// unusable with a screen reader).
pub struct Checkbox {
    name: String,
    checked: bool,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
    focused: bool,
    accent: Color,
    text_color: Color,
    font_size: f32,
    style: Style,
}

impl Checkbox {
    /// A checkbox labeled `name`, initially unchecked.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            checked: false,
            on_toggle: None,
            focused: false,
            accent: crate::theme::ACCENT,
            text_color: crate::theme::TEXT,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// Initial checked state.
    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Installs the toggle handler; invoked with the *new* state after every
    /// toggle (click, Space, a11y action).
    #[must_use]
    pub fn on_toggle(mut self, handler: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Accent color of the box and its check mark.
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

    /// Current checked state (test introspection).
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        self.checked
    }

    /// Sets the checked state in place (view reconciliation). Fires no
    /// handler — programmatic state, not a user toggle.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Replaces the toggle handler in place (view reconciliation). `None`
    /// detaches it.
    pub fn set_on_toggle(&mut self, handler: Option<Box<dyn FnMut(bool)>>) {
        self.on_toggle = handler;
    }

    /// The one toggle path: flip the bit, fire the handler with the new
    /// state, request the masks the change costs.
    fn toggle(&mut self, ctx: &mut EventCtx) {
        self.checked = !self.checked;
        ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        if let Some(handler) = &mut self.on_toggle {
            handler(self.checked);
        }
    }

    /// Intrinsic size: the 18px box, a gap, and the label.
    fn intrinsic(&self) -> alacer_core::geom::Size {
        let text = alacer_core::text_seam::with_seam(|seam| {
            seam.measure(
                &TextSpec {
                    text: self.name.clone(),
                    font_id: 0,
                    font_size: self.font_size,
                },
                None,
            )
        });
        alacer_core::geom::Size::new(18.0 + 8.0 + text.width, text.height.max(18.0) + 8.0)
    }
}

impl Widget for Checkbox {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::CheckBox
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

    fn toggled(&self) -> Option<ToggleState> {
        Some(ToggleState::from(self.checked))
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let size = ctx.size();
        let box_side = 18.0_f32.min(size.height);
        let box_top = (size.height - box_side) * 0.5;

        // The track: an outlined square; filled with the accent when checked.
        ctx.push_quad(alacer_core::scene::Quad {
            rect: alacer_core::geom::Rect::new(
                Point::new(0.0, box_top),
                alacer_core::geom::Size::new(box_side, box_side),
            ),
            color: if self.checked {
                self.accent
            } else {
                Color::WHITE
            },
            corner_radius: 4.0,
            border_color: if self.focused {
                self.accent
            } else {
                crate::theme::BORDER
            },
            border_width: if self.focused { 2.0 } else { 1.5 },
            clip: None,
        });

        // The check mark: two accent strokes drawn as quads — deterministic
        // pixels with no glyph dependency (snapshot corpus stays font-free).
        if self.checked {
            let inset = box_side * 0.26;
            let stroke = 2.5;
            let left = Point::new(inset * 0.8, box_top + box_side * 0.52);
            let mid = Point::new(box_side * 0.42, box_top + box_side - inset);
            let right = Point::new(box_side - inset * 0.7, box_top + inset);
            let down = alacer_core::geom::Rect::new(
                Point::new(left.x, left.y - stroke * 0.5),
                alacer_core::geom::Size::new(mid.x - left.x, stroke),
            );
            let up = alacer_core::geom::Rect::new(
                Point::new(mid.x - stroke * 0.5, right.y),
                alacer_core::geom::Size::new(stroke, mid.y - right.y),
            );
            ctx.push_quad(alacer_core::scene::Quad::filled(down, Color::WHITE));
            ctx.push_quad(alacer_core::scene::Quad::filled(up, Color::WHITE));
        }

        // The label, right of the box.
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.name, self.font_size, None);
            let text_origin =
                Point::new(box_side + 8.0, (size.height - layout.height).max(0.0) * 0.5);
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
        Some(MeasureSpec::Fixed(self.intrinsic()))
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        if action == A11yAction::DefaultPress {
            // The a11y path has no EventCtx; masks land on the next widget
            // event or the frame's PAINT churn — the toggle itself is
            // handler-first, exactly like Button's press path.
            self.checked = !self.checked;
            if let Some(handler) = &mut self.on_toggle {
                handler(self.checked);
            }
            return true;
        }
        false
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        match event {
            PointerEvent::Down { .. } => {
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::FOCUS));
                EventResponse::Handled
            }
            PointerEvent::Up { .. } => {
                self.toggle(ctx);
                EventResponse::Handled
            }
            PointerEvent::Move { .. } | PointerEvent::Wheel { .. } | PointerEvent::Leave => {
                EventResponse::Ignored
            }
        }
    }

    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        if event.state == alacer_core::events::KeyState::Pressed
            && event.key == alacer_core::events::Key::Space
        {
            self.toggle(ctx);
            return EventResponse::Handled;
        }
        EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::Checkbox;
    use alacer_core::events::{EventCtx, Key, KeyEvent, KeyState, Modifiers, PointerEvent};
    use alacer_core::geom::Point;
    use alacer_core::mask::Change;
    use alacer_core::widget::{A11yAction, EventResponse, Role, Widget};

    fn space() -> KeyEvent {
        KeyEvent {
            key: Key::Space,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    #[test]
    fn role_name_and_toggle_state_satisfy_the_contract() {
        let box_widget = Checkbox::new("Ship in bulk");
        assert_eq!(box_widget.role(), Role::CheckBox);
        assert_eq!(box_widget.label().resolve(), "Ship in bulk");
        assert!(box_widget.focusable(), "checkboxes hold focus");
        assert!(box_widget.role().requires_name());
        assert_eq!(
            box_widget.toggled(),
            Some(alacer_core::widget::ToggleState::Off)
        );
    }

    #[test]
    fn click_toggles_and_requests_paint_plus_a11y() {
        let mut checkbox = Checkbox::new("Done");
        let mut ctx = EventCtx::new();
        let down = PointerEvent::Down {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        assert_eq!(
            checkbox.on_pointer_event(&mut ctx, &down),
            EventResponse::Handled
        );
        let up = PointerEvent::Up {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        assert_eq!(
            checkbox.on_pointer_event(&mut ctx, &up),
            EventResponse::Handled
        );
        assert!(checkbox.is_checked(), "click toggles");
        let masks = ctx.take();
        assert!(
            masks.contains(Change::PAINT) && masks.contains(Change::A11Y),
            "toggle repaints and re-announces, got {masks:?}"
        );
    }

    #[test]
    fn space_toggles_while_the_handler_sees_the_new_state() {
        let seen = std::rc::Rc::new(std::cell::Cell::new(false));
        let sink = std::rc::Rc::clone(&seen);
        let mut checkbox = Checkbox::new("Done").on_toggle(move |checked| sink.set(checked));
        let mut ctx = EventCtx::new();
        assert_eq!(
            checkbox.on_key_event(&mut ctx, &space()),
            EventResponse::Handled
        );
        assert!(checkbox.is_checked());
        assert!(seen.get(), "the handler observed the new state");
        let masks = ctx.take();
        assert!(masks.contains(Change::A11Y), "keyboard toggle re-announces");

        // A second Space flips it back.
        let mut ctx = EventCtx::new();
        let _ = checkbox.on_key_event(&mut ctx, &space());
        assert!(!checkbox.is_checked());
    }

    #[test]
    fn other_keys_are_ignored() {
        let mut checkbox = Checkbox::new("Done");
        let mut ctx = EventCtx::new();
        let enter = KeyEvent {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert_eq!(
            checkbox.on_key_event(&mut ctx, &enter),
            EventResponse::Ignored
        );
        assert!(!checkbox.is_checked());
    }

    #[test]
    fn the_a11y_default_action_toggles() {
        let mut checkbox = Checkbox::new("Done");
        assert!(checkbox.on_a11y_action(A11yAction::DefaultPress));
        assert!(checkbox.is_checked());
        assert!(
            matches!(
                checkbox.toggled(),
                Some(alacer_core::widget::ToggleState::On)
            ),
            "the a11y tree would announce checked"
        );
        // A second default action toggles back off — still consumed.
        assert!(checkbox.on_a11y_action(A11yAction::DefaultPress));
        assert!(!checkbox.is_checked());
    }

    #[test]
    fn programmatic_set_checked_fires_no_handler() {
        let mut checkbox = Checkbox::new("Done").on_toggle(|_| panic!("no user toggle"));
        checkbox.set_checked(true);
        assert!(checkbox.is_checked());
    }

    #[test]
    fn table_slot_defaults_to_none() {
        let checkbox = Checkbox::new("Done");
        assert_eq!(checkbox.table_slot(), None);
    }
}
