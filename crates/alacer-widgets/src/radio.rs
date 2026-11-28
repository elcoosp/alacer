//! The radio tier (M124, ADR-0128): the one-of-N control and the named
//! group it reads as.
//!
//! Exclusive choice was a vocabulary hole: an app wiring "one of these,
//! not many" reached for a checkbox — the wrong verb (a checkbox
//! toggles itself off, a radio never deselects itself) — or for a bare
//! button wearing no selection fact at all. The radio answers the M6
//! toggle contract's *selection* half: [`Role::Radio`] carrying the
//! checked state the a11y tree already announces for the checkbox
//! ([`ToggleState`], never the mixed summary — a radio has no third
//! bit), [`Role::RadioGroup`] the named container the set reads as,
//! and the set tier's own words (ADR-0123) answering "which one, of
//! how many" — [`Radio::position_in_set`] on the member,
//! [`RadioGroup::size_of_set`] on the group — without a second tier.
//!
//! Three doors, one selection, one contract: a click, the Space key
//! while focused, and the AccessKit default action all *select* — the
//! checked member never deselects itself. A redundant ask (the second
//! click, the second Space, the AT's ask on the selected member) is
//! consumed and moves nothing: the widget's own doors request no masks
//! and fire no handler — the M21 idle-frame property, at the widget
//! seam. The a11y door answers `true` for every select ask on an
//! available member — the ask is the radio's own tier — and the
//! re-announcement it costs rides the dispatch funnel's coarse
//! `consumed` contract (the funnel requests `PAINT | A11Y` for every
//! consumed ask; a redundant select's delta is empty): the one coarse
//! corner, named here where the consumer can read it. Exclusivity —
//! the sibling that deselects when this one selects — is the app's
//! wiring: the [`Radio::on_select`] handler is where the set's writer
//! clears the others (the framework never rewrites widgets it does not
//! own, A1). The one deselect door is the programmatic write
//! ([`Radio::set_checked`], view reconciliation) — programmatic state,
//! never a user door, the checkbox's own rule.
//!
//! The widget never writes a11y directly (A1: masks are the only
//! currency), and the keyboard's activation names itself (M70,
//! ADR-0076): the Space-select streams one `ActionRecord` with
//! `Keyboard` beside it — answered only when the selection moved, the
//! redundant ask streaming nothing (one fact per trail, the M39 rule
//! at the activation tier).

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, KeyEvent, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, ToggleState, Widget};

/// A single-choice control with a mandatory name (ADR-5: an unnamed
/// radio is a bare mark) and the selection the a11y tree announces —
/// the checkbox contract's selection half, at the one-of-N tier.
pub struct Radio {
    name: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    checked: bool,
    enabled: bool,
    /// The member's place in the set (M119, ADR-0123); `None` the
    /// honest absence — a member that declared no place announces none.
    position_in_set: Option<usize>,
    on_select: Option<Box<dyn FnMut(bool)>>,
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

impl Radio {
    /// A radio labeled `name`, initially unchecked.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            checked: false,
            enabled: true,
            position_in_set: None,
            on_select: None,
            focused: false,
            accent: None,
            text_color: None,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// Initial availability (M22, ADR-0028). An unavailable radio
    /// paints muted, holds no focus, and refuses every select door
    /// honestly — the AT reads the refusal before it happens.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// The member's place in the set (M119, ADR-0123): the word the AT
    /// reads as "2 of 3" beside the group's span. One-based, the ARIA
    /// convention the tier's documentation carries.
    #[must_use]
    pub fn position_in_set(mut self, position_in_set: usize) -> Self {
        self.position_in_set = Some(position_in_set);
        self
    }

