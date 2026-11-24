//! The pass contract (architecture §7.2): each cross-cutting concern is one
//! explicit, ordered, independently traceable stage — never scattered through
//! callbacks.
//!
//! The ten named passes ([`PassKind`]), in contract order:
//!
//! | # | Pass | Responsibility |
//! |---|------|----------------|
//! | 1 | `event` | hit-test + capture→bubble routing (§10.1) |
//! | 2 | `update_focusable` | recompute focus candidates after tree changes |
//! | 3 | `update_focus` | resolve active focus; emit focus diffs |
//! | 4 | `mutate` | drain queued `WidgetMut` closures; flags merge on drop |
//! | 5 | `style` | reason-flag recalc; diffs invalidate layout/paint |
//! | 6 | `layout` | the injected [`LayoutEngine`] recomputes geometry from dirty nodes |
//! | 7 | `compose` | translation-only scroll updates — no re-measure (§8.2) |
//! | 8 | `paint` | re-emit dirty layers into the [`SceneJournal`]; clean layers replay |
//! | 9 | `accessibility` | build the a11y update from dirty nodes only (§11.1) |
//! | 10 | `action` | typed actions to the app driver (side-channel, never masks) |
//!
//! Every pass is traced under the `alacer::pass` target; with timings on, each
//! ledger entry carries its wall time and the M1 overhead gate measures the
//! pass-system bookkeeping share (§7.2 kill criterion).

use crate::arena::{Arena, NodeId};
use crate::damage::{Damage, DamageBuilder};
use crate::events::{EventPhase, ImeEvent, KeyEvent, PointerEvent};
use crate::flux::Flux;
use crate::geom::{Point, Rect, Size, Vector};
use crate::inspect::{
    DIRTY_HEATMAP_CAP, DirtyNode, EditRecord, FocusRecord, FocusRider, FrameSummary, InputKind,
    InputRecord, InspectEvent, InspectObserver, KeyRecord, PassTiming,
};
use crate::keymap::{Chord, KeymapConsult};
use crate::mask::Change;
use crate::scene::{PaintCtx, Primitive, SceneSnapshot};
use crate::style::Style;
use crate::widget::{EventResponse, Role, Widget};
use rustc_hash::{FxHashMap, FxHashSet};

/// Errors surfaced by the layout engine.
///
/// The failing node — when the engine knows it — is *data*
/// ([`LayoutError::node`]), not prose: a consumer reads the id it can
/// resolve against its own arena, and the [`std::fmt::Display`] still
/// names it for a human (ADR-0023). The type stays an honest leaf: no
/// fake `source()` chain, the engine's summary is the whole failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutError {
    /// Human-readable failure description.
    pub message: String,
    /// The node the engine failed on, when the failure is about one node.
    pub node: Option<NodeId>,
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "layout error")?;
        if let Some(node) = self.node {
            write!(f, " at node {node}")?;
        }
        write!(f, ": {}", self.message)
    }
}

impl std::error::Error for LayoutError {}

/// The layout stage, injected so `alacer-core` stays independent of Taffy.
pub trait LayoutEngine {
    /// Syncs structural changes into the persistent tree, styles dirty nodes,
    /// and recomputes geometry from the closest common ancestor of the dirty
    /// set — the persistent tree is never cleared (§8.1).
    ///
    /// Writes absolute bounds back into the arena.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutError`] when the persistent tree cannot absorb a
    /// structural or style change.
    fn sync_and_layout(
        &mut self,
        arena: &mut Arena,
        root: NodeId,
        dirty: &FxHashMap<NodeId, Change>,
        viewport: Size,
    ) -> Result<(), LayoutError>;

    /// Number of leaf measure invocations since the last
    /// [`Self::reset_measure_calls`] — the translation-only-update probe.
    #[must_use]
    fn measure_calls(&self) -> u64;

    /// Resets the measure-call counter (harness/bench convenience).
    fn reset_measure_calls(&mut self);
}

/// One node of the accessibility snapshot: role, name, bounds, children — the
/// neutral vocabulary `alacer-a11y` converts into AccessKit nodes. M6 adds
/// the toggle state and the table slot: the checkbox's checked state and the
/// data-table grid geometry ride the same snapshot.
#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "a tree node's declared facts are exactly independent booleans by domain definition — availability, secret entry, reveal, read-only and required are the tiers' own words, each `false` an honest absence, never a state machine"
)]
pub struct TreeNode {
    /// AccessKit-stable node id: the id-path hash (§6.3).
    pub id: u64,
    /// Semantic role.
    pub role: Role,
    /// Resolved accessible name.
    pub label: String,
    /// Absolute bounds.
    pub bounds: Option<Rect>,
    /// Child ids in tree order.
    pub children: Vec<u64>,
    /// The widget's exposed value (editable widgets: their text). `None` for
    /// valueless roles (M3 a11y contract: inputs carry their content).
    pub value: Option<String>,
    /// The widget's toggle state (M6): `Some` for checkbox-like roles.
    pub toggled: Option<crate::widget::ToggleState>,
    /// The widget's table geometry (M6): `Some` for tables, headers, cells.
    pub table_slot: Option<crate::widget::TableSlot>,
    /// The widget's text selection (M11, ADR-0017): anchor and caret in
    /// UTF-16 units, `Some` on the focused editable — the range a screen
    /// reader announces, riding the same snapshot as the value it selects.
    pub selection: Option<crate::widget::TextSelection>,
    /// The node's `aria-live` priority (§11.6, ADR-0024): `Some` only on
    /// the pipeline's reserved announcement region — assistive
    /// technologies speak the changes of a live-region node.
    pub live: Option<crate::announce::Politeness>,
    /// The widget's availability (M22, ADR-0028): `false` on a control
    /// that must refuse — the AT reads the truth (AccessKit's `disabled`
    /// property), the same snapshot every other property rides.
    pub enabled: bool,
    /// The widget's description (M95, ADR-0099): the help text the AT
    /// reads beside the name (AccessKit's `Description` property).
    /// `None` is the honest absence — a widget with nothing to explain
    /// announces nothing, never a guess.
    pub description: Option<String>,
    /// The widget's placeholder (M96, ADR-0100): the hint an empty
    /// editable announces before the user types (AccessKit's
    /// `Placeholder` property). `None` is the honest absence — an empty
    /// string is the absence, not an empty hint.
    pub placeholder: Option<String>,
    /// The widget's declared value range (M97, ADR-0101): the span every
    /// value-tier write funnels through, carried as AccessKit's
    /// `MinNumericValue` / `MaxNumericValue` beside the
    /// `NumericValueStep`. `None` is the honest absence — a widget with
    /// no span announces nothing, never a guessed default.
    pub value_range: Option<crate::widget::ValueRange>,
    /// The widget's numeric now (M99, ADR-0103): the number behind the
    /// value's spoken word — the scalar the value tier's funnel holds,
    /// carried as AccessKit's `NumericValue` property beside the text
    /// the tree already carried (ARIA's `aria-valuenow` beside its
    /// `aria-valuetext`). `None` is the honest absence — a widget whose
    /// value is prose announces no number, never a parsed guess.
    pub numeric_value: Option<f64>,
    /// The field's declared secret entry (M100, ADR-0104): the
    /// constructor word the IME contract has carried since M4, carried
    /// as AccessKit's `PasswordInput` role. The value the node carries
    /// beside the flag is the mask — one masking character per code
    /// point of the committed value, never the secret itself (the
    /// clipboard trail's payload rule, ADR-0045, at the tree tier).
    /// `false` is the honest absence — a field that declared nothing is
    /// no secure field, and the tree reads exactly as it read before
    /// the tier.
    pub secure_entry: bool,
    /// The scroll container's declared axis (M98, ADR-0102): the
    /// single-axis word the AT's arrows read before they ride, carried
    /// as AccessKit's `Orientation` property. `None` is the honest
    /// absence — a container that declared both axes says the absence
    /// ARIA spells `undefined`, never a guessed half.
    pub scroll_orientation: Option<crate::widget::ScrollOrientation>,
    /// The scroll container's resolved position and extent (M101,
    /// ADR-0105): where the scroll doors landed and the walls they
    /// clamp to, carried as AccessKit's `ScrollY` / `ScrollYMin` /
    /// `ScrollYMax` properties (the `ScrollX` halves beside them on a
    /// two-axis container). Each axis rides its own honesty: an axis
    /// whose span is empty carries no position, and a container with
    /// no live axis at all answers `None` — the honest absence, never
    /// a guessed zero.
    pub scroll_position: Option<crate::widget::ScrollPosition>,
    /// The field's revealed word (M102, ADR-0106): whether a
    /// declared-secure field's value rides revealed — the "show
    /// password" toggle's state beside the secure word. The value
    /// the node carries is the field's ordinary value while the
    /// word holds (the mask yielding to the reveal), and AccessKit's
    /// `PasswordInput` mapping yields with it — the sink's own
    /// contract names it. `false` is the honest absence — the
    /// masked answer M100 handed over.
    pub secure_revealed: bool,
    /// The field's read-only word (M105, ADR-0109): whether the
    /// control's value is the app's — the declared word the read
    /// doors answer beside (focus, caret, selection, copy) and the
    /// write doors refuse at (the typing door, the clipboard's write
    /// chords, the IME's commit, the AT's `SetValue` /
    /// `ReplaceSelectedText`), carried as AccessKit's `ReadOnly`
    /// property. `false` is the honest absence — an editable field
    /// declares nothing, and the tree reads exactly as it read before
    /// the tier.
    pub read_only: bool,
    /// The control's required word (M106, ADR-0110): whether it must
    /// carry a value before the form's submit fires — the word the AT
    /// reads before it asks (AccessKit's `Required` property). The
    /// word prices nothing but the word: the control's doors answer
    /// exactly as they always did. `false` is the honest absence — a
    /// control that declared nothing announces nothing, never a
    /// guessed default.
    pub required: bool,
    /// The control's invalid word (M107, ADR-0111): whether its value
    /// fails the form's contract, and of what kind — the rejection the
    /// AT reads while the user edits (AccessKit's `Invalid` property,
    /// ARIA's `aria-invalid`). The word prices nothing but the word:
    /// the control's doors answer exactly as they always did. `None`
    /// is the honest absence — a control that declared nothing
    /// announces nothing, never a guessed default.
    pub invalid: Option<crate::widget::InvalidKind>,
    /// The collection member's selected word (M108, ADR-0112): whether
    /// the member is the collection's pick — the word the AT reads
    /// beside the position the M6 slot already carried (AccessKit's
    /// `Selected` property, ARIA's `aria-selected`). The word prices
    /// nothing but the word: the member's doors answer exactly as they
    /// always did. The trio is the honest shape (the ADR-0087 presence
    /// rule at the tier): `None` the member of no selection collection
    /// — the absence ARIA spells `undefined`, never a guessed `false`;
    /// `Some(false)` the member standing unpicked beside its picked
    /// siblings; `Some(true)` the pick.
    pub selected: Option<bool>,
    /// The collection container's multiselectable word (M109, ADR-0113):
    /// whether the container allows more than one pick — the capacity
    /// the AT reads beside the member picks the selected tier already
    /// carried (AccessKit's `Multiselectable` property, ARIA's
    /// `aria-multiselectable`). The word prices nothing but the word:
    /// the container's doors answer exactly as they always did.
    /// `false` is the honest absence — a container that declared
    /// nothing announces no capacity, never a guessed one.
    pub multiselectable: bool,
    /// The container's busy word (M117, ADR-0121): whether its subtree
    /// is being updated and its announcements should be withheld until
    /// it is ready — the word the AT reads before it speaks what the
    /// stream is still writing (AccessKit's `Busy` property, ARIA's
    /// `aria-busy`). The word prices nothing but the word: the
    /// container's doors answer exactly as they always did. `false` is
    /// the honest absence — a container that declared nothing announces
    /// nothing, never a guessed "updating".
    pub busy: bool,
    /// The collection member's expanded word (M110, ADR-0114): whether
    /// the member opens and closes — the disclosure the AT reads beside
    /// the picks the selected tier already carried (AccessKit's
    /// `Expanded` property, ARIA's `aria-expanded`). The word prices
    /// nothing but the word: the member's doors answer exactly as they
    /// always did. The trio is the honest shape (the ADR-0087 presence
    /// rule at the tier, the selected shape): `None` the member that
    /// opens and closes not — the absence ARIA spells `undefined`,
    /// never a guessed `false`; `Some(false)` the member standing
    /// collapsed; `Some(true)` the member standing open.
    pub expanded: Option<bool>,
    /// The field's auto-complete word (M111, ADR-0115): the suggestion
    /// contract the field's completions ride — the word the AT reads
    /// beside the hint the placeholder tier already carried
    /// (AccessKit's `AutoComplete` property, ARIA's
    /// `aria-autocomplete`). The word prices nothing but the word: the
    /// field's doors answer exactly as they always did. `None` is the
    /// honest absence — a field that declared nothing announces no
    /// suggestion contract, never a guessed mode.
    pub auto_complete: Option<crate::widget::AutoCompleteKind>,
    /// The sorted column's sort word (M112, ADR-0116): the order the
    /// column holds its rows in — the word the AT reads beside the
    /// position the M6 slot already carried (AccessKit's
    /// `SortDirection` property, ARIA's `aria-sort`). The word prices
    /// nothing but the word: the column's doors answer exactly as they
    /// always did. `None` is the honest absence — a header that
    /// declared nothing announces no order, never a guessed direction.
    pub sort_direction: Option<crate::widget::SortDirection>,
    /// The nested member's level word (M113, ADR-0117): how deep the
    /// member stands in its collection — the word the AT reads beside
    /// the grid position the M6 slot already carried (AccessKit's
    /// `Level` property, zero-based where ARIA's `aria-level` is
    /// one-based — the base AccessKit maps, never a silent cast). The
    /// word prices nothing but the word: the member's doors answer
    /// exactly as they always did. `None` is the honest absence — a
    /// member that declared nothing announces no depth, never a
    /// guessed `0`.
    pub level: Option<usize>,
    /// The set's current-item word (M114, ADR-0118): which item of a
    /// set the reader stands in — the word the AT reads beside the name
    /// and the position the M6 slot already carried (AccessKit's
    /// `AriaCurrent` property, ARIA's `aria-current`). The word prices
    /// nothing but the word: the item's doors answer exactly as they
    /// always did. `None` is the honest absence — an item that declared
    /// nothing announces no currency, never a guessed flavor.
    pub current: Option<crate::widget::CurrentKind>,
    /// The opener's has-popup word (M115, ADR-0119): what the control
    /// opens — the word the AT reads beside the name (AccessKit's
    /// `HasPopup` property, ARIA's `aria-haspopup`). The word prices
    /// nothing but the word: the opener's doors answer exactly as they
    /// always did. `None` is the honest absence — a control that opens
    /// nothing announces no target, never a guessed one.
    pub has_popup: Option<crate::widget::HasPopupKind>,
    /// The custom control's role-description word (M116, ADR-0120): the
    /// role the control is *read as* — the word the AT reads beside the
    /// generic role (AccessKit's `RoleDescription` property, ARIA's
    /// `aria-roledescription`). The word prices nothing but the word:
    /// the control's doors answer exactly as they always did. `None` is
    /// the honest absence — a control that declared nothing is read as
    /// the generic role it already was, never a guessed specialization.
    pub role_description: Option<String>,
    /// The member's place in its collection (M119, ADR-0123): the set
    /// tier's member word — the zero-based index the AT reads beside
    /// the depth the level tier carried (AccessKit's `PositionInSet`
    /// property, ARIA's `aria-posinset`). The word prices nothing but
    /// the word: the member's doors answer exactly as they always did.
    /// `None` is the honest absence — a member that declared nothing
    /// announces no place, never a guessed `0`.
    pub position_in_set: Option<usize>,
    /// The collection's declared item count (M119, ADR-0123): the set
    /// tier's container word — the span the AT reads beside the member
    /// places (AccessKit's `SizeOfSet` property, ARIA's
    /// `aria-setsize`, on the container per AccessKit's own contract).
    /// The word prices nothing but the word: the collection's doors
    /// answer exactly as they always did. `None` is the honest absence
    /// — a collection that declared nothing announces no span, never a
    /// guessed one.
    pub size_of_set: Option<usize>,
    /// The control's shortcut word (M121, ADR-0125): the chord the
    /// control is invoked with — the word the AT reads beside the name
    /// ("Save — Control+S", AccessKit's `KeyboardShortcut` property,
    /// ARIA's `aria-keyshortcuts`). The word prices nothing but the
    /// word: the control's doors answer exactly as they always did.
    /// `None` is the honest absence — a control that declared no chord
    /// announces none, never a guessed binding.
    pub keyboard_shortcut: Option<String>,
    /// The field's capacity word (M122, ADR-0126): whether the field
    /// takes multiple lines — the word the AT reads beside the hint the
    /// placeholder tier already carried (AccessKit's `Multiline` flag,
    /// ARIA's `aria-multiline`). The word prices nothing but the word:
    /// the field's doors answer exactly as they always did. `false` is
    /// the honest absence — a field that declared nothing announces no
    /// capacity, never a guessed "multiline".
    pub multiline: bool,
    /// The text's language word (M123, ADR-0127): the language the
    /// widget's text is written in — the tag the AT voices the text
    /// with (AccessKit's `Language` property, the BCP 47 tag the app
    /// declared). The word prices nothing but the word: the widget's
    /// doors answer exactly as they always did. `None` is the honest
    /// absence — a widget that declared no language announces none,
    /// never a guessed tag.
    pub language: Option<String>,
    /// The field's purpose word (M132, ADR-0136): whether the field
    /// takes search criteria — the word the AT announces beside the
    /// name and the find conventions key off (AccessKit's
    /// `SearchInput` role, ARIA's `role="searchbox"`). The word prices
    /// nothing but the word: the field's doors answer exactly as they
    /// always did. `false` is the honest absence — a field that
    /// declared nothing announces no purpose, never a guessed
    /// "search".
    pub search_field: bool,
}

/// The full accessibility snapshot of the widget tree, built by pass 9.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TreeSnapshot {
    /// Nodes in pre-order (the tree walk order).
    pub nodes: Vec<TreeNode>,
    /// Root node id, if a root is mounted.
    pub root: Option<u64>,
}

impl TreeSnapshot {
    /// Finds the first node with `role` whose name contains `name` — the
    /// a11y-driven query API (§11.5, `find(ROLE).named(...)` semantics).
    #[must_use]
    pub fn find(&self, role: Role, name: &str) -> Option<&TreeNode> {
        let needle = name.to_lowercase();
        self.nodes
            .iter()
            .find(|n| n.role == role && n.label.to_lowercase().contains(&needle))
    }

    /// Every node with `role` whose name contains `name`, in tree order —
    /// the plural query behind the harness's count/all builders.
    #[must_use]
    pub fn find_all(&self, role: Role, name: &str) -> Vec<&TreeNode> {
        self.search(name, Some(role))
    }

    /// The word ask's query (M147, ADR-0151): every node whose resolved
    /// name carries `needle` — case-insensitive substring, [`Self::find`]'s
    /// own rule — the role narrowing the matches when named, `None` the
    /// whole vocabulary. Tree order; an empty needle names every resolved
    /// name (the ask's honest every-node answer, never a refusal).
    /// [`Self::find_all`] is this funnel with the role always named — one
    /// substring funnel, two doors, the two unable to drift apart.
    #[must_use]
    pub fn search(&self, needle: &str, role: Option<Role>) -> Vec<&TreeNode> {
        let lower = needle.to_lowercase();
        self.nodes
            .iter()
            .filter(|n| role.is_none_or(|r| n.role == r) && n.label.to_lowercase().contains(&lower))
            .collect()
    }

    /// Whether any node with `role` is named `name`.
    #[must_use]
    pub fn contains(&self, role: Role, name: &str) -> bool {
        self.find(role, name).is_some()
    }

    /// The node with the exact id-path hash (the `AccessKit` `NodeId`).
    #[must_use]
    pub fn by_id(&self, id: u64) -> Option<&TreeNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// The nodes reachable from `from`'s subtree (depth-first, tree order).
    /// `None` selects the whole tree; an empty tree walks to nothing.
    #[must_use]
    pub fn walk(&self, from: Option<u64>) -> Vec<&TreeNode> {
        let Some(start) = from.or(self.root) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut stack = vec![start];
        while let Some(id) = stack.pop() {
            if let Some(node) = self.by_id(id) {
                out.push(node);
                for child in node.children.iter().rev() {
                    stack.push(*child);
                }
            }
        }
        out
    }

    /// The point ask's geometry query (M146, ADR-0150): which node does
    /// `point` touch? The snapshot's query vocabulary ([`Self::find`],
    /// [`Self::by_id`], [`Self::walk`]) names, addresses, and walks the
    /// tree; this door locates — the innermost node whose bounds
    /// contain the point, beside the ancestry the walk descended
    /// through (root-first, the hit node last).
    ///
    /// The land rule is the paint order's own: among the nodes
    /// containing the point, the last in pre-order wins — the tree walk
    /// order is the paint order, later siblings composite on top, and
    /// the answer names what the pixels would answer. Containment is
    /// the geometry's own half-open convention (euclid's
    /// `Rect::contains`: the min edges in, the max edges out), the
    /// convention the damage tests and the scroll clamps already
    /// speak.
    ///
    /// Every miss is a named species — [`HitMiss`] — never a collapsed
    /// `None`: the walk that cannot honestly answer names why (no tree
    /// stands, no geometry stands, the point touches nothing, or the
    /// ask itself is not a finite coordinate — the NaN refused before
    /// it corrupts the comparisons, the scroll tier's rule).
    #[must_use]
    pub fn hit_at(&self, point: Point) -> HitAnswer<'_> {
        if !point.x.is_finite() || !point.y.is_finite() {
            return HitAnswer::Missed(HitMiss::Malformed);
        }
        let Some(root) = self.root else {
            return HitAnswer::Missed(HitMiss::NoTree);
        };
        // The walk: pre-order (the children pushed reversed, the pop
        // order the tree order, `walk`'s own shape), the last
        // containing node remembered beside the depth it sits at — the
        // parentage the walk fills as it goes, the ancestry rebuilt
        // from it without a clone per landing.
        let mut parents: FxHashMap<u64, u64> = FxHashMap::default();
        let mut stack = vec![(root, 0_usize)];
        let mut winner: Option<(&TreeNode, usize)> = None;
        let mut geometry = false;
        while let Some((id, depth)) = stack.pop() {
            let Some(node) = self.by_id(id) else {
                continue;
            };
            for child in node.children.iter().rev() {
                parents.insert(*child, id);
                stack.push((*child, depth + 1));
            }
            geometry |= node.bounds.is_some();
            if let Some(bounds) = node.bounds
                && bounds.contains(point)
            {
                winner = Some((node, depth));
            }
        }
        let Some((node, depth)) = winner else {
            // No containing node: the species splits the two honest
            // absences — a tree the layout pass never wrote bounds for
            // answers `no_geometry`, a standing geometry the point
            // misses answers `outside`.
            return HitAnswer::Missed(if geometry {
                HitMiss::Outside
            } else {
                HitMiss::NoGeometry
            });
        };
        // The ancestry: the parent chain from the hit node up to the
        // root, reversed root-first, the hit node last — the walk's
        // own path, the shape a panel walks back to the window from.
        let mut chain: Vec<u64> = Vec::with_capacity(depth + 1);
        let mut cursor = node.id;
        loop {
            chain.push(cursor);
            let Some(parent) = parents.get(&cursor) else {
                break;
            };
            cursor = *parent;
        }
        chain.reverse();
        let ancestry: Vec<&TreeNode> = chain
            .iter()
            // Every id the chain names was resolved during the walk —
            // the parent map only ever records live nodes' children —
            // so the filter carries the invariant, never a panic path.
            .filter_map(|id| self.by_id(*id))
            .collect();
        HitAnswer::Landed { node, ancestry }
    }

    /// The region ask's geometry query (M148, ADR-0152): which nodes
    /// does `rect` touch? The point ask's plural sibling — the nodes
    /// whose bounds intersect the region, in tree order (pre-order, the
    /// paint order's own), the matches a panel folds into the shape the
    /// dump ask already gave it. [`Self::find`] names, [`Self::by_id`]
    /// addresses, [`Self::walk`] enumerates, [`Self::hit_at`] locates a
    /// point; this door answers the shape between two corners.
    ///
    /// The touch rule is the geometry's own intersection convention
    /// (euclid's `Rect::intersection`): a rect naming no positive area
    /// — inverted or empty extents — intersects nothing, and
    /// comparisons against a non-finite coordinate are false, so a
    /// malformed ask honestly answers the empty walk rather than a
    /// munged half (the scroll tier's NaN rule; on the wire a
    /// non-finite rect is already the malformed frame's close — JSON
    /// cannot carry NaN or infinity, the point ask's own note). An
    /// empty answer is the honest absence — no tree resolved yet, no
    /// bounds written, or the region touching nothing standing — never
    /// a refusal (the word ask's empty answer, at the region tier).
    #[must_use]
    pub fn hit_region(&self, rect: Rect) -> Vec<&TreeNode> {
        self.nodes
            .iter()
            .filter(|node| {
                node.bounds
                    .is_some_and(|bounds| bounds.intersection(&rect).is_some())
            })
            .collect()
    }

    /// The path ask's ancestry door (M149, ADR-0153): where does the
    /// asked node stand? The point ask's walk names the ancestry the
    /// point descended through ([`Self::hit_at`]'s landing); this door
    /// answers it for the id a panel already holds — from the dump
    /// ask's shape, the find ask's matches, the ask family composing —
    /// root-first, the asked node last.
    ///
    /// Every miss is a named species — [`PathMiss`] — never a collapsed
    /// `None`: no tree stands yet, or the asked id names nothing in it
    /// (the point ask's species discipline at the address tier). The
    /// chain is the parentage the resolved tree carries: the root's
    /// ancestry is the root alone, and the asked node rides last.
    #[must_use]
    pub fn ancestry_of(&self, node: u64) -> PathAnswer<'_> {
        if self.root.is_none() {
            return PathAnswer::Missed(PathMiss::NoTree);
        }
        let Some(asked) = self.by_id(node) else {
            return PathAnswer::Missed(PathMiss::NoNode);
        };
        // The parentage: one pass over the resolved nodes, every child
        // mapped to the parent that carries it — the walk `hit_at`
        // descends with, rebuilt from the snapshot's own shape.
        let mut parents: FxHashMap<u64, u64> = FxHashMap::default();
        for entry in &self.nodes {
            for child in &entry.children {
                parents.insert(*child, entry.id);
            }
        }
        let mut chain: Vec<u64> = Vec::new();
        let mut cursor = asked.id;
        loop {
            chain.push(cursor);
            let Some(parent) = parents.get(&cursor) else {
                break;
            };
            cursor = *parent;
        }
        chain.reverse();
        let ancestry: Vec<&TreeNode> = chain
            .iter()
            // Every id the chain names is a standing node — the parent
            // map only ever records live nodes' children, and the asked
            // node was resolved above — so the filter carries the
            // invariant, never a panic path.
            .filter_map(|id| self.by_id(*id))
            .collect();
        PathAnswer::Landed { ancestry }
    }

    /// The descendant ask's beneath query (M150, ADR-0154): what stands
    /// under `node`? The snapshot's query vocabulary ([`Self::find`],
    /// [`Self::by_id`], [`Self::walk`], [`Self::hit_at`],
    /// [`Self::ancestry_of`]) names, addresses, walks, locates, and
    /// answers where a node stands; this door answers what the walk
    /// would find standing under it — the subtree beneath, tree order
    /// (pre-order, the paint order's own), the asked node itself the
    /// anchor the ask already names (never riding its own answer; the
    /// subtree's root is *beneath*, not *at*).
    ///
    /// The landed-or-refused shape is the path ask's own at the address
    /// tier: a landing names the subtree (a leaf names nothing beneath —
    /// the honest absence, never a refusal), every refusal a named
    /// species — [`PathMiss`], the path ask's vocabulary shared, one
    /// species list two doors (`no_tree` before the first a11y pass
    /// resolved a tree, `no_node` for an id naming nothing standing),
    /// never a collapsed `false`.
    #[must_use]
    pub fn descendants_of(&self, node: u64) -> DescendantsAnswer<'_> {
        if self.root.is_none() {
            return DescendantsAnswer::Missed(PathMiss::NoTree);
        }
        let Some(asked) = self.by_id(node) else {
            return DescendantsAnswer::Missed(PathMiss::NoNode);
        };
        // The walk: pre-order over the asked node's children (the
        // children pushed reversed, the pop order the tree order,
        // `walk`'s own shape) — the asked node itself stays out (the
        // anchor the ask names, never its own answer's first row).
        let mut beneath: Vec<&TreeNode> = Vec::new();
        let mut stack: Vec<u64> = asked.children.iter().rev().copied().collect();
        while let Some(id) = stack.pop() {
            let Some(child) = self.by_id(id) else {
                continue;
            };
            beneath.push(child);
            for grandchild in child.children.iter().rev() {
                stack.push(*grandchild);
            }
        }
        DescendantsAnswer::Landed { beneath }
    }

    /// The sibling ask's beside query (M151, ADR-0155): what stands in
    /// the parent's row beside `node`? The snapshot's relative
    /// vocabulary is whole at the tier — [`Self::ancestry_of`] answers
    /// the walk that reaches the node, [`Self::descendants_of`] the
    /// subtree beneath it, this door the row beside it: the parent's
    /// other children, tree order (pre-order, the paint order's own),
    /// the asked node itself the anchor the ask already names (never
    /// riding its own answer).
    ///
    /// The landed-or-refused shape is the family's own at the address
    /// tier: a landing names the row (the root stands beside nothing,
    /// and so does an only child — the honest absence, never a
    /// refusal), every refusal a named species — [`PathMiss`], the
    /// family's vocabulary shared, one species list three doors
    /// (`no_tree` before the first a11y pass resolved a tree,
    /// `no_node` for an id naming nothing standing), never a collapsed
    /// `false`.
    #[must_use]
    pub fn siblings_of(&self, node: u64) -> SiblingsAnswer<'_> {
        if self.root.is_none() {
            return SiblingsAnswer::Missed(PathMiss::NoTree);
        }
        let Some(asked) = self.by_id(node) else {
            return SiblingsAnswer::Missed(PathMiss::NoNode);
        };
        // The row: the parentage the resolved tree carries (the walk
        // `ancestry_of` rebuilds), the asked node's parent's children
        // beside it — the root's parent no parent at all, the absence
        // the landing's own.
        let mut parents: FxHashMap<u64, u64> = FxHashMap::default();
        for entry in &self.nodes {
            for child in &entry.children {
                parents.insert(*child, entry.id);
            }
        }
        let beside = parents
            .get(&asked.id)
            .and_then(|parent| self.by_id(*parent))
            .map_or(Vec::new(), |parent| {
                parent
                    .children
                    .iter()
                    .filter(|child| **child != asked.id)
                    .filter_map(|child| self.by_id(*child))
                    .collect()
            });
        SiblingsAnswer::Landed { beside }
    }
}

/// The point ask's answer (M146, ADR-0150) — the geometry query's
/// landed-or-refused shape (the focus walk's own, at the snapshot
/// tier): the landing names the node the point touches beside the
/// ancestry it descended through, every refusal its species, never a
/// collapsed `false`.
#[derive(Debug, Clone, PartialEq)]
pub enum HitAnswer<'a> {
    /// The point touched a node: the topmost one (the last pre-order
    /// walk order, the paint order's own rule), beside the ancestry
    /// the walk descended through — root-first, the hit node last.
    Landed {
        /// The node the point touches.
        node: &'a TreeNode,
        /// The walk's path — root-first, the hit node last.
        ancestry: Vec<&'a TreeNode>,
    },
    /// The point touched nothing — the species names why.
    Missed(HitMiss),
}

/// The point ask's refusal species — every miss named, never munged
/// into an empty answer (the named-refusals rule, the focus walk's
/// species vocabulary at the geometry tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitMiss {
    /// No tree stands (before the first a11y pass resolved one).
    NoTree,
    /// Nodes stand, none carry bounds (the layout pass never ran).
    NoGeometry,
    /// Geometry stands; the point touches no node.
    Outside,
    /// The asked point is not a finite coordinate — the NaN refusal
    /// before it corrupts the comparisons (the scroll tier's rule).
    Malformed,
}

impl HitMiss {
    /// The species' `snake_case` wire name — the one word the protocol
    /// document and the answer both speak (the focus refusal's naming
    /// door, at the geometry tier).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoTree => "no_tree",
            Self::NoGeometry => "no_geometry",
            Self::Outside => "outside",
            Self::Malformed => "malformed",
        }
    }
}

/// The path ask's answer (M149, ADR-0153) — the ancestry query's
/// landed-or-refused shape (the point ask's own, at the address tier):
/// the landing names the chain the asked node stands in, every refusal
/// its species, never a collapsed `false`.
#[derive(Debug, Clone, PartialEq)]
pub enum PathAnswer<'a> {
    /// The asked node stands: its ancestry — root-first, the asked
    /// node last.
    Landed {
        /// The asked node's chain — root-first, the asked node last.
        ancestry: Vec<&'a TreeNode>,
    },
    /// The asked node names nothing — the species names why.
    Missed(PathMiss),
}

/// The path ask's refusal species — every miss named, never munged
/// into an empty answer (the named-refusals rule, the point ask's
/// species vocabulary at the address tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathMiss {
    /// No tree stands (before the first a11y pass resolved one).
    NoTree,
    /// A tree stands; the asked id names nothing in it.
    NoNode,
}

impl PathMiss {
    /// The species' `snake_case` wire name — the one word the protocol
    /// document and the answer both speak (the hit miss's naming door,
    /// at the address tier).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoTree => "no_tree",
            Self::NoNode => "no_node",
        }
    }
}

/// The descendant ask's answer (M150, ADR-0154) — the beneath query's
/// landed-or-refused shape (the path ask's own, at the address tier):
/// the landing names the subtree that stands under the asked node,
/// every refusal its species, never a collapsed `false`. The refusal
/// species are the address tier's own — [`PathMiss`], the path ask's
/// vocabulary shared, one species list two doors.
#[derive(Debug, Clone, PartialEq)]
pub enum DescendantsAnswer<'a> {
    /// The asked node stands: what stands beneath it — the subtree in
    /// tree order (pre-order), the asked node itself the anchor the
    /// ask already names (never riding its own answer).
    Landed {
        /// The subtree beneath — tree order; a leaf names nothing.
        beneath: Vec<&'a TreeNode>,
    },
    /// The asked node names nothing — the species names why.
    Missed(PathMiss),
}

/// The sibling ask's answer (M151, ADR-0155) — the beside query's
/// landed-or-refused shape (the path ask's own, at the address tier):
/// the landing names the row the asked node stands in — the parent's
/// other children, tree order, the asked node itself the anchor the
/// ask already names — every refusal its species, never a collapsed
/// `false`. The refusal species are the address tier's own —
/// [`PathMiss`], the family's vocabulary shared, one species list
/// three doors.
#[derive(Debug, Clone, PartialEq)]
pub enum SiblingsAnswer<'a> {
    /// The asked node stands: what stands beside it — the parent's
    /// other children in tree order; the root (and an only child)
    /// name the honest absence.
    Landed {
        /// The row beside — tree order; the root stands beside
        /// nothing.
        beside: Vec<&'a TreeNode>,
    },
    /// The asked node names nothing — the species names why.
    Missed(PathMiss),
}

/// The a11y contract's per-frame product (§11.1, M3): what changed in
/// accessibility terms, and nothing more. One [`A11yDelta`] per a11y pass run:
/// dirty-node-only re-emits, the ids that disappeared, and the focused node —
/// the exact delta an incremental platform adapter (AccessKit) needs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct A11yDelta {
    /// The id-path hashes whose a11y entry changed this frame (dirty nodes
    /// only — a clean subtree costs nothing, §11.1 item 4).
    pub updated: FxHashSet<u64>,
    /// Ids present in the last a11y frame but gone from the tree now (dead
    /// widgets). Platform adapters drop them; parents of removed subtrees are
    /// always in `updated` (STRUCTURE propagates A11Y to ancestors), so
    /// parent-referenced removals are expressible in one update.
    pub removed: Vec<u64>,
    /// The focused node's id-path hash, `None` when no widget holds focus.
    pub focus: Option<u64>,
    /// Whether this delta (re)initializes the whole tree (first frame or a
    /// structural rebuild): adapters treat `updated` as the full forest.
    pub initializes: bool,
}

impl A11yDelta {
    /// A delta carrying nothing — the explicit no-op.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }
}

/// The accessibility stage, injected so `alacer-core` stays independent of
/// AccessKit.
pub trait A11ySink {
    /// Applies the snapshot incrementally: only `delta.updated` ids (id-path
    /// hashes) are re-emitted downstream (§11.1 item 4); `delta.focus` moves
    /// platform focus; `delta.initializes` marks a full (re)build.
    ///
    /// The snapshot arrives shared (`std::sync::Arc`): the pass builds it once, the sink
    /// keeps it for queries — no per-frame deep clone of the tree (the p99
    /// budget measures `TreeUpdate` production, not bookkeeping).
    fn apply(&mut self, snapshot: std::sync::Arc<TreeSnapshot>, delta: &A11yDelta);

    /// The sink's resolved tree, for harness queries.
    #[must_use]
    fn resolved_tree(&self) -> Option<&TreeSnapshot> {
        None
    }

    /// The type-erased escape hatch (the [`Widget::as_any_mut`] pattern):
    /// lets an assembly site reach the concrete sink it installed — the
    /// platform downcasts to its AccessKit sink to drain the produced
    /// `TreeUpdate`s (ADR-0015) — while the trait and this crate stay
    /// independent of any concrete adapter vocabulary.
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }
}

