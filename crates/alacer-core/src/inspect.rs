//! The devtools observation seam (§13.3): the causal vocabulary the
//! inspector consumes, and the observer sink the pipeline notifies.
//!
//! Everything here is **dev-mode and opt-in**: an unset observer and a
//! disabled capture flag cost one branch per frame — the A7 idle contract
//! and the M1 overhead gate are unchanged. With them on, the seam answers
//! the Causality inspector's questions from recorded truth:
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
//! | "Did the AT's press land — or why not?" | [`InspectEvent::Action`] — the action trail: one record per activation ask the funnel routes (the landing or the named refusal species beside the aim), streamed the tick after the ask resolves (ADR-0075) |
//! | "Why did the pointer's shape move — which face named it?" | [`InspectEvent::Cursor`] — the cursor trail: one record per resolved change (the style beside the source that named it), streamed the tick after the hover moved (ADR-0157) |

use crate::announce::Politeness;
use crate::arena::NodeId;
use crate::color::Color;
use crate::damage::Damage;
use crate::geom::Rect;
use crate::geom::Size;
use crate::history::EditAction;
use crate::keymap::{Chord, KeymapConsult};
use crate::mask::Change;
use crate::passes::{DegradationStep, FocusMove, FocusMoveOutcome, PassKind};
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
    /// The degradation ladder's deepest rung this frame reached (§14.4)
    /// — what a frame over budget gave up, recorded where the decision
    /// resolved. [`DegradationStep::None`] on disarmed pipelines.
    pub degradation_step: DegradationStep,
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
    /// The field the journal serves (M66, ADR-0072 — the M43 stamp rule
    /// at the journal tier): the AccessKit-stable id the routed event
    /// arrived under, stamped where the echo resolves — the key route's
    /// and the AT action route's guard filling what the widget cannot
    /// know. `None` when the journal answered outside any route (the
    /// app's direct rides — the honest absence, the M43 rule's own
    /// word).
    pub target: Option<u64>,
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

/// The hover's resolved cursor fact (M153, ADR-0157): one record per
/// resolved *change* — the style the platform now applies beside the
/// source that named it. The length rule at the cursor tier is the
/// source's own: the widget's node id and role ride beside the style
/// (the words the record needs to answer "which face named this"), and
/// the label never does — a hover is not a payload, and the log is a
/// devtools surface, not a shadow tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorRecord {
    /// The style the hover resolves to — the shape the platform's delta
    /// applies to the window.
    pub style: crate::cursor::CursorStyle,
    /// The hover the style rode: the widget whose hook named it (the
    /// node and its role beside it), or the honest default when nothing
    /// accepted the hover.
    pub source: CursorSource,
}

/// Where one cursor fact's style came from (M153, ADR-0157): the hover
/// resolution's two honest answers — a widget named the shape, or
/// nothing stood under the pointer and the default answered. The names
/// are the wire's stable tags (the protocol's `snake_case` rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorSource {
    /// Nothing accepting stood under the pointer: the platform's
    /// default — the absence, named as a source, never guessed onto a
    /// widget that did not answer.
    Default,
    /// The widget named the style — the node and its role ride beside
    /// (the lengths rule: the label never does).
    Widget {
        /// The node whose hook named the style (the id-path hash's
        /// vocabulary, the wire's stable `u64`).
        node: u64,
        /// The node's semantic role — the face the style names, the
        /// word the log reads beside the node id.
        role: crate::widget::Role,
    },
}

impl CursorSource {
    /// The source's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Widget { .. } => "widget",
        }
    }
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
    /// The AT's `ScrollLeft` (M91, ADR-0095): the horizontal
    /// vocabulary beside the vertical pair — a scroll area's scroll
    /// command on the x axis, traveling by the requested unit (an item
    /// is one line, a page is the viewport's width). The record's
    /// horizontal fact carries the ask's story beside the vertical
    /// fields riding the identity answer (the axis unasked, the same
    /// convention the content door's width half answers by).
    ScrollLeft,
    /// The AT's `ScrollRight` (M91, ADR-0095): the horizontal
    /// vocabulary beside the vertical pair — a scroll area's scroll
    /// command on the x axis, traveling by the requested unit (an item
    /// is one line, a page is the viewport's width). The record's
    /// horizontal fact carries the ask's story beside the vertical
    /// fields riding the identity answer (the axis unasked, the same
    /// convention the content door's width half answers by).
    ScrollRight,
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
    /// The geometry-driven ask (M88, ADR-0092): the pipeline's
    /// bring-into-view door read the bounds ledger, computed the
    /// minimal offset that lands the aimed node fully inside the
    /// nearest scrollable ancestor's viewport, and handed the ask to
    /// the container's own clamp arithmetic. The record's `target`
    /// names the container whose offset moved (the route's stamp) and
    /// the additive `aimed` field names the node the ask was *for*
    /// (the M30 attribution rule at the scroll tier) — the two ids one
    /// ask produced, never a munge of either onto the other.
    BringIntoView,
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
            Self::ScrollLeft => "scroll_left",
            Self::ScrollRight => "scroll_right",
            Self::Increment => "increment",
            Self::Decrement => "decrement",
            Self::ScrollTo => "scroll_to",
            Self::BringIntoView => "bring_into_view",
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
    /// The node the ask was *for* (M88, ADR-0092): the
    /// AccessKit-stable id of the node the bring-into-view ask aimed
    /// at — the record's `target` names the container whose offset
    /// moved, this names the node the ask was for, the two ids one ask
    /// produced (the M30 attribution rule at the scroll tier, the
    /// aimed id verbatim the route-drop trail's rule). `None` on every
    /// ask that rode no aimed node: the wheel, the AT's unit scrolls
    /// and value steps, the app's raw-offset door, the content's own
    /// clamp — the absence honest, never a guessed id. Additive at
    /// version 1 (the additive-field rule the wire's scroll event
    /// carries): clients that ignore unknown fields are unaffected.
    pub aimed: Option<u64>,
    /// The horizontal half of the ask's story (M89, ADR-0093 —
    /// ADR-0056's own axis rule cashed): the axis's outcome, landed
    /// offset, ask, and wall beside the vertical fields, the same
    /// rules spoken per axis — the door priced the ask from the
    /// geometry, the container's declared span answered it (or
    /// refused it by its own named wall). `None` on the stories with
    /// no horizontal half: the AT's unit scrolls and value steps ride
    /// their vertical vocabulary untouched, the content's own clamp
    /// is a height's story — the absence honest, never a guessed
    /// zero. Additive at version 1 (the additive-field rule the
    /// wire's scroll event carries): clients that ignore unknown
    /// fields are unaffected.
    pub horizontal: Option<HorizontalScrollFact>,
}