    /// Initial checked state: `true` mounts the member as the set's
    /// selection.
    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Installs the selection handler; invoked with `true` after the
    /// member *became* the selection (click, Space, a11y action). A
    /// redundant ask moves nothing and fires nothing — the idle-frame
    /// property at the handler seam too.
    #[must_use]
    pub fn on_select(mut self, handler: impl FnMut(bool) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// The description (M95, ADR-0099): the help text beside the name.
    /// Without it the radio announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Accent color of the ring and the selection dot. Without it the
    /// radio borrows the theme's accent role at paint time (ADR-0079).
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

    /// Current checked state (test introspection).
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        self.checked
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

    /// The one deselect door: sets the checked state in place (view
    /// reconciliation). Fires no handler — programmatic state, never a
    /// user door. The *user's* doors select and never deselect; this
    /// write is how the app's wiring clears a member when another
    /// becomes the selection.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// The one place write (view reconciliation, the set tier): `None`
    /// clears to the honest absence. Report whether the place moved —
    /// the caller requests `A11Y` only when this answers `true`, so a
    /// write that lands where the place already is queues nothing (the
    /// M21 idle-frame property, at the set tier).
    pub fn set_position_in_set(&mut self, position_in_set: Option<usize>) -> bool {
        if self.position_in_set == position_in_set {
            return false;
        }
        self.position_in_set = position_in_set;
        true
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

    /// Replaces the selection handler in place (view reconciliation).
    /// `None` detaches it.
    pub fn set_on_select(&mut self, handler: Option<Box<dyn FnMut(bool)>>) {
        self.on_select = handler;
    }

    /// The one select path: a checked member is already the selection —
    /// the ask is consumed and moves nothing (`false`, no masks, no
    /// handler: the M21 idle frame); an unchecked member becomes the
    /// selection (`true`, the caller requests the masks the move costs,
    /// the handler seeing the new state). The availability tier guards
    /// the funnel's single entry (M22, ADR-0028): an unavailable radio
    /// refuses, never munges (the M11 rule).
    fn select(&mut self, ctx: Option<&mut EventCtx>) -> bool {
        if !self.enabled || self.checked {
            return false;
        }
        self.checked = true;
        if let Some(ctx) = ctx {
            ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        }
        if let Some(handler) = &mut self.on_select {
            handler(true);
        }
        true
    }

    /// Intrinsic size: the 18px ring, a gap, and the label.
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

impl Widget for Radio {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Radio
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the radio has nothing to explain.
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
        Some(ToggleState::from(self.checked))
    }

    /// The availability tier (M22, ADR-0028): the flag the a11y tree
    /// carries — an unavailable radio reports itself before the AT
    /// tries the select it will refuse.
    fn enabled(&self) -> bool {
        self.enabled
    }

    /// The set tier's member word (M119, ADR-0123): the place the AT
    /// reads beside the group's span — "2 of 3", never a guessed count.
    fn position_in_set(&self) -> Option<usize> {
        self.position_in_set
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let size = ctx.size();
        let ring_side = 18.0_f32.min(size.height);
        let ring_top = (size.height - ring_side) * 0.5;
        // The borrowed roles resolve per paint pass (ADR-0079); the
        // look still tells the truth the tree carries (M22, ADR-0028):
        // an unavailable radio paints the muted ring and the muted
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

        // The ring: a round outline; filled with the accent whenever
        // the member carries the selection (a radio has no mixed
        // summary — the dot is the whole story, ADR-0128).
        ctx.push_quad(alacer_core::scene::Quad {
            rect: alacer_core::geom::Rect::new(
                Point::new(0.0, ring_top),
                alacer_core::geom::Size::new(ring_side, ring_side),
            ),
            color: if self.checked { accent } else { palette.panel },
            corner_radius: ring_side * 0.5,
            border_color: if self.focused { accent } else { palette.border },
            border_width: if self.focused { 2.0 } else { 1.5 },
            clip: None,
        });

        // The selection dot: a white disc inside the filled ring —
        // deterministic pixels with no glyph dependency (the snapshot
        // corpus stays font-free).
        if self.checked {
            let dot_side = ring_side * 0.42;
            let dot = alacer_core::geom::Rect::new(
                Point::new(
                    (ring_side - dot_side) * 0.5,
                    ring_top + (ring_side - dot_side) * 0.5,
                ),
                alacer_core::geom::Size::new(dot_side, dot_side),
            );
            ctx.push_quad(alacer_core::scene::Quad::filled(dot, Color::WHITE));
        }

        // The label, right of the ring.
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.name, self.font_size, None);
            let text_origin = Point::new(
                ring_side + 8.0,
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

    /// The action tier (M156, ADR-0161): the select the door answers,
    /// said while the radio is available — the word is the door's own
    /// answer, so a closed door (the M22 availability tier) advertises
    /// none.
    fn a11y_actions(&self) -> alacer_core::widget::A11yActions {
        if self.enabled() {
            alacer_core::widget::A11yActions::DEFAULT_PRESS
        } else {
            alacer_core::widget::A11yActions::empty()
        }
    }

    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        // The availability tier (M22, ADR-0028): an unavailable radio
        // has no write door — the select is refused honestly, never
        // munged (the M11 rule).
        if !self.enabled {
            return false;
        }
        if action == A11yAction::DefaultPress {
            // The a11y path has no EventCtx; the dispatch funnel queues
            // `PAINT | A11Y` itself when the door consumes — the door
            // answers every select ask on an available member (the ask
            // is the radio's own tier), the redundant one landing where
            // the selection already was: the one coarse corner the
            // funnel's `consumed` contract pays, named at the module
            // docs. The select itself is handler-first, exactly like
            // Button's press path.
            let _ = self.select(None);
            return true;
        }
        false
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        match event {
            PointerEvent::Down { .. } => {
                // The wall (M22, ADR-0028): a disabled radio consumes
                // the press but requests nothing — no focus, no repaint.
                if !self.enabled {
                    return EventResponse::Handled;
                }
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::FOCUS));
                EventResponse::Handled
            }
            PointerEvent::Up { .. } => {
                // A redundant ask is consumed and moves nothing — no
                // masks, no handler: the idle frame (the M21 property
                // at the widget's own doors).
                let _ = self.select(Some(ctx));
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
            // The keyboard activation names itself (M70, ADR-0076):
            // the select-on-press ride — the focused radio answering
            // Space like a click — streams one `ActionRecord` with
            // `Keyboard` beside it, *answered only when the selection
            // moved*: a redundant ask moves nothing and streams
            // nothing (one fact per trail, the M39 rule at the
            // activation tier; the route the funnel's own record does
            // not munge). The stamp rides the key route's guard (M45,
            // ADR-0051) — the focused control's id, never guessed at
            // the note site. One branch when nobody listens (the A7
            // idle contract).
            let moved = self.select(Some(ctx));
            if moved {
                alacer_core::action::note_action(alacer_core::ActionRecord {
                    ask: alacer_core::inspect::ActionAsk::Press,
                    target: None,
                    outcome: alacer_core::inspect::ActionOutcome::Delivered,
                    rider: alacer_core::inspect::ActionRider::Keyboard,
                });
            }
            return EventResponse::Handled;
        }
        EventResponse::Ignored
    }
}

/// The named container a set of radios reads as (the radio tier's
/// container half, M124, ADR-0128): the group word the AT announces
/// before it walks the members, the set tier's span word
/// ([`RadioGroup::size_of_set`]) its natural carrier. Children ride the
/// arena exactly as a [`crate::Column`]'s do — the group is the
/// members' layout (a vertical stack), never a second channel to them.
pub struct RadioGroup {
    name: String,
    /// The group's declared span (M119, ADR-0123); `None` the honest
    /// absence — a group that declared no size announces none.
    size_of_set: Option<usize>,
    style: Style,
}

impl RadioGroup {
    /// A radio group labeled `name` — an unnamed group reads as a bare
    /// box, the set the AT cannot name is a set it cannot walk (ADR-5).
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size_of_set: None,
            style: Style::column(),
        }
    }

