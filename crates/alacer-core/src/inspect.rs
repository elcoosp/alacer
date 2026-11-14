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
//! | "Why did the drag move the value — where did it start, where did it land?" | [`InspectEvent::Gesture`] — the drag's own story: one per-gesture record per phase (the beginning, the end), streamed the moment the gesture's door resolves (ADR-0052) |

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
    /// paste never invents content either. Since M42 (ADR-0048) the
    /// record's `read` field splits the collapse: a paste that refused
    /// for the *read* names the engine's species beside this verdict, a
    /// paste that refused for the *field* carries the read's absence.
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
/// log is a devtools surface, not a shadow clipboard. Since M41
/// (ADR-0047, ADR-0045's own re-open rule) the record carries the
/// engine's platform-side fact beside the write refusal — the log
/// answering "which clipboard failed it". Since M42 (ADR-0048,
/// ADR-0047's third re-open rule) the record carries the read's
/// platform-side fact beside the paste's refusal — the M39
/// `empty_clipboard` collapse splitting at last (a paste that refused
/// for the *read* names the engine's species; a paste that refused for
/// the *field* carries the read's absence). The fields are the additive
/// wire rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardRecord {
    /// The round trip the field answered.
    pub action: ClipboardAction,
    /// What the field answered — the delivered payload or the named
    /// refusal.
    pub outcome: ClipboardOutcome,
    /// The engine's platform-side fact when the write was refused —
    /// `Some(species)` when the round trip's `write_refused` carried an
    /// engine answer (`unnamed` when the engine answered only its
    /// `bool`), `None` on a delivery and on every other refusal: a
    /// paste reads, a no-selection copy never asked the engine. The
    /// M41 additive field (ADR-0047) — the wire's documented `engine`
    /// beside the refusal species.
    pub engine: Option<crate::clipboard::ClipboardWriteRefusal>,
    /// The engine's read-side fact when the read was refused (M42,
    /// ADR-0048) — `Some(species)` when the paste refused for the read
    /// (`unnamed` when the engine answered only its `Option`), `None`
    /// on a delivery, on the field's own refusal (a payload that
    /// sanitized to nothing — the read delivered), and on every
    /// write-side round trip: a copy never asked a read question. The
    /// wire's documented `read` beside the refusal species.
    pub read: Option<crate::clipboard::ClipboardReadRefusal>,
    /// The payload's UTF-8 scalar count when the round trip delivered —
    /// the span copied, the span cut, the text pasted. Zero on a
    /// refusal: a refusal moves nothing.
    pub chars: usize,
}

/// What the value tier just answered (§13.3, ADR-0046): the value
/// trail's action vocabulary, one [`ValueRecord`] per AT write. The
/// names are the wire's stable tags ([`ValueAction::name`], the
/// protocol's `snake_case` rule). Since M47 (ADR-0053, ADR-0051's kill
/// arm) the vocabulary names the bounds' jumps too — the keyboard's
/// Home/End, never munged into a `SetValue` (the M11 rule's own
/// munging) — the named species the kill rule pre-registered, the
/// record riding the key route's stamp (M45) like every other ride.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueAction {
    /// The AT's `SetValue`: a payload that parses, then rides the same
    /// clamp-and-snap funnel every other write rides (M20, ADR-0026).
    SetValue,
    /// The AT's `Increment`: one step up the range, the funnel's own
    /// arithmetic.
    Increment,
    /// The AT's `Decrement`: one step down the range, the funnel's own
    /// arithmetic.
    Decrement,
    /// The keyboard's Home: a jump to the range's minimum, the funnel's
    /// own arithmetic (M47, ADR-0053) — the named species, never a
    /// munged `SetValue` (the M11 rule).
    JumpToMin,
    /// The keyboard's End: a jump to the range's maximum, the funnel's
    /// own arithmetic (M47, ADR-0053) — the named species, never a
    /// munged `SetValue` (the M11 rule).
    JumpToMax,
}

impl ValueAction {
    /// The action's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values —
    /// [`ClipboardAction::name`] for clipboard round trips, this for
    /// value-tier writes).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SetValue => "set_value",
            Self::Increment => "increment",
            Self::Decrement => "decrement",
            Self::JumpToMin => "jump_to_min",
            Self::JumpToMax => "jump_to_max",
        }
    }
}

