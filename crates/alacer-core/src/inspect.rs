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
//! | "Why did the live region speak — or refuse?" | [`InspectEvent::Announce`] — the announcement trail: one record per speech act, streamed the moment the pipeline answers it (ADR-0039) |
//! | "Did a shortcut take this key — or why not?" | [`InspectEvent::Key`] — the keymap consult's answer: one record per funnel consult (the chord beside the species), streamed the moment the consult resolves (ADR-0042) |

use crate::announce::Politeness;
use crate::arena::NodeId;
use crate::damage::Damage;
use crate::geom::Rect;
use crate::geom::Size;
use crate::history::EditAction;
use crate::keymap::{Chord, KeymapConsult};
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

/// Why one speech act did not land on the reserved live region (§13.3,
/// ADR-0039): the two honest refusals the announcement contract names —
/// an empty or all-whitespace text speaks nothing, and before a root is
/// mounted there is no tree to attach the region to. The refusal streams
/// too (the M28 honesty at the speech-act tier): the fact a trail reader
/// wants is exactly the act that never reached the AT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnounceRefusal {
    /// The text was empty or all whitespace — nothing honest to ship
    /// (the push route and the direct door both refuse before anything
    /// queues).
    EmptyText,
    /// No root is mounted — the region has no tree to join, so the act
    /// is refused at the direct door or dropped at the tick's drain.
    NoRoot,
}

impl AnnounceRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag
    /// (the protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::EmptyText => "empty_text",
            Self::NoRoot => "no_root",
        }
    }
}

/// What the pipeline answered one speech act (§13.3, ADR-0039): the
/// record is the door's own outcome ([`crate::passes::FocusMoveOutcome`]
/// 's shape at the announcement tier) — the act landed on the reserved
/// region, the act was replaced before it could land (last write wins,
/// visible as data instead of a silent drop), or the act was refused
/// with its named species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnounceOutcome {
    /// The act landed on the reserved live region — its content and
    /// politeness ride the frame's a11y delta (the M18 contract's
    /// accepted route).
    Delivered,
    /// A later act in the same batch replaced this one before the tick
    /// drained it — last write wins (ADR-0024), the replaced act's fact
    /// streaming so a trail reader answers "why did the AT never say
    /// this" from the log.
    Superseded,
    /// The act was refused — [`AnnounceRefusal`] names the species.
    Refused(AnnounceRefusal),
}

impl AnnounceOutcome {
    /// The outcome's name, the wire vocabulary's `snake_case` tag
    /// (the protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Superseded => "superseded",
            Self::Refused(_) => "refused",
        }
    }
}

/// One speech act's record (§13.3, ADR-0039): the causal log's answer
/// to "why did the live region speak — or refuse". The app's voice —
/// state changes that own no widget — had no route on the causal log:
/// an announcement landed on the reserved region and left no trace a
/// trail reader could route on. Every act the pipeline answers streams
/// one record the moment the answer resolves — the direct door's
/// acceptance and both refusals at the door, the push-side empty
/// refusal on the thread's trail, and the drain's batch answers with
/// the last-write-wins rule visible — so a status line, an inspector
/// trail, or a test reads the announcement seam without
/// re-implementing the observation at the call site. The politeness is
/// the act's own ([`Politeness`], the `aria-live` priority the region
/// carries); the fields are the additive wire rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnnounceRecord {
    /// The `aria-live` priority the act asked for.
    pub politeness: Politeness,
    /// What the pipeline answered — the landing, the replacement, or
    /// the named refusal.
    pub outcome: AnnounceOutcome,
}

/// One keymap consult's record (§13.3, ADR-0042): the causal log's
/// answer to "did a shortcut take this key — or why not". ADR-0041's
/// funnel consult answered a bare `bool` whose `false` collapsed three
/// different facts — the firing, the table's silence, the absent
/// window — and its own re-open rule priced the record: the funnel
/// streams one consult the moment it resolves, before the fallbacks'
/// focus records, so a key event's trail reads keymap → focus → input
/// → frame. The chord is the event's own shape
/// ([`Chord::with_modifiers`], the exact modifier flags the funnel
/// asked about) — one binding per chord (the M35 invariant), so the
/// chord *is* the fired binding's identity, and since M37 (ADR-0043)
/// the record carries the binding's prose name beside it — answered by
/// the seam's provided `fired_name` read beside the consult, captured
/// before the consult's dispatch, never a second consult channel. The
/// outcome is the consult's own
/// ([`KeymapConsult`]) — the firing, the table's fall-through, or the
/// uninstalled window, each named; a consult the focused widget's
/// answer pre-empted never ran and streams nothing (the M35
/// precedence, visible as an absent record). The fields are the
/// additive wire rule's shape (ADR-0010).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRecord {
    /// The chord the funnel asked the app's shortcuts about — the
    /// event's key plus its exact modifier shape.
    pub chord: Chord,
    /// What the consult answered — the fired binding, the table's
    /// silence, or the uninstalled window.
    pub outcome: KeymapConsult,
    /// The fired binding's prose name (M37, ADR-0043) — what the
    /// seam's provided `fired_name` answered when the consult fired;
    /// `None` when nothing fired, or when the seam that fired names no
    /// binding (the provided method's default, the record staying
    /// honest about it).
    pub name: Option<String>,
}

