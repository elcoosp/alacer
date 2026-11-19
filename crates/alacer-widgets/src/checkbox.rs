//! The checkbox: the todo canon's toggle control (M6), grown the tri-state
//! summary (M94, ADR-0098). Three ways to toggle, one contract: a click,
//! the Space key while focused, and the AccessKit default action all
//! resolve the same state and request the same masks — `PAINT | A11Y` —
//! so the mark repaints and the a11y tree re-announces the state. The
//! widget never writes a11y directly (A1: masks are the only currency).
//!
//! The mixed state is a *summary*, never a third bit (ADR-0098's
//! resolution rule): a checkbox holding `ToggleState::Mixed` — the
//! select-all parent's "some of the children are" — answers every user
//! door by resolving fully checked, the way every toolkit's tri-state
//! parent behaves. Only the programmatic door
//! ([`Checkbox::set_state`], the view-reconciliation path) writes the
//! summary back; user doors resolve it.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, KeyEvent, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, ToggleState, Widget};

/// A toggle with a mandatory name (ADR-5: an unnamed checkbox is
/// unusable with a screen reader) and the full [`ToggleState`]
/// vocabulary — the binary pair plus the mixed summary (M94, ADR-0098).
pub struct Checkbox {
    name: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    state: ToggleState,
    enabled: bool,
    /// The required tier's flag (M106, ADR-0110): the word the AT
    /// reads before it asks, the doors unchanged beside it.
    required: bool,
    /// The invalid tier's word (M107, ADR-0111): whether the answer
    /// fails the form's contract, and of what kind — the rejection
    /// the AT reads while the user edits, the doors unchanged beside
    /// it. `None` the honest absence.
    invalid: Option<alacer_core::InvalidKind>,
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

impl Checkbox {
    /// A checkbox labeled `name`, initially unchecked.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            state: ToggleState::Off,
            enabled: true,
            required: false,
            invalid: None,
            on_toggle: None,
            focused: false,
            accent: None,
            text_color: None,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// Initial availability (M22, ADR-0028). An unavailable checkbox
    /// paints muted, holds no focus, and refuses every toggle door
    /// honestly — the AT reads the refusal before it happens.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Mounts the checkbox required (M106, ADR-0110): the form
    /// contract's declared word — AccessKit's `Required` flag (ARIA's
    /// `aria-required`) says "this one must be answered" to the AT
    /// before it asks. The word prices nothing but the word: the
    /// checkbox's doors answer exactly as they always did.
    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Mounts the checkbox invalid (M107, ADR-0111): the form's
    /// rejection declared — AccessKit's `Invalid` property (ARIA's
    /// `aria-invalid`) says "this one is not valid as given" to the AT
    /// while the user edits, the kind (`true` / `grammar` /
    /// `spelling`) riding beside the word. The word prices nothing but
    /// the word: the checkbox's doors answer exactly as they always
    /// did.
    #[must_use]
    pub fn invalid(mut self, invalid: alacer_core::InvalidKind) -> Self {
        self.invalid = Some(invalid);
        self
    }

    /// Initial checked state — the binary door: `true` checked, `false`
    /// unchecked. The tri-state summary rides [`Checkbox::state`].
    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.state = ToggleState::from(checked);
        self
    }

    /// Initial tri-state (M94, ADR-0098): `ToggleState::Mixed` mounts the
    /// summary — the select-all parent's "some of the children are".
    /// Every user door resolves it (the resolution rule: the first press
    /// lands fully checked); only [`Checkbox::set_state`] writes it back.
    #[must_use]
    pub fn state(mut self, state: ToggleState) -> Self {
        self.state = state;
        self
    }

    /// Installs the toggle handler; invoked with the *new* state after every
    /// toggle (click, Space, a11y action).
    #[must_use]
    pub fn on_toggle(mut self, handler: impl FnMut(bool) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// The description (M95, ADR-0099): the help text the a11y tree
    /// carries beside the name — the instruction the label is too short
    /// to carry. Without it the checkbox announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Accent color of the box and its check mark. Without it the box
    /// borrows the theme's accent role at paint time (ADR-0079).
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

    /// Current checked state (test introspection): `true` only when the
    /// box is fully checked — the mixed summary reads `false` here; read
    /// [`Checkbox::toggle_state`] for the tri-state truth.
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        matches!(self.state, ToggleState::On)
    }