/// Why one value-tier write moved nothing (§13.3, ADR-0046): the honest
/// refusals the value tier names (the M20/M21 refuse-don't-munge rules,
/// now facts on the log — the widget's own `bool` collapsed these into
/// one silence). The refusal streams too — the fact a trail reader
/// wants is exactly the write that never landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueRefusal {
    /// The availability tier holds the control down (M22, ADR-0028) —
    /// every door refuses before the request is even parsed, and the
    /// tree keeps the value the app wrote.
    Unavailable,
    /// The AT's payload was not a number the funnel could parse — a
    /// `SetValue` never invents a value from prose.
    InvalidPayload,
    /// The payload parsed but the funnel carries only finite scalars —
    /// "inf" parses fine, and the funnel's wall is its own fact (the
    /// m20 suite's two refusals, named apart at last).
    NonFiniteValue,
    /// The control has no write door for the action: a reporting
    /// widget's read-only tier (M21, ADR-0027), every AT write refused,
    /// never munged into a step or a press (the M11 rule).
    NoWriteDoor,
}

impl ValueRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::InvalidPayload => "invalid_payload",
            Self::NonFiniteValue => "non_finite_value",
            Self::NoWriteDoor => "no_write_door",
        }
    }
}

/// What the value tier answered one AT write (§13.3, ADR-0046): the
/// record is the write's own outcome ([`ClipboardOutcome`]'s shape at
/// the value tier) — the value landed, or the write was refused with
/// its named species.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueOutcome {
    /// The write landed: the control carries the value the request
    /// asked for. A write that lands where the value already was is an
    /// answered request, not a refusal (M20's own rule — the idle-frame
    /// property the log now names).
    Delivered,
    /// The write moved nothing — [`ValueRefusal`] names the species.
    Refused(ValueRefusal),
}

impl ValueOutcome {
    /// The outcome's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Refused(_) => "refused",
        }
    }
}

/// One value-tier write's record (§13.3, ADR-0046): the causal log's
/// answer to "did the AT's write land — where, or why not". The value
/// tier's AT actions (M20, ADR-0026 — the `Slider`; M21, ADR-0027 — the
/// read-only `Progress`) answered the widget's own `bool`, whose
/// `false` collapsed four different facts — the control the
/// availability tier holds down, the payload that parsed to nothing,
/// the payload the funnel cannot carry, the control that never had a
/// write door — and the ride the M31/M33/M36/M39 rhythm pre-registers
/// prices the record: every value-tier action the AT asks streams one
/// fact, the refused writes included. The record carries the value the
/// control holds as the record resolves — the funnel's clamp-and-snap
/// result on a delivery, the untouched current value on a refusal: a
/// slider's value is the tree's own vocabulary the AT already reads,
/// never a secret. Since M43 (ADR-0049, ADR-0046's first re-open rule)
/// the record names its control beside the action's routing — the
/// dispatch funnel knows the id the action arrived under, the widget
/// does not, so the stamp side lives on the funnel and the facts queued
/// beneath a routed action carry it. Since M44 (ADR-0050, ADR-0049's
/// second re-open rule) the record carries the ask beside the landing —
/// the finite scalar the AT asked before the control's clamp-and-snap
/// moved it, a clamp or a snap the trail could not show as one record's
/// story. Since M49 (ADR-0055, ADR-0053's second kill rule) the record
/// names the wall the funnel pinned the landing at: the range's own
/// edge a past ask was turned back by, beside the species — the ask's
/// own wall, never a second action species. The fields are the additive
/// wire rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValueRecord {
    /// The value-tier action the AT asked.
    pub action: ValueAction,
    /// What the control answered — the landed value or the named
    /// refusal.
    pub outcome: ValueOutcome,
    /// The value the control holds as the record resolves — the
    /// funnel's clamp-and-snap result on a delivery, the untouched
    /// current value on a refusal (the tree keeps the value the app
    /// wrote).
    pub value: f32,
    /// The finite scalar the AT asked, before the control's
    /// clamp-and-snap moved it (M44, ADR-0050): the ask beside the
    /// landing — a clamp or a snap the trail could not show is one
    /// record's story now. The fact lives where the knowledge lives:
    /// the widget's own door computes the ask, the note site carries
    /// it — never a value reconstructed from the landing (the target
    /// stamp's rule, ADR-0049). `None` when the ask never became a
    /// finite scalar: the unparsable payload, the door refused before
    /// arithmetic (the availability tier, the door-less reporting bar),
    /// the ask itself non-finite (the funnel's wall — the wire cannot
    /// carry an infinite ask honestly, `null` would claim an absence
    /// that never was, and the refusal species names the wall).
    pub requested: Option<f32>,
    /// The control the action arrived under (M43, ADR-0049): the
    /// AccessKit-stable id hash the dispatch funnel routed — the
    /// identity a screen reader's client, an inspector pane, or a test
    /// correlates. `None` when the write rode no routed action: the
    /// drag queues its facts from a funnel that routes no per-node
    /// action — the second kill rule's open arm, the absence honest.
    /// Since M45 (ADR-0051, ADR-0049's first kill rule) the keyboard's
    /// step rides the log attributed too: the dispatch funnel's key
    /// route holds the same stamp for the focused control's key call,
    /// the arrows' rides carrying the id like the AT tier's own.
    pub target: Option<u64>,
    /// The bound the funnel pinned the landing at (M49, ADR-0055,
    /// ADR-0053's second kill rule): `Some(species)` when the ask was
    /// turned back at the range's own edge — the landing at the wall,
    /// the ask somewhere else (the clamp, the snap, the wall that held
    /// a no-op against a past ask) — `None` when the ask wanted the
    /// wall exactly (the M47 jumps: the species already names the
    /// bound it asked for) and on every landing mid-range and every
    /// refusal (a write that moved nothing pinned nothing). The ask's
    /// own wall, beside the species — never a second action species
    /// (the kill rule's own words).
    pub pinned: Option<RangeBound>,
}

