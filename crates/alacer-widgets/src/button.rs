//! The button: clickable, pressable, named.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, Key, KeyEvent, KeyState, PointerEvent, PointerEvent as PE};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{CurrentKind, EventResponse, HasPopupKind, LabelSource, Role, Widget};

/// An actionable button (role `Button`, name mandatory — ADR-5).
pub struct Button {
    label: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    /// The current tier's word (M114, ADR-0118): which item of a set
    /// the reader stands in; `None` is the honest absence.
    current: Option<CurrentKind>,
    /// The has-popup tier's word (M115, ADR-0119): what the control
    /// opens; `None` is the honest absence.
    has_popup: Option<HasPopupKind>,
    /// The role-description tier's word (M116, ADR-0120): the role the
    /// control is *read as*; `None` is the honest absence.
    role_description: Option<String>,
    /// The shortcut tier's word (M121, ADR-0125): the chord the control
    /// is invoked with; `None` is the honest absence.
    keyboard_shortcut: Option<String>,
    pressed: bool,
    enabled: Enabled,
    /// The app's fill override; `None` borrows the theme's accent role
    /// at paint time (ADR-0079).
    fill: Option<Color>,
    /// The app's pressed-fill override; `None` borrows the theme's
    /// pressed-accent role at paint time (ADR-0079).
    fill_pressed: Option<Color>,
    /// The app's text-color override; `None` borrows the theme's
    /// on-accent role at paint time (ADR-0079).
    text_color: Option<Color>,
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
            description: None,
            current: None,
            has_popup: None,
            role_description: None,
            keyboard_shortcut: None,
            pressed: false,
            enabled: Enabled::Fixed(true),
            fill: None,
            fill_pressed: None,
            text_color: None,
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

    /// The current word (M114, ADR-0118): which item of a set the
    /// reader stands in — the nav's page, the wizard's step — the
    /// currency the tree says from the first frame
    /// (`aria-current`, AccessKit's `AriaCurrent` property).
    #[must_use]
    pub fn current(mut self, current: CurrentKind) -> Self {
        self.current = Some(current);
        self
    }

    /// The has-popup word (M115, ADR-0119): what the control opens —
    /// the menu button's menu, the combobox's listbox — the target the
    /// tree says from the first frame (`aria-haspopup`, AccessKit's
    /// `HasPopup` property).
    #[must_use]
    pub fn has_popup(mut self, has_popup: HasPopupKind) -> Self {
        self.has_popup = Some(has_popup);
        self
    }

    /// The role-description word (M116, ADR-0120): the role the control
    /// is *read as* — the star rating behind an ordinary slider-shaped
    /// control, the wizard page behind an ordinary button — the
    /// specialization the tree says from the first frame
    /// (`aria-roledescription`, AccessKit's `RoleDescription` property).
    #[must_use]
    pub fn role_description(mut self, role_description: impl Into<String>) -> Self {
        self.role_description = Some(role_description.into());
        self
    }

    /// The shortcut word (M121, ADR-0125): the chord the control is
    /// invoked with — the toolbar save's "Control+S", the print
    /// dialog's "Control+Shift+P" — the chord the tree says from the
    /// first frame (`aria-keyshortcuts`, AccessKit's `KeyboardShortcut`
    /// property). The word names the chord the app announced, it never
    /// binds a key: the keymap seam (ADR-0043) owns the dispatch.
    #[must_use]
    pub fn keyboard_shortcut(mut self, keyboard_shortcut: impl Into<String>) -> Self {
        self.keyboard_shortcut = Some(keyboard_shortcut.into());
        self
    }

    /// The description (M95, ADR-0099): the help text the a11y tree
    /// carries beside the name — the instruction the label is too short
    /// to carry (a format, a consequence, the "what does this do?"
    /// sentence). Without it the button announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Fill colors `(rest, pressed)`. Without them the button borrows
    /// the theme's accent roles at paint time (ADR-0079).
    #[must_use]
    pub fn colors(mut self, rest: Color, pressed: Color) -> Self {
        self.fill = Some(rest);
        self.fill_pressed = Some(pressed);
        self
    }

    /// Text color. Without it the label borrows the theme's on-accent
    /// role at paint time (ADR-0079).
    #[must_use]
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
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

    /// The one description write (M95, ADR-0099, view reconciliation):
    /// `None` clears to the honest absence. Report whether the text
    /// moved — the caller requests `A11Y` only when this answers `true`,
    /// so a write that lands where the text already is queues nothing
    /// (the M21 idle-frame property, at the description tier).
    pub fn set_description(&mut self, description: Option<String>) -> bool {
        if self.description == description {
            return false;
        }
        self.description = description;
        true
    }

