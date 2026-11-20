//! The tree tier (M135, ADR-0139): the hierarchical collection the AT
//! walks by disclosure and depth, said by the tree.
//!
//! "This collection nests" was a vocabulary hole: through M134 the flat
//! collections had the words — the M6 table grid (rows × columns), the
//! M108 picks, the M110 disclosures, the M113 depths, the M119 places
//! and spans — while the *hierarchical* collection, the file explorer
//! or category outline or nested navigation, had no word of its own. An
//! app mounting one reached for the table's grid (a grid's geometry is
//! rows × columns; a tree's is parentage — the wrong geometry wearing
//! the wrong word) or for a bare [`crate::List`] whose members could
//! say nothing about the branches they stood in.
//! [`Role::Tree`](alacer_core::widget::Role::Tree) and
//! [`Role::TreeItem`](alacer_core::widget::Role::TreeItem) carry the
//! collection word and the member word AccessKit's own roles spell —
//! the member-and-container shape (ADR-0128) grown to the hierarchical
//! tier.
//!
//! The tier prices the words and nothing else: the hierarchy *is* the
//! arena's own nesting (a tree item's children ride under it, exactly
//! a row's cells do), and every word the members say is a tier the
//! tree already published — the M108 pick, the M110 disclosure, the
//! M113 depth, the M119 place, the M119 container span, the M109
//! capacity, the M117 updating word, the M120 declared region — never
//! a second vocabulary. A tree and its items are not controls: they
//! hold no focus and answer no activation (the M126/M129 container
//! rule, at the tree tier) — the controls the app mounts inside the
//! items hold the focus, and the words name facts, they wall nothing.

use alacer_core::announce::Politeness;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::style::Style;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};

/// The named collection the AT walks by disclosure and depth (the tree
/// tier's container half, M135, ADR-0139): the "reads as a
/// hierarchical collection" word the AT announces before it walks the
/// branches — an unnamed collection reads as a bare box, the set the
/// AT cannot name is a set it cannot walk (ADR-5).
pub struct Tree {
    name: String,
    /// The help text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    /// The multiselectable tier's word (M109, ADR-0113): whether the
    /// collection allows more than one pick — `false` the honest
    /// absence.
    multiselectable: bool,
    /// The busy tier's word (M117, ADR-0121): whether the collection's
    /// subtree is being updated — `false` the honest absence.
    busy: bool,
    /// The set tier's container word (M119, ADR-0123): the collection's
    /// declared item count (AccessKit's own placement: the size rides
    /// the container) — `None` the honest absence, the span the tree
    /// declared not.
    size_of_set: Option<usize>,
    /// The live tier's word (M120, ADR-0124): the tree's declared live
    /// region — `None` the honest absence, the container that declared
    /// no region.
    live: Option<Politeness>,
    style: Style,
}