/// One drag gesture's record (§13.3, ADR-0052): the causal log's answer
/// to "why did the slider's value move while the hand dragged it — where
/// did the drag start, where did it land". ADR-0046 priced the value
/// tier's AT writes and ADR-0051 priced the keyboard's steps, but the
/// drag stayed off the log — the per-pointer-move flood the second kill
/// rule refused — so a trail reader could not answer the drag's story
/// without re-implementing the observation at the gesture site. The
/// price the kill rule names is the record: one per *gesture* phase,
/// never per move — a [`GesturePhase::Started`] the moment the press
/// arms the drag (the value it found), a [`GesturePhase::Ended`] the
/// moment the release or the leave resolves it (the value it left the
/// control holding, the beginning remembered beside it — each record
/// self-contained). One hundred moves inside one gesture stream exactly
/// two records; the flood rule holds by construction. The `from`/`to`
/// pair is the value tier's own vocabulary (the same scalars the AT
/// reads through [`crate::widget::Widget::value`]), and the control is
/// the funnel's own stamp — the pointer route knows the deepest hit
/// that captured (ADR-0025), the same discipline the action and key
/// routes ride (ADR-0049, ADR-0051). Since M48 (ADR-0054, ADR-0053's
/// first kill rule) the record names the wall a landing pinned: the
/// bound the drag's ask was turned back by, beside the landing it
/// produced — the gesture vocabulary's own growth, never the AT tier's
/// species munged onto a pointer act. The fields are the additive wire
/// rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GestureRecord {
    /// The gesture's phase — the moment the record resolves: the press
    /// that armed the drag, or the release/leave that answered it.
    pub phase: GesturePhase,
    /// The value the gesture found — the control's own scalar as the
    /// press landed (the value the drag started *from*).
    pub from: f32,
    /// The value the gesture left the control holding — the drag's
    /// landing. `None` on [`GesturePhase::Started`]: the landing is the
    /// gesture's answer, not its beginning's fact.
    pub to: Option<f32>,
    /// The control the gesture rides (the funnel's stamp, ADR-0052):
    /// the AccessKit-stable id hash of the node the pointer route
    /// resolved — the deepest hit that captured. `None` when the fact
    /// queued outside a routed pointer event (the honest absence, the
    /// action stamp's own rule).
    pub target: Option<u64>,
    /// The bound the drag's landing pinned (M48, ADR-0054, ADR-0053's
    /// first kill rule): `Some(species)` when the landing sits at the
    /// range's own edge — the wall the pointer's ask was turned back
    /// by, named beside the landing it produced — `None` on a
    /// mid-track landing and on every [`GesturePhase::Started`] record
    /// (the beginning is not the wall's fact: the landing, and so the
    /// bound, is the gesture's answer). The gesture vocabulary's own
    /// growth — a bound jump the *pointer* asked for reads the drag's
    /// record, never the AT tier's species (the kill rule's own words).
    pub bound: Option<RangeBound>,
}

/// The range's own edges, named apart (M48, ADR-0054): the two walls a
/// bounded scalar's funnel can pin a landing against. The gesture
/// record's [`GestureRecord::bound`] and the value record's pinned
/// species both name their wall with this vocabulary — one concept, the
/// range's own edges, spoken identically wherever a landing names it.
/// The names are the wire's stable tags ([`RangeBound::name`], the
/// protocol's `snake_case` rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeBound {
    /// The range's minimum — the low wall.
    Min,
    /// The range's maximum — the high wall.
    Max,
}