    /// The group's declared span: the word the AT reads as "of 3"
    /// beside the members' places. One-based places, the ARIA
    /// convention the tier's documentation carries.
    #[must_use]
    pub fn size_of_set(mut self, size_of_set: usize) -> Self {
        self.size_of_set = Some(size_of_set);
        self
    }

    /// Gap between the members.
    #[must_use]
    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = gap;
        self
    }

    /// Uniform padding.
    #[must_use]
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding = padding;
        self
    }

    /// Grow factor.
    #[must_use]
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.grow = grow;
        self
    }

    /// Fixed height.
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.style.size.height = Some(height);
        self
    }

    /// Fixed width.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.style.size.width = Some(width);
        self
    }

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl RadioGroup {
    /// Shared-borrow layout style access for view reconciliation (§6.1):
    /// patches write styles in place instead of rebuilding the widget.
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    /// Replaces the group's accessible name in place (view
    /// reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// The one span write (view reconciliation): `None` clears to the
    /// honest absence. Report whether the span moved — the caller
    /// requests `A11Y` only when this answers `true`, so a write that
    /// lands where the span already is queues nothing (the M21
    /// idle-frame property, at the set tier).
    pub fn set_size_of_set(&mut self, size_of_set: Option<usize>) -> bool {
        if self.size_of_set == size_of_set {
            return false;
        }
        self.size_of_set = size_of_set;
        true
    }
}