/// Why the platform's Input Method Editor is needed for the focused widget
/// (§10.3, the IME contract's request half).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ImePurpose {
    /// No IME: the focused widget does not accept text.
    #[default]
    Off,
    /// Normal free-form text entry (fcitx5/ibus composition on Linux).
    Text,
    /// Digits and separators only — numeric pads, PIN fields.
    Number,
    /// Secret entry: platforms suppress preedit echo and suggestion bars.
    Secure,
}

impl ImePurpose {
    /// Whether an IME should be attached to the focused widget at all.
    #[must_use]
    pub const fn enabled(self) -> bool {
        !matches!(self, Self::Off)
    }
}

/// The platform IME request derived from the focused widget each frame
/// (§10.3): what the windowing system must do about input methods now.
#[derive(Debug, Clone, PartialEq)]
pub struct ImeRequest {
    /// The focused editable widget's id-path hash (the AccessKit identity).
    pub node: u64,
    /// The requested IME flavor.
    pub purpose: ImePurpose,
    /// The caret/preedit anchor region in window coordinates — where platform
    /// candidate windows anchor. `None` when the widget reports no caret yet.
    pub caret: Option<Rect>,
}

/// The scene journal, injected so `alacer-core` owns the paint pass without
/// owning journal storage (§9.1).
pub trait SceneJournal {
    /// Replaces node `node`'s recorded layer with `primitives`.
    fn commit_layer(&mut self, node: NodeId, primitives: Vec<Primitive>);

    /// Removes node `node`'s layer (subtree removals).
    fn drop_layer(&mut self, node: NodeId);

    /// A copy of node `node`'s recorded layer, if any — the *old* paint extent
    /// source for damage tracking (§9.3: geometry changes damage old and new).
    #[must_use]
    fn layer(&self, node: NodeId) -> Option<Vec<Primitive>>;

    /// Materializes the ordered scene: layers concatenated in `order` (the
    /// arena's pre-order walk — the paint order).
    #[must_use]
    fn snapshot(&self, order: &[NodeId]) -> SceneSnapshot;

    /// Keeps only the layers of nodes in `order` (structural churn cleanup).
    fn retain(&mut self, order: &[NodeId]);

    /// Drops every layer (structural rebuilds).
    fn clear(&mut self);

    /// The journal's own ledger (ADR-0029): the commits and drops it has
    /// recorded and the layers currently live. The consumer reads it
    /// through [`PassPipeline::journal_stats`] — the frame's cost, as the
    /// journal itself accounts it (one write batch ⇒ one layer re-commit;
    /// clean layers replay untouched; idle frames write nothing).
    #[must_use]
    fn stats(&self) -> JournalStats;
}

/// The scene journal's ledger (ADR-0029): what the paint pass has cost the
/// journal so far. `SceneJournalImpl` accounts it exactly; a journal that
/// does not account answers the zero value — the honest default for
/// storage that keeps no books.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JournalStats {
    /// Layer commits so far (re-emissions included).
    pub commits: u64,
    /// Layer drops (structural churn).
    pub drops: u64,
    /// Layers currently live.
    pub live_layers: usize,
}

/// Counters for the frame-level scheduling contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameCounters {
    /// Layout passes run.
    pub layout_runs: u64,
    /// Paint passes run.
    pub paint_runs: u64,
    /// Accessibility passes run.
    pub a11y_runs: u64,
    /// Pointer events routed.
    pub events_routed: u64,
}

/// The ten named passes of the pass contract (§7.2) — each explicit, ordered,
/// and independently traceable. Masonry's pass system as contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassKind {
    /// Hit-test + capture→bubble routing (§10.1). Dispatch-driven.
    Event,
    /// Recompute focus candidates after tree changes.
    UpdateFocusable,
    /// Resolve the active focus; emit focus diffs.
    UpdateFocus,
    /// Drain queued widget mutations; flags merge on drop.
    Mutate,
    /// Recalc styles with reason flags; diffs invalidate layout/paint.
    Style,
    /// Incremental measure/layout over the persistent tree.
    Layout,
    /// Scroll offsets and translation-only updates (§8.2).
    Compose,
    /// Append scene ops to the journal; clean layers replay.
    Paint,
    /// Build incremental AccessKit updates from dirty nodes only.
    Accessibility,
    /// Propagate typed actions to the app driver (side-channel, never masks).
    Action,
}

impl PassKind {
    /// The pass's contract name, used for tracing spans and the ledger.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::UpdateFocusable => "update_focusable",
            Self::UpdateFocus => "update_focus",
            Self::Mutate => "mutate",
            Self::Style => "style",
            Self::Layout => "layout",
            Self::Compose => "compose",
            Self::Paint => "paint",
            Self::Accessibility => "accessibility",
            Self::Action => "action",
        }
    }

    /// The passes [`PassPipeline::tick`] drives, in contract order (§7.2
    /// passes 2–9; `event` and `action` are dispatch-driven entry points).
    #[must_use]
    pub const fn tick_order() -> [Self; 8] {
        [
            Self::Mutate,
            Self::UpdateFocusable,
            Self::UpdateFocus,
            Self::Style,
            Self::Layout,
            Self::Compose,
            Self::Paint,
            Self::Accessibility,
        ]
    }
}

/// One pass's ledger entry: did it run, and how long did its span take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassRun {
    /// Which pass.
    pub kind: PassKind,
    /// Whether the pass had work (skipped passes are still recorded).
    pub ran: bool,
    /// Wall time of the pass body, when timings are on (else 0).
    pub nanos: u64,
    /// Wall time of the pass's *substantive* work (the injected stage call).
    /// The difference `nanos − work_nanos` is the pass-system bookkeeping the
    /// M1 gate measures. Zero when unmeasured.
    pub work_nanos: u64,
}

/// The focus walk's directions (§10.3, ADR-0030/ADR-0032): the traversal
/// vocabulary one funnel serves. The keyboard's Tab/Shift+Tab fallback and
/// the consumer's [`PassPipeline::focus_move`] door both name the same
/// walk — the order is the pipeline's business (the pre-order candidate
/// list pass 2 recomputes, the availability tier's cycle-time filter
/// applied), never the caller's. Since M26 the vocabulary answers the
/// window as well as the tree: the four directional moves read the bounds
/// pass 6 wrote and select the neighbor the *geometry* names, while the
/// two tree-order moves keep the pre-order list and its wrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusMove {
    /// The next candidate in tree order (Tab's direction); wraps from the
    /// last candidate back to the first.
    Next,
    /// The previous candidate in tree order (Shift+Tab's direction);
    /// wraps from the first candidate back to the last.
    Prev,
    /// The nearest candidate fully left of the focused node's bounds;
    /// never wraps — space has no wrap, the window's edge is the walk's
    /// end (ADR-0032).
    Left,
    /// The nearest candidate fully right of the focused node's bounds;
    /// never wraps.
    Right,
    /// The nearest candidate fully above the focused node's bounds;
    /// never wraps.
    Up,
    /// The nearest candidate fully below the focused node's bounds;
    /// never wraps.
    Down,
}

impl FocusMove {
    /// The direction's name, used by the seam's tracing spans and tests.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::Prev => "prev",
            Self::Left => "left",
            Self::Right => "right",
            Self::Up => "up",
            Self::Down => "down",
        }
    }

    /// `true` for the four moves that select by geometry
    /// (`Left`/`Right`/`Up`/`Down`) — the moves that read the bounds pass
    /// 6 wrote instead of the pre-order candidate list's positions
    /// (ADR-0032). The tree-order moves wrap; the directional ones never
    /// do.
    #[must_use]
    pub const fn is_directional(self) -> bool {
        matches!(self, Self::Left | Self::Right | Self::Up | Self::Down)
    }
}

/// Where the focus walk came to rest (M27, ADR-0033): the endpoints of a
/// move that landed. The `from` endpoint is `None` for the entry — the
/// walk's first landing from no focus, either direction — and the `to`
/// endpoint is the node the walk rests on now, the same one
/// [`PassPipeline::focus`] answers. `wrapped` says the tree-order move
/// crossed a list end on its way (the directional moves never wrap —
/// space has no wrap, ADR-0032), the fact the doors' boolean past could
/// not carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusLanding {
    /// The node that held focus before the move, `None` for the entry.
    pub from: Option<NodeId>,
    /// The node the walk rests on now.
    pub to: NodeId,
    /// The tree-order move crossed a list end (forward past the last
    /// candidate, backward past the first). The directional moves never
    /// wrap, so their landings always carry `false`.
    pub wrapped: bool,
}

/// Why the walk refused (M27, ADR-0033): the refusal species the doors'
/// boolean past collapsed into one `false`. Every species keeps the M11
/// rule's shape — no masks, an idle frame, focus never munged — the
/// answer just stops making the caller guess which refusal it was. The
/// per-species help stays on the `alacer::focus` target as plain
/// diagnostic text (ADR-0029's rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusRefusal {
    /// No candidate the availability tier still answers for (M22): an
    /// empty arena, every control unavailable, or a trap (the scoped
    /// door) holding nothing available.
    NoCandidates,
    /// The scoped door named a dead scope: the subtree the walk was to
    /// bound to is gone (M25's refusal, now named).
    DeadScope {
        /// The scope node the caller passed.
        scope: NodeId,
    },
    /// The focused node carries no geometry yet — a directional move
    /// before the first layout wrote the bounds it reads (ADR-0032).
    NoGeometry {
        /// The node the walk tried to read geometry from.
        node: NodeId,
    },
    /// Nothing lies in the requested direction: no candidate fully
    /// beyond the focused node's edge, and space never wraps (ADR-0032).
    NothingInDirection {
        /// The node the walk searched from.
        node: NodeId,
        /// The direction that came up empty.
        direction: FocusMove,
    },
    /// The walk resolved to the node already holding focus — the
    /// single-candidate stall: the only candidate's next stop is itself,
    /// the walk refuses rather than munge (the M11 rule).
    AlreadyFocused {
        /// The node holding focus.
        node: NodeId,
    },
}

/// The IME event's stable `snake_case` tag (M59, ADR-0065) — the
/// boundary's own name for what arrived, the route-drop record's
/// action field riding it verbatim (the M57 rule: the ask the
/// boundary refused, never the core action it did not become).
fn ime_event_tag(event: &ImeEvent) -> &'static str {
    match event {
        ImeEvent::Preedit { .. } => "ime_preedit",
        ImeEvent::Commit { .. } => "ime_commit",
    }
}

/// The focus walk's answer as data (M27, ADR-0033): what the two doors
/// (`focus_move`, `focus_move_in`) return instead of the boolean they
/// took through M26. ADR-0031's pre-registered re-open rule — "a door
/// needs to observe the walk's refusals as data" — grown into the
/// vocabulary: a [`FocusLanding`] when the walk moved (endpoints and
/// wrap), a [`FocusRefusal`] naming the species when it refused. The
/// walk behind the doors is untouched — the same candidate list, the
/// same availability filter, the same wrap, the same immediate
/// resolution, the same no-mask idle frames on refusals (the M11 rule
/// stays; the answer stops being silent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusMoveOutcome {
    /// The walk moved focus to the landing's node this call — the same
    /// immediate resolution the keyboard's Tab requires.
    Moved(FocusLanding),
    /// The walk refused honestly: no masks, an idle frame, focus left
    /// where it was, the species named.
    Refused(FocusRefusal),
}

impl FocusMoveOutcome {
    /// `true` when the walk moved focus — the doors' boolean past, kept
    /// for the call sites that only route on the verdict (the keyboard's
    /// Tab fallback takes the key when the walk moved).
    #[must_use]
    pub const fn moved(&self) -> bool {
        matches!(self, Self::Moved(_))
    }

    /// The landing when the walk moved, `None` when it refused.
    #[must_use]
    pub const fn landing(&self) -> Option<FocusLanding> {
        match *self {
            Self::Moved(landing) => Some(landing),
            Self::Refused(_) => None,
        }
    }

    /// The refusal when the walk refused, `None` when it moved.
    #[must_use]
    pub const fn refusal(&self) -> Option<FocusRefusal> {
        match *self {
            Self::Moved(_) => None,
            Self::Refused(reason) => Some(reason),
        }
    }
}

/// Why a bring-into-view ask never reached a scroll answer (M88,
/// ADR-0092): the door's own refusals, named where the ask died —
/// never a munged landing, never a tier fact for a question the scroll
/// tier never answered (the ADR-0046 rule the scroll tier rides: a
/// question outside the tier is not a tier fact, so these refuse as
/// data on this door's typed answer alone and stream no record).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollIntoViewRefusal {
    /// The aimed id is not a live node — the sentinel, a stale handle,
    /// an id the tree never allocated. The honest absence: no geometry
    /// exists to read, no container to ask.
    UnknownNode,
    /// The node is live but the bounds ledger holds no answer for it
    /// yet — mounted this batch, the layout pass has not run. The door
    /// never guesses geometry the ledger has not written.
    NotLaidOut,
    /// No scroll container sits above the aimed node — the ask lands
    /// nowhere because nothing here scrolls. The nearest-ancestor walk
    /// answered honestly before any container was guessed.
    NoScrollableAncestor,
    /// The nearest ancestor claims the scroll slot
    /// ([`crate::widget::Widget::is_scroll_container`]) but ships no
    /// bring-into-view door — the contract gap named where the
    /// consumer can read it, never skipped in silence.
    NoScrollDoor,
}

/// The bring-into-view door's landing (M88, ADR-0092): the container
/// that moved beside the container's own answer ([`crate::widget::
/// ScrollLanding`], flattened — the door adds the identity the widget's
/// door cannot know, the M43 rule's shape: the widget answers its range,
/// the route names the control the ask rode). Since M89 (ADR-0093) the
/// flat fields answer the *vertical* axis and the horizontal axis's own
/// answer rides beside them ([`crate::widget::ScrollAxisLanding`], the
/// same shape flattened from the container's answer).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollIntoViewLanding {
    /// The nearest scrollable ancestor whose offset moved — the
    /// [`NodeId`] the ask resolved to (the record's `target` stamp
    /// carries the same fact on the log, as the AccessKit-stable id
    /// hash).
    pub container: NodeId,
    /// What the container's clamp answered — delivered, or the named
    /// refusal species ([`crate::inspect::ScrollRefusal`]).
    pub outcome: crate::inspect::ScrollOutcome,
    /// The offset the container holds as the answer resolves.
    pub offset: f32,
    /// The wall the clamp pinned a past ask at (`min`/`max`), `None`
    /// when the landing is the ask or nothing moved.
    pub pinned: Option<crate::inspect::RangeBound>,
    /// The horizontal axis's own answer (M89, ADR-0093), beside the
    /// vertical — the same clamp arithmetic, the container's own
    /// declared span, the refusals named per axis.
    pub horizontal: crate::widget::ScrollAxisLanding,
}

/// The bring-into-view door's answer (M88, ADR-0092) — the M27 rule
/// at the scroll tier: the ask is data at every landing, the landed
/// offset beside the outcome, the named refusal beside the silence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollIntoViewOutcome {
    /// The ask landed: the container's clamp answered the minimal ask
    /// the bounds ledger priced, the landing carrying the container
    /// that moved, the offset and the wall (if the clamp held one)
    /// beside them. One `ScrollRecord` streamed under
    /// [`crate::inspect::ScrollAction::BringIntoView`], the stamps
    /// naming the container and the aimed node.
    Landed(ScrollIntoViewLanding),
    /// The aimed node already sits fully inside the viewport — nothing
    /// to move, nothing to ask: no masks, no record, the idle frame
    /// the content clamp's own rule rides (the M51 idle property at
    /// the door's tier).
    Visible,
    /// The ask died before any container's door: the species names
    /// where ([`ScrollIntoViewRefusal`]), no masks queued, no record
    /// streamed.
    Refused(ScrollIntoViewRefusal),
}

/// The degradation ladder's step (§14.4): what a frame over budget gave
/// up, in the ladder's own order. One step per frame — the decisions run
/// in ladder order, so the step recorded is the deepest rung the tick
/// reached. The ladder arms only where a budget is installed
/// ([`PassPipeline::set_frame_budget`]): a disarmed pipeline — the
/// default — reports [`DegradationStep::None`] every frame, so the
/// shipped shape is byte-for-byte the pre-ladder tick's and no pass ever
/// skips uninvited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationStep {
    /// Every pass ran inside the budget — nothing was given up.
    None,
    /// The input drain blew its share of the budget: the batch's
    /// remainder stays queued and rides the next frame's drain (§14.4
    /// rung 1 — coalesce the flux work into the next frame). Never a
    /// dropped event: a postponed one — the queue drains to empty on the
    /// frames that can afford it.
    DropInput,
    /// The paint pass blew its share of the budget on a partial frame:
    /// the merged regions collapse to one full-frame present (§14.4 rung
    /// 3 — present full frame). The verdict widens, the scene does not —
    /// a full present shows the same frame, the backend just stops
    /// paying region bookkeeping while behind.
    ForceFullPaint,
    /// The frame was past its budget's halfway line when the a11y pass
    /// came due: the pass's *dirty updates* defer — their masks re-queue
    /// and the next frame's pass lands the delta whole. Focus moves,
    /// announcements, first frames, and structural churn never defer
    /// (§14.4's own contract: latency may degrade, correctness may not)
    /// — the deferral is one frame of latency for the dirty entries,
    /// never a lost fact, and a sustained overload defers every frame
    /// rather than pushing any past its budget.
    SkipA11y,
}

impl DegradationStep {
    /// The step's name — the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values; see
    /// `docs/inspector-protocol.md`, the `frame` event).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::DropInput => "drop_input",
            Self::ForceFullPaint => "force_full_paint",
            Self::SkipA11y => "skip_a11y",
        }
    }
}

/// The ladder's decision thresholds, as shares of the installed frame
/// budget (§14.4): the input drain owns the first quarter, the per-pass
/// overrun line is the halfway mark — a frame that reaches the a11y pass
/// past it defers the dirty updates ([`DegradationStep::SkipA11y`]), a
/// paint pass past it collapses a partial present to one full frame
/// ([`DegradationStep::ForceFullPaint`]).
const DEGRADATION_INPUT_SHARE: u32 = 4;

/// The input drain's batch slice: the ladder drains the queue in slices
/// of this many events so the budget is checked between dispatches —
/// the granularity the rung's honesty rides (a storm of thousands
/// routes its head, the remainder stays queued, never a lost event).
const DEGRADATION_INPUT_BATCH: usize = 32;

/// One frame's outcome, returned by [`PassPipeline::tick`].
#[derive(Debug, Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "a frame report is genuinely independent pass verdicts plus focus state; packing them into bitflags would trade every match site's readability for nothing"
)]
pub struct FrameReport {
    /// `true` when the layout pass ran.
    pub layout_ran: bool,
    /// `true` when the paint pass ran.
    pub paint_ran: bool,
    /// `true` when the accessibility pass ran.
    pub a11y_ran: bool,
    /// The materialized scene (empty when nothing painted). Shared, not
    /// cloned: idle and replay-only frames reuse the last materialization
    /// (ADR-0008 item 4, §14.3's warm path).
    pub scene: std::sync::Arc<SceneSnapshot>,
    /// The frame's paint damage (§9.3, M3): up to three merged region boxes,
    /// or the full-frame fallback. `Damage::NONE` frames skip the present
    /// entirely; partial frames let backends repaint regions only — the same
    /// damage drives the GPU scissor path and the software dirty-rect blit.
    pub damage: Damage,
    /// The a11y delta pass 9 produced (empty when the pass skipped).
    pub a11y_delta: A11yDelta,
    /// The pass ledger, in contract order — every tick pass, ran or
    /// skipped. Shared, not rebuilt (§14.3's warm path, A7's budget):
    /// a warm idle frame's ledger is the same eight skipped runs every
    /// tick, so the report hands out an [`std::sync::Arc`] handle to the
    /// cached canonical ledger and a warm tick allocates nothing (M84,
    /// ADR-0088). A frame where a pass ran — or timings ride — carries
    /// its own freshly built ledger.
    pub passes: std::sync::Arc<Vec<PassRun>>,
    /// Focus moved this frame.
    pub focus_changed: bool,
    /// The degradation ladder's deepest rung this frame reached (§14.4).
    /// [`DegradationStep::None`] on every disarmed pipeline and on every
    /// frame that fit its budget.
    pub degradation_step: DegradationStep,
    /// Total tick wall time, when timings are on (else 0).
    pub total_nanos: u64,
}

impl FrameReport {
    /// `true` when no pass had work — the idle contract (A7). A frame
    /// that deferred a11y updates ([`DegradationStep::SkipA11y`]) is
    /// never idle: its work is queued, the next frame owes it.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        !self.layout_ran
            && !self.paint_ran
            && !self.a11y_ran
            && self.degradation_step == DegradationStep::None
    }

    /// The ledger entry for `kind`.
    #[must_use]
    pub fn pass(&self, kind: PassKind) -> Option<&PassRun> {
        self.passes.iter().find(|run| run.kind == kind)
    }

    /// The four injected stages do the frame's substantive work; everything
    /// else in the tick is pass-system bookkeeping (§7.2 kill criterion).
    /// Gate-only measurement: the M1 gate test is the sole reader (the M28
    /// sweep's finding — the introspection was never a public surface).
    #[cfg(test)]
    fn work_nanos(&self) -> u64 {
        self.passes
            .iter()
            .filter(|run| {
                matches!(
                    run.kind,
                    PassKind::Layout
                        | PassKind::Paint
                        | PassKind::Accessibility
                        | PassKind::Compose
                )
            })
            .map(|run| run.work_nanos)
            .sum()
    }

    /// Pass-system bookkeeping nanoseconds: the tick minus the injected
    /// stages' substantive work. Only meaningful with timings on.
    ///
    /// Crate-private: the §7.2 M1 gate is the only consumer (the M25
    /// species — a public accessor whose callers were all in-crate tests).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn pass_overhead_nanos(&self) -> u64 {
        self.total_nanos.saturating_sub(self.work_nanos())
    }

    /// `pass_overhead_nanos / total_nanos` — the M1 gate ratio: the collapse
    /// trigger fires above 0.15 on the 10k-node churn workload (§7.2).
    /// Crate-private: the gate test is the only consumer (the M25
    /// species).
    #[cfg(test)]
    #[must_use]
    pub(crate) fn pass_overhead_ratio(&self) -> f32 {
        if self.total_nanos == 0 {
            return 0.0;
        }
        crate::num::u64_to_f32(self.pass_overhead_nanos())
            / crate::num::u64_to_f32(self.total_nanos)
    }
}

/// The pass pipeline: owns the arena, the flux, and the injected stages.
pub struct PassPipeline {
    arena: Arena,
    flux: Flux,
    layout: Box<dyn LayoutEngine>,
    journal: Box<dyn SceneJournal>,
    a11y: Box<dyn A11ySink>,
    counters: FrameCounters,
    scroll_offsets: FxHashMap<NodeId, Vector>,
    last_compose_events: u64,
    /// The last materialized scene — replay frames share it instead of
    /// re-walking the journal (ADR-0008 item 4).
    scene_cache: Option<std::sync::Arc<SceneSnapshot>>,
    /// The pre-order paint order, valid until tree shape changes (it is a
    /// pure function of shape). Rebuilt on structural frames.
    order_cache: Option<Vec<NodeId>>,
    /// Per-pass wall timing (the M1 gate instrumentation); off by default.
    timings: bool,
    /// The clock the timing rides (M104, ADR-0108): the time seam the
    /// [`Self::set_clock`] door installs. `None` in every default path —
    /// the reads answer the fallback anchor (real monotonic time), the
    /// shipped shape. A seam clock makes the timed frame deterministic:
    /// the tests install one and assert the ledger exactly.
    clock: Option<Box<dyn crate::time::TimeSeam>>,
    /// The fallback reads' anchor: the pipeline's own construction
    /// instant. Only differences of reads are priced, so one anchor per
    /// pipeline is the honest shape — the deltas equal the deltas the
    /// direct `Instant::now` reads answered, from one fixed origin.
    clock_anchor: std::time::Instant,
    /// The frame budget the degradation ladder arms against (§14.4).
    /// `None` — the default — leaves every pass unconditional: the
    /// shipped shape is the pre-ladder tick's. A budget installed
    /// ([`Self::set_frame_budget`]) makes the tick's over-budget
    /// decisions rung by rung: the input drain's remainder coalesces
    /// into the next frame, a behind-schedule paint presents one full
    /// frame, and a frame past the halfway line defers the a11y pass's
    /// dirty updates ([`DegradationStep::SkipA11y`]).
    frame_budget: Option<std::time::Duration>,
    /// Pass ledger entries for dispatch-driven passes (event, action),
    /// attributed to the next tick's report.
    interframe: Vec<PassRun>,
    /// The canonical idle-frame ledger (timings off): the tick passes,
    /// all skipped — the one ledger a warm tick hands out by handle
    /// instead of rebuilding (A7's budget, M84, ADR-0088). Timed frames
    /// never consult it: their runs carry per-frame nanos, no two ticks
    /// agree.
    idle_ledger: Option<std::sync::Arc<Vec<PassRun>>>,
    /// Style cache — pass 5 diffs `widget.style()` against it (§7.2 pass 5).
    style_cache: FxHashMap<NodeId, Style>,
    /// Focus candidates, recomputed by pass 2.
    focusables: Vec<NodeId>,
    /// The node currently holding focus (pass 3 output).
    focused: Option<NodeId>,
    /// A focus request awaiting pass 3.
    pending_focus: Option<NodeId>,
    /// The IME request derived from the focused widget (§10.3; `None` when no
    /// editable holds focus). The platform layer compares frame over frame.
    ime_request: Option<ImeRequest>,
    /// The IME derivation's cache key (A7, M84, ADR-0088): the request is a
    /// pure function of the focused node and its resolved caret anchor
    /// between structure and mutation frames, so a warm focused tick whose
    /// key is unchanged re-derives nothing — the id path's chain vector
    /// (3 blocks / 96 bytes per idle frame, priced by the
    /// `zero_alloc_warm_frame` suite) no longer dies with every tick. Any
    /// key's move, any structure or mutation frame, re-derives honestly.
    ime_cache: Option<(Option<NodeId>, Option<Rect>)>,
    /// The id-path hashes pass 9 emitted last time — the removals source for
    /// the a11y delta (dead widgets leave the set; their ids surface once).
    last_a11y_ids: Option<FxHashSet<u64>>,
    /// The devtools observer (§13.3, ADR-0010). `None` in every default
    /// path: the frame event is built only when someone is listening.
    observer: Option<Box<dyn InspectObserver>>,
    /// The installed theme seam (§7.2 pass 8, ADR-0079): `None` in every
    /// default path — the paint pass resolves [`crate::theme::Palette::CANON`]
    /// and a default build paints the constants the blessed corpus was
    /// blessed with.
    theme: Option<Box<dyn crate::theme::ThemeSeam>>,
    /// The installed setting seam (§7.2 pass 8, ADR-0080): the OS's
    /// dark-mode fact the platform adapter pushes through the door.
    /// `None` in every default path — the paint pass resolves the
    /// canon light fact, the fact the blessed corpus was blessed with.
    settings: Option<Box<dyn crate::settings::SettingSeam>>,
    /// The installed composition seam (§7.2 pass 8, ADR-0081): the
    /// palette the OS's fact resolves into — the composition door
    /// resolves it at install time and on every flip that lands.
    /// `None` in every default path — the palette follows nothing, the
    /// app's own theme door the only mover (the M74 shape).
    composer: Option<Box<dyn crate::settings::SettingComposer>>,
    /// The frame-batched input queue (§10.2, ADR-0014): raw platform
    /// pointer events coalesce here and route at tick start — the same
    /// one-frame currency the flux bridge gives signal writes (A1, ADR-2).
    input: crate::input::EventCoalescer,
    /// The pointer capture (§10.1, ADR-0025): the node the active press
    /// holds. `None` when no press is in flight — the routing then stays
    /// the pure per-event hit test this field rides beside. The capture
    /// is implicit (every `Down` holds its deepest accepting hit) and
    /// single-pointer (the event vocabulary carries no pointer id —
    /// ADR-0014's shape), so one slot is the honest width.
    captured: Option<NodeId>,
    /// The app's last announcement (§11.6, ADR-0024): the content the
    /// reserved live-region node carries. `None` until the first accepted
    /// `announce` — the region then rides the snapshot with an empty name.
    announcement: Option<crate::announce::Announcement>,
    /// Whether an announcement arrived since the last a11y pass — the
    /// pass-9 run trigger beside the dirty set and focus moves.
    announcement_dirty: bool,
    /// Monotonic frame counter (the frame summary's number).
    frames: u64,
}

impl PassPipeline {
    /// Assembles the pipeline from its stages.
    #[must_use]
    pub fn new(
        flux: Flux,
        layout: Box<dyn LayoutEngine>,
        journal: Box<dyn SceneJournal>,
        a11y: Box<dyn A11ySink>,
    ) -> Self {
        Self {
            arena: Arena::new(),
            flux,
            layout,
            journal,
            a11y,
            counters: FrameCounters::default(),
            scroll_offsets: FxHashMap::default(),
            scene_cache: None,
            order_cache: None,
            last_compose_events: 0,
            timings: false,
            clock: None,
            clock_anchor: std::time::Instant::now(),
            frame_budget: None,
            interframe: Vec::new(),
            idle_ledger: None,
            style_cache: FxHashMap::default(),
            focusables: Vec::new(),
            focused: None,
            pending_focus: None,
            ime_request: None,
            ime_cache: None,
            last_a11y_ids: None,
            observer: None,
            theme: None,
            settings: None,
            composer: None,
            input: crate::input::EventCoalescer::new(),
            captured: None,
            announcement: None,
            announcement_dirty: false,
            frames: 0,
        }
    }

    /// Installs the devtools observer (§13.3): the pipeline forwards frame
    /// summaries, routed-input records, focus-door answers, the edit
    /// trail (ADR-0037 — the journal's facts ride the tick's start drain),
    /// and the announcement trail (ADR-0039 — the speech acts' facts ride
    /// the doors and the same drain).
    /// Replaces any previous observer. Installing one also enables the
    /// edit trail, the announcement trail, the text-write trail
    /// (ADR-0061), the preedit trail (ADR-0067), the a11y-echo
    /// trail (ADR-0073), the action trail (ADR-0075), the
    /// frame-refusal trail (ADR-0077), and the theme trail (ADR-0079):
    /// the journal,
    /// the push route, the AT's editing tier, the composing tier, the
    /// adapter tier, the activation tier, the frame tier, and the theme
    /// tier note their facts on this thread from here on, and the
    /// disabled paths stay one branch per action.
    pub fn set_observer(&mut self, observer: Box<dyn InspectObserver>) {
        self.observer = Some(observer);
        crate::history::set_edit_trail_enabled(true);
        crate::announce::set_announce_trail_enabled(true);
        crate::clipboard::set_clipboard_trail_enabled(true);
        crate::value::set_value_trail_enabled(true);
        crate::gesture::set_gesture_trail_enabled(true);
        crate::scroll::set_scroll_trail_enabled(true);
        crate::text_write::set_text_write_trail_enabled(true);
        crate::route_drop::set_route_drop_trail_enabled(true);
        crate::key_edit::set_key_edit_trail_enabled(true);
        crate::preedit::set_preedit_trail_enabled(true);
        crate::a11y_echo::set_a11y_echo_trail_enabled(true);
        crate::action::set_action_trail_enabled(true);
        crate::frame_refusal::set_frame_refusal_trail_enabled(true);
        crate::view_patch::set_view_patch_trail_enabled(true);
        crate::theme::set_theme_trail_enabled(true);
        crate::settings::set_setting_trail_enabled(true);
        crate::cursor::set_cursor_trail_enabled(true);
    }

    /// Installs the theme seam (§7.2 pass 8, ADR-0079): the palette the
    /// widgets borrow for their un-overridden paint roles. The next
    /// paint pass resolves the roles once and repaints every node — the
    /// swap broadcasts `Change::PAINT` on the tree, the mask's own
    /// documented semantics ("a color change repaints this node only"),
    /// never a private repaint channel — and the swap streams one
    /// [`InspectEvent::Theme`] the tick after it lands (the theme
    /// trail's door-side note, the route-drop trail's shape).
    ///
    /// Answers `false` on a swap that lands where the palette already
    /// is: no broadcast, no record, no facts (the idle-frame property,
    /// A1's rule at the theme tier — the value tier's write rule, priced
    /// where the write resolves). Answers `true` when the palette
    /// landed. The app's explicit overrides are never touched: the
    /// roles a widget borrowed re-resolve, the colors it was handed
    /// stay its own.
    pub fn set_theme(&mut self, theme: Box<dyn crate::theme::ThemeSeam>) -> bool {
        let palette = theme.palette();
        if self
            .theme
            .as_deref()
            .is_some_and(|installed| installed.palette() == palette)
        {
            return false;
        }
        // The broadcast: every node repaints — the palette the whole tree
        // borrowed moved. Collected first (`for_each_pre_order` borrows
        // the arena immutably), merged after; the walk's length *is* the
        // swap's reach, the record's count computed here where the
        // broadcast resolves, never re-counted at the reader (ADR-0084 —
        // the lengths rule, the ids never riding).
        let mut nodes = Vec::new();
        self.arena.for_each_pre_order(&mut |id, _| nodes.push(id));
        let repainted = nodes.len();
        crate::theme::note_theme(crate::inspect::ThemeRecord {
            palette: theme.name().to_string(),
            repainted,
        });
        self.theme = Some(theme);
        for node in nodes {
            self.arena.merge_mask(node, Change::PAINT);
        }
        true
    }

    /// Installs the setting seam (§7.2 pass 8, ADR-0080): the OS's
    /// facts the platform adapter pushes through the door —
    /// on winit, the window's opening fact and the `ThemeChanged`
    /// event. The next paint pass resolves the facts once and repaints
    /// every node — the flip broadcasts `Change::PAINT` on the tree,
    /// the mask's own documented semantics ("a color change repaints
    /// this node only"), never a private repaint channel — and the
    /// flip streams one [`InspectEvent::Setting`] the tick after it
    /// lands (the theme trail's door-side note, ADR-0079's shape).
    /// Since M77 (ADR-0083) the seam carries the OS's reduced-motion
    /// fact beside the dark one, since M79 (ADR-0085) the OS's
    /// high-contrast fact beside the two, and since M81 (ADR-0087) the
    /// OS's accent-color fact beside the three (riding its presence —
    /// `None` is the honest absence) — one seam, one door, the flip
    /// landing when *any* fact moves.
    ///
    /// The facts never munge the palette: the roles the widgets
    /// borrowed re-resolve from the *installed palette* exactly as
    /// they were — an app that wants palette-following installs the
    /// composition door ([`Self::set_composer`], ADR-0081), the
    /// pipeline resolving the composed palette on this door's landed
    /// flips.
    ///
    /// Answers `false` on a flip that lands where the facts already are
    /// — including the canon light installed where nothing is (the
    /// default build resolves light): no broadcast, no record, no
    /// facts (the idle-frame property, A1's rule at the setting tier —
    /// the theme tier's write rule, priced where the write resolves).
    /// Answers `true` when the flip landed.
    pub fn set_settings(&mut self, settings: Box<dyn crate::settings::SettingSeam>) -> bool {
        let dark = settings.dark_mode();
        let reduced_motion = settings.reduced_motion();
        let high_contrast = settings.high_contrast();
        let accent = settings.accent_color();
        let (installed_dark, installed_motion, installed_contrast, installed_accent) = self
            .settings
            .as_deref()
            .map_or((false, false, false, None), |installed| {
                (
                    installed.dark_mode(),
                    installed.reduced_motion(),
                    installed.high_contrast(),
                    installed.accent_color(),
                )
            });
        if installed_dark == dark
            && installed_motion == reduced_motion
            && installed_contrast == high_contrast
            && installed_accent == accent
        {
            return false;
        }
        // The broadcast: every node repaints — the facts the whole tree
        // consulted moved. Collected first (`for_each_pre_order` borrows
        // the arena immutably), merged after; the walk's length *is* the
        // flip's reach, the record's count computed here where the
        // broadcast resolves, never re-counted at the reader (ADR-0082 —
        // the lengths rule, the ids never riding).
        let mut nodes = Vec::new();
        self.arena.for_each_pre_order(&mut |id, _| nodes.push(id));
        let repainted = nodes.len();
        crate::settings::note_setting(crate::inspect::SettingRecord {
            dark,
            reduced_motion,
            high_contrast,
            accent,
            repainted,
        });
        self.settings = Some(settings);
        for node in nodes {
            self.arena.merge_mask(node, Change::PAINT);
        }
        // The composition half (§7.2 pass 8, ADR-0081): the flip's own
        // record resolves the installed composer — the composed palette
        // re-swapped the way `set_theme` swaps (the equality check, the
        // theme fact tagged with the composer's name). The swap rides
        // the flip's own broadcast — the whole tree repaints either
        // way, no second walk over the arena — and the composed swap's
        // record names the same reach the flip broadcast (ADR-0084: one
        // flip, one count, however many palettes the composition
        // resolved). A flip that composes the palette already installed
        // re-swaps nothing — the swap half of the door is the no-op
        // (the idle rule at the theme tier), the flip itself still
        // landed (the fact moved).
        self.compose_from(
            crate::settings::OsFacts {
                dark,
                reduced_motion,
                high_contrast,
                accent,
            },
            false,
            repainted,
        );
        true
    }