impl RangeBound {
    /// The bound's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values —
    /// [`GesturePhase::name`] for gesture phases, this for the range's
    /// edges).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Min => "min",
            Self::Max => "max",
        }
    }
}

/// The drag's phase ([`GestureRecord::phase`], ADR-0052): the two
/// moments a gesture answers, named apart so a trail reader never
/// guesses which edge a record carries. The names are the wire's
/// stable tags ([`GesturePhase::name`], the protocol's `snake_case`
/// rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GesturePhase {
    /// The press armed the drag — the record carries the value the
    /// gesture found ([`GestureRecord::from`]), the landing absent.
    Started,
    /// The release or the leave answered the drag — the record carries
    /// the value the gesture left the control holding
    /// ([`GestureRecord::to`]), the beginning remembered beside it.
    Ended,
}

impl GesturePhase {
    /// The phase's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values —
    /// [`ValueAction::name`] for value-tier writes, this for gesture
    /// phases).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Ended => "ended",
        }
    }
}

/// What the scroll tier just answered (§13.3, ADR-0056): the scroll
/// trail's action vocabulary, one [`ScrollRecord`] per routed ask —
/// the wheel's notch, the AT's unit scroll, the AT's value step (the
/// scrollbar's value *is* the offset, M10/ADR-0016), the app's own
/// programmatic scroll. The names are the wire's stable tags
/// ([`ScrollAction::name`], the protocol's `snake_case` rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAction {
    /// The pointer's wheel: a notch the coalescer accumulated upstream
    /// (§10.2), the hover's hit the route the ask rides.
    Wheel,
    /// The AT's `ScrollUp`: a scroll area's scroll command, traveling
    /// by the requested unit (M10, ADR-0016).
    ScrollUp,
    /// The AT's `ScrollDown`: a scroll area's scroll command, traveling
    /// by the requested unit (M10, ADR-0016).
    ScrollDown,
    /// The AT's `Increment`: the honest value-step for a scrollbar
    /// whose value *is* the offset — one page (M10, ADR-0016).
    Increment,
    /// The AT's `Decrement`: the honest value-step for a scrollbar
    /// whose value *is* the offset — one page (M10, ADR-0016).
    Decrement,
    /// The app's own programmatic scroll — the shared handle's door
    /// (tests, flings, the app's code). No route rides behind it: the
    /// stamp's honest absence.
    ScrollTo,
    /// The content's own story (M51, ADR-0057): the shared handle's
    /// `set_content_height` re-clamping the offset when the content
    /// shrank under it — the door with no ask (the clamp is the
    /// content's story, not the ask's, ADR-0056's third kill rule's own
    /// words). No route rides behind it: the stamp's honest absence,
    /// the app door's rule — and no ask beside the landing, the field
    /// riding its honest absence (a resize is not an ask).
    ContentClamp,
}

impl ScrollAction {
    /// The action's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values —
    /// [`ValueAction::name`] for value-tier writes, this for the
    /// scroll tier's asks).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wheel => "wheel",
            Self::ScrollUp => "scroll_up",
            Self::ScrollDown => "scroll_down",
            Self::Increment => "increment",
            Self::Decrement => "decrement",
            Self::ScrollTo => "scroll_to",
            Self::ContentClamp => "content_clamp",
        }
    }
}

/// Why one scroll ask moved nothing (§13.3, ADR-0056): the honest
/// refusals the scroll tier names (the M20 refuse-don't-munge rules at
/// the tier whose funnel knows no range — ADR-0055's second kill
/// rule's own words). The refusal streams too — the fact a trail
/// reader wants is exactly the ask that never scrolled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollRefusal {
    /// The range has no span: the content fits the viewport, so no
    /// scroll exists to deliver — the offset holds where it is, the
    /// ask's wall absent (a write that moved nothing pinned nothing).
    NoScrollableContent,
    /// The ask was not a finite scalar — a non-finite wheel delta the
    /// clamp arithmetic would have spread into the scroll state (the
    /// corruption the trail's design exposed). The door refuses before
    /// the arithmetic: the state keeps the offset the ask found.
    NonFiniteAsk,
    /// The content height was not a finite scalar (M51, ADR-0057) —
    /// the ask tier's own wall (`non_finite_ask`), at the content's
    /// door: a non-finite height rode the clamp arithmetic and spread
    /// `NaN` into the offset *and* the stored span alike (the
    /// corruption class the M50 trail exposed at the wheel, still open
    /// at the content's own door — the micro-debt the record's design
    /// priced, cashed with it). The door refuses before the
    /// arithmetic: the state keeps the height and offset it held.
    NonFiniteContent,
}

