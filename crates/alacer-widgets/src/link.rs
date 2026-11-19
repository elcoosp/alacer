//! The link tier (M125, ADR-0129): the hyperlink control and its
//! activation verb.
//!
//! "This goes somewhere" was a vocabulary hole: an app wiring a
//! hyperlink reached for a button — the wrong verb (a button's
//! activation is a *press*, a link's is an *open*, and the AT's
//! vocabulary names the difference: AccessKit's `Link` role is its own
//! fact, not a button wearing styled text) — or for styled text
//! wearing no activation fact at all. [`Role::Link`] carries the
//! "opens something" fact, and the [`Link`] control answers the
//! activation door with the open act the app's navigation rides
//! ([`Link::on_open`]) — the destination is the app's: the tree says
//! "opens", never where, the name and the description carrying the
//! words the app declared.
//!
//! The keyboard's verb is the link's own: **Enter** opens (the
//! platform convention every document vocabulary rides), and Space is
//! handed on — the scroller's key, the framework's to take (the M10
//! parity at the link tier: the activation keys are the control's own,
//! not every key's). A click opens on release (the button's GTK rule:
//! an auto-repeat hold cannot machine-gun the navigation). The AT's
//! default action is the open ask the door answers. The availability
//! tier guards every door (M22, ADR-0028): an unavailable link
//! refuses, never munges (the M11 rule).
//!
//! The keyboard's activation names itself (M70, ADR-0076): the
//! Enter-open streams one `ActionRecord` with `Keyboard` beside it —
//! the attribution declared where the route is known. The widget never
//! writes a11y directly (A1: masks are the only currency).

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, KeyEvent, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, Widget};

/// A hyperlink control with a mandatory name (ADR-5: an unnamed link
/// reads as bare styled text — the "goes somewhere" the AT cannot
/// announce) and the open act the app's navigation rides.
pub struct Link {
    name: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    enabled: bool,
    on_open: Option<Box<dyn FnMut()>>,
    focused: bool,
    /// The app's text-color override; `None` borrows the theme's
    /// accent role at paint time (ADR-0079) — a link reads as the
    /// accent-colored text the vocabulary already speaks.
    text_color: Option<Color>,
    font_size: f32,
    style: Style,
}

impl Link {
    /// A link labeled `name`.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            enabled: true,
            on_open: None,
            focused: false,
            text_color: None,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// Initial availability (M22, ADR-0028). An unavailable link
    /// paints muted, holds no focus, and refuses every open door
    /// honestly — the AT reads the refusal before it happens.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Installs the open handler; invoked after every activation
    /// (click, Enter, a11y action) — the destination the app's
    /// navigation rides.
    #[must_use]
    pub fn on_open(mut self, handler: impl FnMut() + 'static) -> Self {
        self.on_open = Some(Box::new(handler));
        self
    }

    /// The description (M95, ADR-0099): the help text beside the name
    /// — the destination the name is too short to carry. Without it the
    /// link announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Text color of the label. Without it the label borrows the
    /// theme's accent role at paint time (ADR-0079) — the
    /// link-colored text the vocabulary already speaks.
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

    /// Replaces the open handler in place (view reconciliation). `None`
    /// detaches it.
    pub fn set_on_open(&mut self, handler: Option<Box<dyn FnMut()>>) {
        self.on_open = handler;
    }

    /// The one open path: fire the handler and request the masks the
    /// activation costs. The availability tier guards the funnel's
    /// single entry (M22, ADR-0028): an unavailable link refuses, never
    /// munges (the M11 rule).
    fn open(&mut self, ctx: Option<&mut EventCtx>) {
        if !self.enabled {
            return;
        }
        if let Some(ctx) = ctx {
            ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        }
        if let Some(handler) = &mut self.on_open {
            handler();
        }
    }

    /// Intrinsic size: the label the link is — the accent-colored text
    /// the underline's absence keeps font-free (the snapshot corpus's
    /// own discipline), the activation fact the tree carries.
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
        alacer_core::geom::Size::new(text.width + 4.0, text.height.max(18.0) + 4.0)
    }
}

impl Widget for Link {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Link
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the link has nothing to explain.
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn style(&self) -> Style {
        Style {
            padding: 2.0,
            ..self.style.clone()
        }
    }