    /// Installs the composition seam (§7.2 pass 8, ADR-0081): the
    /// palette the OS's fact resolves into — ADR-0080's pre-registered
    /// re-open rule cashing ("a consumer needs the palette to *follow*
    /// the OS fact → the pipeline grows the composition door, the
    /// installed palette re-swapped on the flip's own record, priced by
    /// a real reader — never a palette that reaches into the platform's
    /// adapter").
    ///
    /// The install resolves *now*: the composer answers its palette for
    /// the fact that is (the installed setting seam's, or the canon
    /// light fact where nothing is installed), and the door swaps it in
    /// the way [`Self::set_theme`] swaps — the `PartialEq` idle check,
    /// one [`InspectEvent::Theme`] tagged with the composer's own
    /// [`crate::settings::SettingComposer::name`], `Change::PAINT` on the tree. Every
    /// later flip that lands through [`Self::set_settings`] re-resolves
    /// the composition the same way — the palette follows the fact
    /// while the composer is installed.
    ///
    /// Answers `false` on an install that composed the palette already
    /// live — including the canon composition over the canon default
    /// (the default build resolves the canon palette): no broadcast, no
    /// record, the composer still riding for the next flip. Answers
    /// `true` when the install moved the palette. The app's own
    /// [`Self::set_theme`] ask lands over the composed answer and holds
    /// until the next flip — the manual door wins while the fact is
    /// still, the composition owning the palette only when the OS
    /// moves.
    pub fn set_composer(&mut self, composer: Box<dyn crate::settings::SettingComposer>) -> bool {
        let facts = self
            .settings
            .as_deref()
            .map_or(crate::settings::OsFacts::CANON, |installed| {
                crate::settings::OsFacts {
                    dark: installed.dark_mode(),
                    reduced_motion: installed.reduced_motion(),
                    high_contrast: installed.high_contrast(),
                    accent: installed.accent_color(),
                }
            });
        self.composer = Some(composer);
        // The install path walks its own broadcast — no flip reach to
        // name, the count resolved where the install's walk runs
        // (ADR-0084's shape at the composition door).
        let landed = self.compose_from(facts, true, 0);
        if !landed {
            tracing::trace!(
                target: "alacer::settings",
                "composer installed where the fact already resolves its palette"
            );
        }
        landed
    }

    /// The composition's resolution half (ADR-0081): the installed
    /// composer answers its palette by the facts the door resolved
    /// (ADR-0086 — the signature the composition asks for), and the
    /// answer swaps in as the installed theme the roles borrow — the
    /// theme door's own swap rules, priced where the swap resolves.
    /// Answers whether a palette moved; a composer resolving the
    /// palette already installed (or no composer at all) is the no-op.
    /// `broadcast` merges the repaint currency itself — the install
    /// path carries it, the flip path skips it (the flip's own
    /// broadcast already walked the tree). `flip_reach` is the count
    /// the flip's own broadcast resolved: the composed swap's record
    /// names it on the flip path (ADR-0084 — one flip, one count),
    /// while the install path walks its own broadcast and names that
    /// walk's length.
    fn compose_from(
        &mut self,
        facts: crate::settings::OsFacts,
        broadcast: bool,
        flip_reach: usize,
    ) -> bool {
        let Some(resolved) = self.composer.as_deref().map(|composer| {
            (
                composer.palette(
                    facts.dark,
                    facts.reduced_motion,
                    facts.high_contrast,
                    facts.accent,
                ),
                composer.name(),
            )
        }) else {
            return false;
        };
        let (palette, name) = resolved;
        // The effective palette the tree paints: the installed theme's
        // roles, or the canon constants where nothing is installed —
        // the paint pass's own resolution (ADR-0079). A composition
        // resolving the palette already live is the no-op: the canon
        // composition over the canon default (the default build's own
        // answer) composes nothing, the theme tier's idle rule.
        let installed = self.theme.as_deref().map_or(
            crate::theme::Palette::CANON,
            crate::theme::ThemeSeam::palette,
        );
        if installed == palette {
            return false;
        }
        // The install path's own broadcast (`set_composer` reaches this
        // door outside a flip): the palette the whole tree borrowed
        // moved, the currency's own rule. Collected first
        // (`for_each_pre_order` borrows the arena immutably), merged
        // after — `set_theme`'s shape, the walk's length the count
        // (ADR-0084). The flip path skips the walk: the flip's own
        // broadcast already walked the tree, the count it resolved
        // riding the composed swap's record unchanged.
        let mut nodes = Vec::new();
        let repainted = if broadcast {
            self.arena.for_each_pre_order(&mut |id, _| nodes.push(id));
            nodes.len()
        } else {
            flip_reach
        };
        crate::theme::note_theme(crate::inspect::ThemeRecord {
            palette: name.to_string(),
            repainted,
        });
        self.theme = Some(Box::new(crate::settings::ComposedPalette::new(
            palette, name,
        )));
        if !broadcast {
            return true;
        }
        for node in nodes {
            self.arena.merge_mask(node, Change::PAINT);
        }
        true
    }

    /// Enables per-pass wall timing and the overhead ledger (the M1 gate
    /// instrumentation; benches and the overhead test turn this on).
    pub fn set_timings(&mut self, on: bool) {
        self.timings = on;
    }

    /// Installs the clock the timing reads answer (M104, ADR-0108): the
    /// time seam's own door beside [`Self::set_timings`]. Replaces any
    /// previous clock — the newest install wins, one door to swap a
    /// deterministic [`crate::time::ManualClock`] in and the canon real
    /// time back out. With no clock installed every read answers the
    /// fallback anchor (the pipeline's construction instant — real
    /// monotonic time), the shipped shape, so the default build's
    /// behavior is byte-for-byte the pre-seam tick's.
    pub fn set_clock(&mut self, clock: Box<dyn crate::time::TimeSeam>) {
        self.clock = Some(clock);
    }

    /// Installs the frame budget the degradation ladder arms against
    /// (§14.4): a frame that can't fit its budget degrades rung by rung —
    /// the input batch's remainder coalesces into the next frame
    /// ([`DegradationStep::DropInput`]), a behind-schedule paint
    /// presents one full frame ([`DegradationStep::ForceFullPaint`]),
    /// and a frame past the budget's halfway line defers the a11y
    /// pass's dirty updates ([`DegradationStep::SkipA11y`]) — their
    /// masks re-queue, the next frame lands the delta, and focus moves,
    /// announcements, first frames, and structural churn never defer
    /// (latency may degrade, correctness may not). `None` disarms the
    /// ladder: every pass runs unconditionally, the shipped default.
    ///
    /// The decisions read the same clock the ledger rides — the
    /// installed [`crate::time::TimeSeam`] answers them when one is
    /// installed, real monotonic time otherwise — so a seam clock makes
    /// the ladder deterministic in tests, exactly as it makes the
    /// ledger.
    pub fn set_frame_budget(&mut self, budget: Option<std::time::Duration>) {
        self.frame_budget = budget;
    }

    /// The installed frame budget, `None` when the ladder is disarmed
    /// (the default).
    #[must_use]
    pub const fn frame_budget(&self) -> Option<std::time::Duration> {
        self.frame_budget
    }

    /// One timing read: the installed seam's monotonic nanos, or the
    /// fallback anchor's elapsed when nothing is installed. Durations
    /// are differences of two reads (the shape both the pass ledger and
    /// the chain window ride), so the anchor's origin is arbitrary —
    /// never a calendar fact.
    fn read_clock_nanos(&self) -> u64 {
        match &self.clock {
            Some(clock) => clock.now_nanos(),
            None => crate::num::u128_to_u64(self.clock_anchor.elapsed().as_nanos()),
        }
    }

    /// Requests keyboard focus for `node` (resolved by pass 3 next tick).
    /// Requests for non-focusable, unavailable, or dead nodes are dropped
    /// silently (the focusable derivation includes the availability tier,
    /// M22 — ADR-0028).
    pub fn request_focus(&mut self, node: NodeId) {
        self.pending_focus = Some(node);
    }

    /// Moves focus one candidate in `movement`'s direction (§10.3,
    /// ADR-0030): the consumer's door over the same walk the keyboard's
    /// Tab/Shift+Tab fallback rides — one funnel, two doors, the order
    /// never duplicated at the call site. The walk follows the pre-order
    /// candidate list pass 2 recomputes, skips controls the availability
    /// tier holds down (the cycle-time `enabled` check, M22 — ADR-0028),
    /// and wraps at both ends; from no focus, either direction takes the
    /// first candidate (the Tab contract as shipped).
    ///
    /// Since M26 the vocabulary answers the window as well as the tree
    /// (ADR-0032): the four directional moves (`Left`/`Right`/`Up`/`Down`)
    /// read the bounds pass 6 wrote and select the candidate the geometry
    /// names — fully beyond the focused node's edge in the direction,
    /// nearest edge distance first — and never wrap (space has no wrap;
    /// the window's edge is the walk's end). A direction with no candidate
    /// refuses exactly like an empty walk: a named [`FocusRefusal`], no
    /// masks, the help
    /// carried as plain diagnostic text on this door's `alacer::focus`
    /// span.
    ///
    /// The resolution is immediate — focus moves *this* call, the way key
    /// routing requires — and the move rides the one invalidation
    /// currency: `FOCUS | PAINT` masks on the two endpoints, one frame of
    /// work. The refusal is honest (the M11 rule, at the focus seam):
    /// when no candidate exists — an empty arena, every control
    /// unavailable, or nothing in the named direction — the door answers
    /// a named [`FocusRefusal`] and queues nothing (an idle frame); it
    /// never munges the walk into a first-candidate reset.
    ///
    /// Since M27 the answer is data (ADR-0033, ADR-0031's re-open rule):
    /// a [`FocusMoveOutcome`] carrying the [`FocusLanding`]'s endpoints
    /// and wrap when the walk moved, the [`FocusRefusal`] species when it
    /// refused — the caller no longer guesses at a bare `false`.
    pub fn focus_move(&mut self, movement: FocusMove) -> FocusMoveOutcome {
        // The door is the app's own call: the record names the consumer
        // as the rider (ADR-0036) — the keyboard's and the ring's rides
        // stream through [`Self::focus_move_via`], the same funnel.
        self.focus_move_via(FocusRider::Consumer, movement)
    }

    /// Moves focus one candidate inside the subtree rooted at `scope`
    /// (§10.3, ADR-0031): the walk the keyboard's Tab fallback and
    /// [`Self::focus_move`] ride, bounded to a subtree — the modal's
    /// focus trap, the re-open rule ADR-0030 pre-registered. The scope is
    /// the subtree's root node (the cheapest honest predicate a real UI
    /// prices): candidates inside the trap follow the same pre-order
    /// list, the same cycle-time `enabled` filter (M22 — ADR-0028), the
    /// same wrap at both ends, and from no focus — or from a focus
    /// outside the trap — the walk takes the trap's first candidate. The
    /// scope bounds the walk, never the candidate list: `focusables()`
    /// and [`Self::focus`] keep answering the whole tree, and the
    /// unscoped door stays the keyboard's funnel.
    ///
    /// The resolution is immediate and the move rides the one invalidation
    /// currency (`FOCUS | PAINT` masks on the two endpoints, one frame of
    /// work), exactly as the unscoped door. The refusal is honest (the
    /// M11 rule, at the focus seam): a dead scope, an empty trap, or a
    /// trap the availability tier holds down answers a named
    /// [`FocusRefusal`] and queues nothing — an idle frame, focus left
    /// where it was, never a first-candidate reset. The refusals carry
    /// their help on the `alacer::focus` target as plain diagnostic text
    /// (ADR-0029's rule — the `help:` sentence the ADR-0029 species
    /// ships, never the diagnostic crate it is named for). The direction
    /// is the caller's vocabulary (M26, ADR-0032): the directional moves
    /// read the same bounds and never wrap, the scope bounding the
    /// directional walk exactly as it bounds the tree-order one — a
    /// direction with no candidate inside the trap refuses honestly; it
    /// never escapes the scope. Since M27 the answer is the outcome
    /// vocabulary (ADR-0033): the dead scope names itself
    /// ([`FocusRefusal::DeadScope`]), the held-down trap names the empty
    /// list ([`FocusRefusal::NoCandidates`]).
    pub fn focus_move_in(&mut self, scope: NodeId, movement: FocusMove) -> FocusMoveOutcome {
        let span = tracing::trace_span!(
            target: "alacer::focus",
            "focus_move_in",
            direction = movement.name(),
            scope = scope.raw()
        );
        let _guard = span.enter();
        if !self.node_is_live(scope) {
            tracing::trace!(
                target: "alacer::focus",
                scope = scope.raw(),
                "refused: the scope node is dead — help: focus_move_in walks the live subtree rooted at `scope`; the unscoped door is focus_move"
            );
            let refused = FocusMoveOutcome::Refused(FocusRefusal::DeadScope { scope });
            // The early refusal streams too (ADR-0034): the causal log
            // answers "why didn't focus change" for the dead scope. The
            // door is the app's own call — the rider is the consumer
            // (ADR-0036).
            self.notify_focus(FocusRider::Consumer, movement, Some(scope), refused);
            return refused;
        }
        let outcome = self.cycle_focus_in(Some(scope), movement);
        // The scoped ride streams with its scope named (ADR-0034) — the
        // trap's landings and refusals trail beside the unscoped ones.
        // The door is the app's own call: the rider is the consumer
        // (ADR-0036).
        self.notify_focus(FocusRider::Consumer, movement, Some(scope), outcome);
        if let Some(landing) = outcome.landing() {
            tracing::trace!(
                target: "alacer::focus",
                node = landing.to.raw(),
                scope = scope.raw(),
                "the walk rests on the next candidate inside the trap"
            );
        } else {
            tracing::trace!(
                target: "alacer::focus",
                scope = scope.raw(),
                "refused: no candidate inside the scope — help: a trap the availability tier holds down (or an empty subtree) leaves the walk nothing to offer; an idle frame, focus untouched"
            );
        }
        outcome
    }

    /// Brings `node` fully inside its nearest scrollable ancestor's
    /// viewport (§8.2, M88, ADR-0092): the geometry-driven scroll ask —
    /// the counterpart the offset-priced doors (the wheel, the AT's
    /// unit scrolls, the app's raw-offset handle) have left open,
    /// because the ask consumers actually hold is priced in *nodes*
    /// ("this item, this row, this focused control"), not offsets.
    ///
    /// The pipeline reads the bounds ledger it already owns — pass 6's
    /// absolute composed bounds, pass 7's applied offsets — and prices
    /// the minimal offset that lands the node's rect fully inside the
    /// container's viewport: the node above the window scrolls to its
    /// top edge, below it scrolls to its bottom edge, a rect taller
    /// than the viewport pins its top edge (deterministic, the
    /// scroll-into-view convention), and a node already fully visible
    /// answers [`ScrollIntoViewOutcome::Visible`] — nothing moved,
    /// nothing queued (the M51 idle property at the door's tier). The
    /// ask then rides the container's own door
    /// ([`crate::widget::Widget::bring_into_view`]): the clamp
    /// arithmetic — the span walls, the named refusals — answers where
    /// the range lives, never the pipeline guessing at a widget-private
    /// span. Since M89 (ADR-0093) the ask carries both halves
    /// ([`crate::widget::BringIntoViewAsk`]): the vertical half exactly
    /// the M88 ask, the horizontal half beside it priced by the same
    /// edge-pin convention from the same ledger read — left of the
    /// window scrolls to the left edge, right of it to the right edge,
    /// a rect wider than the viewport pins its left edge, and the axis
    /// that already holds the node asks its own identity (the clamp
    /// delivering the offset it holds). The container answers each axis
    /// by its own declared span; the landing reports both, the
    /// refusals named per axis — a container whose horizontal span was
    /// never declared answers
    /// [`crate::inspect::ScrollRefusal::NoScrollableContent`] on the x
    /// half while the vertical half lands, the honest wall where the
    /// consumer can read it.
    ///
    /// The answer is data at every landing (the M27 rule at the scroll
    /// tier): a [`ScrollIntoViewOutcome::Landed`] carrying the
    /// [`ScrollIntoViewLanding`] (the container that moved beside the
    /// container's own answer), [`ScrollIntoViewOutcome::Visible`]
    /// beside the honest no-op, and a named [`ScrollIntoViewRefusal`]
    /// when the ask died before any scroll answer — a dead id, a node
    /// the layout has not answered yet, no scrollable ancestor, a
    /// container claiming the scroll slot without the door. The
    /// refusals queue no masks and stream no record: a question
    /// outside the tier is not a tier fact (the ADR-0046 rule the
    /// scroll tier rides). The landing streams
    /// one [`crate::inspect::ScrollRecord`] under
    /// [`crate::inspect::ScrollAction::BringIntoView`] — the record
    /// naming the container it rode (`target`) and the node it was for
    /// (the additive `aimed` stamp, the M30 attribution rule at the
    /// scroll tier) — and queues `PAINT | A11Y` on the container, the
    /// same currency the wheel's dispatch merges on consumption (§7.2
    /// pass 10). The nearest ancestor only: a node nested two scrolls
    /// deep asks the one it sits in — the deeper ones are the next
    /// ask's fact, never a cascade this door guesses at.
    pub fn scroll_into_view(&mut self, node: NodeId) -> ScrollIntoViewOutcome {
        // The dead id: no geometry exists to read, no container to ask.
        if !self.arena.is_live(node) {
            tracing::trace!(
                target: "alacer::scroll",
                node = node.raw(),
                "refused: the aimed id is not a live node — help: a stale handle or a sentinel cannot be brought anywhere; an idle frame, nothing queued"
            );
            return ScrollIntoViewOutcome::Refused(ScrollIntoViewRefusal::UnknownNode);
        }
        // The unlaid-out node: the bounds ledger holds no answer for it
        // yet — the door never guesses geometry the layout has not
        // written.
        let Some(child_abs) = self.arena.bounds(node) else {
            tracing::trace!(
                target: "alacer::scroll",
                node = node.raw(),
                "refused: the aimed node has no composed bounds yet — help: settle a tick so pass 6 writes the ledger the door reads; an idle frame, nothing queued"
            );
            return ScrollIntoViewOutcome::Refused(ScrollIntoViewRefusal::NotLaidOut);
        };
        // The nearest scrollable ancestor, from the node upward — the
        // aimed node itself excluded (a container does not scroll
        // itself into itself; its own rect rides its ancestor's
        // viewport).
        let Some(container) = self.nearest_scrollable_ancestor(node) else {
            tracing::trace!(
                target: "alacer::scroll",
                node = node.raw(),
                "refused: no scrollable ancestor above the aimed node — help: the ask lands nowhere because nothing here scrolls; an idle frame, nothing queued"
            );
            return ScrollIntoViewOutcome::Refused(ScrollIntoViewRefusal::NoScrollableAncestor);
        };
        let Some(container_abs) = self.arena.bounds(container) else {
            // A descendant with bounds under a container without them
            // cannot happen after a layout pass; the honest refusal if
            // it ever does.
            tracing::trace!(
                target: "alacer::scroll",
                node = node.raw(),
                "refused: the container has no composed bounds yet — an idle frame, nothing queued"
            );
            return ScrollIntoViewOutcome::Refused(ScrollIntoViewRefusal::NotLaidOut);
        };
        let Some(ask) = self.price_two_axis_ask(container, node, child_abs, container_abs) else {
            return ScrollIntoViewOutcome::Visible;
        };
        // The stamps (M43, ADR-0049, the M50 discipline at the scroll
        // tier): the route knows the container it resolved and the
        // node the ask was for — the facts the container's door queues
        // carry both, the widget never guessing its own identity. The
        // idle contract holds: while nobody listens the stamps are one
        // `Cell` read and no id-path walk.
        let container_hash = self.id_hash(container);
        let aimed_hash = self.id_hash(node);
        let _target_stamp = crate::scroll::scroll_trail_enabled()
            .then(|| crate::scroll::ScrollTargetGuard::for_target(Some(container_hash)));
        let _aim_stamp = crate::scroll::scroll_trail_enabled()
            .then(|| crate::scroll::ScrollAimGuard::for_aim(Some(aimed_hash)));
        let answer = {
            let mut guard = self.arena.mutate(container);
            guard.widget_mut().bring_into_view(ask)
        };
        let Some(landing) = answer else {
            tracing::trace!(
                target: "alacer::scroll",
                node = node.raw(),
                "refused: the container claims the scroll slot without a bring-into-view door — the contract gap named, never skipped in silence"
            );
            return ScrollIntoViewOutcome::Refused(ScrollIntoViewRefusal::NoScrollDoor);
        };
        if landing.outcome == crate::inspect::ScrollOutcome::Delivered
            || landing.horizontal.outcome == crate::inspect::ScrollOutcome::Delivered
        {
            // The same currency the wheel's dispatch merges on
            // consumption (§7.2 pass 10): the repaint and the a11y
            // bounds re-announce ride the one batch, compose
            // translating the moved descendants without a measure
            // call. Either axis's delivery moves pixels — the mask
            // rides the axis that answered.
            self.arena
                .merge_mask(container, crate::Change::PAINT.union(crate::Change::A11Y));
        }
        tracing::trace!(
            target: "alacer::scroll",
            node = node.raw(),
            container = container.raw(),
            offset = landing.offset,
            offset_x = landing.horizontal.offset,
            "the bring-into-view ask answered beside the container's own clamp"
        );
        ScrollIntoViewOutcome::Landed(ScrollIntoViewLanding {
            container,
            outcome: landing.outcome,
            offset: landing.offset,
            pinned: landing.pinned,
            horizontal: landing.horizontal,
        })
    }

    /// The two-axis ask's price (M89, ADR-0093), from the ledger the
    /// door already owns: the node's content-space rect rides the
    /// bounds' own difference on both axes, each priced by the same
    /// edge-pin convention — above/left of the window scrolls to that
    /// edge (a rect taller/wider than the viewport pins its top/left
    /// edge), below/right to the bottom/right edge, and the axis that
    /// already holds the node asks its own identity, the clamp
    /// delivering the offset it holds. `None` when the node sits fully
    /// inside on both axes — the honest no-op the door answers
    /// `Visible` with, nothing to ask, the container never disturbed.
    fn price_two_axis_ask(
        &self,
        container: NodeId,
        node: NodeId,
        child_abs: crate::geom::Rect,
        container_abs: crate::geom::Rect,
    ) -> Option<crate::widget::BringIntoViewAsk> {
        let offsets = self.arena.widget(container).scroll_offset();
        let current_y = offsets.y;
        let current_x = offsets.x;
        let content_top = child_abs.origin.y - container_abs.origin.y;
        let content_bottom = content_top + child_abs.size.height;
        let viewport_height = container_abs.size.height;
        let visible_top = content_top - current_y;
        let visible_bottom = content_bottom - current_y;
        let content_left = child_abs.origin.x - container_abs.origin.x;
        let content_right = content_left + child_abs.size.width;
        let viewport_width = container_abs.size.width;
        let visible_left = content_left - current_x;
        let visible_right = content_right - current_x;
        if visible_top >= 0.0
            && visible_bottom <= viewport_height
            && visible_left >= 0.0
            && visible_right <= viewport_width
        {
            tracing::trace!(
                target: "alacer::scroll",
                node = node.raw(),
                "visible: the aimed node sits fully inside the viewport — nothing to move, nothing queued"
            );
            return None;
        }
        let ask_y = if visible_top < 0.0 {
            content_top
        } else if visible_bottom > viewport_height {
            content_bottom - viewport_height
        } else {
            current_y
        };
        let ask_x = if visible_left < 0.0 {
            content_left
        } else if visible_right > viewport_width {
            content_right - viewport_width
        } else {
            current_x
        };
        Some(crate::widget::BringIntoViewAsk {
            vertical: ask_y,
            horizontal: ask_x,
        })
    }

    /// The nearest ancestor of `node` whose widget claims the scroll
    /// slot ([`crate::widget::Widget::is_scroll_container`]) — the
    /// arena's own parent links, one walk per ask, the aimed node
    /// itself excluded.
    fn nearest_scrollable_ancestor(&self, node: NodeId) -> Option<NodeId> {
        let mut cursor = self.arena.parent(node);
        while let Some(id) = cursor {
            if self.arena.widget(id).is_scroll_container() {
                return Some(id);
            }
            cursor = self.arena.parent(id);
        }
        None
    }

    /// Queues a raw pointer event into the frame-batched input queue (§10.2,
    /// ADR-0014). The event coalesces with the pending batch — compatible
    /// consecutive moves merge, same-direction wheel deltas accumulate — and
    /// the batch routes at the start of the next [`Self::tick`]: a storm
    /// costs one frame of work, not N events. The platform layer pushes and
    /// never drains.
    pub fn queue_pointer(&mut self, event: crate::events::PointerEvent) {
        self.input.push(event);
    }

    /// The node currently holding the pointer capture (§10.1, ADR-0025),
    /// if a press is in flight. Introspection for tests, probes, and the
    /// inspector — the routing itself reads the field directly.
    #[must_use]
    pub const fn captured(&self) -> Option<NodeId> {
        self.captured
    }

    /// Announces `text` to assistive technologies (§11.6, ADR-0024): the
    /// reserved live-region node's content and politeness update, and the
    /// next tick's a11y delta re-emits that node — the same snapshot, the
    /// same `Change::A11Y` currency as every other a11y fact (A1; no
    /// second channel, no platform round trip).
    ///
    /// The contract ([`crate::announce`]): the region is always present
    /// (the root's last child from the first frame), last write wins
    /// (rapid calls replace — one batch, one spoken fact), and the
    /// refusals are honest — an empty or all-whitespace `text` speaks
    /// nothing, and before a root is mounted there is no tree to attach
    /// the region to. A refused announcement queues no masks (an idle
    /// frame) and returns `false`; an accepted one returns `true`.
    ///
    /// Since M33 every answer streams one `InspectEvent::Announce` to the
    /// observer the moment it resolves (ADR-0039) — the focus doors'
    /// shape at the speech-act tier: the acceptance and both refusals
    /// are facts the causal log carries.
    pub fn announce(
        &mut self,
        text: impl Into<String>,
        politeness: crate::announce::Politeness,
    ) -> bool {
        let text = text.into();
        if text.trim().is_empty() {
            tracing::trace!(
                target: "alacer::announce",
                "refused: an empty announcement speaks nothing"
            );
            self.notify_announce(crate::inspect::AnnounceRecord {
                politeness,
                outcome: crate::inspect::AnnounceOutcome::Refused(
                    crate::inspect::AnnounceRefusal::EmptyText,
                ),
            });
            return false;
        }
        if self.arena.root().is_none() {
            tracing::trace!(
                target: "alacer::announce",
                "refused: no root is mounted, the region has no tree to join"
            );
            self.notify_announce(crate::inspect::AnnounceRecord {
                politeness,
                outcome: crate::inspect::AnnounceOutcome::Refused(
                    crate::inspect::AnnounceRefusal::NoRoot,
                ),
            });
            return false;
        }
        let announcement = crate::announce::Announcement { text, politeness };
        tracing::trace!(
            target: "alacer::announce",
            politeness = announcement.politeness.name(),
            chars = announcement.text.len(),
            "announcement rides the next a11y delta"
        );
        self.announcement = Some(announcement);
        self.announcement_dirty = true;
        self.notify_announce(crate::inspect::AnnounceRecord {
            politeness,
            outcome: crate::inspect::AnnounceOutcome::Delivered,
        });
        true
    }

    /// The last accepted announcement (introspection): what the reserved
    /// region carries right now. `None` before the first accepted
    /// `announce` — the region exists from the first frame with an empty
    /// name, but nothing has been spoken yet.
    #[must_use]
    pub const fn announcement(&self) -> Option<&crate::announce::Announcement> {
        self.announcement.as_ref()
    }

    /// The focused node, if any (pass 3 output).
    #[must_use]
    pub const fn focus(&self) -> Option<NodeId> {
        self.focused
    }

    /// The focus candidates pass 2 last computed.
    #[must_use]
    pub fn focusables(&self) -> &[NodeId] {
        &self.focusables
    }

    /// Mounts the root widget.
    #[must_use]
    pub fn mount_root(&mut self, widget: Box<dyn Widget>) -> NodeId {
        self.arena.set_root(widget)
    }

    /// Appends a child widget.
    #[must_use]
    pub fn add_child(&mut self, parent: NodeId, widget: Box<dyn Widget>) -> NodeId {
        self.arena.add_child(parent, widget)
    }

    /// Shared borrow of the arena.
    #[must_use]
    pub const fn arena(&self) -> &Arena {
        &self.arena
    }

    /// Mutable borrow of the arena (widget wiring between frames).
    #[must_use]
    pub const fn arena_mut(&mut self) -> &mut Arena {
        &mut self.arena
    }

    /// The flux (signal handles stay valid; bindings are wired through here).
    #[must_use]
    pub const fn flux(&self) -> &Flux {
        &self.flux
    }

    /// Mutable access to the flux for binding new signals.
    #[must_use]
    pub const fn flux_mut(&mut self) -> &mut Flux {
        &mut self.flux
    }

    /// Frame counters.
    #[must_use]
    pub const fn counters(&self) -> FrameCounters {
        self.counters
    }

    /// The scene journal's ledger (ADR-0029): commits, drops, and live
    /// layers, as the journal itself accounts them. One read, no second
    /// channel — the consumer's window into what one write batch cost
    /// (`examples/frame_ledger.rs` is the demonstration).
    #[must_use]
    pub fn journal_stats(&self) -> JournalStats {
        self.journal.stats()
    }

    /// The a11y sink's resolved tree (harness queries).
    #[must_use]
    pub fn a11y_snapshot(&self) -> Option<&TreeSnapshot> {
        self.a11y.resolved_tree()
    }

    /// The installed a11y sink, mutably — the platform's drain seam
    /// (ADR-0015): downcast through [`A11ySink::as_any_mut`] to the concrete
    /// sink and read the updates the pass produced. One accessor, no second
    /// pipeline path.
    pub fn a11y_sink_mut(&mut self) -> &mut dyn A11ySink {
        self.a11y.as_mut()
    }

    /// The IME request the last tick derived from the focused widget (§10.3);
    /// `None` when no editable widget holds focus. The platform layer applies
    /// the delta frame over frame: attach/detach the IME, move the candidate
    /// window to `caret`.
    #[must_use]
    pub const fn ime_request(&self) -> Option<&ImeRequest> {
        self.ime_request.as_ref()
    }

    /// The layout engine's measure-call counter.
    #[must_use]
    pub fn measure_calls(&self) -> u64 {
        self.layout.measure_calls()
    }

    /// Resets the layout engine's measure-call counter.
    pub fn reset_measure_calls(&mut self) {
        self.layout.reset_measure_calls();
    }

    /// Pass 1 (`event`): route the event through the capture→bubble contract
    /// (§10.1). While a press holds its capture (ADR-0025), the event rides
    /// the captured node's ancestor chain; otherwise the pointer position is
    /// hit-tested first. Either way the phases are the same: ancestors see
    /// it first during capture (root → target), the target and its ancestors
    /// see it again during bubble (target → root); the first `Handled` stops
    /// the phase. Z-order-aware: the deepest, last-painted hit is the target.
    pub fn dispatch_pointer(&mut self, event: &PointerEvent) -> EventResponse {
        self.counters.events_routed += 1;
        let mut run = PassRun {
            kind: PassKind::Event,
            ran: false,
            nanos: 0,
            work_nanos: 0,
        };
        let span =
            tracing::trace_span!(target: "alacer::pass", "pass", pass = PassKind::Event.name());
        let start = self.timings.then(|| self.read_clock_nanos());
        let guard = span.enter();

        let (response, target) = self.route_pointer(event, &mut run.ran);

        run.nanos = start.map_or(0, |t| self.read_clock_nanos().saturating_sub(t));
        drop(guard);
        self.interframe.push(run);
        self.notify_input(Self::pointer_kind(event), target);
        response
    }

    /// The pointer hover's cursor ask (M152, ADR-0156): the
    /// [`crate::cursor::CursorStyle`] the platform applies while the
    /// pointer rests at `point` — the deepest accepting hit's declared
    /// [`crate::widget::Widget::cursor_style`] word, or the default when
    /// nothing accepts the hover. The walk is the pointer route's own
    /// (the same pre-order bounds sweep, the same deepest-hit chain), so
    /// the hover the cursor names is exactly the hover the clicks land
    /// on: two vocabularies, one geometry. A read, never a pass — no
    /// dirt, no frames, no pass run: the platform resolves per frame and
    /// owns the delta discipline (the IME request's own rule, §10.3).
    ///
    /// Since M153 (ADR-0157) the resolution is the cursor trail's note
    /// site: one [`crate::inspect::CursorRecord`] per resolved *change*
    /// queues where the answer lands (the clipboard trail's shape, the
    /// flood rule at the cursor tier — a style that did not move is not
    /// a fact), streamed at the tick's start drain. The disabled trail's
    /// whole cost is the note's one branch.
    #[must_use]
    pub fn cursor_at(&self, point: crate::geom::Point) -> crate::cursor::CursorStyle {
        let mut deepest: Option<NodeId> = None;
        self.arena.for_each_pre_order(&mut |id, slot| {
            let contains = slot.bounds.is_some_and(|b| b.contains(point));
            let accepts = slot.widget().accepts_pointer();
            if contains && accepts {
                deepest = Some(id);
            }
        });
        let (style, source) = deepest.map_or(
            (
                crate::cursor::CursorStyle::Default,
                crate::inspect::CursorSource::Default,
            ),
            |id| {
                let widget = self.arena.widget(id);
                (
                    widget.cursor_style(),
                    crate::inspect::CursorSource::Widget {
                        node: id.raw(),
                        role: widget.role(),
                    },
                )
            },
        );
        crate::cursor::note_resolution(style, source);
        style
    }

    /// The event pass body: the captured route when a press is in flight
    /// (§10.1, ADR-0025), else the hit test with its capture walk and
    /// bubble walk. Returns the target the inspector records — the
    /// captured node, or the deepest hit.
    fn route_pointer(
        &mut self,
        event: &PointerEvent,
        ran: &mut bool,
    ) -> (EventResponse, Option<NodeId>) {
        // The captured route (ADR-0025): while a press holds a node, its
        // Move/Up/Leave stream routes through that node's ancestor chain —
        // wherever the pointer is — until the Up or Leave ends the
        // gesture. `Down` and `Wheel` stay on the hit test below: a press
        // re-captures, and the wheel follows the hover, not the capture
        // (the DOM's shape). A dead target releases the pointer and falls
        // through to the ordinary route.
        if !matches!(
            event,
            PointerEvent::Down { .. } | PointerEvent::Wheel { .. }
        ) && let Some(routed) = self.route_captured(event, ran)
        {
            return routed;
        }
        let Some(position) = event.position() else {
            return (EventResponse::Ignored, None);
        };
        let mut chain = Vec::new();
        self.arena.for_each_pre_order(&mut |id, slot| {
            let contains = slot.bounds.is_some_and(|b| b.contains(position));
            let accepts = slot.widget().accepts_pointer();
            if contains && accepts {
                chain.push(id);
            }
        });
        if chain.is_empty() {
            return (EventResponse::Ignored, None);
        }
        *ran = true;
        // The implicit capture (ADR-0025): the press holds its deepest
        // accepting hit — the target the bubble walk answers first —
        // so the gesture's remaining Move/Up stream keeps arriving.
        if let PointerEvent::Down { .. } = event
            && let Some(target) = chain.last().copied()
        {
            self.captured = Some(target);
            tracing::trace!(
                target: "alacer::capture",
                node = target.raw(),
                "captured: the press holds its target until the release"
            );
        }

        // The gesture trail's stamp (M46, ADR-0052): the route knows the
        // control it is resolving — the deepest hit — the facts a value
        // tier's drag door queues beneath this event carrying it (the
        // M43/M45 stamp discipline at the pointer tier). The idle
        // contract holds: while nobody listens the stamp is one `Cell`
        // read and no id-path walk (the trail is off, no fact can
        // queue); under an observer the guard lives for the length of
        // this event's walks and dies with them — a panicking widget
        // leaves no stale identity on the thread.
        let _stamp = crate::gesture::gesture_trail_enabled().then(|| {
            crate::gesture::GestureTargetGuard::for_target(
                chain.last().copied().map(|id| self.id_hash(id)),
            )
        });

        // The scroll trail's stamp (M50, ADR-0056): the route knows the
        // control it is resolving — the deepest hit under the hover —
        // the facts a scroll door queues beneath a routed *wheel*
        // carrying it (the M46 discipline at the scroll tier; the wheel
        // is the only pointer ask the scroll tier answers, so the stamp
        // rides the wheel asks alone). The idle contract holds: while
        // nobody listens the stamp is one `Cell` read and no id-path
        // walk (the trail is off, no fact can queue); under an observer
        // the guard lives for the length of this event's walks and dies
        // with them — a panicking widget leaves no stale identity on
        // the thread.
        let _scroll_stamp = (matches!(event, PointerEvent::Wheel { .. })
            && crate::scroll::scroll_trail_enabled())
        .then(|| {
            crate::scroll::ScrollTargetGuard::for_target(
                chain.last().copied().map(|id| self.id_hash(id)),
            )
        });

        // Capture: root → target (§10.1). First `Handled` stops the phase.
        for node in &chain {
            let response = self.visit_with_phase(*node, EventPhase::Capture, event);
            if response == EventResponse::Handled {
                return (EventResponse::Handled, chain.last().copied());
            }
        }
        // Bubble: target → root. Reverse pre-order tail = deepest, topmost
        // (last painted) hit first.
        for node in chain.iter().rev() {
            let response = self.visit_with_phase(*node, EventPhase::Bubble, event);
            if response == EventResponse::Handled {
                return (EventResponse::Handled, chain.last().copied());
            }
        }
        (EventResponse::Ignored, chain.last().copied())
    }