impl ScrollRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoScrollableContent => "no_scrollable_content",
            Self::NonFiniteAsk => "non_finite_ask",
            Self::NonFiniteContent => "non_finite_content",
        }
    }
}

/// What the scroll tier answered one ask (§13.3, ADR-0056): the
/// record is the ask's own outcome ([`ValueOutcome`]'s shape at the
/// scroll tier) — the offset landed, or the ask was refused with its
/// named species.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollOutcome {
    /// The ask landed: the offset carries the ask's clamp — a landing
    /// the clamp moved, the ask answered, or the wall held (the pin
    /// names the wall that held a no-op against a past ask — the M49
    /// sentence at the scroll tier).
    Delivered,
    /// The ask moved nothing — [`ScrollRefusal`] names the species.
    Refused(ScrollRefusal),
}

impl ScrollOutcome {
    /// The outcome's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Refused(_) => "refused",
        }
    }
}

/// One scroll ask's record (§13.3, ADR-0056): the causal log's answer
/// to "did the scroll land — where, or why not". The scroll doors
/// (the wheel, the AT's unit scrolls and value steps, the app's own
/// handle) answered the dispatch contract in silence: a wheel past the
/// content's end clamped without a word, a wheel over a fitting list
/// moved nothing unexplained. ADR-0055's second kill rule pre-registered
/// the price — a consumer needs the wall named on a tier whose funnel
/// knows no range, "that tier's own record grows its own fact beside
/// its species, never the value record's" — and the record is the fact:
/// the landed offset beside the ask (the M44 shape at the scroll tier),
/// the wall the clamp pinned ([`RangeBound`], ADR-0054's one named-edges
/// concept spoken identically wherever a landing names it), the
/// refusals named. The record carries the offset the list holds as the
/// record resolves — the clamp's result on a delivery, the untouched
/// offset on a refusal: a scroll area's offset is the tree's own
/// vocabulary the AT already reads, never a secret. The control is the
/// funnel's own stamp — the wheel route knows the hit it resolved, the
/// action route the id the action arrived under (the M43/M45/M46
/// discipline at the scroll tier), the app's handle the honest absence.
/// Since M51 (ADR-0057) the content's own door rides the record too —
/// the clamp `set_content_height` answers when the content shrank under
/// the offset, the ask-side doors' wall spoken identically (ADR-0056's
/// third kill rule). Since M53 (ADR-0059) the wheel's coalesced run
/// names its magnitude too — the notional line count beside the
/// [`ScrollAction::Wheel`] action, the run's own story the distance
/// cannot reconstruct through a clamp. The fields are the additive
/// wire rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollRecord {
    /// The scroll-tier action the ask rode.
    pub action: ScrollAction,
    /// What the scroll area answered — the landed offset or the named
    /// refusal.
    pub outcome: ScrollOutcome,
    /// The offset the scroll area holds as the record resolves — the
    /// clamp's result on a delivery, the untouched offset on a refusal
    /// (the tree keeps the offset the app set).
    pub offset: f32,
    /// The finite offset the ask wanted, before the clamp moved it
    /// (the M44 rule at the scroll tier): the ask beside the landing —
    /// a clamp the trail could not show is one record's story now. The
    /// fact lives where the knowledge lives: the widget's own door
    /// computes the ask, the note site carries it — never an offset
    /// reconstructed from the landing. `None` when the ask never
    /// became a finite scalar: the ask itself non-finite (the funnel's
    /// wall — the wire cannot carry an infinite ask honestly, `null`
    /// would claim an absence that never was, and the refusal species
    /// names the wall). The content's own story rides the same absence
    /// (M51, ADR-0057): a resize is not an ask —
    /// [`ScrollAction::ContentClamp`] never carries one.
    pub requested: Option<f32>,
    /// The control the ask rode (the funnel's stamp, ADR-0056): the
    /// AccessKit-stable id hash of the node the wheel route resolved —
    /// the deepest accepting hit under the hover — or the id the AT
    /// action arrived under. `None` when the ask rode no route: the
    /// app's own handle door (the honest absence, the stamp rule's
    /// shape).
    pub target: Option<u64>,
    /// The wall the clamp pinned the landing at (ADR-0055's second
    /// kill rule): `Some(species)` when the ask was turned back at the
    /// range's own edge — the landing at the wall, the ask somewhere
    /// else (the clamp, the wall that held a no-op against a past ask)
    /// — `None` when the ask wanted the wall exactly (the landing *is*
    /// the ask), on every landing mid-range, and on every refusal (a
    /// write that moved nothing pinned nothing). The ask's own wall,
    /// beside the species — never a second action species (the kill
    /// rule's own words).
    pub pinned: Option<RangeBound>,
    /// The asker's unit on the AT's unit scrolls and value steps
    /// (M52, ADR-0058 — ADR-0056's first kill rule cashed; M54
    /// (ADR-0060) — ADR-0058's first kill rule cashed): one item or
    /// one page beside the [`ScrollAction::ScrollUp`] and
    /// [`ScrollAction::ScrollDown`] actions, the ask the distance is
    /// priced against, and the step's page beside the
    /// [`ScrollAction::Increment`] and [`ScrollAction::Decrement`]
    /// species — the value step's page is a unit-like distance,
    /// attributed the same way (the step steps a page, always, by the
    /// action vocabulary's own contract). The honest absence on the
    /// stories with no unit: the wheel's notch is a coalesced run,
    /// not a unit ask (M53 priced it its own magnitude instead); the
    /// app's own door and the content's own clamp are stories with no
    /// unit at all.
    pub unit: Option<crate::widget::ScrollUnit>,
    /// The wheel run's notional line count (M53, ADR-0059 — ADR-0058's
    /// second kill rule cashed): the coalesced run's magnitude priced
    /// by the platform's own convention — one line = 40 px (§10.2),
    /// the exact rule the platform's wheel conversion prices
    /// `LineDelta` with, so a notch answers 1.0 and the trackpad's
    /// pixel runs answer their honest fraction — beside the
    /// [`ScrollAction::Wheel`] action only, computed at the door that
    /// already holds the delta, before the clamp arithmetic (a pinned
    /// run still names the run it was: the count is the ask's story,
    /// never the landing's). The honest absence on every other action:
    /// the AT's unit scrolls ask by their unit (M52 — a second
    /// magnitude would be the same story told twice), the value steps
    /// by the page (the step is the distance), the app's own door and
    /// the content's own clamp are stories with no wheel run behind
    /// them. The non-finite run rides the absence too: the ask never
    /// became a finite scalar, the refusal (`non_finite_ask`) names
    /// the wall, and a count of a run that never was would be the lie
    /// the M44 rule forbids.
    pub lines: Option<f32>,
}

