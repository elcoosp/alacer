//! The devtools observation seam (§13.3): the causal vocabulary the
//! inspector consumes, and the observer sink the pipeline notifies.
//!
//! Everything here is **dev-mode and opt-in**: an unset observer and a
//! disabled capture flag cost one branch per frame — the A7 idle contract
//! and the M1 overhead gate are unchanged. With them on, the seam answers
//! the Causality inspector's five questions from recorded truth:
//!
//! | Question | Answered by |
//! |---|---|
//! | "Why did this render?" | [`InspectEvent::Edge`] — reactive edge activations carrying the masks they queued, resolved into the frame's dirty set |
//! | "Who wrote this signal?" | [`InspectEvent::Write`] — provenance-tagged writes (`Flux::write`) |
//! | "Who subscribes to this signal?" | the flux's edge table (`Flux::edge_table`) — the binding registrations |
//! | "Why is this frame slow?" | [`FrameSummary::passes`] — the per-pass ledger with wall times |
//! | "What did this mask touch?" | [`FrameSummary::dirty`] — the mask heatmap: role, label, bounds, mask per dirty node |
//! | "Why did focus change — or why didn't it?" | [`InspectEvent::Focus`] — the walk's own answer, streamed as the doors resolve (ADR-0034) |
//! | "Who rode the walk — the keyboard, the code, or the ring?" | [`FocusRider`] — the attribution every [`FocusRecord`] carries beside its movement (ADR-0036) |
//! | "What did this field's journal do?" | [`InspectEvent::Edit`] — the edit trail: one record per journal action, streamed the moment the journal answers (ADR-0037) |

use crate::arena::NodeId;
use crate::damage::Damage;
use crate::geom::Rect;
use crate::geom::Size;
use crate::history::EditAction;
use crate::mask::Change;
use crate::passes::{FocusMove, FocusMoveOutcome, PassKind};
use crate::widget::Role;

/// Identity of one signal in the causal log: a 64-bit key hashed from the
/// reactive handle (dev-mode only; collisions are log-noise, never
/// correctness).
pub type SignalKey = u64;

/// Identity of one binding registration (one signal→node edge), monotonic
/// per flux.
pub type BindingId = u64;

/// One reactive edge activation (§13.3 "why did this render"): the effect
/// behind the `signal → node` binding re-ran and queued `masks` for `node`.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeActivation {
    /// The binding (the stable signal→node edge) that activated.
    pub binding: BindingId,
    /// The signal whose write triggered the rerun.
    pub signal: SignalKey,
    /// The node the masks landed on.
    pub node: NodeId,
    /// The masks the edge queued for the node.
    pub masks: Change,
}

/// One provenance-tagged signal write (§13.3 "who wrote this signal").
#[derive(Debug, Clone, PartialEq)]
pub struct SignalWrite {
    /// The written signal.
    pub signal: SignalKey,
    /// Monotonic write sequence (ordering within the log).
    pub sequence: u64,
    /// The origin the writer declared (`Flux::write`).
    pub origin: std::borrow::Cow<'static, str>,
}

/// The input events the inspector sees routed through pass 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// A pointer button went down.
    PointerDown,
    /// A pointer button came up.
    PointerUp,
    /// The pointer moved.
    PointerMove,
    /// The pointer left the window.
    PointerLeave,
    /// The wheel scrolled.
    Wheel,
    /// A keyboard event routed to the focused widget.
    Key,
    /// An IME event routed to the focused widget.
    Ime,
}

/// One routed input event (§10): its kind and the widget it targeted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputRecord {
    /// The event kind.
    pub kind: InputKind,
    /// The target widget, when the router resolved one (the deepest hit for
    /// pointer events, the focused widget for key/IME).
    pub target: Option<NodeId>,
}

/// One pass's ledger entry as the inspector sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassTiming {
    /// Which pass.
    pub kind: PassKind,
    /// Whether it had work.
    pub ran: bool,
    /// Wall time (0 when timings are off).
    pub nanos: u64,
}