    /// The captured route (§10.1, ADR-0025): `event` rides the captured
    /// node's ancestor chain — capture walk, then bubble walk, the same
    /// phases the hit-test route walks; only the target comes from the
    /// press, not from where the pointer is now. Returns `None` when the
    /// captured node left the arena mid-gesture: the capture releases and
    /// the caller falls through to the ordinary hit test.
    fn route_captured(
        &mut self,
        event: &PointerEvent,
        ran: &mut bool,
    ) -> Option<(EventResponse, Option<NodeId>)> {
        let captured = self.captured?;
        if !self.arena.is_live(captured) {
            self.captured = None;
            tracing::trace!(
                target: "alacer::capture",
                "capture released: the captured node left the arena mid-gesture"
            );
            return None;
        }
        // The ancestor chain root → captured — the same chain shape the
        // hit test's pre-order walk produces for the press's target.
        let mut chain = Vec::new();
        let mut cursor = Some(captured);
        while let Some(id) = cursor {
            chain.push(id);
            cursor = self.arena.parent(id);
        }
        chain.reverse();
        *ran = true;
        // The release ends the capture on arrival: the widget still sees
        // its Up/Leave — the walk below routes it — and the next event
        // hit-tests again.
        if matches!(event, PointerEvent::Up { .. } | PointerEvent::Leave) {
            self.captured = None;
            tracing::trace!(
                target: "alacer::capture",
                node = captured.raw(),
                "capture released with the gesture's end"
            );
        }
        // The gesture trail's stamp (M46, ADR-0052): the captured route
        // knows the control the press holds — the drag's moves and its
        // end ride it wherever the pointer goes — so the facts the drag
        // door queues beneath this event carry the captured node's id,
        // the same stamp the hit-test route held at the press (one
        // gesture, one identity). The idle contract is the hit-test
        // route's own: one `Cell` read while nobody listens, the guard
        // alive only for the length of this event's walks.
        let _stamp = crate::gesture::gesture_trail_enabled()
            .then(|| crate::gesture::GestureTargetGuard::for_target(Some(self.id_hash(captured))));
        // Capture: root → target (§10.1). First `Handled` stops the phase.
        for node in &chain {
            let response = self.visit_with_phase(*node, EventPhase::Capture, event);
            if response == EventResponse::Handled {
                return Some((EventResponse::Handled, Some(captured)));
            }
        }
        // Bubble: target → root. Reverse pre-order tail = deepest, topmost
        // (last painted) hit first.
        for node in chain.iter().rev() {
            let response = self.visit_with_phase(*node, EventPhase::Bubble, event);
            if response == EventResponse::Handled {
                return Some((EventResponse::Handled, Some(captured)));
            }
        }
        Some((EventResponse::Ignored, Some(captured)))
    }

    /// One capture/bubble visit: route the event with the phase set, merge
    /// the widget's requested masks upward. A widget that requests `FOCUS`
    /// is asking for keyboard focus — the click-to-focus seam (§10.3): the
    /// request becomes pass 3's pending focus, resolved same tick.
    ///
    /// The visit also hands the node's resolved frame to the widget
    /// (§10.3, ADR-0018): window coordinates become widget-local math only
    /// against the bounds layout resolved — the caret-from-click seam's
    /// half that lives in the router.
    fn visit_with_phase(
        &mut self,
        node: NodeId,
        phase: EventPhase,
        event: &PointerEvent,
    ) -> EventResponse {
        let bounds = self.arena.bounds(node);
        let (response, requested) = {
            let mut guard = self.arena.mutate(node);
            let mut ctx = crate::events::EventCtx::with_phase_and_bounds(phase, bounds);
            let response = guard.widget_mut().on_pointer_event(&mut ctx, event);
            (response, ctx.take())
        };
        if !requested.is_empty() {
            if requested.contains(Change::FOCUS) {
                self.pending_focus = Some(node);
            }
            self.arena.merge_mask(node, requested);
        }
        response
    }

    /// Pass 1 (`event`, keyboard half): routes a key event to the *focused*
    /// widget only (§10.3 — keyboard input is focus-routed, never hit-tested).
    /// `Tab` / `Shift+Tab` cycle focus when the focused widget ignores them —
    /// the framework-level key semantic every platform expects.
    pub fn dispatch_key(&mut self, event: &KeyEvent) -> EventResponse {
        self.counters.events_routed += 1;
        let mut run = PassRun {
            kind: PassKind::Event,
            ran: false,
            nanos: 0,
            work_nanos: 0,
        };
        let span =
            tracing::trace_span!(target: "alacer::pass", "pass", pass = PassKind::Event.name());
        let start = self.timings.then(|| self.read_clock_nanos());
        let guard = span.enter();

        let mut response = EventResponse::Ignored;
        if let Some(focused) = self.focused
            && self.node_is_live(focused)
        {
            run.ran = true;
            response = self.visit_key(focused, event);
        }

        // The app's shortcuts (M35, ADR-0041): a chord the focused widget
        // did not answer is the keymap's to name — the explicit beats the
        // implicit, so an app that binds `Tab` in a modal means it — and
        // the answer does not need a focused widget at all: window-level
        // vocabulary answers wherever the focus sits. The installed seam
        // runs the binding's action and the key is consumed before the
        // framework's fallbacks (Tab cycling below, the directional ring
        // after) take it. No seam installed answers `false` and the funnel
        // falls through honestly (the M11 rule at the dispatch seam — an
        // optional seam has no panic door).
        //
        // Since M36 (ADR-0042) the consult answers as data and streams:
        // every consult the funnel runs — and only a consult; the
        // focused widget's answer pre-empts the keymap entirely —
        // forwards one [`KeyRecord`] the moment it resolves, before the
        // fallbacks' focus records, so a key event's trail reads keymap
        // → focus → input → frame. The behavior is the M35 funnel
        // byte-for-byte: [`KeymapConsult::fired`] is the
        // `dispatch_installed` verdict that always stood here.
        //
        // Since M37 (ADR-0043) the record names the binding too: the
        // prose name is captured *before* the consult's dispatch — the
        // name the consult resolves to, never what a mid-flight rebind
        // leaves behind — and only when an observer rides (the name
        // leg's read never runs on the unobserved funnel: the A7 idle
        // contract, one branch). One consult channel still: the name
        // leg runs no action, the consult's dispatch stays the only
        // door.
        if response == EventResponse::Ignored {
            let name = if self.observer.is_some() {
                crate::keymap::consult_name(event)
            } else {
                None
            };
            let consult = crate::keymap::consult(event);
            self.notify_key(event, consult, name);
            if consult.fired() {
                response = EventResponse::Handled;
                run.ran = true;
            }
        }

        // Focus cycling: Tab walks the candidate list forward, Shift+Tab
        // backward (§10.3) — through the same [`Self::focus_move`] funnel
        // the consumer's door rides (ADR-0030: one walk, two doors; the
        // outcome is the door's answer as data, M27 — ADR-0033). The
        // widget's `Ignored` lets the framework take the key.
        if response == EventResponse::Ignored && event.presses(&crate::events::Key::Tab) {
            let movement = if event.modifiers.shift {
                FocusMove::Prev
            } else {
                FocusMove::Next
            };
            // The keyboard's ride is named as the keyboard's (M30,
            // ADR-0036): the same funnel, the record attributing the
            // user's navigation to the keyboard's fallback.
            if self.focus_move_via(FocusRider::Keyboard, movement).moved() {
                response = EventResponse::Handled;
            }
            run.ran = true;
        }

        // The directional keys (M29, ADR-0035): a plain arrow the
        // focused widget did not answer — a D-pad button, a TV
        // remote's ring — maps onto its [`FocusMove`] at the same
        // fallback seam and rides the same funnel (ADR-0030's rule; the
        // platform's key map already carries the arrows). The widget's
        // answer always took precedence above — a focused input's
        // caret and a slider's step are the widget's vocabulary. The
        // chords stay the widgets' vocabulary too: a modifier held (a
        // caret's Shift+Arrow, a word hop's Alt+Arrow) is never the
        // window's walk, and a refused direction falls through — the
        // key is nobody's, the M11 rule at the dispatch seam, the M28
        // trail carrying the refusal like every ride.
        if response == EventResponse::Ignored
            && event.state == crate::events::KeyState::Pressed
            && !event.modifiers.shift
            && !event.modifiers.control
            && !event.modifiers.alt
            && !event.modifiers.meta
            && let Some(movement) = match event.key {
                crate::events::Key::ArrowLeft => Some(FocusMove::Left),
                crate::events::Key::ArrowRight => Some(FocusMove::Right),
                crate::events::Key::ArrowUp => Some(FocusMove::Up),
                crate::events::Key::ArrowDown => Some(FocusMove::Down),
                _ => None,
            }
        {
            // The ring's ride is named as the ring's (M30, ADR-0036): a
            // D-pad button or a remote's ring is the user's navigation,
            // distinguishable in the trail from the app's own doors.
            if self.focus_move_via(FocusRider::Ring, movement).moved() {
                response = EventResponse::Handled;
            }
            run.ran = true;
        }

        run.nanos = start.map_or(0, |t| self.read_clock_nanos().saturating_sub(t));
        drop(guard);
        self.interframe.push(run);
        self.notify_input(InputKind::Key, self.focused);
        response
    }

    /// Pass 1 (`event`, IME half): routes an IME event to the focused widget
    /// (§10.3). Preedit is composing input — it never reaches `on_key_event`.
    pub fn dispatch_ime(&mut self, event: &ImeEvent) -> EventResponse {
        self.counters.events_routed += 1;
        let mut run = PassRun {
            kind: PassKind::Event,
            ran: false,
            nanos: 0,
            work_nanos: 0,
        };
        let span =
            tracing::trace_span!(target: "alacer::pass", "pass", pass = PassKind::Event.name());
        let start = self.timings.then(|| self.read_clock_nanos());
        let guard = span.enter();

        let mut response = EventResponse::Ignored;
        if let Some(focused) = self.focused
            && self.node_is_live(focused)
        {
            run.ran = true;
            // The funnel's stamp (M43, ADR-0049; the key route's own
            // discipline since M58, ADR-0064): the IME route holds the
            // guard for the length of one event, a committed
            // composition's fact naming the control it rode — the same
            // vocabulary the key route stamps, the identity never
            // guessed at the note site. While nobody listens the stamp
            // is one `Cell` read and no id-path walk (the trail is off,
            // no fact can queue); the guard dies with the widget's call
            // — a panicking widget leaves no stale identity on the
            // thread.
            let _stamp = crate::value::value_trail_enabled()
                .then(|| crate::value::ActionTargetGuard::for_target(Some(self.id_hash(focused))));
            let (handled, requested) = {
                let mut guard = self.arena.mutate(focused);
                let mut ctx = crate::events::EventCtx::with_phase(EventPhase::Bubble);
                let handled = guard.widget_mut().on_ime_event(&mut ctx, event);
                (handled, ctx.take())
            };
            if !requested.is_empty() {
                if requested.contains(Change::FOCUS) {
                    self.pending_focus = Some(focused);
                }
                self.arena.merge_mask(focused, requested);
            }
            response = handled;
        } else {
            // The boundary's own silence, named (M59, ADR-0065): the
            // IME event no live field held — no field has focus, or
            // the focused node has gone — drops before any widget's
            // door, the consumed `Ignored` verdict the one answer it
            // left. A committed composition vanishing is a fact a
            // consumer prices (ADR-0063's first kill rule): the record
            // is the route's own vocabulary, the event named by its
            // stable tag, the aim riding its honest absence (the
            // boundary had no aim to name — the drop *is* the missing
            // field). Since M65 (ADR-0071) the silence splits — the
            // dead focused node's drop prices apart from the
            // never-focused one (ADR-0065's first kill rule) — and the
            // record names the route that dropped the ask (the M30
            // rider rule at the route tier, ADR-0071).
            let species = if self.focused.is_none() {
                crate::inspect::RouteDropSpecies::NoFocusedField
            } else {
                crate::inspect::RouteDropSpecies::DeadFocus
            };
            crate::route_drop::note_route_drop(crate::inspect::RouteDropRecord {
                action: ime_event_tag(event),
                species,
                rider: crate::inspect::RouteDropRider::ImeRoute,
                target: None,
            });
        }

        run.nanos = start.map_or(0, |t| self.read_clock_nanos().saturating_sub(t));
        drop(guard);
        self.interframe.push(run);
        self.notify_input(InputKind::Ime, self.focused);
        response
    }

