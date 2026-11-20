//! The tooltip tier (M136, ADR-0140): the named help bubble the
//! pointer pair summons.
//!
//! "This holds the hover-help" was a vocabulary hole: an app wiring a
//! tooltip reached for a second [`crate::Label`] wearing no container
//! word (the bubble the AT cannot name is a bubble it cannot offer on
//! demand, the ADR-5 rule the named roles carry) — or for a
//! [`crate::Dialog`] wearing the wrong word (a dialog is a modal
//! window the walk bounds itself into; a tooltip is a transient
//! bubble the pointer pair summons, and the AT's vocabulary names the
//! difference: AccessKit's `Tooltip` role is its own fact).
//! [`Role::Tooltip`] carries the bubble word the AT announces while
//! the bubble is up.
//!
//! The bubble is the help's layout, never a second channel to it: the
//! children ride the arena exactly as a [`crate::TabPanel`]'s do (a
//! vertical run, the shape a bubble is), each child keeping the role
//! and label it already carries — the label reads, the bubble naming
//! what it helps. A tooltip is not a control: it holds no focus and
//! answers no activation — the M126/M129 container rule, at the hover
//! tier; the controls the app mounts around it hold the focus. And
//! the word prices nothing but the word: no door behind it changes,
//! the help's behaviour the behaviour it always was. Whether the
//! bubble is up is the app's wiring — the view mounts and unmounts it
//! (the reconciler's own shape, the M1 conditional-mount precedent),
//! the presence never a second `visible` flag the tree would have to
//! guess at.

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::style::Style;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// The named help bubble the pointer pair summons (the tooltip tier,
/// M136, ADR-0140): the bubble word the AT announces while the bubble
/// is up — an unnamed bubble reads as a bare box, the help the AT
/// cannot name is a help it cannot offer on demand (ADR-5).
pub struct Tooltip {
    name: String,
    style: Style,
}

impl Tooltip {
    /// A tooltip labeled `name` — the help the app mounts reads as the
    /// bubble's members.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            style: Style::column(),
        }
    }

    /// Gap between the bubble's children.
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

    /// Replaces the bubble's accessible name in place (view
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

impl Widget for Tooltip {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Tooltip
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
    use super::Tooltip;
    use alacer_core::widget::{Role, Widget};

    #[test]
    fn role_name_and_container_contract_satisfy_the_tier() {
        let bubble = Tooltip::new("Copy shortcut");
        assert_eq!(bubble.role(), Role::Tooltip);
        assert_eq!(bubble.label().resolve(), "Copy shortcut");
        assert!(bubble.role().requires_name());
        assert!(
            !bubble.focusable(),
            "the bubble is the help's layout — the controls around it hold the focus, never the bubble"
        );
        assert_eq!(bubble.role().name(), "tooltip");
    }

    #[test]
    fn the_name_write_moves_the_word() {
        let mut bubble = Tooltip::new("Copy shortcut");
        bubble.set_name("Paste shortcut");
        assert_eq!(bubble.label().resolve(), "Paste shortcut");
    }

    #[test]
    fn the_style_words_stay_layout_words() {
        let mut bubble = Tooltip::new("Copy shortcut")
            .gap(4.0)
            .padding(8.0)
            .grow(1.0);
        bubble.style_mut().gap = 6.0;
        let style = bubble.style();
        assert_eq!(style.gap, 6.0, "the shared-borrow door patches in place");
        assert_eq!(style.padding, 8.0);
    }
}