    /// Whether the box holds the mixed summary (test introspection).
    #[must_use]
    pub const fn is_mixed(&self) -> bool {
        matches!(self.state, ToggleState::Mixed)
    }

    /// The full tri-state the box carries (test introspection, M94
    /// ADR-0098): the vocabulary the a11y tree announces.
    #[must_use]
    pub const fn toggle_state(&self) -> ToggleState {
        self.state
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

    /// Sets the checked state in place (view reconciliation). Fires no
    /// handler — programmatic state, not a user toggle. The tri-state
    /// summary rides [`Checkbox::set_state`].
    pub fn set_checked(&mut self, checked: bool) {
        self.state = ToggleState::from(checked);
    }

    /// Sets the full tri-state in place (view reconciliation, M94
    /// ADR-0098): the app's select-all write lands the summary the tree
    /// announces. Fires no handler — programmatic state, never a user
    /// door, so the resolution rule does not apply.
    pub fn set_state(&mut self, state: ToggleState) {
        self.state = state;
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

    /// The one toggle path: resolve the state, fire the handler with the
    /// new checked flag, request the masks the change costs, and report
    /// whether the toggle *landed* — `false` when the availability wall
    /// held (an unavailable checkbox toggles nothing) or when the state
    /// the ask resolves to is the state it already was, so the record the
    /// keyboard's route streams answers the door it actually answered
    /// (the funnel's own truth at the action route: a refused ask is a
    /// refusal, never a munged `Delivered`). The availability tier guards
    /// the funnel's single entry (M22, ADR-0028); the resolution rule
    /// (M94, ADR-0098) resolves a mixed summary fully checked — the
    /// user's first press lands "all".
    fn toggle(&mut self, ctx: &mut EventCtx) -> bool {
        if !self.enabled {
            return false;
        }
        self.state = match self.state {
            ToggleState::Off | ToggleState::Mixed => ToggleState::On,
            ToggleState::On => ToggleState::Off,
        };
        ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        let resolved = self.is_checked();
        if let Some(handler) = &mut self.on_toggle {
            handler(resolved);
        }
        true
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

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the checkbox has nothing to explain.
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
        Some(self.state)
    }

    /// The availability tier (M22, ADR-0028): the flag the a11y tree
    /// carries — an unavailable checkbox reports itself before the AT
    /// tries the toggle it will refuse.
    fn enabled(&self) -> bool {
        self.enabled
    }

    /// The required tier (M106, ADR-0110): the word the tree says
    /// beside the form contract — "this one must be answered", the AT
    /// reading it before it asks. The doors answer unchanged beside
    /// it.
    fn required(&self) -> bool {
        self.required
    }

    /// The invalid tier (M107, ADR-0111): the word the tree says
    /// beside the form's rejection — "this one is not valid as given",
    /// the kind beside it, the AT reading it while the user edits. The
    /// doors answer unchanged beside it.
    fn invalid(&self) -> Option<alacer_core::InvalidKind> {
        self.invalid
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let size = ctx.size();
        let box_side = 18.0_f32.min(size.height);
        let box_top = (size.height - box_side) * 0.5;
        // The borrowed roles resolve per paint pass (ADR-0079); the
        // look still tells the truth the tree carries (M22, ADR-0028):
        // an unavailable checkbox paints the muted rail and the muted
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

        // The track: an outlined square; filled with the accent whenever
        // the box carries a state (the mixed summary fills like the
        // checked pair — the dash tells the two apart, M94 ADR-0098).
        ctx.push_quad(alacer_core::scene::Quad {
            rect: alacer_core::geom::Rect::new(
                Point::new(0.0, box_top),
                alacer_core::geom::Size::new(box_side, box_side),
            ),
            color: if self.state == ToggleState::Off {
                palette.panel
            } else {
                accent
            },
            corner_radius: 4.0,
            border_color: if self.focused { accent } else { palette.border },
            border_width: if self.focused { 2.0 } else { 1.5 },
            clip: None,
        });

        // The check mark: two accent strokes drawn as quads — deterministic
        // pixels with no glyph dependency (snapshot corpus stays font-free).
        // The mixed summary draws the horizontal dash instead: the same
        // quad discipline, the summary's own truth in pixels.
        if self.state == ToggleState::On {
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
        } else if self.state == ToggleState::Mixed {
            let stroke = 2.5;
            let dash_width = box_side * 0.5;
            let dash = alacer_core::geom::Rect::new(
                Point::new(
                    (box_side - dash_width) * 0.5,
                    box_top + (box_side - stroke) * 0.5,
                ),
                alacer_core::geom::Size::new(dash_width, stroke),
            );
            ctx.push_quad(alacer_core::scene::Quad::filled(dash, Color::WHITE));
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
        // The availability tier (M22, ADR-0028): an unavailable checkbox
        // has no write door — the toggle is refused honestly, never
        // munged (the M11 rule).
        if !self.enabled {
            return false;
        }
        if action == A11yAction::DefaultPress {
            // The a11y path has no EventCtx; the dispatch funnel queues
            // `PAINT | A11Y` itself when the door consumes — the toggle
            // itself is handler-first, exactly like Button's press path.
            // The resolution rule (M94, ADR-0098): the AT's press
            // resolves a mixed summary fully checked, the same answer
            // the pointer's and keyboard's doors give.
            self.state = match self.state {
                ToggleState::Off | ToggleState::Mixed => ToggleState::On,
                ToggleState::On => ToggleState::Off,
            };
            let resolved = self.is_checked();
            if let Some(handler) = &mut self.on_toggle {
                handler(resolved);
            }
            return true;
        }
        false
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        match event {
            PointerEvent::Down { .. } => {
                // The wall (M22, ADR-0028): a disabled checkbox consumes
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
            // toggle-on-press ride — the focused checkbox answering
            // Space like a click — streams one `ActionRecord` with
            // `Keyboard` beside it, the attribution declared where the
            // route is known (the M30 rider rule at the activation
            // tier). The record answers the door it actually answered:
            // a toggle that landed is `Delivered`; one the availability
            // wall held is the refusal the funnel's own route computes
            // from the tree's one `enabled` truth (M22, ADR-0028) —
            // never a munged `Delivered`. The stamp rides the key
            // route's guard (M45, ADR-0051) — the focused control's id,
            // never guessed at the note site. One branch when nobody
            // listens (the A7 idle contract).
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
    use super::Checkbox;
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
            click_count: 1,
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

    #[test]
    fn the_mixed_summary_carries_and_resolves_by_every_door() {
        // M94, ADR-0098: the summary mounts, the tree announces it, and
        // every user door resolves it fully checked — never a third bit.
        let mut checkbox = Checkbox::new("Select all").state(ToggleState::Mixed);
        assert_eq!(checkbox.toggle_state(), ToggleState::Mixed);
        assert!(checkbox.is_mixed());
        assert!(!checkbox.is_checked(), "the summary is not the bool pair");

        // The AT's press resolves.
        assert!(checkbox.on_a11y_action(A11yAction::DefaultPress));
        assert_eq!(
            checkbox.toggle_state(),
            ToggleState::On,
            "the AT's press lands fully checked"
        );

        // A fresh summary resolves through the pointer's click.
        let mut checkbox = Checkbox::new("Select all").state(ToggleState::Mixed);
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
        let _ = checkbox.on_pointer_event(&mut ctx, &down);
        let _ = checkbox.on_pointer_event(&mut ctx, &up);
        assert_eq!(checkbox.toggle_state(), ToggleState::On);

        // And the keyboard's Space resolves it too.
        let mut checkbox = Checkbox::new("Select all").state(ToggleState::Mixed);
        let mut ctx = EventCtx::new();
        let _ = checkbox.on_key_event(&mut ctx, &space());
        assert_eq!(checkbox.toggle_state(), ToggleState::On);
    }

    #[test]
    fn the_resolution_fires_the_handler_with_the_resolved_bool() {
        let seen = std::rc::Rc::new(std::cell::Cell::new(None));
        let sink = std::rc::Rc::clone(&seen);
        let mut checkbox = Checkbox::new("Select all")
            .state(ToggleState::Mixed)
            .on_toggle(move |checked| sink.set(Some(checked)));
        let mut ctx = EventCtx::new();
        let _ = checkbox.on_key_event(&mut ctx, &space());
        assert_eq!(
            seen.get(),
            Some(true),
            "the handler saw the resolved boolean, never a third bit"
        );
    }

    #[test]
    fn programmatic_set_state_writes_the_summary_without_a_door() {
        let mut checkbox = Checkbox::new("Select all").on_toggle(|_| panic!("no user door ran"));
        checkbox.set_state(ToggleState::Mixed);
        assert_eq!(checkbox.toggle_state(), ToggleState::Mixed);
        assert!(
            checkbox.is_mixed(),
            "the programmatic write lands the summary verbatim"
        );
        checkbox.set_checked(true);
        assert_eq!(checkbox.toggle_state(), ToggleState::On);
        checkbox.set_checked(false);
        assert_eq!(
            checkbox.toggle_state(),
            ToggleState::Off,
            "the binary door keeps its own contract"
        );
    }

    #[test]
    fn a_disabled_checkbox_refuses_every_toggle_door() {
        // M22, ADR-0028: the AT tier is refused, the pointer arms nothing,
        // Space flips nothing, and the write that moved the flag is the
        // only mask the tier costs.
        let mut checkbox = Checkbox::new("Done").enabled(false).on_toggle(|_| {
            panic!("an unavailable checkbox must not toggle");
        });
        let mut ctx = EventCtx::new();

        assert!(
            !checkbox.on_a11y_action(A11yAction::DefaultPress),
            "an unavailable toggle has no write door"
        );
        assert_eq!(
            checkbox.on_pointer_event(
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
            checkbox.on_key_event(&mut ctx, &space()),
            EventResponse::Handled,
            "Space is an activation key, so the event is handled…"
        );
        assert!(!checkbox.is_checked(), "…but the toggle wrote nothing");
        assert!(
            !checkbox.focusable(),
            "an unavailable control holds no focus"
        );

        // The write answers the move (the M21 idle-frame property).
        assert!(!checkbox.set_enabled(false), "already unavailable");
        assert!(checkbox.set_enabled(true), "the write moved");
        assert!(checkbox.is_enabled());
    }

    #[test]
    fn the_description_rides_the_trait_and_the_write_reports_the_move() {
        // M95, ADR-0099: the honest absence by default, the help text the
        // builder mounts, and the one write whose moved answer is the
        // only mask the tier costs.
        let mut checkbox = Checkbox::new("Ship in bulk");
        assert_eq!(
            Widget::description(&checkbox),
            None,
            "an undescribed checkbox announces the absence"
        );
        checkbox = checkbox.description("Marks every open order");
        assert_eq!(
            Widget::description(&checkbox),
            Some("Marks every open order"),
            "the help text the tree carries beside the name"
        );

        // The write that landed where the text already is, then the write
        // that moved (the M21 idle-frame property, at the description
        // tier).
        assert!(
            !checkbox.set_description(Some("Marks every open order".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(
            checkbox.set_description(Some("Marks every *sealed* order".to_string())),
            "the moved write answers true"
        );
        assert_eq!(
            Widget::description(&checkbox),
            Some("Marks every *sealed* order")
        );
        assert!(
            checkbox.set_description(None),
            "clearing to the absence moved"
        );
        assert_eq!(Widget::description(&checkbox), None);
        assert!(
            !checkbox.set_description(None),
            "the absence is already there"
        );
    }
}
