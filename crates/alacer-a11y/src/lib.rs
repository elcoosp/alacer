//! Alacer's accessibility: the AccessKit contract (ADR-5, M3 edition).
//!
//! §11.1, implemented:
//!
//! - AccessKit `NodeId`s are the id-path hashes — the same identity as
//!   hit-testing (§6.3); the a11y tree *is* the widget tree.
//! - Updates are **incremental deltas** (§11.1 item 4): pass 9 hands us an
//!   [`A11yDelta`] — dirty ids, removals, focus — and only dirty nodes are
//!   re-emitted in the [`TreeUpdate`] (never a full-tree rebuild per frame,
//!   the egui cost we refuse). The M3 exit gate measures exactly this:
//!   **`TreeUpdate` p99 < 0.5 ms @ 1k nodes** (`tests/m3_exit_criteria.rs`).
//! - Roles map from the core `Role` enum; labels resolve through the
//!   required `label()` source — an unnamed widget is impossible by
//!   construction. Editable widgets expose their content as the node's value.
//! - Focus travels in the delta: `TreeUpdate::focus` follows the pipeline's
//!   pass-3 output every a11y frame.

use accesskit::{
    AutoComplete, Invalid, Live, Node, NodeId, Rect, Role as AccessKitRole, TextPosition,
    TextSelection, Toggled, TreeId, TreeInfo, TreeUpdate,
};
use alacer_core::Role;
use alacer_core::announce::Politeness;
use alacer_core::geom::Rect as AlacerRect;
use alacer_core::passes::{A11yDelta, A11ySink, TreeSnapshot};
use alacer_core::widget::TableSlot;
use rustc_hash::FxHashSet;
use std::collections::VecDeque;

/// Builds an AccessKit node id from its raw hash (§6.3: same identity).
#[must_use]
pub fn accesskit_node_id(hash: u64) -> NodeId {
    NodeId(hash)
}

/// Maps Alacer's semantic roles onto AccessKit roles. The M6 canon roles
/// (checkbox, table family) land on their AccessKit counterparts so a real
/// platform adapter sees a checkbox and a navigable grid.
#[must_use]
pub fn map_role(role: Role) -> AccessKitRole {
    match role {
        Role::Decoration => AccessKitRole::Unknown,
        Role::Container => AccessKitRole::GenericContainer,
        Role::Label => AccessKitRole::Label,
        Role::Button => AccessKitRole::Button,
        Role::TextInput => AccessKitRole::TextInput,
        Role::ScrollArea => AccessKitRole::ScrollView,
        Role::ListItem => AccessKitRole::ListItem,
        Role::List => AccessKitRole::List,
        Role::CheckBox => AccessKitRole::CheckBox,
        // The slider (M20, ADR-0026): the AT reads the bounded value the
        // widget exposes through the M3 `value()` contract and writes it
        // back through the value tier (`SetValue`, `Increment`,
        // `Decrement`).
        Role::Slider => AccessKitRole::Slider,
        // The progress indicator (M21, ADR-0027): the AT reads the value
        // the app reports through the M3 `value()` contract — the read
        // half of the value tier, mapped faithfully.
        Role::ProgressIndicator => AccessKitRole::ProgressIndicator,
        Role::Table => AccessKitRole::Table,
        Role::TableRow => AccessKitRole::Row,
        Role::TableCell => AccessKitRole::Cell,
        Role::ColumnHeader => AccessKitRole::ColumnHeader,
        // The canvas (M82): a free-form custom-paint surface. AccessKit's
        // canvas role says exactly that — the AT announces the name and
        // nothing more; a canvas has no write door and no value tier.
        Role::Canvas => AccessKitRole::Canvas,
        // The announcement region (§11.6, ADR-0024): the polite case's ARIA
        // role ("status"). The assertive case re-roles to "alert" in
        // [`map_announcement_role`], where the politeness is known.
        Role::LiveRegion => AccessKitRole::Status,
        // The radio (M124, ADR-0128): the one-of-N control. AccessKit's
        // `RadioButton` role carries the selection the `toggled` contract
        // already announces for the checkbox (the same `Checked` state the
        // sink maps at the toggle tier) — the AT reads "checked" on the
        // member the set holds.
        Role::Radio => AccessKitRole::RadioButton,
        // The radio group (M124, ADR-0128): the named container the set
        // reads as. AccessKit's radio group role says exactly that — the
        // AT announces the group's name before it walks the members; the
        // set tier's span word (`SizeOfSet`) rides the same node.
        Role::RadioGroup => AccessKitRole::RadioGroup,
        // The link (M125, ADR-0129): the "opens something" fact
        // AccessKit's own link role spells — the activation verb the AT
        // names differently from a button's press, the destination the
        // app's to say (the name and the description carry it).
        Role::Link => AccessKitRole::Link,
        // The heading (M126, ADR-0130): the "names a section" fact
        // AccessKit's own heading role spells — the document outline's
        // landmark the AT's quick-nav walks. The depth rides the level
        // word the M113 tier published (the `Level` property the sink
        // maps at the level tier), no second branch.
        Role::Heading => AccessKitRole::Heading,
        // The tab (M127, ADR-0131): the "selects a page" fact
        // AccessKit's own tab role spells. The selection rides the
        // selected word the M108 tier published (the `Selected`
        // property the sink maps at the selected tier), no second
        // branch.
        Role::Tab => AccessKitRole::Tab,
        // The tab list (M127, ADR-0131): the named strip the pages read
        // as. AccessKit's tab list role says exactly that — the AT
        // announces the strip's name before it walks the tabs.
        Role::TabList => AccessKitRole::TabList,
        // The spin button (M128, ADR-0132): the "writes one bounded
        // number" fact AccessKit's own spin button role spells — the
        // field the AT walks by arrows. The value words ride the value
        // tier's own properties (the `NumericValue` /
        // `MinNumericValue` / `MaxNumericValue` / step set the sink
        // maps at the M97/M99 tiers), no second branch.
        Role::SpinButton => AccessKitRole::SpinButton,
        // The toolbar (M129, ADR-0133): the named strip the tools read
        // as. AccessKit's toolbar role says exactly that — the AT
        // announces the strip's name before it walks the tools; the
        // tools keep the roles the members already carry.
        Role::Toolbar => AccessKitRole::Toolbar,
        // The switch (M130, ADR-0134): the instant on/off control.
        // AccessKit's switch role says exactly that — the AT announces
        // the on/off fact, never the tri-state summary a checkbox
        // resolves. The binary pair rides the `Toggled` property the
        // sink has mapped since the toggle tier (the M6 contract's
        // `toggled` half), no second branch.
        Role::Switch => AccessKitRole::Switch,
        // The tab panel (M131, ADR-0135): the named pane the selected
        // page's content lives in. AccessKit's tab panel role says
        // exactly that — the AT announces the pane's name before it
        // reads the content; the children keep the roles the members
        // already carry, the tab tier's member-and-container shape
        // grown to its third word (the M127 tab/tab-list pair beside
        // this branch), no patch-tier growth.
        Role::TabPanel => AccessKitRole::TabPanel,
        // The meter (M133, ADR-0137): the "reads as a gauge" fact
        // AccessKit's own meter role spells — a scalar measurement on
        // the scale the app declared, never the progress tier's task
        // word. The reading rides the value tier's own properties (the
        // `NumericValue` / `MinNumericValue` / `MaxNumericValue` set
        // the sink maps at the M97/M99 tiers), no second branch.
        Role::Meter => AccessKitRole::Meter,
        // The dialog (M134, ADR-0138): the "reads as a modal window"
        // fact AccessKit's own dialog role spells — the named container
        // the AT announces before it walks the conversation; the
        // children keep the roles the members already carry, the
        // M126/M129 container rule at the modal tier, no patch-tier
        // growth.
        Role::Dialog => AccessKitRole::Dialog,
        // The tree (M135, ADR-0139): the "reads as a hierarchical
        // collection" fact AccessKit's own tree role spells — the named
        // container the AT announces before it walks the branches; the
        // members keep the roles the arena's own nesting gives them,
        // the M126/M129 container rule at the hierarchical tier, no
        // patch-tier growth.
        Role::Tree => AccessKitRole::Tree,
        // The tree item (M135, ADR-0139): the "reads as a branch or a
        // leaf" fact AccessKit's own tree-item role spells. The
        // disclosure, pick, depth and place ride the words the M108,
        // M110, M113 and M119 tiers published (the properties the sink
        // maps generically), never a second vocabulary.
        Role::TreeItem => AccessKitRole::TreeItem,
        // The tooltip (M136, ADR-0140): the named help bubble the
        // pointer pair summons. AccessKit's tooltip role says exactly
        // that — the AT announces the bubble while it is up; the
        // children keep the roles the members already carry, the
        // M126/M129 container rule at the hover tier, no patch-tier
        // growth.
        Role::Tooltip => AccessKitRole::Tooltip,
    }
}

/// Maps an announcement's politeness onto the AccessKit live priority —
/// the `aria-live` property a screen reader watches (`Live::Polite` speaks
/// at the next break, `Live::Assertive` interrupts).
#[must_use]
pub const fn map_live(politeness: Politeness) -> Live {
    match politeness {
        Politeness::Polite => Live::Polite,
        Politeness::Assertive => Live::Assertive,
    }
}

/// The announcement node's role for its politeness: the ARIA pairing —
/// `role="status"` speaks politely, `role="alert"` interrupts. The base
/// mapping ([`map_role`]) answers `Status` for the role alone; the sink
/// re-roles where the politeness is known.
#[must_use]
pub const fn map_announcement_role(politeness: Politeness) -> AccessKitRole {
    match politeness {
        Politeness::Polite => AccessKitRole::Status,
        Politeness::Assertive => AccessKitRole::Alert,
    }
}

/// Converts an Alacer rect into AccessKit's `f64`-based rect.
#[must_use]
pub fn map_bounds(rect: AlacerRect) -> Rect {
    Rect::new(
        f64::from(rect.min_x()),
        f64::from(rect.min_y()),
        f64::from(rect.max_x()),
        f64::from(rect.max_y()),
    )
}

/// The AccessKit sink: incremental [`TreeUpdate`]s from [`A11yDelta`]s.
///
/// Produced updates accumulate in a bounded ring (see [`UPDATE_RING_CAP`]):
/// a windowed app drains them at frame boundaries (ADR-0015), and an
/// undrained sink — an app with no adapter attached — is capped instead of
/// growing forever.
#[derive(Debug, Default)]
pub struct AccessKitSink {
    updates: Vec<TreeUpdate>,
    initialized: bool,
    last_snapshot: Option<std::sync::Arc<TreeSnapshot>>,
    /// The ids included in the most recent update (harness/test introspection).
    last_update_ids: FxHashSet<u64>,
    /// The ids the most recent update dropped (dead widgets; introspection).
    last_removed_ids: Vec<u64>,
    update_count: u64,
}