impl Widget for RadioGroup {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::RadioGroup
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// The set tier's container word (M119, ADR-0123): the span the AT
    /// reads beside the members' places — "of 3", never a guessed one.
    fn size_of_set(&self) -> Option<usize> {
        self.size_of_set
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::{Radio, RadioGroup};
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

    fn click(ctx: &mut EventCtx, radio: &mut Radio) {
        let down = PointerEvent::Down {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 1,
        };
        let up = PointerEvent::Up {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        assert_eq!(radio.on_pointer_event(ctx, &down), EventResponse::Handled);
        assert_eq!(radio.on_pointer_event(ctx, &up), EventResponse::Handled);
    }

    #[test]
    fn role_name_and_selection_state_satisfy_the_contract() {
        let radio = Radio::new("Standard");
        assert_eq!(radio.role(), Role::Radio);
        assert_eq!(radio.label().resolve(), "Standard");
        assert!(radio.focusable(), "radios hold focus");
        assert!(radio.role().requires_name());
        assert_eq!(radio.toggled(), Some(ToggleState::Off));
        let group = RadioGroup::new("Shipping");
        assert_eq!(group.role(), Role::RadioGroup);
        assert_eq!(group.label().resolve(), "Shipping");
        assert!(group.role().requires_name());
        assert_eq!(
            Widget::size_of_set(&group),
            None,
            "the honest absence by default"
        );
    }

    #[test]
    fn click_selects_and_requests_paint_plus_a11y() {
        let mut radio = Radio::new("Standard");
        let mut ctx = EventCtx::new();
        click(&mut ctx, &mut radio);
        assert!(radio.is_checked(), "click selects");
        let masks = ctx.take();
        assert!(
            masks.contains(Change::PAINT) && masks.contains(Change::A11Y),
            "the selection repaints and re-announces, got {masks:?}"
        );
    }

    #[test]
    fn a_second_click_is_consumed_and_idle() {
        // The M21 property at the radio's own doors: the redundant ask
        // moves nothing, fires nothing — and the *select* half of the
        // click queues nothing. The press-down still arms focus (the
        // click aims focus at a focusable member — the FOCUS currency's
        // own honest work, the checkbox's shape), so the full click's
        // masks are the arm's, never the select's.
        let mut radio = Radio::new("Standard")
            .checked(true)
            .on_select(|_| panic!("a redundant ask fires no handler"));
        let mut ctx = EventCtx::new();
        click(&mut ctx, &mut radio);
        assert!(radio.is_checked(), "a radio never deselects itself");
        let masks = ctx.take();
        assert!(
            !masks.contains(Change::A11Y),
            "a redundant click re-announces nothing — got {masks:?}"
        );

        // And the second Space is the true idle frame: no arm, no
        // select, no masks at all.
        let mut ctx = EventCtx::new();
        assert_eq!(
            radio.on_key_event(&mut ctx, &space()),
            EventResponse::Handled
        );
        assert!(radio.is_checked());
        assert!(
            ctx.take().is_empty(),
            "a redundant Space queues no masks — an idle frame"
        );
    }

    #[test]
    fn space_selects_while_the_handler_sees_the_selection() {
        let seen = std::rc::Rc::new(std::cell::Cell::new(false));
        let sink = std::rc::Rc::clone(&seen);
        let mut radio = Radio::new("Standard").on_select(move |selected| sink.set(selected));
        let mut ctx = EventCtx::new();
        assert_eq!(
            radio.on_key_event(&mut ctx, &space()),
            EventResponse::Handled
        );
        assert!(radio.is_checked());
        assert!(seen.get(), "the handler observed the selection");
        let masks = ctx.take();
        assert!(masks.contains(Change::A11Y), "the selection re-announces");
    }

    #[test]
    fn other_keys_are_ignored() {
        let mut radio = Radio::new("Standard");
        let mut ctx = EventCtx::new();
        let enter = KeyEvent {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        };
        assert_eq!(radio.on_key_event(&mut ctx, &enter), EventResponse::Ignored);
        assert!(!radio.is_checked());
    }

    #[test]
    fn the_a11y_default_action_selects_and_the_redundant_ask_is_consumed() {
        let mut radio = Radio::new("Standard");
        assert!(radio.on_a11y_action(A11yAction::DefaultPress));
        assert!(radio.is_checked());
        assert!(
            matches!(radio.toggled(), Some(ToggleState::On)),
            "the a11y tree would announce checked"
        );
        // The redundant ask: consumed (the door answers the select tier),
        // the selection unmoved.
        assert!(radio.on_a11y_action(A11yAction::DefaultPress));
        assert!(radio.is_checked(), "a radio never deselects itself");
    }

    #[test]
    fn programmatic_set_checked_fires_no_handler() {
        let mut radio = Radio::new("Standard").on_select(|_| panic!("no user select"));
        radio.set_checked(true);
        assert!(radio.is_checked(), "the programmatic write lands");
        // The one deselect door: programmatic state, never a user door.
        radio.set_checked(false);
        assert!(!radio.is_checked());
    }

    #[test]
    fn the_set_words_carry_and_the_write_reports_the_move() {
        // M119's words at the radio tier: the member's place and the
        // group's span, the honest absences by default.
        let mut radio = Radio::new("Standard");
        assert_eq!(
            Widget::position_in_set(&radio),
            None,
            "a placeless member announces no place"
        );
        radio = radio.position_in_set(2);
        assert_eq!(Widget::position_in_set(&radio), Some(2));
        assert!(
            !radio.set_position_in_set(Some(2)),
            "the write that lands where the place already is queues nothing"
        );
        assert!(radio.set_position_in_set(None), "the clear moved");
        assert_eq!(
            Widget::position_in_set(&radio),
            None,
            "the absence restored — the set tier's honest answer"
        );

        let mut group = RadioGroup::new("Shipping");
        assert!(!group.set_size_of_set(None), "the absence is already there");
        assert!(group.set_size_of_set(Some(3)), "the span moved");
        assert_eq!(Widget::size_of_set(&group), Some(3));
        assert!(
            !group.set_size_of_set(Some(3)),
            "the write that lands where the span already is queues nothing"
        );
        assert!(group.set_size_of_set(None), "clearing to the absence moved");
    }

    #[test]
    fn a_disabled_radio_refuses_every_select_door() {
        // M22, ADR-0028: the AT tier is refused, the pointer arms nothing,
        // Space flips nothing, and the write that moved the flag is the
        // only mask the tier costs.
        let mut radio = Radio::new("Fax").enabled(false).on_select(|_| {
            panic!("an unavailable radio must not select");
        });
        let mut ctx = EventCtx::new();

        assert!(
            !radio.on_a11y_action(A11yAction::DefaultPress),
            "an unavailable select has no write door"
        );
        assert_eq!(
            radio.on_pointer_event(
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
            radio.on_key_event(&mut ctx, &space()),
            EventResponse::Handled,
            "Space is an activation key, so the event is handled…"
        );
        assert!(!radio.is_checked(), "…but the select wrote nothing");
        assert!(!radio.focusable(), "an unavailable control holds no focus");

        // The write answers the move (the M21 idle-frame property).
        assert!(!radio.set_enabled(false), "already unavailable");
        assert!(radio.set_enabled(true), "the write moved");
        assert!(radio.is_enabled());
    }

    #[test]
    fn the_description_rides_the_trait_and_the_write_reports_the_move() {
        // M95, ADR-0099: the honest absence by default, the help text the
        // builder mounts, and the one write whose moved answer is the
        // only mask the tier costs.
        let mut radio = Radio::new("Ship in bulk");
        assert_eq!(
            Widget::description(&radio),
            None,
            "an undescribed radio announces the absence"
        );
        radio = radio.description("The set's slowest member");
        assert_eq!(
            Widget::description(&radio),
            Some("The set's slowest member"),
            "the help text the tree carries beside the name"
        );
        assert!(
            !radio.set_description(Some("The set's slowest member".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(radio.set_description(None), "clearing to the absence moved");
        assert_eq!(Widget::description(&radio), None);
    }

    #[test]
    fn the_group_is_the_members_layout_and_the_set_words_ride_their_carriers() {
        // The group stacks vertically (the members' layout, never a
        // second channel), carries no selection fact of its own, and
        // holds the span word the members' places answer against.
        let mut group = RadioGroup::new("Shipping").size_of_set(3).gap(6.0);
        assert_eq!(group.style().gap, 6.0, "the gap rides the layout style");
        assert_eq!(
            group.style().flex_direction,
            alacer_core::style::FlexDirection::Column,
            "the group stacks its members — the column canon's own direction"
        );
        assert_eq!(
            group.toggled(),
            None,
            "a container carries no selection fact"
        );
        group.set_name("Delivery");
        assert_eq!(group.label().resolve(), "Delivery");
    }
}
