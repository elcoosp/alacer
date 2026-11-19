//! The toolbar tier (M129, ADR-0133): the named strip a set of tools
//! reads as.
//!
//! "This strip holds the tools" was a vocabulary hole: an app wiring a
//! toolbar reached for a bare [`crate::Row`] — a container wearing no
//! name (the set the AT cannot name is a set it cannot walk, the ADR-5
//! rule the named roles carry) — or for a [`crate::TabList`] wearing
//! the wrong word (a tab strip *selects pages*; a toolbar *offers
//! tools*, and the AT's vocabulary names the difference: AccessKit's
//! `Toolbar` role is its own fact). [`Role::Toolbar`] carries the
//! strip word the AT announces before it walks the tools — the
//! tab-list/radio-group shape (the named container, ADR-0128's
//! member-and-container discipline) at the tool tier.
//!
//! The strip is the tools' layout, never a second channel to them: the
//! children ride the arena exactly as a [`crate::Row`]'s do (a
//! horizontal run, the shape a toolbar is), each tool keeping the role
//! and label it already carries — the button presses, the input edits,
//! the strip naming the set they read as. A toolbar is not a control:
//! it holds no focus and answers no activation — the tools hold the
//! focus, the word names the set they read as. And the word prices
//! nothing but the word: no door behind it changes, the tools'
//! behaviour the behaviour it always was.

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::style::Style;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// The named strip a set of tools reads as (the toolbar tier, M129,
/// ADR-0133): the strip word the AT announces before it walks the
/// tools — an unnamed strip reads as a bare box, the set the AT cannot
/// name is a set it cannot walk (ADR-5).
pub struct Toolbar {
    name: String,
    style: Style,
}

impl Toolbar {
    /// A toolbar labeled `name` — the tools the app mounts read as the
    /// strip's members.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            style: Style::row(),
        }
    }

    /// Gap between the tools.
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

impl Widget for Toolbar {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Toolbar
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
    use super::Toolbar;
    use alacer_core::widget::{Role, Widget};

    #[test]
    fn role_name_and_container_contract_satisfy_the_tier() {
        let strip = Toolbar::new("Formatting");
        assert_eq!(strip.role(), Role::Toolbar);
        assert_eq!(strip.label().resolve(), "Formatting");
        assert!(strip.role().requires_name());
        assert!(
            !strip.focusable(),
            "the strip is the tools' layout — the tools hold the focus, never the container"
        );
        assert_eq!(strip.role().name(), "toolbar");
    }

    #[test]
    fn the_name_write_moves_the_word() {
        let mut strip = Toolbar::new("Formatting");
        strip.set_name("Review");
        assert_eq!(strip.label().resolve(), "Review");
    }

    #[test]
    fn the_style_words_stay_layout_words() {
        let mut strip = Toolbar::new("Formatting").gap(4.0).padding(8.0).grow(1.0);
        strip.style_mut().gap = 6.0;
        let style = strip.style();
        assert_eq!(style.gap, 6.0, "the shared-borrow door patches in place");
        assert_eq!(style.padding, 8.0);
    }
}