/// The horizontal half of one scroll ask's record (M89, ADR-0093):
/// the axis's own story, the same shape the vertical fields ride —
/// the outcome beside the landed offset, the ask beside both (the
/// M44 rule at the scroll tier, spoken per axis), the wall the
/// clamp pinned (ADR-0055's second kill rule, per axis). The record's
/// `horizontal` carries it as a whole: one additive field, the fact
/// never split across three loose scalars a reader could
/// mis-pair with the vertical story.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizontalScrollFact {
    /// What the container's clamp answered on the horizontal axis —
    /// delivered, or the named refusal species (an undeclared span
    /// answers `no_scrollable_content`, the wall named where the
    /// consumer can read it).
    pub outcome: ScrollOutcome,
    /// The offset the container holds on the horizontal axis as the
    /// record resolves — the clamp's result on a delivery, the
    /// untouched offset on a refusal.
    pub offset: f32,
    /// The finite offset the horizontal ask wanted, before the clamp
    /// moved it — the ask beside the landing, per axis. `None` when
    /// the ask never became a finite scalar (the funnel's wall).
    pub requested: Option<f32>,
    /// The wall the clamp pinned the horizontal landing at — `None`
    /// when the landing is the ask, on every landing mid-range, and
    /// on every refusal (a write that moved nothing pinned nothing).
    pub pinned: Option<RangeBound>,
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
    /// The field's value is the app's (M105, ADR-0109): the read-only
    /// tier's wall — the write ask (`SetValue`,
    /// `ReplaceSelectedText`) landed on the declared word. The
    /// selection door answers beside it: a read-only field reads.
    /// The availability wall stays the outer one, the read-only
    /// species the inner tier's.
    ReadOnly,
}

impl TextWriteRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::ReadOnly => "read_only",
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
    /// The event rode no route at all — no field has focus, so the ask
    /// dropped before any widget's door (M59, ADR-0065): the IME's
    /// committed composition, the platform's other silence, named where
    /// the boundary refuses. Since M65 (ADR-0071) this is the
    /// *never-focused* half of the silence — the focused node's death
    /// rides its own species beside it ([`RouteDropSpecies::DeadFocus`],
    /// the split the kill rule pre-registered).
    NoFocusedField,
    /// The field that held focus has gone — the focused node died, and
    /// the event addressed a door the tree no longer carries (M65,
    /// ADR-0071, ADR-0065's first kill rule): a committed composition
    /// with nowhere to land prices the *why* apart from the
    /// never-focused one. The aim rides its honest absence either way —
    /// the drop *is* the missing field.
    DeadFocus,
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
            Self::NoFocusedField => "no_focused_field",
            Self::DeadFocus => "dead_focus",
        }
    }
}

/// Which route dropped the ask (M65, ADR-0071 — the M30 rider rule at
/// the route-drop tier): the boundary's mapping arms, the dispatch
/// funnel's own refusal, and the IME route's silence all resolve drops,
/// and a trail reader asking "which route refused my ask" reads them
/// apart from the record alone. The attribution is declared where the
/// drop resolves — the note site knows its own tier, the record never
/// guesses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteDropRider {
    /// The platform boundary's mapping arms — the ask never became a
    /// core action (the unknown action, the payloadless write, the
    /// off-target selection; ADR-0063's original four).
    Boundary,
    /// The dispatch funnel's own refusal — the ask routed into the
    /// pipeline and was refused there, before any widget's door (the
    /// unknown target; ADR-0063).
    Funnel,
    /// The IME route — the composing event no live field held (M59,
    /// ADR-0065; the split silence rides the same rider, M65).
    ImeRoute,
}

impl RouteDropRider {
    /// The rider's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Boundary => "boundary",
            Self::Funnel => "funnel",
            Self::ImeRoute => "ime_route",
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
/// ask). Since M65 (ADR-0071) the record names its rider too — the
/// route that dropped the ask, the M30 attribution rule at the route
/// tier. `Copy` like the records beside it: two strings' lengths, an
/// id, a species, a rider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteDropRecord {
    /// The dropped action's stable tag (`set_value`, `focus`, …) — the
    /// platform vocabulary's own name for what arrived, never a munge
    /// onto the core action it did not become.
    pub action: &'static str,
    /// Why the boundary dropped the ask.
    pub species: RouteDropSpecies,
    /// Which route dropped the ask (M65, ADR-0071): the platform
    /// boundary's mapping arms, the dispatch funnel's refusal, or the
    /// IME route's silence — declared where the drop resolves, never
    /// guessed from the species.
    pub rider: RouteDropRider,
    /// The id the request aimed at, verbatim — the identity the AT's
    /// client correlates against the tree it believes exists. `None`
    /// when the boundary had no aim to name (an ask with no target at
    /// all).
    pub target: Option<u64>,
}