impl Tree {
    /// A collection named `name` — the branches and leaves the app
    /// mounts read as its members.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            multiselectable: false,
            busy: false,
            size_of_set: None,
            live: None,
            style: Style {
                gap: 2.0,
                ..Style::column()
            },
        }
    }

    /// The description (M95, ADR-0099): the help text the a11y tree
    /// carries beside the name — the instruction the label is too short
    /// to carry. Without it the collection announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mounts the collection multi-pick (M109, ADR-0113): the
    /// capacity said by the tree from the first frame — AccessKit's
    /// `Multiselectable` property names the capacity to the AT beside
    /// the member picks the selected tier (ADR-0112) already carries.
    /// The word prices nothing but the word: the tree's doors answer
    /// exactly as they always did — the word names capacity, it walls
    /// nothing. The app's live door rides [`Tree::set_multiselectable`].
    #[must_use]
    pub fn multiselectable(mut self, multiselectable: bool) -> Self {
        self.multiselectable = multiselectable;
        self
    }

    /// The multiselectable door (M109, ADR-0113): the app's capacity
    /// moved — the word the tree says follows it on the re-announcement
    /// the caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). Report whether the
    /// word moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the multiselectable
    /// tier).
    pub fn set_multiselectable(&mut self, multiselectable: bool) -> bool {
        if self.multiselectable == multiselectable {
            return false;
        }
        self.multiselectable = multiselectable;
        true
    }

    /// Mounts the collection's updating word (M117, ADR-0121): the
    /// subtree's fact said by the tree from the first frame —
    /// AccessKit's `Busy` property names the region the AT holds its
    /// speech on while the stream is still writing it, the declared
    /// counterpart of the announcement machinery the framework already
    /// owns (§11.6, ADR-0024). The word prices nothing but the word:
    /// the tree's doors answer exactly as they always did — the word
    /// names the updating fact, it walls nothing. The app's live door
    /// rides [`Tree::set_busy`].
    #[must_use]
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    /// The busy door (M117, ADR-0121): the app's updating fact moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). Report whether the
    /// word moved — the caller requests `A11Y` only when this answers
    /// `true`, so a write that lands where the word already is queues
    /// nothing (the M21 idle-frame property, at the busy tier).
    pub fn set_busy(&mut self, busy: bool) -> bool {
        if self.busy == busy {
            return false;
        }
        self.busy = busy;
        true
    }

    /// Mounts the collection's span (M119, ADR-0123): the set tier's
    /// container word said by the tree from the first frame —
    /// AccessKit's `SizeOfSet` property names the collection's item
    /// count to the AT beside the member places the tier's
    /// `position_in_set` word carries. The word prices nothing but the
    /// word: the tree's doors answer exactly as they always did — the
    /// word names the span, it walls nothing. The app's live door
    /// rides [`Tree::set_size_of_set`].
    #[must_use]
    pub fn size_of_set(mut self, size_of_set: usize) -> Self {
        self.size_of_set = Some(size_of_set);
        self
    }

    /// The size door (M119, ADR-0123): the app's collection moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the span the tree declared
    /// not. Report whether the word moved — the caller requests `A11Y`
    /// only when this answers `true`, so a write that lands where the
    /// word already is queues nothing (the M21 idle-frame property, at
    /// the set tier).
    pub fn set_size_of_set(&mut self, size_of_set: Option<usize>) -> bool {
        if self.size_of_set == size_of_set {
            return false;
        }
        self.size_of_set = size_of_set;
        true
    }

    /// Mounts the collection's declared live region (M120, ADR-0124):
    /// the region whose changes the AT speaks, said by the tree from
    /// the first frame — AccessKit's `Live` property names the region
    /// to the AT, the politeness ([`Politeness`]) the announcement
    /// seam's own vocabulary carries, the declared counterpart of the
    /// pipeline's reserved region (§11.6, ADR-0024 — the imperative
    /// half the framework owns, the declarative half the app says).
    /// The word prices nothing but the word: the tree's doors answer
    /// exactly as they always did — the word names the region the AT
    /// watches, it walls nothing. The app's live door rides
    /// [`Tree::set_live`].
    #[must_use]
    pub fn live(mut self, live: Politeness) -> Self {
        self.live = Some(live);
        self
    }

    /// The live door (M120, ADR-0124): the app's declared region
    /// moved — the word the tree says follows it on the re-announcement
    /// the caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the container that declared
    /// no region. Report whether the word moved — the caller requests
    /// `A11Y` only when this answers `true`, so a write that lands
    /// where the word already is queues nothing (the M21 idle-frame
    /// property, at the live tier).
    pub fn set_live(&mut self, live: Option<Politeness>) -> bool {
        if self.live == live {
            return false;
        }
        self.live = live;
        true
    }

    /// Gap between the collection's members.
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

    /// Replaces the collection's accessible name in place (view
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

impl Widget for Tree {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Tree
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The description tier (M95, ADR-0099): the help text beside the
    /// name, the honest absence when the collection has nothing to
    /// explain.
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn multiselectable(&self) -> bool {
        self.multiselectable
    }

    fn busy(&self) -> bool {
        self.busy
    }

    fn size_of_set(&self) -> Option<usize> {
        self.size_of_set
    }