    /// The availability tier (M22, ADR-0028): the flag the a11y tree
    /// carries — an unavailable link reports itself before the AT
    /// tries the open it will refuse.
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let size = ctx.size();
        // The borrowed roles resolve per paint pass (ADR-0079); the
        // look still tells the truth the tree carries (M22, ADR-0028):
        // an unavailable link paints the muted text role — the link
        // that opens nothing reads as the muted text it is. A focused
        // link paints the accent underline the keyboard walk lands on
        // — the focus indicator the glyphs sit on, deterministic
        // pixels with no glyph dependency (the snapshot corpus stays
        // font-free).
        let palette = ctx.palette();
        let label_color = if self.enabled {
            self.text_color.unwrap_or(palette.accent)
        } else {
            palette.text_muted
        };

        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.name, self.font_size, None);
            let text_origin = Point::new(2.0, (size.height - layout.height).max(0.0) * 0.5);
            // The focus indicator (the M125 paint's own promise, kept
            // where it was left unkept): a focused link's underline in
            // the accent the link already speaks — the keyboard walk's
            // landmark, priced once per focused frame and never on the
            // idle one.
            if self.focused && self.enabled {
                ctx.push_quad(alacer_core::scene::Quad::filled(
                    alacer_core::geom::Rect::new(
                        Point::new(0.0, size.height - 2.0),
                        alacer_core::geom::Size::new(layout.width.max(size.width), 2.0),
                    ),
                    self.text_color.unwrap_or(palette.accent),
                ));
            }
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
        // The availability tier (M22, ADR-0028): an unavailable link
        // has no write door — the open is refused honestly, never
        // munged (the M11 rule).
        if !self.enabled {
            return false;
        }
        if action == A11yAction::DefaultPress {
            // The a11y path has no EventCtx; the dispatch funnel queues
            // `PAINT | A11Y` itself when the door consumes — the open
            // itself is handler-first, exactly like Button's press path.
            self.open(None);
            return true;
        }
        false
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        match event {
            PointerEvent::Down { .. } => {
                // The wall (M22, ADR-0028): a disabled link consumes
                // the press but requests nothing — no focus, no repaint.
                if !self.enabled {
                    return EventResponse::Handled;
                }
                ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::FOCUS));
                EventResponse::Handled
            }
            PointerEvent::Up { .. } => {
                // The button's GTK rule (M6): the open fires on release,
                // so an auto-repeat hold cannot machine-gun the
                // navigation.
                self.open(Some(ctx));
                EventResponse::Handled
            }
            PointerEvent::Move { .. } | PointerEvent::Wheel { .. } | PointerEvent::Leave => {
                EventResponse::Ignored
            }
        }
    }

    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        // The keyboard's verb is the link's own: Enter opens (the
        // platform convention every document vocabulary rides), and
        // Space is handed on — the scroller's key, the framework's to
        // take (the M10 parity at the link tier: the activation keys
        // are the control's own, not every key's).
        if event.state == alacer_core::events::KeyState::Pressed
            && event.key == alacer_core::events::Key::Enter
        {
            if self.enabled {
                self.open(Some(ctx));
                // The keyboard activation names itself (M70, ADR-0076):
                // the open-on-press ride — the focused link answering
                // Enter like a click — streams one `ActionRecord` with
                // `Keyboard` beside it, the attribution declared where
                // the route is known (the M30 rider rule at the
                // activation tier). The stamp rides the key route's
                // guard (M45, ADR-0051) — the focused control's id,
                // never guessed at the note site. One branch when
                // nobody listens (the A7 idle contract).
                alacer_core::action::note_action(alacer_core::ActionRecord {
                    ask: alacer_core::inspect::ActionAsk::Press,
                    target: None,
                    outcome: alacer_core::inspect::ActionOutcome::Delivered,
                    rider: alacer_core::inspect::ActionRider::Keyboard,
                });
            }
            // An unavailable link consumes the activation key and
            // writes nothing — the wall consumes the press (M22,
            // ADR-0028), the widget's own shape at its keyboard door.
            return EventResponse::Handled;
        }
        EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::Link;
    use alacer_core::events::{EventCtx, Key, KeyEvent, KeyState, Modifiers, PointerEvent};
    use alacer_core::geom::Point;
    use alacer_core::mask::Change;
    use alacer_core::widget::{A11yAction, EventResponse, Role, Widget};

    fn enter(state: KeyState) -> KeyEvent {
        KeyEvent {
            key: Key::Enter,
            state,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    fn space() -> KeyEvent {
        KeyEvent {
            key: Key::Space,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    #[test]
    fn role_name_and_availability_satisfy_the_contract() {
        let link = Link::new("Read the architecture");
        assert_eq!(link.role(), Role::Link);
        assert_eq!(link.label().resolve(), "Read the architecture");
        assert!(link.focusable(), "links hold focus");
        assert!(link.role().requires_name());
        assert!(link.is_enabled());
        assert_eq!(link.toggled(), None, "a link carries no selection fact");
        assert_eq!(link.table_slot(), None, "a link carries no grid slot");
    }

    #[test]
    fn click_opens_on_release_and_requests_paint_plus_a11y() {
        let opened = std::rc::Rc::new(std::cell::Cell::new(false));
        let sink = opened.clone();
        let mut link = Link::new("Docs").on_open(move || sink.set(true));
        let mut ctx = EventCtx::new();
        let down = PointerEvent::Down {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 1,
        };
        assert_eq!(
            link.on_pointer_event(&mut ctx, &down),
            EventResponse::Handled
        );
        assert!(!opened.get(), "the arm opens nothing — the release does");
        let up = PointerEvent::Up {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        assert_eq!(link.on_pointer_event(&mut ctx, &up), EventResponse::Handled);
        assert!(opened.get(), "the release opens");
        let masks = ctx.take();
        assert!(
            masks.contains(Change::PAINT) && masks.contains(Change::A11Y),
            "the open repaints and re-announces, got {masks:?}"
        );
    }

    #[test]
    fn enter_opens_and_space_is_handed_on() {
        // The link's keyboard verb is its own: Enter opens (the M10
        // parity at the link tier), Space is the scroller's key the
        // framework hands on.
        let opened = std::rc::Rc::new(std::cell::Cell::new(false));
        let sink = opened.clone();
        let mut link = Link::new("Docs").on_open(move || sink.set(true));
        let mut ctx = EventCtx::new();
        assert_eq!(
            link.on_key_event(&mut ctx, &enter(KeyState::Pressed)),
            EventResponse::Handled
        );
        assert!(opened.get(), "Enter opens");
        let masks = ctx.take();
        assert!(masks.contains(Change::A11Y), "the open re-announces");

        let mut ctx = EventCtx::new();
        assert_eq!(
            link.on_key_event(&mut ctx, &space()),
            EventResponse::Ignored,
            "Space is handed on — the scroller's key, the framework's to take"
        );
        assert!(opened.get(), "Space opened nothing");
    }

    #[test]
    fn the_a11y_default_action_opens() {
        let opened = std::rc::Rc::new(std::cell::Cell::new(false));
        let sink = opened.clone();
        let mut link = Link::new("Docs").on_open(move || sink.set(true));
        assert!(link.on_a11y_action(A11yAction::DefaultPress));
        assert!(opened.get(), "the AT's ask opens");
    }

    #[test]
    fn a_disabled_link_refuses_every_open_door() {
        // M22, ADR-0028: the AT tier is refused, the pointer arms nothing,
        // Enter writes nothing, and the write that moved the flag is the
        // only mask the tier costs.
        let mut link = Link::new("Archived")
            .enabled(false)
            .on_open(|| panic!("an unavailable link must not open"));
        let mut ctx = EventCtx::new();

        assert!(
            !link.on_a11y_action(A11yAction::DefaultPress),
            "an unavailable open has no write door"
        );
        assert_eq!(
            link.on_pointer_event(
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
            link.on_key_event(&mut ctx, &enter(KeyState::Pressed)),
            EventResponse::Handled,
            "Enter is the activation key, so the event is handled…"
        );
        assert!(
            ctx.take().is_empty(),
            "…but the refused open queues no masks"
        );
        assert!(!link.focusable(), "an unavailable control holds no focus");

        // The write answers the move (the M21 idle-frame property).
        assert!(!link.set_enabled(false), "already unavailable");
        assert!(link.set_enabled(true), "the write moved");
        assert!(link.is_enabled());
    }

    #[test]
    fn the_description_rides_the_trait_and_the_write_reports_the_move() {
        // M95, ADR-0099: the honest absence by default, the help text the
        // builder mounts, and the one write whose moved answer is the
        // only mask the tier costs.
        let mut link = Link::new("Read the architecture");
        assert_eq!(
            Widget::description(&link),
            None,
            "an undescribed link announces the absence"
        );
        link = link.description("Opens the docs site");
        assert_eq!(
            Widget::description(&link),
            Some("Opens the docs site"),
            "the help text the tree carries beside the name"
        );
        assert!(
            !link.set_description(Some("Opens the docs site".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(
            link.set_description(Some("Opens the API reference".to_string())),
            "the moved write answers true"
        );
        assert!(link.set_description(None), "clearing to the absence moved");
        assert_eq!(Widget::description(&link), None);
        assert!(!link.set_description(None), "the absence is already there");
    }

    #[test]
    fn programmatic_writes_fire_no_handler() {
        let mut link = Link::new("Docs").on_open(|| panic!("no programmatic open"));
        link.set_name("API reference");
        assert_eq!(link.label().resolve(), "API reference");
        link.set_enabled(false);
        assert!(!link.is_enabled());
        // The one handler write: the detach lands, the next open (if
        // the app re-arms) the re-armed handler's — the write door
        // pricing nothing but the swap.
        link.set_on_open(None);
        link.set_enabled(true);
        assert!(link.on_a11y_action(A11yAction::DefaultPress));
    }
}
