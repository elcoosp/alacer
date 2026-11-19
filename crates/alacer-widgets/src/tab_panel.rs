//! The tab-panel tier (M131, ADR-0135): the named pane the selected
//! page's content lives in, said by the tree.
//!
//! "This holds the selected page's content" was a vocabulary hole: an
//! app wiring a tab strip mounted the pages (the member, M127) inside
//! the named strip (the container, M127) and showed the selection's
//! content in a bare [`crate::Panel`] — a container wearing no name
//! (the set the AT cannot name is a set it cannot walk, the ADR-5 rule
//! the named roles carry) — or for a second [`crate::TabList`] wearing
//! the wrong word (a tab list *names the pages' strip*; a tab panel
//! *names the page's content*, and the AT's vocabulary names the
//! difference: AccessKit's `TabPanel` role is its own fact).
//! [`Role::TabPanel`] carries the pane word the AT announces before it
//! reads the selection's content — the tab tier's member-and-container
//! shape (ADR-0128, the radio tier's own discipline) grown to its third
//! word: the member ([`crate::Tab`]), the strip ([`crate::TabList`]),
//! the pane ([`TabPanel`]).
//!
//! The pane is the content's layout, never a second channel to it: the
//! children ride the arena exactly as a [`crate::Column`]'s do (a
//! vertical run, the shape a pane is), each child keeping the role and
//! label it already carries — the button presses, the label reads, the
//! pane naming what they read as. A pane is not a control: it holds no
//! focus and answers no activation — the M126/M129 container rule, the
//! word names structure, it walls nothing. And the word prices nothing
//! but the word: no door behind it changes, the content's behaviour
//! the behaviour it always was. Which page's content the pane holds is
//! the app's wiring — the [`crate::Tab::on_select`] handler and the
//! view's programmatic doors, the exclusivity rule the tab tier
//! published (the framework never rewrites widgets it does not own,
//! A1).

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::style::Style;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// The named pane the selected page's content lives in (the tab-panel
/// tier, M131, ADR-0135): the pane word the AT announces before it
/// reads the content — an unnamed pane reads as a bare box, the set the
/// AT cannot name is a set it cannot walk (ADR-5).
pub struct TabPanel {
    name: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    style: Style,
}

impl TabPanel {
    /// A pane labeled `name` — the content the app mounts reads as the
    /// pane's members.
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
    /// to carry. Without it the pane announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Gap between the pane's children.
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

    /// Replaces the pane's accessible name in place (view
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

impl Widget for TabPanel {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::TabPanel
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the pane has nothing to explain.
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
    use super::TabPanel;
    use alacer_core::widget::{Role, Widget};

    #[test]
    fn role_name_and_container_contract_satisfy_the_tier() {
        let pane = TabPanel::new("Inbox content");
        assert_eq!(pane.role(), Role::TabPanel);
        assert_eq!(pane.label().resolve(), "Inbox content");
        assert!(pane.role().requires_name());
        assert!(
            !pane.focusable(),
            "the pane is the content's layout — the content holds the focus, never the container"
        );
        assert_eq!(pane.role().name(), "tab_panel");
        assert_eq!(
            Widget::description(&pane),
            None,
            "an undescribed pane announces the absence"
        );
    }

    #[test]
    fn the_name_write_moves_the_word() {
        let mut pane = TabPanel::new("Inbox content");
        pane.set_name("Sent content");
        assert_eq!(pane.label().resolve(), "Sent content");
    }

    #[test]
    fn the_description_write_reports_the_move() {
        // M95, ADR-0099: the honest absence by default, the help text the
        // builder mounts, and the one write whose moved answer is the
        // only mask the tier costs (the M21 idle-frame property).
        let mut pane = TabPanel::new("Inbox content");
        pane = pane.description("The mail the folder holds");
        assert_eq!(
            Widget::description(&pane),
            Some("The mail the folder holds"),
            "the help text the tree carries beside the name"
        );
        assert!(
            !pane.set_description(Some("The mail the folder holds".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(
            pane.set_description(Some("The mail you sent".to_string())),
            "the moved write answers true"
        );
        assert!(pane.set_description(None), "clearing to the absence moved");
        assert!(!pane.set_description(None), "the absence is already there");
    }

    #[test]
    fn the_style_words_stay_layout_words() {
        let mut pane = TabPanel::new("Inbox content")
            .gap(4.0)
            .padding(8.0)
            .grow(1.0);
        pane.style_mut().gap = 6.0;
        let style = pane.style();
        assert_eq!(style.gap, 6.0, "the shared-borrow door patches in place");
        assert_eq!(style.padding, 8.0);
    }
}