    fn live(&self) -> Option<Politeness> {
        self.live
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

/// One member of a hierarchical collection (the tree tier's member
/// half, M135, ADR-0139): the branch or leaf word the AT walks by — an
/// unnamed member reads as a bare mark, the ADR-5 rule the named roles
/// carry. The member's children ride the arena exactly a row's cells
/// do: nested items under a branch, the app's own controls under
/// either.
pub struct TreeItem {
    name: String,
    /// The selected tier's word (M108, ADR-0112): the member's declared
    /// pick — `None` the honest absence, the member of no selection
    /// collection.
    selected: Option<bool>,
    /// The expanded tier's word (M110, ADR-0114): the member's declared
    /// disclosure — `None` the honest absence, the member that opens
    /// and closes not.
    expanded: Option<bool>,
    /// The level tier's word (M113, ADR-0117): the member's declared
    /// hierarchical depth, zero-based (AccessKit's own base, beside
    /// ARIA's one-based `aria-level`) — `None` the honest absence, the
    /// depth the member declared not.
    level: Option<usize>,
    /// The set tier's member word (M119, ADR-0123): the member's
    /// declared place in the collection, zero-based (AccessKit's own
    /// base, beside ARIA's one-based `aria-posinset`) — `None` the
    /// honest absence, the unplaced member.
    position_in_set: Option<usize>,
    style: Style,
}

impl TreeItem {
    /// A member named `name` — the branch or leaf the AT walks by.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            selected: None,
            expanded: None,
            level: None,
            position_in_set: None,
            style: Style {
                gap: 4.0,
                padding: 2.0,
                ..Style::row()
            },
        }
    }

    /// Mounts the member picked (or explicitly unpicked) (M108,
    /// ADR-0112): the collection's word said by the tree from the
    /// first frame — AccessKit's `Selected` property names the pick to
    /// the AT beside the place the set tier already carries. The word
    /// prices nothing but the word: the member's doors answer exactly
    /// as they always did — the word names state, it walls nothing.
    /// The app's live door rides [`TreeItem::set_selected`].
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// The selected door (M108, ADR-0112): the app's selection moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the member of no selection
    /// collection. Report whether the word moved — the caller requests
    /// `A11Y` only when this answers `true`, so a write that lands
    /// where the word already is queues nothing (the M21 idle-frame
    /// property, at the selected tier).
    pub fn set_selected(&mut self, selected: Option<bool>) -> bool {
        if self.selected == selected {
            return false;
        }
        self.selected = selected;
        true
    }

    /// Mounts the member opened (or explicitly collapsed) (M110,
    /// ADR-0114): the collection member's disclosure said by the tree
    /// from the first frame — AccessKit's `Expanded` property names
    /// the disclosure to the AT beside the picks the selected tier
    /// already carries. The word prices nothing but the word: the
    /// member's doors answer exactly as they always did — the word
    /// names state, it walls nothing, the open and close routes the
    /// widget's own. The app's live door rides [`TreeItem::set_expanded`].
    #[must_use]
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// The expanded door (M110, ADR-0114): the app's disclosure moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the member that opens and
    /// closes not. Report whether the word moved — the caller requests
    /// `A11Y` only when this answers `true`, so a write that lands
    /// where the word already is queues nothing (the M21 idle-frame
    /// property, at the expanded tier).
    pub fn set_expanded(&mut self, expanded: Option<bool>) -> bool {
        if self.expanded == expanded {
            return false;
        }
        self.expanded = expanded;
        true
    }

    /// Mounts the member's depth (M113, ADR-0117): the nested member's
    /// place in the collection said by the tree from the first frame —
    /// AccessKit's `Level` property (zero-based, beside ARIA's
    /// one-based `aria-level`) names the depth to the AT beside the
    /// place the set tier already carries. The word prices nothing but
    /// the word: the member's doors answer exactly as they always did
    /// — the word names the depth, it walls nothing. The app's live
    /// door rides [`TreeItem::set_level`].
    #[must_use]
    pub fn level(mut self, level: usize) -> Self {
        self.level = Some(level);
        self
    }

    /// The level door (M113, ADR-0117): the app's depth moved — the
    /// word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the depth the member
    /// declared not. Report whether the word moved — the caller
    /// requests `A11Y` only when this answers `true`, so a write that
    /// lands where the word already is queues nothing (the M21
    /// idle-frame property, at the level tier).
    pub fn set_level(&mut self, level: Option<usize>) -> bool {
        if self.level == level {
            return false;
        }
        self.level = level;
        true
    }