/// How many produced updates the sink retains before the oldest one is
/// dropped. An assistive technology that connects mid-run re-requests the
/// whole tree ([`AccessKitSink::full_tree_update`]), so dropping stale
/// deltas never loses correctness — it only bounds memory.
pub const UPDATE_RING_CAP: usize = 8;

impl AccessKitSink {
    /// An empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of updates produced so far.
    #[must_use]
    pub const fn update_count(&self) -> u64 {
        self.update_count
    }

    /// The most recent update (test/platform consumption).
    #[must_use]
    pub fn last_update(&self) -> Option<&TreeUpdate> {
        self.updates.last()
    }

    /// The ids re-emitted by the most recent update.
    #[must_use]
    pub fn last_update_ids(&self) -> &FxHashSet<u64> {
        &self.last_update_ids
    }

    /// The ids dropped by the most recent update (dead widgets).
    #[must_use]
    pub fn last_removed_ids(&self) -> &[u64] {
        &self.last_removed_ids
    }

    /// Takes every update produced since the last drain, in order — the
    /// platform's steady-state frame-boundary step (ADR-0015): the loop
    /// hands each one to the AccessKit adapter, which forwards it while an
    /// assistive technology is connected and drops it otherwise.
    ///
    /// Draining is the memory contract, too: a windowed app that never
    /// calls this (no adapter attached) is still bounded by
    /// [`UPDATE_RING_CAP`].
    #[must_use]
    pub fn drain_updates(&mut self) -> Vec<TreeUpdate> {
        std::mem::take(&mut self.updates)
    }

    /// The full current tree as one `TreeUpdate` — the response to an
    /// adapter-activation request (`InitialTreeRequested`), built by the
    /// same node builder the deltas use. Pure: no bookkeeping changes, no
    /// update enters the ledger. `None` before the first
    /// [`A11ySink::apply`] (no resolved tree yet — the platform defers to
    /// the first frame, whose init delta covers it).
    ///
    /// Since M67 (ADR-0073) the build echoes: the a11y tier notes its own
    /// fact on the trail ([`alacer_core::a11y_echo::note_a11y_echo`],
    /// the tree named by its node count — the lengths rule, the payload
    /// never rides), the fact streaming at the next tick's start drain.
    /// The note is the trail's one branch while nobody listens (the A7
    /// idle contract): the build's own purity is unchanged — no
    /// bookkeeping changes, no update enters the ledger.
    #[must_use]
    pub fn full_tree_update(&self) -> Option<TreeUpdate> {
        let snapshot = self.last_snapshot.as_ref()?;
        let delta = A11yDelta {
            initializes: true,
            ..A11yDelta::empty()
        };
        let update = Self::build_update(snapshot, &delta, true, &mut FxHashSet::default());
        alacer_core::a11y_echo::note_a11y_echo(alacer_core::A11yEchoRecord {
            nodes: update.nodes.len(),
        });
        Some(update)
    }

    /// Builds one [`TreeUpdate`] from a snapshot and its delta — the single
    /// materialization path for every update this crate ever produces.
    /// `force_full` (an initializing delta, or the sink's first frame) builds
    /// the whole tree; otherwise only the updated ids re-emit (§11.1 item 4 —
    /// the p99 budget's whole point). `ids` receives the emitted ids (the
    /// sink's introspection bookkeeping; a scratch set when the caller wants
    /// a pure build, as [`Self::full_tree_update`] does).
    fn build_update(
        snapshot: &TreeSnapshot,
        delta: &A11yDelta,
        force_full: bool,
        ids: &mut FxHashSet<u64>,
    ) -> TreeUpdate {
        ids.clear();
        let mut update = TreeUpdate {
            nodes: Vec::new(),
            tree: None,
            tree_id: TreeId::ROOT,
            focus: delta.focus.map_or(NodeId(0), accesskit_node_id),
        };

        if force_full {
            update.tree = snapshot
                .root
                .map(|root| TreeInfo::new(accesskit_node_id(root)));
        }
        for node in &snapshot.nodes {
            if !force_full && !delta.updated.contains(&node.id) {
                continue;
            }
            ids.insert(node.id);
            update
                .nodes
                .push((accesskit_node_id(node.id), Self::access_node(node)));
        }
        update
    }

