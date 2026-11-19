//! The tab tier (M127, ADR-0131): the page selector and the named
//! strip it reads as.
//!
//! "This selects a page" was a vocabulary hole: an app wiring a tab
//! strip reached for a row of buttons — the wrong shape (a button's
//! activation is a *press* with no after-state, a tab's activation
//! *selects the page*, and the AT's vocabulary names the difference:
//! AccessKit's `Tab` role is its own fact) — or for bare labels
//! wearing no selection fact at all. [`Role::Tab`] carries the
//! "selects a page" fact, the selection riding the selected word the
//! tree already carries ([`Widget::selected`], AccessKit's
//! `Selected`), and [`Role::TabList`] the named strip the set reads
//! as — the radio tier's own member-and-container shape (ADR-0128) at
//! the page tier.
//!
//! Three doors, one selection, one contract: a click, the Space key
//! while focused, and the AccessKit default action all *select* — the
//! selected page never deselects itself. A redundant ask (the second
//! click, the second Space, the AT's ask on the selected page) is
//! consumed and moves nothing: the widget's own doors request no masks
//! and fire no handler — the M21 idle-frame property, at the widget
//! seam, the radio's rule restated. The a11y door answers `true` for
//! every select ask on an available tab — the ask is the tab's own
//! tier — and the re-announcement it costs rides the dispatch funnel's
//! coarse `consumed` contract (the funnel requests `PAINT | A11Y` for
//! every consumed ask; a redundant select's delta is empty): the one
//! coarse corner, named here where the consumer can read it.
//! Exclusivity — the sibling that deselects when this page becomes the
//! selection — is the app's wiring: the [`Tab::on_select`] handler is
//! where the strip's writer clears the others (the framework never
//! rewrites widgets it does not own, A1). The one deselect door is the
//! programmatic write ([`Tab::set_selected`], view reconciliation) —
//! programmatic state, never a user door, the checkbox's own rule.
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
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, Widget};

