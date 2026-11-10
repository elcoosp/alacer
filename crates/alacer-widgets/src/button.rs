//! The button: clickable, pressable, named.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, Key, KeyEvent, KeyState, PointerEvent, PointerEvent as PE};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// An actionable button (role `Button`, name mandatory — ADR-5).
pub struct Button {
    label: String,
    pressed: bool,
    enabled: Enabled,
    fill: Color,
    fill_pressed: Color,
    text_color: Color,
    font_size: f32,
    corner_radius: f32,
    style: Style,
    on_click: Option<Box<dyn FnMut()>>,
}

/// Where the availability comes from (M22, ADR-0028): the app's
/// imperative write (`set_enabled`) or the reactive binding the flux
/// bridge drives — the value tier's two doors, at the availability tier.
enum Enabled {
    Fixed(bool),
    Binding(RenderBinding<bool>),
}

impl Enabled {
    /// The availability the widget reports, from wherever the app writes
    /// it.
    fn resolve(&self) -> bool {
        match self {
            Self::Fixed(available) => *available,
            Self::Binding(binding) => binding.with(|available| *available),
        }
    }
}

impl Button {
    /// A button labeled `label`.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            pressed: false,
            enabled: Enabled::Fixed(true),
            fill: crate::theme::ACCENT,
            fill_pressed: crate::theme::ACCENT_PRESSED,
            text_color: Color::WHITE,
            font_size: crate::theme::FONT_SIZE,
            corner_radius: 6.0,
            style: Style::default(),
            on_click: None,
        }
    }

    /// Initial availability (M22, ADR-0028). An unavailable button paints
    /// muted, holds no focus, and refuses every door honestly — the AT
    /// reads the refusal before it happens.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Enabled::Fixed(enabled);
        self
    }

    /// Tracks availability through a [`RenderBinding`] — the app writes a
    /// signal, the flux bridge queues the caller's masks, and the widget
    /// paints and reports from the binding (the `Progress::bound` rule,
    /// at the availability tier).
    #[must_use]
    pub fn enabled_binding(mut self, binding: RenderBinding<bool>) -> Self {
        self.enabled = Enabled::Binding(binding);
        self
    }

    /// Installs the click handler; invoked on release inside the button.
    #[must_use]
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

    /// The availability the widget reports — the same flag the M22
    /// `enabled()` contract carries into the a11y tree (test
    /// introspection).
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled.resolve()
    }

    /// The one availability write: report whether the flag moved. The
    /// caller requests the write's own masks — `PAINT | A11Y` — only when
    /// this answers `true`, so a write that lands where the flag already
    /// is queues nothing (the M21 idle-frame property, at the widget
    /// seam). A no-op on a binding-driven button: its availability
    /// belongs to the reactive binding (the `Label::set_text` rule).
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        let Enabled::Fixed(available) = &mut self.enabled else {
            return false;
        };
        if *available == enabled {
            return false;
        }
        *available = enabled;
        true
    }

    /// Replaces the label text in place (view reconciliation, §6.1).
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Sets the fill (and pressed fill) in place (view reconciliation).
    pub fn set_fill(&mut self, fill: Color) {
        self.fill = fill;
        self.fill_pressed = fill;
    }

    /// Sets the text color in place (view reconciliation).
    pub fn set_text_color(&mut self, color: Color) {
        self.text_color = color;
    }

    /// Replaces the click handler in place (view reconciliation). `None`
    /// detaches the handler.
    pub fn set_on_click(&mut self, handler: Option<Box<dyn FnMut()>>) {
        self.on_click = handler;
    }
}