/// Who drove the edit the typing door landed (M58, ADR-0064): the
/// M30 attribution rule at the editing tier's typing door. The
/// keyboard's own keys and the IME's commits both ride the field's
/// one edit funnel, and a trail reader can only price a typed run
/// against a committed composition when the record names its driver.
/// Since M60 (ADR-0066) the clipboard's chords ride their own driver
/// too — a paste or a cut is the clipboard's field change, never the
/// keyboard's and never a munge onto the keys it mirrors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEditDriver {
    /// The keyboard's own keys — a typed character, the editing keys.
    Keyboard,
    /// The IME's committed composition — the preedit the field showed
    /// landing as final text.
    Ime,
    /// The clipboard's chords — the paste's insertion and the cut's
    /// deletion, the field's change behind the round trip (M60,
    /// ADR-0066; the round trip itself keeps the clipboard trail's
    /// fact, ADR-0045).
    Clipboard,
}

impl KeyEditDriver {
    /// The driver's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Ime => "ime",
            Self::Clipboard => "clipboard",
        }
    }
}

/// What the typing door did to the field's value (M58, ADR-0064): the
/// editing actions the keyboard's keys and the IME's commits carry,
/// named where the edit resolves. Every species is the field's own
/// edit vocabulary — never a munge onto the AT's action tier (a typed
/// character is not a `SetValue`, the record is the typing door's, not
/// the action route's) and never a collapse of the word chords into
/// the single-unit deletes they mirror. Since M60 (ADR-0066) the
/// clipboard's chords carry their own species — the paste's insertion
/// and the cut's deletion named by the chord that drove them, never
/// munged onto the insert and delete they mirror (the cut that
/// streamed a delete fact and a cut fact would double the edit, the
/// round trip's own rule, ADR-0020).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEditAction {
    /// Text landed at the caret (or onto the selection it consumed) —
    /// a typed character, an IME's committed composition.
    Insert,
    /// The text before the caret (or the active selection) went — the
    /// Backspace key, one unit or one scalar value at a time.
    Backspace,
    /// The text after the caret (or the active selection) went — the
    /// forward Delete key.
    Delete,
    /// The word run behind the caret went — the Ctrl/Alt+Backspace
    /// word chord (ADR-0019).
    WordDeleteBack,
    /// The word run ahead of the caret went — the Ctrl/Alt+Delete
    /// word chord, the mirror of [`KeyEditAction::WordDeleteBack`].
    WordDeleteForward,
    /// The clipboard's text landed at the caret (or onto the selection
    /// it consumed) — the paste chord (M60, ADR-0066), the field's
    /// change behind the round trip (the payload's shape named by
    /// lengths, never the payload).
    Paste,
    /// The active selection moved out to the clipboard and went — the
    /// cut chord (M60, ADR-0066), the deletion that follows an
    /// accepted write (ADR-0020's refuse-don't-munge rule) named by
    /// the chord, never a munged [`KeyEditAction::Backspace`].
    Cut,
    /// The selection moved out to the clipboard and the field kept it —
    /// the copy chord (M68, ADR-0074), the ask whose field change is
    /// nothing. A delivered copy answers no fact (the m60 rule: the
    /// wall prices edit asks, and a delivered copy is none); the
    /// payloadless copy — the read that found nothing to move — names
    /// this action beside its refusal (the ask the door's own match
    /// named, the M62 rule at the read tier).
    Copy,
    /// The journal's newest undone edit came back (M105, ADR-0109) —
    /// the redo chord's ask, the field change the journal drives when
    /// the door answers it. A wall that turns the chord away names the
    /// ask here (the read-only tier's): the undo word's mirror.
    Redo,
    /// The newest landed edit went back (M105, ADR-0109) — the undo
    /// chord's ask, the field change the journal drives when the door
    /// answers it. A wall that turns the chord away names the ask here:
    /// the journal chords price their walls like every other edit ask.
    Undo,
}

impl KeyEditAction {
    /// The action's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Insert => "insert",
            Self::Backspace => "backspace",
            Self::Delete => "delete",
            Self::WordDeleteBack => "word_delete_back",
            Self::WordDeleteForward => "word_delete_forward",
            Self::Paste => "paste",
            Self::Cut => "cut",
            Self::Copy => "copy",
            Self::Redo => "redo",
            Self::Undo => "undo",
        }
    }
}

/// Why the typing door refused an edit (M62, ADR-0068): the wall's own
/// species, the value tier's refusal vocabulary ([`ValueRefusal`],
/// ADR-0046) named at the typing tier. A refusal prices the *ask* —
/// the edit an open door would have answered — never a delivered
/// shape, and never a munge onto the landing selection it left
/// untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEditRefusal {
    /// The availability tier's wall (M22, ADR-0028): the field is held
    /// down, the routed edit key ignored so the framework keeps it
    /// (Tab cycling), the IME's commit landing nowhere. The value
    /// tier's own species (`unavailable` — the tag the AT tier's
    /// closed door streams, ADR-0027) at the typing tier.
    Unavailable,
    /// The read's wall (M68, ADR-0074): the door is open, the chord's
    /// read answered, and the selection was empty — nothing was there
    /// to move out (the payloadless copy or cut). The field's own
    /// answer beside the clipboard trail's `NoSelection` round trip —
    /// one fact per trail, never a double-priced ask, the delivered
    /// chord's two-fact shape mirrored at the wall.
    EmptySelection,
    /// The read-only tier's wall (M105, ADR-0109): the field is
    /// available and focusable — the read doors answer — but its value
    /// is the app's, so the write asks (typing, deletes, paste, cut,
    /// undo, redo, the IME's commit) land on the declared word. The
    /// availability wall stays the outer one: a field both disabled and
    /// read-only names [`KeyEditRefusal::Unavailable`], the read-only
    /// species the inner tier's.
    ReadOnly,
}

impl KeyEditRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::EmptySelection => "empty_selection",
            Self::ReadOnly => "read_only",
        }
    }
}

