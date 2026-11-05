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
use crate::mask::Change;
use crate::scene::{PaintCtx, Primitive, SceneSnapshot};
use crate::style::Style;
use crate::widget::{EventResponse, Role, Widget};
use rustc_hash::{FxHashMap, FxHashSet};

/// Errors surfaced by the layout engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutError {
    /// Human-readable failure description, widget ids included.
    pub message: String,
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "layout error: {}", self.message)
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
/// neutral vocabulary `alacer-a11y` converts into AccessKit nodes.
#[derive(Debug, Clone, PartialEq)]
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
        let needle = name.to_lowercase();
        self.nodes
            .iter()
            .filter(|n| n.role == role && n.label.to_lowercase().contains(&needle))
            .collect()
    }

    /// Whether any node with `role` is named `name`.
    #[must_use]
    pub fn contains(&self, role: Role, name: &str) -> bool {
        self.find(role, name).is_some()
    }

    /// The node with the exact id-path hash (the AccessKit NodeId).
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
    /// The snapshot arrives shared (`Arc`): the pass builds it once, the sink
    /// keeps it for queries — no per-frame deep clone of the tree (the p99
    /// budget measures TreeUpdate production, not bookkeeping).
    fn apply(&mut self, snapshot: std::sync::Arc<TreeSnapshot>, delta: &A11yDelta);

    /// The sink's resolved tree, for harness queries.
    #[must_use]
    fn resolved_tree(&self) -> Option<&TreeSnapshot> {
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

/// One frame's outcome, returned by [`PassPipeline::tick`].
#[derive(Debug, Clone)]
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
    /// The pass ledger, in contract order — every tick pass, ran or skipped.
    pub passes: Vec<PassRun>,
    /// Focus moved this frame.
    pub focus_changed: bool,
    /// Total tick wall time, when timings are on (else 0).
    pub total_nanos: u64,
}

impl FrameReport {
    /// `true` when no pass had work — the idle contract (A7).
    #[must_use]
    pub fn is_idle(&self) -> bool {
        !self.layout_ran && !self.paint_ran && !self.a11y_ran
    }

    /// The ledger entry for `kind`.
    #[must_use]
    pub fn pass(&self, kind: PassKind) -> Option<&PassRun> {
        self.passes.iter().find(|run| run.kind == kind)
    }

    /// The four injected stages do the frame's substantive work; everything
    /// else in the tick is pass-system bookkeeping (§7.2 kill criterion).
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
    #[must_use]
    pub fn pass_overhead_nanos(&self) -> u64 {
        self.total_nanos.saturating_sub(self.work_nanos())
    }