impl Widget for Button {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Button
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.label.clone())
    }

    /// The availability tier (M22, ADR-0028): the flag the a11y tree
    /// carries — an unavailable button reports itself before the AT tries
    /// the press it will refuse.
    fn enabled(&self) -> bool {
        self.is_enabled()
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
        let available = self.is_enabled();
        // The look tells the truth the tree carries (M22, ADR-0028): an
        // unavailable button paints the muted rail and the muted text —
        // the theme constants the disabled state borrows.
        let fill = if !available {
            crate::theme::BORDER
        } else if self.pressed {
            self.fill_pressed
        } else {
            self.fill
        };
        let text_color = if available {
            self.text_color
        } else {
            crate::theme::TEXT_MUTED
        };
        crate::paint_background(
            ctx,
            size,
            fill,
            self.corner_radius,
            (Color::TRANSPARENT, 0.0),
        );

        // Centered text; the label wraps at the button's laid-out width —
        // the same constraint the measure pass saw, so line breaks match.
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.label, self.font_size, Some(size.0));
            let text_origin = Point::new(
                (size.0 - layout.width).max(0.0) * 0.5,
                (size.1 - layout.height).max(0.0) * 0.5,
            );
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
        let text_size = alacer_core::text_seam::with_seam(|seam| {
            seam.measure(
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
        // The availability tier (M22, ADR-0028): an unavailable button has
        // no write door — every action is refused honestly, never munged
        // (the M11 rule). The AT read the refusal before it happened: the
        // tree carries the disabled flag.
        if !self.is_enabled() {
            return false;
        }
        if action == alacer_core::widget::A11yAction::DefaultPress {
            if let Some(handler) = &mut self.on_click {
                handler();
            }
            return true;
        }
        false
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PE) -> EventResponse {
        // The availability tier (M22, ADR-0028): a disabled button is a
        // wall, not a window — the gesture is consumed so it never clicks
        // through to an ancestor, it arms nothing, and it requests no
        // masks (an idle frame; the HTML `disabled` semantics).
        if !self.is_enabled()
            && matches!(
                event,
                PointerEvent::Down { .. } | PointerEvent::Up { .. } | PointerEvent::Leave
            )
        {
            return EventResponse::Handled;
        }
        match event {
            PointerEvent::Down {
                button: _button, ..
            } => {
                self.pressed = true;
                // Click-to-focus: buttons take keyboard focus on press (§10.3),
                // which also detaches the IME from any editing widget.
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::FOCUS));
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

    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        // The availability tier (M22, ADR-0028): the keyboard never
        // reaches an unavailable control — it holds no focus (the
        // `focusable` derivation), and a routed activation key is ignored
        // so the framework keeps the key (nothing else consumes Space).
        if !self.is_enabled() {
            return EventResponse::Ignored;
        }
        // Keyboard activation (M10, §10.3 parity): a focused button answers
        // Space and Enter exactly like a pointer press-and-release. GTK
        // semantics — the click fires on key *release* — so an auto-repeat
        // hold cannot machine-gun the handler.
        let activation = matches!(event.key, Key::Enter | Key::Space);
        if !activation {
            return EventResponse::Ignored;
        }
        match event.state {
            KeyState::Pressed => {
                self.pressed = true;
                ctx.request(alacer_core::Change::PAINT);
            }
            KeyState::Released => {
                if self.pressed {
                    self.pressed = false;
                    ctx.request(alacer_core::Change::PAINT);
                    if let Some(handler) = &mut self.on_click {
                        handler();
                    }
                }
            }
        }
        EventResponse::Handled
    }
}

#[cfg(test)]
mod tests {
    use super::Button;
    use alacer_core::events::{
        EventCtx, Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent,
    };
    use alacer_core::flux::RenderBinding;
    use alacer_core::geom::Point;
    use alacer_core::widget::{A11yAction, EventResponse, Role, Widget};
    use std::cell::Cell;
    use std::rc::Rc;

    fn key(key: Key, state: KeyState) -> KeyEvent {
        KeyEvent {
            key,
            state,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    fn counted_button() -> (Button, Rc<Cell<u32>>) {
        let clicks = Rc::new(Cell::new(0));
        let counter = Rc::clone(&clicks);
        let button = Button::new("Increment").on_click(move || counter.set(counter.get() + 1));
        (button, clicks)
    }

    #[test]
    fn space_and_enter_activate_a_focused_button() {
        let (mut button, clicks) = counted_button();
        let mut ctx = EventCtx::new();

        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Space, KeyState::Pressed)),
            EventResponse::Handled,
            "space press shows the pressed state"
        );
        assert!(button.is_pressed(), "space press shows the pressed state");
        assert_eq!(clicks.get(), 0, "the click waits for the release (GTK)");

        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Space, KeyState::Released)),
            EventResponse::Handled
        );
        assert!(!button.is_pressed());
        assert_eq!(clicks.get(), 1, "exactly one click per press-and-release");

        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Enter, KeyState::Pressed)),
            EventResponse::Handled
        );
        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Enter, KeyState::Released)),
            EventResponse::Handled
        );
        assert_eq!(clicks.get(), 2, "enter activates too");

        // A held key auto-repeats presses; the release guard must keep the
        // handler to one run per physical press-and-release.
        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Enter, KeyState::Pressed)),
            EventResponse::Handled
        );
        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Enter, KeyState::Pressed)),
            EventResponse::Handled,
            "repeat presses on an already-pressed button are absorbed"
        );
        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Enter, KeyState::Released)),
            EventResponse::Handled
        );
        assert_eq!(clicks.get(), 3, "still one click per release");
    }

    #[test]
    fn a_stray_release_and_other_keys_do_not_activate() {
        let (mut button, clicks) = counted_button();
        let mut ctx = EventCtx::new();

        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Space, KeyState::Released)),
            EventResponse::Handled,
            "space is an activation key, so the event is handled…"
        );
        assert_eq!(
            clicks.get(),
            0,
            "…but a release without a press is not a click"
        );

        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Char('x'), KeyState::Pressed)),
            EventResponse::Ignored,
            "non-activation keys bubble to the framework (Tab cycling)"
        );
        assert_eq!(clicks.get(), 0);
    }

    #[test]
    fn pointer_click_still_fires_on_release_inside() {
        let (mut button, clicks) = counted_button();
        let mut ctx = EventCtx::new();
        let at = Point::new(10.0, 10.0);
        let _ = button.on_pointer_event(
            &mut ctx,
            &PointerEvent::Down {
                position: at,
                button: alacer_core::events::MouseButton::Left,
                click_count: 1,
            },
        );
        let _ = button.on_pointer_event(
            &mut ctx,
            &PointerEvent::Up {
                position: at,
                button: alacer_core::events::MouseButton::Left,
            },
        );
        assert_eq!(clicks.get(), 1, "the pointer path is unchanged by M10");
    }

    #[test]
    fn a11y_default_press_fires_and_step_actions_are_refused() {
        let (mut button, clicks) = counted_button();

        assert!(
            !button.on_a11y_action(A11yAction::Increment),
            "an increment is not a press — the platform must not munge it"
        );
        assert_eq!(clicks.get(), 0, "the refused action fired nothing");

        assert!(button.on_a11y_action(A11yAction::DefaultPress));
        assert_eq!(clicks.get(), 1);

        assert_eq!(button.role(), Role::Button);
    }

    #[test]
    fn a_disabled_button_refuses_every_door() {
        // M22, ADR-0028: the availability tier — the AT tier is refused
        // (never munged), the pointer is a wall (the gesture arms nothing
        // and queues no masks), the keyboard never activates, and the
        // write that flipped the flag is the only mask the tier costs.
        let (mut button, clicks) = counted_button();
        assert!(button.set_enabled(false), "the write moved the flag");
        let mut ctx = EventCtx::new();

        assert!(
            !button.on_a11y_action(A11yAction::DefaultPress),
            "an unavailable control has no write door"
        );
        assert_eq!(clicks.get(), 0, "the refusal fired nothing");

        let down = PointerEvent::Down {
            position: Point::new(10.0, 10.0),
            button: MouseButton::Left,
            click_count: 1,
        };
        assert_eq!(
            button.on_pointer_event(&mut ctx, &down),
            EventResponse::Handled,
            "the wall consumes the gesture so it never clicks through"
        );
        assert!(!button.is_pressed(), "the gesture armed nothing");
        assert!(
            ctx.take().is_empty(),
            "a refused gesture queues no masks — an idle frame"
        );

        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Space, KeyState::Pressed)),
            EventResponse::Ignored,
            "the keyboard never reaches an unavailable control"
        );
        assert_eq!(
            button.on_key_event(&mut ctx, &key(Key::Space, KeyState::Released)),
            EventResponse::Ignored
        );
        assert_eq!(clicks.get(), 0);
        assert!(!button.focusable(), "an unavailable control holds no focus");
    }

    #[test]
    fn set_enabled_answers_the_move_and_the_binding_owns_its_own() {
        // The M21 idle-frame property, at the availability seam: a write
        // that lands where the flag already is answers `false` — the
        // caller queues nothing.
        let mut button = Button::new("Send invite");
        assert!(!button.set_enabled(true), "already available: no move");
        assert!(button.set_enabled(false), "the write moved");
        assert!(!button.is_enabled());
        assert!(!button.set_enabled(false), "the wall answers no move");

        // The binding door: the imperative write is a no-op on a
        // binding-driven button (the `Label::set_text` rule).
        let binding = RenderBinding::new(false);
        let mut bound = Button::new("Send invite").enabled_binding(binding.clone());
        assert!(!bound.is_enabled(), "the binding is the source");
        assert!(!bound.set_enabled(true), "the flag belongs to the binding");
        binding.set(true);
        assert!(bound.is_enabled(), "the read is live");
        assert!(bound.is_enabled() == Widget::enabled(&bound), "one truth");
    }
}