/// What the typing door answered one edit (M62, ADR-0068): the record
/// is the edit's own outcome (the value trail's own refusal
/// vocabulary, [`ValueOutcome`]'s shape at the typing tier) — the
/// edit landed with the shape the record prices, or the door refused
/// with its named species and the shape priced at nothing. The M58
/// records all landed; the field is the additive default (the wire
/// rule ADR-0010 names), never a munged delivered edit — a refusal
/// behind a delivered shape would double the ask the wall never
/// answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEditOutcome {
    /// The edit landed: the shape the record prices (`replaced` /
    /// `inserted`) is the field's own change (the M58 shape).
    Landed,
    /// The door moved nothing — [`KeyEditRefusal`] names the species,
    /// the shape priced at nothing (the value and the selection
    /// untouched).
    Refused(KeyEditRefusal),
}

impl KeyEditOutcome {
    /// The outcome's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Landed => "landed",
            Self::Refused(_) => "refused",
        }
    }
}

/// One typing-door edit's record (M58, ADR-0064): the causal log's
/// answer to "what did the typist change in this field". The journal's
/// own [`EditRecord`] streams the depths the edit left; this record
/// rides beside it with the change's *shape* — the UTF-16 spans the
/// edit consumed and inserted, never the payload (a field's value can
/// carry a secret, the clipboard trail's rule, ADR-0045) — the landing
/// selection, the driver who wrote it, and whether the journal folded
/// the edit into its tip. Since M60 (ADR-0066) the clipboard's chords
/// name their field changes here too — the paste's shape and the cut's
/// span behind their own actions and driver — while the round trip
/// keeps the clipboard trail's fact (one fact per trail, never a
/// double-priced edit). Since M62 (ADR-0068) the record names what the
/// door answered — the landed edit or the wall's refusal ([`KeyEditOutcome`],
/// ADR-0064's second kill rule cashed): the routed edit key a disabled
/// field ignored and the IME's commit a closed field turned away
/// stream their ask behind the wall's species, the shape priced at
/// nothing, never a munged delivered edit. `Copy` like the records
/// beside it: lengths, a selection, a driver, an action, an outcome, a
/// bit, an id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEditRecord {
    /// What the typing door did.
    pub action: KeyEditAction,
    /// Who drove the edit — the keyboard's keys or the IME's commit.
    pub driver: KeyEditDriver,
    /// The span the edit consumed, in UTF-16 units of the field's
    /// value — an active selection's span, the deleted unit's run; `0`
    /// on a pure insertion. The shape named by its size — the payload
    /// itself never rides the log.
    pub replaced: usize,
    /// The payload's own length, in UTF-16 units — the insertion's
    /// size beside the span it consumed; `0` on a pure deletion.
    pub inserted: usize,
    /// The selection the field carries as the edit resolves — the
    /// caret the edit left, anchor collapsed onto it (the M11 edit
    /// rule). A field's selection is the tree's own vocabulary the AT
    /// already reads, never a secret. On a refusal the selection is
    /// the untouched one — the wall moved nothing.
    pub selection: crate::widget::TextSelection,
    /// What the door answered (M62, ADR-0068): the landed edit or the
    /// wall's refusal (the value trail's own vocabulary, ADR-0046) —
    /// the ask the open door would have answered named by its action,
    /// the shape priced at nothing on a refusal, never a munged
    /// delivered edit.
    pub outcome: KeyEditOutcome,
    /// Whether the journal folded this edit into its tip (the
    /// pure-append rule, ADR-0021) — the flood rule's own bit: the
    /// trail supersedes the run's record so N keystrokes in one
    /// coalesced run land as one fact, never a per-keystroke flood.
    pub coalesced: bool,
    /// The control the edit rode (the funnel's stamp, the M43 rule at
    /// the typing tier — ADR-0049): the AccessKit-stable id the key
    /// route held; `None` when the fact rode no route (the app's
    /// direct mutate seam) or the route holds no stamp.
    pub target: Option<u64>,
}

/// What the composing door answered (M61, ADR-0067): the phases the
/// field's composing state walks, named where the preedit resolves.
/// The preedit is not an edit — it lands nowhere, the journal never
/// hears of it — so the vocabulary never munges a composition onto the
/// edit tier's actions: a commit's landing is the key-edit trail's
/// fact (M58, ADR-0064), the composing state's own story is this one's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreeditAction {
    /// The IME owns the canvas: the field shows the composition's
    /// inline shape at the caret — the live session's own fact, one
    /// record superseded in place while the session grows (the flood
    /// rule at the composing tier, never an echo per composition
    /// frame).
    Composing,
    /// The composing state ended with no landing — the empty preedit
    /// the platform delivers when the session dissolves. The shape is
    /// the field's honest `0`: whatever the session had shown is named
    /// by the `Composing` records beside it. A commit's landing rides
    /// the key-edit trail instead — one fact per trail.
    Cleared,
}

impl PreeditAction {
    /// The action's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Composing => "composing",
            Self::Cleared => "cleared",
        }
    }
}

/// One composing-state fact (M61, ADR-0067): the causal log's answer
/// to "what is the field showing while the IME composes". The
/// composition's inline shape named by its UTF-16 length — never the
/// payload (a field's value can carry a secret, ADR-0011's rule at the
/// composition tier; the secret-entry field suppresses the preedit
/// echo at the platform, and the trail keeps the same discipline) —
/// the platform's cursor inside the preedit beside it, and the funnel's
/// stamp (the M43 rule at the composing tier). One live session is one
/// record: each arrival supersedes the session's record in place while
/// nobody drained, the shape growing to the composition's — `Copy`
/// like the records beside it: a phase, a length, a cursor, an id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreeditRecord {
    /// What the composing door answered.
    pub action: PreeditAction,
    /// The composition's inline shape, in UTF-16 units of the preedit
    /// text — `0` on a cleared state. The shape named by its size, the
    /// payload itself never riding the log (the secret-safety rule).
    pub shape: usize,
    /// The platform's cursor inside the preedit, in UTF-16 units of
    /// the preedit text — the caret the composition paints within its
    /// own inline shape; `None` when the platform named none. A
    /// preedit-local position, never a field offset (the composition
    /// lands nowhere).
    pub cursor: Option<usize>,
    /// The control the composition rode (the funnel's stamp, the M43
    /// rule at the composing tier — ADR-0049): the AccessKit-stable id
    /// the IME route held; `None` when the fact rode no route.
    pub target: Option<u64>,
}

