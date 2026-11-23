//! The landmark tier (M143, ADR-0147; the growth, M144, ADR-0148): the
//! page-structure words the AT walks a document by — the banner, the
//! navigation strip, the main region, the footer, and (M144) the
//! complementary aside, the form, the generic named region and the
//! search strip.
//!
//! "This is the page's furniture" was a vocabulary hole: an app wiring
//! a header, a nav strip, a content region and a footer reached for
//! bare [`crate::Panel`]s wearing no container word (the region the AT
//! cannot name is a region it cannot offer on demand, the ADR-5 rule
//! the named roles carry) — or for a [`crate::Toolbar`] wearing the
//! wrong word (a toolbar *offers tools*, the M129 strip the controls
//! live in; a landmark *names the document's own structure* — the
//! masthead, the way through, the content, the colophon, the words
//! AccessKit's `Banner`, `Navigation`, `Main` and `ContentInfo` roles
//! spell and the AT's landmarks list reads).
//!
//! A landmark is the furniture's layout, never a second channel to it:
//! the children ride the arena exactly as a [`crate::TabPanel`]'s do
//! (a vertical run, the shape a page section is), each child keeping
//! the role and label it already carries. A landmark is not a control:
//! it holds no focus and answers no activation — the M126/M129
//! container rule, at the landmark tier; the controls the app mounts
//! inside it hold the focus. And the words price nothing but the
//! words: no door behind them changes, the furniture's behaviour the
//! behaviour it always was. Which landmark a constructor names is the
//! app's declaration — the four words are four facts about the same
//! layout, not four widgets.

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::style::Style;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// The named page furniture the AT's landmarks list walks (the
/// landmark tier, M143, ADR-0147; the growth, M144, ADR-0148): the
/// banner, navigation, main, content-info, complementary, form,
/// region or search word the AT announces for the region — an unnamed
/// region reads as a bare box, the structure the AT cannot name is a
/// structure it cannot offer on demand (ADR-5). Which word is the
/// constructor's own: [`Landmark::banner`], [`Landmark::navigation`],
/// [`Landmark::main`], [`Landmark::content_info`],
/// [`Landmark::complementary`], [`Landmark::form`], [`Landmark::region`],
/// [`Landmark::search`].
pub struct Landmark {
    role: Role,
    name: String,
    style: Style,
}

impl Landmark {
    /// A banner labeled `name` — the page's opening strip, the
    /// masthead the AT's landmarks list walks first.
    #[must_use]
    pub fn banner(name: impl Into<String>) -> Self {
        Self::with_role(Role::Banner, name)
    }

    /// A navigation region labeled `name` — the document's way through,
    /// the section links the landmarks list walks beside the banner and
    /// the main.
    #[must_use]
    pub fn navigation(name: impl Into<String>) -> Self {
        Self::with_role(Role::Navigation, name)
    }

    /// A main region labeled `name` — the document's central content,
    /// the landmarks list's own destination.
    #[must_use]
    pub fn main(name: impl Into<String>) -> Self {
        Self::with_role(Role::Main, name)
    }

    /// A content-info region labeled `name` — the page's closing strip,
    /// the colophon the landmarks list walks last.
    #[must_use]
    pub fn content_info(name: impl Into<String>) -> Self {
        Self::with_role(Role::ContentInfo, name)
    }

    /// A complementary region labeled `name` — the content the main
    /// reads beside, the aside the landmarks list walks beside the
    /// central region (the landmark tier's growth, M144, ADR-0148).
    #[must_use]
    pub fn complementary(name: impl Into<String>) -> Self {
        Self::with_role(Role::Complementary, name)
    }

    /// A form region labeled `name` — the region whose controls submit
    /// together, the checkout the landmarks list walks as one unit (the
    /// landmark tier's growth, M144, ADR-0148).
    #[must_use]
    pub fn form(name: impl Into<String>) -> Self {
        Self::with_role(Role::Form, name)
    }

    /// A generic named region labeled `name` — the word beside the
    /// specific ones, for the section whose fact none of them spells
    /// and whose name the app still declares (the landmark tier's
    /// growth, M144, ADR-0148).
    #[must_use]
    pub fn region(name: impl Into<String>) -> Self {
        Self::with_role(Role::Region, name)
    }

    /// A search region labeled `name` — the region whose controls find
    /// things, the query strip the landmarks list walks as one unit
    /// (the landmark tier's growth, M144, ADR-0148).
    #[must_use]
    pub fn search(name: impl Into<String>) -> Self {
        Self::with_role(Role::Search, name)
    }

    /// The constructor the words ride — one shape, eight facts.
    fn with_role(role: Role, name: impl Into<String>) -> Self {
        Self {
            role,
            name: name.into(),
            style: Style::column(),
        }
    }

    /// Gap between the landmark's children.
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

    /// Replaces the landmark's accessible name in place (view
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

impl Widget for Landmark {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        self.role
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
    use super::Landmark;
    use alacer_core::widget::{Role, Widget};

    #[test]
    fn the_words_carry_their_roles() {
        let banner = Landmark::banner("Catalog");
        assert_eq!(banner.role(), Role::Banner);
        assert_eq!(banner.role().name(), "banner");
        let nav = Landmark::navigation("Catalog sections");
        assert_eq!(nav.role(), Role::Navigation);
        assert_eq!(nav.role().name(), "navigation");
        let main = Landmark::main("The catalog");
        assert_eq!(main.role(), Role::Main);
        assert_eq!(main.role().name(), "main");
        let footer = Landmark::content_info("Catalog colophon");
        assert_eq!(footer.role(), Role::ContentInfo);
        assert_eq!(footer.role().name(), "content_info");
        // The growth (M144, ADR-0148): the remaining ARIA facts join as
        // constructor words beside the four — one shape, eight facts.
        let aside = Landmark::complementary("Related reading");
        assert_eq!(aside.role(), Role::Complementary);
        assert_eq!(aside.role().name(), "complementary");
        let form = Landmark::form("Checkout details");
        assert_eq!(form.role(), Role::Form);
        assert_eq!(form.role().name(), "form");
        let region = Landmark::region("Featured panel");
        assert_eq!(region.role(), Role::Region);
        assert_eq!(region.role().name(), "region");
        let search = Landmark::search("Catalog search");
        assert_eq!(search.role(), Role::Search);
        assert_eq!(search.role().name(), "search");
    }

    #[test]
    fn role_name_and_container_contract_satisfy_the_tier() {
        let region = Landmark::main("The catalog");
        assert_eq!(region.label().resolve(), "The catalog");
        assert!(
            region.role().requires_name(),
            "an unnamed landmark reads as a bare box — the ADR-5 rule the named roles carry"
        );
        assert!(
            !region.focusable(),
            "the furniture is the page's layout — the controls inside it hold the focus, never the furniture"
        );
    }

    #[test]
    fn the_name_write_moves_the_word() {
        let mut region = Landmark::main("The catalog");
        region.set_name("The whole catalog");
        assert_eq!(region.label().resolve(), "The whole catalog");
    }

    #[test]
    fn the_style_words_stay_layout_words() {
        let mut region = Landmark::banner("Catalog").gap(4.0).padding(8.0).grow(1.0);
        region.style_mut().gap = 6.0;
        let style = region.style();
        assert_eq!(style.gap, 6.0, "the shared-borrow door patches in place");
        assert_eq!(style.padding, 8.0);
    }
}