/// The text-write action vocabulary (M55, ADR-0061): the three
/// asks a screen reader can make at a field's write door. The names
/// are the wire's stable tags ([`TextWriteAction::name`], the
/// protocol's `snake_case` rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWriteAction {
    /// The AT's `SetValue`: the whole value replaced — the same write
    /// the view's reconciliation makes, the journal resetting beneath
    /// it (ADR-0021's reset rule: undo never fights the write).
    SetValue,
    /// The AT's `ReplaceSelectedText`: the selection's span replaced
    /// by the payload, like a paste the reader made (M11, ADR-0017).
    ReplaceSelection,
    /// The AT's `SetTextSelection`: the selection moved directly, the
    /// forgiving read side flooring and clamping the ask (M11).
    SetSelection,
}

impl TextWriteAction {
    /// The action's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SetValue => "set_value",
            Self::ReplaceSelection => "replace_selection",
            Self::SetSelection => "set_selection",
        }
    }
}

/// Who made the write (M56, ADR-0062): the attribution every
/// [`TextWriteRecord`] carries beside its action — the M30 rule at the
/// text tier. One write vocabulary, two doors: a trail reader can
/// answer "did the screen reader rewrite this field, or did the app?"
/// from the record alone, the way the focus trail answers "did the
/// user navigate, or did the app?" from its rider
/// ([`FocusRider`], ADR-0036). The door is the route's name, not a
/// second channel: the write behind the doors is the field's own
/// whole-value rule, and the wire carries the source's
/// [`TextWriteSource::name`] beside the action (the protocol's
/// additive rule, ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWriteSource {
    /// The AT's ask at the input's action door — the dispatch funnel
    /// routed it (M55, ADR-0061; the stamp beside the source names the
    /// control the ask arrived under).
    AtDoor,
    /// The app's own write — the imperative `set_value` door (the
    /// mutate seam) and the view reconciliation's patches, which ride
    /// the same door. No funnel route held the stamp: the identity is
    /// the write's own absence, honest the way the widget doors'
    /// always were.
    AppDoor,
}

impl TextWriteSource {
    /// The source's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::AtDoor => "at_door",
            Self::AppDoor => "app_door",
        }
    }
}