/// The adapter's full-tree echo (§13.3, ADR-0073): the a11y tier's own
/// fact — the assistive technology connected or reconnected, and the
/// sink answered with the whole resolved tree, the same builder the
/// deltas use. The tree is named by its node count: the shape rides,
/// the payload itself never does (the lengths rule at every tier — a
/// resolved tree carries the app's labels, and a label can carry a
/// secret). `Copy` like the records beside it: one count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct A11yEchoRecord {
    /// The resolved tree's size — the node count the full-tree build
    /// answered with, the same answer the adapter forwards. `0` never
    /// rides: an activation before the first frame resolves no tree,
    /// and no fact streams for it (the honest absence — the trail
    /// never invents a size).
    pub nodes: usize,
}

/// The activation ask's vocabulary (§13.3, ADR-0075): the asks the
/// action trail prices, one [`ActionRecord`] per ask. The names are the
/// wire's stable tags ([`ActionAsk::name`], the protocol's
/// `snake_case` rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionAsk {
    /// The default accessibility action — "press" for buttons, "toggle"
    /// for checkboxes: the AT's activation ask, the door the keyboard's
    /// Space/Enter answers like a click (the M10 contract's faithful
    /// routing, ADR-0016).
    Press,
}

impl ActionAsk {
    /// The ask's name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Press => "press",
        }
    }
}

/// Why one activation ask moved nothing (§13.3, ADR-0075): the two
/// honest refusals the action tier names — the collapsed `false` the
/// door's `bool` answered with, split where both facts live. The
/// refusal streams too — the fact a trail reader wants is exactly the
/// ask that never landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionRefusal {
    /// The target grew no door that answers the ask — a press aimed at
    /// a heading, an increment aimed at a button: the ask fell outside
    /// every tier the target answers, and no widget door ever consumed
    /// it (the M10 route's honest refusal, now a named fact).
    NoDoor,
    /// The availability tier holds the control down (M22, ADR-0028) —
    /// every door refuses before the request is even parsed, and the
    /// tree keeps the state the app wrote. The route computes the
    /// species from the tree's one `enabled` truth, never from a
    /// widget-private guess.
    Unavailable,
}

impl ActionRefusal {
    /// The refusal's name, the wire vocabulary's `snake_case` tag
    /// (the protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoDoor => "no_door",
            Self::Unavailable => "unavailable",
        }
    }
}

/// What the action tier answered one activation ask (§13.3, ADR-0075):
/// the record is the route's own outcome ([`ClipboardOutcome`]'s shape
/// at the activation tier) — the ask landed at the target's door, or
/// the ask was refused with its named species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOutcome {
    /// The ask landed: the target's door consumed it (the press the
    /// button answered, the toggle the checkbox answered).
    Delivered,
    /// The ask moved nothing — [`ActionRefusal`] names the species.
    Refused(ActionRefusal),
}

impl ActionOutcome {
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

/// Who rode the activation ask (M70, ADR-0076): the attribution every
/// [`ActionRecord`] carries beside its ask — the M30 rider rule at the
/// activation tier. The press has two doors: the platform's action
/// route (the AT's activation ask, the dispatch funnel the AccessKit
/// adapter drives) and the keyboard's own (the focused control's
/// Space/Enter, the M10 parity the widget answers like a click). Until
/// the record named its rider, a trail reader could not answer "did the
/// screen reader press it, or did the user hit Space". The rider is the
/// route's name, not a second channel: the doors behind the riders are
/// untouched, and the wire carries the rider's [`ActionRider::name`]
/// beside the ask (the protocol's additive rule, ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionRider {
    /// The platform's action route — the AT's activation ask the
    /// dispatch funnel routed (the door `PassPipeline::dispatch_a11y_action`
    /// is; the M10 contract's faithful routing).
    ActionRoute,
    /// The keyboard's own activation — the focused control's
    /// Space/Enter answered like a click (the M10 parity, §10.3; the
    /// button's GTK release rule, the checkbox's toggle-on-press).
    Keyboard,
}

impl ActionRider {
    /// The rider's name, the wire vocabulary's `snake_case` tag
    /// ([`ActionAsk::name`] for asks, this for riders — the protocol's
    /// stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ActionRoute => "action_route",
            Self::Keyboard => "keyboard",
        }
    }
}

/// One activation ask's record (§13.3, ADR-0075): the causal log's
/// answer to "did the AT's press land — or why not". The action tier's
/// most common ask — the default accessibility action the platform's
/// action route aims at a control — answered the funnel's own `bool`,
/// whose `false` collapsed the availability wall and the doorless
/// silence into one indistinguishable answer, and the ride the
/// M31/M33/M36/M39/M57 rhythm prices records the ask: every activation
/// ask the funnel routes streams one fact, the refused asks included.
/// The species is computed where both facts live — the route holds the
/// ask, the aimed id, the door's answer, and the tree's one `enabled`
/// truth (M22, ADR-0028) — never guessed at a widget-private call site.
/// One fact per trail (the M39 rule at the activation tier): the asks
/// whose tiers already price themselves — the value tier's writes, the
/// scroll tier's asks, the text-write tier's rewrites — stream no
/// action fact; the action trail prices the activation ask only. Since
/// M70 (ADR-0076, ADR-0075's own kill rule) the record names its rider
/// beside the ask — the AT's action route or the keyboard's own
/// activation, the attribution declared where the route is known. The
/// fields are the additive wire rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionRecord {
    /// The activation ask the route routed.
    pub ask: ActionAsk,
    /// The control the ask arrived under — the AccessKit-stable id the
    /// route resolved (the M43 rule's vocabulary at the activation
    /// tier); `None` never rides from the funnel's own note site (the
    /// route notes only the asks it resolved a target for — an ask no
    /// target answered at all is the route-drop trail's fact,
    /// ADR-0063's species, not this one's), and the keyboard's own
    /// activation rides the key route's stamp (M45, ADR-0051 — the
    /// focused control's id, the guard the key visit holds).
    pub target: Option<u64>,
    /// What the tier answered — the landing or the named refusal.
    pub outcome: ActionOutcome,
    /// Who rode the ask (M70, ADR-0076) — the AT's action route or the
    /// keyboard's own activation, the attribution declared where the
    /// route is known (the M30 rider rule at the activation tier).
    pub rider: ActionRider,
}

