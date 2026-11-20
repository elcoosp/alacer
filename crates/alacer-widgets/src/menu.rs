//! The menu tier (M137, ADR-0141): the named strip of menus, the
//! summoned list, and the command entries.
//!
//! "This holds the commands" was a vocabulary hole: an app wiring a
//! menu bar reached for a [`crate::Toolbar`] wearing the wrong word (a
//! toolbar *offers tools*, a menu bar *names the menus the commands
//! live in* — AccessKit's `MenuBar` role is its own fact), for a
//! [`crate::Dialog`] wearing the wrong transient (a dialog is a modal
//! window the walk bounds itself into; a menu is a transient list the
//! opener summons — AccessKit's `Menu` role is its own fact), or for a
//! bare [`crate::Button`] wearing no entry word at all (the command the
//! AT cannot name is a command it cannot fire — AccessKit's `MenuItem`
//! role is its own fact). [`Role::MenuBar`], [`Role::Menu`] and
//! [`Role::MenuItem`] carry the words the AT announces — the
//! member-and-container shape (ADR-0128) grown to the command tier:
//! the strip, the summoned list, the entry.
//!
//! The strip and the list are the entries' layout, never a second
//! channel to them: the children ride the arena exactly as a
//! [`crate::Toolbar`]'s and a pane's do (a horizontal run for the
//! strip, a vertical run for the list — the shapes a menu bar and a
//! menu are), each child keeping the role and label it already carries.
//! Neither container is a control: they hold no focus and answer no
//! activation — the M126/M129 container rule, at the menu tier; the
//! entries hold the focus. And the words price nothing but the words:
//! no door behind them changes, the commands' behaviour the behaviour
//! it always was. Whether the list is up is the app's wiring — the view
//! mounts and unmounts it (the reconciler's own shape, the tooltip
//! tier's presence rule, ADR-0140), never a second `visible` flag the
//! tree would have to guess at.
//!
//! The entry is the tier's only control, and its funnel is the button
//! tier's own (the M10 parity): the click on release, Enter while
//! focused, and the AT's default action the door answers — the
//! availability tier guarding every door (M22, ADR-0028: an unavailable
//! entry refuses, never munges, holds no focus).

use alacer_core::action::note_action;
use alacer_core::color::Color;
use alacer_core::events::{EventCtx, Key, KeyEvent, KeyState, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Style, TextSpec};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, Widget};

/// The named strip of menus a menu bar reads as (the menu tier's
/// container half, M137, ADR-0141): the strip word the AT announces
/// before it walks the menus — an unnamed strip reads as a bare box,
/// the set the AT cannot name is a set it cannot walk (ADR-5).
pub struct MenuBar {
    name: String,
    style: Style,
}

impl MenuBar {
    /// A menu bar labeled `name` — the menus the app mounts read as the
    /// strip's members.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            style: Style::row(),
        }
    }

    /// Gap between the menus.
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

impl Widget for MenuBar {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::MenuBar
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

/// The named list of commands a menu offers (the menu tier's summoned
/// half, M137, ADR-0141): the list word the AT announces while the menu
/// is up — an unnamed list reads as a bare box, the ADR-5 rule the
/// named roles carry. Presence is the app's wiring: the view mounts and
/// unmounts the list exactly while the menu is summoned.
pub struct Menu {
    name: String,
    style: Style,
}

impl Menu {
    /// A menu labeled `name` — the entries the app mounts read as the
    /// list's members.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            style: Style::column(),
        }
    }

    /// Gap between the entries.
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

    /// Replaces the list's accessible name in place (view
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

