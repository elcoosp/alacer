//! The switch tier (M130, ADR-0134): the instant on/off control, said
//! by the tree. Three ways to flip, one contract: a click, the Space
//! key while focused, and the AccessKit default action all resolve the
//! same binary state and request the same masks — `PAINT | A11Y` — so
//! the thumb repaints and the a11y tree re-announces the fact. The
//! widget never writes a11y directly (A1: masks are the only currency).
//!
//! A switch is not a checkbox wearing the wrong word: a checkbox's
//! write resolves a *form contract* — checked, unchecked, the mixed
//! summary the select-all parent needs (ADR-0098) — while a switch's
//! write is the instant on/off the settings vocabulary speaks. The
//! binary pair rides the M6 `toggled` contract — `On` or `Off`, never
//! `Mixed`: a switch has no summary to hold, never a guessed third bit.
//!
//! The programmatic door ([`Switch::set_on`], the view-reconciliation
//! path) fires no handler — programmatic state, never a user toggle —
//! and answers whether the flag moved, so a write that lands where the
//! on/off already is queues nothing (the M21 idle-frame property, at
//! the switch tier).

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, KeyEvent, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, ToggleState, Widget};

/// The pill track and the thumb's geometry (the switch's whole paint
/// story, deterministic pixels with no glyph dependency).
const TRACK_WIDTH: f32 = 34.0;
const TRACK_HEIGHT: f32 = 20.0;

/// An instant on/off control with a mandatory name (ADR-5: an unnamed
/// switch reads as a bare pill — the set the AT cannot announce is a
/// toggle it cannot ride) and the binary pair the M6 `toggled`
/// contract announces (M130, ADR-0134) — `On` or `Off`, never `Mixed`.
pub struct Switch {
    name: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    on: bool,
    enabled: bool,
    on_toggle: Option<Box<dyn FnMut(bool)>>,
    focused: bool,
    /// The app's accent override; `None` borrows the theme's accent
    /// role at paint time (ADR-0079).
    accent: Option<Color>,
    /// The app's text-color override; `None` borrows the theme's text
    /// role at paint time (ADR-0079).
    text_color: Option<Color>,
    font_size: f32,
    style: Style,
}