/// Why one frame answered with no pixels (§13.3, ADR-0077): the frame
/// tier's honest refusals, named where each resolves. Every other tier
/// already prices itself on the log — but the frame itself, the tier
/// every other tier's work rides, answered nothing when it went
/// unpainted: the platform's present ladder logged and moved on, and
/// "why did nothing paint?" was unanswerable from the log alone. The
/// species are the ladder's own stages — the stage that refused is the
/// fact, never a collapsed `false`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameRefusalSpecies {
    /// The tick's layout stage refused ([`crate::passes::LayoutError`],
    /// ADR-0023's error shape): the engine seam's own failure, or the
    /// no-root wiring bug — masks outlived their tree. The frame never
    /// composed; the record carries the failing node's id when the
    /// engine named one.
    LayoutRefused,
    /// The backend's renderer refused a draw the frame asked for — the
    /// full-frame path (or the wgpu surface draw) failed, the window's
    /// pixels stale.
    RenderFailed,
    /// A partial repaint refused (§9.3): the damage regions' scissored
    /// draw failed — the framebuffer survives, the repaired regions
    /// stay stale until a full frame.
    PartialRepaintFailed,
    /// The software path's persistent framebuffer could not be built
    /// (§9.3): no backing store, no frame — the present is skipped
    /// until one allocates.
    FramebufferLost,
    /// The backend refused to surface the frame: the pixels were drawn
    /// and the present returned an error — the frame the user never
    /// saw.
    PresentFailed,
}

impl FrameRefusalSpecies {
    /// The species' name, the wire vocabulary's `snake_case` tag (the
    /// protocol's stable, documented field values).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LayoutRefused => "layout_refused",
            Self::RenderFailed => "render_failed",
            Self::PartialRepaintFailed => "partial_repaint_failed",
            Self::FramebufferLost => "framebuffer_lost",
            Self::PresentFailed => "present_failed",
        }
    }
}

/// One frame refusal's fact (§13.3, ADR-0077): the frame tier's own
/// record, the route-drop trail's shape (ADR-0063) at the frame tier —
/// the species computed where the refusal resolves (the tick's layout
/// block, the backends' present ladder), the failing node's id riding
/// verbatim when a node is the fact (`Arena`'s own vocabulary — the id
/// a consumer resolves against its arena, ADR-0023's rule), the honest
/// absence when the frame had no node to name. The record rides
/// *beside* the refusal's own door — the tick still answers
/// `Err(LayoutError)`, the platform still skips the frame it cannot
/// lay out — never instead of it; payloads never ride (the engines'
/// prose summaries stay on the error types and the trail's trace
/// lines, the payload rule's shape). The fields are the additive wire
/// rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRefusalRecord {
    /// Which stage of the frame refused — the ladder's own name for
    /// what went missing.
    pub species: FrameRefusalSpecies,
    /// The node the refusal is about, verbatim ([`crate::arena::NodeId`]
    /// 's raw id, ADR-0023's vocabulary — the engine named it, the
    /// consumer resolves it against its own arena); `None` when the
    /// frame had no node to name — a present-tier failure is the
    /// window's fact, not a widget's.
    pub node: Option<u64>,
}

/// One view patch's fact (§13.3, ADR-0078): the view tier's own
/// record, the route-drop trail's shape (ADR-0063) at the view tier —
/// the reconciler's four counts computed where the patch resolves
/// (the view root's `mount` build, its `update` diff — the `alacer_view`
/// tier's own doors), named by its lengths
/// (the payload rule's shape: a fact, not a message — the props a
/// patch applied never ride, the text they carry can). The record
/// rides *beside* the door's own `PatchReport` — `update` still
/// answers the report it always answered — never instead of it. One
/// fact per patch that moved something, never per frame: the frames
/// that re-render nothing stream none, and the no-op re-render — a
/// diff that moved nothing — streams none too (the idle-frame
/// property, the m32 canon's own rule; the flood rule at the view
/// tier). The fields are the additive wire rule's shape (ADR-0010).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewPatchRecord {
    /// Elements created (subtree mounts) — the build's opening count.
    pub created: usize,
    /// Subtrees removed.
    pub removed: usize,
    /// Elements patched in place (a visual prop applied).
    pub patched: usize,
    /// Keyed children moved (LIS non-members, §6.2).
    pub moved: usize,
}

/// One theme swap's fact (§7.2 pass 8, ADR-0079): the theme tier's own
/// record — the palette's tag beside the door that landed it, the
/// route-drop trail's shape (ADR-0063) at the theme tier. The tag is
/// the seam's own [`crate::theme::ThemeSeam::name`] (the wire
/// vocabulary's stable, documented field values), never the palette's
/// colors: the payload rule's shape — the log names the palette, the
/// paint pass resolves it. Since M78 (ADR-0084, ADR-0082's second
/// pre-registered re-open rule reaching the theme tier) the record
/// grows the broadcast's node count beside the tag — the lengths rule
/// (the count is the swap's reach, the ids never ride), the count
/// computed where the swap's broadcast resolves, never re-counted at
/// the reader, and the composed swap naming the same reach the flip's
/// broadcast resolved (one flip, one count). The record rides *beside*
/// the door's `bool` answer — the door still answers whether the swap
/// landed — never instead of it; a swap that lands where the palette
/// already is queues no record (the idle-frame property, the m32
/// canon's own rule). One fact per swap, never per frame: the frames
/// that paint the new palette stream no theme fact of their own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeRecord {
    /// The palette's tag — the installed [`crate::theme::ThemeSeam`]
    /// 's own `name()` (`"canon"`, `"midnight"`, `"high_contrast"`), the
    /// log answering "which palette is live now" from the row alone.
    pub palette: String,
    /// The broadcast's node count — how many nodes the swap's
    /// `Change::PAINT` merge touched (ADR-0084): the lengths rule's
    /// shape, the count naming the swap's reach, the ids never riding
    /// (the payload rule — a count is not an identity). The count is
    /// the fact of the swap's own moment: a tree that grows later
    /// never rewrites a queued record, and a composed swap names the
    /// same reach the flip's broadcast resolved — one flip, one count.
    pub repainted: usize,
}