    /// Forwards one routed-input record to the observer, when installed
    /// (§13.3: the input trail that ends in "why did this render").
    fn notify_input(&mut self, kind: InputKind, target: Option<NodeId>) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Input(InputRecord { kind, target }));
        }
    }

    /// Forwards one focus-door answer to the observer, when installed
    /// (§13.3, ADR-0034): the walk's landing or its named refusal — the
    /// "why did focus change — or why didn't it" trail. The record is the
    /// door's own outcome verbatim ([`FocusRecord`]), streamed when the
    /// door resolves — before the input record the dispatch forwards and
    /// the frame summary the tick forwards — so a key event's trail reads
    /// focus → input → frame. Since M30 the record names its rider
    /// (ADR-0036): the attribution the caller declared — the app's doors
    /// stream [`FocusRider::Consumer`], the dispatch fallbacks stream the
    /// keyboard and the ring. One branch when no
    /// observer is installed: the A7 idle contract is unchanged.
    fn notify_focus(
        &mut self,
        rider: FocusRider,
        movement: FocusMove,
        scope: Option<NodeId>,
        outcome: FocusMoveOutcome,
    ) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Focus(FocusRecord {
                movement,
                scope,
                rider,
                outcome,
            }));
        }
    }

    /// Forwards one journal fact to the observer, when installed (§13.3,
    /// ADR-0037): the edit trail — the journal's own record of the action
    /// it answered, depths beside it. The record is the journal's own
    /// fact verbatim ([`EditRecord`]), streamed at the tick's start
    /// drain — one frame after the chord, the announcement drain's
    /// currency. One branch when no observer is installed: the A7 idle
    /// contract is unchanged.
    fn notify_edit(&mut self, record: EditRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Edit(record));
        }
    }

    /// Forwards one clipboard fact to the observer, when installed
    /// (§13.3, ADR-0045): the clipboard trail — the round trip the
    /// editing chords answered, the payload's count beside the outcome,
    /// the refusals named. The record is the widget's own fact verbatim
    /// ([`crate::inspect::ClipboardRecord`]), streamed at the tick's
    /// start drain — one frame after the chord, the edit trail's
    /// currency. One branch when no observer is installed: the A7 idle
    /// contract is unchanged.
    fn notify_clipboard(&mut self, record: crate::inspect::ClipboardRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Clipboard(record));
        }
    }

    /// Forwards one cursor fact to the observer, when installed (§13.3,
    /// ADR-0157): the cursor trail — the hover's resolved change, the
    /// style beside the source that named it. The record is the
    /// resolution door's own fact verbatim
    /// ([`crate::inspect::CursorRecord`]), streamed at the tick's start
    /// drain — one frame after the change, the clipboard trail's
    /// currency. One branch when no observer is installed: the A7 idle
    /// contract is unchanged.
    fn notify_cursor(&mut self, record: crate::inspect::CursorRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Cursor(record));
        }
    }

    /// Forwards one value fact to the observer, when installed (§13.3,
    /// ADR-0046): the value trail — the write the AT's value tier asked,
    /// the landed value beside the outcome, the refusals named. The
    /// record is the widget's own fact verbatim
    /// ([`crate::inspect::ValueRecord`]), streamed at the tick's start
    /// drain — one frame after the act, the clipboard trail's currency.
    /// One branch when no observer is installed: the A7 idle contract
    /// is unchanged.
    fn notify_value(&mut self, record: crate::inspect::ValueRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Value(record));
        }
    }

    /// Forwards one gesture fact to the observer, when installed (§13.3,
    /// ADR-0052): the gesture trail — the drag's own story, one record
    /// per phase (the press that armed it, the release or leave that
    /// answered it), never per move (the flood rule). The record is the
    /// widget's own fact verbatim
    /// ([`crate::inspect::GestureRecord`]), streamed at the tick's
    /// start drain — one frame after the phase, the value trail's
    /// currency. One branch when no observer is installed: the A7 idle
    /// contract is unchanged.
    fn notify_gesture(&mut self, record: crate::inspect::GestureRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Gesture(record));
        }
    }

    /// Forwards one scroll fact to the observer, when installed (§13.3,
    /// ADR-0056): the scroll trail — the ask the scroll doors answered,
    /// the landed offset beside the ask, the refusals named. The
    /// record is the widget's own fact verbatim
    /// ([`crate::inspect::ScrollRecord`]), streamed at the tick's
    /// start drain — one frame after the ask, the gesture trail's
    /// currency. One branch when no observer is installed: the A7 idle
    /// contract is unchanged.
    fn notify_scroll(&mut self, record: crate::inspect::ScrollRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Scroll(record));
        }
    }

    /// Forwards one text-write fact to the observer, when installed
    /// (§13.3, ADR-0061): the text-write trail — the write the AT's
    /// editing tier asked, the shape named by lengths beside the
    /// landing selection, the refusals named. The record is the
    /// widget's own fact verbatim
    /// ([`crate::inspect::TextWriteRecord`]), streamed at the tick's
    /// start drain — one frame after the write, the scroll trail's
    /// currency. One branch when no observer is installed: the A7 idle
    /// contract is unchanged.
    fn notify_text_write(&mut self, record: crate::inspect::TextWriteRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::TextWrite(record));
        }
    }

    /// Forwards one route-drop fact to the observer, when installed
    /// (§13.3, ADR-0063): the route-drop trail — the ask the boundary
    /// dropped before any widget's door, the species and the aimed id
    /// beside it. The record is the route's own fact verbatim
    /// ([`crate::inspect::RouteDropRecord`]), streamed at the tick's
    /// start drain — one frame after the drop, the text-write trail's
    /// currency. One branch when no observer is installed: the A7 idle
    /// contract is unchanged.
    fn notify_route_drop(&mut self, record: crate::inspect::RouteDropRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::RouteDrop(record));
        }
    }

    /// Forwards one typing-door edit to the observer, when installed
    /// (§13.3, ADR-0064): the key-edit trail's fact — the shape the
    /// keyboard's keys or the IME's commits landed, named by lengths,
    /// the landing selection beside it, a coalesced typed run riding as
    /// one record (the flood rule). One branch when no observer is
    /// installed: the A7 idle contract is unchanged.
    fn notify_key_edit(&mut self, record: crate::inspect::KeyEditRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::KeyEdit(record));
        }
    }

    /// Forwards one composing-state fact to the observer, when
    /// installed (§13.3, ADR-0067): the preedit trail's fact — the
    /// composition's inline shape named by lengths beside the
    /// platform's cursor, one live session riding as one record (the
    /// flood rule at the composing tier). One branch when no observer
    /// is installed: the A7 idle contract is unchanged.
    fn notify_preedit(&mut self, record: crate::inspect::PreeditRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Preedit(record));
        }
    }

    /// Forwards one adapter-echo fact to the observer, when installed
    /// (§13.3, ADR-0073): the a11y trail — the full-tree echo the a11y
    /// tier answered a connect with, the tree named by its node count.
    /// The record is the sink's own fact verbatim
    /// ([`crate::inspect::A11yEchoRecord`]), streamed at the tick's
    /// start drain ahead of the frame's routes — a connect predates the
    /// asks. One branch when no observer is installed: the A7 idle
    /// contract is unchanged.
    fn notify_a11y_echo(&mut self, record: crate::inspect::A11yEchoRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::A11yEcho(record));
        }
    }

    /// Forwards one activation-ask fact to the observer, when installed
    /// (§13.3, ADR-0075): the action trail — the press the AT's action
    /// route aimed at a control, the landing or the named refusal
    /// species beside the aim ([`crate::inspect::ActionRecord`]),
    /// streamed at the tick's start drain, one frame after the ask. One
    /// branch when no observer is installed: the A7 idle contract is
    /// unchanged.
    fn notify_action(&mut self, record: crate::inspect::ActionRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Action(record));
        }
    }

    /// Forwards one frame-refusal fact to the observer, when installed
    /// (§13.3, ADR-0077): the frame tier's trail — the refusal the
    /// tick's layout stage or the present ladder's stage answered with,
    /// the species beside the failing node when one is the fact
    /// ([`crate::inspect::FrameRefusalRecord`]), streamed at the tick's
    /// start drain, one frame after the frame that refused. One branch
    /// when no observer is installed: the A7 idle contract is
    /// unchanged.
    fn notify_frame_refusal(&mut self, record: crate::inspect::FrameRefusalRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::FrameRefusal(record));
        }
    }

    /// Forwards one view-patch fact to the observer, when installed
    /// (§13.3, ADR-0078): the view tier's trail — the reconciler's
    /// four counts beside each other, the mount's build and every
    /// re-render's diff named by its lengths
    /// ([`crate::inspect::ViewPatchRecord`]), streamed at the tick's
    /// start drain, one frame after the re-render that asked for it.
    /// One branch when no observer is installed: the A7 idle contract
    /// is unchanged.
    fn notify_view_patch(&mut self, record: crate::inspect::ViewPatchRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::ViewPatch(record));
        }
    }

    /// Forwards one theme-swap fact to the observer, when installed
    /// (§7.2 pass 8, ADR-0079): the theme trail — the palette's tag
    /// beside the door that landed it, one fact per swap, the no-op
    /// swap streaming none. One branch when no observer is installed:
    /// the A7 idle contract is unchanged.
    fn notify_theme(&mut self, record: crate::inspect::ThemeRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Theme(record));
        }
    }

    /// Forwards one setting-flip fact to the observer, when installed
    /// (§7.2 pass 8, ADR-0080): the setting trail — the dark-mode fact
    /// beside the door that landed it, one fact per flip, the no-op
    /// flip streaming none. Streams ahead of the theme trail's facts:
    /// the flip is the fact the palette resolves beside, the fact
    /// explaining the repaint the swap's facts answer into. One branch
    /// when no observer is installed: the A7 idle contract is
    /// unchanged.
    fn notify_setting(&mut self, record: crate::inspect::SettingRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Setting(record));
        }
    }

    /// Forwards one speech-act fact to the observer, when installed
    /// (§13.3, ADR-0039): the announcement trail — the record the
    /// pipeline answered the act with, the politeness beside it. The
    /// direct door streams where the answer resolves, the push-side
    /// refusal rides the thread trail to the tick's start drain. One
    /// branch when no observer is installed: the A7 idle
    /// contract is unchanged.
    fn notify_announce(&mut self, record: crate::inspect::AnnounceRecord) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Announce(record));
        }
    }

    /// Forwards one keymap consult to the observer, when installed
    /// (§13.3, ADR-0042): the shortcut trail — the chord the funnel
    /// asked the app's shortcuts about beside the species the consult
    /// answered ([`KeymapConsult`]), streamed the moment the consult
    /// resolves. Since M37 (ADR-0043) the record names the binding too:
    /// the prose name the seam answered beside the consult, carried
    /// only on a fired consult (a miss and an uninstalled window name
    /// no binding). The consult is the funnel's own route: the focused
    /// widget's answer pre-empts it entirely (the M35 precedence), so a
    /// pre-empted key streams the input record and no key record — the
    /// absent record *is* the precedence, visible in the trail. One
    /// branch when no observer is installed: the A7 idle contract is
    /// unchanged.
    fn notify_key(&mut self, event: &KeyEvent, consult: KeymapConsult, name: Option<String>) {
        if let Some(observer) = self.observer.as_mut() {
            observer.observe(InspectEvent::Key(KeyRecord {
                chord: Chord::with_modifiers(event.key, event.modifiers),
                outcome: consult,
                name: if consult.fired() { name } else { None },
            }));
        }
    }

    /// The one funnel, ridden with a name (M30, ADR-0036): the unscoped
    /// walk the doors and the dispatch fallbacks share, streaming the
    /// record under the rider the route declares — the app's doors name
    /// [`FocusRider::Consumer`], the keyboard's Tab fallback names
    /// [`FocusRider::Keyboard`], the ring's arrows name
    /// [`FocusRider::Ring`]. The walk behind the doors is untouched (the
    /// same candidate list, the same availability filter, the same wrap,
    /// the same immediate resolution); only the record's attribution
    /// moves with the route, so a trail reader answers "who moved focus —
    /// or why didn't it" without re-implementing the observation.
    fn focus_move_via(&mut self, rider: FocusRider, movement: FocusMove) -> FocusMoveOutcome {
        let span = tracing::trace_span!(
            target: "alacer::focus",
            "focus_move",
            direction = movement.name(),
            rider = rider.name()
        );
        let _guard = span.enter();
        let outcome = self.cycle_focus(movement);
        self.notify_focus(rider, movement, None, outcome);
        if let Some(landing) = outcome.landing() {
            tracing::trace!(
                target: "alacer::focus",
                node = landing.to.raw(),
                "the walk rests on the next candidate"
            );
        } else {
            tracing::trace!(
                target: "alacer::focus",
                "refused: the walk has no candidate to offer — no masks, an idle frame"
            );
        }
        outcome
    }

    /// Builds and forwards the frame's causal summary (§13.3): the drained
    /// edge activations and provenance writes, then the frame itself with
    /// the mask heatmap resolved to roles/labels/bounds. No-op without an
    /// observer.
    fn notify_frame(
        &mut self,
        report: &FrameReport,
        dirty: &FxHashMap<NodeId, Change>,
        viewport: Size,
    ) {
        let Some(observer) = self.observer.as_mut() else {
            return;
        };
        for write in self.flux.take_signal_writes() {
            observer.observe(InspectEvent::Write(write));
        }
        for activation in self.flux.take_edge_activations() {
            observer.observe(InspectEvent::Edge(activation));
        }
        let mut dirty_nodes: Vec<DirtyNode> =
            Vec::with_capacity(dirty.len().min(DIRTY_HEATMAP_CAP));
        let mut truncated = 0_usize;
        for (node, mask) in dirty {
            if dirty_nodes.len() >= DIRTY_HEATMAP_CAP {
                truncated += 1;
                continue;
            }
            let (role, label, bounds) = if self.arena.is_live(*node) {
                let widget = self.arena.widget(*node);
                (
                    widget.role(),
                    widget.label().resolve(),
                    self.arena.bounds(*node),
                )
            } else {
                (Role::Decoration, String::new(), None)
            };
            dirty_nodes.push(DirtyNode {
                node: *node,
                mask: *mask,
                role,
                label,
                bounds,
            });
        }
        observer.observe(InspectEvent::Frame(FrameSummary {
            number: self.frames,
            layout_ran: report.layout_ran,
            paint_ran: report.paint_ran,
            a11y_ran: report.a11y_ran,
            focus_changed: report.focus_changed,
            degradation_step: report.degradation_step,
            damage: report.damage.clone(),
            viewport,
            a11y_updated: report.a11y_delta.updated.len(),
            a11y_removed: report.a11y_delta.removed.len(),
            total_nanos: report.total_nanos,
            passes: report
                .passes
                .iter()
                .map(|run| PassTiming {
                    kind: run.kind,
                    ran: run.ran,
                    nanos: run.nanos,
                })
                .collect(),
            dirty: dirty_nodes,
            dirty_truncated: truncated,
        }));
    }

    /// The input-kind half of a pointer event (the inspector's vocabulary).
    fn pointer_kind(event: &PointerEvent) -> InputKind {
        match event {
            PointerEvent::Down { .. } => InputKind::PointerDown,
            PointerEvent::Up { .. } => InputKind::PointerUp,
            PointerEvent::Move { .. } => InputKind::PointerMove,
            PointerEvent::Wheel { .. } => InputKind::Wheel,
            PointerEvent::Leave => InputKind::PointerLeave,
        }
    }

    /// One focus-routed key visit: route with the phase set, merge masks.
    /// Since M45 (ADR-0051, ADR-0049's first kill rule cashed) the visit
    /// holds the value trail's stamp: a value-tier widget's key door
    /// notes its ride on the log, and the funnel knows the control the
    /// key rode — the focused node's AccessKit-stable id, the same
    /// vocabulary the action route stamps (ADR-0049).
    fn visit_key(&mut self, node: NodeId, event: &KeyEvent) -> EventResponse {
        // The idle contract holds: while nobody listens the stamp is
        // one `Cell` read and no id-path walk (the trail is off, no
        // fact can queue); under an observer the guard lives for the
        // length of the focused widget's key call and dies with it — a
        // panicking widget leaves no stale identity on the thread (the
        // action route's own discipline, ADR-0049).
        let _stamp = crate::value::value_trail_enabled()
            .then(|| crate::value::ActionTargetGuard::for_target(Some(self.id_hash(node))));
        let (response, requested) = {
            let mut guard = self.arena.mutate(node);
            let mut ctx = crate::events::EventCtx::with_phase(EventPhase::Bubble);
            let response = guard.widget_mut().on_key_event(&mut ctx, event);
            (response, ctx.take())
        };
        if !requested.is_empty() {
            if requested.contains(Change::FOCUS) {
                self.pending_focus = Some(node);
            }
            self.arena.merge_mask(node, requested);
        }
        response
    }

    /// Moves focus one candidate in `movement`'s direction — the one walk
    /// every focus-move door shares (the [`Self::focus_move` funnel]
    /// (`Self::focus_move`), ADR-0030/ADR-0032). The tree-order moves wrap
    /// at both ends; the directional moves answer from the laid-out bounds
    /// and never wrap. The outcome carries the landing — endpoints and
    /// wrap — or the named refusal species (M27, ADR-0033).
    fn cycle_focus(&mut self, movement: FocusMove) -> FocusMoveOutcome {
        self.cycle_focus_in(None, movement)
    }

    /// The walk behind both doors, bounded to `scope` when one is named
    /// (M25, ADR-0031; M26, ADR-0032): the same candidate list pass 2
    /// recomputes, the same cycle-time availability filter, the same
    /// immediate-resolution rules — the scope only bounds it, the
    /// direction only aims it. `None` is the unscoped walk Tab and
    /// [`Self::focus_move`] have ridden since M3; `Some(root)` walks the
    /// subtree rooted at `root`. The tree-order moves follow the list's
    /// positions and wrap at both ends; the directional moves read the
    /// bounds pass 6 wrote ([`Self::directional_neighbor`]) and never
    /// wrap. From no focus — or from a focus outside the filtered
    /// candidate list (a scope bounding it, or the tier holding the
    /// focused control down since pass 2) — the walk enters at the list's
    /// first candidate, both directions: the entry contract ADR-0031 has
    /// documented since M25 ("the walk takes the trap's first candidate"),
    /// whose M25-era positional arithmetic actually landed the list's
    /// second candidate — the drift the M26 cut repairs. Since M27 the
    /// walk answers the outcome vocabulary (ADR-0033): the landing's
    /// endpoints and wrap, or the refusal species — the entry is never a
    /// wrap, the stall names itself.
    fn cycle_focus_in(&mut self, scope: Option<NodeId>, movement: FocusMove) -> FocusMoveOutcome {
        // The candidates the tier still answers for (M22, ADR-0028): the
        // cache may name a control that became unavailable since pass 2
        // recomputed it — the cycle-time check keeps Tab honest without
        // paying a recompute on every availability write.
        let candidates: Vec<NodeId> = self
            .focusables
            .iter()
            .copied()
            .filter(|id| self.arena.widget(*id).enabled())
            .filter(|id| scope.is_none_or(|root| self.is_in_scope(*id, root)))
            .collect();
        if candidates.is_empty() {
            return FocusMoveOutcome::Refused(FocusRefusal::NoCandidates);
        }
        let mut wrapped = false;
        let target = if movement.is_directional() {
            match self.focused {
                // The entry contract is direction-agnostic: the walk must
                // rest somewhere before a direction can mean anything.
                None => candidates[0],
                Some(current) => {
                    if self.arena.bounds(current).is_none() {
                        tracing::trace!(
                            target: "alacer::focus",
                            node = current.raw(),
                            "refused: the focused node carries no geometry yet — help: the \
                             directional walk reads the bounds pass 6 (layout) wrote; tick one \
                             frame first, or ride the tree-order doors focus_move Next/Prev"
                        );
                        return FocusMoveOutcome::Refused(FocusRefusal::NoGeometry {
                            node: current,
                        });
                    }
                    let Some(neighbor) = self.directional_neighbor(&candidates, current, movement)
                    else {
                        tracing::trace!(
                            target: "alacer::focus",
                            node = current.raw(),
                            "refused: no candidate lies in the requested direction — help: \
                             the directional walk selects what the geometry names, fully \
                             beyond the focused node's edge, and never wraps (space has no \
                             wrap); the tree-order doors are focus_move Next/Prev"
                        );
                        return FocusMoveOutcome::Refused(FocusRefusal::NothingInDirection {
                            node: current,
                            direction: movement,
                        });
                    };
                    neighbor
                }
            }
        } else {
            match self.focused {
                None => candidates[0],
                Some(current) => {
                    match candidates
                        .iter()
                        .position(|candidate| *candidate == current)
                    {
                        Some(position) => {
                            // The wrap is the tree-order contract and only
                            // the tree-order contract: forward past the
                            // last candidate, backward past the first —
                            // and the crossing is the landing's data now.
                            let (next, crossed) = if movement == FocusMove::Prev {
                                if position == 0 {
                                    (candidates.len() - 1, true)
                                } else {
                                    (position - 1, false)
                                }
                            } else if position + 1 == candidates.len() {
                                (0, true)
                            } else {
                                (position + 1, false)
                            };
                            wrapped = crossed;
                            candidates[next]
                        }
                        // Outside the filtered list: the walk enters at the
                        // list's first candidate, both directions — the
                        // documented entry contract (ADR-0031), never an
                        // approach from the list's far end.
                        None => candidates[0],
                    }
                }
            }
        };
        if Some(target) == self.focused {
            return FocusMoveOutcome::Refused(FocusRefusal::AlreadyFocused { node: target });
        }
        let from = self.focused;
        self.pending_focus = Some(target);
        // Resolve immediately: key routing must see the new focus this event.
        self.update_focus();
        FocusMoveOutcome::Moved(FocusLanding {
            from,
            to: target,
            wrapped,
        })
    }

    /// The spatial neighbor a directional move names (M26, ADR-0032): the
    /// qualifying candidate nearest the focused node's bounds along the
    /// movement's axis. A candidate qualifies when its bounds lie fully
    /// beyond the origin's edge in the direction — no overlap with the
    /// origin on that axis — and the ranking is the edge distance first,
    /// the cross-axis center misalignment breaking ties, then tree order
    /// (the pre-order list the walk already follows): deterministic, and
    /// the same list Tab reads. Candidates without bounds cannot answer a
    /// directional call (they carry no geometry yet — the first frame's
    /// pre-layout state) and are skipped; the focused node itself is
    /// never its own neighbor. `None` is the honest refusal: nothing lies
    /// in the direction, and space does not wrap.
    fn directional_neighbor(
        &self,
        candidates: &[NodeId],
        current: NodeId,
        movement: FocusMove,
    ) -> Option<NodeId> {
        let origin = self.arena.bounds(current)?;
        let origin_center_x = origin.origin.x + origin.size.width * 0.5;
        let origin_center_y = origin.origin.y + origin.size.height * 0.5;
        let mut best: Option<(f32, f32, usize, NodeId)> = None;
        for (order, &candidate) in candidates.iter().enumerate() {
            if candidate == current {
                continue;
            }
            let Some(bounds) = self.arena.bounds(candidate) else {
                continue;
            };
            let center_x = bounds.origin.x + bounds.size.width * 0.5;
            let center_y = bounds.origin.y + bounds.size.height * 0.5;
            let (edge, cross) = match movement {
                FocusMove::Right if bounds.min_x() >= origin.max_x() => (
                    bounds.min_x() - origin.max_x(),
                    (center_y - origin_center_y).abs(),
                ),
                FocusMove::Left if bounds.max_x() <= origin.min_x() => (
                    origin.min_x() - bounds.max_x(),
                    (center_y - origin_center_y).abs(),
                ),
                FocusMove::Down if bounds.min_y() >= origin.max_y() => (
                    bounds.min_y() - origin.max_y(),
                    (center_x - origin_center_x).abs(),
                ),
                FocusMove::Up if bounds.max_y() <= origin.min_y() => (
                    origin.min_y() - bounds.max_y(),
                    (center_x - origin_center_x).abs(),
                ),
                _ => continue,
            };
            let better = match best {
                None => true,
                Some((best_edge, best_cross, best_order, _)) => {
                    edge.total_cmp(&best_edge)
                        .then_with(|| cross.total_cmp(&best_cross))
                        .then_with(|| order.cmp(&best_order))
                        == std::cmp::Ordering::Less
                }
            };
            if better {
                best = Some((edge, cross, order, candidate));
            }
        }
        best.map(|(_, _, _, node)| node)
    }

    /// `true` when `node` lies inside the subtree rooted at `scope` — the
    /// ancestor walk the scoped door's candidate filter rides (M25,
    /// ADR-0031). The scope root itself qualifies.
    fn is_in_scope(&self, node: NodeId, scope: NodeId) -> bool {
        let mut up = Some(node);
        while let Some(current) = up {
            if current == scope {
                return true;
            }
            up = self.arena.parent(current);
        }
        false
    }

    /// Runs exactly one frame through the pass contract (§7.2): `mutate` →
    /// `update_focusable` → `update_focus` → `style` → `layout` → `compose` →
    /// `paint` → `accessibility`. When no mask is pending every pass is
    /// skipped — idle
    /// cost zero (A7). Every pass is traced (`alacer::pass`) and, with
    /// timings on, measured for the M1 overhead gate.
    ///
    /// # Panics
    ///
    /// When a layout-forcing mask exists but no root is mounted — mounting
    /// before ticking is the arena's own contract, so this is a caller bug,
    /// not a runtime condition.
    ///
    /// # Errors
    ///
    /// Propagates [`LayoutError`] from the layout pass.
    #[expect(
        clippy::too_many_lines,
        reason = "the ten-pass tick is one readable frame-by-frame narrative; splitting it would hide the pass ordering the contract is about"
    )]
    pub fn tick(&mut self, viewport: Size) -> Result<FrameReport, LayoutError> {
        let total_start = self.timings.then(|| self.read_clock_nanos());
        // The degradation ladder's start read (§14.4): the frame's own
        // elapsed time, read at every decision point against the
        // installed budget. One read when armed, none when disarmed —
        // the default pipeline prices the ladder at nothing.
        let ladder_budget = self.frame_budget;
        let ladder_start = ladder_budget.map(|_| self.read_clock_nanos());
        let mut degradation = DegradationStep::None;

        // The a11y-echo drain (§13.3, ADR-0073): the adapter tier's facts
        // — the full-tree echo a connect or reconnect answered — ride the
        // observer at the tick's start, ahead of the frame's routes: a
        // connect predates the asks the frame carries, and the echo
        // explains the tree those asks answer into (the echo reads
        // first, the routes beside it). Every fact streams (a bounded
        // queue, never collapsed), and the drain empties the queue
        // whether an observer listens or not: a fact streams once, an
        // absent reader just never sees it.
        for record in crate::a11y_echo::take_a11y_echo_trail() {
            self.notify_a11y_echo(record);
        }

        // Frame-batched input (§10.2, ADR-0014): the queued storm coalesces
        // and routes here, before the ledger is taken — its event passes are
        // this frame's, exactly as if the platform had dispatched one event.
        // The degradation ladder's first rung rides the drain (§14.4): a
        // frame behind its input share routes the batch's head and keeps the
        // remainder queued — the next frame's drain routes it (coalesce the
        // flux work into the next frame). Never a lost event: a postponed
        // one, and the rung is recorded where the decision resolves.
        match ladder_budget {
            Some(budget) => {
                let drain_start = self.read_clock_nanos();
                let input_share =
                    crate::num::u128_to_u64((budget / DEGRADATION_INPUT_SHARE).as_nanos());
                loop {
                    let (batch, remaining) = self.input.drain_batch(DEGRADATION_INPUT_BATCH);
                    for event in batch {
                        self.dispatch_pointer(&event);
                    }
                    if remaining == 0 {
                        break;
                    }
                    let elapsed = self.read_clock_nanos().saturating_sub(drain_start);
                    if elapsed > input_share {
                        degradation = DegradationStep::DropInput;
                        break;
                    }
                }
            }
            None => {
                for event in self.input.drain() {
                    self.dispatch_pointer(&event);
                }
            }
        }

        // The announcement drain (§11.6, ADR-0024): the speech acts event
        // handlers queued on this thread land in the reserved region at the
        // tick's start — the input queue's shape (push now, the tick
        // routes, one frame). The whole batch drains and last write wins:
        // the replaced acts stream their `Superseded` facts (ADR-0039 —
        // the replace rule visible as data, not a silent drop), and the
        // survivor's answer streams too — a region with no root refuses
        // here, honestly, with nothing shipped.
        let batch = crate::announce::take_pending();
        let survivor = batch.len().saturating_sub(1);
        for (index, pending) in batch.into_iter().enumerate() {
            if index < survivor {
                tracing::trace!(
                    target: "alacer::announce",
                    politeness = pending.politeness.name(),
                    "replaced at the drain: last write wins, the region speaks once"
                );
                self.notify_announce(crate::inspect::AnnounceRecord {
                    politeness: pending.politeness,
                    outcome: crate::inspect::AnnounceOutcome::Superseded,
                });
                continue;
            }
            if self.arena.root().is_some() {
                let politeness = pending.politeness;
                self.announcement = Some(pending);
                self.announcement_dirty = true;
                self.notify_announce(crate::inspect::AnnounceRecord {
                    politeness,
                    outcome: crate::inspect::AnnounceOutcome::Delivered,
                });
            } else {
                tracing::trace!(
                    target: "alacer::announce",
                    "dropped at the drain: no root is mounted, the region has no tree"
                );
                self.notify_announce(crate::inspect::AnnounceRecord {
                    politeness: pending.politeness,
                    outcome: crate::inspect::AnnounceOutcome::Refused(
                        crate::inspect::AnnounceRefusal::NoRoot,
                    ),
                });
            }
        }

        // The announcement trail's drain (§13.3, ADR-0039): the speech-act
        // facts the push route queued on this thread (the empty refusal
        // answered before anything queued) ride the observer at the tick's
        // start — the edit trail's shape, and the drain empties the queue
        // whether an observer listens or not: a fact streams once.
        for record in crate::announce::take_announce_trail() {
            self.notify_announce(record);
        }

        // The clipboard-trail drain (§13.3, ADR-0045): the round-trip
        // facts the editing chords queued on this thread ride the
        // observer at the tick's start — the edit trail's shape, one
        // frame after the chord, the drain running ahead of the edit
        // drain so a cut's trail reads clipboard → edit (the causal
        // order the round trip ran: what moved out, then the deletion
        // it drove). Every fact streams (a bounded queue, never
        // collapsed), and the drain empties the queue whether an
        // observer listens or not: a fact streams once, an absent
        // reader just never sees it.
        for record in crate::clipboard::take_clipboard_trail() {
            self.notify_clipboard(record);
        }

        // The cursor-trail drain (§13.3, ADR-0157): the hover facts the
        // resolution door queued on this thread ride the observer at the
        // tick's start — the clipboard drain's shape, one frame after
        // the change. Every fact streams (a bounded queue, never
        // collapsed), and the drain empties the queue whether an
        // observer listens or not: a fact streams once, an absent
        // reader just never sees it.
        for record in crate::cursor::take_cursor_trail() {
            self.notify_cursor(record);
        }

        // The edit-trail drain (§13.3, ADR-0037): the journal facts the
        // edit funnels queued on this thread ride the observer at the
        // tick's start — the announcement drain's shape, one frame after
        // the chord. Every fact streams (a bounded queue, never
        // collapsed — the journal's own history), and the drain empties
        // the queue whether an observer listens or not: a fact streams
        // once, an absent reader just never sees it.
        for record in crate::history::take_edit_trail() {
            self.notify_edit(record);
        }

        // The value-trail drain (§13.3, ADR-0046): the value-tier facts
        // the AT's writes queued on this thread ride the observer at
        // the tick's start — the clipboard trail's shape, one frame
        // after the act, the drain running after the edit drain (a
        // value write explains itself; it deletes nothing). Every fact
        // streams (a bounded queue, never collapsed), and the drain
        // empties the queue whether an observer listens or not: a fact
        // streams once, an absent reader just never sees it.
        for record in crate::value::take_value_trail() {
            self.notify_value(record);
        }

        // The gesture-trail drain (§13.3, ADR-0052): the drag's facts
        // the gesture doors queued on this thread ride the observer at
        // the tick's start — the value trail's shape, one frame after
        // the phase, the drain running after the value drain (a gesture
        // explains the value the pointer moved; it streams beside it).
        // Every fact streams (a bounded queue, never collapsed — the
        // phases are the whole per-gesture story), and the drain empties
        // the queue whether an observer listens or not: a fact streams
        // once, an absent reader just never sees it.
        for record in crate::gesture::take_gesture_trail() {
            self.notify_gesture(record);
        }

        // The scroll-trail drain (§13.3, ADR-0056): the scroll facts
        // the scroll doors queued on this thread ride the observer at
        // the tick's start — the gesture trail's shape, one frame
        // after the ask, the drain running after the gesture drain (a
        // scroll explains the viewport the wheel moved; it streams
        // beside the gestures). Every fact streams (a bounded queue,
        // never collapsed), and the drain empties the queue whether an
        // observer listens or not: a fact streams once, an absent
        // reader just never sees it.
        for record in crate::scroll::take_scroll_trail() {
            self.notify_scroll(record);
        }

        // The text-write drain (§13.3, ADR-0061): the text-write facts
        // the input's AT door queued on this thread ride the observer at
        // the tick's start — the scroll trail's shape, one frame after
        // the write, the drain running after the scroll drain (a text
        // write explains the field the reader rewrote; it streams beside
        // the scrolls). Every fact streams (a bounded queue, never
        // collapsed), and the drain empties the queue whether an
        // observer listens or not: a fact streams once, an absent
        // reader just never sees it.
        for record in crate::text_write::take_text_write_trail() {
            self.notify_text_write(record);
        }

        // The route-drop drain (§13.3, ADR-0063): the boundary's drops
        // — the unknown action, the payloadless write, the off-target
        // selection, the unknown target — ride the observer at the
        // tick's start, the text-write trail's shape, one frame after
        // the drop, the drain running after the text-write drain (a
        // drop explains the ask the boundary refused; it streams beside
        // the doors' facts). Every fact streams (a bounded queue, never
        // collapsed), and the drain empties the queue whether an
        // observer listens or not: a fact streams once, an absent
        // reader just never sees it.
        for record in crate::route_drop::take_route_drop_trail() {
            self.notify_route_drop(record);
        }

        // The key-edit drain (§13.3, ADR-0064): the typing door's facts
        // the keyboard's keys and the IME's commits queued on this thread
        // ride the observer at the tick's start — the route-drop trail's
        // shape, one frame after the edit, the drain running after the
        // route-drop drain (a key edit explains the field the typist
        // changed; it streams beside the doors' facts and the boundary's).
        // Every fact streams (a bounded queue, never collapsed — the
        // journal's own fold bounds a typed run to one record), and the
        // drain empties the queue whether an observer listens or not: a
        // fact streams once, an absent reader just never sees it.
        for record in crate::key_edit::take_key_edit_trail() {
            self.notify_key_edit(record);
        }

        // The preedit drain (§13.3, ADR-0067): the composing-state facts
        // the IME route queued on this thread ride the observer at the
        // tick's start — the key-edit trail's shape, one frame after the
        // preedit, the drain running after the key-edit drain (a
        // composition's shape explains the field the IME is composing
        // into; it streams beside the commit's own key-edit fact, the
        // trails tellable apart). Every fact streams (a bounded queue,
        // never collapsed — the session's own supersede bounds a live
        // composition to one record), and the drain empties the queue
        // whether an observer listens or not: a fact streams once, an
        // absent reader just never sees it.
        for record in crate::preedit::take_preedit_trail() {
            self.notify_preedit(record);
        }

        // The action-trail drain (§13.3, ADR-0075): the activation-ask
        // facts the dispatch funnel queued on this thread ride the
        // observer at the tick's start — the preedit trail's shape, one
        // frame after the ask, the drain running after the preedit drain
        // (a press explains the control the AT aimed at; it streams
        // beside the doors' facts). Every fact streams (a bounded queue,
        // never collapsed), and the drain empties the queue whether an
        // observer listens or not: a fact streams once, an absent reader
        // just never sees it.
        for record in crate::action::take_action_trail() {
            self.notify_action(record);
        }

        // The frame-refusal drain (§13.3, ADR-0077): the facts the
        // tick's layout stage and the present ladder's stages queued on
        // this thread ride the observer at the tick's start — one frame
        // after the frame that refused (a fact queues where the
        // refusal resolves, the record beside the door that still
        // answers `Err`, never instead of it). Every fact streams (a
        // bounded queue, never collapsed), and the drain empties the
        // queue whether an observer listens or not: a fact streams
        // once, an absent reader just never sees it.
        for record in crate::frame_refusal::take_frame_refusal_trail() {
            self.notify_frame_refusal(record);
        }

        // The view-patch drain (§13.3, ADR-0078): the facts the view
        // reconciliation queued on this thread — the mount's build, the
        // re-render's diff — ride the observer at the tick's start, one
        // frame after the re-render that asked for the patch (a fact
        // queues where the patch resolves, the record beside the
        // `PatchReport` the door still answers, never instead of it).
        // Every fact streams (a bounded queue, never collapsed), and the
        // drain empties the queue whether an observer listens or not: a
        // fact streams once, an absent reader just never sees it.
        for record in crate::view_patch::take_view_patch_trail() {
            self.notify_view_patch(record);
        }

        // The setting trail's drain (§7.2 pass 8, ADR-0080): the flip
        // facts the set_settings door queued on this thread ride the
        // observer at the tick's start — the theme drain's shape, one
        // frame after the door answered, the drain running just ahead
        // of the theme trail's so a frame's trail reads the fact the
        // palette resolved beside → the palette that fact painted
        // into. Every fact streams (a bounded queue, never collapsed),
        // and the drain empties the queue whether an observer listens
        // or not: a fact streams once, an absent reader just never
        // sees it.
        for record in crate::settings::take_setting_trail() {
            self.notify_setting(record);
        }

        // The theme trail's drain (§7.2 pass 8, ADR-0079): the swap
        // facts the set_theme door queued on this thread ride the
        // observer at the tick's start — the view-patch drain's shape,
        // one frame after the door answered, the drain running last so
        // a frame's trail reads routes → tiers → the palette those
        // routes painted into. Every fact streams (a bounded queue,
        // never collapsed), and the drain empties the queue whether an
        // observer listens or not: a fact streams once, an absent
        // reader just never sees it.
        for record in crate::theme::take_theme_trail() {
            self.notify_theme(record);
        }

        let mut ledger: Vec<PassRun> = std::mem::take(&mut self.interframe);

        // Flux drain: the mask bridge lands masks in the arena (A1). Bookkeeping
        // ahead of pass 4 — the bridge is the scheduling contract's seam.
        for (node, change) in self.flux.take_pending_masks() {
            self.arena.merge_mask(node, change);
        }

        // Drain the arena's masks into the frame's dirty map.
        let mut dirty: FxHashMap<NodeId, Change> =
            self.arena.take_masks(Change::ALL).into_iter().collect();
        let root = self.arena.root();
        let first_frame = root.is_some_and(|r| self.arena.bounds(r).is_none());
        let structure_changed = dirty.values().any(|m| m.contains(Change::STRUCTURE));

        // Pass 4 — mutate: drain queued widget mutations; flags merge on drop.
        let mut mutated: Vec<NodeId> = Vec::new();
        ledger.push(self.run_pass(PassKind::Mutate, |s| {
            mutated = s.arena.drain_mutations();
            !mutated.is_empty()
        }));

        // Pass 2 — update_focusable: recompute candidates after tree changes.
        let recompute_focusables = first_frame || structure_changed;
        ledger.push(self.run_pass(PassKind::UpdateFocusable, |s| {
            if recompute_focusables {
                s.recompute_focusables();
            }
            recompute_focusables
        }));

        // Pass 3 — update_focus: resolve the pending request, drop focus held
        // by dead widgets, emit FOCUS diffs on moves, and notify the old and
        // new focus holders (focus-ring repaint is the widget's own PAINT
        // request; §7.2 pass 3 + §10.3).
        let mut focus_changed = false;
        ledger.push(self.run_pass(PassKind::UpdateFocus, |s| {
            focus_changed = s.update_focus();
            focus_changed || s.pending_focus.is_some() || recompute_focusables
        }));
        // Pass 3's merges (FOCUS|PAINT on both focus endpoints) land mid-tick;
        // drain everything pending into this frame's dirty map now — no
        // one-frame focus-ring lag, no lingering masks (the currency coalesces
        // per frame, A1).
        for (node, change) in self.arena.take_masks(Change::ALL) {
            *dirty.entry(node).or_insert(Change::empty()) |= change;
        }

        // Pass 5 — style: diff styles against the cache; diffs invalidate.
        // Candidates: every node the frame already considers dirty, plus the
        // nodes pass 4 just mutated (a style change without an explicit mask
        // is only reachable through a mutation).
        let style_candidates = !dirty.is_empty() || !mutated.is_empty();
        let mut style_invalidated: Vec<(NodeId, Change)> = Vec::new();
        ledger.push(self.run_pass(PassKind::Style, |s| {
            style_invalidated = s.style_pass(&dirty, &mutated);
            style_candidates
        }));
        // Style diffs join this frame's dirty set immediately — no one-frame
        // delay, no lingering masks.
        for (node, change) in &style_invalidated {
            dirty.entry(*node).or_insert(*change);
        }

        // Pass 6 — layout (incremental; the persistent Taffy tree is never
        // cleared between frames, §8.1).
        let layout_ran = first_frame || dirty.values().any(|m| m.forces_layout());
        let mut layout_work = 0_u64;
        if layout_ran {
            // The honest refusal (ADR-0023's error shape): a layout frame
            // with no root mounted is a wiring bug — masks outlived their
            // tree — and the tick's own contract answers `LayoutError`
            // instead of panicking out of a `Result` door, the platform
            // logging the refusal and skipping the frame it cannot lay
            // out.
            let Some(root) = root else {
                // The frame tier's own fact (ADR-0077): the refusal
                // queues where it resolves — the wiring bug the log
                // names, one frame later, beside the `Err` the door
                // still answers.
                crate::frame_refusal::note_frame_refusal(crate::inspect::FrameRefusalRecord {
                    species: crate::inspect::FrameRefusalSpecies::LayoutRefused,
                    node: None,
                });
                return Err(LayoutError {
                    message: "layout ran with no root mounted — masks outlived their tree; \
                              mount a root before the first frame"
                        .to_string(),
                    node: None,
                });
            };
            let start = self.timings.then(|| self.read_clock_nanos());
            // The engine seam's own failure (ADR-0023's error shape):
            // the same note site the no-root bug rides — the species is
            // the stage's, the node the engine named, verbatim. The
            // `?`'s propagation is spelled out so the fact queues
            // exactly where the refusal resolves, never one arm later.
            if let Err(error) = self
                .layout
                .sync_and_layout(&mut self.arena, root, &dirty, viewport)
            {
                crate::frame_refusal::note_frame_refusal(crate::inspect::FrameRefusalRecord {
                    species: crate::inspect::FrameRefusalSpecies::LayoutRefused,
                    node: error.node.map(crate::NodeId::raw),
                });
                return Err(error);
            }
            layout_work = start.map_or(0, |t| self.read_clock_nanos().saturating_sub(t));
            self.counters.layout_runs += 1;
        }
        ledger.push(self.run_pass(PassKind::Layout, |_| layout_ran));
        if let Some(run) = ledger.last_mut() {
            run.work_nanos = layout_work;
            run.nanos = layout_work; // the pass body IS the injected call
        }

        // The IME request (§10.3): derived from the focused widget after
        // layout — the caret anchor is the widget's resolved bounds. `None`
        // when no editable widget holds focus (platform detaches the IME).
        //
        // The derivation is a pure function of the focused node and its
        // caret anchor between structure and mutation frames (A7, M84,
        // ADR-0088 — the instrument's find-and-fix shape): a warm focused
        // tick re-derives nothing, the `id_hash` walk's chain vector no
        // longer allocated-and-freed by every idle frame. The key names
        // every input the request reads — the focused node (the walk's
        // root and the purpose's owner), the caret anchor (the bounds),
        // and the frame's structure/mutation flags (a reparent or a
        // widget mutation can move the path or the purpose without moving
        // the anchor) — so any change re-derives exactly once.
        let focused_caret = self.focused.and_then(|focused| self.arena.bounds(focused));
        let ime_key = (self.focused, focused_caret);
        if structure_changed || !mutated.is_empty() || self.ime_cache != Some(ime_key) {
            self.ime_request = self.focused.and_then(|focused| {
                let purpose = self.arena.widget(focused).ime_purpose();
                if !purpose.enabled() {
                    return None;
                }
                Some(ImeRequest {
                    node: self.id_hash(focused),
                    purpose,
                    caret: self.arena.bounds(focused),
                })
            });
            self.ime_cache = Some(ime_key);
        }

        // Pass 7 — compose: translation-only scroll updates. Runs whenever a
        // repaint will: a wheel event moves a scroll offset without touching
        // layout, and programmatic scrolls (the mutate-guard `request` seam)
        // surface as PAINT masks — paint must see translated bounds (§8.2).
        let events_since = self.counters.events_routed - self.last_compose_events;
        let compose_due = layout_ran
            || events_since > 0
            || structure_changed
            || dirty.values().any(|m| m.intersects(Change::PAINT));
        let mut translated = Vec::new();
        let mut compose_work = 0_u64;
        ledger.push(self.run_pass(PassKind::Compose, |s| {
            if compose_due {
                let start = s.timings.then(|| s.read_clock_nanos());
                translated = s.compose_scrolls();
                compose_work = start.map_or(0, |t| s.read_clock_nanos().saturating_sub(t));
                s.last_compose_events = s.counters.events_routed;
            }
            compose_due
        }));
        if let Some(run) = ledger.last_mut() {
            run.work_nanos = compose_work;
            run.nanos = compose_work;
        }

        // Pass 8 — paint: re-emit dirty layers; clean layers replay (§9.1).
        // The frame's damage (§9.3, M3) merges every repainted node's *old*
        // journal extent with its *new* primitive extent — geometry changes
        // damage both rects — into at most three regions, with the
        // full-frame escape hatch. Structural frames rebuild everything and
        // report [`Damage::FULL`].
        let paint_set: FxHashSet<NodeId> = dirty
            .iter()
            .filter(|(_, m)| m.intersects(Change::PAINT.union(Change::LAYOUT)))
            .map(|(n, _)| *n)
            .collect();
        let paint_ran =
            first_frame || structure_changed || !paint_set.is_empty() || !translated.is_empty();
        let mut paint_work = 0_u64;
        let mut scene = None;
        let mut damage = DamageBuilder::new();
        // The ladder's own read around the paint pass (§14.4): measured
        // whenever a budget is installed, independent of the ledger's
        // timings gate — the rung's honesty does not depend on the M1
        // gate's instrumentation being on.
        let paint_ladder_start = ladder_budget.map(|_| self.read_clock_nanos());
        ledger.push(self.run_pass(PassKind::Paint, |s| {
            if paint_ran {
                let start = s.timings.then(|| s.read_clock_nanos());
                if structure_changed {
                    s.journal.clear();
                    s.order_cache = None;
                }
                s.paint_dirty(structure_changed, &paint_set, &translated, &mut damage);
                s.counters.paint_runs += 1;
                // The frame's scene materialization is the paint pass's
                // product: clean layers replay from the journal (§9.1).
                s.ensure_paint_order();
                let order = s.order_cache.as_deref().expect("order just built");
                let snapshot = std::sync::Arc::new(s.journal.snapshot(order));
                s.scene_cache = Some(std::sync::Arc::clone(&snapshot));
                scene = Some(snapshot);
                paint_work = start.map_or(0, |t| s.read_clock_nanos().saturating_sub(t));
            }
            paint_ran
        }));
        if let Some(run) = ledger.last_mut() {
            run.work_nanos = paint_work;
            run.nanos = paint_work;
        }
        // The damage ladder (§9.3): structural/first frames repaint in full,
        // silent frames cost nothing, partial frames carry their regions.
        let mut damage = if !paint_ran {
            Damage::NONE
        } else if structure_changed || first_frame {
            Damage::FULL
        } else {
            damage.finish(viewport)
        };
        // The degradation ladder's third rung (§14.4): a paint pass past
        // the budget's halfway line, on a partial frame, collapses its
        // merged regions to one full-frame present — present full frame.
        // The scene is unchanged (a full present shows the same frame);
        // only the region bookkeeping stops while the frame is behind.
        let paint_ladder_nanos =
            paint_ladder_start.map_or(0, |t| self.read_clock_nanos().saturating_sub(t));
        if let Some(budget) = ladder_budget {
            let halfway = crate::num::u128_to_u64((budget / 2).as_nanos());
            if paint_ran && !damage.is_empty() && !damage.is_full() && paint_ladder_nanos > halfway
            {
                damage = Damage::FULL;
                degradation = DegradationStep::ForceFullPaint;
            }
        }

        // Pass 9 — accessibility: the M3 a11y contract (§11.1). The delta
        // carries dirty ids only, the ids structural churn removed, and the
        // focused node; a focus move runs the pass even with no other a11y
        // work (platform focus lives in the update, not in a node).
        let mut a11y_set: FxHashSet<u64> = dirty
            .iter()
            .filter(|(_, m)| m.forces_a11y() || m.intersects(Change::LAYOUT))
            .map(|(n, _)| self.id_hash(*n))
            .chain(translated.iter().map(|n| self.id_hash(*n)))
            .collect();
        // The announcement route (§11.6, ADR-0024): a pending announcement
        // is the pass's third trigger beside the dirty set and focus moves
        // — the region's node id joins the delta's updated set, and the
        // sink re-emits exactly that node.
        let announcement_dirty = self.announcement_dirty;
        if announcement_dirty {
            a11y_set.insert(crate::announce::ANNOUNCEMENT_NODE_HASH);
        }
        // The degradation ladder's deferral rung (§14.4): a frame past the
        // budget's halfway line when the a11y pass came due defers the
        // pass's *dirty updates* — the masks re-queue and the next frame
        // lands the delta. The rung is for dirty/translated updates only:
        // focus moves, announcements, first frames, and structural churn
        // never defer (latency may degrade, correctness may not — the
        // facts those triggers carry cannot be reconstructed from a
        // re-queued mask).
        let mut a11y_ran = first_frame || !a11y_set.is_empty() || focus_changed;
        let a11y_deferred = match ladder_budget {
            Some(budget) => {
                let halfway = crate::num::u128_to_u64((budget / 2).as_nanos());
                let elapsed = self
                    .read_clock_nanos()
                    .saturating_sub(ladder_start.unwrap_or_default());
                a11y_ran
                    && !focus_changed
                    && !announcement_dirty
                    && !first_frame
                    && !structure_changed
                    && elapsed > halfway
            }
            None => false,
        };
        let mut a11y_work = 0_u64;
        let mut a11y_delta = A11yDelta::empty();
        ledger.push(self.run_pass(PassKind::Accessibility, |s| {
            if a11y_deferred {
                // The deferral's own bookkeeping: the masks that drove the
                // pass re-queue, so the next frame's pass is due with the
                // same set (the dirty nodes keep their A11Y mask; the
                // translated nodes re-join it the same way). No fact lost:
                // a postponed one.
                for (node, mask) in &dirty {
                    if mask.forces_a11y() || mask.intersects(Change::LAYOUT) {
                        s.arena.merge_mask(*node, Change::A11Y);
                    }
                }
                for node in &translated {
                    s.arena.merge_mask(*node, Change::A11Y);
                }
                return false;
            }
            if a11y_ran {
                s.announcement_dirty = false;
                let start = s.timings.then(|| s.read_clock_nanos());
                let snapshot = std::sync::Arc::new(s.build_tree_snapshot());
                let mut delta = A11yDelta {
                    updated: a11y_set.clone(),
                    removed: Vec::new(),
                    focus: s.focused.map(|f| s.id_hash(f)),
                    initializes: first_frame || structure_changed,
                };
                // Removals: the previous id set minus the current one — dead
                // widgets surface exactly once (§11.1 item 4, incremental
                // updates with no ghosts).
                if let Some(previous) = s.last_a11y_ids.take() {
                    let current: FxHashSet<u64> = snapshot.nodes.iter().map(|n| n.id).collect();
                    delta.removed = previous.difference(&current).copied().collect();
                    s.last_a11y_ids = Some(current);
                } else {
                    s.last_a11y_ids = Some(snapshot.nodes.iter().map(|n| n.id).collect());
                }
                s.a11y.apply(std::sync::Arc::clone(&snapshot), &delta);
                // The dump seam's door (ADR-0010): the observer that opted
                // in receives the same Arc the sink saw — one branch when
                // none (the A7 idle contract), the tree riding no ring and
                // no broadcast, the recorder retaining it as state.
                if let Some(observer) = s.observer.as_mut() {
                    observer.tree(std::sync::Arc::clone(&snapshot));
                }
                a11y_delta = delta;
                a11y_work = start.map_or(0, |t| s.read_clock_nanos().saturating_sub(t));
                s.counters.a11y_runs += 1;
            }
            a11y_ran
        }));
        if let Some(run) = ledger.last_mut() {
            run.work_nanos = a11y_work;
            run.nanos = a11y_work;
        }
        if a11y_deferred {
            degradation = DegradationStep::SkipA11y;
            // The report's flag stays honest: the pass did not run this
            // frame — the delta is queued, not applied (the ledger entry
            // and this flag name the same fact).
            a11y_ran = false;
        }

        let total_nanos = total_start.map_or(0, |t| self.read_clock_nanos().saturating_sub(t));
        // A frame that skipped paint replays the cached materialization —
        // the cheap replay IS the journal's point (§9.1): no walk, no
        // primitive clones, one Arc clone (§14.3's warm path).
        let scene = scene
            .or_else(|| self.scene_cache.as_ref().map(std::sync::Arc::clone))
            .unwrap_or_default();
        self.frames += 1;
        let passes = self.ledger_handle(ledger);
        let report = FrameReport {
            layout_ran,
            paint_ran,
            a11y_ran,
            scene,
            damage,
            a11y_delta,
            passes,
            focus_changed,
            degradation_step: degradation,
            total_nanos,
        };
        self.notify_frame(&report, &dirty, viewport);
        Ok(report)
    }

    /// The report's ledger handle (A7's budget, M84, ADR-0088): a warm
    /// idle frame with timings off produces the same eight skipped runs
    /// every tick, so the canonical ledger is built once and handed out
    /// by [`std::sync::Arc`] handle — a warm tick allocates nothing. The
    /// ledger buffer rides back into the pipeline on every cache hit (its
    /// capacity is the reuse), so the take-push cycle at the tick's start
    /// re-pays nothing in steady state. A frame where a pass ran, a
    /// dispatch-driven entry rides, or timings are on carries its own
    /// freshly built ledger, honestly priced.
    fn ledger_handle(&mut self, ledger: Vec<PassRun>) -> std::sync::Arc<Vec<PassRun>> {
        let canonical = !self.timings && ledger.iter().all(|run| !run.ran);
        if canonical {
            if let Some(cached) = &self.idle_ledger
                && cached.len() == ledger.len()
            {
                // The buffer's capacity rides back for the next tick —
                // the tick's take-push cycle re-pays nothing.
                self.interframe = ledger;
                self.interframe.clear();
                return std::sync::Arc::clone(cached);
            }
            let arc = std::sync::Arc::new(ledger);
            self.idle_ledger = Some(std::sync::Arc::clone(&arc));
            arc
        } else {
            std::sync::Arc::new(ledger)
        }
    }

    /// Runs one pass under its tracing span, recording the ledger entry.
    /// `body` returns whether the pass had work.
    fn run_pass(&mut self, kind: PassKind, body: impl FnOnce(&mut Self) -> bool) -> PassRun {
        let span = tracing::trace_span!(target: "alacer::pass", "pass", pass = kind.name());
        let start = self.timings.then(|| self.read_clock_nanos());
        let guard = span.enter();
        let ran = body(self);
        let nanos = start.map_or(0, |t| self.read_clock_nanos().saturating_sub(t));
        drop(guard);
        PassRun {
            kind,
            ran,
            nanos,
            work_nanos: 0,
        }
    }

    /// Pass 2 body: walk the arena and collect the focusable candidates
    /// (§7.2 pass 2 — recomputed after structure changes).
    fn recompute_focusables(&mut self) {
        let mut focusables = Vec::new();
        self.arena.for_each_pre_order(&mut |id, slot| {
            if slot.widget().focusable() {
                focusables.push(id);
            }
        });
        self.focusables = focusables;
    }

    /// Pass 3 body: resolve the pending focus request, drop focus held by
    /// dead widgets, emit `FOCUS` masks on moves, and notify the old and new
    /// focus holders via [`Widget::set_focused`] (the focus-ring contract:
    /// the widget repaints itself, the framework only flags PAINT). Returns
    /// whether focus actually moved (§7.2 pass 3 — focus diffs).
    fn update_focus(&mut self) -> bool {
        let mut changed = false;
        if let Some(requested) = self.pending_focus.take() {
            let valid =
                self.node_is_live(requested) && self.arena.slot(requested).widget().focusable();
            if valid && self.focused != Some(requested) {
                let old = self.focused.replace(requested);
                self.arena
                    .merge_mask(requested, Change::FOCUS.union(Change::PAINT));
                let mut guard = self.arena.mutate(requested);
                guard.widget_mut().set_focused(true);
                drop(guard);
                if let Some(old) = old {
                    self.arena
                        .merge_mask(old, Change::FOCUS.union(Change::PAINT));
                    let mut guard = self.arena.mutate(old);
                    guard.widget_mut().set_focused(false);
                }
                changed = true;
            }
        }
        // Focus held by a removed widget is dropped (Masonry's sentinel
        // discipline: a dead focus target must never survive the frame). The
        // dead slot is gone from the index — no mask to merge, just release.
        // The same discipline evicts a control that became unavailable
        // (M22, ADR-0028): the slot is live, so the release repainting the
        // focus ring is the widget's own PAINT merge, exactly like a move.
        if let Some(current) = self.focused {
            let dead = !self.node_is_live(current);
            let unavailable = !dead && !self.arena.widget(current).enabled();
            if dead || unavailable {
                if unavailable {
                    self.arena
                        .merge_mask(current, Change::FOCUS.union(Change::PAINT));
                    let mut guard = self.arena.mutate(current);
                    guard.widget_mut().set_focused(false);
                }
                self.focused = None;
                changed = true;
            }
        }
        changed
    }

    /// Whether `node` is a live arena slot.
    fn node_is_live(&self, node: NodeId) -> bool {
        self.arena.is_live(node)
    }

    /// Pass 5 body: diff each candidate's style against the cache and return
    /// the nodes whose style changed, tagged with the invalidation reason
    /// (Floem's reason-flag recalc, §7.2 pass 5). Candidates are the frame's
    /// dirty nodes plus the nodes the mutate pass just drained. The tick
    /// merges results into its local dirty set — same frame, no one-frame
    /// delay, no lingering arena masks.
    fn style_pass(
        &mut self,
        dirty: &FxHashMap<NodeId, Change>,
        mutated: &[NodeId],
    ) -> Vec<(NodeId, Change)> {
        let mut invalidated: Vec<(NodeId, Change)> = Vec::new();
        let candidates = dirty.keys().copied().chain(mutated.iter().copied());
        for node in candidates {
            let style = self.arena.widget(node).style();
            let Some(cached) = self.style_cache.get(&node) else {
                self.style_cache.insert(node, style);
                continue; // first sighting: the mount masks already force layout
            };
            if *cached == style {
                continue;
            }
            // `clip_overflow` paints (viewport clipping); any other style
            // diff is layout — the reason flag the branch split already
            // carries (the pre-M25 sweep's dead comparison removed: the
            // normalized clone was computed and discarded every frame).
            let paint_diff = cached.clip_overflow != style.clip_overflow;
            let change = if paint_diff {
                Change::LAYOUT.union(Change::PAINT)
            } else {
                Change::LAYOUT
            };
            self.style_cache.insert(node, style);
            invalidated.push((node, change));
        }
        // Structural churn retires cache entries for dead nodes.
        if dirty.values().any(|m| m.contains(Change::STRUCTURE)) {
            let dead: Vec<NodeId> = self
                .style_cache
                .keys()
                .copied()
                .filter(|node| !self.node_is_live(*node))
                .collect();
            for node in dead {
                self.style_cache.remove(&node);
            }
        }
        invalidated
    }

    /// Pass 7 (`compose`): applies scroll-offset deltas to descendants without
    /// re-measuring — the translation-only workhorse (§8.2 item 4).
    ///
    /// Returns the descendants whose bounds moved (they need repaint + a11y
    /// bounds updates, never layout).
    fn compose_scrolls(&mut self) -> Vec<NodeId> {
        // Candidates come from the arena's scrollable index (attach-time
        // `is_scroll_container`), so this is O(scrollables), not O(tree).
        // Offsets are compared against the last applied value, so scrolling
        // back to zero translates too — not just non-zero offsets.
        let candidates: Vec<(NodeId, Vector)> = self
            .arena
            .scrollables()
            .filter_map(|id| {
                let slot = self.arena.widget(id);
                let offset = slot.scroll_offset();
                let last = self
                    .scroll_offsets
                    .get(&id)
                    .copied()
                    .unwrap_or(Vector::zero());
                (offset != last).then_some((id, offset))
            })
            .collect();

        let mut moved = Vec::new();
        for (container, offset) in candidates {
            let last = self
                .scroll_offsets
                .get(&container)
                .copied()
                .unwrap_or(Vector::zero());
            let delta = last - offset;
            if delta == Vector::zero() {
                continue;
            }
            let mut descendants = Vec::new();
            self.collect_subtree(container, &mut descendants);
            for id in descendants {
                if let Some(bounds) = self.arena.bounds(id) {
                    self.arena.update_bounds(id, bounds.translate(delta));
                    moved.push(id);
                }
            }
            self.scroll_offsets.insert(container, offset);
        }
        moved
    }

    fn collect_subtree(&self, node: NodeId, out: &mut Vec<NodeId>) {
        out.push(node);
        for child in self.arena.children(node).to_vec() {
            self.collect_subtree(child, out);
        }
    }

    /// Pass 8's re-emission: repaint exactly the dirty subtrees; clean layers
    /// are left in the journal untouched (cached-range replay, §9.1). Every
    /// committed node feeds the frame's damage: the *old* journal extent plus
    /// the *new* primitive extent — geometry changes damage both rects (§9.3).
    fn paint_dirty(
        &mut self,
        full_rebuild: bool,
        paint_set: &FxHashSet<NodeId>,
        translated: &[NodeId],
        damage: &mut DamageBuilder,
    ) {
        let targets: Vec<NodeId> = if full_rebuild {
            // Structural frames re-emit every layer, in paint order.
            self.ensure_paint_order();
            self.order_cache
                .as_deref()
                .expect("order just built")
                .to_vec()
        } else {
            // Layers are keyed by node, so commit order is irrelevant: the
            // dirty set plus the composed translations is the whole target
            // list — no walk over clean subtrees (ADR-0008 item 4).
            paint_set.iter().chain(translated.iter()).copied().collect()
        };
        // The frame's palette, resolved once per paint pass (ADR-0079):
        // one dynamic call per frame — the installed seam's roles, or
        // the canon constants when nothing is installed — handed to
        // every node's context as the copy it paints from. Never per
        // node: the resolution is the pass's own overhead, priced by
        // the pass-overhead contract, not by the node count.
        let palette = self.theme.as_deref().map_or(
            crate::theme::Palette::CANON,
            crate::theme::ThemeSeam::palette,
        );
        // The frame's OS facts, resolved once per paint pass
        // (ADR-0080, ADR-0083, ADR-0085, ADR-0087): one dynamic call per
        // fact per frame — the installed seam's dark-mode, reduced-motion,
        // high-contrast, and accent-color answers (the accent riding its
        // presence — `None` the honest absence), or the canon facts when
        // nothing is installed — handed to every node's context beside
        // the palette. Never per node: the resolutions ride the
        // palette's own once-per-pass overhead, priced by the
        // pass-overhead contract.
        let dark = self
            .settings
            .as_deref()
            .is_some_and(crate::settings::SettingSeam::dark_mode);
        let reduced_motion = self
            .settings
            .as_deref()
            .is_some_and(crate::settings::SettingSeam::reduced_motion);
        let high_contrast = self
            .settings
            .as_deref()
            .is_some_and(crate::settings::SettingSeam::high_contrast);
        let accent = self
            .settings
            .as_deref()
            .and_then(crate::settings::SettingSeam::accent_color);
        for node in targets {
            // The old layer's extent is damage too — a moved/resized widget
            // must erase where it *was*, not only paint where it is. An
            // empty (or absent) old layer painted nothing: there is nothing
            // to erase, and guessing from bounds would over-damage
            // container nodes whose layers are empty by construction.
            if let Some(old) = self.journal.layer(node)
                && let Some(extent) = crate::scene::primitives_extent(&old)
            {
                damage.add(extent);
            } else if self.journal.layer(node).is_none()
                && let Some(bounds) = self.arena.bounds(node)
            {
                // No layer at all: first-ever paint (first frame or after a
                // structural rebuild — both FULL-damage frames), or a layer
                // dropped by churn. Bounds are the only trace left.
                damage.add(crate::scene::inflate(
                    bounds,
                    crate::scene::PAINT_DAMAGE_MARGIN,
                ));
            }
            let primitives =
                self.paint_node(node, palette, dark, reduced_motion, high_contrast, accent);
            if let Some(extent) = crate::scene::primitives_extent(&primitives) {
                damage.add(extent);
            }
            // An identical re-emission is a replay, not a commit (the M23
            // ledger's own rule — "clean layers replay untouched", read to
            // its end): the `PAINT` propagation merges the mask up the
            // ancestor chain (mask.rs's currency table), so a child's
            // repaint also dirties the containers above it — containers
            // whose layers are their own primitives, not their children's.
            // Re-emitting the identical layer would only churn the ledger
            // (one write batch ⇒ N commits, the frame_ledger contract
            // dead on arrival); skipping it keeps the ledger honest — the
            // count is the work the frame actually did. The dirty pixels
            // are unaffected: an identical layer paints nothing new, and
            // its empty extent never added damage anyway.
            let replayed = self
                .journal
                .layer(node)
                .is_some_and(|old| old == primitives);
            if replayed {
                continue;
            }
            self.journal.commit_layer(node, primitives);
        }
        self.ensure_paint_order();
        self.journal
            .retain(self.order_cache.as_deref().expect("order just built"));
    }

    /// Rebuilds the pre-order paint order cache when it is stale. The cache
    /// is a pure function of tree shape, so only structural change (which
    /// resets it) and the first frame pay the walk (ADR-0008 item 4).
    fn ensure_paint_order(&mut self) {
        if self.order_cache.is_none() {
            let mut order = Vec::new();
            self.arena.for_each_pre_order(&mut |id, _| order.push(id));
            self.order_cache = Some(order);
        }
    }

    fn paint_node(
        &mut self,
        node: NodeId,
        palette: crate::theme::Palette,
        dark: bool,
        reduced_motion: bool,
        high_contrast: bool,
        accent: Option<crate::color::Color>,
    ) -> Vec<Primitive> {
        struct JournalVec<'a> {
            out: &'a mut Vec<Primitive>,
        }
        impl crate::scene::PaintSink for JournalVec<'_> {
            fn push(&mut self, primitive: Primitive) {
                self.out.push(primitive);
            }
        }
        let bounds = self.arena.bounds(node);
        let origin = bounds.map_or(Point::zero(), |b| b.origin);
        let size = bounds.map_or(crate::geom::Size::zero(), |b| b.size);
        let clip = self.ancestor_clip(node);
        // The compose tier's inheritance product (§14.2's W3 mechanism):
        // the node's own compose state beside every ancestor's, folded by
        // the walk before the context exists — the pushes apply it at
        // record time, and an identity product records the baseline
        // primitives byte for byte.
        let compose = self.ancestor_compose(node);
        let mut out = Vec::new();
        {
            let mut sink = JournalVec { out: &mut out };
            let mut ctx = PaintCtx::new(
                &mut sink,
                origin,
                size,
                clip,
                palette,
                dark,
                reduced_motion,
                high_contrast,
                accent,
            );
            ctx.set_compose(compose);
            let widget = &self.arena.slot(node).widget;
            widget.paint(&mut ctx);
        }
        out
    }

    /// The compose state in effect at `node` (the compose tier's
    /// inheritance, §14.2's W3 mechanism): the node's own beside every
    /// ancestor's — fades multiplying down the tree (a faded parent fades
    /// its subtree, the CSS rule), translations adding — the same
    /// ancestor walk [`Self::ancestor_clip`] prices, one `compose()` read
    /// per visited node (the pass-overhead class the clip walk already
    /// pays, over painted nodes only). The identity when nobody composes:
    /// the recorded scene is the byte-identical baseline.
    fn ancestor_compose(&self, node: NodeId) -> crate::scene::Compose {
        let mut compose = crate::scene::Compose::IDENTITY;
        let mut cursor = Some(node);
        while let Some(id) = cursor {
            let slot = self.arena.slot(id);
            let own = slot.widget().compose();
            compose.opacity *= own.opacity;
            compose.translation += own.translation;
            cursor = slot.parent;
        }
        compose
    }

    /// The most restrictive ancestor clip in effect at `node` (scroll
    /// viewports clip children; A8).
    fn ancestor_clip(&self, node: NodeId) -> Option<Rect> {
        let mut cursor = self.arena.parent(node);
        let mut clip: Option<Rect> = None;
        while let Some(id) = cursor {
            let slot = self.arena.slot(id);
            if slot.widget().style().clip_overflow
                && let Some(bounds) = slot.bounds
            {
                clip = Some(match clip {
                    Some(c) => c.intersection(&bounds).unwrap_or_default(),
                    None => bounds,
                });
            }
            cursor = slot.parent;
        }
        clip
    }

    /// The mask a secure entry's tree value carries (M100, ADR-0104):
    /// one masking character per code point of the committed value —
    /// the shape the AT reads, never the secret itself. AccessKit's own
    /// contract: a `PasswordInput` node's value "must only contain
    /// masking characters"; the tree is a fact surface, and a secret is
    /// a payload no tree carries (the clipboard trail's rule, ADR-0045,
    /// at the tree tier).
    fn mask_secret(value: &str) -> String {
        "•".repeat(value.chars().count())
    }

    /// Builds the a11y snapshot from the arena (pass 9 input).
    fn build_tree_snapshot(&self) -> TreeSnapshot {
        let mut snapshot = TreeSnapshot::default();
        self.arena.for_each_pre_order(&mut |id, slot| {
            // The secure tier (M100, ADR-0104): the constructor word the
            // field declared rides the snapshot, and the value the tree
            // resolves for a secured field is the mask the word prices —
            // the length the AT reads, never the secret itself.
            let secure_entry = slot.widget().secure_entry();
            // The reveal tier (M102, ADR-0106): the mask the secure
            // word prices yields to the revealed state — a revealed
            // field's tree value is its ordinary value again, the
            // pixels and the tree telling one truth (AccessKit's own
            // contract: the mapping reads `Role::TextInput` once the
            // password is revealed).
            let secure_revealed = secure_entry && slot.widget().secure_revealed();
            let value = match slot.widget().value() {
                Some(value) if secure_entry && !secure_revealed => Some(Self::mask_secret(&value)),
                other => other,
            };
            let node = crate::passes::TreeNode {
                id: self.id_hash(id),
                role: slot.widget().role(),
                label: slot.widget().label().resolve(),
                bounds: slot.bounds,
                children: slot.children.iter().map(|c| self.id_hash(*c)).collect(),
                value,
                toggled: slot.widget().toggled(),
                table_slot: slot.widget().table_slot(),
                selection: slot.widget().selection(),
                // The live tier (M120, ADR-0124): the app's declared
                // live region — the word the containers carry beside
                // the pipeline's own reserved region, whose politeness
                // rides the same field below.
                live: slot.widget().live(),
                enabled: slot.widget().enabled(),
                description: slot.widget().description().map(str::to_owned),
                placeholder: slot.widget().placeholder().map(str::to_owned),
                value_range: slot.widget().value_range(),
                numeric_value: slot.widget().numeric_value(),
                secure_entry,
                secure_revealed,
                scroll_orientation: slot.widget().scroll_orientation(),
                scroll_position: slot.widget().scroll_position(),
                read_only: slot.widget().read_only(),
                required: slot.widget().required(),
                invalid: slot.widget().invalid(),
                selected: slot.widget().selected(),
                multiselectable: slot.widget().multiselectable(),
                busy: slot.widget().busy(),
                expanded: slot.widget().expanded(),
                auto_complete: slot.widget().auto_complete(),
                sort_direction: slot.widget().sort_direction(),
                level: slot.widget().level(),
                current: slot.widget().current(),
                has_popup: slot.widget().has_popup(),
                role_description: slot.widget().role_description().map(str::to_owned),
                position_in_set: slot.widget().position_in_set(),
                size_of_set: slot.widget().size_of_set(),
                keyboard_shortcut: slot.widget().keyboard_shortcut().map(str::to_owned),
                multiline: slot.widget().multiline(),
                language: slot.widget().language().map(str::to_owned),
                search_field: slot.widget().search_field(),
            };
            if snapshot.root.is_none() {
                snapshot.root = Some(node.id);
            }
            snapshot.nodes.push(node);
        });
        // The announcement region (§11.6, ADR-0024): the app's reserved
        // live-region node — always present from the first frame, the
        // root's last child, its name the last announcement's text. One
        // node in the snapshot, never a widget: the pipeline owns the
        // region, the AT speaks its changes, and announcing never moves
        // the tree's shape.
        // The announcement region (§11.6, ADR-0024): the app's reserved
        // live-region node — always present from the first frame, the
        // root's last child, its name the last announcement's text
        // ([`Self::announcement_node`], the pipeline's own node).
        if let Some(root) = snapshot.nodes.first_mut() {
            root.children.push(crate::announce::ANNOUNCEMENT_NODE_HASH);
            snapshot.nodes.push(self.announcement_node());
        }
        snapshot
    }

    /// The announcement region's snapshot node (§11.6, ADR-0024): the
    /// reserved live region the pipeline owns — one node in the snapshot,
    /// never a widget: the pipeline owns the region, the AT speaks its
    /// changes, and announcing never moves the tree's shape. Its content
    /// is the last announcement; every word the widgets carry is
    /// honestly absent — the region is the speech, it carries no other.
    fn announcement_node(&self) -> crate::passes::TreeNode {
        let (text, politeness) = self
            .announcement
            .as_ref()
            .map_or((String::new(), crate::announce::Politeness::Polite), |a| {
                (a.text.clone(), a.politeness)
            });
        crate::passes::TreeNode {
            id: crate::announce::ANNOUNCEMENT_NODE_HASH,
            role: crate::widget::Role::LiveRegion,
            label: text,
            bounds: None,
            children: Vec::new(),
            value: None,
            toggled: None,
            table_slot: None,
            selection: None,
            live: Some(politeness),
            enabled: true,
            // The pipeline's own node: the region explains nothing —
            // the announcement *is* its content (the honest absence).
            description: None,
            placeholder: None,
            value_range: None,
            numeric_value: None,
            secure_entry: false,
            secure_revealed: false,
            scroll_orientation: None,
            scroll_position: None,
            // The pipeline's own node: no read-only word — the
            // region's content is the announcement itself.
            read_only: false,
            required: false,
            // The pipeline's own node: no invalid word — the
            // region's content is the announcement itself.
            invalid: None,
            // The pipeline's own node: no selected word — the
            // region belongs to no selection collection.
            selected: None,
            // The pipeline's own node: no capacity word — the
            // region carries no collection to size.
            multiselectable: false,
            // The pipeline's own node: no updating word — the
            // region is never withheld; it is the speech. Its
            // politeness above is the live tier's own rider (M120,
            // ADR-0124): the region the imperative channel speaks,
            // beside the regions the app declares.
            busy: false,
            // The pipeline's own node: no disclosure word — the
            // region opens and closes not.
            expanded: None,
            // The pipeline's own node: no suggestion contract —
            // the announcement is the region's content.
            auto_complete: None,
            // The pipeline's own node: no sorted column — the
            // region orders nothing.
            sort_direction: None,
            // The pipeline's own node: no depth to say — the
            // region nests not.
            level: None,
            // The pipeline's own node: no set to stand in — the
            // region is current for no item.
            current: None,
            // The pipeline's own node: nothing to open — the
            // region speaks its own content.
            has_popup: None,
            // The pipeline's own node: no read-as role — the region
            // is read as the live region it is.
            role_description: None,
            // The pipeline's own node: no place in a set — the
            // region stands in no collection.
            position_in_set: None,
            // The pipeline's own node: no span to size — the
            // region collects nothing.
            size_of_set: None,
            // The pipeline's own node: no chord to say — the region
            // is invoked by no key.
            keyboard_shortcut: None,
            // The pipeline's own node: no capacity to take — the
            // region is no field.
            multiline: false,
            // The pipeline's own node: no language to say — the
            // region speaks its own content.
            language: None,
            search_field: false,
        }
    }

    /// Pass 10 (`action`): routes an accessibility action to the widget whose
    /// id-path hash is `target` (§7.2 pass 10: side-channel, never masks).
    ///
    /// A *consumed* action invalidates through the one currency: the widget's
    /// state changed, so `PAINT | A11Y` merge on the target — the repaint and
    /// the re-announcement ride the same masks a user interaction would queue
    /// (A1; the §13.3 inspector sees the action as an input event).
    pub fn dispatch_a11y_action(&mut self, target: u64, action: crate::widget::A11yAction) -> bool {
        let Some(node) = self.node_for_hash(target) else {
            // The funnel's own defensive drop (M57, ADR-0063): the ask
            // was aimed at an id no tree node carries — no widget's door
            // ever answered, so the route names its own fact instead of
            // a munged widget-door one: the action beside the aimed id,
            // verbatim, the species the boundary refused with. Since
            // M65 (ADR-0071) the record names its rider — the funnel
            // declared where it stands (the M30 attribution rule at the
            // route tier), the boundary's mapping arms and the IME
            // route's drops tellable apart on one log. One branch when
            // nobody listens (the A7 idle contract).
            crate::route_drop::note_route_drop(crate::inspect::RouteDropRecord {
                action: action.name(),
                species: crate::inspect::RouteDropSpecies::UnknownTarget,
                rider: crate::inspect::RouteDropRider::Funnel,
                target: Some(target),
            });
            return false;
        };
        let mut guard = self.arena.mutate(node);
        // The stamp side (M43, ADR-0049, ADR-0046's first re-open rule):
        // the facts the widget queues while answering carry the id the
        // action arrived under — the funnel knows it, the widget does
        // not, so the guard lives here and dies here (a panicking
        // widget leaves no stale identity on the thread).
        let stamp = crate::value::ActionTargetGuard::for_target(Some(target));
        // The scroll trail's stamp (M50, ADR-0056): the same rule at the
        // scroll tier — the AT's scroll asks (the unit scrolls, the
        // value steps) queue their facts beneath the id the action
        // arrived under, the wheel's stamp discipline beside the
        // action route's own.
        let scroll_stamp = crate::scroll::ScrollTargetGuard::for_target(Some(target));
        // The activation ask rides the action trail (M69, ADR-0075) —
        // the ask's tier read before the door consumes the action.
        let is_press = matches!(action, crate::widget::A11yAction::DefaultPress);
        let consumed = guard.widget_mut().on_a11y_action(action);
        if consumed {
            guard.request(Change::PAINT.union(Change::A11Y));
        }
        drop(guard);
        drop(stamp);
        drop(scroll_stamp);
        // The action trail's note site (M69, ADR-0075): the activation
        // ask answers as data — the landing, or the species computed
        // where both facts live. The route holds the ask, the aimed id,
        // the door's answer, and the tree's one `enabled` truth (M22,
        // ADR-0028): a refused ask names the availability wall when the
        // flag held it and the doorless silence when nothing did — the
        // collapsed `false` split, never guessed at a widget-private
        // call site. Since M70 (ADR-0076) the record names its rider —
        // the action route's own note site, the keyboard's activation
        // naming itself at the widget's door (the M30 rider rule at the
        // activation tier). One fact per trail (the M39 rule at the
        // activation tier): the asks whose tiers already price
        // themselves — the value tier's writes, the scroll tier's asks,
        // the text-write tier's rewrites — stream no action fact; the
        // funnel notes the activation ask only. An ask the boundary
        // dropped before this door (no target to resolve) is the
        // route-drop trail's fact (ADR-0063), never a munged action
        // record. One branch when nobody listens (the A7 idle
        // contract).
        if is_press {
            let outcome = if consumed {
                crate::inspect::ActionOutcome::Delivered
            } else if !self.arena.widget(node).enabled() {
                crate::inspect::ActionOutcome::Refused(crate::inspect::ActionRefusal::Unavailable)
            } else {
                crate::inspect::ActionOutcome::Refused(crate::inspect::ActionRefusal::NoDoor)
            };
            crate::action::note_action(crate::inspect::ActionRecord {
                ask: crate::inspect::ActionAsk::Press,
                target: Some(target),
                outcome,
                rider: crate::inspect::ActionRider::ActionRoute,
            });
        }
        consumed
    }

    /// Maps an id-path hash back to its arena node (walk = id-path rebuild).
    fn node_for_hash(&self, target: u64) -> Option<NodeId> {
        let mut found = None;
        self.arena.for_each_pre_order(&mut |id, _| {
            if found.is_none() && self.id_hash(id) == target {
                found = Some(id);
            }
        });
        found
    }

    /// AccessKit-stable node id: the id-path hash of the arena path (§6.3).
    fn id_hash(&self, node: NodeId) -> u64 {
        let mut chain = Vec::new();
        let mut cursor = Some(node);
        while let Some(id) = cursor {
            chain.push(id);
            cursor = self.arena.parent(id);
        }
        let mut path = crate::identity::IdPath::default();
        for id in chain.into_iter().rev() {
            path.push(id.raw());
        }
        path.hash()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        A11ySink, FocusLanding, FocusMove, FocusMoveOutcome, FocusRefusal, JournalStats,
        LayoutEngine, LayoutError, PassKind, PassPipeline, SceneJournal, TreeSnapshot,
    };
    use crate::arena::{Arena, NodeId};
    use crate::color::Color;
    use crate::events::PointerEvent;
    use crate::flux::Flux;
    use crate::geom::{Point, Rect, Size, Vector};
    use crate::inspect::{InspectEvent, InspectObserver};
    use crate::mask::Change;
    use crate::scene::Quad;
    use crate::style::Style;
    use crate::widget::{EventResponse, LabelSource, Role, Widget};
    use reactive_graph::traits::Set;
    use rustc_hash::{FxHashMap, FxHashSet};
    use std::sync::Mutex;

    struct Fixed {
        size: crate::geom::Size,
    }

    impl Widget for Fixed {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn role(&self) -> Role {
            Role::Container
        }

        fn label(&self) -> LabelSource {
            LabelSource::text("fixed")
        }

        fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
            Some(crate::style::MeasureSpec::Fixed(self.size))
        }
    }

    /// A layout engine that stacks children vertically with fixed sizes —
    /// deterministic geometry for pipeline tests, zero Taffy.
    struct StackLayout {
        measure_calls: u64,
    }

    impl StackLayout {
        fn new() -> Self {
            Self { measure_calls: 0 }
        }
    }

    impl LayoutEngine for StackLayout {
        fn sync_and_layout(
            &mut self,
            arena: &mut Arena,
            root: NodeId,
            _dirty: &FxHashMap<NodeId, Change>,
            viewport: Size,
        ) -> Result<(), LayoutError> {
            fn layout_subtree(
                engine: &mut StackLayout,
                arena: &mut Arena,
                node: NodeId,
                origin: Point,
                viewport: Size,
            ) {
                let spec = arena.widget(node).measure_spec();
                let (w, h) = match spec {
                    Some(crate::style::MeasureSpec::Fixed(size)) => {
                        engine.measure_calls += 1;
                        (size.width.min(viewport.width), size.height)
                    }
                    _ => (viewport.width, 0.0),
                };
                let bounds = Rect::new(origin, Size::new(w, h));
                arena.update_bounds(node, bounds);
                let mut y = origin.y;
                for child in arena.children(node).to_vec() {
                    layout_subtree(engine, arena, child, Point::new(origin.x, y), viewport);
                    y = arena.bounds(child).map_or(y, |b| b.max_y());
                }
            }
            layout_subtree(self, arena, root, Point::zero(), viewport);
            Ok(())
        }

        fn measure_calls(&self) -> u64 {
            self.measure_calls
        }

        fn reset_measure_calls(&mut self) {
            self.measure_calls = 0;
        }
    }

    #[derive(Default)]
    struct MemoryJournal {
        layers: Mutex<FxHashMap<NodeId, Vec<crate::scene::Primitive>>>,
        commits: Mutex<u64>,
    }

    impl SceneJournal for MemoryJournal {
        fn commit_layer(&mut self, node: NodeId, primitives: Vec<crate::scene::Primitive>) {
            *self.commits.lock().expect("journal lock") += 1;
            self.layers
                .lock()
                .expect("journal lock")
                .insert(node, primitives);
        }

        fn drop_layer(&mut self, node: NodeId) {
            self.layers.lock().expect("journal lock").remove(&node);
        }

        fn layer(&self, node: NodeId) -> Option<Vec<crate::scene::Primitive>> {
            self.layers
                .lock()
                .expect("journal lock")
                .get(&node)
                .cloned()
        }

        fn snapshot(&self, order: &[NodeId]) -> crate::scene::SceneSnapshot {
            let layers = self.layers.lock().expect("journal lock");
            let primitives = order
                .iter()
                .filter_map(|n| layers.get(n))
                .flatten()
                .cloned()
                .collect();
            crate::scene::SceneSnapshot { primitives }
        }

        fn retain(&mut self, order: &[NodeId]) {
            let keep: FxHashSet<NodeId> = order.iter().copied().collect();
            self.layers
                .lock()
                .expect("journal lock")
                .retain(|n, _| keep.contains(n));
        }

        fn clear(&mut self) {
            self.layers.lock().expect("journal lock").clear();
        }

        fn stats(&self) -> JournalStats {
            let commits = *self.commits.lock().expect("journal lock");
            let live_layers = self.layers.lock().expect("journal lock").len();
            JournalStats {
                commits,
                drops: 0,
                live_layers,
            }
        }
    }

    #[derive(Default)]
    struct RecordingA11y {
        updates: u64,
        last_delta: crate::passes::A11yDelta,
        tree: Option<TreeSnapshot>,
    }

    impl A11ySink for RecordingA11y {
        fn apply(
            &mut self,
            snapshot: std::sync::Arc<TreeSnapshot>,
            delta: &crate::passes::A11yDelta,
        ) {
            self.updates += 1;
            self.last_delta = delta.clone();
            self.tree = Some((*snapshot).clone());
        }

        fn resolved_tree(&self) -> Option<&TreeSnapshot> {
            self.tree.as_ref()
        }

        fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
            Some(self)
        }
    }

    /// Reads the recording sink back through the pipeline's `Any` seam —
    /// the same downcast the platform drain uses (ADR-0015).
    fn recording_of(p: &mut PassPipeline) -> &mut RecordingA11y {
        p.a11y_sink_mut()
            .as_any_mut()
            .and_then(|any| any.downcast_mut::<RecordingA11y>())
            .expect("the test pipeline carries the recording sink")
    }

    fn pipeline() -> PassPipeline {
        PassPipeline::new(
            Flux::new(),
            Box::new(StackLayout::new()),
            Box::new(MemoryJournal::default()),
            Box::new(RecordingA11y::default()),
        )
    }

    fn viewport() -> Size {
        Size::new(200.0, 400.0)
    }

    // ------------------------------------------------------------------
    // M104 (ADR-0108): the tick's clock rides the time seam — the pass
    // ledger's wall pricing answers whatever clock the pipeline holds.
    // A seam clock makes the timed frame exact: every read one step past
    // the last, two ticks pricing identically, a frozen clock pricing
    // zero — no sleeps, no hoping the scheduler agrees.
    // ------------------------------------------------------------------

    /// The seam the tests install: every read one step past the last — a
    /// clock the test controls without sleeping, the deterministic
    /// frame's own shape.
    struct StepClock {
        nanos: std::cell::Cell<u64>,
        step: u64,
    }

    impl StepClock {
        fn new(step: u64) -> Self {
            Self {
                nanos: std::cell::Cell::new(0),
                step,
            }
        }
    }

    impl crate::time::TimeSeam for StepClock {
        fn now_nanos(&self) -> u64 {
            let next = self.nanos.get().saturating_add(self.step);
            self.nanos.set(next);
            next
        }
    }

    /// The pass ledger's prices, plainly — the determinism the seam
    /// claims, as data.
    fn ledger_prices(report: &super::FrameReport) -> Vec<u64> {
        report.passes.iter().map(|run| run.nanos).collect()
    }

    #[test]
    fn a_seam_clock_prices_two_ticks_identically() {
        let mut p = pipeline();
        p.set_timings(true);
        p.set_clock(Box::new(StepClock::new(1_000_000)));
        let _root = p.mount_root(Box::new(Fixed {
            size: Size::new(400.0, 800.0),
        }));

        let first = p.tick(viewport()).expect("the first timed frame");
        let second = p.tick(viewport()).expect("the second timed frame");
        let third = p.tick(viewport()).expect("the third timed frame");

        let first_prices = ledger_prices(&first);
        assert!(
            !first_prices.is_empty() && first_prices.iter().all(|nanos| *nanos > 0),
            "the structural frame runs every pass: every read priced"
        );
        for nanos in first_prices.iter().chain(ledger_prices(&second).iter()) {
            assert_eq!(
                nanos % 1_000_000,
                0,
                "every pass prices whole steps of the seam's own read ({nanos})"
            );
        }
        assert_eq!(
            first.total_nanos % 1_000_000,
            0,
            "the frame's total prices whole steps too"
        );
        assert_eq!(
            ledger_prices(&second),
            ledger_prices(&third),
            "two idle ticks of the same tree price identically — the seam's determinism, as data"
        );
    }

    #[test]
    fn a_frozen_manual_clock_prices_the_tick_zero() {
        let mut p = pipeline();
        p.set_timings(true);
        p.set_clock(Box::new(crate::time::ManualClock::new()));
        let _root = p.mount_root(Box::new(Fixed {
            size: Size::new(400.0, 800.0),
        }));

        let report = p.tick(viewport()).expect("the timed frame");
        for run in report.passes.iter() {
            assert_eq!(
                run.nanos, 0,
                "a clock the test never advanced prices zero — the deterministic frame"
            );
        }
        assert_eq!(
            report.total_nanos, 0,
            "the frame's total answers the same frozen clock"
        );
    }

    #[test]
    fn set_clock_replaces_the_installed_clock() {
        let mut p = pipeline();
        p.set_timings(true);
        p.set_clock(Box::new(crate::time::ManualClock::new()));
        let _root = p.mount_root(Box::new(Fixed {
            size: Size::new(400.0, 800.0),
        }));
        let frozen = p.tick(viewport()).expect("the frozen frame");
        assert!(
            frozen.passes.iter().all(|run| run.nanos == 0),
            "the frozen clock prices zero"
        );

        p.set_clock(Box::new(StepClock::new(500_000)));
        let stepped = p.tick(viewport()).expect("the stepped frame");
        assert!(
            stepped.passes.iter().all(|run| run.nanos % 500_000 == 0)
                && stepped.passes.iter().any(|run| run.nanos > 0),
            "the newest installed clock answers — one door, the last install wins"
        );
    }

    #[test]
    fn without_a_seam_the_tick_still_measures_wall_time() {
        let mut p = pipeline();
        p.set_timings(true);
        let _root = p.mount_root(Box::new(Fixed {
            size: Size::new(400.0, 800.0),
        }));

        let report = p.tick(viewport()).expect("the timed frame");
        assert!(
            report.total_nanos > 0,
            "no seam installed: the fallback anchor still measures — the shipped default's shape"
        );
    }

    #[test]
    fn a_layout_error_names_its_node_as_data_when_it_knows_one() {
        // ADR-0023: the contract's error type carries the failing node as
        // a field a consumer can read — the "widget ids included" promise
        // kept as data, not prose — and stays the honest, sourceless leaf.
        let named = LayoutError {
            message: "the subtree will not fit".to_string(),
            node: Some(NodeId::allocate()),
        };
        let summary = named.to_string();
        assert!(
            summary.contains("the subtree will not fit"),
            "the human summary survives: {summary}"
        );
        assert!(
            summary.contains("node #"),
            "the Display names the node for a human: {summary}"
        );
        assert!(
            named.node.is_some(),
            "the id is data a consumer can resolve, not prose"
        );
        assert!(
            std::error::Error::source(&named).is_none(),
            "the layout failure is the honest leaf — no cause to fake"
        );

        let unnamed = LayoutError {
            message: "the tree will not sync".to_string(),
            node: None,
        };
        assert_eq!(
            unnamed.to_string(),
            "layout error: the tree will not sync",
            "the nodeless form reads as before"
        );
    }

    #[test]
    fn first_frame_runs_every_pass_and_reports_geometry() {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }));
        let report = p.tick(viewport()).expect("first frame");

        assert!(report.layout_ran && report.paint_ran && report.a11y_ran);
        let bounds = p.arena().bounds(root).expect("bounds after layout");
        assert_eq!(bounds, Rect::new(Point::zero(), Size::new(100.0, 30.0)));
        assert_eq!(p.counters().layout_runs, 1);
        assert_eq!(p.counters().paint_runs, 1);
        assert_eq!(p.counters().a11y_runs, 1);
    }

    #[test]
    fn idle_tick_skips_every_pass() {
        let mut p = pipeline();
        let _ = p.mount_root(Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }));
        p.tick(viewport()).expect("first frame");
        let counters = p.counters();

        let report = p.tick(viewport()).expect("idle frame");
        assert!(report.is_idle());
        assert_eq!(p.counters(), counters, "no pass ran when nothing changed");
    }

    #[test]
    fn a11y_snapshot_uses_id_path_hashes_as_node_ids() {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }));
        p.tick(viewport()).expect("frame");

        let tree = p.a11y_snapshot().expect("tree built");
        let root_node = tree
            .nodes
            .iter()
            .find(|n| n.id == root_id_hash(root))
            .expect("root in tree");
        assert_eq!(root_node.role, Role::Container);
        assert_eq!(root_node.label, "fixed");
    }

    fn root_id_hash(root: NodeId) -> u64 {
        let mut path = crate::identity::IdPath::default();
        path.push(root.raw());
        path.hash()
    }

    #[test]
    fn pointer_events_route_to_the_topmost_hit_and_stop_on_handled() {
        struct Handling {
            at: Rect,
        }

        impl Widget for Handling {
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn role(&self) -> Role {
                Role::Button
            }

            fn label(&self) -> LabelSource {
                LabelSource::text("hit")
            }

            fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
                Some(crate::style::MeasureSpec::Fixed(self.at.size))
            }

            fn on_pointer_event(
                &mut self,
                _ctx: &mut crate::events::EventCtx,
                _event: &PointerEvent,
            ) -> EventResponse {
                EventResponse::Handled
            }
        }

        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 400.0),
        }));
        let _btn = p.add_child(
            root,
            Box::new(Handling {
                at: Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 30.0)),
            }),
        );
        p.tick(viewport()).expect("frame");

        let down = PointerEvent::Down {
            position: Point::new(10.0, 10.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        };
        assert_eq!(
            p.dispatch_pointer(&down),
            EventResponse::Handled,
            "button hit first, bubbling stops"
        );
        assert_eq!(p.counters().events_routed, 1);

        let miss = PointerEvent::Down {
            position: Point::new(199.0, 399.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        };
        assert_eq!(
            p.dispatch_pointer(&miss),
            EventResponse::Ignored,
            "routes to root, which ignores"
        );
    }

    #[test]
    fn compose_translates_scrolled_children_without_re_measuring() {
        struct Scroll {
            offset: std::sync::Arc<Mutex<f32>>,
        }

        impl Widget for Scroll {
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn role(&self) -> Role {
                Role::ScrollArea
            }

            fn label(&self) -> LabelSource {
                LabelSource::text("scroll")
            }

            fn style(&self) -> crate::style::Style {
                crate::style::Style {
                    clip_overflow: true,
                    ..crate::style::Style::default()
                }
            }

            fn scroll_offset(&self) -> Vector {
                Vector::new(0.0, *self.offset.lock().expect("scroll lock"))
            }

            fn is_scroll_container(&self) -> bool {
                true
            }
        }

        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 400.0),
        }));
        let offset_cell = std::sync::Arc::new(Mutex::new(0.0_f32));
        let scroll = p.add_child(
            root,
            Box::new(Scroll {
                offset: std::sync::Arc::clone(&offset_cell),
            }),
        );
        let item = p.add_child(
            scroll,
            Box::new(Fixed {
                size: Size::new(100.0, 50.0),
            }),
        );
        p.tick(viewport()).expect("frame");
        let before = p.arena().bounds(item).expect("item bounds");
        p.reset_measure_calls();

        // A wheel event moved the scroll offset — no layout mask anywhere.
        *offset_cell.lock().expect("scroll lock") = 30.0;
        // Dispatching an event marks input as arrived (compose runs).
        let wheel = PointerEvent::Wheel {
            delta: Vector::new(0.0, -30.0),
            position: Point::new(10.0, 10.0),
        };
        p.dispatch_pointer(&wheel);
        let report = p.tick(viewport()).expect("scroll frame");

        assert!(
            !report.layout_ran,
            "scroll must not relayout (translation-only, §8.2)"
        );
        assert!(report.paint_ran, "translated nodes repaint");
        assert_eq!(
            p.measure_calls(),
            0,
            "zero measure calls for translation-only updates"
        );
        let after = p.arena().bounds(item).expect("item bounds");
        assert_eq!(
            after.max_y(),
            before.max_y() - 30.0,
            "child moved up by the delta"
        );
        assert_eq!(after.width(), before.width(), "size untouched by compose");
    }

    #[test]
    fn one_write_batch_means_one_layout_and_paint_cycle() {
        // Serialize against the process-global BATCH state (see
        // `flux::batch_test_lock` for why this is required).
        let _guard = crate::flux::batch_test_lock();

        // The ADR-6 kill-switch, at pipeline level: 5 synchronous writes inside
        // one batch ⇒ exactly one layout pass and one paint pass.
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 400.0),
        }));
        let child = p.add_child(
            root,
            Box::new(Fixed {
                size: Size::new(100.0, 30.0),
            }),
        );
        p.tick(viewport()).expect("first frame");

        let signal = p.flux().signal(0i32);
        p.flux_mut().bind(signal, child, |v: &i32| {
            let _ = v;
            Change::LAYOUT.union(Change::PAINT)
        });
        p.tick(viewport()).expect("binding frame");

        p.flux().batch(|| {
            for i in 1..=5 {
                signal.set(i);
            }
        });
        let report = p.tick(viewport()).expect("write-batch frame");

        assert!(report.layout_ran && report.paint_ran);
        assert_eq!(
            p.counters().layout_runs,
            3,
            "first + binding + batch — never 7"
        );
        assert_eq!(p.counters().paint_runs, 3);

        let idle = p.tick(viewport()).expect("idle frame");
        assert!(idle.is_idle());
        assert_eq!(p.counters().layout_runs, 3, "nothing pending: idle");
    }

    /// A fixed-size widget with a semantic role (focus tests).
    struct Rolied {
        size: crate::geom::Size,
        role: Role,
    }

    impl Widget for Rolied {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn role(&self) -> Role {
            self.role
        }

        fn label(&self) -> LabelSource {
            LabelSource::text("rolied")
        }

        fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
            Some(crate::style::MeasureSpec::Fixed(self.size))
        }
    }

    /// A button whose availability flips through the mutate seam — the
    /// M22 exit route's test double (the tier's single source of truth,
    /// read by the focus passes and the snapshot alike).
    struct Switchable {
        size: crate::geom::Size,
        enabled: bool,
        pressed: bool,
    }

    impl Switchable {
        fn new(size: crate::geom::Size) -> Self {
            Self {
                size,
                enabled: true,
                pressed: false,
            }
        }
    }

    impl Widget for Switchable {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn role(&self) -> Role {
            Role::Button
        }

        fn label(&self) -> LabelSource {
            LabelSource::text("switchable")
        }

        fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
            Some(crate::style::MeasureSpec::Fixed(self.size))
        }

        fn enabled(&self) -> bool {
            self.enabled
        }

        fn on_pointer_event(
            &mut self,
            ctx: &mut crate::events::EventCtx,
            event: &PointerEvent,
        ) -> EventResponse {
            match event {
                PointerEvent::Down { .. } => {
                    self.pressed = true;
                    ctx.request(Change::PAINT.union(Change::FOCUS));
                    EventResponse::Handled
                }
                PointerEvent::Up { .. } => {
                    self.pressed = false;
                    ctx.request(Change::PAINT);
                    EventResponse::Handled
                }
                _ => EventResponse::Ignored,
            }
        }
    }

    /// Records `(phase, name)` for every routed event into a shared log; can
    /// stop a phase early to prove capture/bubble semantics.
    struct Recorder {
        name: &'static str,
        role: Role,
        log: std::sync::Arc<Mutex<Vec<(&'static str, &'static str)>>>,
        handle_capture: bool,
    }

    impl Widget for Recorder {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn role(&self) -> Role {
            self.role
        }

        fn label(&self) -> LabelSource {
            LabelSource::text(self.name)
        }

        fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
            Some(crate::style::MeasureSpec::Fixed(Size::new(50.0, 20.0)))
        }

        fn on_pointer_event(
            &mut self,
            ctx: &mut crate::events::EventCtx,
            _event: &PointerEvent,
        ) -> EventResponse {
            self.log.lock().expect("log lock").push((
                match ctx.phase() {
                    crate::events::EventPhase::Capture => "capture",
                    crate::events::EventPhase::Bubble => "bubble",
                },
                self.name,
            ));
            if ctx.phase() == crate::events::EventPhase::Capture && self.handle_capture {
                return EventResponse::Handled;
            }
            EventResponse::Ignored
        }
    }

    #[test]
    fn pass_ledger_lists_every_contract_pass_in_order() {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }));
        let _button = p.add_child(
            root,
            Box::new(Rolied {
                size: Size::new(80.0, 20.0),
                role: Role::Button,
            }),
        );
        let report = p.tick(viewport()).expect("first frame");

        let kinds: Vec<_> = report.passes.iter().map(|run| run.kind).collect();
        assert_eq!(
            kinds,
            PassKind::tick_order().to_vec(),
            "the ledger is the contract order, verbatim"
        );
        for run in report.passes.iter() {
            if run.kind == PassKind::Mutate {
                assert!(
                    !run.ran,
                    "nothing was queued on mount: the mutate pass skips"
                );
            } else {
                assert!(
                    run.ran,
                    "the first frame processes every other pass: {}",
                    run.kind.name()
                );
            }
        }
        // Dispatch-driven passes surface in the next report's ledger.
        p.dispatch_pointer(&PointerEvent::Move {
            position: Point::new(10.0, 10.0),
        });
        let report = p.tick(viewport()).expect("post-event frame");
        assert!(
            report
                .passes
                .iter()
                .any(|run| run.kind == PassKind::Event && run.ran),
            "the event pass is ledgered with the frame that followed it"
        );
    }

    #[test]
    fn idle_frames_skip_every_pass_but_still_ledger_them() {
        let mut p = pipeline();
        let _root = p.mount_root(Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }));
        let _ = p.tick(viewport()).expect("first frame");
        let idle = p.tick(viewport()).expect("idle frame");

        assert!(idle.is_idle());
        for run in idle.passes.iter() {
            assert!(
                !run.ran,
                "idle frames skip {}: still ledgered",
                run.kind.name()
            );
        }
    }

    #[test]
    fn replay_frames_share_the_cached_scene_allocation() {
        // ADR-0008 item 4: idle and replay-only frames reuse the last
        // materialization (one Arc clone, zero primitive clones); a repaint
        // rematerializes.
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }));
        let painted = p.tick(viewport()).expect("paint frame");
        assert!(painted.paint_ran);
        let replay = p.tick(viewport()).expect("replay frame");
        assert!(!replay.paint_ran);
        assert!(
            std::sync::Arc::ptr_eq(&painted.scene, &replay.scene),
            "the replay frame shares the painted scene allocation"
        );

        // A repaint produces a fresh materialization with the same content.
        let mut guard = p.arena_mut().mutate(root);
        guard.request(Change::PAINT);
        drop(guard);
        let repainted = p.tick(viewport()).expect("repaint frame");
        assert!(repainted.paint_ran);
        assert!(
            !std::sync::Arc::ptr_eq(&painted.scene, &repainted.scene),
            "a repaint rematerializes the scene"
        );
        assert_eq!(*painted.scene, *repainted.scene, "content is stable");
    }

    #[test]
    fn capture_precedes_bubble_and_handled_stops_the_phase() {
        let log = std::sync::Arc::new(Mutex::new(Vec::new()));
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Recorder {
            name: "root",
            role: Role::Container,
            log: std::sync::Arc::clone(&log),
            handle_capture: false,
        }));
        let target = p.add_child(
            root,
            Box::new(Recorder {
                name: "target",
                role: Role::Container,
                log: std::sync::Arc::clone(&log),
                handle_capture: false,
            }),
        );
        let _ = p.tick(viewport()).expect("geometry");

        p.dispatch_pointer(&PointerEvent::Down {
            position: Point::new(10.0, 10.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        });
        let order = log.lock().expect("log lock").clone();
        assert_eq!(
            order,
            vec![
                ("capture", "root"),
                ("capture", "target"),
                ("bubble", "target"),
                ("bubble", "root"),
            ],
            "capture descends root→target, bubble ascends target→root"
        );
        let _ = target;
    }

    #[test]
    fn handled_capture_stops_the_route_before_bubble() {
        let log = std::sync::Arc::new(Mutex::new(Vec::new()));
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Recorder {
            name: "root",
            role: Role::Container,
            log: std::sync::Arc::clone(&log),
            handle_capture: true, // stops capture immediately
        }));
        let _target = p.add_child(
            root,
            Box::new(Recorder {
                name: "target",
                role: Role::Container,
                log: std::sync::Arc::clone(&log),
                handle_capture: false,
            }),
        );
        let _ = p.tick(viewport()).expect("geometry");

        let response = p.dispatch_pointer(&PointerEvent::Down {
            position: Point::new(10.0, 10.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        });
        assert_eq!(response, EventResponse::Handled);
        assert_eq!(
            log.lock().expect("log lock").as_slice(),
            [("capture", "root")],
            "a handled capture ends the route: no target, no bubble"
        );
    }

    #[test]
    fn a_directional_move_without_geometry_refuses_by_name() {
        // M27, ADR-0033: the NoGeometry species. Layout never wrote the
        // focused node's bounds, so the directional walk has nothing to
        // read — it refuses with the species named (the trace carries the
        // help: tick one frame first, or ride the tree-order doors),
        // focus untouched, the tree-order door still answering.
        let mut p = PassPipeline::new(
            Flux::new(),
            Box::new(NoLayout),
            Box::new(MemoryJournal::default()),
            Box::new(RecordingA11y::default()),
        );
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let a = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        let b = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        p.tick(viewport()).expect("the candidate list populates");

        // The tree-order door needs no geometry: the walk enters.
        assert!(p.focus_move(FocusMove::Next).moved());
        assert_eq!(p.focus(), Some(a));

        // The directional door reads the bounds pass 6 wrote — none — and
        // refuses by name.
        assert_eq!(
            p.focus_move(FocusMove::Right),
            FocusMoveOutcome::Refused(FocusRefusal::NoGeometry { node: a }),
            "a geometry-less focus names the species"
        );
        assert_eq!(p.focus(), Some(a), "the refusal holds focus");
        assert!(
            p.focus_move(FocusMove::Next).moved(),
            "the tree-order door still walks"
        );
        assert_eq!(p.focus(), Some(b));
    }

    #[test]
    fn the_outcome_accessors_answer_the_species() {
        // M27, ADR-0033: the accessors the routing call sites ride —
        // `moved()` for the verdict, `landing()`/`refusal()` for the data.
        let moved = FocusMoveOutcome::Moved(FocusLanding {
            from: None,
            to: NodeId::allocate(),
            wrapped: false,
        });
        let refused = FocusMoveOutcome::Refused(FocusRefusal::NoCandidates);
        assert!(moved.moved());
        assert!(moved.landing().is_some());
        assert!(moved.refusal().is_none());
        assert!(!refused.moved());
        assert!(refused.landing().is_none());
        assert_eq!(refused.refusal(), Some(FocusRefusal::NoCandidates));
    }

    #[test]
    fn the_entry_is_never_a_wrap_and_the_single_candidate_stalls() {
        // M27, ADR-0033: the wrap flag is the crossing's data. The entry
        // landing (from no focus) never carries it; the single-candidate
        // walk's next stop is itself — the stall names its node.
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let a = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        let b = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        p.tick(viewport()).expect("first frame");

        let outcome = p.focus_move(FocusMove::Next);
        assert_eq!(
            outcome,
            FocusMoveOutcome::Moved(FocusLanding {
                from: None,
                to: a,
                wrapped: false,
            }),
            "the entry is a landing, not a wrap"
        );
        // Backward from the first candidate wraps to the last — the
        // crossing the old boolean could only hint at.
        let outcome = p.focus_move(FocusMove::Prev);
        assert_eq!(
            outcome,
            FocusMoveOutcome::Moved(FocusLanding {
                from: Some(a),
                to: b,
                wrapped: true,
            }),
            "backward past the first candidate is the wrap"
        );

        // One candidate: its next stop is itself — the stall, named.
        let mut solo = pipeline();
        let root = solo.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let only = solo.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        solo.tick(viewport()).expect("first frame");
        assert!(solo.focus_move(FocusMove::Next).moved(), "the entry lands");
        assert_eq!(
            solo.focus_move(FocusMove::Next),
            FocusMoveOutcome::Refused(FocusRefusal::AlreadyFocused { node: only }),
            "the single candidate's next stop is itself — named, not munged"
        );
    }

    #[test]
    fn update_focusable_recomputes_after_structure_changes() {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let _ = p.tick(viewport()).expect("first frame");
        assert!(p.focusables().is_empty(), "no focusable widgets yet");

        let button = p.add_child(
            root,
            Box::new(Rolied {
                size: Size::new(80.0, 20.0),
                role: Role::Button,
            }),
        );
        let _ = p.tick(viewport()).expect("structure frame");
        assert_eq!(
            p.focusables(),
            &[button],
            "the button is the only candidate"
        );
    }

    #[test]
    fn an_unavailable_control_is_no_focus_candidate_and_no_tab_stop() {
        // M22, ADR-0028: the availability tier joins the focusable
        // derivation — a control that must refuse is never a candidate,
        // and a request aimed at it is refused silently (the M18
        // species' rule, one predicate wider).
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let disabled = p.add_child(
            root,
            Box::new(Switchable {
                size: Size::new(80.0, 20.0),
                enabled: false,
                pressed: false,
            }),
        );
        let _ = p.tick(viewport()).expect("first frame");
        assert!(
            p.focusables().is_empty(),
            "the unavailable control is no candidate"
        );

        p.request_focus(disabled);
        let _ = p.tick(viewport()).expect("request frame");
        assert_eq!(p.focus(), None, "the request is refused");
    }

    #[test]
    fn focus_move_walks_wraps_and_refuses_the_empty_arena() {
        // M24, ADR-0030: the consumer door over the Tab walk. Three
        // buttons in tree order; the walk steps, wraps, and refuses the
        // way the funnel documents.
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let a = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        let b = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        let c = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        let _ = p.tick(viewport()).expect("first frame");

        assert!(p.focus().is_none(), "no walk yet, no focus");
        assert!(p.focus_move(FocusMove::Next).moved(), "the walk moves");
        assert_eq!(p.focus(), Some(a), "from no focus the walk takes the first");
        assert!(p.focus_move(FocusMove::Next).moved());
        assert_eq!(p.focus(), Some(b));
        assert!(p.focus_move(FocusMove::Next).moved());
        assert_eq!(p.focus(), Some(c), "the walk reaches the last candidate");
        assert!(
            p.focus_move(FocusMove::Next).moved(),
            "the walk always has a next stop while candidates exist"
        );
        assert_eq!(p.focus(), Some(a), "the walk wraps past the end");
        assert!(p.focus_move(FocusMove::Prev).moved());
        assert_eq!(
            p.focus(),
            Some(c),
            "backward from the first wraps to the last"
        );

        // The empty arena refuses honestly — no candidates, no masks, no
        // first-candidate reset (the M11 rule, at the focus seam).
        let mut empty = pipeline();
        assert!(
            !empty.focus_move(FocusMove::Next).moved(),
            "an arena without candidates refuses"
        );
        assert_eq!(empty.focus(), None, "a refused walk holds no focus");
    }

    #[test]
    fn focus_is_evicted_when_a_control_becomes_unavailable() {
        // The sentinel discipline one predicate wider: focus held by a
        // control that becomes unavailable does not survive the frame —
        // and the release repaints the focus ring (the live slot merges
        // FOCUS | PAINT, exactly like a move).
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let button = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        let _ = p.tick(viewport()).expect("first frame");
        p.request_focus(button);
        let _ = p.tick(viewport()).expect("focus frame");
        assert_eq!(p.focus(), Some(button));

        // The mutate seam: the availability write in place, the write's
        // own masks on the guard (the walk the m22 exit suite drives).
        let mut guard = p.arena_mut().mutate(button);
        let widget = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Switchable>()
            .expect("the node is the switchable");
        widget.enabled = false;
        drop(guard);
        let _ = p.tick(viewport()).expect("eviction frame");
        assert_eq!(p.focus(), None, "an unavailable control holds no focus");
    }

    #[test]
    fn the_availability_flag_rides_the_a11y_snapshot() {
        // M22, ADR-0028: pass 9 carries the flag the widget reports, so
        // the AT reads the truth without a second channel — and the
        // reserved live region stays available (it is never a control).
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let button = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        let _ = p.tick(viewport()).expect("first frame");
        let tree = p.a11y_snapshot().expect("a11y tree");
        let node = tree
            .nodes
            .iter()
            .find(|n| n.role == Role::Button)
            .expect("the button is in the snapshot");
        assert!(node.enabled, "an available control reports available");

        let mut guard = p.arena_mut().mutate(button);
        guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Switchable>()
            .expect("the node is the switchable")
            .enabled = false;
        drop(guard);
        p.arena_mut()
            .merge_mask(button, Change::PAINT.union(Change::A11Y));
        let _ = p.tick(viewport()).expect("availability frame");
        let tree = p.a11y_snapshot().expect("a11y tree");
        let node = tree
            .nodes
            .iter()
            .find(|n| n.role == Role::Button)
            .expect("the button is in the snapshot");
        assert!(!node.enabled, "the snapshot carries the refusal");
    }

    #[test]
    fn a_refused_press_costs_an_idle_frame() {
        // The M11 rule at the availability seam: the action is refused —
        // never munged — and the refusal queues no masks, so the frame
        // after the refused action runs nothing at all.
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let _button = p.add_child(
            root,
            Box::new(Switchable {
                size: Size::new(80.0, 20.0),
                enabled: false,
                pressed: false,
            }),
        );
        let _ = p.tick(viewport()).expect("first frame");

        let hash = {
            let tree = p.a11y_snapshot().expect("a11y tree");
            tree.nodes
                .iter()
                .find(|n| n.role == Role::Button)
                .expect("the button is in the snapshot")
                .id
        };
        assert!(
            !p.dispatch_a11y_action(hash, crate::widget::A11yAction::DefaultPress),
            "the press is refused"
        );
        let report = p.tick(viewport()).expect("the frame after the refusal");
        assert!(
            !report.paint_ran && !report.a11y_ran,
            "a refused action costs an idle frame"
        );
    }

    #[test]
    fn focus_resolves_requests_and_emits_one_diff() {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let a = p.add_child(
            root,
            Box::new(Rolied {
                size: Size::new(80.0, 20.0),
                role: Role::Button,
            }),
        );
        let b = p.add_child(
            root,
            Box::new(Rolied {
                size: Size::new(80.0, 20.0),
                role: Role::TextInput,
            }),
        );
        let _ = p.tick(viewport()).expect("first frame");

        p.request_focus(a);
        let report = p.tick(viewport()).expect("focus frame");
        assert_eq!(p.focus(), Some(a));
        assert!(report.focus_changed, "the move is reported once");

        p.request_focus(b);
        let report = p.tick(viewport()).expect("second focus frame");
        assert_eq!(p.focus(), Some(b));
        assert!(report.focus_changed);

        // Re-requesting the same node is a no-op, not a diff.
        p.request_focus(b);
        let report = p.tick(viewport()).expect("no-op focus frame");
        assert!(!report.focus_changed, "same-node focus request is silent");
    }

    #[test]
    fn focus_requests_for_non_focusable_or_dead_widgets_are_dropped() {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let button = p.add_child(
            root,
            Box::new(Rolied {
                size: Size::new(80.0, 20.0),
                role: Role::Button,
            }),
        );
        let _ = p.tick(viewport()).expect("first frame");

        p.request_focus(root);
        let _ = p.tick(viewport()).expect("drop frame");
        assert_eq!(p.focus(), None, "a container cannot take focus");

        p.request_focus(button);
        let _ = p.tick(viewport()).expect("focus frame");
        assert_eq!(p.focus(), Some(button));

        p.arena_mut().remove_subtree(button);
        let report = p.tick(viewport()).expect("removal frame");
        assert_eq!(p.focus(), None, "dead focus holders are dropped");
        assert!(report.focus_changed);
    }

    #[test]
    fn style_diffs_invalidate_layout_in_the_same_frame() {
        // A widget whose style lives in a shared cell: the mutation changes
        // `style()` without any explicit mask — pass 5 is the recalc that
        // must notice and invalidate.
        struct Styler {
            size: crate::geom::Size,
            style: std::sync::Arc<Mutex<Style>>,
        }

        impl Widget for Styler {
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn role(&self) -> Role {
                Role::Container
            }

            fn label(&self) -> LabelSource {
                LabelSource::text("styler")
            }

            fn style(&self) -> Style {
                self.style.lock().expect("style lock").clone()
            }

            fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
                Some(crate::style::MeasureSpec::Fixed(self.size))
            }
        }

        let _guard = crate::flux::batch_test_lock();
        let style = std::sync::Arc::new(Mutex::new(Style::default()));
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let child = p.add_child(
            root,
            Box::new(Styler {
                size: Size::new(80.0, 20.0),
                style: std::sync::Arc::clone(&style),
            }),
        );
        let _ = p.tick(viewport()).expect("first frame");
        let layout_runs_before = p.counters().layout_runs;

        // Mutate the style WITHOUT any explicit mask: the mutation goes
        // through the mutate pass, which feeds the node to pass 5's recalc —
        // the diff must invalidate layout in the same frame.
        let cell = std::sync::Arc::clone(&style);
        p.arena_mut().queue_mutation(
            child,
            Box::new(move |_| {
                cell.lock().expect("style lock").gap = 12.0;
            }),
        );
        let report = p.tick(viewport()).expect("style-diff frame");
        assert!(
            report.pass(PassKind::Style).is_some_and(|run| run.ran),
            "pass 5 saw the style diff"
        );
        assert!(
            report.layout_ran,
            "pass 5 turned the style diff into layout"
        );
        assert_eq!(
            p.counters().layout_runs,
            layout_runs_before + 1,
            "exactly one extra layout run"
        );
        let _ = child;
    }

    #[test]
    fn pass_overhead_stays_under_the_15_percent_gate_on_10k_node_churn() {
        // The §7.2 kill criterion, as a test: on the 10k-node churn workload
        // the pass-system bookkeeping must stay under 15% of frame time, or
        // the contract collapses to three passes. Median over 40 churn
        // frames keeps the assertion stable against scheduler noise.

        /// A container whose style lives in a shared cell (churn driver).
        struct ChurnBox {
            size: crate::geom::Size,
            style: std::sync::Arc<Mutex<Style>>,
        }

        impl Widget for ChurnBox {
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn role(&self) -> Role {
                Role::Container
            }

            fn label(&self) -> LabelSource {
                LabelSource::text("churn")
            }

            fn style(&self) -> Style {
                self.style.lock().expect("style lock").clone()
            }

            fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
                Some(crate::style::MeasureSpec::Fixed(self.size))
            }
        }

        const CONTAINERS: usize = 100;
        const CHILDREN: usize = 100;

        let _guard = crate::flux::batch_test_lock();
        let mut p = pipeline();
        let root = p.mount_root(Box::new(ChurnBox {
            size: Size::new(400.0, 800.0),
            style: std::sync::Arc::new(Mutex::new(Style::default())),
        }));

        // 100 containers × 100 children = 10,101 nodes.
        let mut churn_cells: Vec<(NodeId, std::sync::Arc<Mutex<Style>>)> = Vec::new();
        for _ in 0..CONTAINERS {
            let style = std::sync::Arc::new(Mutex::new(Style::default()));
            let container = p.add_child(
                root,
                Box::new(ChurnBox {
                    size: Size::new(200.0, 4.0),
                    style: std::sync::Arc::clone(&style),
                }),
            );
            churn_cells.push((container, style));
            for _ in 0..CHILDREN {
                let _leaf = p.add_child(
                    container,
                    Box::new(ChurnBox {
                        size: Size::new(2.0, 2.0),
                        style: std::sync::Arc::new(Mutex::new(Style::default())),
                    }),
                );
            }
        }
        assert_eq!(p.arena().len(), CONTAINERS * CHILDREN + CONTAINERS + 1);

        p.set_timings(true);
        let first = p.tick(viewport()).expect("first frame");
        assert!(
            first.total_nanos > 0,
            "timings are on: the frame is measured"
        );

        // Churn: each frame re-styles 100 containers through the mutate pass
        // — the realistic per-frame invalidation load (1% of the tree).
        let mut ratios = Vec::new();
        for frame in 0..60_u32 {
            let window = frame as usize % CONTAINERS;
            for (node, cell) in churn_cells.iter().cycle().skip(window).take(100) {
                let cell = std::sync::Arc::clone(cell);
                // +1 keeps the first churn frame distinct from the mounted
                // default (gap 0.0), so every measured frame invalidates.
                let gap = (crate::num::u32_to_f32(frame) + 1.0) * 0.1;
                p.arena_mut().queue_mutation(
                    *node,
                    Box::new(move |_| {
                        cell.lock().expect("style lock").gap = gap;
                    }),
                );
            }
            let report = p.tick(viewport()).expect("churn frame");
            assert!(report.layout_ran, "churn invalidates layout every frame");
            if frame >= 20 {
                ratios.push(report.pass_overhead_ratio());
            }
        }

        ratios.sort_by(f32::total_cmp);
        let median = ratios[ratios.len() / 2];
        assert!(
            median < 0.15,
            "M1 gate: pass-system overhead {median:.3} ≥ 15% on the 10k-node \
             churn workload — collapse to 3 passes (§7.2)"
        );
        assert!(median > 0.0, "the instrumentation measures something");
    }

    /// A scroll-area widget that records every action routed to it, refusing
    /// only `DefaultPress` (the vocabulary is the contract under test).
    struct Stepper {
        seen: std::sync::Arc<Mutex<Vec<crate::widget::A11yAction>>>,
    }

    impl Widget for Stepper {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn role(&self) -> Role {
            Role::ScrollArea
        }
        fn label(&self) -> LabelSource {
            LabelSource::text("stepper")
        }
        fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
            Some(crate::style::MeasureSpec::Fixed(Size::new(100.0, 30.0)))
        }
        fn on_a11y_action(&mut self, action: crate::widget::A11yAction) -> bool {
            self.seen.lock().expect("seen lock").push(action);
            true
        }
    }

    #[test]
    fn pass10_routes_the_full_a11y_action_vocabulary() {
        use crate::widget::{A11yAction, ScrollUnit};

        let _guard = crate::flux::batch_test_lock();
        let seen = std::sync::Arc::new(Mutex::new(Vec::new()));
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let _ = p.add_child(
            root,
            Box::new(Stepper {
                seen: std::sync::Arc::clone(&seen),
            }),
        );
        let _ = p.tick(viewport()).expect("first frame");

        let snapshot = p.a11y_snapshot().expect("a11y snapshot after first frame");
        let target = snapshot
            .nodes
            .iter()
            .find(|node| node.role == Role::ScrollArea)
            .map(|node| node.id)
            .expect("the stepper is in the a11y tree");

        let route =
            |p: &mut PassPipeline, action: A11yAction| p.dispatch_a11y_action(target, action);
        assert!(route(&mut p, A11yAction::Increment), "increment consumed");
        assert!(route(&mut p, A11yAction::Decrement), "decrement consumed");
        assert!(
            route(&mut p, A11yAction::ScrollUp(ScrollUnit::Item)),
            "scroll up by an item consumed"
        );
        assert!(
            route(&mut p, A11yAction::ScrollDown(ScrollUnit::Page)),
            "scroll down by a page consumed"
        );
        assert_eq!(
            seen.lock().expect("seen lock").as_slice(),
            [
                A11yAction::Increment,
                A11yAction::Decrement,
                A11yAction::ScrollUp(ScrollUnit::Item),
                A11yAction::ScrollDown(ScrollUnit::Page),
            ],
            "pass 10 delivers the action unchanged — routing, not munging"
        );

        // A consumed action invalidates through the one currency: the next
        // tick runs paint and a11y for the target, exactly like a user
        // interaction would (§7.2 pass 10).
        let report = p.tick(viewport()).expect("post-action frame");
        assert!(
            report.pass(PassKind::Paint).is_some_and(|run| run.ran),
            "a consumed action repaints its target"
        );
        assert!(
            report
                .pass(PassKind::Accessibility)
                .is_some_and(|run| run.ran),
            "a consumed action re-announces its target"
        );
    }

    #[test]
    fn pass10_reports_unconsumed_actions_without_masks() {
        use crate::widget::A11yAction;

        let _guard = crate::flux::batch_test_lock();
        let mut p = pipeline();
        let _root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let _ = p.tick(viewport()).expect("first frame");
        let target = p
            .a11y_snapshot()
            .expect("snapshot")
            .nodes
            .iter()
            .find(|node| node.role == Role::Container)
            .map(|node| node.id)
            .expect("the fixed root is in the a11y tree");

        // `Fixed` implements no actions: the route reports `false` and — the
        // M10 no-munging contract — leaves no masks behind.
        assert!(
            !p.dispatch_a11y_action(target, A11yAction::Increment),
            "an action the widget does not implement is not consumed"
        );
        let report = p.tick(viewport()).expect("idle frame");
        assert!(
            report.is_idle(),
            "an unconsumed action costs no pass work — it must never be \
             munged into a press or any other action"
        );
    }

    #[test]
    fn an_empty_announcement_refuses_and_queues_nothing() {
        // §11.6: an empty or all-whitespace announcement speaks nothing —
        // refused honestly, no masks, an idle frame.
        let _guard = crate::flux::batch_test_lock();
        let mut p = pipeline();
        let _root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        p.tick(viewport()).expect("first frame");
        let counters = p.counters();

        assert!(!p.announce("", crate::announce::Politeness::Polite));
        assert!(!p.announce("   \n\t ", crate::announce::Politeness::Polite));
        let report = p.tick(viewport()).expect("frame after refusals");
        assert!(report.is_idle(), "a refused announcement queues no masks");
        assert_eq!(p.counters(), counters, "no pass ran for a refusal");
        assert!(p.announcement().is_none(), "nothing was spoken");
    }

    #[test]
    fn an_announcement_before_a_root_refuses() {
        // There is no tree to attach the region to — the refusal is the
        // honest answer, never a panic and never a deferred queue.
        let _guard = crate::flux::batch_test_lock();
        let mut p = pipeline();
        assert!(!p.announce("Saved", crate::announce::Politeness::Polite));
        assert!(p.announcement().is_none());
    }

    #[test]
    fn an_announcement_rides_the_a11y_delta_alone() {
        // The reserved region is the root's last child from the first
        // frame; announcing re-emits exactly that node — no shape change,
        // no other node in the delta (A1: the same currency, nothing else).
        let _guard = crate::flux::batch_test_lock();
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        p.tick(viewport()).expect("first frame");

        // The region is present from the start, named empty, polite.
        let tree = p.a11y_snapshot().expect("snapshot");
        let root_node = tree.by_id(tree.root.expect("root id")).expect("root node");
        assert_eq!(
            root_node.children.last().copied(),
            Some(crate::announce::ANNOUNCEMENT_NODE_HASH),
            "the reserved region is the root's last child"
        );
        let region = tree
            .by_id(crate::announce::ANNOUNCEMENT_NODE_HASH)
            .expect("the region rides the snapshot from the first frame");
        assert_eq!(region.role, Role::LiveRegion);
        assert_eq!(region.label, "", "nothing has been spoken yet");
        assert_eq!(region.live, Some(crate::announce::Politeness::Polite));
        let root_hash = tree.root.expect("root hash");

        assert!(p.announce("Saved", crate::announce::Politeness::Polite));
        assert_eq!(
            p.announcement().map(|a| a.text.as_str()),
            Some("Saved"),
            "the introspection read answers what the region carries"
        );
        let report = p.tick(viewport()).expect("the announcement's frame");
        assert!(report.a11y_ran, "the announcement is the pass's trigger");
        assert!(
            !report.layout_ran && !report.paint_ran,
            "only the a11y pass runs"
        );

        let recording = recording_of(&mut p);
        assert!(
            recording
                .last_delta
                .updated
                .contains(&crate::announce::ANNOUNCEMENT_NODE_HASH)
        );
        assert_eq!(
            recording.last_delta.updated.len(),
            1,
            "the announcement's delta re-emits the region and nothing else"
        );
        assert!(recording.last_delta.removed.is_empty(), "no ghost removals");
        assert_eq!(
            recording.last_delta.focus, None,
            "focus never moves for a speech act"
        );
        assert_eq!(recording.updates, 2, "one update per a11y pass run");

        let region = recording
            .tree
            .as_ref()
            .and_then(|tree| tree.by_id(crate::announce::ANNOUNCEMENT_NODE_HASH))
            .expect("the region in the applied snapshot");
        assert_eq!(region.label, "Saved");
        let _ = root;
        let _ = root_hash;
    }

    #[test]
    fn rapid_announcements_replace_last_write_wins() {
        // One live region, one batch, one spoken fact: the second
        // announcement replaces the first before the tick — never an
        // unbounded queue inside a frame (§11.6's replace rule).
        let _guard = crate::flux::batch_test_lock();
        let mut p = pipeline();
        let _root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        p.tick(viewport()).expect("first frame");

        assert!(p.announce("Saved", crate::announce::Politeness::Polite));
        assert!(p.announce("Connection lost", crate::announce::Politeness::Assertive));
        let report = p.tick(viewport()).expect("one frame for both");
        assert!(report.a11y_ran);
        assert_eq!(p.counters().a11y_runs, 2, "one pass run, not one per call");

        let recording = recording_of(&mut p);
        assert_eq!(recording.updates, 2, "the two calls ride one update");
        let region = recording
            .tree
            .as_ref()
            .and_then(|tree| tree.by_id(crate::announce::ANNOUNCEMENT_NODE_HASH))
            .expect("the region");
        assert_eq!(region.label, "Connection lost", "last write wins");
        assert_eq!(region.live, Some(crate::announce::Politeness::Assertive));
    }

    // ── The pointer capture (§10.1, ADR-0025) ─────────────────────────────

    /// A fixed panel that records the tag of every routed pointer event —
    /// the probe the capture contract's tests read.
    struct Probe {
        size: crate::geom::Size,
        log: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>,
    }

    impl Widget for Probe {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn role(&self) -> Role {
            Role::Container
        }

        fn label(&self) -> LabelSource {
            LabelSource::text("probe")
        }

        fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
            Some(crate::style::MeasureSpec::Fixed(self.size))
        }

        fn on_pointer_event(
            &mut self,
            _ctx: &mut crate::events::EventCtx,
            event: &PointerEvent,
        ) -> EventResponse {
            let tag = match event {
                PointerEvent::Down { .. } => "down",
                PointerEvent::Up { .. } => "up",
                PointerEvent::Move { .. } => "move",
                PointerEvent::Leave => "leave",
                PointerEvent::Wheel { .. } => "wheel",
            };
            self.log.borrow_mut().push(tag);
            EventResponse::Handled
        }
    }

    /// A root filling the viewport with a 100×60 probe at the origin —
    /// the press lands inside the probe, the drag wanders past it.
    fn probe_pipeline() -> (
        PassPipeline,
        NodeId,
        std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>,
    ) {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 400.0),
        }));
        let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let probe = p.add_child(
            root,
            Box::new(Probe {
                size: Size::new(100.0, 60.0),
                log: std::rc::Rc::clone(&log),
            }),
        );
        p.tick(viewport()).expect("first frame");
        (p, probe, log)
    }

    #[test]
    fn a_captured_drag_keeps_routing_outside_the_target() {
        let (mut p, _probe, log) = probe_pipeline();
        // Press inside the probe's frame (100×60 at the origin).
        p.dispatch_pointer(&PointerEvent::Down {
            position: Point::new(50.0, 30.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        });
        assert!(p.captured().is_some(), "the press holds its target");
        // The drag wanders far below the probe — the stream still routes.
        p.dispatch_pointer(&PointerEvent::Move {
            position: Point::new(50.0, 380.0),
        });
        p.dispatch_pointer(&PointerEvent::Up {
            position: Point::new(50.0, 380.0),
            button: crate::events::MouseButton::Left,
        });
        assert_eq!(
            log.borrow().as_slice(),
            ["down", "move", "up"],
            "the captured stream rides the target's chain, wherever the pointer is"
        );
        assert!(p.captured().is_none(), "the release ends the capture");
    }

    #[test]
    fn the_release_returns_the_pointer_to_the_hit_test() {
        let (mut p, _probe, log) = probe_pipeline();
        p.dispatch_pointer(&PointerEvent::Down {
            position: Point::new(50.0, 30.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        });
        p.dispatch_pointer(&PointerEvent::Up {
            position: Point::new(50.0, 30.0),
            button: crate::events::MouseButton::Left,
        });
        // After the release, a move past the probe routes by the hit test
        // again — the old press holds nothing.
        p.dispatch_pointer(&PointerEvent::Move {
            position: Point::new(50.0, 380.0),
        });
        assert_eq!(
            log.borrow().as_slice(),
            ["down", "up"],
            "the uncaptured move follows the hover, not the finished press"
        );
        // One slot, last press wins: a second press re-captures.
        p.dispatch_pointer(&PointerEvent::Down {
            position: Point::new(50.0, 30.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        });
        assert!(
            p.captured().is_some(),
            "the second press holds its target again"
        );
    }

    #[test]
    fn leave_ends_the_capture_and_reaches_the_target() {
        let (mut p, _probe, log) = probe_pipeline();
        p.dispatch_pointer(&PointerEvent::Down {
            position: Point::new(50.0, 30.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        });
        // Leave carries no position: without a capture it routes nowhere;
        // with one it rides the captured chain and ends the gesture.
        p.dispatch_pointer(&PointerEvent::Leave);
        assert_eq!(
            log.borrow().as_slice(),
            ["down", "leave"],
            "the captured target sees the leave"
        );
        assert!(p.captured().is_none(), "the leave releases the pointer");
    }

    #[test]
    fn a_wheel_follows_the_hover_not_the_capture() {
        let (mut p, _probe, log) = probe_pipeline();
        p.dispatch_pointer(&PointerEvent::Down {
            position: Point::new(50.0, 30.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        });
        p.dispatch_pointer(&PointerEvent::Wheel {
            delta: Vector::new(0.0, -12.0),
            position: Point::new(50.0, 380.0),
        });
        assert_eq!(
            log.borrow().as_slice(),
            ["down"],
            "the wheel stays hit-tested: it scrolls what is under the pointer"
        );
    }

    #[test]
    fn a_dead_target_releases_the_pointer() {
        let (mut p, probe, _log) = probe_pipeline();
        p.dispatch_pointer(&PointerEvent::Down {
            position: Point::new(50.0, 30.0),
            button: crate::events::MouseButton::Left,
            click_count: 1,
        });
        assert!(p.captured().is_some());
        // The subtree leaves the arena mid-gesture: the capture releases
        // and the pointer routes by the hit test again — no panic, no
        // ghost route.
        p.arena_mut().remove_subtree(probe);
        p.dispatch_pointer(&PointerEvent::Move {
            position: Point::new(50.0, 380.0),
        });
        assert!(
            p.captured().is_none(),
            "the dead target cannot hold the pointer"
        );
    }

    #[test]
    fn an_uncaptured_storm_routes_by_the_hit_test_alone() {
        let (mut p, _probe, log) = probe_pipeline();
        // Fifty moves across and past the probe's frame: the hit test
        // routes exactly the ones inside it — the capture adds no route
        // of its own while no press is in flight.
        for i in 0..50 {
            p.dispatch_pointer(&PointerEvent::Move {
                position: Point::new(50.0, 30.0 + crate::num::usize_to_f32(i)),
            });
        }
        assert_eq!(log.borrow().len(), 30, "the moves inside the frame only");
        assert!(
            log.borrow().iter().all(|tag| *tag == "move"),
            "moves only — no synthetic down or up"
        );
        assert_eq!(p.counters().events_routed, 50, "every event was routed");
        assert!(p.captured().is_none(), "no press, no capture");
    }

    /// A focusable fixed-size probe: `Role::Button` joins the default
    /// `focusable()` derivation (M22), the fixed spec keeps the walk's
    /// geometry deterministic.
    struct FocusProbe {
        size: crate::geom::Size,
    }

    impl Widget for FocusProbe {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn role(&self) -> Role {
            Role::Button
        }

        fn label(&self) -> LabelSource {
            LabelSource::text("stop")
        }

        fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
            Some(crate::style::MeasureSpec::Fixed(self.size))
        }
    }

    /// A layout engine that writes no bounds — the pre-layout state the
    /// directional walk's `NoGeometry` refusal reads (M27, ADR-0033).
    struct NoLayout;

    impl LayoutEngine for NoLayout {
        fn sync_and_layout(
            &mut self,
            _arena: &mut Arena,
            _root: NodeId,
            _dirty: &FxHashMap<NodeId, Change>,
            _viewport: Size,
        ) -> Result<(), LayoutError> {
            Ok(())
        }

        fn measure_calls(&self) -> u64 {
            0
        }

        fn reset_measure_calls(&mut self) {}
    }

    /// A layout engine that lays the root's children out in one row —
    /// deterministic two-axis geometry for the directional walk's tests,
    /// zero Taffy.
    struct RowLayout;

    impl LayoutEngine for RowLayout {
        fn sync_and_layout(
            &mut self,
            arena: &mut Arena,
            root: NodeId,
            _dirty: &FxHashMap<NodeId, Change>,
            _viewport: Size,
        ) -> Result<(), LayoutError> {
            let mut x = 0.0;
            for child in arena.children(root).to_vec() {
                let (w, h) = match arena.widget(child).measure_spec() {
                    Some(crate::style::MeasureSpec::Fixed(size)) => (size.width, size.height),
                    _ => (40.0, 20.0),
                };
                arena.update_bounds(child, Rect::new(Point::new(x, 0.0), Size::new(w, h)));
                x += w;
            }
            arena.update_bounds(root, Rect::new(Point::zero(), Size::new(x, 20.0)));
            Ok(())
        }

        fn measure_calls(&self) -> u64 {
            0
        }

        fn reset_measure_calls(&mut self) {}
    }

    /// A pipeline over the row layout: three 60-wide stops at x = 0, 60,
    /// 120 — one axis, three candidates.
    fn row_pipeline() -> PassPipeline {
        PassPipeline::new(
            Flux::new(),
            Box::new(RowLayout),
            Box::new(MemoryJournal::default()),
            Box::new(RecordingA11y::default()),
        )
    }

    /// A fixed-size box that paints one quad of its own color and carries
    /// a compose state — the compose tier's test rig: the parent composes
    /// over its subtree, the child answers the walk.
    struct QuadBox {
        size: crate::geom::Size,
        color: Color,
        compose: crate::scene::Compose,
    }

    impl QuadBox {
        fn new(size: crate::geom::Size, color: Color) -> Self {
            Self {
                size,
                color,
                compose: crate::scene::Compose::IDENTITY,
            }
        }

        fn with_compose(mut self, compose: crate::scene::Compose) -> Self {
            self.compose = compose;
            self
        }
    }

    impl Widget for QuadBox {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn role(&self) -> Role {
            Role::Container
        }
        fn label(&self) -> LabelSource {
            LabelSource::text("quad box")
        }
        fn style(&self) -> Style {
            Style {
                size: crate::style::Size2 {
                    width: Some(self.size.width),
                    height: Some(self.size.height),
                },
                ..Style::default()
            }
        }
        fn compose(&self) -> crate::scene::Compose {
            self.compose
        }
        fn paint(&self, ctx: &mut crate::scene::PaintCtx<'_>) {
            ctx.push_quad(Quad::filled(
                Rect::new(Point::zero(), ctx.size()),
                self.color,
            ));
        }
        fn measure_spec(&self) -> Option<crate::style::MeasureSpec> {
            Some(crate::style::MeasureSpec::Fixed(self.size))
        }
    }

    /// The compose rig: a root, a composing parent, and a child under it —
    /// the snapshot's first quad is the parent's (blue), the second the
    /// child's (red), pre-order paint order. `StackLayout` puts the parent
    /// at (0, 0) and the child stacked below it.
    fn compose_tree(
        parent: crate::scene::Compose,
        child: crate::scene::Compose,
    ) -> crate::scene::SceneSnapshot {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 400.0),
        }));
        let box_node = p.add_child(
            root,
            Box::new(
                QuadBox::new(Size::new(30.0, 10.0), Color::rgb(0, 0, 255)).with_compose(parent),
            ),
        );
        let _ = p.add_child(
            box_node,
            Box::new(
                QuadBox::new(Size::new(30.0, 10.0), Color::rgb(255, 0, 0)).with_compose(child),
            ),
        );
        let report = p.tick(viewport()).expect("the compose frame");
        (*report.scene).clone()
    }

    fn quad_at(scene: &crate::scene::SceneSnapshot, index: usize) -> Quad {
        match scene.primitives.get(index) {
            Some(crate::scene::Primitive::Quad(quad)) => *quad,
            Some(other) => panic!("expected a quad at {index}, got {other:?}"),
            None => panic!("primitive {index} absent"),
        }
    }

    #[test]
    fn compose_fade_propagates_from_the_parent_to_the_child() {
        let scene = compose_tree(
            crate::scene::Compose::faded(0.5),
            crate::scene::Compose::IDENTITY,
        );
        let parent = quad_at(&scene, 0);
        assert_eq!(
            parent.color.a, 128,
            "the parent's own quad paints at its fade (255 × 0.5)"
        );
        let child = quad_at(&scene, 1);
        assert_eq!(
            child.color.a, 128,
            "the child inherits the parent's fade through the compose walk"
        );
        assert_eq!(
            (child.color.r, child.color.g, child.color.b),
            (255, 0, 0),
            "a fade is an alpha fact, never a color rewrite"
        );
    }

    #[test]
    fn nested_fades_multiply_down_the_tree() {
        let scene = compose_tree(
            crate::scene::Compose::faded(0.5),
            crate::scene::Compose::faded(0.5),
        );
        let child = quad_at(&scene, 1);
        assert_eq!(
            child.color.a, 64,
            "0.5 × 0.5 folds to 0.25 (255 × 0.25, rounded once) — the CSS rule"
        );
    }

    #[test]
    fn compose_translation_propagates_and_adds() {
        let scene = compose_tree(
            crate::scene::Compose::shifted(Vector::new(10.0, 0.0)),
            crate::scene::Compose::shifted(Vector::new(5.0, 0.0)),
        );
        let parent = quad_at(&scene, 0);
        assert_eq!(
            parent.rect.origin,
            Point::new(10.0, 0.0),
            "the parent's own quad rides its shift"
        );
        let child = quad_at(&scene, 1);
        assert_eq!(
            child.rect.origin,
            Point::new(15.0, 0.0),
            "the child rides its own shift beside the parent's (additive) — the sole child stacks at the parent's origin"
        );
        assert_eq!(child.color.a, 255, "a shift is not a fade");
    }

    #[test]
    fn identity_compose_replays_the_baseline_scene() {
        let scene = compose_tree(
            crate::scene::Compose::IDENTITY,
            crate::scene::Compose::IDENTITY,
        );
        let parent = quad_at(&scene, 0);
        assert_eq!(parent.color.a, 255, "nobody composed: full alpha");
        assert_eq!(parent.rect.origin, Point::new(0.0, 0.0));
        let child = quad_at(&scene, 1);
        assert_eq!(child.color.a, 255);
        assert_eq!(
            child.rect.origin,
            Point::new(0.0, 0.0),
            "the sole child's stacked position, compose untouched"
        );
    }

    fn mount_row(p: &mut PassPipeline) -> Vec<NodeId> {
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(180.0, 20.0),
        }));
        let stops: Vec<NodeId> = (0..3)
            .map(|_| {
                p.add_child(
                    root,
                    Box::new(FocusProbe {
                        size: Size::new(60.0, 20.0),
                    }),
                )
            })
            .collect();
        p.tick(viewport()).expect("the row lays out");
        stops
    }

    #[test]
    fn the_directional_walk_selects_what_the_geometry_names() {
        let mut p = row_pipeline();
        let stops = mount_row(&mut p);

        // The entry contract is direction-agnostic: from no focus the walk
        // rests on the first candidate, whichever direction was named.
        assert!(p.focus_move(FocusMove::Right).moved());
        assert_eq!(p.focus(), Some(stops[0]));

        // `Right` lands the stop fully beyond the focused node's right
        // edge — the geometry's answer, not the tree's next call for it.
        assert!(p.focus_move(FocusMove::Right).moved());
        assert_eq!(p.focus(), Some(stops[1]));
        assert!(p.focus_move(FocusMove::Right).moved());
        assert_eq!(p.focus(), Some(stops[2]));
        assert!(p.focus_move(FocusMove::Left).moved());
        assert_eq!(p.focus(), Some(stops[1]), "left reverses right");

        // Up and Down have no candidate in a single row: the bounds all
        // overlap the origin's vertical span, nothing lies fully below or
        // above — the honest refusal, named on the seam's target.
        assert!(!p.focus_move(FocusMove::Down).moved());
        assert_eq!(p.focus(), Some(stops[1]), "a refusal never moves focus");
        assert!(!p.focus_move(FocusMove::Up).moved());
        assert_eq!(p.focus(), Some(stops[1]));

        // The right edge refuses — space does not wrap — while the
        // tree-order funnel still wraps from the same spot.
        assert!(p.focus_move(FocusMove::Right).moved());
        assert_eq!(p.focus(), Some(stops[2]));
        assert!(
            !p.focus_move(FocusMove::Right).moved(),
            "the edge is the walk's end"
        );
        assert!(
            p.focus_move(FocusMove::Next).moved(),
            "Next wraps; Right never does"
        );
        assert_eq!(p.focus(), Some(stops[0]));
    }

    #[test]
    fn the_entry_contract_lands_the_first_candidate_from_outside_the_list() {
        // ADR-0031's documented contract: from a focus outside the
        // filtered candidate list, the walk takes the list's first
        // candidate — both directions. The M25-era arithmetic landed the
        // list's second candidate forward and its last backward; the cut
        // repairs the drift and the test pins the document.
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(180.0, 40.0),
        }));
        let group = p.add_child(
            root,
            Box::new(Fixed {
                size: Size::new(120.0, 40.0),
            }),
        );
        let first = p.add_child(
            group,
            Box::new(FocusProbe {
                size: Size::new(60.0, 20.0),
            }),
        );
        let second = p.add_child(
            group,
            Box::new(FocusProbe {
                size: Size::new(60.0, 20.0),
            }),
        );
        let outside = p.add_child(
            root,
            Box::new(FocusProbe {
                size: Size::new(60.0, 20.0),
            }),
        );
        p.tick(viewport()).expect("the tree lays out");

        // The unscoped walk rests on the outside probe (the tree's third
        // candidate — the group itself is not focusable).
        assert!(p.focus_move(FocusMove::Next).moved());
        assert!(p.focus_move(FocusMove::Next).moved());
        assert!(p.focus_move(FocusMove::Next).moved());
        assert_eq!(p.focus(), Some(outside));

        // The scoped door enters at the trap's first candidate — forward.
        assert!(p.focus_move_in(group, FocusMove::Next).moved());
        assert_eq!(
            p.focus(),
            Some(first),
            "from a focus outside the scope, forward takes the scope's first candidate"
        );

        // And backward from outside takes the same first candidate — the
        // walk enters the list, it does not approach it from its end.
        assert!(p.focus_move_in(group, FocusMove::Prev).moved());
        assert_eq!(
            p.focus(),
            Some(second),
            "from the first candidate, backward wraps to the scope's last — the in-list rule"
        );
        // The unscoped funnel still answers the whole tree: from the
        // scope's last candidate, Next walks past the group to the
        // outside probe.
        assert!(p.focus_move(FocusMove::Next).moved());
        assert_eq!(
            p.focus(),
            Some(outside),
            "the unscoped walk leaves the group"
        );
        assert!(p.focus_move_in(group, FocusMove::Prev).moved());
        assert_eq!(
            p.focus(),
            Some(first),
            "from a focus outside the scope, backward takes the scope's first candidate too"
        );
    }

    #[test]
    fn a_directional_move_rides_the_same_currency() {
        // One directional move queues the FOCUS | PAINT masks on the two
        // endpoints; the next frame repaints the pair and the frame after
        // is idle (A7) — the directional walk buys nothing new.
        let mut p = row_pipeline();
        let stops = mount_row(&mut p);
        assert!(p.focus_move(FocusMove::Right).moved());
        assert!(p.focus().is_some(), "the walk resolves immediately");
        assert_eq!(
            p.focus(),
            Some(stops[0]),
            "the entry contract, directional form"
        );
        let frame = p.tick(viewport()).expect("the frame lays out");
        assert!(!frame.is_idle(), "the move's masks cost one frame");
        let after = p.tick(viewport()).expect("the frame lays out");
        assert!(after.is_idle(), "the frame after the move is idle again");
    }

    // ------------------------------------------------------------------
    // The inspector's tree dump (ADR-0010's dump seam): the observer
    // that opted in receives the resolved tree the a11y pass answered —
    // the same Arc the sink saw, retained by the devtools that need the
    // whole shape. An absent observer prices nothing: one branch when
    // none (the A7 idle contract, the fact-forwarding helpers' own
    // shape), and a skipped a11y pass forwards nothing (the tree door
    // answers what the pass built, never a guess past it).
    // ------------------------------------------------------------------

    /// The dump seam's catcher: retains every tree the pipeline hands
    /// it beside the fact count it observed — the door a devtools
    /// recorder overrides, the events it keeps counting.
    struct TreeCatcher {
        trees: std::rc::Rc<std::cell::RefCell<Vec<std::sync::Arc<TreeSnapshot>>>>,
        facts: std::rc::Rc<std::cell::Cell<u64>>,
    }

    impl InspectObserver for TreeCatcher {
        fn observe(&mut self, _event: InspectEvent) {
            self.facts.set(self.facts.get() + 1);
        }

        fn tree(&mut self, snapshot: std::sync::Arc<TreeSnapshot>) {
            self.trees.borrow_mut().push(snapshot);
        }
    }

    #[test]
    fn the_observers_tree_door_answers_the_resolved_snapshot() {
        let mut p = pipeline();
        let root = p.mount_root(Box::new(Fixed {
            size: Size::new(200.0, 100.0),
        }));
        let _toggled = p.add_child(root, Box::new(Switchable::new(Size::new(80.0, 20.0))));
        let trees = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let facts = std::rc::Rc::new(std::cell::Cell::new(0_u64));
        p.set_observer(Box::new(TreeCatcher {
            trees: std::rc::Rc::clone(&trees),
            facts: std::rc::Rc::clone(&facts),
        }));
        p.tick(viewport()).expect("the first frame builds the tree");
        let caught = trees.borrow();
        assert_eq!(
            caught.len(),
            1,
            "one a11y run, one tree door — the pass's own count"
        );
        // The door hands the snapshot the pass answered — the same one
        // the sink saw, byte-for-byte (one Arc, zero copies, no munge).
        let sink_tree = recording_of(&mut p).tree.clone().expect("the sink saw it");
        assert_eq!(
            &**caught.first().expect("one tree"),
            &sink_tree,
            "the door's answer is the sink's answer"
        );
        assert!(
            caught
                .first()
                .expect("one tree")
                .nodes
                .iter()
                .any(|node| node.role == Role::Button),
            "the mounted control rides the dump's shape"
        );
        let facts = facts.get();
        assert!(
            facts > 0,
            "the fact stream keeps counting what the events carry"
        );

        // An idle frame rebuilds nothing: the pass skips, the door
        // stays silent, the observer's retained tree is the last one.
        let _ = p.tick(viewport()).expect("the idle frame");
        assert_eq!(
            trees.borrow().len(),
            1,
            "a skipped a11y pass answers no tree door"
        );
    }

    // ------------------------------------------------------------------
    // M146 (ADR-0150): the point ask's geometry query. The snapshot's
    // query vocabulary (`find`, `by_id`, `walk`) names, addresses, and
    // walks; the one door it lacked locates — which node does this
    // point touch? The land rule is the paint order's own (the last
    // pre-order node containing the point wins); every miss is a named
    // species, never a collapsed `false`.
    // ------------------------------------------------------------------

    /// Builds one node with the full field list — the honest
    /// explicitness the tree node's own review demands (the dump
    /// helper's rule, at the core tier).
    fn hit_node(
        id: u64,
        role: Role,
        label: &str,
        bounds: Option<Rect>,
        children: Vec<u64>,
    ) -> super::TreeNode {
        super::TreeNode {
            id,
            role,
            label: label.to_string(),
            bounds,
            children,
            value: None,
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
            scroll_orientation: None,
            scroll_position: None,
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
    fn hit_at_lands_the_innermost_node_with_the_ancestry_it_descended() {
        let snapshot = TreeSnapshot {
            nodes: vec![
                hit_node(
                    1,
                    Role::Container,
                    "window",
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(800.0, 600.0))),
                    vec![2],
                ),
                hit_node(
                    2,
                    Role::Container,
                    "panel",
                    Some(Rect::new(Point::new(10.0, 10.0), Size::new(300.0, 200.0))),
                    vec![3],
                ),
                hit_node(
                    3,
                    Role::Button,
                    "Save",
                    Some(Rect::new(Point::new(20.0, 20.0), Size::new(80.0, 24.0))),
                    Vec::new(),
                ),
            ],
            root: Some(1),
        };
        let super::HitAnswer::Landed { node, ancestry } = snapshot.hit_at(Point::new(30.0, 30.0))
        else {
            panic!("the point sits on the button — the walk landed");
        };
        assert_eq!(node.id, 3, "the innermost node the point touches");
        assert_eq!(node.label, "Save", "the node the walk names");
        let ids: Vec<u64> = ancestry.iter().map(|n| n.id).collect();
        assert_eq!(
            ids,
            vec![1, 2, 3],
            "the ancestry root-first, the hit node last"
        );
    }

    #[test]
    fn hit_at_names_the_last_preorder_node_where_siblings_overlap() {
        // Two sibling buttons sharing the same geometry: the later one
        // paints on top (the tree walk order is the paint order), so
        // the point in the overlap names the last one in pre-order.
        let snapshot = TreeSnapshot {
            nodes: vec![
                hit_node(
                    1,
                    Role::Container,
                    "window",
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(800.0, 600.0))),
                    vec![2, 3],
                ),
                hit_node(
                    2,
                    Role::Button,
                    "under",
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 40.0))),
                    Vec::new(),
                ),
                hit_node(
                    3,
                    Role::Button,
                    "over",
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 40.0))),
                    Vec::new(),
                ),
            ],
            root: Some(1),
        };
        let super::HitAnswer::Landed { node, .. } = snapshot.hit_at(Point::new(10.0, 10.0)) else {
            panic!("the point sits in the overlap — the walk landed");
        };
        assert_eq!(
            node.id, 3,
            "the last pre-order node wins — the paint order's own rule"
        );
    }

    #[test]
    fn hit_at_misses_outside_the_geometry_it_stands_with() {
        let snapshot = TreeSnapshot {
            nodes: vec![hit_node(
                1,
                Role::Container,
                "window",
                Some(Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 100.0))),
                Vec::new(),
            )],
            root: Some(1),
        };
        assert_eq!(
            snapshot.hit_at(Point::new(500.0, 500.0)),
            super::HitAnswer::Missed(super::HitMiss::Outside),
            "geometry stands, the point touches nothing — the species names it"
        );
    }

    #[test]
    fn hit_at_refuses_the_geometry_less_tree_as_no_geometry() {
        // The tree stands, the layout pass never wrote bounds — the
        // focus walk's own NoGeometry species, at the snapshot tier.
        let snapshot = TreeSnapshot {
            nodes: vec![
                hit_node(1, Role::Container, "window", None, vec![2]),
                hit_node(2, Role::Button, "Save", None, Vec::new()),
            ],
            root: Some(1),
        };
        assert_eq!(
            snapshot.hit_at(Point::new(10.0, 10.0)),
            super::HitAnswer::Missed(super::HitMiss::NoGeometry),
            "no node carries bounds — the honest refusal, never a guessed miss"
        );
    }

    #[test]
    fn hit_at_refuses_the_absent_tree_as_no_tree() {
        assert_eq!(
            TreeSnapshot::default().hit_at(Point::new(1.0, 1.0)),
            super::HitAnswer::Missed(super::HitMiss::NoTree),
            "no tree resolved yet — the honest absence"
        );
    }

    #[test]
    fn hit_at_refuses_the_nan_point_before_it_corrupts() {
        // A non-finite ask names its species before the comparisons
        // run: euclid's containment would munge NaN into a plain
        // `outside` miss (every comparison false), and the munged
        // answer reads as a clean geometry verdict. The scroll tier's
        // own rule — the NaN refused before it corrupts.
        let snapshot = TreeSnapshot {
            nodes: vec![hit_node(
                1,
                Role::Button,
                "Save",
                Some(Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 20.0))),
                Vec::new(),
            )],
            root: Some(1),
        };
        assert_eq!(
            snapshot.hit_at(Point::new(f32::NAN, 10.0)),
            super::HitAnswer::Missed(super::HitMiss::Malformed),
            "the NaN ask names its malformation, never a munged miss"
        );
        assert_eq!(
            snapshot.hit_at(Point::new(10.0, f32::INFINITY)),
            super::HitAnswer::Missed(super::HitMiss::Malformed),
            "an infinite coordinate is the same malformed ask"
        );
    }

    #[test]
    fn hit_miss_names_the_species_the_wire_speaks() {
        // The wire's tag vocabulary is `snake_case`; the species' name
        // door is the one word the protocol document and the answer
        // both speak (the focus refusal's naming shape).
        for (species, word) in [
            (super::HitMiss::NoTree, "no_tree"),
            (super::HitMiss::NoGeometry, "no_geometry"),
            (super::HitMiss::Outside, "outside"),
            (super::HitMiss::Malformed, "malformed"),
        ] {
            assert_eq!(species.name(), word, "the species' wire word");
        }
    }

    // ------------------------------------------------------------------
    // M147 (ADR-0151): the word ask's query. The snapshot's query
    // vocabulary grows the role-optional door beside `find`/`find_all`
    // — one substring funnel, the role narrowing it when named — and
    // the Role vocabulary grows the inverse name door, the wire's word
    // parsing beside `name`'s (one vocabulary, two doors, the two
    // unable to drift apart).
    // ------------------------------------------------------------------

    #[test]
    fn search_answers_the_case_insensitive_needle_in_tree_order() {
        let snapshot = TreeSnapshot {
            nodes: vec![
                hit_node(1, Role::Container, "window", None, vec![2, 3]),
                hit_node(2, Role::Button, "Save", None, Vec::new()),
                hit_node(3, Role::Label, "save the doc", None, Vec::new()),
            ],
            root: Some(1),
        };
        let ids: Vec<u64> = snapshot
            .search("save", Some(Role::Button))
            .iter()
            .map(|n| n.id)
            .collect();
        assert_eq!(ids, vec![2], "the role narrows the needle's matches");
        let ids: Vec<u64> = snapshot.search("SAVE", None).iter().map(|n| n.id).collect();
        assert_eq!(
            ids,
            vec![2, 3],
            "the needle is case-insensitive, the matches in tree order"
        );
    }

    #[test]
    fn search_without_a_role_answers_the_whole_vocabulary() {
        let snapshot = TreeSnapshot {
            nodes: vec![
                hit_node(1, Role::Container, "window", None, vec![2]),
                hit_node(2, Role::Button, "Save", None, Vec::new()),
            ],
            root: Some(1),
        };
        // The empty needle names every resolved name — the ask's honest
        // every-node answer, not a refusal (the replay burst's empty
        // answer is an absence, never an error).
        let count = snapshot.search("", None).len();
        assert_eq!(count, 2, "the whole vocabulary rides the empty needle");
    }

    #[test]
    fn find_all_rides_the_same_substring_funnel_search_carries() {
        // One funnel, two doors: the plural query the harness rides and
        // the word ask's role-optional door answer the same matches —
        // the two unable to drift apart.
        let snapshot = TreeSnapshot {
            nodes: vec![
                hit_node(1, Role::Container, "window", None, vec![2, 3]),
                hit_node(2, Role::Button, "Save", None, Vec::new()),
                hit_node(3, Role::Button, "save as", None, Vec::new()),
            ],
            root: Some(1),
        };
        let funnel: Vec<u64> = snapshot
            .find_all(Role::Button, "save")
            .iter()
            .map(|n| n.id)
            .collect();
        let door: Vec<u64> = snapshot
            .search("save", Some(Role::Button))
            .iter()
            .map(|n| n.id)
            .collect();
        assert_eq!(funnel, door, "one substring funnel, two doors");
    }

    #[test]
    fn role_from_name_round_trips_every_role_word() {
        // One vocabulary, two doors: every role's word parses back to
        // the role it named — the drift pin the name door and the parse
        // door cannot pass apart.
        for role in [
            Role::Decoration,
            Role::Container,
            Role::Label,
            Role::Button,
            Role::TextInput,
            Role::ScrollArea,
            Role::ListItem,
            Role::List,
            Role::CheckBox,
            Role::Slider,
            Role::ProgressIndicator,
            Role::Table,
            Role::TableRow,
            Role::TableCell,
            Role::ColumnHeader,
            Role::Canvas,
            Role::LiveRegion,
            Role::Radio,
            Role::RadioGroup,
            Role::Link,
            Role::Heading,
            Role::Tab,
            Role::TabList,
            Role::SpinButton,
            Role::Toolbar,
            Role::Switch,
            Role::TabPanel,
            Role::Meter,
            Role::Dialog,
            Role::Tree,
            Role::TreeItem,
            Role::Tooltip,
            Role::MenuBar,
            Role::Menu,
            Role::MenuItem,
            Role::Image,
            Role::Banner,
            Role::Navigation,
            Role::Main,
            Role::ContentInfo,
            Role::Complementary,
            Role::Form,
            Role::Region,
            Role::Search,
        ] {
            assert_eq!(
                Role::from_name(role.name()),
                Some(role),
                "the word {} parses back to the role it names",
                role.name()
            );
        }
    }

    #[test]
    fn role_from_name_refuses_the_unknown_word() {
        assert_eq!(
            Role::from_name("gizmo"),
            None,
            "a word outside the vocabulary is no role"
        );
        assert_eq!(Role::from_name(""), None, "the empty word is no role");
    }

    // ------------------------------------------------------------------
    // M148 (ADR-0152): the region ask's geometry query. The point ask's
    // plural sibling beside the query vocabulary — the nodes whose
    // bounds the rect touches, tree order (the paint order's own), the
    // honest empty answer for the region nothing stands in.
    // ------------------------------------------------------------------

    #[test]
    fn hit_region_answers_the_touched_nodes_in_tree_order() {
        let snapshot = TreeSnapshot {
            nodes: vec![
                hit_node(
                    1,
                    Role::Container,
                    "window",
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(800.0, 600.0))),
                    vec![2, 3],
                ),
                hit_node(
                    2,
                    Role::Button,
                    "left",
                    Some(Rect::new(Point::new(10.0, 10.0), Size::new(80.0, 24.0))),
                    Vec::new(),
                ),
                hit_node(
                    3,
                    Role::Button,
                    "right",
                    Some(Rect::new(Point::new(500.0, 10.0), Size::new(80.0, 24.0))),
                    Vec::new(),
                ),
            ],
            root: Some(1),
        };
        let ids: Vec<u64> = snapshot
            .hit_region(Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 600.0)))
            .iter()
            .map(|node| node.id)
            .collect();
        assert_eq!(
            ids,
            vec![1, 2],
            "the region touches the window beside the left button — tree order, never the right one"
        );
    }

    #[test]
    fn hit_region_answers_the_honest_absence_when_nothing_touches() {
        // Four honest absences, one empty answer: no tree resolved, no
        // bounds written, a region nothing stands in — and the inverted
        // rect naming no positive area among them (the geometry's own
        // convention, never a guessed normalization).
        let empty = TreeSnapshot::default();
        assert!(
            empty
                .hit_region(Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)))
                .is_empty(),
            "no tree resolved yet — the honest absence"
        );
        let geometry_less = TreeSnapshot {
            nodes: vec![hit_node(1, Role::Button, "Save", None, Vec::new())],
            root: Some(1),
        };
        assert!(
            geometry_less
                .hit_region(Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0)))
                .is_empty(),
            "no bounds written — the layout pass never ran, the honest absence"
        );
        let snapshot = TreeSnapshot {
            nodes: vec![hit_node(
                1,
                Role::Button,
                "Save",
                Some(Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 20.0))),
                Vec::new(),
            )],
            root: Some(1),
        };
        assert!(
            snapshot
                .hit_region(Rect::new(Point::new(500.0, 500.0), Size::new(10.0, 10.0)))
                .is_empty(),
            "the region touches nothing standing — the honest absence"
        );
        assert!(
            snapshot
                .hit_region(Rect::new(Point::new(10.0, 10.0), Size::new(-5.0, 20.0)))
                .is_empty(),
            "an inverted rect names no area — the honest absence, never a munged half"
        );
    }

    // ------------------------------------------------------------------
    // M149 (ADR-0153): the path ask's ancestry door. The point ask's
    // walk names the ancestry the point descended through; this door
    // answers it for the id a panel already holds — root-first, the
    // asked node last, every miss a named species.
    // ------------------------------------------------------------------

    #[test]
    fn ancestry_of_answers_the_parent_chain_root_first() {
        let snapshot = TreeSnapshot {
            nodes: vec![
                hit_node(
                    1,
                    Role::Container,
                    "window",
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(800.0, 600.0))),
                    vec![2],
                ),
                hit_node(
                    2,
                    Role::Container,
                    "panel",
                    Some(Rect::new(Point::new(10.0, 10.0), Size::new(300.0, 200.0))),
                    vec![3],
                ),
                hit_node(
                    3,
                    Role::Button,
                    "Save",
                    Some(Rect::new(Point::new(20.0, 20.0), Size::new(80.0, 24.0))),
                    Vec::new(),
                ),
            ],
            root: Some(1),
        };
        let super::PathAnswer::Landed { ancestry } = snapshot.ancestry_of(3) else {
            panic!("the asked node stands — the chain answered");
        };
        let ids: Vec<u64> = ancestry.iter().map(|node| node.id).collect();
        assert_eq!(
            ids,
            vec![1, 2, 3],
            "the ancestry root-first, the asked node last — the walk's own path"
        );
        let super::PathAnswer::Landed { ancestry } = snapshot.ancestry_of(1) else {
            panic!("the root stands — the chain answered");
        };
        let ids: Vec<u64> = ancestry.iter().map(|node| node.id).collect();
        assert_eq!(ids, vec![1], "the root's ancestry is the root alone");
    }

    #[test]
    fn ancestry_of_refuses_the_unknown_node_as_no_node() {
        let snapshot = TreeSnapshot {
            nodes: vec![hit_node(
                1,
                Role::Container,
                "window",
                Some(Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 20.0))),
                Vec::new(),
            )],
            root: Some(1),
        };
        assert_eq!(
            snapshot.ancestry_of(9),
            super::PathAnswer::Missed(super::PathMiss::NoNode),
            "the asked id names nothing standing — the species names it, never a collapsed answer"
        );
    }

    #[test]
    fn ancestry_of_refuses_the_absent_tree_as_no_tree() {
        assert_eq!(
            TreeSnapshot::default().ancestry_of(1),
            super::PathAnswer::Missed(super::PathMiss::NoTree),
            "no tree resolved yet — the honest absence"
        );
    }

    #[test]
    fn path_miss_names_the_species_the_wire_speaks() {
        // The wire's tag vocabulary is `snake_case`; the species' name
        // door is the one word the protocol document and the answer
        // both speak (the hit miss's naming shape).
        for (species, word) in [
            (super::PathMiss::NoTree, "no_tree"),
            (super::PathMiss::NoNode, "no_node"),
        ] {
            assert_eq!(species.name(), word, "the species' wire word");
        }
    }

    // ------------------------------------------------------------------
    // M150 (ADR-0154): the descendant ask's door — what stands beneath.
    // The path ask names the walk that reaches the node; this door
    // answers what the walk would find standing under it — the subtree
    // beneath, tree order, the asked node itself the anchor the ask
    // already names (never riding its own answer), every miss a named
    // species at the address tier.
    // ------------------------------------------------------------------

    fn relative_snapshot() -> TreeSnapshot {
        TreeSnapshot {
            nodes: vec![
                hit_node(
                    1,
                    Role::Container,
                    "window",
                    Some(Rect::new(Point::new(0.0, 0.0), Size::new(800.0, 600.0))),
                    vec![2],
                ),
                hit_node(
                    2,
                    Role::Container,
                    "panel",
                    Some(Rect::new(Point::new(10.0, 10.0), Size::new(300.0, 200.0))),
                    vec![3, 4],
                ),
                hit_node(
                    3,
                    Role::Button,
                    "Save",
                    Some(Rect::new(Point::new(20.0, 20.0), Size::new(80.0, 24.0))),
                    Vec::new(),
                ),
                hit_node(
                    4,
                    Role::Label,
                    "Status",
                    Some(Rect::new(Point::new(20.0, 60.0), Size::new(80.0, 24.0))),
                    Vec::new(),
                ),
            ],
            root: Some(1),
        }
    }

    #[test]
    fn descendants_of_answers_the_subtree_beneath_in_tree_order() {
        let snapshot = relative_snapshot();
        let super::DescendantsAnswer::Landed { beneath } = snapshot.descendants_of(1) else {
            panic!("the asked node stands — the subtree answered");
        };
        let ids: Vec<u64> = beneath.iter().map(|node| node.id).collect();
        assert_eq!(
            ids,
            vec![2, 3, 4],
            "the whole subtree beneath, pre-order — the paint order's own"
        );
        let super::DescendantsAnswer::Landed { beneath } = snapshot.descendants_of(2) else {
            panic!("the panel stands — the subtree answered");
        };
        let ids: Vec<u64> = beneath.iter().map(|node| node.id).collect();
        assert_eq!(ids, vec![3, 4], "the panel's children, tree order");
    }

    #[test]
    fn descendants_of_a_leaf_answers_the_honest_absence() {
        let snapshot = relative_snapshot();
        let super::DescendantsAnswer::Landed { beneath } = snapshot.descendants_of(3) else {
            panic!("the leaf stands — the landing answered");
        };
        assert!(
            beneath.is_empty(),
            "nothing stands beneath a leaf — the honest absence, never a refusal"
        );
    }

    #[test]
    fn descendants_of_refuses_the_unknown_node_as_no_node() {
        let snapshot = relative_snapshot();
        assert_eq!(
            snapshot.descendants_of(9),
            super::DescendantsAnswer::Missed(super::PathMiss::NoNode),
            "the asked id names nothing standing — the species names it, never a collapsed answer"
        );
    }

    #[test]
    fn descendants_of_refuses_the_absent_tree_as_no_tree() {
        assert_eq!(
            TreeSnapshot::default().descendants_of(1),
            super::DescendantsAnswer::Missed(super::PathMiss::NoTree),
            "no tree resolved yet — the honest absence"
        );
    }

    // ------------------------------------------------------------------
    // M151 (ADR-0155): the sibling ask's door — what stands beside.
    // The path ask names the walk that reaches the node; the descendant
    // ask names what stands under it; this door answers what stands in
    // the parent's row beside it — the parent's other children, tree
    // order, the asked node the anchor the ask already names. The
    // root's landing names the honest absence (the root stands beside
    // nothing), every miss a named species at the address tier.
    // ------------------------------------------------------------------

    #[test]
    fn siblings_of_answers_the_children_beside_in_tree_order() {
        let snapshot = relative_snapshot();
        let super::SiblingsAnswer::Landed { beside } = snapshot.siblings_of(3) else {
            panic!("the asked node stands — the row answered");
        };
        let ids: Vec<u64> = beside.iter().map(|node| node.id).collect();
        assert_eq!(
            ids,
            vec![4],
            "the parent's other children, tree order — the asked node left out"
        );
        let super::SiblingsAnswer::Landed { beside } = snapshot.siblings_of(4) else {
            panic!("the label stands — the row answered");
        };
        let ids: Vec<u64> = beside.iter().map(|node| node.id).collect();
        assert_eq!(ids, vec![3], "the row answers from either side");
        let super::SiblingsAnswer::Landed { beside } = snapshot.siblings_of(2) else {
            panic!("the panel stands — the row answered");
        };
        assert!(
            beside.is_empty(),
            "the window's only child stands beside nothing — the honest absence"
        );
    }

    #[test]
    fn siblings_of_the_root_answers_the_honest_absence() {
        let snapshot = relative_snapshot();
        let super::SiblingsAnswer::Landed { beside } = snapshot.siblings_of(1) else {
            panic!("the root stands — the landing answered");
        };
        assert!(
            beside.is_empty(),
            "the root stands beside nothing — the landing names the absence, never a refusal"
        );
    }

    #[test]
    fn siblings_of_refuses_the_unknown_node_as_no_node() {
        let snapshot = relative_snapshot();
        assert_eq!(
            snapshot.siblings_of(9),
            super::SiblingsAnswer::Missed(super::PathMiss::NoNode),
            "the asked id names nothing standing — the species names it, never a collapsed answer"
        );
    }

    #[test]
    fn siblings_of_refuses_the_absent_tree_as_no_tree() {
        assert_eq!(
            TreeSnapshot::default().siblings_of(1),
            super::SiblingsAnswer::Missed(super::PathMiss::NoTree),
            "no tree resolved yet — the honest absence"
        );
    }
}