/// Why a text write moved nothing (M55, ADR-0061): the closed door's
/// species. The availability tier (M22, ADR-0028) holds an
/// unavailable field's write door shut — the refusal streams the way
/// the value tier's own four do (ADR-0046's rhythm at the editing
/// tier), the ask that never entered arithmetic riding the honest
/// absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWriteRefusal {
    /// The field is unavailable — no write door (M22, ADR-0028). The
    /// same fact the value tier names `unavailable` (one concept, the
    /// availability tier, spoken identically wherever a door names
    /// it).
    Unavailable,
}

impl TextWriteRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
        }
    }
}

/// What the field answered one AT text write (M55, ADR-0061): the
/// record is the write's own outcome ([`ValueOutcome`]'s shape at the
/// editing tier) — the write landed, or it was refused with its named
/// species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWriteOutcome {
    /// The write landed: the field carries the value or selection the
    /// ask asked for.
    Delivered,
    /// The write moved nothing — [`TextWriteRefusal`] names the
    /// species.
    Refused(TextWriteRefusal),
}

impl TextWriteOutcome {
    /// The outcome's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Refused(_) => "refused",
        }
    }
}

/// One AT text write's record (M55, ADR-0061): the causal log's
/// answer to "did the screen reader rewrite this field — what shape,
/// where did the caret land, or why not". The input's AT actions
/// (M11, ADR-0017) answered the widget's own `bool`, whose `false`
/// collapsed the closed door into the same silence as an answered
/// write — and the answered write itself left no fact a trail reader
/// could price (the journal's reset and record echoes rode, but they
/// are the journal's facts about the depths, not the door's facts
/// about the ask). The write's shape is named by **lengths** — the
/// span the write consumed and the payload's own length, in UTF-16
/// units — never the payload itself: a field's value can carry a
/// secret, and the clipboard trail's rule (ADR-0045) is the house
/// rule ("the length names the fact, never the payload"). The
/// selection door carries the ask beside the landing (the M44 rule,
/// ADR-0050, at the text tier): the forgiving read side's floor and
/// clamp visible as data, never reconstructed. Since M43
/// (ADR-0049) the record names its control beside the action's
/// routing — the dispatch funnel knows the id the action arrived
/// under, the widget does not, so the stamp side lives on the funnel
/// and the facts queued beneath a routed action carry it. The fields
/// are the additive wire rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextWriteRecord {
    /// The text-write action the AT asked.
    pub action: TextWriteAction,
    /// What the field answered — the landed write or the named
    /// refusal.
    pub outcome: TextWriteOutcome,
    /// The span the write consumed, in UTF-16 units of the field's
    /// value — the whole value for a `set_value` (the value the field
    /// carried), the selection's span for a `replace_selection`.
    /// `None` when the write consumed no span: the selection ask (a
    /// selection replaces nothing) and every refusal (a write that
    /// moved nothing consumed nothing).
    pub replaced: Option<usize>,
    /// The payload's own length, in UTF-16 units — the write's shape
    /// named by its size, the payload itself never riding the log.
    /// `None` when the write carried no payload: the selection ask and
    /// every refusal.
    pub inserted: Option<usize>,
    /// The selection the AT asked for, before the forgiving read side
    /// floored or clamped it (the ask beside the landing, the M44
    /// rule at the text tier). `None` on the writes that ask with a
    /// payload — the payload's shape rides `replaced`/`inserted` —
    /// and on every refusal (a write that moved nothing asked
    /// nothing the door could forgive).
    pub requested: Option<crate::widget::TextSelection>,
    /// The selection the field carries as the record resolves — the
    /// caret the write left (a whole-value write parks it at the
    /// write's end, a replacement at the insertion's end, a selection
    /// ask at its floored-and-clamped landing), or the field's own
    /// untouched selection on a refusal.
    pub selection: crate::widget::TextSelection,
    /// The control the action arrived under (the funnel's stamp, the
    /// M43 rule): the AccessKit-stable id hash the dispatch funnel
    /// routed — the identity a screen reader's client, an inspector
    /// pane, or a test correlates. `None` when the fact queued
    /// outside a routed action (the honest absence, the stamp's own
    /// rule).
    pub target: Option<u64>,
    /// The door the write rode (M56, ADR-0062 — the M30 attribution
    /// rule at the text tier): the AT's ask at the action door
    /// ([`TextWriteSource::AtDoor`]) or the app's own write at the
    /// imperative door the view reconciliation's patches ride
    /// ([`TextWriteSource::AppDoor`]) — the trail reader's answer to
    /// "did the screen reader rewrite this field, or did the app?",
    /// named where the record is built, never reconstructed.
    pub source: TextWriteSource,
}