impl Widget for Menu {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Menu
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

/// One command a menu offers (the menu tier's entry half, M137,
/// ADR-0141): the entry word the AT walks to and fires — an unnamed
/// entry reads as a bare mark (ADR-5). The funnel is the button tier's
/// own: the click on release, Enter while focused (the M10 parity), and
/// the AT's default action the door answers — the availability tier
/// guarding every door (M22).
pub struct MenuItem {
    label: String,
    pressed: bool,
    enabled: bool,
    /// The app's fill override; `None` borrows the theme's panel role
    /// at paint time (ADR-0079).
    fill: Option<Color>,
    font_size: f32,
    corner_radius: f32,
    style: Style,
    on_click: Option<Box<dyn FnMut()>>,
}

impl MenuItem {
    /// A menu item labeled `label`.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            pressed: false,
            enabled: true,
            fill: None,
            font_size: crate::theme::FONT_SIZE,
            corner_radius: 4.0,
            style: Style::default(),
            on_click: None,
        }
    }

    /// Initial availability (M22, ADR-0028). An unavailable entry paints
    /// muted, holds no focus, and refuses every door honestly — the AT
    /// reads the refusal before it happens.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// The app's fill override; `None` borrows the theme's panel role at
    /// paint time (ADR-0079).
    #[must_use]
    pub fn fill(mut self, fill: Color) -> Self {
        self.fill = Some(fill);
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

    /// Shared-borrow layout style access for view reconciliation (§6.1):
    /// patches write styles in place instead of rebuilding the widget.
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    /// Whether the entry is currently pressed (test introspection).
    #[must_use]
    pub const fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// The availability the widget reports — the same flag the M22
    /// `enabled()` contract carries into the a11y tree (test
    /// introspection).
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The one availability write: report whether the flag moved. The
    /// caller requests the write's own masks — `PAINT | A11Y` — only
    /// when this answers `true`, so a write that lands where the flag
    /// already is queues nothing (the M21 idle-frame property, at the
    /// menu tier).
    pub fn set_enabled(&mut self, enabled: bool) -> bool {
        if self.enabled == enabled {
            return false;
        }
        self.enabled = enabled;
        true
    }

    /// Replaces the entry's label in place (view reconciliation, §6.1).
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Installs the click handler; invoked on release inside the entry.
    #[must_use]
    pub fn on_click(mut self, handler: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Replaces the click handler in place (view reconciliation). `None`
    /// detaches the handler.
    pub fn set_on_click(&mut self, handler: Option<Box<dyn FnMut()>>) {
        self.on_click = handler;
    }

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for MenuItem {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::MenuItem
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.label.clone())
    }

    /// The availability tier (M22, ADR-0028): the flag the a11y tree
    /// carries — an unavailable entry reports itself before the AT tries
    /// the press it will refuse.
    fn enabled(&self) -> bool {
        self.enabled
    }

    fn style(&self) -> Style {
        Style {
            padding: 8.0,
            ..self.style.clone()
        }
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        let size = (ctx.size().width, ctx.size().height);
        let available = self.enabled;
        // The borrowed roles resolve per paint pass (ADR-0079): a plain
        // entry borrows the panel role, a pressed entry the accent (the
        // look tells the truth the funnel carries), an unavailable one
        // the muted rail — the same discipline the button tier paints.
        let palette = ctx.palette();
        let fill = if !available {
            palette.border
        } else if self.pressed {
            palette.accent
        } else {
            self.fill.unwrap_or(palette.panel)
        };
        let text_color = if !available {
            palette.text_muted
        } else if self.pressed {
            palette.on_accent
        } else {
            palette.text
        };
        crate::paint_background(
            ctx,
            size,
            fill,
            self.corner_radius,
            (Color::TRANSPARENT, 0.0),
        );

        // Left-aligned text (a menu entry's own shape — never the
        // centered button); the label wraps at the entry's laid-out
        // width — the same constraint the measure pass saw, so line
        // breaks match.
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
            text_size.height.max(24.0) + 12.0,
        )))
    }

    fn on_a11y_action(&mut self, action: A11yAction) -> bool {
        // The availability tier (M22, ADR-0028): an unavailable entry has
        // no write door — every action is refused honestly, never munged
        // (the M11 rule). The AT read the refusal before it happened: the
        // tree carries the disabled flag.
        if !self.enabled {
            return false;
        }
        if action == A11yAction::DefaultPress {
            if let Some(handler) = &mut self.on_click {
                handler();
            }
            return true;
        }
        false
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        // The availability tier (M22, ADR-0028): a disabled entry is a
        // wall, not a window — the gesture is consumed so it never
        // clicks through to an ancestor, it arms nothing, and it
        // requests no masks (an idle frame; the HTML `disabled`
        // semantics).
        if !self.enabled
            && matches!(
                event,
                PointerEvent::Down { .. } | PointerEvent::Up { .. } | PointerEvent::Leave
            )
        {
            return EventResponse::Handled;
        }
        match event {
            PointerEvent::Down { .. } => {
                self.pressed = true;
                // Click-to-focus: entries take keyboard focus on press
                // (§10.3), which also detaches the IME from any editing
                // widget.
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
        if !self.enabled {
            return EventResponse::Ignored;
        }
        // Keyboard activation (M10, §10.3 parity): a focused entry answers
        // Space and Enter exactly like a pointer press-and-release. GTK
        // semantics — the activation fires on key *release* — so an
        // auto-repeat hold cannot machine-gun the handler.
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
                    // entry answering Space/Enter like a click —
                    // streams one `ActionRecord` with `Keyboard`
                    // beside it, the attribution declared where the
                    // route is known (the M30 rider rule at the
                    // activation tier). One fact per activation (the
                    // flood rule): the pressed phase armed nothing on
                    // the log, the release is where the door answered.
                    // One branch when nobody listens (the A7 idle
                    // contract).
                    note_action(alacer_core::ActionRecord {
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
    use super::{Menu, MenuBar, MenuItem};
    use alacer_core::widget::{Role, Widget};

    #[test]
    fn the_strip_carries_the_menu_bar_contract() {
        let strip = MenuBar::new("Main");
        assert_eq!(strip.role(), Role::MenuBar);
        assert_eq!(strip.label().resolve(), "Main");
        assert!(strip.role().requires_name());
        assert!(
            !strip.focusable(),
            "the strip is the menus' layout — the entries hold the focus, never the container"
        );
        assert_eq!(strip.role().name(), "menu_bar");
    }

    #[test]
    fn the_list_carries_the_menu_contract() {
        let list = Menu::new("File");
        assert_eq!(list.role(), Role::Menu);
        assert_eq!(list.label().resolve(), "File");
        assert!(list.role().requires_name());
        assert!(
            !list.focusable(),
            "the list is the entries' layout — the entries hold the focus, never the container"
        );
        assert_eq!(list.role().name(), "menu");
    }

    #[test]
    fn the_entry_is_the_tiers_only_control() {
        let entry = MenuItem::new("New");
        assert_eq!(entry.role(), Role::MenuItem);
        assert_eq!(entry.label().resolve(), "New");
        assert!(entry.role().requires_name());
        assert!(
            entry.focusable(),
            "an entry is a control — the walk lands it, never the containers"
        );
        assert_eq!(entry.role().name(), "menu_item");
    }

    #[test]
    fn the_name_writes_move_the_words() {
        let mut strip = MenuBar::new("Main");
        strip.set_name("Commands");
        assert_eq!(strip.label().resolve(), "Commands");
        let mut list = Menu::new("File");
        list.set_name("Edit");
        assert_eq!(list.label().resolve(), "Edit");
        let mut entry = MenuItem::new("New");
        entry.set_label("Open");
        assert_eq!(entry.label().resolve(), "Open");
    }

    #[test]
    fn the_availability_write_reports_the_move() {
        let mut entry = MenuItem::new("Export");
        assert!(entry.is_enabled());
        assert!(entry.set_enabled(false), "the move answers true");
        assert!(!entry.is_enabled());
        assert!(
            !entry.set_enabled(false),
            "the write that lands where the flag already is queues nothing"
        );
        assert!(entry.set_enabled(true), "the move back answers true");
        assert!(entry.is_enabled());
    }

    #[test]
    fn the_style_words_stay_layout_words() {
        let mut list = Menu::new("File").gap(4.0).padding(8.0).grow(1.0);
        list.style_mut().gap = 6.0;
        let style = list.style();
        assert_eq!(style.gap, 6.0, "the shared-borrow door patches in place");
        assert_eq!(style.padding, 8.0);
    }
}