    /// The one current write (M114, ADR-0118, view reconciliation):
    /// `None` clears to the honest absence. Report whether the word
    /// moved — the caller requests `PAINT | A11Y` only when this
    /// answers `true`, so a write that lands where the word already is
    /// queues nothing (the M21 idle-frame property, at the current tier).
    pub fn set_current(&mut self, current: Option<CurrentKind>) -> bool {
        if self.current == current {
            return false;
        }
        self.current = current;
        true
    }

    /// The one has-popup write (M115, ADR-0119, view reconciliation):
    /// `None` clears to the honest absence. Report whether the word
    /// moved — the caller requests `PAINT | A11Y` only when this
    /// answers `true`, so a write that lands where the word already is
    /// queues nothing (the M21 idle-frame property, at the has-popup
    /// tier).
    pub fn set_has_popup(&mut self, has_popup: Option<HasPopupKind>) -> bool {
        if self.has_popup == has_popup {
            return false;
        }
        self.has_popup = has_popup;
        true
    }

    /// The one role-description write (M116, ADR-0120, view
    /// reconciliation): `None` clears to the honest absence. Report
    /// whether the word moved — the caller requests `A11Y` only when
    /// this answers `true`, so a write that lands where the word
    /// already is queues nothing (the M21 idle-frame property, at the
    /// role-description tier).
    pub fn set_role_description(&mut self, role_description: Option<String>) -> bool {
        if self.role_description == role_description {
            return false;
        }
        self.role_description = role_description;
        true
    }

    /// Sets the fill (and pressed fill) in place (view reconciliation) —
    /// an explicit override, immune to rethemes.
    pub fn set_fill(&mut self, fill: Color) {
        self.fill = Some(fill);
        self.fill_pressed = Some(fill);
    }

    /// Sets the text color in place (view reconciliation) — an explicit
    /// override, immune to rethemes.
    pub fn set_text_color(&mut self, color: Color) {
        self.text_color = Some(color);
    }

