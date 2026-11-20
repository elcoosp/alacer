//! The dialog tier (M134, ADR-0138): the window the modal vocabulary
//! speaks as "this is a dialog", said by the tree.
//!
//! "This is the window the conversation lives in" was a vocabulary
//! hole: an app wiring a confirmation, a settings sheet, a file picker
//! mounted the conversation in a bare [`crate::Panel`] — a container
//! wearing no name (the set the AT cannot name is a set it cannot
//! walk, the ADR-5 rule the named roles carry) — or for a second
//! [`crate::TabPanel`] wearing the wrong word (a tab panel names *the
//! selected page's content*; a dialog names *the window the
//! conversation lives in*, and the AT's vocabulary names the
//! difference: AccessKit's `Dialog` role is its own fact).
//! [`Role::Dialog`](alacer_core::widget::Role::Dialog) carries the
//! window word the AT announces before it walks the conversation — the
//! named-container shape (ADR-0128) grown to the modal tier.
//!
//! The window is the content's layout, never a second channel to it:
//! the children ride the arena exactly as a [`crate::TabPanel`]'s do
//! (a vertical run, the shape a conversation is), each child keeping
//! the role and label it already carries — the field edits, the
//! button presses, the window naming what they read as. A dialog is
//! not a control: it holds no focus and answers no activation — the
//! M126/M129 container rule, the word names structure, it walls
//! nothing. And the word prices nothing but the word: no door behind
//! it changes, the content's behaviour the behaviour it always was.
//! The modal *semantics* — the walk bounded to the window's subtree
//! while it is up — ride the focus scope the M25 tier published
//! (ADR-0031's `focus_move_in`), never a second channel: the word
//! names the window, the walk bounds it.

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::style::Style;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// The window the modal vocabulary speaks as "this is a dialog" (the
/// dialog tier, M134, ADR-0138): the window word the AT announces
/// before it walks the conversation — an unnamed window reads as a
/// bare box, the set the AT cannot name is a set it cannot walk
/// (ADR-5).
pub struct Dialog {
    name: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    style: Style,
}

impl Dialog {
    /// A window labeled `name` — the conversation the app mounts reads
    /// as the window's members.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            style: Style::column(),
        }
    }

    /// The description (M95, ADR-0099): the help text the a11y tree
    /// carries beside the name — the instruction the label is too short
    /// to carry. Without it the window announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Gap between the window's children.
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

    /// Replaces the window's accessible name in place (view
    /// reconciliation).
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

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for Dialog {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Dialog
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the window has nothing to
    /// explain.
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
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
    use super::Dialog;
    use alacer_core::widget::{Role, Widget};

    #[test]
    fn role_name_and_window_contract_satisfy_the_tier() {
        let window = Dialog::new("Confirm publish");
        assert_eq!(window.role(), Role::Dialog);
        assert_eq!(window.label().resolve(), "Confirm publish");
        assert!(window.role().requires_name());
        assert!(
            !window.focusable(),
            "the window is the conversation's layout — the controls inside it hold the focus, never the container"
        );
        assert_eq!(window.role().name(), "dialog");
        assert_eq!(
            Widget::description(&window),
            None,
            "an undescribed window announces the absence"
        );
    }

    #[test]
    fn the_name_write_moves_the_word() {
        let mut window = Dialog::new("Confirm publish");
        window.set_name("Confirm deploy");
        assert_eq!(window.label().resolve(), "Confirm deploy");
    }

    #[test]
    fn the_description_write_reports_the_move() {
        // M95, ADR-0099: the honest absence by default, the help text the
        // builder mounts, and the one write whose moved answer is the
        // only mask the tier costs (the M21 idle-frame property).
        let mut window = Dialog::new("Confirm publish");
        window = window.description("The file goes live");
        assert_eq!(
            Widget::description(&window),
            Some("The file goes live"),
            "the help text the tree carries beside the name"
        );
        assert!(
            !window.set_description(Some("The file goes live".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(
            window.set_description(Some("The branch goes live".to_string())),
            "the moved write answers true"
        );
        assert!(
            window.set_description(None),
            "clearing to the absence moved"
        );
        assert!(
            !window.set_description(None),
            "the absence is already there"
        );
    }

    #[test]
    fn the_style_words_stay_layout_words() {
        let mut window = Dialog::new("Confirm publish")
            .gap(4.0)
            .padding(8.0)
            .grow(1.0);
        window.style_mut().gap = 6.0;
        let style = window.style();
        assert_eq!(style.gap, 6.0, "the shared-borrow door patches in place");
        assert_eq!(style.padding, 8.0);
    }
}