    /// Converts one neutral snapshot node into an AccessKit node. The M6
    /// vocabulary rides along: toggle state becomes the `Toggled` property,
    /// and table slots become the grid attributes (row/column counts and
    /// zero-based indexes) that make a table navigable, not just visible.
    /// Since M95 (ADR-0099) the description rides along too: the help text
    /// beside the name, the `Description` property the AT reads when the
    /// name alone does not answer.
    #[expect(
        clippy::too_many_lines,
        reason = "the sink's whole-node mapping is one readable property-by-property walk; splitting it would hide the tier mappings the ADRs name"
    )]
    fn access_node(node: &alacer_core::passes::TreeNode) -> Node {
        let mut access_node = Node::new(map_role(node.role));
        access_node.set_label(node.label.as_str());
        if let Some(bounds) = node.bounds {
            access_node.set_bounds(map_bounds(bounds));
        }
        // The announcement region (§11.6, ADR-0024) and, since M120
        // (ADR-0124), the app's declared live regions: the politeness
        // rides the `aria-live` property on both — and the role follows
        // the ARIA pairing (polite is "status", assertive interrupts as
        // "alert") for the pipeline's own region only, whose
        // announcement contract names the pairing. A container that
        // *declares* the word keeps the role it already was — the word
        // names the region the AT watches, it re-roles nothing.
        if let Some(politeness) = node.live {
            access_node.set_live(map_live(politeness));
            if node.role == Role::LiveRegion {
                access_node.set_role(map_announcement_role(politeness));
            }
        }
        if let Some(value) = &node.value {
            access_node.set_value(value.as_str());
        }
        // The description tier (M95, ADR-0099): the help text beside the
        // name — AccessKit's `Description` property, ARIA's
        // `aria-description`. The honest absence maps to the absence: a
        // widget with nothing to explain sets nothing, and the AT reads
        // the node exactly as it read the tree M94 handed over.
        if let Some(description) = &node.description {
            access_node.set_description(description.as_str());
        }
        // The placeholder tier (M96, ADR-0100): the hint an empty
        // editable announces before the user types — AccessKit's
        // `Placeholder` property. The honest absence maps to the
        // absence: a field with no hint sets nothing.
        if let Some(placeholder) = &node.placeholder {
            access_node.set_placeholder(placeholder.as_str());
        }
        // The role-description tier (M116, ADR-0120): the role a custom
        // control is *read as* — AccessKit's `RoleDescription`
        // property, ARIA's `aria-roledescription` — the specialization
        // the AT reads beside the generic role the `Role` mapping
        // already spells. The word prices nothing but the word: the
        // control's doors answer exactly as they always did. No kind
        // enum to mirror — the property is a string (the description
        // precedent, ADR-0099, at the string tier) — and the honest
        // absence (`None`) maps to the absence: a control that claims
        // no specialization sets nothing, is read as the generic role
        // it already was. The property rides the same node-only delta
        // as every other property — a role-description write
        // re-announces exactly that node.
        if let Some(role_description) = &node.role_description {
            access_node.set_role_description(role_description.as_str());
        }
        // The set tier (M119, ADR-0123): the collection's span and the
        // member's place map AccessKit's `SizeOfSet` (ARIA's
        // `aria-setsize`) and `PositionInSet` (ARIA's `aria-posinset`)
        // properties — the numbers the AT reads as "row 3 of 500"
        // beside the picks, the capacity and the depth the collection
        // tiers already carried. The words price nothing but the
        // words: the collection's doors answer exactly as they always
        // did. AccessKit's own contracts ride — the size on the
        // container ("`aria-setsize` is set on each item, while this
        // property is set on the container"), the place zero-based
        // ("`aria-posinset` is one-based, while this property is
        // zero-based") — the differences the trait's documentation
        // carries, one branch with no translation. The honest absence
        // (`None`) maps to the absence: a collection that declared
        // nothing and a member that declared no place set nothing,
        // never a guessed count. The properties ride the same
        // node-only delta as every other property — a set write
        // re-announces exactly that node.
        if let Some(size_of_set) = node.size_of_set {
            access_node.set_size_of_set(size_of_set);
        }
        if let Some(position_in_set) = node.position_in_set {
            access_node.set_position_in_set(position_in_set);
        }
        // The shortcut tier (M121, ADR-0125): the chord the control is
        // invoked with — AccessKit's `KeyboardShortcut` property, ARIA's
        // `aria-keyshortcuts` — the word the AT announces beside the
        // name ("Save — Control+S") before it walks to the keymap's
        // dispatch (ADR-0043's consult, the trail leg the framework's
        // own chords ride). The word prices nothing but the word: the
        // control's doors answer exactly as they always did. The word is
        // the string the app named — the property is a string, the
        // spellings the app's vocabulary (the description precedent,
        // ADR-0099, at the string tier), no enum to mirror — and the
        // honest absence (`None`) maps to the absence: a control that
        // declared no chord sets nothing, never a guessed binding. The
        // property rides the same node-only delta as every other
        // property — a shortcut write re-announces exactly that node.
        if let Some(shortcut) = &node.keyboard_shortcut {
            access_node.set_keyboard_shortcut(shortcut.as_str());
        }
        // The multiline tier (M122, ADR-0126): the field's line capacity —
        // AccessKit's `MultilineTextInput` role (the capacity word the
        // vocabulary carries; there is no separate property to mirror),
        // ARIA's `aria-multiline` beside the `textbox` role — the word
        // the AT reads beside the hint the placeholder tier already
        // carried ("this field wraps onto many lines"). The word prices
        // nothing but the word: the field's doors answer exactly as they
        // always did — the word names the capacity, it walls nothing. The
        // honest absence (`false`) maps to the absence: a field that
        // declared no capacity reads as the ordinary `TextInput` role it
        // always was, never a guessed "multiline". The tier rides the
        // same role branch the secure tier resolved first (M100): the
        // secure field's `PasswordInput` word wins the branch it has
        // carried since M100, and the capacity word resolves only when
        // the role is still the ordinary editable one — both words stay
        // true in the snapshot the tree hands over. A capacity write
        // re-announces exactly that node, the same node-only delta as
        // every other property.
        // The secured-and-hidden field keeps the `PasswordInput` word it
        // resolved above (the M100 branch): the mask the word prices
        // yields to the reveal (M102, ADR-0106), so the capacity word
        // resolves only when the role is still the ordinary editable
        // one — both words stay true in the snapshot the tree hands
        // over. A capacity write re-announces exactly that node, the
        // same node-only delta as every other property.
        let hidden_secure = node.secure_entry && !node.secure_revealed;
        if node.multiline && node.role == alacer_core::Role::TextInput && !hidden_secure {
            access_node.set_role(AccessKitRole::MultilineTextInput);
        }
        // The purpose tier (M132, ADR-0136): a field declared for search
        // maps AccessKit's `SearchInput` role — the word every platform
        // AT announces and the find conventions key off ("search, edit
        // text"). One role per node: the tree never guesses a composite —
        // the masked contract first (the hidden secure field keeps the
        // `PasswordInput` word the M100 branch resolves below), the
        // capacity word claiming a field that declared both (a multiline
        // search box keeps the `MultilineTextInput` word the M122 branch
        // resolved — the re-open rule the ADR pre-registers beside it),
        // and the purpose word re-rolling the single-line, non-secured
        // field. The branch prices nothing: a declared field's doors
        // answer exactly as they always did.
        if node.search_field
            && node.role == alacer_core::Role::TextInput
            && !node.multiline
            && !hidden_secure
        {
            access_node.set_role(AccessKitRole::SearchInput);
        }
        // The language tier (M123, ADR-0127): the language the widget's
        // text is written in — AccessKit's `Language` property, the BCP
        // 47 tag the platform AT voices the text with — the word the AT
        // reads beside the text before it speaks ("this label is
        // French"). The word prices nothing but the word: the widget's
        // doors answer exactly as they always did — the word names the
        // language, it walls nothing. The tag is the string the app
        // declared — the property is a string (the description
        // precedent, ADR-0099, at the string tier), no enum to mirror —
        // and the honest absence (`None`) maps to the absence: a widget
        // that declared no language sets nothing, never a guessed tag.
        // The property rides the same node-only delta as every other
        // property — a language write re-announces exactly that node.
        if let Some(language) = &node.language {
            access_node.set_language(language.as_str());
        }
        // The range tier (M97, ADR-0101): the span every value-tier write
        // funnels through — AccessKit's `MinNumericValue` /
        // `MaxNumericValue` properties, ARIA's `aria-valuemin` /
        // `aria-valuemax`, with the declared step beside them when the
        // widget declares one. The honest absence maps to the absence: a
        // widget with no span sets nothing, never a guessed default.
        if let Some(range) = &node.value_range {
            access_node.set_min_numeric_value(range.min);
            access_node.set_max_numeric_value(range.max);
            if let Some(step) = range.step {
                access_node.set_numeric_value_step(step);
            }
        }
        // The numeric tier (M99, ADR-0103): the number behind the value's
        // spoken word — AccessKit's `NumericValue` property, ARIA's
        // `aria-valuenow`, riding the text the tree already carried as
        // the value's `aria-valuetext` (the answer ADR-0101's re-open
        // rule pre-registered: one setter beside the walls). The honest
        // absence maps to the absence: a widget whose value is prose
        // sets no number, never a parsed guess.
        if let Some(now) = node.numeric_value {
            access_node.set_numeric_value(now);
        }
        // The secure tier (M100, ADR-0104): a field declared secure maps
        // AccessKit's `PasswordInput` role — the word every platform AT
        // announces ("secure text entry"), the declared-but-unreported
        // gap the IME contract has carried since M4. The value the node
        // carries is the mask pass 9 resolved (never the secret — the
        // role's own contract, and the tree's payload rule). The tier
        // maps no other role: a secure checkbox is not a word the
        // vocabulary declares, and a non-input widget's answer is the
        // honest absence.
        // The reveal tier (M102, ADR-0106): the mapping yields to the
        // revealed state — AccessKit's own contract names it ("when
        // the password is revealed, use `Role::TextInput` instead").
        // A revealed field reads as the ordinary editable word, the
        // value beside it the field's ordinary value (the mask
        // yielding at the snapshot's own resolution); a field that
        // revealed nothing answers exactly the word M100 mapped.
        if node.secure_entry && node.role == alacer_core::Role::TextInput && !node.secure_revealed {
            access_node.set_role(AccessKitRole::PasswordInput);
        }
        // The orientation tier (M98, ADR-0102): the scroll container's
        // declared axis — AccessKit's `Orientation` property, ARIA's
        // `aria-orientation`, the word the AT's arrows read before they
        // ride. The honest absence maps to the absence: a container that
        // declared both axes says the word ARIA spells `undefined` (the
        // spec's own name for "both axes") — the property stays unset
        // rather than a wrong half, and AccessKit's enum carries no
        // both variant to guess with.
        if let Some(orientation) = node.scroll_orientation {
            access_node.set_orientation(match orientation {
                alacer_core::widget::ScrollOrientation::Vertical => {
                    accesskit::Orientation::Vertical
                }
                alacer_core::widget::ScrollOrientation::Horizontal => {
                    accesskit::Orientation::Horizontal
                }
            });
        }
        // The scroll position tier (M101, ADR-0105): where the scroll
        // doors landed and the walls they clamp to — AccessKit's
        // `ScrollY` / `ScrollYMin` / `ScrollYMax` properties (the
        // `ScrollX` halves beside them on a two-axis container), the
        // arithmetic the platform ATs announce as "scrolled 200 of
        // 0..600". Each axis rides its own honesty: an axis whose span
        // is empty sets nothing — the same wall every scroll door
        // answers `no_scrollable_content` with — and the floor the
        // funnel clamps to is carried, not guessed: the clamp
        // arithmetic's own zero. The honest absence maps to the
        // absence: a container with no live axis sets nothing, and the
        // AT reads the node exactly as it read the tree M100 handed
        // over.
        if let Some(position) = &node.scroll_position {
            if let Some(vertical) = &position.vertical {
                access_node.set_scroll_y(alacer_core::num::f32_to_f64(vertical.offset));
                access_node.set_scroll_y_min(0.0);
                access_node.set_scroll_y_max(alacer_core::num::f32_to_f64(vertical.max));
            }
            if let Some(horizontal) = &position.horizontal {
                access_node.set_scroll_x(alacer_core::num::f32_to_f64(horizontal.offset));
                access_node.set_scroll_x_min(0.0);
                access_node.set_scroll_x_max(alacer_core::num::f32_to_f64(horizontal.max));
            }
        }
        // The availability tier (M22, ADR-0028): the widget's `enabled`
        // answer becomes AccessKit's `disabled` flag, so the AT reads an
        // unavailable control instead of announcing an action the widget
        // will refuse. The default is the honest absence: an available
        // control sets nothing. The flag rides the same node-only delta
        // as every other property — an availability write re-announces
        // exactly that node.
        if !node.enabled {
            access_node.set_disabled();
        }
        // The read-only tier (M105, ADR-0109): the field whose value is
        // the app's maps AccessKit's `ReadOnly` flag, ARIA's
        // `aria-readonly` — the word the AT reads beside the value, the
        // declared-but-unreported gap the write walls priced since M55
        // ("a read-only field" had no tree word). The honest absence
        // maps to the absence: an editable field sets nothing, and the
        // AT reads the node exactly as it read the tree M104 handed
        // over. The flag rides the same node-only delta as every other
        // property — a read-only write re-announces exactly that node.
        if node.read_only {
            access_node.set_read_only();
        }
        // The required tier (M106, ADR-0110): the control that must carry
        // a value before the form's submit fires maps AccessKit's
        // `Required` flag, ARIA's `aria-required` — the word the AT reads
        // before it asks. The word prices nothing but the word: the
        // control's doors answer exactly as they always did. The honest
        // absence maps to the absence: a control that declared nothing
        // sets nothing, never a guessed default. The flag rides the same
        // node-only delta as every other property — a required write
        // re-announces exactly that node.
        if node.required {
            access_node.set_required();
        }
        // The invalid tier (M107, ADR-0111): the control whose value
        // fails the form's contract maps AccessKit's `Invalid` property,
        // ARIA's `aria-invalid` — the rejection the AT reads while the
        // user edits, the kind (`true` / `grammar` / `spelling`) riding
        // beside the word. The word prices nothing but the word: the
        // control's doors answer exactly as they always did. The honest
        // absence maps to the absence: a control that declared nothing
        // sets nothing, never a guessed default. The property rides the
        // same node-only delta as every other property — an invalid
        // write re-announces exactly that node.
        if let Some(kind) = node.invalid {
            access_node.set_invalid(match kind {
                alacer_core::widget::InvalidKind::True => Invalid::True,
                alacer_core::widget::InvalidKind::Grammar => Invalid::Grammar,
                alacer_core::widget::InvalidKind::Spelling => Invalid::Spelling,
            });
        }
        // The selected tier (M108, ADR-0112): the collection member the
        // collection picked maps AccessKit's `Selected` property, ARIA's
        // `aria-selected` — the pick the AT reads beside the position the
        // M6 slot already carried. The word prices nothing but the word:
        // the member's doors answer exactly as they always did. The honest
        // trio maps to the honest trio: `None` the member of no selection
        // collection sets nothing (the absence ARIA spells `undefined`,
        // never a guessed `false`), `Some(false)` the member standing
        // unpicked beside its picked siblings, `Some(true)` the pick. The
        // property rides the same node-only delta as every other property
        // — a selected write re-announces exactly that node.
        if let Some(selected) = node.selected {
            access_node.set_selected(selected);
        }
        // The multiselectable tier (M109, ADR-0113): the container that
        // allows more than one pick maps AccessKit's `Multiselectable`
        // property, ARIA's `aria-multiselectable` — the capacity the AT
        // reads beside the member picks the selected tier already
        // carried. The word prices nothing but the word: the container's
        // doors answer exactly as they always did. The honest absence
        // maps to the absence: a container that declared nothing sets
        // nothing, never a guessed capacity. The property rides the same
        // node-only delta as every other property — a multiselectable
        // write re-announces exactly that node.
        if node.multiselectable {
            access_node.set_multiselectable();
        }
        // The busy tier (M117, ADR-0121): the container whose subtree is
        // being updated maps AccessKit's `Busy` property, ARIA's
        // `aria-busy` — the updating fact the AT defers its speech on,
        // beside the announcement region the imperative door already
        // reserved (§11.6, ADR-0024: the declared counterpart — the app
        // says *which* region is settling, the AT holds what it would
        // announce while it is). The word prices nothing but the word:
        // the container's doors answer exactly as they always did. The
        // honest absence maps to the absence: a container that declared
        // nothing sets nothing, never a guessed "updating". The property
        // rides the same node-only delta as every other property — a
        // busy write re-announces exactly that node.
        if node.busy {
            access_node.set_busy();
        }
        // The expanded tier (M110, ADR-0114): the collection member that
        // opens and closes maps AccessKit's `Expanded` property, ARIA's
        // `aria-expanded` — the disclosure the AT reads beside the picks
        // the selected tier already carried. The word prices nothing but
        // the word: the member's doors answer exactly as they always
        // did. The honest trio maps to the honest trio: `None` the
        // member that opens and closes not sets nothing (the absence
        // ARIA spells `undefined`, never a guessed `false`),
        // `Some(false)` the member standing collapsed, `Some(true)` the
        // member standing open — AccessKit's own contract naming the
        // mapping ("setting this to `false` means the node is collapsed;
        // omitting it means this state isn't applicable"), one branch
        // with no translation. The property rides the same node-only
        // delta as every other property — an expanded write
        // re-announces exactly that node.
        if let Some(expanded) = node.expanded {
            access_node.set_expanded(expanded);
        }
        // The auto-complete tier (M111, ADR-0115): the field whose
        // completions ride a suggestion contract maps AccessKit's
        // `AutoComplete` property, ARIA's `aria-autocomplete` — the
        // contract the AT reads beside the hint the placeholder tier
        // already carried. The word prices nothing but the word: the
        // field's doors answer exactly as they always did. The kind
        // mirrors AccessKit's own enum values — one branch with no
        // translation — and the honest absence (`None`, ARIA's `none`)
        // maps to the absence: a field that declared nothing sets
        // nothing, never a guessed mode. The property rides the same
        // node-only delta as every other property — an auto-complete
        // write re-announces exactly that node.
        if let Some(kind) = node.auto_complete {
            access_node.set_auto_complete(match kind {
                alacer_core::widget::AutoCompleteKind::Inline => AutoComplete::Inline,
                alacer_core::widget::AutoCompleteKind::List => AutoComplete::List,
                alacer_core::widget::AutoCompleteKind::Both => AutoComplete::Both,
            });
        }
        // The sort direction tier (M112, ADR-0116): the sorted column's
        // order maps AccessKit's `SortDirection` property, ARIA's
        // `aria-sort` — the order the AT reads beside the position the
        // M6 slot already carried. The word prices nothing but the
        // word: the column's doors answer exactly as they always did.
        // The kind mirrors AccessKit's own enum values — one branch
        // with no translation — and the honest absence (`None`, ARIA's
        // `none`) maps to the absence: a header that declared nothing
        // sets nothing, never a guessed direction. The property rides
        // the same node-only delta as every other property — a sort
        // write re-announces exactly that node.
        if let Some(kind) = node.sort_direction {
            access_node.set_sort_direction(match kind {
                alacer_core::widget::SortDirection::Ascending => {
                    accesskit::SortDirection::Ascending
                }
                alacer_core::widget::SortDirection::Descending => {
                    accesskit::SortDirection::Descending
                }
                alacer_core::widget::SortDirection::Other => accesskit::SortDirection::Other,
            });
        }
        // The level tier (M113, ADR-0117): the nested member's depth
        // maps AccessKit's `Level` property — the zero-based count the
        // AT reads beside the grid position the M6 slot already
        // carried. The word prices nothing but the word: the member's
        // doors answer exactly as they always did. The count rides
        // AccessKit's own base (zero-based, where ARIA's `aria-level`
        // is one-based — the base the property maps, the difference the
        // trait's documentation carries, never a silent cast), and the
        // honest absence (`None`) maps to the absence: a member that
        // declared nothing sets nothing, never a guessed `0`. The
        // property rides the same node-only delta as every other
        // property — a level write re-announces exactly that node.
        if let Some(level) = node.level {
            access_node.set_level(level);
        }
        // The current tier (M114, ADR-0118): the set's current item
        // maps AccessKit's `AriaCurrent` property, ARIA's
        // `aria-current` — the currency the AT reads beside the name
        // and the position the M6 slot already carried. The word
        // prices nothing but the word: the item's doors answer exactly
        // as they always did. The kind mirrors AccessKit's own enum
        // values — one branch with no translation — and the honest
        // absence (`None`, ARIA's `false`) maps to the absence: an item
        // that declared nothing sets nothing, never a guessed flavor.
        // The property rides the same node-only delta as every other
        // property — a current write re-announces exactly that node.
        if let Some(kind) = node.current {
            access_node.set_aria_current(match kind {
                alacer_core::widget::CurrentKind::True => accesskit::AriaCurrent::True,
                alacer_core::widget::CurrentKind::Page => accesskit::AriaCurrent::Page,
                alacer_core::widget::CurrentKind::Step => accesskit::AriaCurrent::Step,
                alacer_core::widget::CurrentKind::Location => accesskit::AriaCurrent::Location,
                alacer_core::widget::CurrentKind::Date => accesskit::AriaCurrent::Date,
                alacer_core::widget::CurrentKind::Time => accesskit::AriaCurrent::Time,
            });
        }
        // The has-popup tier (M115, ADR-0119): the opener's target
        // maps AccessKit's `HasPopup` property, ARIA's `aria-haspopup`
        // — the target the AT reads beside the name. The word prices
        // nothing but the word: the opener's doors answer exactly as
        // they always did. The kind mirrors AccessKit's own enum
        // values — one branch with no translation — and the honest
        // absence (`None`, the attribute unset) maps to the absence: a
        // control that opens nothing sets nothing, never a guessed
        // target. The property rides the same node-only delta as every
        // other property — a has-popup write re-announces exactly that
        // node.
        if let Some(kind) = node.has_popup {
            access_node.set_has_popup(match kind {
                alacer_core::widget::HasPopupKind::Menu => accesskit::HasPopup::Menu,
                alacer_core::widget::HasPopupKind::Listbox => accesskit::HasPopup::Listbox,
                alacer_core::widget::HasPopupKind::Tree => accesskit::HasPopup::Tree,
                alacer_core::widget::HasPopupKind::Grid => accesskit::HasPopup::Grid,
                alacer_core::widget::HasPopupKind::Dialog => accesskit::HasPopup::Dialog,
            });
        }
        if let Some(toggled) = node.toggled {
            access_node.set_toggled(match toggled {
                alacer_core::widget::ToggleState::Off => Toggled::False,
                alacer_core::widget::ToggleState::On => Toggled::True,
                alacer_core::widget::ToggleState::Mixed => Toggled::Mixed,
            });
        }
        match node.table_slot {
            Some(TableSlot::Table { rows, columns }) => {
                access_node.set_row_count(rows);
                access_node.set_column_count(columns);
            }
            Some(TableSlot::Header { column }) => {
                access_node.set_row_index(0);
                access_node.set_column_index(column);
            }
            Some(TableSlot::Cell { row, column }) => {
                access_node.set_row_index(row);
                access_node.set_column_index(column);
            }
            None => {}
        }
        // The selection (M11, ADR-0017): anchor and focus as character
        // indexes on the value-bearing node itself. AccessKit's full rich
        // text model points `TextPosition`s at `TextRun` children; Alacer's
        // a11y tree is coarse (the value lives on the editable node), so the
        // positions point at the node that owns the string — a documented
        // simplification, UNVERIFIED at strict platform adapters, recorded
        // in the ADR like every other unverified seam.
        if let Some(selection) = node.selection {
            let node_id = accesskit_node_id(node.id);
            access_node.set_text_selection(TextSelection {
                anchor: TextPosition {
                    node: node_id,
                    character_index: selection.anchor,
                },
                focus: TextPosition {
                    node: node_id,
                    character_index: selection.focus,
                },
            });
        }
        access_node.set_children(
            node.children
                .iter()
                .map(|c| accesskit_node_id(*c))
                .collect::<Vec<_>>(),
        );
        access_node
    }
}