    /// The one shortcut write (M121, ADR-0125, view reconciliation):
    /// `None` clears to the honest absence. Report whether the word
    /// moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the shortcut tier).
    pub fn set_keyboard_shortcut(&mut self, keyboard_shortcut: Option<String>) -> bool {
        if self.keyboard_shortcut == keyboard_shortcut {
            return false;
        }
        self.keyboard_shortcut = keyboard_shortcut;
        true
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

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the button has nothing to explain.
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// The current tier (M114, ADR-0118): which item of a set the
    /// reader stands in, the honest absence when the button stands in
    /// no set — an item word, so every other role inherits the default.
    fn current(&self) -> Option<CurrentKind> {
        self.current
    }

    /// The has-popup tier (M115, ADR-0119): what the control opens,
    /// the honest absence when the button opens nothing — an opener
    /// word, so every other role inherits the default.
    fn has_popup(&self) -> Option<HasPopupKind> {
        self.has_popup
    }

    /// The role-description tier (M116, ADR-0120): the role the control
    /// is *read as*, the honest absence when the button claims no
    /// specialization — a custom-control word, so every other role
    /// inherits the default.
    fn role_description(&self) -> Option<&str> {
        self.role_description.as_deref()
    }

    /// The shortcut tier (M121, ADR-0125): the chord the control is
    /// invoked with, the honest absence when the button carries none —
    /// an actionable word, so every other role inherits the default.
    fn keyboard_shortcut(&self) -> Option<&str> {
        self.keyboard_shortcut.as_deref()
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
        // The borrowed roles resolve per paint pass (ADR-0079): a plain
        // button follows the installed palette, an overridden one keeps
        // the colors its app set.
        let palette = ctx.palette();
        // The look tells the truth the tree carries (M22, ADR-0028): an
        // unavailable button paints the muted rail and the muted text —
        // the border and muted roles the disabled state borrows.
        let fill = if !available {
            palette.border
        } else if self.pressed {
            self.fill_pressed.unwrap_or(palette.accent_pressed)
        } else {
            self.fill.unwrap_or(palette.accent)
        };
        let text_color = if available {
            self.text_color.unwrap_or(palette.on_accent)
        } else {
            palette.text_muted
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
                    // The keyboard activation names itself (M70,
                    // ADR-0076): the M10 parity's ride — the focused
                    // button answering Space/Enter like a click —
                    // streams one `ActionRecord` with `Keyboard`
                    // beside it, the attribution declared where the
                    // route is known (the M30 rider rule at the
                    // activation tier). One fact per activation (the
                    // flood rule): the pressed phase armed nothing on
                    // the log, the release is where the door answered.
                    // The stamp rides the key route's guard (M45,
                    // ADR-0051) — the focused control's id, never
                    // guessed at the note site. One branch when nobody
                    // listens (the A7 idle contract).
                    alacer_core::action::note_action(alacer_core::ActionRecord {
                        ask: alacer_core::inspect::ActionAsk::Press,
                        target: None,
                        outcome: alacer_core::inspect::ActionOutcome::Delivered,
                        rider: alacer_core::inspect::ActionRider::Keyboard,
                    });
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

    #[test]
    fn the_description_rides_the_trait_and_the_write_reports_the_move() {
        // M95, ADR-0099: the honest absence by default, the help text the
        // builder mounts, and the one write whose moved answer is the
        // only mask the tier costs.
        let mut button = Button::new("Publish");
        assert_eq!(
            Widget::description(&button),
            None,
            "an undescribed button announces the absence"
        );
        button = button.description("Saves the draft and queues it");
        assert_eq!(
            Widget::description(&button),
            Some("Saves the draft and queues it"),
            "the help text the tree carries beside the name"
        );
        assert!(
            !button.set_description(Some("Saves the draft and queues it".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(
            button.set_description(None),
            "clearing to the absence moved"
        );
        assert_eq!(Widget::description(&button), None);
        assert!(
            !button.set_description(None),
            "the absence is already there"
        );
    }

    #[test]
    fn the_current_word_rides_the_builder_and_the_door_reports_the_move() {
        // M114, ADR-0118: the honest absence by default, the currency
        // the builder mounts, and the one write whose moved answer is
        // the only mask the tier costs — the kind riding the word, the
        // clear restoring the absence.
        use alacer_core::widget::CurrentKind;
        let mut button = Button::new("Home");
        assert_eq!(
            Widget::current(&button),
            None,
            "an unworded item announces no currency — never a guessed `false`"
        );
        button = button.current(CurrentKind::Page);
        assert_eq!(
            Widget::current(&button),
            Some(CurrentKind::Page),
            "the currency the tree carries beside the name"
        );
        assert!(
            !button.set_current(Some(CurrentKind::Page)),
            "the idle write answers false — no masks"
        );
        assert!(
            button.set_current(Some(CurrentKind::Step)),
            "the kind moved"
        );
        assert_eq!(Widget::current(&button), Some(CurrentKind::Step));
        assert!(button.set_current(None), "clearing to the absence moved");
        assert_eq!(Widget::current(&button), None);
        assert!(!button.set_current(None), "the absence is already there");
    }

    #[test]
    fn the_has_popup_word_rides_the_builder_and_the_door_reports_the_move() {
        // M115, ADR-0119: the honest absence by default, the target the
        // builder mounts, and the one write whose moved answer is the
        // only mask the tier costs — the kind riding the word, the
        // clear restoring the absence.
        use alacer_core::widget::HasPopupKind;
        let mut button = Button::new("Actions");
        assert_eq!(
            Widget::has_popup(&button),
            None,
            "a control that opens nothing announces no target — never a guessed one"
        );
        button = button.has_popup(HasPopupKind::Menu);
        assert_eq!(
            Widget::has_popup(&button),
            Some(HasPopupKind::Menu),
            "the target the tree carries beside the name"
        );
        assert!(
            !button.set_has_popup(Some(HasPopupKind::Menu)),
            "the idle write answers false — no masks"
        );
        assert!(
            button.set_has_popup(Some(HasPopupKind::Dialog)),
            "the kind moved"
        );
        assert_eq!(Widget::has_popup(&button), Some(HasPopupKind::Dialog));
        assert!(button.set_has_popup(None), "clearing to the absence moved");
        assert_eq!(Widget::has_popup(&button), None);
        assert!(!button.set_has_popup(None), "the absence is already there");
    }

    #[test]
    fn the_role_description_word_rides_the_builder_and_the_door_reports_the_move() {
        // M116, ADR-0120: the honest absence by default, the read-as
        // role the builder mounts, and the one write whose moved answer
        // is the only mask the tier costs — the word the tree says
        // beside the generic role, the clear restoring the absence.
        let mut button = Button::new("Chapter 3");
        assert_eq!(
            Widget::role_description(&button),
            None,
            "a control that claims no specialization is read as the generic role — never a guessed one"
        );
        button = button.role_description("wizard page");
        assert_eq!(
            Widget::role_description(&button),
            Some("wizard page"),
            "the read-as role the tree carries beside the generic one"
        );
        assert!(
            !button.set_role_description(Some("wizard page".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(
            button.set_role_description(Some("checklist step".to_string())),
            "the word moved"
        );
        assert_eq!(
            Widget::role_description(&button),
            Some("checklist step"),
            "the same door moves the specialization"
        );
        assert!(
            button.set_role_description(None),
            "clearing to the absence moved"
        );
        assert_eq!(Widget::role_description(&button), None);
        assert!(
            !button.set_role_description(None),
            "the absence is already there"
        );
    }
}