/// A page selector with a mandatory name (ADR-5: an unnamed tab is a
/// bare mark) and the selection the a11y tree announces — the page the
/// strip's content shows, said by the tree.
pub struct Tab {
    name: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    selected: bool,
    enabled: bool,
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

impl Tab {
    /// A tab labeled `name`, initially unselected.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            selected: false,
            enabled: true,
            on_select: None,
            focused: false,
            accent: None,
            text_color: None,
            font_size: crate::theme::FONT_SIZE,
            style: Style::default(),
        }
    }

    /// Initial availability (M22, ADR-0028). An unavailable tab paints
    /// muted, holds no focus, and refuses every select door honestly —
    /// the AT reads the refusal before it happens.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Initial selection state: `true` mounts the tab as the strip's
    /// selection — the page the strip's content shows.
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Installs the selection handler; invoked with `true` after the
    /// tab *became* the selection (click, Space, a11y action). A
    /// redundant ask moves nothing and fires nothing — the idle-frame
    /// property at the handler seam too.
    #[must_use]
    pub fn on_select(mut self, handler: impl FnMut(bool) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// The description (M95, ADR-0099): the help text beside the name.
    /// Without it the tab announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Accent color of the underline and the focus ring. Without it
    /// the tab borrows the theme's accent role at paint time
    /// (ADR-0079).
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

    /// Current selection state (test introspection).
    #[must_use]
    pub const fn is_selected(&self) -> bool {
        self.selected
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

    /// The one deselect door: sets the selection in place (view
    /// reconciliation). Fires no handler — programmatic state, never a
    /// user door. The *user's* doors select and never deselect; this
    /// write is how the app's wiring clears a tab when another becomes
    /// the selection.
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// The one description write (M95, ADR-0099, view reconciliation):
    /// `None` clears to the honest absence. Report whether the text
    /// moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the text already is queues
    /// nothing (the M21 idle-frame property, at the description tier).
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

    /// The one select path: a selected tab is already the selection —
    /// the ask is consumed and moves nothing (`false`, no masks, no
    /// handler: the M21 idle frame); an unselected tab becomes the
    /// selection (`true`, the caller requests the masks the move
    /// costs, the handler seeing the new state). The availability tier
    /// guards the funnel's single entry (M22, ADR-0028): an
    /// unavailable tab refuses, never munges (the M11 rule).
    fn select(&mut self, ctx: Option<&mut EventCtx>) -> bool {
        if !self.enabled || self.selected {
            return false;
        }
        self.selected = true;
        if let Some(ctx) = ctx {
            ctx.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
        }
        if let Some(handler) = &mut self.on_select {
            handler(true);
        }
        true
    }

    /// Intrinsic size: the page's word, the underline the selection
    /// paints beneath it — the strip's own discipline keeps the glyph
    /// math font-free.
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

impl Widget for Tab {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Tab
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the tab has nothing to explain.
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn style(&self) -> Style {
        Style {
            padding: 2.0,
            ..self.style.clone()
        }
    }

    /// The selected word (M108, ADR-0112's contract at the page tier):
    /// the fact the tree carries — the page the strip's content shows,
    /// the word the AT announces as "selected".
    fn selected(&self) -> Option<bool> {
        Some(self.selected)
    }

    /// The availability tier (M22, ADR-0028): the flag the a11y tree
    /// carries — an unavailable tab reports itself before the AT tries
    /// the select it will refuse.
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let size = ctx.size();
        // The borrowed roles resolve per paint pass (ADR-0079); the
        // look still tells the truth the tree carries (M22, ADR-0028):
        // an unavailable tab paints the muted text role, and the
        // selected tab paints the accent — the page the strip shows
        // reads as the page it is.
        let palette = ctx.palette();
        let label_color = if self.enabled {
            if self.selected {
                self.accent.unwrap_or(palette.accent)
            } else {
                self.text_color.unwrap_or(palette.text)
            }
        } else {
            palette.text_muted
        };

        // The label, the page's own word.
        alacer_core::text_seam::with_seam(|seam| {
            let layout = seam.layout_glyphs(&self.name, self.font_size, None);
            let text_origin = Point::new(2.0, (size.height - layout.height).max(0.0) * 0.5);
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

            // The selected page's underline: the accent fact the
            // glyphs sit on — deterministic pixels with no glyph
            // dependency (the snapshot corpus stays font-free).
            if self.selected && self.enabled {
                let accent = self.accent.unwrap_or(palette.accent);
                ctx.push_quad(alacer_core::scene::Quad::filled(
                    alacer_core::geom::Rect::new(
                        Point::new(0.0, size.height - 2.0),
                        alacer_core::geom::Size::new(layout.width.max(size.width), 2.0),
                    ),
                    accent,
                ));
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
        // The availability tier (M22, ADR-0028): an unavailable tab
        // has no write door — the select is refused honestly, never
        // munged (the M11 rule).
        if !self.enabled {
            return false;
        }
        if action == A11yAction::DefaultPress {
            // The a11y path has no EventCtx; the dispatch funnel queues
            // `PAINT | A11Y` itself when the door consumes — the door
            // answers every select ask on an available tab (the ask is
            // the tab's own tier), the redundant one landing where the
            // selection already was: the one coarse corner the
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
                // The wall (M22, ADR-0028): a disabled tab consumes the
                // press but requests nothing — no focus, no repaint.
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
            // the select-on-press ride — the focused tab answering
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

/// The named strip a set of tabs reads as (the tab tier's container
/// half, M127, ADR-0131): the strip word the AT announces before it
/// walks the pages. Children ride the arena exactly as a
/// [`crate::Row`]'s do — the strip is the pages' layout (a horizontal
/// run, the shape a tab strip is), never a second channel to them.
pub struct TabList {
    name: String,
    style: Style,
}

impl TabList {
    /// A tab strip labeled `name` — an unnamed strip reads as a bare
    /// box, the set the AT cannot name is a set it cannot walk (ADR-5).
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            style: Style::row(),
        }
    }

    /// Gap between the pages.
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

    /// Shared-borrow layout style access for view reconciliation (§6.1):
    /// patches write styles in place instead of rebuilding the widget.
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    /// Replaces the strip's accessible name in place (view
    /// reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for TabList {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::TabList
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::{Tab, TabList};
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

    fn enter() -> KeyEvent {
        KeyEvent {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            text: None,
        }
    }

    fn click(ctx: &mut EventCtx, tab: &mut Tab) {
        let down = PointerEvent::Down {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 1,
        };
        let up = PointerEvent::Up {
            position: Point::new(4.0, 4.0),
            button: alacer_core::events::MouseButton::Left,
        };
        assert_eq!(tab.on_pointer_event(ctx, &down), EventResponse::Handled);
        assert_eq!(tab.on_pointer_event(ctx, &up), EventResponse::Handled);
    }

    #[test]
    fn role_name_and_selection_state_satisfy_the_contract() {
        let tab = Tab::new("Inbox");
        assert_eq!(tab.role(), Role::Tab);
        assert_eq!(tab.label().resolve(), "Inbox");
        assert!(tab.focusable(), "tabs hold focus");
        assert!(tab.role().requires_name());
        assert_eq!(
            Widget::selected(&tab),
            Some(false),
            "an unselected page announces the fact, never the absence — the strip's own shape"
        );
        let strip = TabList::new("Mailboxes");
        assert_eq!(strip.role(), Role::TabList);
        assert_eq!(strip.label().resolve(), "Mailboxes");
        assert!(strip.role().requires_name());
        assert!(
            !strip.focusable(),
            "the strip is the pages' layout — the pages hold the focus, never the container"
        );
    }

    #[test]
    fn click_selects_and_requests_paint_plus_a11y() {
        let mut tab = Tab::new("Inbox");
        let mut ctx = EventCtx::new();
        click(&mut ctx, &mut tab);
        assert!(tab.is_selected(), "click selects");
        let masks = ctx.take();
        assert!(
            masks.contains(Change::PAINT) && masks.contains(Change::A11Y),
            "the selection repaints and re-announces, got {masks:?}"
        );
    }

    #[test]
    fn space_selects_and_enter_is_handed_on() {
        // The tab's keyboard verb is its own: Space selects (the radio
        // vocabulary's own verb at the selection tier), Enter is handed
        // on — not the tab's key.
        let mut tab = Tab::new("Inbox");
        let mut ctx = EventCtx::new();
        assert_eq!(tab.on_key_event(&mut ctx, &space()), EventResponse::Handled);
        assert!(tab.is_selected(), "Space selects");
        let masks = ctx.take();
        assert!(masks.contains(Change::A11Y), "the selection re-announces");

        let mut ctx = EventCtx::new();
        assert_eq!(
            tab.on_key_event(&mut ctx, &enter()),
            EventResponse::Ignored,
            "Enter is handed on — not the tab's key"
        );
    }

    #[test]
    fn a_redundant_ask_is_the_idle_frame() {
        // The second click on the selected page: consumed, moves
        // nothing, fires nothing — no masks from the select funnel, the
        // M21 idle frame. The press's own arm (the pointer door's
        // PAINT | FOCUS, every press's shape) rides beside it; the
        // re-announcement is what the idle frame never queues.
        let mut tab = Tab::new("Inbox")
            .selected(true)
            .on_select(|_| panic!("a redundant ask fires nothing"));
        let mut ctx = EventCtx::new();
        click(&mut ctx, &mut tab);
        assert!(tab.is_selected(), "the selection never moved");
        let masks = ctx.take();
        assert!(
            masks.contains(Change::PAINT) && masks.contains(Change::FOCUS),
            "the press arms the pointer door, got {masks:?}"
        );
        assert!(
            !masks.contains(Change::A11Y),
            "the idle frame re-announces nothing — the select's delta is empty"
        );

        // The AT's ask on the selected page: the door answers (the ask
        // is the tab's own tier), the funnel's coarse `consumed`
        // contract the one corner — the widget's own doors moved
        // nothing.
        assert!(tab.on_a11y_action(A11yAction::DefaultPress));
    }

    #[test]
    fn a_disabled_tab_refuses_every_select_door() {
        // M22, ADR-0028: the AT tier is refused, the pointer arms
        // nothing, Space writes nothing, and the write that moved the
        // flag is the only mask the tier costs.
        let mut tab = Tab::new("Archive")
            .enabled(false)
            .on_select(|_| panic!("an unavailable tab must not select"));
        let mut ctx = EventCtx::new();

        assert!(
            !tab.on_a11y_action(A11yAction::DefaultPress),
            "an unavailable tab has no write door"
        );
        assert_eq!(
            tab.on_pointer_event(
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
            tab.on_key_event(&mut ctx, &space()),
            EventResponse::Handled,
            "Space is the activation key, so the event is handled…"
        );
        assert!(
            ctx.take().is_empty(),
            "…but the refused select queues no masks"
        );
        assert!(!tab.focusable(), "an unavailable control holds no focus");

        // The write answers the move (the M21 idle-frame property).
        assert!(!tab.set_enabled(false), "already unavailable");
        assert!(tab.set_enabled(true), "the write moved");
        assert!(tab.is_enabled());
    }

    #[test]
    fn the_description_and_programmatic_writes_answer_the_contract() {
        // M95, ADR-0099: the honest absence by default, the help text
        // the builder mounts, and the one write whose moved answer is
        // the only mask the tier costs.
        let mut tab = Tab::new("Inbox").on_select(|_| panic!("no programmatic select"));
        assert_eq!(
            Widget::description(&tab),
            None,
            "an undescribed tab announces the absence"
        );
        tab = tab.description("The mail the inbox holds");
        assert_eq!(
            Widget::description(&tab),
            Some("The mail the inbox holds"),
            "the help text the tree carries beside the name"
        );
        assert!(
            !tab.set_description(Some("The mail the inbox holds".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(tab.set_description(None), "clearing to the absence moved");

        // The one deselect door: programmatic state, never a user door.
        tab.set_selected(true);
        assert!(tab.is_selected());
        tab.set_selected(false);
        assert!(!tab.is_selected());
        tab.set_name("Junk");
        assert_eq!(tab.label().resolve(), "Junk");
    }
}