/// One OS setting flip's fact (§7.2 pass 8, ADR-0080): the setting
/// tier's own record — the OS facts beside the door that landed
/// them, the theme trail's shape (ADR-0079) at the setting tier. The
/// bools *are* the facts; since M76 (ADR-0082, ADR-0080's second
/// pre-registered re-open rule) the record grows the broadcast's node
/// count beside them — the lengths rule (the count is the flip's reach,
/// the ids never ride), the count computed where the broadcast
/// resolves, never re-counted at the reader. Since M77 (ADR-0083,
/// ADR-0080's first pre-registered re-open rule) the record grows the
/// reduced-motion fact beside the dark fact, and since M79 (ADR-0085,
/// the same rule cashed a second time) the high-contrast fact beside
/// the two, and since M81 (ADR-0087, the same rule cashed a third
/// time) the accent-color fact beside the three — the fact riding its
/// presence ([`None`], the honest absence, never a guessed color) —
/// and since M154 (ADR-0158, the same rule cashed a fourth time) the
/// reduced-transparency fact beside the four — one record per seam,
/// the facts the vocabulary consults riding beside each other, never a
/// second seam or event per fact.
/// The record rides *beside*
/// the door's `bool` answer — the door still answers whether the flip
/// landed — never instead of it; a flip that lands where the facts
/// already are queues no record (the idle-frame property, A1's rule at
/// the setting tier). One fact per flip, never per frame: the frames
/// that paint under the facts stream no setting fact of their own.
#[derive(Debug, Clone, Copy, PartialEq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the record carries the OS facts beside the reach they repainted — the bools *are* the facts (ADR-0080's rule), one field per fact the flip landed (the M155 cut grew the scale beside the five), the payload rule keeping the ids out"
)]
pub struct SettingRecord {
    /// The OS's dark-mode fact — the installed
    /// [`crate::settings::SettingSeam`]'s own answer, resolved
    /// once per paint pass into `PaintCtx` (the palette the roles
    /// borrow resolved beside it).
    pub dark: bool,
    /// The OS's reduced-motion fact (ADR-0083) — the same seam's
    /// answer, resolved once per paint pass into `PaintCtx` beside the
    /// dark fact: the role-independent decision the vocabulary consults
    /// when motion itself is the question (a spinner's spin, a
    /// thumbnail's autoplay), never a second seam per fact.
    pub reduced_motion: bool,
    /// The OS's high-contrast fact (ADR-0085) — the same seam's answer,
    /// resolved once per paint pass into `PaintCtx` beside the facts it
    /// sits beside: the role-independent decision the vocabulary
    /// consults when contrast itself is the question (a hairline's
    /// weight, a focus ring's width), never a second seam per fact.
    pub high_contrast: bool,
    /// The OS's accent-color fact (ADR-0087) — the same seam's answer,
    /// resolved once per paint pass into `PaintCtx` beside the facts it
    /// sits beside: the role-independent decision the vocabulary
    /// consults when tint itself is the question (an accent that
    /// follows the OS's pick), and the fact the composition resolves
    /// its accent roles by. The fact rides its presence: `None` is the
    /// honest absence — a platform that reports no accent answering
    /// `None`, never a guessed color.
    pub accent: Option<Color>,
    /// The OS's reduced-transparency fact (ADR-0158) — the same seam's
    /// answer, resolved once per paint pass into `PaintCtx` beside the
    /// facts it sits beside: the role-independent decision the
    /// vocabulary consults when translucency itself is the question (a
    /// backdrop that mutes its glass, a selection that holds its fill),
    /// never a second seam per fact. The fact answers the ask: `false`
    /// is effects allowed — the canon answer, never a guessed ask.
    pub reduced_transparency: bool,
    /// The OS's text-scale fact (ADR-0159) — the same seam's answer,
    /// resolved once per paint pass into `PaintCtx` beside the facts it
    /// sits beside: the role-independent decision the vocabulary
    /// consults when typography itself is the question (a label that
    /// follows the OS's preferred reading size), never a second seam
    /// per fact. The fact rides its presence: `None` is the honest
    /// absence — a platform that reports no read answering `None`,
    /// never a guessed multiplier.
    pub text_scale: Option<f32>,
    /// The broadcast's node count — how many nodes the flip's
    /// `Change::PAINT` merge touched (ADR-0082): the lengths rule's
    /// shape, the count naming the flip's reach, the ids never riding
    /// (the payload rule — a count is not an identity). The count is
    /// the fact of the flip's own moment: a tree that grows later
    /// never rewrites a queued record.
    pub repainted: usize,
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
    /// A typing-door edit answered — the key-edit trail (ADR-0064): the
    /// character the keyboard typed, the composition the IME committed,
    /// the unit or word a chord deleted — the change's shape named by
    /// lengths beside the landing selection and the driver who wrote it,
    /// a coalesced typed run landing as one record (the flood rule).
    /// Streams with the frame's own records (the tick's start drain, the
    /// route-drop drain's shape — the fact queues where the edit
    /// resolves, one frame after the key).
    KeyEdit(KeyEditRecord),
    /// A composing state answered — the preedit trail (ADR-0067): the
    /// composition the field shows while the IME owns the canvas, the
    /// inline shape named by lengths beside the platform's cursor, one
    /// live session riding as one record (the flood rule at the
    /// composing tier). The preedit is not an edit — it lands nowhere —
    /// and a commit's landing streams on the key-edit trail beside it
    /// (M58's record). Streams with the frame's own records (the tick's
    /// start drain, the key-edit drain's shape — the fact queues where
    /// the composing state resolves, one frame after the preedit).
    Preedit(PreeditRecord),
    /// The adapter's full-tree echo — the a11y trail (ADR-0073): the
    /// assistive technology connected or reconnected and the sink
    /// answered with the whole resolved tree, the tree named by its
    /// node count (the lengths rule — the payload never rides the
    /// log). Streams ahead of the frame's routes at the tick's start
    /// drain (a connect predates the asks the frame carries, the echo
    /// explains the tree those asks answer into).
    A11yEcho(A11yEchoRecord),
    /// An activation ask answered — the action trail (ADR-0075): the
    /// press the AT's action route aimed at a control, the landing or
    /// the named refusal species beside the aimed id. Streams with the
    /// frame's own records (the tick's start drain, the preedit drain's
    /// shape — the fact queues where the ask resolves, one frame after
    /// the ask).
    Action(ActionRecord),
    /// A frame refused to paint — the frame tier's own trail
    /// (ADR-0077): the species the present ladder's stage named beside
    /// the failing node when one is the fact. Streams with the frame's
    /// own records (the tick's start drain — a refusal queues where it
    /// resolves, one frame after the frame that refused).
    FrameRefusal(FrameRefusalRecord),
    /// A view patch resolved — the view tier's own trail (ADR-0078):
    /// the reconciler's four counts beside each other (created,
    /// removed, patched, moved — the mount's build and every
    /// re-render's diff, named by its lengths, the payload never
    /// riding). Streams with the frame's own records (the tick's
    /// start drain — a patch queues where it resolves, one frame
    /// after the re-render that asked for it).
    ViewPatch(ViewPatchRecord),
    /// An OS setting flip landed — the setting tier's own trail
    /// (ADR-0080): the dark-mode fact the platform adapter pushed
    /// through the door, beside the door. Streams ahead of the theme
    /// drain at the tick's start (the fact the palette resolves
    /// beside, the flip explaining the repaint the swap's facts
    /// answer into).
    Setting(SettingRecord),
    /// A theme swap landed — the theme tier's own trail (ADR-0079):
    /// the palette's tag beside the door, one fact per swap that
    /// landed, the no-op swap streaming none. Streams with the frame's
    /// own records (the tick's start drain — a swap queues where it
    /// lands, one frame after the door answered).
    Theme(ThemeRecord),
    /// A hover's cursor fact resolved — the cursor tier's own trail
    /// (M153, ADR-0157): the style the pointer's hover now resolves to
    /// beside the source that named it (the widget whose hook answered,
    /// the node and role beside it — the lengths rule: the label never
    /// rides), one record per resolved *change*, never per frame (the
    /// flood rule at the cursor tier — a style that did not move is not
    /// a fact). Streams with the frame's own records (the tick's start
    /// drain, the clipboard drain's shape — the fact queues where the
    /// hover resolves, one frame after the change).
    Cursor(CursorRecord),
    /// A frame completed — the frame's causal summary.
    Frame(FrameSummary),
}