impl Switch {
    /// A switch labeled `name`, initially off.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            on: false,
            enabled: true,
            on_toggle: None,
            focused: false,
            accent: None,
            text_color: None,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// Initial availability (M22, ADR-0028). An unavailable switch
    /// paints muted, holds no focus, and refuses every toggle door
    /// honestly — the AT reads the refusal before it happens.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Initial on/off fact — `true` mounts the switch on, the binary
    /// pair the tree announces from the first frame.
    #[must_use]
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// Installs the toggle handler; invoked with the *new* on/off flag
    /// after every toggle (click, Space, a11y action).
    #[must_use]
    pub fn on_toggle(mut self, handler: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// The description (M95, ADR-0099): the help text the a11y tree
    /// carries beside the name — the instruction the label is too short
    /// to carry. Without it the switch announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Accent color of the track. Without it the track borrows the
    /// theme's accent role at paint time (ADR-0079).
    #[must_use]
    pub fn accent(mut self, accent: Color) -> Self {
        self.accent = Some(accent);
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

    /// Shared-borrow layout style access for view reconciliation (§6.1):
    /// patches write styles in place instead of rebuilding the widget.
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    /// Current on/off fact (test introspection): `true` only when the
    /// switch is on.
    #[must_use]
    pub const fn is_on(&self) -> bool {
        self.on
    }

    /// The availability the widget reports — the same flag the M22
    /// `enabled()` contract carries into the a11y tree (test
    /// introspection).
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The one availability write: report whether the flag moved (the
    /// M21 idle-frame property, at the widget seam).
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        if self.enabled == enabled {
            return false;
        }
        self.enabled = enabled;
        true
    }

    /// The one programmatic on/off write (view reconciliation): report
    /// whether the flag moved — the caller requests `PAINT | A11Y` only
    /// when this answers `true`, so a write that lands where the on/off
    /// already is queues nothing (the M21 idle-frame property, at the
    /// switch tier). Fires no handler — programmatic state, never a
    /// user door.
    pub fn set_on(&mut self, on: bool) -> bool {
        if self.on == on {
            return false;
        }
        self.on = on;
        true
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
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

    /// Replaces the toggle handler in place (view reconciliation). `None`
    /// detaches it.
    pub fn set_on_toggle(&mut self, handler: Option<Box<dyn FnMut(bool)>>) {
        self.on_toggle = handler;
    }

    /// The one toggle path: flip the binary pair, fire the handler with
    /// the new on/off flag, request the masks the change costs, and
    /// report whether the toggle *landed* — `false` when the
    /// availability wall held (an unavailable switch toggles nothing),
    /// so the record the keyboard's route streams answers the door it
    /// actually answered (the funnel's own truth at the action route: a
    /// refused ask is a refusal, never a munged `Delivered`). The
    /// availability tier guards the funnel's single entry (M22,
    /// ADR-0028); the binary pair never holds a summary (ADR-0098's
    /// rule at the switch tier — a user door always moves the tree's
    /// truth, the toggle the switch is).
    fn toggle(&mut self, ctx: &mut EventCtx) -> bool {
        if !self.enabled {
            return false;
        }
        self.on = !self.on;
        ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        if let Some(handler) = &mut self.on_toggle {
            handler(self.on);
        }
        true
    }

    /// Intrinsic size: the pill track, a gap, and the label.
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
        alacer_core::geom::Size::new(
            TRACK_WIDTH + 8.0 + text.width,
            text.height.max(TRACK_HEIGHT) + 8.0,
        )
    }
}

impl Widget for Switch {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Switch
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the switch has nothing to explain.
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn style(&self) -> Style {
        Style {
            padding: 4.0,
            ..self.style.clone()
        }
    }

    fn toggled(&self) -> Option<ToggleState> {
        Some(ToggleState::from(self.on))
    }

    /// The availability tier (M22, ADR-0028): the flag the a11y tree
    /// carries — an unavailable switch reports itself before the AT
    /// tries the toggle it will refuse.
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let size = ctx.size();
        let track_height = TRACK_HEIGHT.min(size.height);
        let track_top = (size.height - track_height) * 0.5;
        // The borrowed roles resolve per paint pass (ADR-0079); the
        // look still tells the truth the tree carries (M22, ADR-0028):
        // an unavailable switch paints the muted track and the muted
        // label — the border and muted roles the disabled state borrows.
        let palette = ctx.palette();
        let accent = if self.enabled {
            self.accent.unwrap_or(palette.accent)
        } else {
            palette.border
        };
        let label_color = if self.enabled {
            self.text_color.unwrap_or(palette.text)
        } else {
            palette.text_muted
        };

        // The track: a filled pill — accent when the switch is on, the
        // panel when it is off; the border carries the focus ring.
        let track = alacer_core::geom::Rect::new(
            Point::new(0.0, track_top),
            alacer_core::geom::Size::new(TRACK_WIDTH, track_height),
        );
        ctx.push_quad(alacer_core::scene::Quad {
            rect: track,
            color: if self.on { accent } else { palette.panel },
            corner_radius: track_height * 0.5,
            border_color: if self.focused { accent } else { palette.border },
            border_width: if self.focused { 2.0 } else { 1.5 },
            clip: None,
        });

        // The thumb: a white disc riding the on/off fact — left when
        // off, right when on (deterministic pixels with no glyph
        // dependency; the snapshot corpus stays font-free).
        let thumb_side = track_height - 6.0;
        let thumb_x = if self.on {
            TRACK_WIDTH - thumb_side - 3.0
        } else {
            3.0
        };
        let thumb = alacer_core::geom::Rect::new(
            Point::new(thumb_x, track_top + (track_height - thumb_side) * 0.5),
            alacer_core::geom::Size::new(thumb_side, thumb_side),
        );
        ctx.push_quad(alacer_core::scene::Quad {
            rect: thumb,
            color: Color::WHITE,
            corner_radius: thumb_side * 0.5,
            border_color: Color::WHITE,
            border_width: 0.0,
            clip: None,
        });

        // The label, right of the track.
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.name, self.font_size, None);
            let text_origin = Point::new(
                TRACK_WIDTH + 8.0,
                (size.height - layout.height).max(0.0) * 0.5,
            );
            for glyph in &layout.glyphs {
                ctx.push_glyph(alacer_core::scene::Glyph {
                    position: Point::new(text_origin.x + glyph.x, text_origin.y + glyph.y),
                    size: glyph.size,
                    color: label_color,
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
        // The availability tier (M22, ADR-0028): an unavailable switch
        // has no write door — the toggle is refused honestly, never
        // munged (the M11 rule).
        if !self.enabled {
            return false;
        }
        if action == A11yAction::DefaultPress {
            // The a11y path has no EventCtx; the dispatch funnel queues
            // `PAINT | A11Y` itself when the door consumes — the toggle
            // itself is handler-first, exactly like Button's press path.
            // The binary pair never holds a summary: every ask moves
            // the tree's truth, the toggle the switch is.
            self.on = !self.on;
            if let Some(handler) = &mut self.on_toggle {
                handler(self.on);
            }
            return true;
        }
        false
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        match event {
            PointerEvent::Down { .. } => {
                // The wall (M22, ADR-0028): a disabled switch consumes
                // the press but requests nothing — no focus, no repaint.
                if !self.enabled {
                    return EventResponse::Handled;
                }
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
            let landed = self.toggle(ctx);
            // The keyboard activation names itself (M70, ADR-0076): the
            // toggle-on-press ride — the focused switch answering Space
            // like a click — streams one `ActionRecord` with `Keyboard`
            // beside it, the attribution declared where the route is
            // known (the M30 rider rule at the activation tier). The
            // record answers the door it actually answered: a toggle
            // that landed is `Delivered`; one the availability wall
            // held is the refusal the funnel's own route computes from
            // the tree's one `enabled` truth (M22, ADR-0028) — never a
            // munged `Delivered`. The stamp rides the key route's guard
            // (M45, ADR-0051) — the focused control's id, never guessed
            // at the note site. One branch when nobody listens (the A7
            // idle contract).
            alacer_core::action::note_action(alacer_core::ActionRecord {
                ask: alacer_core::inspect::ActionAsk::Press,
                target: None,
                outcome: if landed {
                    alacer_core::inspect::ActionOutcome::Delivered
                } else {
                    alacer_core::inspect::ActionOutcome::Refused(
                        alacer_core::inspect::ActionRefusal::Unavailable,
                    )
                },
                rider: alacer_core::inspect::ActionRider::Keyboard,
            });
            return EventResponse::Handled;
        }
        EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::Switch;
    use alacer_core::events::{EventCtx, Key, KeyEvent, KeyState, Modifiers, PointerEvent};
    use alacer_core::geom::Point;
    use alacer_core::mask::Change;
    use alacer_core::widget::{A11yAction, EventResponse, Role, ToggleState, Widget};

    fn space() -> KeyEvent {
        KeyEvent {
            key: Key::Space,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    #[test]
    fn role_name_and_on_off_fact_satisfy_the_tier() {
        let switch = Switch::new("Wi-Fi");
        assert_eq!(switch.role(), Role::Switch);
        assert_eq!(switch.label().resolve(), "Wi-Fi");
        assert!(switch.focusable(), "switches hold focus");
        assert!(switch.role().requires_name());
        assert_eq!(switch.role().name(), "switch");
        assert_eq!(
            switch.toggled(),
            Some(ToggleState::Off),
            "the binary pair rides the M6 contract — off the honest start"
        );
        assert!(
            !switch.is_on(),
            "the constructor's switch is off — the tree never guesses"
        );
    }

    #[test]
    fn the_binary_pair_never_holds_a_summary() {
        // ADR-0098's rule at the switch tier: every door resolves the
        // binary pair — On or Off, never the mixed summary the
        // select-all parent needs and a switch does not hold.
        let mut switch = Switch::new("Wi-Fi");
        let mut ctx = EventCtx::new();
        let down = PointerEvent::Down {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 1,
        };
        let up = PointerEvent::Up {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        let _ = switch.on_pointer_event(&mut ctx, &down);
        let _ = switch.on_pointer_event(&mut ctx, &up);
        assert_eq!(switch.toggled(), Some(ToggleState::On));
        let _ = switch.on_pointer_event(&mut ctx, &down);
        let _ = switch.on_pointer_event(&mut ctx, &up);
        assert_eq!(
            switch.toggled(),
            Some(ToggleState::Off),
            "a switch toggles — never a summary, the pair is the whole story"
        );
    }

    #[test]
    fn click_toggles_and_requests_paint_plus_a11y() {
        let mut switch = Switch::new("Bluetooth");
        let mut ctx = EventCtx::new();
        let down = PointerEvent::Down {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 1,
        };
        assert_eq!(
            switch.on_pointer_event(&mut ctx, &down),
            EventResponse::Handled
        );
        let up = PointerEvent::Up {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        assert_eq!(
            switch.on_pointer_event(&mut ctx, &up),
            EventResponse::Handled
        );
        assert!(switch.is_on(), "click toggles");
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
        let mut switch = Switch::new("Wi-Fi").on_toggle(move |on| sink.set(on));
        let mut ctx = EventCtx::new();
        assert_eq!(
            switch.on_key_event(&mut ctx, &space()),
            EventResponse::Handled
        );
        assert!(switch.is_on());
        assert!(seen.get(), "the handler observed the new state");
        let masks = ctx.take();
        assert!(masks.contains(Change::A11Y), "keyboard toggle re-announces");

        // A second Space flips it back.
        let mut ctx = EventCtx::new();
        let _ = switch.on_key_event(&mut ctx, &space());
        assert!(!switch.is_on());
    }

    #[test]
    fn other_keys_are_ignored() {
        let mut switch = Switch::new("Wi-Fi");
        let mut ctx = EventCtx::new();
        let enter = KeyEvent {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert_eq!(
            switch.on_key_event(&mut ctx, &enter),
            EventResponse::Ignored
        );
        assert!(!switch.is_on());
    }

    #[test]
    fn the_a11y_default_action_toggles() {
        let mut switch = Switch::new("Wi-Fi");
        assert!(switch.on_a11y_action(A11yAction::DefaultPress));
        assert!(switch.is_on());
        assert!(
            matches!(switch.toggled(), Some(ToggleState::On)),
            "the a11y tree would announce on"
        );
        // A second default action toggles back off — still consumed.
        assert!(switch.on_a11y_action(A11yAction::DefaultPress));
        assert!(!switch.is_on());
    }

    #[test]
    fn programmatic_set_on_fires_no_handler() {
        let mut switch = Switch::new("Wi-Fi").on_toggle(|_| panic!("no user toggle"));
        assert!(switch.set_on(true), "the write moved");
        assert!(switch.is_on());
        assert!(
            !switch.set_on(true),
            "the write that lands where the flag already is answers not moved"
        );
    }

    #[test]
    fn a_disabled_switch_refuses_every_toggle_door() {
        // M22, ADR-0028: the AT tier is refused, the pointer arms nothing,
        // Space flips nothing, and the write that moved the flag is the
        // only mask the tier costs.
        let mut switch = Switch::new("Wi-Fi").enabled(false).on_toggle(|_| {
            panic!("an unavailable switch must not toggle");
        });
        let mut ctx = EventCtx::new();

        assert!(
            !switch.on_a11y_action(A11yAction::DefaultPress),
            "an unavailable toggle has no write door"
        );
        assert_eq!(
            switch.on_pointer_event(
                &mut ctx,
                &PointerEvent::Down {
                    position: Point::new(4.0, 4.0),
                    button: alacer_core::events::MouseButton::Left,
                    click_count: 1,
                }
            ),
            EventResponse::Handled,
            "the wall consumes the press"
        );
        assert!(
            ctx.take().is_empty(),
            "a refused press queues no masks — an idle frame"
        );
        assert_eq!(
            switch.on_key_event(&mut ctx, &space()),
            EventResponse::Handled,
            "Space is an activation key, so the event is handled…"
        );
        assert!(!switch.is_on(), "…but the toggle wrote nothing");
        assert!(!switch.focusable(), "an unavailable control holds no focus");

        // The write answers the move (the M21 idle-frame property).
        assert!(!switch.set_enabled(false), "already unavailable");
        assert!(switch.set_enabled(true), "the write moved");
        assert!(switch.is_enabled());
    }

    #[test]
    fn the_description_rides_the_trait_and_the_write_reports_the_move() {
        // M95, ADR-0099: the honest absence by default, the help text the
        // builder mounts, and the one write whose moved answer is the
        // only mask the tier costs.
        let mut switch = Switch::new("Wi-Fi");
        assert_eq!(
            Widget::description(&switch),
            None,
            "an undescribed switch announces the absence"
        );
        switch = switch.description("Turns the radio off and on");
        assert_eq!(
            Widget::description(&switch),
            Some("Turns the radio off and on"),
            "the help text the tree carries beside the name"
        );

        // The write that landed where the text already is, then the write
        // that moved (the M21 idle-frame property, at the description
        // tier).
        assert!(
            !switch.set_description(Some("Turns the radio off and on".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(
            switch.set_description(Some("Joins the office network".to_string())),
            "the moved write answers true"
        );
        assert_eq!(
            Widget::description(&switch),
            Some("Joins the office network")
        );
        assert!(
            switch.set_description(None),
            "clearing to the absence moved"
        );
        assert_eq!(Widget::description(&switch), None);
        assert!(
            !switch.set_description(None),
            "the absence is already there"
        );
    }
}