    /// Mounts the member's place in its collection (M119, ADR-0123):
    /// the set tier's member word said by the tree from the first
    /// frame — AccessKit's `PositionInSet` property names the place to
    /// the AT beside the depth the level tier already carries. The
    /// word prices nothing but the word: the member's doors answer
    /// exactly as they always did — the word names the place, it walls
    /// nothing. The app's live door rides
    /// [`TreeItem::set_position_in_set`].
    #[must_use]
    pub fn position_in_set(mut self, position_in_set: usize) -> Self {
        self.position_in_set = Some(position_in_set);
        self
    }

    /// The position door (M119, ADR-0123): the app's collection moved —
    /// the word the tree says follows it on the re-announcement the
    /// caller's `PAINT | A11Y` request pays for (the mutate seam's
    /// shape, the way every app-side write rides). `None` clears the
    /// word — the honest absence restored, the unplaced member. Report
    /// whether the word moved — the caller requests `A11Y` only when
    /// this answers `true`, so a write that lands where the word
    /// already is queues nothing (the M21 idle-frame property, at the
    /// set tier).
    pub fn set_position_in_set(&mut self, position_in_set: Option<usize>) -> bool {
        if self.position_in_set == position_in_set {
            return false;
        }
        self.position_in_set = position_in_set;
        true
    }

    /// Layout styles.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Shared-borrow layout style access for view reconciliation (§6.1).
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    /// Replaces the member's accessible name in place (view
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

impl Widget for TreeItem {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::TreeItem
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn selected(&self) -> Option<bool> {
        self.selected
    }

    fn expanded(&self) -> Option<bool> {
        self.expanded
    }

    fn level(&self) -> Option<usize> {
        self.level
    }