/// One dirty node in the frame's mask heatmap (§13.3 "what did this mask
/// touch").
#[derive(Debug, Clone, PartialEq)]
pub struct DirtyNode {
    /// The node.
    pub node: NodeId,
    /// The masks it carried this frame.
    pub mask: Change,
    /// Its semantic role.
    pub role: Role,
    /// Its resolved label.
    pub label: String,
    /// Its bounds, when laid out (removed nodes carry none).
    pub bounds: Option<Rect>,
}

/// One frame's causal summary, assembled by the tick when an observer is
/// installed. The frame's reactive trail travels as separate
/// [`InspectEvent::Edge`] / [`InspectEvent::Write`] events, streamed
/// immediately before this summary.
#[derive(Debug, Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "a frame ledger is genuinely four independent pass verdicts; packing them into bitflags would trade devtools readability for nothing"
)]
pub struct FrameSummary {
    /// Monotonic frame number (per pipeline).
    pub number: u64,
    /// Whether the layout pass ran.
    pub layout_ran: bool,
    /// Whether the paint pass ran.
    pub paint_ran: bool,
    /// Whether the accessibility pass ran.
    pub a11y_ran: bool,
    /// Whether focus moved.
    pub focus_changed: bool,
    /// The frame's paint damage (§9.3).
    pub damage: Damage,
    /// The viewport the frame rendered into.
    pub viewport: Size,
    /// A11y delta: updated id count.
    pub a11y_updated: usize,
    /// A11y delta: removed id count.
    pub a11y_removed: usize,
    /// Total tick wall time (0 when timings are off).
    pub total_nanos: u64,
    /// The pass ledger, in contract order.
    pub passes: Vec<PassTiming>,
    /// The frame's dirty set, resolved to roles/labels/bounds — the mask
    /// heatmap. Bounded: see [`Self::dirty_truncated`].
    pub dirty: Vec<DirtyNode>,
    /// How many dirty nodes were dropped from `dirty` (the heatmap cap).
    pub dirty_truncated: usize,
}

/// Who rode the focus walk (M30, ADR-0036): the attribution the causal
/// log carries beside every [`FocusRecord`]'s movement. One funnel, three
/// riders — the keyboard's Tab/Shift+Tab fallback (the M24 seam), the
/// ring's directional keys (the M29 seam), and the app's own door calls —
/// and until the record named its rider, a trail reader could not answer
/// "did the user navigate, or did the app?". The rider is the route's
/// name, not a second channel: the walk behind the doors is untouched,
/// and the wire carries the rider's [`FocusRider::name`] beside the
/// movement (the protocol's additive rule, ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusRider {
    /// The keyboard's Tab/Shift+Tab fallback — the dispatch seam the
    /// walk joined in M24 (ADR-0030): the user tabbed, the window's
    /// funnel answered.
    Keyboard,
    /// The ring's directional keys — the dispatch seam the four arrows
    /// joined in M29 (ADR-0035): a D-pad button, a TV remote's ring, a
    /// plain arrow the focused widget did not answer.
    Ring,
    /// The app's own code — the [`crate::passes::PassPipeline::focus_move`]
    /// and [`crate::passes::PassPipeline::focus_move_in`] doors the
    /// consumer calls directly (the M24/M25 doors).
    Consumer,
}

impl FocusRider {
    /// The rider's name, the wire vocabulary's `snake_case` tag
    /// ([`crate::passes::FocusMove::name`] for movements, this for
    /// riders — the protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Ring => "ring",
            Self::Consumer => "consumer",
        }
    }
}