/// Why the boundary dropped an ask before any widget's door (M57,
/// ADR-0063): the route's own refusal species. A drop is never a
/// munged widget-door fact — no door ever answered — so the route
/// names its own answers, the way the value tier named its four
/// (ADR-0046's rhythm at the boundary tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteDropSpecies {
    /// The action has no consumer in the framework's vocabulary —
    /// mapping it onto a press would be a correctness bug (the
    /// boundary's own rule, ADR-0016), so it drops.
    UnknownAction,
    /// The ask needs a payload the request did not carry — a
    /// `SetValue` or `ReplaceSelectedText` with no `Value` data. A
    /// mapping without the payload would be a munge; the ask drops.
    NoPayload,
    /// The `SetTextSelection` was aimed at another node — Alacer's
    /// editable nodes carry their own value, the character indexes
    /// cannot be honored here, and the ask drops (the single-node
    /// selection contract, ADR-0017).
    OffTargetSelection,
    /// The ask was routed to an id no tree node carries — the funnel's
    /// own defensive drop, refused before any widget's door.
    UnknownTarget,
}

impl RouteDropSpecies {
    /// The species' name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnknownAction => "unknown_action",
            Self::NoPayload => "no_payload",
            Self::OffTargetSelection => "off_target_selection",
            Self::UnknownTarget => "unknown_target",
        }
    }
}

/// One boundary drop's record (M57, ADR-0063): the causal log's answer
/// to "the AT asked — why did no door ever answer". The record is the
/// route's own fact, queued where the drop resolves — the platform
/// boundary's mapping arms and the dispatch funnel's unknown-target
/// refusal — and never at a widget's door. The action is named by its
/// stable `snake_case` tag (the protocol's field-value rule), the
/// species naming why the boundary refused beside it, and the aimed id
/// riding verbatim — the boundary's aim, never re-resolved against the
/// tree (a re-resolved aim would be the funnel's answer, not the AT's
/// ask). `Copy` like the records beside it: two strings' lengths, an
/// id, a species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteDropRecord {
    /// The dropped action's stable tag (`set_value`, `focus`, …) — the
    /// platform vocabulary's own name for what arrived, never a munge
    /// onto the core action it did not become.
    pub action: &'static str,
    /// Why the boundary dropped the ask.
    pub species: RouteDropSpecies,
    /// The id the request aimed at, verbatim — the identity the AT's
    /// client correlates against the tree it believes exists. `None`
    /// when the boundary had no aim to name (an ask with no target at
    /// all).
    pub target: Option<u64>,
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
    /// A value-tier write answered — the value trail (ADR-0046): the
    /// `SetValue`, `Increment`, or `Decrement` the AT asked, the landed
    /// value beside the outcome, the refusals named. Streams with the
    /// frame's own records (the tick's start drain, the clipboard
    /// trail's shape — the fact queues where the write resolves, one
    /// frame after the act).
    Value(ValueRecord),
    /// A drag gesture answered — the gesture trail (ADR-0052): the
    /// press that armed the drag beside the value it found, or the
    /// release/leave that resolved it beside the value it landed, one
    /// record per phase, never per move (the flood rule). Streams with
    /// the frame's own records (the tick's start drain, the value
    /// drain's shape — the fact queues where the gesture's door
    /// resolves, one frame after the phase).
    Gesture(GestureRecord),
    /// A scroll ask answered — the scroll trail (ADR-0056): the wheel,
    /// the AT's unit scroll or value step, or the app's own programmatic
    /// scroll, the landed offset beside the ask, the refusals named.
    /// Streams with the frame's own records (the tick's start drain,
    /// the gesture drain's shape — the fact queues where the door
    /// resolves, one frame after the ask).
    Scroll(ScrollRecord),
    /// An AT text write answered — the text-write trail (ADR-0061):
    /// the `SetValue`, `ReplaceSelectedText`, or `SetTextSelection` a
    /// screen reader asked at a field's write door, the write's shape
    /// named by lengths beside the landing selection, the refusals
    /// named. Streams with the frame's own records (the tick's start
    /// drain, the scroll drain's shape — the fact queues where the
    /// door resolves, one frame after the write). Since M56 (ADR-0062)
    /// the record names the door the write rode — the AT's ask or the
    /// app's own write.
    TextWrite(TextWriteRecord),
    /// An ask was dropped before any widget's door — the route-drop
    /// trail (ADR-0063): the unknown action, the payloadless write,
    /// the off-target selection, the unknown target — the boundary's
    /// defensive refusals, named where the drop resolves. Streams with
    /// the frame's own records (the tick's start drain, the text-write
    /// drain's shape — the fact queues where the route refuses, one
    /// frame after the drop).
    RouteDrop(RouteDropRecord),
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