impl AccessKitSink {
    /// Queries the resolved tree: first node with `role` and name containing
    /// `name` — the `find(ROLE).named(...)` API (§11.5).
    #[must_use]
    pub fn find(&self, role: Role, name: &str) -> Option<&alacer_core::passes::TreeNode> {
        self.last_snapshot.as_ref()?.find(role, name)
    }
}

impl A11ySink for AccessKitSink {
    fn apply(&mut self, snapshot: std::sync::Arc<TreeSnapshot>, delta: &A11yDelta) {
        self.last_snapshot = Some(std::sync::Arc::clone(&snapshot));
        self.last_removed_ids.clone_from(&delta.removed);
        // The first apply is always a full build, initializing delta or not
        // (the adapter may connect before pass 9 ever marks one).
        let force_full = delta.initializes || !self.initialized;
        let update = Self::build_update(&snapshot, delta, force_full, &mut self.last_update_ids);
        self.initialized = true;
        self.update_count += 1;
        self.updates.push(update);
        if self.updates.len() > UPDATE_RING_CAP {
            self.updates.remove(0);
        }
    }

    fn resolved_tree(&self) -> Option<&TreeSnapshot> {
        self.last_snapshot.as_deref()
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

/// Finds a node in an update by id-path hash (the harness query helper).
#[must_use]
pub fn find_in_update(update: &TreeUpdate, id: u64) -> Option<&Node> {
    update
        .nodes
        .iter()
        .find(|(node_id, _)| node_id.0 == id)
        .map(|(_, node)| node)
}

/// Depth-first walk over the resolved nodes of an update, following children.
///
/// Returns ids in visit order — the harness builds queries from this.
#[must_use]
pub fn walk_update(update: &TreeUpdate) -> Vec<u64> {
    let map: std::collections::HashMap<u64, Vec<u64>> = update
        .nodes
        .iter()
        .map(|(id, node)| (id.0, node.children().iter().map(|c| c.0).collect()))
        .collect();
    let Some(&root) = map.keys().min() else {
        return Vec::new();
    };
    let mut order = Vec::new();
    let mut queue = VecDeque::from([root]);
    while let Some(id) = queue.pop_front() {
        order.push(id);
        if let Some(children) = map.get(&id) {
            for child in children.iter().rev() {
                queue.push_front(*child);
            }
        }
    }
    order
}

#[cfg(test)]
mod tests {
    use super::{
        AccessKitSink, Live, UPDATE_RING_CAP, accesskit_node_id, find_in_update,
        map_announcement_role, map_bounds, map_live, map_role, walk_update,
    };
    use accesskit::AutoComplete;
    use accesskit::NodeId;
    use accesskit::Role as AkRole;
    use accesskit::Toggled;
    use alacer_core::Role;
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::passes::{A11yDelta, A11ySink, TreeNode, TreeSnapshot};
    use alacer_core::widget::TableSlot;
    use rustc_hash::FxHashSet;
    use std::collections::HashMap;

    fn snapshot() -> TreeSnapshot {
        let root_hash = 111_u64;
        let child_hash = 222_u64;
        let other_hash = 333_u64;
        TreeSnapshot {
            root: Some(root_hash),
            nodes: vec![
                TreeNode {
                    bounds: Some(Rect::new(Point::zero(), Size::new(200.0, 100.0))),
                    children: vec![child_hash, other_hash],
                    ..bare_node(root_hash, Role::Container, "app")
                },
                TreeNode {
                    bounds: Some(Rect::new(Point::new(0.0, 0.0), Size::new(60.0, 24.0))),
                    ..bare_node(child_hash, Role::Button, "Save")
                },
                TreeNode {
                    bounds: Some(Rect::new(Point::new(0.0, 30.0), Size::new(60.0, 20.0))),
                    value: Some("hello".into()),
                    ..bare_node(other_hash, Role::TextInput, "Search")
                },
            ],
        }
    }

    #[test]
    fn roles_map_onto_accesskit() {
        assert_eq!(map_role(Role::Button), AkRole::Button);
        assert_eq!(map_role(Role::Label), AkRole::Label);
        assert_eq!(map_role(Role::Container), AkRole::GenericContainer);
        // The M6 canon roles.
        assert_eq!(map_role(Role::CheckBox), AkRole::CheckBox);
        // The M20 value tier.
        assert_eq!(map_role(Role::Slider), AkRole::Slider);
        // The M21 value tier's read half.
        assert_eq!(map_role(Role::ProgressIndicator), AkRole::ProgressIndicator);
        assert_eq!(map_role(Role::Table), AkRole::Table);
        assert_eq!(map_role(Role::TableRow), AkRole::Row);
        assert_eq!(map_role(Role::TableCell), AkRole::Cell);
        assert_eq!(map_role(Role::ColumnHeader), AkRole::ColumnHeader);
        // The M124 radio tier: the member and the group it reads as —
        // AccessKit's `RadioButton` beside its `RadioGroup`.
        assert_eq!(map_role(Role::Radio), AkRole::RadioButton);
        assert_eq!(map_role(Role::RadioGroup), AkRole::RadioGroup);
        // The M125 link tier: the "opens something" fact.
        assert_eq!(map_role(Role::Link), AkRole::Link);
        // The M126 heading tier: the "names a section" fact — the
        // outline's landmark, the level word the level tier already
        // maps.
        assert_eq!(map_role(Role::Heading), AkRole::Heading);
        // The M127 tab tier: the member and the strip it reads as —
        // AccessKit's `Tab` beside its `TabList`.
        assert_eq!(map_role(Role::Tab), AkRole::Tab);
        assert_eq!(map_role(Role::TabList), AkRole::TabList);
        // The M128 spin-button tier: the "writes one bounded number"
        // fact — AccessKit's own `SpinButton`.
        assert_eq!(map_role(Role::SpinButton), AkRole::SpinButton);
        // The M129 toolbar tier: the named strip the tools read as —
        // AccessKit's own `Toolbar`.
        assert_eq!(map_role(Role::Toolbar), AkRole::Toolbar);
        // The M130 switch tier: the instant on/off control —
        // AccessKit's own `Switch`, the binary pair riding the
        // `Toggled` property the toggle tier already maps.
        assert_eq!(map_role(Role::Switch), AkRole::Switch);
        // The M131 tab-panel tier: the named pane the selected page's
        // content lives in — AccessKit's own `TabPanel`, the third word
        // beside the tab pair's member and strip.
        assert_eq!(map_role(Role::TabPanel), AkRole::TabPanel);
        // The M133 meter tier: the scalar measurement the AT reads as
        // a gauge — AccessKit's own `Meter`, the value words riding
        // the M97/M99 properties the sink maps generically.
        assert_eq!(map_role(Role::Meter), AkRole::Meter);
        // The M134 dialog tier: the window the modal vocabulary
        // speaks as "this is a dialog" — AccessKit's own `Dialog`,
        // the named-container shape at the modal tier.
        assert_eq!(map_role(Role::Dialog), AkRole::Dialog);
        // The M135 tree tier: the collection and the member the AT
        // walks by disclosure and depth — AccessKit's own `Tree`
        // beside its `TreeItem`, the words riding the M108/M110/M113/
        // M119 properties the sink maps generically.
        assert_eq!(map_role(Role::Tree), AkRole::Tree);
        assert_eq!(map_role(Role::TreeItem), AkRole::TreeItem);
        // The M136 tooltip tier: the help bubble the pointer pair
        // summons — AccessKit's own `Tooltip`, the named-container
        // shape at the hover tier.
        assert_eq!(map_role(Role::Tooltip), AkRole::Tooltip);
    }

    #[test]
    fn bounds_map_with_f64_edges() {
        let r = map_bounds(Rect::new(Point::new(1.0, 2.0), Size::new(30.0, 40.0)));
        assert_eq!(r, accesskit::Rect::new(1.0, 2.0, 31.0, 42.0));
    }

    #[test]
    fn the_availability_tier_maps_onto_the_disabled_property() {
        // M22, ADR-0028: the widget's `enabled` answer becomes AccessKit's
        // `disabled` property — the AT reads an unavailable control, not
        // an actionable one the widget will refuse. The default is the
        // honest absence: an available control sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].enabled = false; // the "Save" button
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let button = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the disabled node is re-emitted");
        assert!(button.is_disabled(), "the AT reads the unavailable control");
    }

    #[test]
    fn the_read_only_tier_maps_onto_the_read_only_property() {
        // M105, ADR-0109: the field whose value is the app's maps
        // AccessKit's `ReadOnly` flag, ARIA's `aria-readonly` — the AT
        // reads the word beside the value the write walls already
        // priced. The default is the honest absence: an editable field
        // sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].read_only = true; // the "Save" button node's slot — any node carries the word
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let locked = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the read-only node is re-emitted");
        assert!(locked.is_read_only(), "the AT reads the declared word");
    }

    #[test]
    fn the_required_tier_maps_onto_the_required_property() {
        // M106, ADR-0110: the control that must carry a value maps
        // AccessKit's `Required` flag, ARIA's `aria-required` — the word
        // the AT reads before it asks. The default is the honest
        // absence: a control that declared nothing sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].required = true;
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let required = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the required node is re-emitted");
        assert!(required.is_required(), "the AT reads the declared word");
    }

    #[test]
    fn the_selected_tier_maps_onto_the_selected_property() {
        // M108, ADR-0112: the collection member the collection picked
        // maps AccessKit's `Selected` property, ARIA's `aria-selected` —
        // the pick the AT reads beside the position the M6 slot already
        // carried. The honest trio maps to the honest trio: the absence
        // sets nothing (never a guessed `false`), the explicit word
        // rides the node-only delta.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].selected = Some(true);
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let selected = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the selected node is re-emitted");
        assert_eq!(
            selected.is_selected(),
            Some(true),
            "the AT reads the declared word"
        );
    }

    #[test]
    fn the_multiselectable_tier_maps_onto_the_multiselectable_property() {
        // M109, ADR-0113: the container that allows more than one pick
        // maps AccessKit's `Multiselectable` property, ARIA's
        // `aria-multiselectable` — the capacity the AT reads beside the
        // member picks. The default is the honest absence: a container
        // that declared nothing sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].multiselectable = true;
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let multiselectable = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the multiselectable node is re-emitted");
        assert!(
            multiselectable.is_multiselectable(),
            "the AT reads the declared word"
        );
    }

    #[test]
    fn the_busy_tier_maps_onto_the_busy_property() {
        // M117, ADR-0121: the container whose subtree is being updated
        // maps AccessKit's `Busy` property, ARIA's `aria-busy` — the
        // updating fact the AT defers its speech on. The default is the
        // honest absence: a container that declared nothing sets
        // nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].busy = true;
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let busy = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the busy node is re-emitted");
        assert!(busy.is_busy(), "the AT reads the declared word");
    }

    #[test]
    fn the_expanded_tier_maps_onto_the_expanded_property() {
        // M110, ADR-0114: the collection member that opens and closes
        // maps AccessKit's `Expanded` property, ARIA's `aria-expanded` —
        // the disclosure the AT reads beside the picks the selected tier
        // carried. The honest trio maps to the honest trio: the absence
        // sets nothing (never a guessed `false`), the explicit word
        // rides the node-only delta — AccessKit's own contract names it
        // ("setting this to `false` means the node is collapsed;
        // omitting it means this state isn't applicable").
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].expanded = Some(true);
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let expanded = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the expanded node is re-emitted");
        assert_eq!(
            expanded.is_expanded(),
            Some(true),
            "the AT reads the declared word"
        );
    }

    #[test]
    fn the_auto_complete_tier_maps_onto_the_auto_complete_property() {
        // M111, ADR-0115: the field whose suggestions ride a contract
        // maps AccessKit's `AutoComplete` property, ARIA's
        // `aria-autocomplete` — the suggestion contract the AT reads
        // beside the hint the placeholder tier carried. The kind mirrors
        // AccessKit's own enum values (one branch, no translation), the
        // honest absence (`None` — ARIA's `none`) sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].auto_complete = Some(alacer_core::widget::AutoCompleteKind::Both);
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let auto_complete = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the auto-complete node is re-emitted");
        assert_eq!(
            auto_complete.auto_complete(),
            Some(AutoComplete::Both),
            "the AT reads the declared contract, kind beside it"
        );
    }

    #[test]
    fn the_sort_direction_tier_maps_onto_the_sort_direction_property() {
        // M112, ADR-0116: the sorted column's order maps AccessKit's
        // `SortDirection` property, ARIA's `aria-sort` — the order the
        // AT reads beside the position the M6 slot carried. The kind
        // mirrors AccessKit's own enum values (one branch, no
        // translation), the honest absence (`None` — ARIA's `none`)
        // sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].sort_direction = Some(alacer_core::widget::SortDirection::Ascending);
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let sorted = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the sorted node is re-emitted");
        assert_eq!(
            sorted.sort_direction(),
            Some(accesskit::SortDirection::Ascending),
            "the AT reads the declared order, kind beside it"
        );
    }

    #[test]
    fn the_level_tier_maps_onto_the_level_property() {
        // M113, ADR-0117: the nested member's depth maps AccessKit's
        // `Level` property — the zero-based count the AT reads beside
        // the grid position the M6 slot carried (AccessKit's own base,
        // beside ARIA's one-based `aria-level` — one branch, no
        // translation), the honest absence (`None`) sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].level = Some(2);
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let nested = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the nested node is re-emitted");
        assert_eq!(
            nested.level(),
            Some(2),
            "the AT reads the declared depth, zero-based as the property maps"
        );
    }

    #[test]
    fn the_current_tier_maps_onto_the_aria_current_property() {
        // M114, ADR-0118: the set's current item maps AccessKit's
        // `AriaCurrent` property, ARIA's `aria-current` — the currency
        // the AT reads beside the name and the position the M6 slot
        // carried. The kind mirrors AccessKit's own enum values (one
        // branch, no translation), the honest absence (`None` — ARIA's
        // `false`) sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].current = Some(alacer_core::widget::CurrentKind::Page);
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let current = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the current node is re-emitted");
        assert_eq!(
            current.aria_current(),
            Some(accesskit::AriaCurrent::Page),
            "the AT reads the declared currency, kind beside it"
        );
    }

    #[test]
    fn the_has_popup_tier_maps_onto_the_has_popup_property() {
        // M115, ADR-0119: the opener's target maps AccessKit's
        // `HasPopup` property, ARIA's `aria-haspopup` — the target the
        // AT reads beside the name. The kind mirrors AccessKit's own
        // enum values (one branch, no translation), the honest absence
        // (`None` — the attribute unset) sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].has_popup = Some(alacer_core::widget::HasPopupKind::Menu);
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let opener = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the opener node is re-emitted");
        assert_eq!(
            opener.has_popup(),
            Some(accesskit::HasPopup::Menu),
            "the AT reads the declared target, kind beside it"
        );
    }

    #[test]
    fn the_role_description_tier_maps_onto_the_role_description_property() {
        // M116, ADR-0120: the role a custom control is *read as* maps
        // AccessKit's `RoleDescription` property, ARIA's
        // `aria-roledescription` — the specialization the AT reads
        // beside the generic role. No kind to mirror (the property is a
        // string — the description precedent), the honest absence
        // (`None`) sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].role_description = Some("star rating".to_owned());
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let rated = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the rated node is re-emitted");
        assert_eq!(
            rated.role_description(),
            Some("star rating"),
            "the AT reads the declared read-as role beside the generic one"
        );
    }

    #[test]
    fn the_set_tier_maps_onto_the_position_and_size_properties() {
        // M119, ADR-0123: the member's place and the collection's size
        // map AccessKit's `PositionInSet` (ARIA's `aria-posinset`) and
        // `SizeOfSet` (ARIA's `aria-setsize`) properties — the numbers
        // the AT reads as "row 3 of 500". AccessKit's own bases ride
        // (the place zero-based, the size on the container — the
        // differences the trait's documentation carries, one branch
        // with no translation), the honest absence (`None`) sets
        // nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].position_in_set = Some(2);
        snap.nodes[0].size_of_set = Some(500);
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().0);
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let member = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the member node is re-emitted");
        assert_eq!(
            member.position_in_set(),
            Some(2),
            "the AT reads the declared place, zero-based as the property maps"
        );
        let container = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().0)
            .map(|(_, node)| node)
            .expect("the container node is re-emitted");
        assert_eq!(
            container.size_of_set(),
            Some(500),
            "the AT reads the declared span, on the container as the property maps"
        );
    }

    #[test]
    fn the_live_tier_maps_onto_the_live_property() {
        // M120, ADR-0124: the app's declared live region maps
        // AccessKit's `Live` property (ARIA's `aria-live`) — the
        // politeness the announcement seam's own vocabulary carries,
        // beside the pipeline's reserved region the same mapping has
        // served since M18. The word prices the word and nothing else:
        // a declared container's role stays the role it already was —
        // the Status/Alert pairing is the pipeline's own region's, the
        // role the region's announcement contract names — and the
        // honest absence (`None`) sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].live = Some(alacer_core::announce::Politeness::Polite);
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let declared = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the declared node is re-emitted");
        assert_eq!(
            declared.live(),
            Some(Live::Polite),
            "the AT reads the declared region's politeness"
        );
        assert_eq!(
            declared.role(),
            AkRole::Button,
            "a declared container keeps its own role — the pairing is the pipeline's region's"
        );
    }

    /// The `(root, child)` hash pair the `snapshot()` helper mounts.
    fn snap_root_and_child() -> (u64, u64) {
        (111, 222)
    }

    #[test]
    fn the_announcement_region_maps_onto_the_aria_pairing() {
        // §11.6, ADR-0024: the politeness rides the `aria-live` property
        // and the role follows the ARIA pairing — polite speaks as
        // "status", assertive interrupts as "alert".
        assert_eq!(
            map_role(Role::LiveRegion),
            AkRole::Status,
            "the base mapping answers the polite case's role"
        );
        assert_eq!(
            map_live(alacer_core::announce::Politeness::Polite),
            Live::Polite
        );
        assert_eq!(
            map_live(alacer_core::announce::Politeness::Assertive),
            Live::Assertive
        );
        assert_eq!(
            map_announcement_role(alacer_core::announce::Politeness::Polite),
            AkRole::Status
        );
        assert_eq!(
            map_announcement_role(alacer_core::announce::Politeness::Assertive),
            AkRole::Alert
        );

        // End to end through the sink: the region node carries both.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes.push(alacer_core::passes::TreeNode {
            id: alacer_core::announce::ANNOUNCEMENT_NODE_HASH,
            role: Role::LiveRegion,
            label: "Saved".into(),
            bounds: None,
            children: Vec::new(),
            value: None,
            toggled: None,
            table_slot: None,
            selection: None,
            live: Some(alacer_core::announce::Politeness::Assertive),
            enabled: true,
            description: None,
            placeholder: None,
            value_range: None,
            scroll_orientation: None,
            scroll_position: None,
            numeric_value: None,
            secure_entry: false,
            secure_revealed: false,
            read_only: false,
            required: false,
            invalid: None,
            selected: None,
            multiselectable: false,
            busy: false,
            expanded: None,
            auto_complete: None,
            sort_direction: None,
            level: None,
            current: None,
            has_popup: None,
            role_description: None,
            position_in_set: None,
            size_of_set: None,
            keyboard_shortcut: None,
            multiline: false,
            language: None,
            search_field: false,
        });
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());
        let update = sink.last_update().expect("update");
        let region = find_in_update(update, alacer_core::announce::ANNOUNCEMENT_NODE_HASH)
            .expect("the region in the update");
        assert_eq!(
            region.live(),
            Some(Live::Assertive),
            "the AT reads the priority"
        );
        assert_eq!(region.role(), AkRole::Alert, "the AT reads the ARIA role");
        assert_eq!(region.label(), Some("Saved"));
    }

    #[test]
    fn first_update_initializes_the_whole_tree() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());

        assert_eq!(sink.update_count(), 1);
        let update = sink.last_update().expect("update");
        assert!(update.tree.is_some(), "tree info present on init");
        assert_eq!(update.nodes.len(), 3, "every node in the init update");
        let button = find_in_update(update, 222).expect("button");
        assert_eq!(button.role(), AkRole::Button);
        assert_eq!(button.label(), Some("Save"));
        let input = find_in_update(update, 333).expect("input");
        assert_eq!(input.value(), Some("hello"), "value exposed");
    }

    #[test]
    fn later_updates_are_dirty_node_only() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());

        let mut updated = FxHashSet::default();
        updated.insert(222_u64); // only the button changed
        let mut delta = A11yDelta::empty();
        delta.updated = updated;
        sink.apply(std::sync::Arc::new(snap.clone()), &delta);

        let update = sink.last_update().expect("second update");
        assert_eq!(update.nodes.len(), 1, "only the dirty node re-emitted");
        assert_eq!(update.nodes[0].0.0, 222);
        assert!(
            !sink.last_update_ids().contains(&333),
            "clean node not re-emitted"
        );
    }

    #[test]
    fn focus_travels_in_the_delta() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());

        let mut delta = A11yDelta::empty();
        delta.focus = Some(222);
        sink.apply(std::sync::Arc::new(snap.clone()), &delta);
        let update = sink.last_update().expect("update");
        assert_eq!(update.focus, accesskit::NodeId(222));
    }

    #[test]
    fn removals_surface_in_the_sink() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());

        // The tree loses the text input; its parent is updated (STRUCTURE
        // propagation), the removed id surfaces in `delta.removed`.
        let mut smaller = snapshot();
        smaller.nodes.retain(|n| n.id != 333);
        smaller.nodes[0].children = vec![222];
        let mut delta = A11yDelta::empty();
        delta.updated.insert(111);
        delta.removed.push(333);
        sink.apply(std::sync::Arc::new(smaller), &delta);

        assert_eq!(sink.last_removed_ids(), &[333]);
        let update = sink.last_update().expect("update");
        let root = find_in_update(update, 111).expect("root re-emitted");
        assert_eq!(root.children(), &[accesskit::NodeId(222)]);
    }

    #[test]
    fn node_ids_are_the_id_path_hashes() {
        // §6.3: the same hash serves hit-testing and AccessKit.
        let mut path = alacer_core::identity::IdPath::default();
        path.push(7);
        path.push(9);
        assert_eq!(path.hash(), path.hash(), "stable");
        assert_ne!(path.hash(), 0);
    }

    #[test]
    fn walk_follows_children_in_order() {
        let mut sink = AccessKitSink::new();
        let snap = snapshot();
        sink.apply(std::sync::Arc::new(snap.clone()), &A11yDelta::empty());
        let update = sink.last_update().expect("update");
        let order = walk_update(update);
        assert_eq!(order, vec![111, 222, 333]);

        // Parent-child consistency: every child id resolves to a node.
        let map: HashMap<u64, &accesskit::Node> =
            update.nodes.iter().map(|(id, n)| (id.0, n)).collect();
        for (id, node) in &map {
            for child in node.children() {
                assert!(map.contains_key(&child.0), "child of {id} exists");
            }
        }
    }

    /// The M6 vocabulary: a checkbox's toggle state and a data table's grid
    /// geometry must survive the neutral snapshot → AccessKit conversion.
    /// A bare `TreeNode`: every declared word absent — the test
    /// literals' shared base, so a tier's test overrides exactly the
    /// word it exercises and the honest absences stay the default.
    fn bare_node(id: u64, role: Role, label: &str) -> TreeNode {
        TreeNode {
            id,
            role,
            label: label.into(),
            bounds: None,
            children: Vec::new(),
            value: None,
            toggled: None,
            table_slot: None,
            selection: None,
            live: None,
            enabled: true,
            description: None,
            placeholder: None,
            value_range: None,
            scroll_orientation: None,
            scroll_position: None,
            numeric_value: None,
            secure_entry: false,
            secure_revealed: false,
            read_only: false,
            required: false,
            invalid: None,
            selected: None,
            multiselectable: false,
            busy: false,
            expanded: None,
            auto_complete: None,
            sort_direction: None,
            level: None,
            current: None,
            has_popup: None,
            role_description: None,
            position_in_set: None,
            size_of_set: None,
            keyboard_shortcut: None,
            multiline: false,
            language: None,
            search_field: false,
        }
    }

    #[test]
    fn the_shortcut_tier_maps_onto_the_keyboard_shortcut_property() {
        // M121, ADR-0125: the chord the control is invoked with maps
        // AccessKit's `KeyboardShortcut` property (ARIA's
        // `aria-keyshortcuts`) — the word the AT announces beside the
        // name before it walks to the keymap's dispatch. The string the
        // app named rides verbatim (the description precedent at the
        // string tier), and the honest absence (`None`) sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].keyboard_shortcut = Some("Control+S".to_owned());
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let worded = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the worded node is re-emitted");
        assert_eq!(
            worded.keyboard_shortcut(),
            Some("Control+S"),
            "the AT reads the declared chord beside the name"
        );
    }

    #[test]
    fn the_multiline_tier_resolves_the_multiline_text_input_role() {
        // M122, ADR-0126: the field's line capacity resolves AccessKit's
        // `MultilineTextInput` role (ARIA's `aria-multiline` beside the
        // `textbox` role) — the capacity word the vocabulary carries,
        // there being no separate property to mirror. The honest absence
        // (`false`) reads as the ordinary `TextInput` role the field
        // always was, and the secure tier's own role branch (M100) wins
        // the branch it has carried: a secured field stays
        // `PasswordInput`, its capacity word riding the snapshot beside
        // the role.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].role = Role::TextInput;
        snap.nodes[1].multiline = true;
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let worded = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the worded node is re-emitted");
        assert_eq!(
            worded.role(),
            AkRole::MultilineTextInput,
            "the AT reads the declared capacity beside the name"
        );

        // The secure field's role branch wins: the capacity word rides
        // the snapshot, the role the word M100 mapped.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].role = Role::TextInput;
        snap.nodes[1].multiline = true;
        snap.nodes[1].secure_entry = true;
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let secured = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the secured node is re-emitted");
        assert_eq!(
            secured.role(),
            AkRole::PasswordInput,
            "the secure field's role branch wins; the capacity word rides beside it"
        );
    }

    #[test]
    fn the_purpose_tier_resolves_the_search_input_role() {
        // M132, ADR-0136: the field declared for search resolves
        // AccessKit's `SearchInput` role (ARIA's `role="searchbox"`) —
        // the purpose word the vocabulary carries, there being no
        // separate property to mirror. The honest absence (`false`) reads
        // as the ordinary `TextInput` role the field always was, and the
        // tree's other words keep their precedence: the secure field's
        // branch (M100) wins the masked contract, and the capacity word
        // claims a field that declared both (one role per node — the tree
        // never guesses a composite).
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].role = Role::TextInput;
        snap.nodes[1].search_field = true;
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let worded = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the worded node is re-emitted");
        assert_eq!(
            worded.role(),
            AkRole::SearchInput,
            "the AT reads the declared purpose beside the name"
        );

        // The masked contract wins: a hidden secured field keeps the
        // word M100 mapped, whatever purpose it declared.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].role = Role::TextInput;
        snap.nodes[1].search_field = true;
        snap.nodes[1].secure_entry = true;
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let secured = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the secured node is re-emitted");
        assert_eq!(
            secured.role(),
            AkRole::PasswordInput,
            "the secure field's role branch wins; the purpose word yields the masked contract"
        );

        // The capacity word claims the composite: a multiline search box
        // keeps the word M122 mapped — one role per node, the tree never
        // guessing one.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].role = Role::TextInput;
        snap.nodes[1].search_field = true;
        snap.nodes[1].multiline = true;
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let multiline = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the multiline node is re-emitted");
        assert_eq!(
            multiline.role(),
            AkRole::MultilineTextInput,
            "the capacity word claims the composite — one role per node"
        );
    }

    #[test]
    fn the_language_tier_maps_onto_the_language_property() {
        // M123, ADR-0127: the language the widget's text is written in
        // maps AccessKit's `Language` property (the BCP 47 tag the AT
        // voices the text with). The tag the app declared rides verbatim
        // (the description precedent at the string tier), and the honest
        // absence (`None`) sets nothing.
        let mut sink = AccessKitSink::new();
        let mut snap = snapshot();
        snap.nodes[1].language = Some("fr-FR".to_owned());
        sink.apply(
            std::sync::Arc::new(snap),
            &alacer_core::passes::A11yDelta {
                updated: {
                    let mut ids = FxHashSet::default();
                    ids.insert(snap_root_and_child().1);
                    ids
                },
                removed: Vec::new(),
                focus: None,
                initializes: false,
            },
        );
        let update = sink.last_update().expect("the delta produced an update");
        let worded = update
            .nodes
            .iter()
            .find(|(id, _)| id.0 == snap_root_and_child().1)
            .map(|(_, node)| node)
            .expect("the worded node is re-emitted");
        assert_eq!(
            worded.language(),
            Some("fr-FR"),
            "the AT voices the text with the declared tag"
        );
    }

    #[test]
    fn toggled_and_table_slots_map_onto_accesskit_attributes() {
        let mut sink = AccessKitSink::new();
        let snap = TreeSnapshot {
            root: Some(1),
            nodes: vec![
                TreeNode {
                    table_slot: Some(TableSlot::Table {
                        rows: 2,
                        columns: 3,
                    }),
                    children: vec![2, 3],
                    ..bare_node(1, Role::Table, "Inventory")
                },
                TreeNode {
                    table_slot: Some(TableSlot::Header { column: 0 }),
                    ..bare_node(2, Role::ColumnHeader, "Name")
                },
                TreeNode {
                    toggled: Some(alacer_core::widget::ToggleState::On),
                    table_slot: Some(TableSlot::Cell { row: 1, column: 2 }),
                    ..bare_node(3, Role::CheckBox, "Ship in bulk")
                },
            ],
        };
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());

        let update = sink.last_update().expect("update");
        let table = find_in_update(update, 1).expect("table node");
        assert_eq!(table.role(), AkRole::Table);
        assert_eq!(table.row_count(), Some(2), "grid rows published");
        assert_eq!(table.column_count(), Some(3), "grid columns published");

        let header = find_in_update(update, 2).expect("header node");
        assert_eq!(header.role(), AkRole::ColumnHeader);
        assert_eq!(header.column_index(), Some(0), "header column index");

        let checkbox = find_in_update(update, 3).expect("checkbox node");
        assert_eq!(checkbox.role(), AkRole::CheckBox);
        assert_eq!(checkbox.toggled(), Some(Toggled::True), "checked state");
        assert_eq!(checkbox.row_index(), Some(1), "cell row position");
        assert_eq!(checkbox.column_index(), Some(2), "cell column position");
    }

    /// The description tier (M95, ADR-0099): the snapshot's help text maps
    /// onto AccessKit's `Description` property, and the honest absence
    /// maps to the absence — a node without a description sets nothing.
    #[test]
    fn the_description_maps_onto_the_description_property() {
        let mut sink = AccessKitSink::new();
        let snap = TreeSnapshot {
            root: Some(1),
            nodes: vec![
                TreeNode {
                    id: 1,
                    role: Role::Button,
                    label: "Publish".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: None,
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: Some("Saves the draft and queues it".into()),
                    placeholder: None,
                    value_range: None,
                    scroll_orientation: None,
                    scroll_position: None,
                    numeric_value: None,
                    secure_entry: false,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                },
                TreeNode {
                    id: 2,
                    role: Role::Button,
                    label: "Cancel".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: None,
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    scroll_orientation: None,
                    scroll_position: None,
                    numeric_value: None,
                    secure_entry: false,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                },
            ],
        };
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());

        let update = sink.last_update().expect("update");
        let described = find_in_update(update, 1).expect("described node");
        assert_eq!(
            described.description().map(str::to_string).as_deref(),
            Some("Saves the draft and queues it"),
            "the AT reads the help text beside the name"
        );
        let undescribed = find_in_update(update, 2).expect("undescribed node");
        assert_eq!(
            undescribed.description(),
            None,
            "the honest absence sets nothing"
        );
    }

    /// The range tier (M97, ADR-0101): the snapshot's declared span maps
    /// onto AccessKit's `MinNumericValue` / `MaxNumericValue` properties
    /// beside the `NumericValueStep` when the widget declares one, and
    /// the honest absence maps to the absence — a node without a span
    /// sets nothing, never a guessed default.
    /// The all-absence node the mapping tests decorate: every word
    /// unset, the honest absence the mapping must keep honest.
    fn plain_node(id: u64, role: Role, label: &str) -> TreeNode {
        TreeNode {
            id,
            role,
            label: label.into(),
            bounds: None,
            children: Vec::new(),
            value: None,
            toggled: None,
            table_slot: None,
            selection: None,
            live: None,
            enabled: true,
            description: None,
            placeholder: None,
            value_range: None,
            scroll_orientation: None,
            scroll_position: None,
            numeric_value: None,
            secure_entry: false,
            secure_revealed: false,
            read_only: false,
            required: false,
            invalid: None,
            selected: None,
            multiselectable: false,
            busy: false,
            expanded: None,
            auto_complete: None,
            sort_direction: None,
            level: None,
            current: None,
            has_popup: None,
            role_description: None,
            position_in_set: None,
            size_of_set: None,
            keyboard_shortcut: None,
            multiline: false,
            language: None,
            search_field: false,
        }
    }

    #[test]
    fn the_range_maps_onto_the_numeric_properties() {
        let mut sink = AccessKitSink::new();
        let snap = TreeSnapshot {
            root: Some(1),
            nodes: vec![
                TreeNode {
                    value: Some("40".into()),
                    value_range: Some(
                        alacer_core::widget::ValueRange::new(0.0, 100.0).with_step(5.0),
                    ),
                    ..plain_node(1, Role::Slider, "Volume")
                },
                TreeNode {
                    value: Some("30".into()),
                    // The report declares its span and honestly lacks a
                    // step: the absence is the absence.
                    value_range: Some(alacer_core::widget::ValueRange::new(0.0, 100.0)),
                    ..plain_node(2, Role::ProgressIndicator, "Upload")
                },
                plain_node(3, Role::Button, "Cancel"),
            ],
        };
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());

        let update = sink.last_update().expect("update");
        let slider = find_in_update(update, 1).expect("slider node");
        assert_eq!(slider.min_numeric_value(), Some(0.0), "the lower wall");
        assert_eq!(slider.max_numeric_value(), Some(100.0), "the upper wall");
        assert_eq!(
            slider.numeric_value_step(),
            Some(5.0),
            "the declared step rides beside the walls"
        );
        let progress = find_in_update(update, 2).expect("progress node");
        assert_eq!(progress.min_numeric_value(), Some(0.0));
        assert_eq!(progress.max_numeric_value(), Some(100.0));
        assert_eq!(
            progress.numeric_value_step(),
            None,
            "a report with no step announces the absence, never a default"
        );
        let bare = find_in_update(update, 3).expect("bare node");
        assert_eq!(
            bare.min_numeric_value(),
            None,
            "the honest absence sets nothing"
        );
        assert_eq!(bare.max_numeric_value(), None);
        assert_eq!(bare.numeric_value_step(), None);
    }

    /// The orientation tier (M98, ADR-0102): the scroll container's
    /// declared axis maps onto AccessKit's `Orientation` property — the
    /// vertical word beside the horizontal one, each constructor's own
    /// declaration.
    #[test]
    fn the_orientation_maps_onto_the_property() {
        let mut sink = AccessKitSink::new();
        let snap = TreeSnapshot {
            root: Some(1),
            nodes: vec![
                TreeNode {
                    id: 1,
                    role: Role::ScrollArea,
                    label: "Inbox".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: None,
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    // The vertical-only constructor's declaration.
                    scroll_orientation: Some(alacer_core::widget::ScrollOrientation::Vertical),
                    scroll_position: None,
                    numeric_value: None,
                    secure_entry: false,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                },
                TreeNode {
                    id: 2,
                    role: Role::ScrollArea,
                    label: "Sheet".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: None,
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    scroll_orientation: Some(alacer_core::widget::ScrollOrientation::Horizontal),
                    scroll_position: None,
                    numeric_value: None,
                    secure_entry: false,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                },
            ],
        };
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());

        let update = sink.last_update().expect("update");
        let vertical = find_in_update(update, 1).expect("vertical node");
        assert_eq!(
            vertical.orientation(),
            Some(accesskit::Orientation::Vertical),
            "the declared y axis is the tree's truth"
        );
        let horizontal = find_in_update(update, 2).expect("horizontal node");
        assert_eq!(
            horizontal.orientation(),
            Some(accesskit::Orientation::Horizontal),
            "the declared x axis maps beside it"
        );
    }

    /// The orientation tier's honest absence (M98, ADR-0102): a
    /// container that declared both axes sets nothing — ARIA's
    /// `undefined` is the spec's own word for "both axes", never a
    /// wrong half — and a non-scroll role inherits the absence.
    #[test]
    fn the_orientation_absence_maps_to_nothing() {
        let mut sink = AccessKitSink::new();
        let snap = TreeSnapshot {
            root: Some(1),
            nodes: vec![
                TreeNode {
                    id: 3,
                    role: Role::ScrollArea,
                    label: "Canvas pages".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: None,
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    // The two-axis constructor's answer: both axes, the
                    // absence ARIA spells `undefined`.
                    scroll_orientation: None,
                    scroll_position: None,
                    numeric_value: None,
                    secure_entry: false,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                },
                TreeNode {
                    id: 4,
                    role: Role::Button,
                    label: "Save".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: None,
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    scroll_orientation: None,
                    scroll_position: None,
                    numeric_value: None,
                    secure_entry: false,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                },
            ],
        };
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());

        let update = sink.last_update().expect("update");
        let both = find_in_update(update, 3).expect("two-axis node");
        assert_eq!(
            both.orientation(),
            None,
            "a two-axis scroller announces the absence, never a wrong half"
        );
        let bare = find_in_update(update, 4).expect("bare node");
        assert_eq!(
            bare.orientation(),
            None,
            "a non-scroll role inherits the absence"
        );
    }

    /// The numeric tier (M99, ADR-0103): the number behind the value's
    /// spoken word maps onto AccessKit's `NumericValue` property — ARIA's
    /// `aria-valuenow`, riding the text the value already carries as the
    /// `aria-valuetext` — and the honest absence maps to the absence: a
    /// widget whose value is prose sets no number, never a parsed guess.
    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "the numeric tier's contract is one declarative walk through the value shapes the field carries; splitting it would hide the mapping it exists to pin"
    )]
    fn the_numeric_now_maps_onto_the_property() {
        let mut sink = AccessKitSink::new();
        let snap = TreeSnapshot {
            root: Some(1),
            nodes: vec![
                TreeNode {
                    id: 1,
                    role: Role::Slider,
                    label: "Volume".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: Some("40".into()),
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: Some(alacer_core::widget::ValueRange::new(0.0, 100.0)),
                    numeric_value: Some(40.0),
                    secure_entry: false,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                    scroll_orientation: None,
                    scroll_position: None,
                },
                TreeNode {
                    id: 2,
                    role: Role::TextInput,
                    label: "Notes".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: Some("16 digits on the front".into()),
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    numeric_value: None,
                    secure_entry: false,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                    scroll_orientation: None,
                    scroll_position: None,
                },
            ],
        };
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());

        let update = sink.last_update().expect("update");
        let slider = find_in_update(update, 1).expect("slider node");
        assert_eq!(
            slider.numeric_value(),
            Some(40.0),
            "the number rides beside the text — aria-valuenow to its aria-valuetext"
        );
        assert_eq!(
            slider.value(),
            Some("40"),
            "the spoken word rides exactly as it always has"
        );
        let prose = find_in_update(update, 2).expect("text node");
        assert_eq!(
            prose.numeric_value(),
            None,
            "a text field's value is prose — the honest absence, never a parsed guess"
        );
    }

    /// The secure tier (M100, ADR-0104): a field declared secure maps
    /// AccessKit's `PasswordInput` role — the word every platform AT
    /// announces — and the honest absence maps to the absence: a field
    /// that declared nothing is the text input the tree always mapped.
    /// The value the node carries is the mask pass 9 resolved: the sink
    /// ships what the tree hands it, never a secret.
    #[test]
    fn the_secure_entry_maps_onto_the_password_role() {
        let mut sink = AccessKitSink::new();
        let snap = TreeSnapshot {
            root: Some(1),
            nodes: vec![
                TreeNode {
                    id: 1,
                    role: Role::TextInput,
                    label: "Passphrase".into(),
                    bounds: None,
                    children: Vec::new(),
                    // The mask pass 9 resolved for the secured field —
                    // one masking character per code point, never the
                    // secret (AccessKit's PasswordInput contract).
                    value: Some("•••••••".into()),
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    numeric_value: None,
                    secure_entry: true,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                    scroll_orientation: None,
                    scroll_position: None,
                },
                TreeNode {
                    id: 2,
                    role: Role::TextInput,
                    label: "Notes".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: Some("visible".into()),
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    numeric_value: None,
                    secure_entry: false,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                    scroll_orientation: None,
                    scroll_position: None,
                },
            ],
        };
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());

        let update = sink.last_update().expect("update");
        let secured = find_in_update(update, 1).expect("secured node");
        assert_eq!(
            secured.role(),
            AkRole::PasswordInput,
            "the constructor's word is the role the AT reads"
        );
        let bare = find_in_update(update, 2).expect("bare node");
        assert_eq!(
            bare.role(),
            AkRole::TextInput,
            "a field that declared nothing is the input the tree always mapped"
        );
    }

    #[test]
    fn the_revealed_word_yields_the_password_role() {
        // The reveal tier (M102, ADR-0106): AccessKit's own contract
        // names the mapping — "when the password is revealed, use
        // `Role::TextInput` instead". A revealed field reads as the
        // ordinary editable word; a field that revealed nothing
        // answers exactly the word M100 mapped.
        let mut sink = AccessKitSink::new();
        let snap = TreeSnapshot {
            root: Some(1),
            nodes: vec![
                TreeNode {
                    id: 1,
                    role: Role::TextInput,
                    label: "Passphrase".into(),
                    bounds: None,
                    children: Vec::new(),
                    // The revealed field's tree value: the ordinary
                    // value again — the pixels and the tree telling
                    // one truth.
                    value: Some("hunter2".into()),
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    numeric_value: None,
                    secure_entry: true,
                    secure_revealed: true,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                    scroll_orientation: None,
                    scroll_position: None,
                },
                TreeNode {
                    id: 2,
                    role: Role::TextInput,
                    label: "Notes".into(),
                    bounds: None,
                    children: Vec::new(),
                    value: Some("•••••".into()),
                    toggled: None,
                    table_slot: None,
                    selection: None,
                    live: None,
                    enabled: true,
                    description: None,
                    placeholder: None,
                    value_range: None,
                    numeric_value: None,
                    secure_entry: true,
                    secure_revealed: false,
                    read_only: false,
                    required: false,
                    invalid: None,
                    selected: None,
                    multiselectable: false,
                    busy: false,
                    expanded: None,
                    auto_complete: None,
                    sort_direction: None,
                    level: None,
                    current: None,
                    has_popup: None,
                    role_description: None,
                    position_in_set: None,
                    size_of_set: None,
                    keyboard_shortcut: None,
                    multiline: false,
                    language: None,
                    search_field: false,
                    scroll_orientation: None,
                    scroll_position: None,
                },
            ],
        };
        sink.apply(std::sync::Arc::new(snap), &A11yDelta::empty());

        let update = sink.last_update().expect("update");
        let revealed = find_in_update(update, 1).expect("revealed node");
        assert_eq!(
            revealed.role(),
            AkRole::TextInput,
            "the revealed field is the ordinary editable the contract names"
        );
        let masked = find_in_update(update, 2).expect("masked node");
        assert_eq!(
            masked.role(),
            AkRole::PasswordInput,
            "a field that revealed nothing keeps the M100 word"
        );
    }

    // -------------------------------------------------------------------
    // M9 (ADR-0015): the windowed path drains these deltas. The sink owns
    // the whole protocol surface — `drain_updates` for the steady-state
    // frame, `full_tree_update` for the adapter-activation request — so the
    // platform layer never builds a `TreeUpdate` of its own again.
    // -------------------------------------------------------------------

    #[test]
    fn drain_updates_returns_the_produced_updates_and_empties_the_buffer() {
        let mut sink = AccessKitSink::new();
        let snap = std::sync::Arc::new(snapshot());
        sink.apply(
            std::sync::Arc::clone(&snap),
            &A11yDelta {
                initializes: true,
                ..A11yDelta::empty()
            },
        );

        let drained = sink.drain_updates();
        assert_eq!(drained.len(), 1, "the init update drains exactly once");
        assert!(
            drained[0].tree.is_some(),
            "the init update carries the tree"
        );
        assert_eq!(
            sink.drain_updates(),
            Vec::new(),
            "the buffer is empty after the drain"
        );

        // One incremental frame: only the dirty id re-emits, and the drain
        // hands over exactly that update.
        let mut delta = A11yDelta::empty();
        delta.updated.insert(222);
        sink.apply(snap, &delta);
        let drained = sink.drain_updates();
        assert_eq!(drained.len(), 1, "one frame, one update");
        assert!(
            drained[0].tree.is_none(),
            "incremental deltas never rebuild the tree"
        );
        assert_eq!(
            drained[0].nodes.len(),
            1,
            "dirty nodes only — the p99 budget's whole point"
        );
        assert_eq!(drained[0].nodes[0].0, accesskit_node_id(222));
    }

    #[test]
    fn full_tree_update_mirrors_the_resolved_tree_without_mutating_the_sink() {
        let mut sink = AccessKitSink::new();
        // The AT may connect before the first tick: no resolved tree, no
        // update to push.
        assert!(
            sink.full_tree_update().is_none(),
            "no resolved tree yet — the platform defers to the first frame"
        );

        let snap = std::sync::Arc::new(snapshot());
        sink.apply(std::sync::Arc::clone(&snap), &A11yDelta::empty());

        let before = sink.update_count();
        let update = sink.full_tree_update().expect("the tree is resolved");
        assert_eq!(update.nodes.len(), 3, "every resolved node rides along");
        assert_eq!(
            update.tree.as_ref().map(|t| t.root),
            Some(accesskit_node_id(111))
        );
        assert_eq!(
            update.focus,
            NodeId(0),
            "no focus in the delta — the full tree reports none"
        );
        assert_eq!(
            sink.update_count(),
            before,
            "the full-tree build is pure: no phantom update lands in the ledger"
        );
        assert!(
            find_in_update(&update, 333).is_some_and(|n| n.value() == Some("hello")),
            "the full vocabulary rides along"
        );
    }

    #[test]
    fn the_updates_ring_caps_stored_updates() {
        let mut sink = AccessKitSink::new();
        let snap = std::sync::Arc::new(snapshot());
        for _ in 0..(UPDATE_RING_CAP + 6) {
            sink.apply(std::sync::Arc::clone(&snap), &A11yDelta::empty());
        }
        let drained = sink.drain_updates();
        assert!(
            drained.len() <= UPDATE_RING_CAP,
            "{} updates stored, ring cap is {} — unbounded growth was the pre-M9 leak",
            drained.len(),
            UPDATE_RING_CAP
        );
        assert_eq!(sink.drain_updates(), Vec::new());
    }
}