/// The observer sink: devtools implement this and the pipeline forwards
/// [`InspectEvent`]s.
pub trait InspectObserver {
    /// Receives one observation.
    fn observe(&mut self, event: InspectEvent);

    /// The a11y pass's resolved tree — the dump seam's door (ADR-0010).
    /// Every a11y pass that builds a snapshot hands the same `Arc` the
    /// sink saw to this door, one call per run, never on a skipped pass
    /// — a devtools recorder overrides it to retain the latest shape
    /// (the [`FrameSummary`] precedent at the `latest` slot), and the
    /// inspector's dump ask answers with what was retained.
    ///
    /// The provided body is the no-op the honest absence is: an
    /// observer that wants no tree prices nothing, the same
    /// opt-in-everything-else the event stream rides. Unlike the events,
    /// a tree rides no ring and no broadcast — it is *state*, the
    /// current shape, never history.
    fn tree(&mut self, snapshot: std::sync::Arc<crate::passes::TreeSnapshot>) {
        let _ = snapshot;
    }
}

/// The observer's tee (§13.3, ADR-0070): two observers behind one. The
/// pipeline's observer slot is single ([`crate::PassPipeline::set_observer`]),
/// and a consumer pricing two doors at once — the inspector's recorder
/// on the wire beside the trail's provided reader in-process
/// ([`crate::trail::TrailLog`]) — needed either a hand-rolled wrapper
/// or one door silently losing the other. The combinator the slot's
/// own contract implies: each event forwards first to `first` then to
/// `second` (the arrival order the causal reads rely on), and a third
/// reader nests — `Tee::new(a, Tee::new(b, c))` — the composition the
/// slot cannot hold on its own.
pub struct Tee {
    first: Box<dyn InspectObserver>,
    second: Box<dyn InspectObserver>,
}

impl std::fmt::Debug for Tee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tee").finish_non_exhaustive()
    }
}

impl Tee {
    /// Two observers behind one slot: the events forwarded first to
    /// `first` then to `second`.
    #[must_use]
    pub fn new(first: Box<dyn InspectObserver>, second: Box<dyn InspectObserver>) -> Self {
        Self { first, second }
    }
}

impl InspectObserver for Tee {
    fn observe(&mut self, event: InspectEvent) {
        self.first.observe(event.clone());
        self.second.observe(event);
    }

    fn tree(&mut self, snapshot: std::sync::Arc<crate::passes::TreeSnapshot>) {
        // The tree door tees too: each reader retains its own `Arc`
        // clone of the one snapshot the pass built — the arrival order
        // the events ride, first then second.
        self.first.tree(std::sync::Arc::clone(&snapshot));
        self.second.tree(snapshot);
    }
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