    fn position_in_set(&self) -> Option<usize> {
        self.position_in_set
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::{Tree, TreeItem};
    use alacer_core::widget::{Role, Widget};

    #[test]
    fn roles_names_and_the_required_name_set() {
        let collection = Tree::new("Project files");
        let member = TreeItem::new("src");
        assert_eq!(collection.role(), Role::Tree);
        assert_eq!(member.role(), Role::TreeItem);
        assert_eq!(collection.label().resolve(), "Project files");
        assert_eq!(member.label().resolve(), "src");
        assert!(collection.role().requires_name());
        assert!(member.role().requires_name());
        assert_eq!(collection.role().name(), "tree");
        assert_eq!(member.role().name(), "tree_item");
        assert!(
            !collection.focusable() && !member.focusable(),
            "the collection and its members are not controls — the controls the app mounts inside hold the focus, never the containers"
        );
        assert_eq!(
            Widget::description(&collection),
            None,
            "an undescribed collection announces the absence"
        );
    }

    #[test]
    fn the_collection_words_ride_the_published_tiers() {
        // M109/M117/M119/M120: the words the container carries are the
        // tiers the tree already published — the constructor word is
        // the trait door's answer, and the door moves it live, the
        // idle-frame property on the write that lands where the word
        // already is.
        let mut collection = Tree::new("Project files")
            .multiselectable(true)
            .busy(true)
            .size_of_set(3);
        assert!(Widget::multiselectable(&collection), "the capacity rides");
        assert!(Widget::busy(&collection), "the updating word rides");
        assert_eq!(Widget::size_of_set(&collection), Some(3), "the span rides");
        assert!(
            !collection.set_size_of_set(Some(3)),
            "the write that lands where the span already is queues nothing"
        );
        assert!(collection.set_size_of_set(Some(5)), "the moved write lands");
        assert_eq!(Widget::size_of_set(&collection), Some(5));
        assert!(collection.set_size_of_set(None), "the door carries it away");
        assert!(
            Widget::size_of_set(&collection).is_none(),
            "the absence restored"
        );
        assert!(
            !collection.set_size_of_set(None),
            "clearing the absence twice queues nothing"
        );
        assert!(
            !collection.set_multiselectable(true),
            "the idle capacity write queues nothing"
        );
        assert!(
            collection.set_multiselectable(false),
            "the moved capacity write lands"
        );
        assert!(
            !collection.set_busy(true),
            "the busy door reports the word that already is"
        );
        assert!(collection.set_busy(false), "the moved busy write lands");
    }

    #[test]
    fn the_member_words_ride_the_published_tiers() {
        // M108/M110/M113/M119: the words the member says are the tiers
        // the tree already published — the pick, the disclosure, the
        // depth, the place — each door reporting the move, the idle
        // write queueing nothing.
        let mut member = TreeItem::new("src")
            .selected(true)
            .expanded(true)
            .level(2)
            .position_in_set(1);
        assert_eq!(Widget::selected(&member), Some(true), "the pick rides");
        assert_eq!(
            Widget::expanded(&member),
            Some(true),
            "the disclosure rides"
        );
        assert_eq!(
            Widget::level(&member),
            Some(2),
            "the depth rides — zero-based, AccessKit's own base"
        );
        assert_eq!(Widget::position_in_set(&member), Some(1), "the place rides");
        assert!(
            !member.set_selected(Some(true)),
            "the idle pick queues nothing"
        );
        assert!(member.set_selected(None), "the door carries the pick away");
        assert!(Widget::selected(&member).is_none(), "the absence restored");
        assert!(
            !member.set_expanded(Some(true)),
            "the idle disclosure queues nothing"
        );
        assert!(
            member.set_expanded(Some(false)),
            "the moved disclosure lands"
        );
        assert_eq!(Widget::expanded(&member), Some(false));
        assert!(!member.set_level(Some(2)), "the idle depth queues nothing");
        assert!(member.set_level(None), "the door carries the depth away");
        assert!(Widget::level(&member).is_none(), "the flat member restored");
        assert!(
            !member.set_position_in_set(Some(1)),
            "the idle place queues nothing"
        );
        assert!(member.set_position_in_set(Some(0)), "the moved place lands");
        assert_eq!(Widget::position_in_set(&member), Some(0));
    }

    #[test]
    fn a_member_that_declared_nothing_announces_nothing() {
        // The honest absences: a member of no selection collection, a
        // member that opens and closes not, a depth and a place never
        // guessed — `None` the answer every absent word spells, never
        // a `false` or a `0`.
        let member = TreeItem::new("Cargo.toml");
        assert!(Widget::selected(&member).is_none());
        assert!(Widget::expanded(&member).is_none());
        assert!(Widget::level(&member).is_none());
        assert!(Widget::position_in_set(&member).is_none());
    }

    #[test]
    fn the_name_write_moves_the_word() {
        let mut member = TreeItem::new("src");
        member.set_name("lib");
        assert_eq!(member.label().resolve(), "lib");
        let mut collection = Tree::new("Project files");
        collection.set_name("Workspace files");
        assert_eq!(collection.label().resolve(), "Workspace files");
    }

    #[test]
    fn the_description_write_reports_the_move() {
        // M95, ADR-0099: the honest absence by default, the help text
        // the builder mounts, and the one write whose moved answer is
        // the only mask the tier costs (the M21 idle-frame property).
        let mut collection = Tree::new("Project files");
        collection = collection.description("The workspace's files");
        assert_eq!(
            Widget::description(&collection),
            Some("The workspace's files"),
            "the help text the tree carries beside the name"
        );
        assert!(
            !collection.set_description(Some("The workspace's files".to_string())),
            "the idle write answers false — no masks"
        );
        assert!(
            collection.set_description(Some("The repo's files".to_string())),
            "the moved write answers true"
        );
        assert!(
            collection.set_description(None),
            "clearing to the absence moved"
        );
        assert!(
            !collection.set_description(None),
            "the absence is already there"
        );
    }

    #[test]
    fn the_style_words_stay_layout_words() {
        let mut collection = Tree::new("Project files").gap(4.0).padding(8.0).grow(1.0);
        collection.style_mut().gap = 6.0;
        let style = collection.style();
        assert_eq!(style.gap, 6.0, "the shared-borrow door patches in place");
        assert_eq!(style.padding, 8.0);
    }
}