/// What the clipboard tier just answered (§13.3, ADR-0045): the
/// clipboard trail's action vocabulary, one [`ClipboardRecord`] per
/// round trip. The names are the wire's stable tags
/// ([`ClipboardAction::name`], the protocol's `snake_case` rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardAction {
    /// A copy chord wrote the selection's span to the clipboard seam
    /// (the editing contract's Ctrl/Cmd+C, ADR-0020).
    Copy,
    /// A cut chord wrote the selection's span out, then deleted it —
    /// the deletion only ever followed an accepted write.
    Cut,
    /// A paste chord read the clipboard's text, sanitized it through
    /// the single-line rule, and inserted it at the caret.
    Paste,
}

impl ClipboardAction {
    /// The action's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values —
    /// [`EditAction::name`] for journal actions, this for clipboard
    /// round trips).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Cut => "cut",
            Self::Paste => "paste",
        }
    }
}

/// Why one clipboard round trip moved nothing (§13.3, ADR-0045): the
/// honest refusals the editing contract names (ADR-0020's refuse-don't-
/// munge rules, now facts on the log). The refusal streams too — the
/// fact a trail reader wants is exactly the round trip that never moved
/// anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardRefusal {
    /// The chord asked for a selection the field did not carry — a copy
    /// or a cut never invents content, so a degenerate selection is the
    /// honest `no_selection`.
    NoSelection,
    /// The clipboard had nothing to paste: the read answered `None`
    /// (an empty clipboard, a non-text payload, a clipboard this
    /// process cannot open), or the payload sanitized to nothing — a
    /// paste never invents content either.
    EmptyClipboard,
    /// The platform refused the write (the permission/escape story made
    /// observable) — a cut whose write was refused did not delete, the
    /// selection surviving a clipboard that failed it.
    WriteRefused,
}

impl ClipboardRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag
    /// (the protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoSelection => "no_selection",
            Self::EmptyClipboard => "empty_clipboard",
            Self::WriteRefused => "write_refused",
        }
    }
}

/// What the clipboard tier answered one round trip (§13.3, ADR-0045):
/// the record is the round trip's own outcome
/// ([`AnnounceOutcome`]'s shape at the clipboard tier) — the payload
/// moved, or the round trip was refused with its named species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOutcome {
    /// The payload moved: the write landed (a copy or a cut), or the
    /// sanitized read landed at the caret (a paste).
    Delivered,
    /// The round trip moved nothing — [`ClipboardRefusal`] names the
    /// species.
    Refused(ClipboardRefusal),
}

impl ClipboardOutcome {
    /// The outcome's name, the wire vocabulary's `snake_case` tag
    /// (the protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Refused(_) => "refused",
        }
    }
}

/// One clipboard round trip's record (§13.3, ADR-0045): the causal
/// log's answer to "did the text move — or why not". The editing
/// contract's chords (ADR-0020) answered the field's own `bool`, whose
/// `false` collapsed three different facts — the selection that was not
/// there, the clipboard that carried nothing, the platform that refused
/// the write — and the ride the M31/M33/M36 rhythm pre-registered prices
/// the record: every round trip the field answers streams one fact, the
/// refused round trips included. The record carries the payload's
/// UTF-8 scalar count beside the outcome — the length names the fact,
/// never the payload itself: a clipboard can carry a secret, and the
/// log is a devtools surface, not a shadow clipboard. The fields are
/// the additive wire rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardRecord {
    /// The round trip the field answered.
    pub action: ClipboardAction,
    /// What the field answered — the delivered payload or the named
    /// refusal.
    pub outcome: ClipboardOutcome,
    /// The payload's UTF-8 scalar count when the round trip delivered —
    /// the span copied, the span cut, the text pasted. Zero on a
    /// refusal: a refusal moves nothing.
    pub chars: usize,
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
    /// A speech act answered — the announcement trail (ADR-0039): the
    /// delivered act, the superseded act, or the named refusal, the
    /// politeness beside it. Streams where the answer resolves (the
    /// direct door, the push trail, the tick's start drain).
    Announce(AnnounceRecord),
    /// A keymap consult answered — the shortcut trail (ADR-0042): the
    /// chord the funnel asked the app's shortcuts about beside the
    /// species it got back (the firing, the fall-through, the
    /// uninstalled window). Streams the moment the consult resolves —
    /// before the fallbacks' focus records, so a key event's trail
    /// reads keymap → focus → input → frame.
    Key(KeyRecord),
    /// A clipboard round trip answered — the clipboard trail
    /// (ADR-0045): the copy, cut, or paste the editing chords answered,
    /// the payload's count beside the outcome, the refusals named.
    /// Streams with the frame's own records (the tick's start drain,
    /// the edit trail's shape — the fact queues where the round trip
    /// resolves, one frame after the chord).
    Clipboard(ClipboardRecord),
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