    /// `pass_overhead_nanos / total_nanos` — the M1 gate ratio: the collapse
    /// trigger fires above 0.15 on the 10k-node churn workload (§7.2).
    #[must_use]
    pub fn pass_overhead_ratio(&self) -> f32 {
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
    /// Pass ledger entries for dispatch-driven passes (event, action),
    /// attributed to the next tick's report.
    interframe: Vec<PassRun>,
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
    /// The id-path hashes pass 9 emitted last time — the removals source for
    /// the a11y delta (dead widgets leave the set; their ids surface once).
    last_a11y_ids: Option<FxHashSet<u64>>,
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
            interframe: Vec::new(),
            style_cache: FxHashMap::default(),
            focusables: Vec::new(),
            focused: None,
            pending_focus: None,
            ime_request: None,
            last_a11y_ids: None,
        }
    }

    /// Enables per-pass wall timing and the overhead ledger (the M1 gate
    /// instrumentation; benches and the overhead test turn this on).
    pub fn set_timings(&mut self, on: bool) {
        self.timings = on;
    }

    /// Whether per-pass timing is enabled.
    #[must_use]
    pub const fn timings_enabled(&self) -> bool {
        self.timings
    }

    /// Requests keyboard focus for `node` (resolved by pass 3 next tick).
    /// Requests for non-focusable or dead nodes are dropped silently.
    pub fn request_focus(&mut self, node: NodeId) {
        self.pending_focus = Some(node);
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

    /// The a11y sink's resolved tree (harness queries).
    #[must_use]
    pub fn a11y_snapshot(&self) -> Option<&TreeSnapshot> {
        self.a11y.resolved_tree()
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

    /// Pass 1 (`event`): hit-test the pointer position, then route the event
    /// through the full capture→bubble contract (§10.1): ancestors see it
    /// first during capture (root → target), the target and its ancestors see
    /// it again during bubble (target → root); the first `Handled` stops the
    /// phase. Z-order-aware: the deepest, last-painted hit is the target.
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
        let start = self.timings.then(std::time::Instant::now);
        let _guard = span.enter();

        let response = self.route_pointer(event, &mut run.ran);

        run.nanos = start.map_or(0, |t| crate::num::u128_to_u64(t.elapsed().as_nanos()));
        drop(_guard);
        self.interframe.push(run);
        response
    }

    /// The event pass body: capture walk, then bubble walk.
    fn route_pointer(&mut self, event: &PointerEvent, ran: &mut bool) -> EventResponse {
        let Some(position) = event.position() else {
            return EventResponse::Ignored;
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
            return EventResponse::Ignored;
        }
        *ran = true;

        // Capture: root → target (§10.1). First `Handled` stops the phase.
        for node in &chain {
            let response = self.visit_with_phase(*node, EventPhase::Capture, event);
            if response == EventResponse::Handled {
                return EventResponse::Handled;
            }
        }
        // Bubble: target → root. Reverse pre-order tail = deepest, topmost
        // (last painted) hit first.
        for node in chain.iter().rev() {
            let response = self.visit_with_phase(*node, EventPhase::Bubble, event);
            if response == EventResponse::Handled {
                return EventResponse::Handled;
            }
        }
        EventResponse::Ignored
    }

    /// One capture/bubble visit: route the event with the phase set, merge
    /// the widget's requested masks upward. A widget that requests `FOCUS`
    /// is asking for keyboard focus — the click-to-focus seam (§10.3): the
    /// request becomes pass 3's pending focus, resolved same tick.
    fn visit_with_phase(
        &mut self,
        node: NodeId,
        phase: EventPhase,
        event: &PointerEvent,
    ) -> EventResponse {
        let (response, requested) = {
            let mut guard = self.arena.mutate(node);
            let mut ctx = crate::events::EventCtx::with_phase(phase);
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
        let start = self.timings.then(std::time::Instant::now);
        let _guard = span.enter();

        let mut response = EventResponse::Ignored;
        if let Some(focused) = self.focused
            && self.node_is_live(focused)
        {
            run.ran = true;
            response = self.visit_key(focused, event);
        }

        // Focus cycling: Tab walks the candidate list forward, Shift+Tab
        // backward (§10.3). The widget's `Ignored` lets the framework take it.
        if response == EventResponse::Ignored && event.presses(&crate::events::Key::Tab) {
            if self.cycle_focus(event.modifiers.shift) {
                response = EventResponse::Handled;
            }
            run.ran = true;
        }

        run.nanos = start.map_or(0, |t| crate::num::u128_to_u64(t.elapsed().as_nanos()));
        drop(_guard);
        self.interframe.push(run);
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
        let start = self.timings.then(std::time::Instant::now);
        let _guard = span.enter();

        let mut response = EventResponse::Ignored;
        if let Some(focused) = self.focused
            && self.node_is_live(focused)
        {
            run.ran = true;
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
        }

        run.nanos = start.map_or(0, |t| crate::num::u128_to_u64(t.elapsed().as_nanos()));
        drop(_guard);
        self.interframe.push(run);
        response
    }

    /// One focus-routed key visit: route with the phase set, merge masks.
    fn visit_key(&mut self, node: NodeId, event: &KeyEvent) -> EventResponse {
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

    /// Moves focus one candidate forward (or backward with `reverse`).
    /// Wraps at both ends; `true` when focus moved.
    fn cycle_focus(&mut self, reverse: bool) -> bool {
        if self.focusables.is_empty() {
            return false;
        }
        let next = match self.focused {
            Some(current) => {
                let position = self
                    .focusables
                    .iter()
                    .position(|c| *c == current)
                    .unwrap_or(0);
                if reverse {
                    (position + self.focusables.len() - 1) % self.focusables.len()
                } else {
                    (position + 1) % self.focusables.len()
                }
            }
            None => 0,
        };
        let target = self.focusables[next];
        if Some(target) == self.focused {
            return false;
        }
        self.pending_focus = Some(target);
        // Resolve immediately: key routing must see the new focus this event.
        self.update_focus();
        true
    }

    /// Runs exactly one frame through the pass contract (§7.2): mutate →
    /// update_focusable → update_focus → style → layout → compose → paint →
    /// accessibility. When no mask is pending every pass is skipped — idle
    /// cost zero (A7). Every pass is traced (`alacer::pass`) and, with
    /// timings on, measured for the M1 overhead gate.
    ///
    /// # Errors
    ///
    /// Propagates [`LayoutError`] from the layout pass.
    pub fn tick(&mut self, viewport: Size) -> Result<FrameReport, LayoutError> {
        let total_start = self.timings.then(std::time::Instant::now);
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
            let root = root.expect("root mounted for layout");
            let start = self.timings.then(std::time::Instant::now);
            self.layout
                .sync_and_layout(&mut self.arena, root, &dirty, viewport)?;
            layout_work = start.map_or(0, |t| crate::num::u128_to_u64(t.elapsed().as_nanos()));
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
                let start = s.timings.then(std::time::Instant::now);
                translated = s.compose_scrolls();
                compose_work = start.map_or(0, |t| crate::num::u128_to_u64(t.elapsed().as_nanos()));
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
        ledger.push(self.run_pass(PassKind::Paint, |s| {
            if paint_ran {
                let start = s.timings.then(std::time::Instant::now);
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
                paint_work = start.map_or(0, |t| crate::num::u128_to_u64(t.elapsed().as_nanos()));
            }
            paint_ran
        }));
        if let Some(run) = ledger.last_mut() {
            run.work_nanos = paint_work;
            run.nanos = paint_work;
        }
        // The damage ladder (§9.3): structural/first frames repaint in full,
        // silent frames cost nothing, partial frames carry their regions.
        let damage = if !paint_ran {
            Damage::NONE
        } else if structure_changed || first_frame {
            Damage::FULL
        } else {
            damage.finish(viewport)
        };

        // Pass 9 — accessibility: the M3 a11y contract (§11.1). The delta
        // carries dirty ids only, the ids structural churn removed, and the
        // focused node; a focus move runs the pass even with no other a11y
        // work (platform focus lives in the update, not in a node).
        let a11y_set: FxHashSet<u64> = dirty
            .iter()
            .filter(|(_, m)| m.forces_a11y() || m.intersects(Change::LAYOUT))
            .map(|(n, _)| self.id_hash(*n))
            .chain(translated.iter().map(|n| self.id_hash(*n)))
            .collect();
        let a11y_ran = first_frame || !a11y_set.is_empty() || focus_changed;
        let mut a11y_work = 0_u64;
        let mut a11y_delta = A11yDelta::empty();
        ledger.push(self.run_pass(PassKind::Accessibility, |s| {
            if a11y_ran {
                let start = s.timings.then(std::time::Instant::now);
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
                a11y_delta = delta;
                a11y_work = start.map_or(0, |t| crate::num::u128_to_u64(t.elapsed().as_nanos()));
                s.counters.a11y_runs += 1;
            }
            a11y_ran
        }));
        if let Some(run) = ledger.last_mut() {
            run.work_nanos = a11y_work;
            run.nanos = a11y_work;
        }

        let total_nanos =
            total_start.map_or(0, |t| crate::num::u128_to_u64(t.elapsed().as_nanos()));
        // A frame that skipped paint replays the cached materialization —
        // the cheap replay IS the journal's point (§9.1): no walk, no
        // primitive clones, one Arc clone (§14.3's warm path).
        let scene = scene
            .or_else(|| self.scene_cache.as_ref().map(std::sync::Arc::clone))
            .unwrap_or_default();
        Ok(FrameReport {
            layout_ran,
            paint_ran,
            a11y_ran,
            scene,
            damage,
            a11y_delta,
            passes: ledger,
            focus_changed,
            total_nanos,
        })
    }

    /// Runs one pass under its tracing span, recording the ledger entry.
    /// `body` returns whether the pass had work.
    fn run_pass(&mut self, kind: PassKind, body: impl FnOnce(&mut Self) -> bool) -> PassRun {
        let span = tracing::trace_span!(target: "alacer::pass", "pass", pass = kind.name());
        let start = self.timings.then(std::time::Instant::now);
        let _guard = span.enter();
        let ran = body(self);
        let nanos = start.map_or(0, |t| crate::num::u128_to_u64(t.elapsed().as_nanos()));
        drop(_guard);
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
        if let Some(current) = self.focused
            && !self.node_is_live(current)
        {
            self.focused = None;
            changed = true;
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
            // `clip_overflow` paints (viewport clipping); the rest is layout.
            let paint_diff = cached.clip_overflow != style.clip_overflow;
            let mut layout_only = cached.clone();
            layout_only.clip_overflow = style.clip_overflow;
            let _ = layout_only != style; // reason flag: any style diff invalidates layout
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
            let primitives = self.paint_node(node);
            if let Some(extent) = crate::scene::primitives_extent(&primitives) {
                damage.add(extent);
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

    fn paint_node(&mut self, node: NodeId) -> Vec<Primitive> {
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
        let mut out = Vec::new();
        {
            let mut sink = JournalVec { out: &mut out };
            let mut ctx = PaintCtx::new(&mut sink, origin, size, clip);
            let widget = &self.arena.slot(node).widget;
            widget.paint(&mut ctx);
        }
        out
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

    /// Builds the a11y snapshot from the arena (pass 9 input).
    fn build_tree_snapshot(&self) -> TreeSnapshot {
        let mut snapshot = TreeSnapshot::default();
        self.arena.for_each_pre_order(&mut |id, slot| {
            let node = crate::passes::TreeNode {
                id: self.id_hash(id),
                role: slot.widget().role(),
                label: slot.widget().label().resolve(),
                bounds: slot.bounds,
                children: slot.children.iter().map(|c| self.id_hash(*c)).collect(),
                value: slot.widget().value(),
            };
            if snapshot.root.is_none() {
                snapshot.root = Some(node.id);
            }
            snapshot.nodes.push(node);
        });
        snapshot
    }

    /// Pass 10 (`action`): routes an accessibility action to the widget whose
    /// id-path hash is `target` (§7.2 pass 10: side-channel, never masks).
    pub fn dispatch_a11y_action(&mut self, target: u64, action: crate::widget::A11yAction) -> bool {
        let Some(node) = self.node_for_hash(target) else {
            return false;
        };
        let mut guard = self.arena.mutate(node);
        guard.widget_mut().on_a11y_action(action)
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
        A11ySink, LayoutEngine, LayoutError, PassKind, PassPipeline, SceneJournal, TreeSnapshot,
    };
    use crate::arena::{Arena, NodeId};
    use crate::events::PointerEvent;
    use crate::flux::Flux;
    use crate::geom::{Point, Rect, Size, Vector};
    use crate::mask::Change;
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
        for run in &report.passes {
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
        for run in &idle.passes {
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
        });
        assert_eq!(response, EventResponse::Handled);
        assert_eq!(
            log.lock().expect("log lock").as_slice(),
            [("capture", "root")],
            "a handled capture ends the route: no target, no bubble"
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
        let _guard = crate::flux::batch_test_lock();

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

        let mut p = pipeline();
        let root = p.mount_root(Box::new(ChurnBox {
            size: Size::new(400.0, 800.0),
            style: std::sync::Arc::new(Mutex::new(Style::default())),
        }));

        // 100 containers × 100 children = 10,101 nodes.
        const CONTAINERS: usize = 100;
        const CHILDREN: usize = 100;
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
}