/// One focus-move door ride (§13.3, ADR-0034): the causal log's answer to
/// "why did focus change — or why didn't it". The record is the door's own
/// outcome ([`FocusMoveOutcome`]) carried verbatim — the movement the
/// caller asked for, the scope that bounded the walk, and the landing or
/// the named refusal species the walk answered. It streams the moment the
/// door resolves (a refused ride streams too — that is exactly the fact
/// the log exists for), so a key event's trail reads focus → input →
/// frame (the record streams when the door resolves, before the input
/// record the dispatch forwards).
///
/// Since M30 the record names its rider (ADR-0036, the attribution rule
/// ADR-0035 pre-registered): [`FocusRider`] beside the movement — the
/// keyboard's Tab fallback, the ring's arrows, or the app's code — so a
/// trail reader answers "who moved focus" from the log, never from a
/// re-implementation at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusRecord {
    /// The movement the caller asked the walk for.
    pub movement: FocusMove,
    /// The subtree root that bounded the walk — the modal's focus trap
    /// ([`crate::passes::PassPipeline::focus_move_in`]); `None` for the
    /// unscoped funnel ([`crate::passes::PassPipeline::focus_move`]).
    pub scope: Option<NodeId>,
    /// Who rode the walk — the keyboard's Tab fallback, the ring's
    /// directional keys, or the app's own doors (ADR-0036).
    pub rider: FocusRider,
    /// The outcome the door returned — the landing or the refusal
    /// species, the same value the door handed its caller.
    pub outcome: FocusMoveOutcome,
}

/// One journal action's record (§13.3, ADR-0037): the causal log's
/// answer to "what did this field's journal do — and how deep is it
/// now". The journal itself writes the fact the moment [`EditAction`]
/// resolves — an applied step, a refused chord, the app's reset — so an
/// inspector trail, a status line, or a test reads the edit history
/// without re-implementing the observation at the chord site. The depths
/// are the journal's own introspection reads
/// ([`crate::history::EditHistory::undo_depth`] vocabulary) taken when
/// the fact streamed; the fields are the additive wire rule's shape
/// (ADR-0010), `EditAction::name` the single source for the action's tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditRecord {
    /// The action the journal answered.
    pub action: EditAction,
    /// Whether a [`EditAction::Record`] folded into the journal's tip
    /// (the pure-append coalescing rule, ADR-0021) — `false` for every
    /// other action, whose fact never coalesces.
    pub coalesced: bool,
    /// Whether the chord was refused — an exhausted undo or redo branch
    /// (the refuse-don't-munge discipline: the refusal streams too, the
    /// depths naming the journal it did not move). Never `true` for
    /// [`EditAction::Record`] or [`EditAction::Reset`], which cannot
    /// refuse.
    pub refused: bool,
    /// The undo branch's depth when the fact streamed.
    pub undo_depth: usize,
    /// The redo branch's depth when the fact streamed.
    pub redo_depth: usize,
}

/// One observation: what the pipeline and the flux surface to devtools.
#[derive(Debug, Clone)]
pub enum InspectEvent {
    /// A reactive edge activated (the "why did this render" trail).
    Edge(EdgeActivation),
    /// A provenance-tagged signal write.
    Write(SignalWrite),
    /// An input event was routed.
    Input(InputRecord),
    /// A focus-move door answered — the walk's landing or
    /// refusal, the "why did focus change — or why didn't it" trail
    /// (ADR-0034).
    Focus(FocusRecord),
    /// A journal action answered — the edit trail (ADR-0037): the
    /// applied step, the refused chord, or the app's reset, depths
    /// beside it. Streams with the frame's own records (the tick's
    /// start drain, the announcement drain's shape).
    Edit(EditRecord),
    /// A frame completed — the frame's causal summary.
    Frame(FrameSummary),
}

/// The observer sink: devtools implement this and the pipeline forwards
/// [`InspectEvent`]s.
pub trait InspectObserver {
    /// Receives one observation.
    fn observe(&mut self, event: InspectEvent);
}

/// The heatmap cap: dirty nodes carried per [`FrameSummary`] before
/// truncation (a devtools log, not a frame-budget liability).
pub const DIRTY_HEATMAP_CAP: usize = 512;

/// The signal-key hash: identity from the handle's `Hash` impl. Reactivity
/// handles hash by their backing allocation, so one signal keeps one key for
/// its lifetime.
pub(crate) fn signal_key_of(signal: &impl std::hash::Hash) -> SignalKey {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    signal.hash(&mut hasher);
    hasher.finish()
}
