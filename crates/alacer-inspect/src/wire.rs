//! The wire format: the documented WS protocol's JSON vocabulary
//! (`docs/inspector-protocol.md`). These DTOs are the inspector's product —
//! the stable, versioned surface devtools clients code against; the core
//! vocabulary (`alacer_core::inspect`) never leaks onto the wire.

use alacer_core::announce::Politeness;
use alacer_core::events::Key;
use alacer_core::history::EditAction;
use alacer_core::inspect::{
    AnnounceOutcome, AnnounceRecord, ClipboardAction, ClipboardOutcome, ClipboardRecord,
    EdgeActivation, EditRecord, FocusRecord, FrameSummary, InputKind, KeyRecord, SignalWrite,
};
use alacer_core::passes::FocusRefusal;
use serde::{Deserialize, Serialize};

/// The protocol version this server speaks. Clients gate on it.
pub const PROTOCOL_VERSION: u32 = 1;

/// The server's greeting, sent on every accepted connection.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Hello {
    /// Always `"hello"`.
    pub kind: &'static str,
    /// The protocol version (1).
    pub version: u32,
    /// The serving app's name (the facade sets it).
    pub app: String,
}

/// The client's control messages (all optional; the server streams
/// regardless).
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Clear the server-side history (the client keeps its own buffer).
    Clear,
    /// The client is going away; the server closes the socket.
    Bye,
}

/// One protocol event. The JSON tag vocabulary is `snake_case`; the shape
/// mirrors the core `InspectEvent` with identities as plain
/// integers.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WireEvent {
    /// A reactive edge activated — "why did this render".
    Edge {
        /// The binding (signal→node edge) that activated.
        binding: u64,
        /// The signal whose write triggered the rerun.
        signal: u64,
        /// The widget node the masks landed on.
        node: u64,
        /// The masks queued, as bit names ("layout+paint").
        masks: String,
    },
    /// A provenance-tagged signal write — "who wrote this signal".
    Write {
        /// The written signal.
        signal: u64,
        /// The write's sequence number.
        sequence: u64,
        /// The declared origin.
        origin: String,
    },
    /// A routed input event.
    Input {
        /// The input kind (`pointer_down`, `key`, …).
        input: String,
        /// The target widget node, when routed.
        target: Option<u64>,
    },
    /// A focus-move door answered — the walk's landing or its named
    /// refusal ("why did focus change — or why didn't it", ADR-0034).
    /// Since M30 the event names its rider too (ADR-0036): the
    /// keyboard's Tab fallback, the ring's arrows, or the app's own
    /// doors — the additive field the protocol's versioning rule admits
    /// at version 1.
    Focus {
        /// The movement the caller asked for (`next`, `prev`, `left`,
        /// `right`, `up`, `down`).
        movement: String,
        /// Who rode the walk (M30, ADR-0036): `keyboard` (the Tab
        /// fallback), `ring` (the directional keys), or `consumer`
        /// (the app's own door calls).
        rider: String,
        /// The subtree root that bounded the walk (the scoped door's
        /// trap), `null` for the unscoped funnel.
        scope: Option<u64>,
        /// Whether the walk landed (`true`) or refused (`false`).
        moved: bool,
        /// The node that held focus before the move, `null` for the
        /// entry.
        from: Option<u64>,
        /// The node the walk rests on now; `null` on a refusal.
        to: Option<u64>,
        /// The tree-order move crossed a list end (directional moves
        /// never wrap).
        wrapped: bool,
        /// The refusal species (`no_candidates`, `dead_scope`,
        /// `no_geometry`, `nothing_in_direction`, `already_focused`),
        /// `null` on a landing.
        refusal: Option<String>,
        /// The refusal's node (the dead scope, the geometry-less focus,
        /// the searched-from node at the edge, the stall's node); `null`
        /// on a landing.
        node: Option<u64>,
        /// The empty direction at the edge (`nothing_in_direction`
        /// only), else `null`.
        direction: Option<String>,
    },
    /// A journal action answered — the edit trail (ADR-0037): the
    /// applied step, the refused chord, or the app's reset, the depths
    /// beside it. Additive at version 1 (the rider field's rule):
    /// clients that ignore unknown kinds are unaffected.
    Edit {
        /// The action the journal answered (`record`, `undo`, `redo`,
        /// `reset`).
        action: String,
        /// Whether a `record` folded into the journal's tip (the
        /// pure-append coalescing rule); `false` for every other action.
        coalesced: bool,
        /// Whether the chord was refused (an exhausted undo or redo
        /// branch — the depths name the journal it did not move).
        refused: bool,
        /// The undo branch's depth when the fact streamed.
        undo_depth: usize,
        /// The redo branch's depth when the fact streamed.
        redo_depth: usize,
    },
    /// A speech act answered — the announcement trail (ADR-0039): the
    /// delivered act, the superseded act, or the named refusal, the
    /// politeness beside it. Additive at version 1 (the rider field's
    /// rule): clients that ignore unknown kinds are unaffected.
    Announce {
        /// The `aria-live` priority the act asked for (`polite`,
        /// `assertive`).
        politeness: String,
        /// What the pipeline answered (`delivered`, `superseded`,
        /// `refused`).
        outcome: String,
        /// The refusal species (`empty_text`, `no_root`) on a refused
        /// act; `null` otherwise.
        refusal: Option<String>,
    },
    /// A keymap consult answered — the shortcut trail (ADR-0042): the
    /// chord the funnel asked the app's shortcuts about beside the
    /// species it got back. Additive at version 1 (the rider field's
    /// rule): clients that ignore unknown kinds are unaffected. Since
    /// M37 (ADR-0043) the event names the fired binding: the prose
    /// name rides beside the outcome when the consult fired and the
    /// seam names one.
    Key {
        /// The logical key the chord carries (`char`, `enter`, `tab`,
        /// `arrow_left`, … — [`key_name`]'s tags).
        key: String,
        /// The character the chord carries when `key` is `char`;
        /// `null` otherwise.
        character: Option<String>,
        /// The chord's exact modifier shape, canonical order
        /// (`shift`, `ctrl`, `alt`, `meta`) joined with `+`; empty
        /// string when none held.
        modifiers: String,
        /// What the consult answered (`fired`, `fell_through`,
        /// `uninstalled`).
        outcome: String,
        /// The fired binding's prose name (M37, ADR-0043) when the
        /// consult fired and the seam names one — the provided table's
        /// binding name or a custom seam's own answer; `null` on a
        /// miss, an uninstalled window, or a seam that names none.
        name: Option<String>,
    },
    /// A clipboard round trip answered — the clipboard trail (ADR-0045):
    /// the copy, cut, or paste the editing chords answered, the
    /// payload's count beside the outcome, the refusals named. Additive
    /// at version 1 (the rider field's rule): clients that ignore
    /// unknown kinds are unaffected. Since M41 (ADR-0047) the event
    /// carries the engine's platform-side fact beside the write
    /// refusal — the additive field the versioning rule admits at
    /// version 1 (clients that ignore unknown fields are unaffected).
    /// Since M42 (ADR-0048) the event carries the engine's read-side
    /// fact beside the paste's refusal — the same additive rule, the
    /// empty clipboard and the non-text payload named apart.
    Clipboard {
        /// The round trip the field answered (`copy`, `cut`, `paste`).
        action: String,
        /// What the field answered (`delivered`, `refused`).
        outcome: String,
        /// The refusal species (`no_selection`, `empty_clipboard`,
        /// `write_refused`) on a refused round trip; `null` otherwise.
        refusal: Option<String>,
        /// The engine's platform-side species (`unnamed`,
        /// `unsupported`, `occupied`, `conversion`, `os_error`) when
        /// the round trip's refusal was a refused write; `null` on a
        /// delivery and on every other refusal — a paste reads, a
        /// no-selection copy never asked the engine (M41, ADR-0047).
        engine: Option<String>,
        /// The engine's read-side species (`unnamed`, `unsupported`,
        /// `empty`, `non_text`, `os_error`) when the paste refused for
        /// the read; `null` on a delivery, on the field's own refusal
        /// (a payload that sanitized to nothing — the read delivered),
        /// and on every write-side round trip — a copy never asked a
        /// read question (M42, ADR-0048).
        read: Option<String>,
        /// The payload's UTF-8 scalar count when the round trip
        /// delivered — the span copied, the span cut, the text pasted.
        /// Zero on a refusal: a refusal moves nothing. The payload
        /// itself never rides the wire (a clipboard can carry a
        /// secret); the length names the fact.
        chars: usize,
    },
    /// A value-tier write answered — the value trail (ADR-0046): the
    /// `SetValue`, `Increment`, or `Decrement` the AT asked, the landed
    /// value beside the outcome, the refusals named. Additive at
    /// version 1 (the rider field's rule): clients that ignore unknown
    /// kinds are unaffected.
    Value {
        /// The value-tier action the AT asked (`set_value`,
        /// `increment`, `decrement`).
        action: String,
        /// What the control answered (`delivered`, `refused`).
        outcome: String,
        /// The refusal species (`unavailable`, `invalid_payload`,
        /// `non_finite_value`, `no_write_door`) on a refused write;
        /// `null` otherwise.
        refusal: Option<String>,
        /// The value the control holds as the event resolves — the
        /// funnel's clamp-and-snap result on a delivery, the untouched
        /// current value on a refusal (the tree keeps the value the app
        /// wrote). A slider's value is the tree's own vocabulary the AT
        /// already reads, never a secret.
        value: f32,
        /// The finite scalar the AT asked, before the control's
        /// clamp-and-snap moved it (M44, ADR-0050) — the ask beside the
        /// landing, a clamp or a snap the trail could not show visible
        /// as data; `null` when the ask never became a finite scalar
        /// (the unparsable payload, the closed door, the non-finite
        /// wall — JSON cannot carry an infinite ask honestly). Additive
        /// at version 1 (the `target` field's rule): clients that
        /// ignore unknown fields are unaffected.
        requested: Option<f32>,
        /// The control the action arrived under (M43, ADR-0049) — the
        /// AccessKit-stable id the dispatch funnel routed, the identity
        /// a screen reader's client or an inspector pane correlates;
        /// `null` when the write rode no routed action (the drag). Since
        /// M45 (ADR-0051) the keyboard's arrows ride attributed too —
        /// the key route stamps the focused control like the action
        /// route does.
        target: Option<u64>,
        /// The bound the funnel pinned the landing at (M49, ADR-0055) —
        /// the range's own edge (`min`, `max`) a past ask was turned
        /// back by (the clamp, the snap, the wall that held a no-op);
        /// `null` when the ask wanted the wall exactly (the jumps — the
        /// species already names it), on every landing mid-range, and
        /// on every refusal (a write that moved nothing pinned
        /// nothing).
        pinned: Option<String>,
    },
    /// A drag gesture answered — the gesture trail (ADR-0052): the
    /// press that armed the value tier's drag beside the value it
    /// found, or the release/leave that resolved it beside the value it
    /// landed, one event per phase, never per move (the flood rule the
    /// kill criteria pre-registered). Additive at version 1 (the rider
    /// field's rule): clients that ignore unknown kinds are unaffected.
    /// Since M48 (ADR-0054, ADR-0053's first kill rule) the event
    /// carries the bound a landing pinned beside the landing — the
    /// additive field the versioning rule admits at version 1 (clients
    /// that ignore unknown fields are unaffected): the bound jump the
    /// *pointer* asked for, named on the drag's own record.
    Gesture {
        /// The gesture's phase (`started` — the press armed the drag;
        /// `ended` — the release or the leave answered it).
        phase: String,
        /// The value the gesture found — the control's own scalar as
        /// the press landed (the value the drag started from).
        from: f32,
        /// The value the gesture left the control holding; `null` on a
        /// `started` record — the landing is the gesture's answer, not
        /// its beginning's fact.
        to: Option<f32>,
        /// The control the gesture rides (M46, ADR-0052) — the
        /// AccessKit-stable id the dispatch funnel's pointer route
        /// resolved (the deepest hit that captured); `null` when the
        /// fact queued outside a routed pointer event.
        target: Option<u64>,
        /// The bound the landing pinned (M48, ADR-0054) — the range's
        /// own edge (`min`, `max`) the drag's landing sat at, the wall
        /// the pointer's ask was turned back by; `null` on a mid-track
        /// landing and on every `started` record (the beginning is not
        /// the wall's fact).
        bound: Option<String>,
    },
    /// A scroll ask answered — the scroll trail (ADR-0056): the wheel,
    /// the AT's unit scroll or value step, the app's own
    /// programmatic scroll, or the content's own clamp (M51,
    /// ADR-0057), the landed offset beside the ask, the refusals
    /// named. Additive at version 1 (the rider field's rule):
    /// clients that ignore unknown kinds are unaffected.
    Scroll {
        /// The scroll-tier action the ask rode (`wheel`, `scroll_up`,
        /// `scroll_down`, `increment`, `decrement`, `scroll_to`,
        /// `content_clamp` — the content's own story, M51/ADR-0057).
        action: String,
        /// What the scroll area answered (`delivered`, `refused`).
        outcome: String,
        /// The refusal species (`no_scrollable_content`,
        /// `non_finite_ask`, `non_finite_content` — the content's
        /// wall, M51/ADR-0057) on a refused ask; `null` otherwise.
        refusal: Option<String>,
        /// The offset the scroll area holds as the event resolves —
        /// the clamp's result on a delivery, the untouched offset on a
        /// refusal. A scroll area's offset is the tree's own
        /// vocabulary the AT already reads, never a secret.
        offset: f32,
        /// The finite offset the ask wanted, before the clamp moved it
        /// (the M44 rule at the scroll tier) — the ask beside the
        /// landing, a clamp the trail could not show visible as data;
        /// `null` when the ask never became a finite scalar
        /// (`non_finite_ask` — JSON cannot carry an infinite ask
        /// honestly) and on the content's own story (a resize is not
        /// an ask, M51/ADR-0057). Additive at version 1 (the
        /// `requested` field's rule): clients that ignore unknown
        /// fields are unaffected.
        requested: Option<f32>,
        /// The control the ask rode (M50, ADR-0056) — the
        /// AccessKit-stable id the wheel route resolved (the deepest
        /// hit under the hover) or the AT action routed; `null` when
        /// the ask rode no route (the app's own handle door, the
        /// content's clamp beside it — M51/ADR-0057).
        target: Option<u64>,
        /// The wall the clamp pinned the landing at (ADR-0055's second
        /// kill rule) — the range's own edge (`min`, `max`) a past ask
        /// was turned back by; `null` when the ask wanted the wall
        /// exactly (the landing *is* the ask), on every landing
        /// mid-range, and on every refusal (a write that moved nothing
        /// pinned nothing).
        pinned: Option<String>,
        /// The asker's unit on the AT's unit scrolls and value steps
        /// (M52, ADR-0058; M54, ADR-0060) — `item` or `page` beside
        /// the `scroll_up`/`scroll_down` actions, the ask the distance
        /// is priced against, and the step's page beside the
        /// `increment`/`decrement` species (the value step's page is a
        /// unit-like distance, attributed the same way — the field's
        /// contract grew beside its shape, no new field, no version
        /// bump); `null` on the stories with no unit (the wheel's
        /// coalesced run — M53 priced it its own magnitude instead —
        /// the app's own door, the content's own clamp). Additive at
        /// version 1 (the `unit` field's rule): clients that ignore
        /// unknown fields are unaffected.
        unit: Option<String>,
        /// The wheel run's notional line count (M53, ADR-0059) — the
        /// coalesced run's magnitude priced by the platform's own
        /// convention (one line = 40 px, §10.2), so a notch answers
        /// 1.0 and the trackpad's pixel runs answer their honest
        /// fraction, beside the `wheel` action only; `null` on every
        /// other action and on the non-finite run (the refusal names
        /// the wall, a count of a run that never was riding no wire).
        /// Additive at version 1 (the `lines` field's rule): clients
        /// that ignore unknown fields are unaffected.
        lines: Option<f32>,
    },
    /// An AT text write answered — the text-write trail (M55,
    /// ADR-0061): the `SetValue`, `ReplaceSelectedText`, or
    /// `SetTextSelection` a screen reader asked at a field's write
    /// door, the write's shape named by lengths beside the landing
    /// selection, the refusals named. Additive at version 1 (the
    /// event-kind rule): clients that ignore unknown kinds are
    /// unaffected.
    TextWrite {
        /// The text-write action the ask rode (`set_value`,
        /// `replace_selection`, `set_selection` — the AT editing
        /// tier's own vocabulary, M11/ADR-0017).
        action: String,
        /// What the field answered (`delivered`, `refused`).
        outcome: String,
        /// The refusal species (`unavailable` — the availability
        /// tier's closed door, M22/ADR-0028, the same fact the value
        /// tier names `unavailable`) on a refused write; `null`
        /// otherwise.
        refusal: Option<String>,
        /// The span the write consumed, in UTF-16 units of the
        /// field's value — the whole value for a `set_value`, the
        /// selection's span for a `replace_selection`; `null` when
        /// the write consumed no span (the selection ask, every
        /// refusal). The write's shape named by its size — the
        /// payload itself never rides the wire (a field's value can
        /// carry a secret, the clipboard trail's rule, ADR-0045).
        replaced: Option<usize>,
        /// The payload's own length, in UTF-16 units — the write's
        /// size beside the span it consumed; `null` when the write
        /// carried no payload (the selection ask, every refusal).
        inserted: Option<usize>,
        /// The selection the AT asked for, before the forgiving read
        /// side floored or clamped it (the M44 rule at the text
        /// tier) — the ask beside the landing, a clamp the trail
        /// could not show visible as data; `null` on the writes that
        /// ask with a payload (their shape rides `replaced`/
        /// `inserted`) and on every refusal.
        requested: Option<WireSelection>,
        /// The selection the field carries as the event resolves —
        /// the caret the write left, or the field's own untouched
        /// selection on a refusal. A field's selection is the tree's
        /// own vocabulary the AT already reads, never a secret.
        selection: WireSelection,
        /// The control the write rode (M55, ADR-0061 — the funnel's
        /// stamp, the M43 rule at the editing tier) — the
        /// AccessKit-stable id the AT action routed; `null` when the
        /// fact rode no route.
        target: Option<u64>,
        /// The door the write rode (M56, ADR-0062 — the M30
        /// attribution rule at the text tier): `at_door` for the AT's
        /// ask, `app_door` for the app's own write (the imperative
        /// door the view reconciliation's patches ride) — the trail
        /// reader's answer to "did the screen reader rewrite this
        /// field, or did the app?".
        source: String,
    },
    /// An ask was dropped before any widget's door — the route-drop
    /// trail (M57, ADR-0063): the unknown action, the payloadless
    /// write, the off-target selection, the unknown target. The
    /// record rides flat (the additive rule ADR-0010 names): the
    /// dropped action's stable tag beside the drop's species and the
    /// aimed id — clients that ignore unknown kinds are unaffected.
    RouteDrop {
        /// The dropped action's stable tag (`set_value`, `focus`, …)
        /// — the platform vocabulary's own name for what arrived.
        action: String,
        /// Why the boundary dropped the ask (`unknown_action`,
        /// `no_payload`, `off_target_selection`, `unknown_target`,
        /// `no_focused_field` — the route's own silences, the IME
        /// event no live field held joining at M59/ADR-0065).
        species: String,
        /// The id the request aimed at, verbatim — the identity the
        /// AT's client correlates against the tree it believes
        /// exists; `null` when the boundary had no aim to name.
        target: Option<u64>,
    },
    /// A typing-door edit answered — the key-edit trail (M58,
    /// ADR-0064): the character the keyboard typed, the composition
    /// the IME committed, the unit or word a chord deleted, and since
    /// M60 (ADR-0066) the clipboard's chords' field changes — the
    /// paste's shape, the cut's span. The record rides flat (the
    /// additive rule ADR-0010 names): the edit's shape named by
    /// lengths — the payload itself never rides the wire (a field's
    /// value can carry a secret, the clipboard trail's rule, ADR-0045)
    /// — beside the landing selection, the driver, and the journal's
    /// fold.
    KeyEdit {
        /// What the typing door did (`insert`, `backspace`, `delete`,
        /// `word_delete_back`, `word_delete_forward`, and since M60 the
        /// chords' own species `paste`/`cut` — the field's own editing
        /// vocabulary, the M11 edit rule's actions).
        action: String,
        /// Who drove the edit (`keyboard`, `ime`, and since M60 the
        /// chords' `clipboard` — the M30 attribution rule at the
        /// typing tier).
        driver: String,
        /// What the door answered (`landed`, or `refused` with the
        /// species beside it — the value tier's own refusal vocabulary,
        /// ADR-0046, at the typing tier; since M62, ADR-0068: the
        /// closed field's wall streams the ask it turned away, the
        /// shape priced at nothing).
        outcome: String,
        /// The refusal's species (`unavailable` — the availability
        /// tier's wall) when the door refused; `null` on a landing.
        /// The refusal is never a munged delivered edit: the ask rode
        /// the record, the field's value untouched.
        refusal: Option<String>,
        /// The span the edit consumed, in UTF-16 units of the field's
        /// value; `0` on a pure insertion.
        replaced: usize,
        /// The payload's own length, in UTF-16 units; `0` on a pure
        /// deletion. The shape named by its size — a coalesced typed
        /// run's record carries the run's total.
        inserted: usize,
        /// The selection the field carries as the edit resolves — the
        /// caret the edit left, anchor collapsed onto it.
        selection: WireSelection,
        /// Whether the journal folded the edit into its tip — the
        /// flood rule's own bit: a typed run of N keystrokes streams
        /// one record whose shape is the run's.
        coalesced: bool,
        /// The control the edit rode (the funnel's stamp, the M43
        /// rule at the typing tier); `null` when the fact rode no
        /// route (the app's direct mutate seam).
        target: Option<u64>,
    },
    /// A composing state answered — the preedit trail (M61, ADR-0067):
    /// the composition the field shows while the IME owns the canvas.
    /// The record rides flat (the additive rule ADR-0010 names — a new
    /// event kind, version 1, clients that ignore unknown kinds
    /// unaffected): the inline shape named by its length — the payload
    /// itself never rides the wire (a field's value can carry a
    /// secret, ADR-0011's rule at the composition tier) — beside the
    /// platform's preedit-local cursor and the funnel's stamp.
    Preedit {
        /// What the composing door answered (`composing`, `cleared` —
        /// the composing state's own phases, never the edit tier's).
        action: String,
        /// The composition's inline shape, in UTF-16 units of the
        /// preedit text; `0` on a cleared state.
        shape: usize,
        /// The platform's cursor inside the preedit, in UTF-16 units
        /// of the preedit text — preedit-local, never a field offset;
        /// `null` when the platform named none.
        cursor: Option<usize>,
        /// The control the composition rode (the funnel's stamp, the
        /// M43 rule at the composing tier); `null` when the fact rode
        /// no route.
        target: Option<u64>,
    },
    /// A completed frame — the ledger + the mask heatmap.
    Frame {
        /// Monotonic frame number.
        number: u64,
        /// Whether the layout pass ran.
        layout_ran: bool,
        /// Whether the paint pass ran.
        paint_ran: bool,
        /// Whether the a11y pass ran.
        a11y_ran: bool,
        /// Whether focus moved.
        focus_changed: bool,
        /// The damage verdict ("none", "partial", "full").
        damage: String,
        /// The damage region count.
        regions: usize,
        /// A11y delta sizes.
        a11y_updated: usize,
        /// A11y delta removals.
        a11y_removed: usize,
        /// Total tick time in nanoseconds (0 when timings are off).
        total_nanos: u64,
        /// The pass ledger: `(pass, ran, nanos)` triples.
        passes: Vec<WirePass>,
        /// The mask heatmap entries (bounded; see `dirty_truncated`).
        dirty: Vec<WireDirtyNode>,
        /// Dirty nodes dropped from the heatmap.
        dirty_truncated: usize,
    },
}

/// One pass ledger entry on the wire.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct WirePass {
    /// The pass name ("layout", "paint", …).
    pub name: &'static str,
    /// Whether it ran.
    pub ran: bool,
    /// Wall time in nanoseconds.
    pub nanos: u64,
}

/// One mask-heatmap entry on the wire.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WireDirtyNode {
    /// The node id.
    pub node: u64,
    /// The masks, as bit names ("layout+paint").
    pub masks: String,
    /// The widget's role.
    pub role: &'static str,
    /// The widget's resolved label.
    pub label: String,
    /// The bounds, `[x, y, w, h]`, when laid out.
    pub bounds: Option<[f32; 4]>,
}

/// One selection on the wire (M55, ADR-0061): the `(anchor, focus)`
/// pair the text-write trail carries — the M11 vocabulary's own
/// shape, UTF-16 units of the field's value, the pair riding as one
/// object so the ask and the landing read as the single fact each is.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct WireSelection {
    /// Where the selection began, as a UTF-16 offset into the value.
    pub anchor: usize,
    /// The active end (the caret while extending), as a UTF-16
    /// offset.
    pub focus: usize,
}

/// The damage verdict's wire name.
#[must_use]
pub fn damage_name(damage: &alacer_core::Damage) -> &'static str {
    if damage.is_full() {
        "full"
    } else if damage.is_empty() {
        "none"
    } else {
        "partial"
    }
}

/// The `Change` mask's wire name: bit names joined with `+`, lowest bit
/// first, stable for clients to parse.
#[must_use]
pub fn masks_name(mask: alacer_core::Change) -> String {
    let mut names: Vec<&'static str> = Vec::new();
    if mask.contains(alacer_core::Change::STRUCTURE) {
        names.push("structure");
    }
    if mask.contains(alacer_core::Change::LAYOUT) {
        names.push("layout");
    }
    if mask.contains(alacer_core::Change::PAINT) {
        names.push("paint");
    }
    if mask.contains(alacer_core::Change::A11Y) {
        names.push("a11y");
    }
    if mask.contains(alacer_core::Change::FOCUS) {
        names.push("focus");
    }
    names.join("+")
}

/// The input kind's wire name.
#[must_use]
pub const fn input_name(kind: InputKind) -> &'static str {
    match kind {
        InputKind::PointerDown => "pointer_down",
        InputKind::PointerUp => "pointer_up",
        InputKind::PointerMove => "pointer_move",
        InputKind::PointerLeave => "pointer_leave",
        InputKind::Wheel => "wheel",
        InputKind::Key => "key",
        InputKind::Ime => "ime",
    }
}

/// The role's wire name (the CI walker's diagnostics vocabulary).
#[must_use]
pub const fn role_name(role: alacer_core::Role) -> &'static str {
    role.name()
}

/// Converts a core [`FrameSummary`] into the wire frame.
#[must_use]
pub fn wire_frame(summary: &FrameSummary) -> WireEvent {
    let regions = summary.damage.regions(summary.viewport).len();
    WireEvent::Frame {
        number: summary.number,
        layout_ran: summary.layout_ran,
        paint_ran: summary.paint_ran,
        a11y_ran: summary.a11y_ran,
        focus_changed: summary.focus_changed,
        damage: damage_name(&summary.damage).to_string(),
        regions,
        a11y_updated: summary.a11y_updated,
        a11y_removed: summary.a11y_removed,
        total_nanos: summary.total_nanos,
        passes: summary
            .passes
            .iter()
            .map(|pass| WirePass {
                name: pass.kind.name(),
                ran: pass.ran,
                nanos: pass.nanos,
            })
            .collect(),
        dirty: summary
            .dirty
            .iter()
            .map(|node| WireDirtyNode {
                node: node.node.raw(),
                masks: masks_name(node.mask),
                role: role_name(node.role),
                label: node.label.clone(),
                bounds: node
                    .bounds
                    .map(|b| [b.origin.x, b.origin.y, b.size.width, b.size.height]),
            })
            .collect(),
        dirty_truncated: summary.dirty_truncated,
    }
}

/// Converts a core [`EdgeActivation`] into the wire event.
#[must_use]
pub fn wire_edge(activation: &EdgeActivation) -> WireEvent {
    WireEvent::Edge {
        binding: activation.binding,
        signal: activation.signal,
        node: activation.node.raw(),
        masks: masks_name(activation.masks),
    }
}

/// Converts a core [`SignalWrite`] into the wire event.
#[must_use]
pub fn wire_write(write: &SignalWrite) -> WireEvent {
    WireEvent::Write {
        signal: write.signal,
        sequence: write.sequence,
        origin: write.origin.to_string(),
    }
}

/// The refusal species' wire name (`snake_case`, the protocol's tag
/// vocabulary).
#[must_use]
pub fn refusal_name(refusal: &FocusRefusal) -> &'static str {
    match refusal {
        FocusRefusal::NoCandidates => "no_candidates",
        FocusRefusal::DeadScope { .. } => "dead_scope",
        FocusRefusal::NoGeometry { .. } => "no_geometry",
        FocusRefusal::NothingInDirection { .. } => "nothing_in_direction",
        FocusRefusal::AlreadyFocused { .. } => "already_focused",
    }
}

/// The rider's wire name
/// ([`alacer_core::inspect::FocusRider::name`], the protocol's
/// `snake_case` tag vocabulary).
#[must_use]
pub const fn rider_name(rider: alacer_core::inspect::FocusRider) -> &'static str {
    rider.name()
}

/// The edit action's wire name
/// ([`alacer_core::history::EditAction::name`], the protocol's
/// `snake_case` tag vocabulary).
#[must_use]
pub const fn edit_action_name(action: EditAction) -> &'static str {
    action.name()
}

/// Converts a core [`EditRecord`] into the wire event: the journal's own
/// fact, flat on the wire (ADR-0037).
#[must_use]
pub fn wire_edit(record: &EditRecord) -> WireEvent {
    WireEvent::Edit {
        action: edit_action_name(record.action).to_string(),
        coalesced: record.coalesced,
        refused: record.refused,
        undo_depth: record.undo_depth,
        redo_depth: record.redo_depth,
    }
}

/// The politeness's wire name
/// ([`alacer_core::announce::Politeness::name`], the protocol's
/// `snake_case` tag vocabulary).
#[must_use]
pub const fn politeness_name(politeness: Politeness) -> &'static str {
    politeness.name()
}

/// Converts a core [`AnnounceRecord`] into the wire event: the speech
/// act's answer, flat on the wire (ADR-0039) — the focus event's flat
/// rule, the refusal species carried beside the outcome.
#[must_use]
pub fn wire_announce(record: &AnnounceRecord) -> WireEvent {
    WireEvent::Announce {
        politeness: politeness_name(record.politeness).to_string(),
        outcome: record.outcome.name().to_string(),
        refusal: match record.outcome {
            AnnounceOutcome::Refused(refusal) => Some(refusal.name().to_string()),
            AnnounceOutcome::Delivered | AnnounceOutcome::Superseded => None,
        },
    }
}

/// The clipboard action's wire name
/// ([`alacer_core::inspect::ClipboardAction::name`], the protocol's
/// `snake_case` tag vocabulary).
#[must_use]
pub const fn clipboard_action_name(action: ClipboardAction) -> &'static str {
    action.name()
}

/// Converts a core [`ClipboardRecord`] into the wire event: the round
/// trip's answer, flat on the wire (ADR-0045) — the announce event's
/// flat rule, the refusal species carried beside the outcome and the
/// payload's count beside both.
#[must_use]
pub fn wire_clipboard(record: &ClipboardRecord) -> WireEvent {
    WireEvent::Clipboard {
        action: clipboard_action_name(record.action).to_string(),
        outcome: record.outcome.name().to_string(),
        refusal: match record.outcome {
            ClipboardOutcome::Refused(refusal) => Some(refusal.name().to_string()),
            ClipboardOutcome::Delivered => None,
        },
        engine: record
            .engine
            .map(alacer_core::clipboard::ClipboardWriteRefusal::name)
            .map(str::to_string),
        read: record
            .read
            .map(alacer_core::clipboard::ClipboardReadRefusal::name)
            .map(str::to_string),
        chars: record.chars,
    }
}

/// The value action's wire name
/// ([`alacer_core::inspect::ValueAction::name`], the protocol's
/// `snake_case` tag vocabulary).
#[must_use]
pub const fn value_action_name(action: alacer_core::inspect::ValueAction) -> &'static str {
    action.name()
}

/// Converts a core [`alacer_core::inspect::ValueRecord`] into the wire
/// event: the value write's answer, flat on the wire (ADR-0046) — the
/// clipboard event's flat rule, the refusal species carried beside the
/// outcome and the landed value beside both, the ask beside the landing
/// (M44, ADR-0050). Since M49 (ADR-0055) the wall the funnel pinned the
/// landing at rides beside them — the ask's own wall, named the way the
/// refusal species are.
#[must_use]
pub fn wire_value(record: &alacer_core::inspect::ValueRecord) -> WireEvent {
    WireEvent::Value {
        action: value_action_name(record.action).to_string(),
        outcome: record.outcome.name().to_string(),
        refusal: match record.outcome {
            alacer_core::inspect::ValueOutcome::Refused(refusal) => {
                Some(refusal.name().to_string())
            }
            alacer_core::inspect::ValueOutcome::Delivered => None,
        },
        value: record.value,
        requested: record.requested,
        target: record.target,
        pinned: record
            .pinned
            .map(alacer_core::inspect::RangeBound::name)
            .map(str::to_string),
    }
}

/// Converts a core [`alacer_core::inspect::GestureRecord`] into the
/// wire event: the drag's answer, flat on the wire (ADR-0052) — the
/// phase beside the value it found and the value it landed, the control
/// the pointer route stamped beside them. One event per gesture phase,
/// never per move: the flood rule the kill criteria pre-registered
/// holds on the wire by construction. Since M48 (ADR-0054) the bound a
/// landing pinned rides beside the landing — the wall named the way the
/// refusal species are.
#[must_use]
pub fn wire_gesture(record: &alacer_core::inspect::GestureRecord) -> WireEvent {
    WireEvent::Gesture {
        phase: record.phase.name().to_string(),
        from: record.from,
        to: record.to,
        target: record.target,
        bound: record
            .bound
            .map(alacer_core::inspect::RangeBound::name)
            .map(str::to_string),
    }
}

/// Converts a core [`alacer_core::inspect::ScrollRecord`] into the
/// wire event: the scroll ask's answer, flat on the wire (ADR-0056) —
/// the value event's flat rule, the refusal species carried beside the
/// outcome and the landed offset beside both, the ask beside the
/// landing, the wall the clamp pinned beside them (ADR-0055's second
/// kill rule at the tier whose funnel knows no range).
#[must_use]
pub fn wire_scroll(record: &alacer_core::inspect::ScrollRecord) -> WireEvent {
    WireEvent::Scroll {
        action: record.action.name().to_string(),
        outcome: record.outcome.name().to_string(),
        refusal: match record.outcome {
            alacer_core::inspect::ScrollOutcome::Refused(refusal) => {
                Some(refusal.name().to_string())
            }
            alacer_core::inspect::ScrollOutcome::Delivered => None,
        },
        offset: record.offset,
        requested: record.requested,
        target: record.target,
        pinned: record
            .pinned
            .map(alacer_core::inspect::RangeBound::name)
            .map(str::to_string),
        unit: record
            .unit
            .map(alacer_core::widget::ScrollUnit::name)
            .map(str::to_string),
        lines: record.lines,
    }
}

/// Converts a core [`alacer_core::inspect::TextWriteRecord`] into the
/// wire event: the text write's answer, flat on the wire (ADR-0061) —
/// the value event's flat rule, the refusal species carried beside the
/// outcome, the write's shape named by lengths and the landing
/// selection beside both, the ask beside the landing on the selection
/// door (the M44 rule at the text tier). The payload itself never
/// rides the wire — a field's value can carry a secret, the clipboard
/// trail's rule (ADR-0045) — the lengths naming the shape the way the
/// clipboard's count names the round trip.
#[must_use]
pub fn wire_text_write(record: &alacer_core::inspect::TextWriteRecord) -> WireEvent {
    WireEvent::TextWrite {
        action: record.action.name().to_string(),
        outcome: record.outcome.name().to_string(),
        refusal: match record.outcome {
            alacer_core::inspect::TextWriteOutcome::Refused(refusal) => {
                Some(refusal.name().to_string())
            }
            alacer_core::inspect::TextWriteOutcome::Delivered => None,
        },
        replaced: record.replaced,
        inserted: record.inserted,
        requested: record.requested.map(|ask| WireSelection {
            anchor: ask.anchor,
            focus: ask.focus,
        }),
        selection: WireSelection {
            anchor: record.selection.anchor,
            focus: record.selection.focus,
        },
        target: record.target,
        source: record.source.name().to_string(),
    }
}

/// The route-drop record, flat on the wire (M57, ADR-0063): the
/// dropped action's stable tag beside the drop's species and the aimed
/// id — the boundary's own answer, never a munged widget-door fact.
#[must_use]
pub fn wire_route_drop(record: &alacer_core::inspect::RouteDropRecord) -> WireEvent {
    WireEvent::RouteDrop {
        action: record.action.to_string(),
        species: record.species.name().to_string(),
        target: record.target,
    }
}

/// The key-edit record, flat on the wire (M58, ADR-0064): the typing
/// door's edit named by lengths beside the landing selection and the
/// driver — the payload itself never rides the wire (a field's value
/// can carry a secret, the clipboard trail's rule, ADR-0045), the
/// shape naming itself the way the clipboard's count names the round
/// trip. Since M62 (ADR-0068) the record carries what the door
/// answered — the landed edit or the wall's refusal (the value tier's
/// own vocabulary, ADR-0046): the species beside the outcome on a
/// refusal, `null` beside a landing, the ask the closed field turned
/// away named by its action.
#[must_use]
pub fn wire_key_edit(record: &alacer_core::inspect::KeyEditRecord) -> WireEvent {
    WireEvent::KeyEdit {
        action: record.action.name().to_string(),
        driver: record.driver.name().to_string(),
        outcome: record.outcome.name().to_string(),
        refusal: match record.outcome {
            alacer_core::inspect::KeyEditOutcome::Refused(refusal) => {
                Some(refusal.name().to_string())
            }
            alacer_core::inspect::KeyEditOutcome::Landed => None,
        },
        replaced: record.replaced,
        inserted: record.inserted,
        selection: WireSelection {
            anchor: record.selection.anchor,
            focus: record.selection.focus,
        },
        coalesced: record.coalesced,
        target: record.target,
    }
}

/// The composing state, named for the wire
/// ([`alacer_core::inspect::PreeditRecord`]'s own
/// event, ADR-0067): the phase's `snake_case` tag beside the shape
/// named by its length — never the payload — the preedit-local cursor
/// and the funnel's stamp beside it.
#[must_use]
pub fn wire_preedit(record: &alacer_core::inspect::PreeditRecord) -> WireEvent {
    WireEvent::Preedit {
        action: record.action.name().to_string(),
        shape: record.shape,
        cursor: record.cursor,
        target: record.target,
    }
}

/// The chord's key, named for the wire ([`KeyRecord`]'s `key` field):
/// the logical key's `snake_case` tag, the protocol's stable,
/// documented field values — the wire-local naming vocabulary's rule
/// ([`input_name`], [`refusal_name`]).
#[must_use]
pub const fn key_name(key: &Key) -> &'static str {
    match key {
        Key::Char(_) => "char",
        Key::Enter => "enter",
        Key::Backspace => "backspace",
        Key::Delete => "delete",
        Key::Tab => "tab",
        Key::Space => "space",
        Key::Escape => "escape",
        Key::ArrowLeft => "arrow_left",
        Key::ArrowRight => "arrow_right",
        Key::ArrowUp => "arrow_up",
        Key::ArrowDown => "arrow_down",
        Key::Home => "home",
        Key::End => "end",
        Key::Unknown => "unknown",
    }
}

/// The chord's modifier shape, canonical order joined with `+` — the
/// masks field's joined-names rule at the key tier; empty when none
/// held (the exact-shape chord matches exactly this string's flags).
#[must_use]
fn modifiers_wire(modifiers: alacer_core::events::Modifiers) -> String {
    let mut parts: Vec<&'static str> = Vec::with_capacity(4);
    if modifiers.shift {
        parts.push("shift");
    }
    if modifiers.control {
        parts.push("ctrl");
    }
    if modifiers.alt {
        parts.push("alt");
    }
    if modifiers.meta {
        parts.push("meta");
    }
    parts.join("+")
}

/// Converts a core [`KeyRecord`] into the wire event: the consult's
/// answer, flat on the wire (ADR-0042) — the chord's key beside its
/// modifier shape and the species the funnel got back, the fired
/// binding's prose name beside the outcome since M37 (ADR-0043).
#[must_use]
pub fn wire_key(record: &KeyRecord) -> WireEvent {
    WireEvent::Key {
        key: key_name(record.chord.key()).to_string(),
        character: match record.chord.key() {
            Key::Char(character) => Some(character.to_string()),
            _ => None,
        },
        modifiers: modifiers_wire(record.chord.modifiers()),
        outcome: record.outcome.name().to_string(),
        name: record.name.clone(),
    }
}

/// Converts a core [`FocusRecord`] into the wire event: the outcome's
/// landing or refusal resolved into the documented flat shape, the rider
/// named beside the movement (M30, ADR-0036).
#[must_use]
pub fn wire_focus(record: &FocusRecord) -> WireEvent {
    let (moved, from, to, wrapped, refusal, node, direction) = match record.outcome {
        alacer_core::passes::FocusMoveOutcome::Moved(landing) => (
            true,
            landing.from.map(alacer_core::NodeId::raw),
            Some(landing.to.raw()),
            landing.wrapped,
            None,
            None,
            None,
        ),
        alacer_core::passes::FocusMoveOutcome::Refused(refusal) => {
            let node = match refusal {
                FocusRefusal::NoCandidates => None,
                FocusRefusal::DeadScope { scope } => Some(scope.raw()),
                FocusRefusal::NoGeometry { node }
                | FocusRefusal::NothingInDirection { node, .. }
                | FocusRefusal::AlreadyFocused { node } => Some(node.raw()),
            };
            let direction = match refusal {
                FocusRefusal::NothingInDirection { direction, .. } => {
                    Some(direction.name().to_string())
                }
                _ => None,
            };
            (
                false,
                None,
                None,
                false,
                Some(refusal_name(&refusal).to_string()),
                node,
                direction,
            )
        }
    };
    WireEvent::Focus {
        movement: record.movement.name().to_string(),
        rider: rider_name(record.rider).to_string(),
        scope: record.scope.map(alacer_core::NodeId::raw),
        moved,
        from,
        to,
        wrapped,
        refusal,
        node,
        direction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_serializes_with_the_kind_tag() {
        let hello = Hello {
            kind: "hello",
            version: PROTOCOL_VERSION,
            app: "demo".to_string(),
        };
        let json = serde_json::to_string(&hello).expect("hello json");
        assert_eq!(
            json, r#"{"kind":"hello","version":1,"app":"demo"}"#,
            "the documented greeting shape"
        );
    }

    #[test]
    fn client_messages_deserialize_by_tag() {
        let clear: ClientMessage = serde_json::from_str(r#"{"kind":"clear"}"#).expect("clear");
        assert_eq!(clear, ClientMessage::Clear);
        let bye: ClientMessage = serde_json::from_str(r#"{"kind":"bye"}"#).expect("bye");
        assert_eq!(bye, ClientMessage::Bye);
    }

    #[test]
    fn edge_events_carry_the_mask_vocabulary() {
        let event = wire_edge(&EdgeActivation {
            binding: 1,
            signal: 2,
            node: alacer_core::arena::NodeId::allocate(),
            masks: alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        });
        let WireEvent::Edge { masks, .. } = event else {
            panic!("edge event");
        };
        assert_eq!(masks, "layout+paint");
    }

    #[test]
    fn edit_events_carry_the_journal_vocabulary() {
        let event = wire_edit(&EditRecord {
            action: EditAction::Record,
            coalesced: true,
            refused: false,
            undo_depth: 1,
            redo_depth: 0,
        });
        let json = serde_json::to_string(&event).expect("edit json");
        assert_eq!(
            json,
            r#"{"kind":"edit","action":"record","coalesced":true,"refused":false,"undo_depth":1,"redo_depth":0}"#,
            "the documented edit shape (the focus event's flat rule)"
        );
        let WireEvent::Edit {
            action,
            coalesced,
            refused,
            undo_depth,
            redo_depth,
        } = event
        else {
            panic!("edit event");
        };
        assert_eq!(action, "record");
        assert!(coalesced);
        assert!(!refused);
        assert_eq!((undo_depth, redo_depth), (1, 0));
    }

    #[test]
    fn a_refused_chord_carries_the_refusal_flag() {
        let event = wire_edit(&EditRecord {
            action: EditAction::Undo,
            coalesced: false,
            refused: true,
            undo_depth: 0,
            redo_depth: 0,
        });
        let WireEvent::Edit {
            action, refused, ..
        } = event
        else {
            panic!("edit event");
        };
        assert_eq!(action, "undo", "the refusal names its action");
        assert!(refused, "the refusal is the fact the wire carries");
    }

    #[test]
    fn announce_events_carry_the_speech_vocabulary() {
        use alacer_core::inspect::AnnounceOutcome;
        let event = wire_announce(&AnnounceRecord {
            politeness: Politeness::Polite,
            outcome: AnnounceOutcome::Delivered,
        });
        let json = serde_json::to_string(&event).expect("announce json");
        assert_eq!(
            json,
            r#"{"kind":"announce","politeness":"polite","outcome":"delivered","refusal":null}"#,
            "the documented announce shape (the edit event's flat rule)"
        );
        let WireEvent::Announce {
            politeness,
            outcome,
            refusal,
        } = event
        else {
            panic!("announce event");
        };
        assert_eq!(politeness, "polite");
        assert_eq!(outcome, "delivered");
        assert_eq!(refusal, None);

        // A superseded act names its outcome, the refusal `null` — the
        // last-write-wins rule visible as data (ADR-0024).
        let event = wire_announce(&AnnounceRecord {
            politeness: Politeness::Assertive,
            outcome: AnnounceOutcome::Superseded,
        });
        let WireEvent::Announce {
            outcome, refusal, ..
        } = event
        else {
            panic!("announce event");
        };
        assert_eq!(outcome, "superseded");
        assert_eq!(refusal, None);
    }

    #[test]
    fn a_refused_act_carries_its_species() {
        use alacer_core::inspect::{AnnounceOutcome, AnnounceRefusal};
        let event = wire_announce(&AnnounceRecord {
            politeness: Politeness::Polite,
            outcome: AnnounceOutcome::Refused(AnnounceRefusal::EmptyText),
        });
        let json = serde_json::to_string(&event).expect("refusal json");
        assert_eq!(
            json,
            r#"{"kind":"announce","politeness":"polite","outcome":"refused","refusal":"empty_text"}"#,
            "the refusal species rides beside the outcome"
        );
        let WireEvent::Announce { refusal, .. } = event else {
            panic!("announce event");
        };
        assert_eq!(refusal.as_deref(), Some("empty_text"));
    }

    #[test]
    fn clipboard_events_carry_the_round_trip_vocabulary() {
        use alacer_core::inspect::ClipboardAction;
        let event = wire_clipboard(&ClipboardRecord {
            action: ClipboardAction::Copy,
            outcome: ClipboardOutcome::Delivered,
            engine: None,
            read: None,
            chars: 11,
        });
        let json = serde_json::to_string(&event).expect("clipboard json");
        assert_eq!(
            json,
            r#"{"kind":"clipboard","action":"copy","outcome":"delivered","refusal":null,"engine":null,"read":null,"chars":11}"#,
            "the documented clipboard shape (the announce event's flat rule)"
        );
        let WireEvent::Clipboard {
            action,
            outcome,
            refusal,
            engine,
            read,
            chars,
        } = event
        else {
            panic!("clipboard event");
        };
        assert_eq!(action, "copy");
        assert_eq!(outcome, "delivered");
        assert_eq!(refusal, None);
        assert_eq!(engine, None, "an accepted write asked no refusal question");
        assert_eq!(read, None, "a delivery asked no read question either");
        assert_eq!(chars, 11, "the record counts the span it moved");
    }

    #[test]
    fn a_refused_round_trip_carries_its_species() {
        use alacer_core::clipboard::ClipboardWriteRefusal;
        use alacer_core::inspect::ClipboardRefusal;
        let event = wire_clipboard(&ClipboardRecord {
            action: ClipboardAction::Cut,
            outcome: ClipboardOutcome::Refused(ClipboardRefusal::WriteRefused),
            engine: Some(ClipboardWriteRefusal::Unnamed),
            read: None,
            chars: 0,
        });
        let json = serde_json::to_string(&event).expect("refusal json");
        assert_eq!(
            json,
            r#"{"kind":"clipboard","action":"cut","outcome":"refused","refusal":"write_refused","engine":"unnamed","read":null,"chars":0}"#,
            "the refusal species rides beside the outcome, the count honest at zero"
        );
        let WireEvent::Clipboard { refusal, .. } = event else {
            panic!("clipboard event");
        };
        assert_eq!(refusal.as_deref(), Some("write_refused"));
    }

    #[test]
    fn the_read_fact_rides_beside_the_paste_refusal() {
        use alacer_core::clipboard::ClipboardReadRefusal;
        use alacer_core::inspect::ClipboardRefusal;
        // The read-refused paste: the read's species rides beside the
        // verdict — the M39 collapse split, one verdict and two facts
        // (M42, ADR-0048).
        let event = wire_clipboard(&ClipboardRecord {
            action: ClipboardAction::Paste,
            outcome: ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard),
            engine: None,
            read: Some(ClipboardReadRefusal::NonText),
            chars: 0,
        });
        let json = serde_json::to_string(&event).expect("read json");
        assert_eq!(
            json,
            r#"{"kind":"clipboard","action":"paste","outcome":"refused","refusal":"empty_clipboard","engine":null,"read":"non_text","chars":0}"#,
            "the read's species rides beside the paste's verdict"
        );
        let WireEvent::Clipboard { read, .. } = event else {
            panic!("clipboard event");
        };
        assert_eq!(read.as_deref(), Some("non_text"));

        // The field-refused paste: the read delivered, the field's own
        // refusal — the read fact rides null, the absence honest.
        let event = wire_clipboard(&ClipboardRecord {
            action: ClipboardAction::Paste,
            outcome: ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard),
            engine: None,
            read: None,
            chars: 0,
        });
        let json = serde_json::to_string(&event).expect("field-refusal json");
        assert_eq!(
            json,
            r#"{"kind":"clipboard","action":"paste","outcome":"refused","refusal":"empty_clipboard","engine":null,"read":null,"chars":0}"#,
            "the field's own refusal carries the read's absence"
        );
        let WireEvent::Clipboard { read, .. } = event else {
            panic!("clipboard event");
        };
        assert_eq!(read, None, "the read delivered — no read refusal rode");
    }

    #[test]
    fn value_events_carry_the_write_vocabulary() {
        use alacer_core::inspect::ValueAction;
        let event = wire_value(&alacer_core::inspect::ValueRecord {
            action: ValueAction::SetValue,
            outcome: alacer_core::inspect::ValueOutcome::Delivered,
            value: 42.0,
            requested: Some(150.0),
            target: Some(0xAB_CD),
            pinned: None,
        });
        let json = serde_json::to_string(&event).expect("value json");
        assert_eq!(
            json,
            r#"{"kind":"value","action":"set_value","outcome":"delivered","refusal":null,"value":42.0,"requested":150.0,"target":43981,"pinned":null}"#,
            "the documented value shape (the clipboard event's flat rule)"
        );
        let WireEvent::Value {
            action,
            outcome,
            refusal,
            value,
            requested,
            target,
            ..
        } = event
        else {
            panic!("value event");
        };
        assert_eq!(action, "set_value");
        assert_eq!(outcome, "delivered");
        assert_eq!(refusal, None);
        assert_eq!(value, 42.0, "the record carries the landed value");
        assert_eq!(
            requested,
            Some(150.0),
            "the record carries the ask beside the landing (M44, ADR-0050)"
        );
        assert_eq!(
            target,
            Some(0xAB_CD),
            "the record names the control the action arrived under"
        );
    }

    #[test]
    fn the_pinned_wall_rides_the_value_event() {
        use alacer_core::inspect::{RangeBound, ValueAction};
        // The wall the funnel pinned the landing at rides beside the
        // species (M49, ADR-0055, ADR-0053's second kill rule): the ask
        // turned back at the range's edge, named on the write's own
        // record — never a second action species.
        let event = wire_value(&alacer_core::inspect::ValueRecord {
            action: ValueAction::SetValue,
            outcome: alacer_core::inspect::ValueOutcome::Delivered,
            value: 100.0,
            requested: Some(150.0),
            target: Some(0xAB_CD),
            pinned: Some(RangeBound::Max),
        });
        let json = serde_json::to_string(&event).expect("pinned json");
        assert_eq!(
            json,
            r#"{"kind":"value","action":"set_value","outcome":"delivered","refusal":null,"value":100.0,"requested":150.0,"target":43981,"pinned":"max"}"#,
            "the ask's own wall rides beside the landing and the ask"
        );
        let WireEvent::Value { pinned, .. } = event else {
            panic!("value event");
        };
        assert_eq!(
            pinned.as_deref(),
            Some("max"),
            "the bound's snake_case tag rides the wire (the protocol's rule)"
        );
    }

    #[test]
    fn a_refused_value_write_carries_its_species() {
        use alacer_core::inspect::{ValueAction, ValueRefusal};
        let event = wire_value(&alacer_core::inspect::ValueRecord {
            action: ValueAction::Increment,
            outcome: alacer_core::inspect::ValueOutcome::Refused(ValueRefusal::NoWriteDoor),
            value: 0.0,
            requested: None,
            target: None,
            pinned: None,
        });
        let json = serde_json::to_string(&event).expect("refusal json");
        assert_eq!(
            json,
            r#"{"kind":"value","action":"increment","outcome":"refused","refusal":"no_write_door","value":0.0,"requested":null,"target":null,"pinned":null}"#,
            "the refusal species rides beside the outcome, the tree's value honest"
        );
        let WireEvent::Value { refusal, value, .. } = event else {
            panic!("value event");
        };
        assert_eq!(refusal.as_deref(), Some("no_write_door"));
        assert_eq!(value, 0.0, "the tree keeps the value the app wrote");
    }

    #[test]
    fn the_engine_fact_rides_beside_the_write_refusal() {
        use alacer_core::clipboard::ClipboardWriteRefusal;
        use alacer_core::inspect::{ClipboardAction, ClipboardRefusal};
        // The engine named its species: the kiosk's clipboard refusing
        // with the fact the OS answered (M41, ADR-0047).
        let event = wire_clipboard(&ClipboardRecord {
            action: ClipboardAction::Copy,
            outcome: ClipboardOutcome::Refused(ClipboardRefusal::WriteRefused),
            engine: Some(ClipboardWriteRefusal::Unsupported),
            read: None,
            chars: 0,
        });
        let json = serde_json::to_string(&event).expect("engine json");
        assert_eq!(
            json,
            r#"{"kind":"clipboard","action":"copy","outcome":"refused","refusal":"write_refused","engine":"unsupported","read":null,"chars":0}"#,
            "the engine's species rides beside the round trip's verdict"
        );
        // A refusal that never asked the engine: the absence stays the
        // absence (the additive field's null, the documented shape).
        let event = wire_clipboard(&ClipboardRecord {
            action: ClipboardAction::Paste,
            outcome: ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard),
            engine: None,
            read: None,
            chars: 0,
        });
        let json = serde_json::to_string(&event).expect("read refusal json");
        assert_eq!(
            json,
            r#"{"kind":"clipboard","action":"paste","outcome":"refused","refusal":"empty_clipboard","engine":null,"read":null,"chars":0}"#,
            "a read asked no write question — the engine fact rides null"
        );
    }

    #[test]
    fn the_gesture_phases_ride_the_flat_event() {
        use alacer_core::inspect::GesturePhase;
        // The beginning: the value the drag found, the landing absent —
        // the documented shape (the value event's flat rule).
        let event = wire_gesture(&alacer_core::inspect::GestureRecord {
            phase: GesturePhase::Started,
            from: 20.0,
            to: None,
            target: None,
            bound: None,
        });
        let json = serde_json::to_string(&event).expect("gesture json");
        assert_eq!(
            json,
            r#"{"kind":"gesture","phase":"started","from":20.0,"to":null,"target":null,"bound":null}"#,
            "the documented gesture shape (the value event's flat rule, the bound's null)"
        );
        let WireEvent::Gesture {
            phase, from, to, ..
        } = event
        else {
            panic!("gesture event");
        };
        assert_eq!(phase, "started");
        assert!((from - 20.0).abs() < f32::EPSILON);
        assert_eq!(to, None, "the landing is not the beginning's fact");

        // The answer: the value it left the control holding, the stamp
        // beside it — one event per phase, never per move. Since M48
        // (ADR-0054) a mid-track landing's bound rides null (the
        // absence honest, the additive field's rule).
        let event = wire_gesture(&alacer_core::inspect::GestureRecord {
            phase: GesturePhase::Ended,
            from: 20.0,
            to: Some(42.0),
            target: Some(43981),
            bound: None,
        });
        let json = serde_json::to_string(&event).expect("gesture end json");
        assert_eq!(
            json,
            r#"{"kind":"gesture","phase":"ended","from":20.0,"to":42.0,"target":43981,"bound":null}"#,
            "the landing and the stamp ride the end's own event"
        );
    }

    #[test]
    fn the_slammed_wall_rides_the_gesture_event() {
        use alacer_core::inspect::{GesturePhase, RangeBound};
        // The wall a landing pinned rides beside it (M48, ADR-0054,
        // ADR-0053's first kill rule): the bound jump the *pointer*
        // asked for, named on the drag's own record — the gesture
        // vocabulary's own growth, never the AT tier's species.
        let event = wire_gesture(&alacer_core::inspect::GestureRecord {
            phase: GesturePhase::Ended,
            from: 20.0,
            to: Some(100.0),
            target: Some(43981),
            bound: Some(RangeBound::Max),
        });
        let json = serde_json::to_string(&event).expect("bound json");
        assert_eq!(
            json,
            r#"{"kind":"gesture","phase":"ended","from":20.0,"to":100.0,"target":43981,"bound":"max"}"#,
            "the pinned wall rides the end's own event as data"
        );
        let WireEvent::Gesture { bound, .. } = event else {
            panic!("gesture event");
        };
        assert_eq!(
            bound.as_deref(),
            Some("max"),
            "the bound's snake_case tag rides the wire (the protocol's rule)"
        );
    }

    /// M50 (ADR-0056, this cut): the scroll event carries the scroll
    /// vocabulary — the action and outcome species, the landed offset
    /// beside the ask, the wall the clamp pinned, the refusals named —
    /// the value event's flat rule at the tier whose funnel knows no
    /// range (ADR-0055's second kill rule).
    #[test]
    fn scroll_events_carry_the_scroll_vocabulary() {
        use alacer_core::inspect::{ScrollAction, ScrollOutcome, ScrollRecord, ScrollRefusal};
        let delivered = ScrollRecord {
            action: ScrollAction::Wheel,
            outcome: ScrollOutcome::Delivered,
            offset: 400.0,
            requested: Some(450.0),
            target: Some(0xAB_CD),
            pinned: Some(alacer_core::inspect::RangeBound::Max),
            unit: None,
            lines: Some(1.0),
        };
        let WireEvent::Scroll {
            action,
            outcome,
            refusal,
            offset,
            requested,
            target,
            pinned,
            unit,
            lines,
        } = wire_scroll(&delivered)
        else {
            panic!("scroll event");
        };
        assert_eq!(
            action, "wheel",
            "the action's snake_case tag rides the wire"
        );
        assert_eq!(outcome, "delivered");
        assert_eq!(refusal, None, "a delivery names no refusal");
        assert_eq!(offset, 400.0, "the landing rides beside the ask");
        assert_eq!(
            requested,
            Some(450.0),
            "the ask rides beside the landing (the M44 rule at the scroll tier)"
        );
        assert_eq!(target, Some(0xAB_CD), "the route's stamp rides");
        assert_eq!(
            pinned.as_deref(),
            Some("max"),
            "the wall's snake_case tag rides the wire (the protocol's rule)"
        );
        assert_eq!(
            unit, None,
            "the wheel is a coalesced run — the unit rides its absence (M52)"
        );
        assert_eq!(
            lines,
            Some(1.0),
            "the run prices its notional lines beside the wheel (M53, ADR-0059)"
        );

        let refused = ScrollRecord {
            action: ScrollAction::ScrollDown,
            outcome: ScrollOutcome::Refused(ScrollRefusal::NoScrollableContent),
            offset: 0.0,
            requested: Some(40.0),
            target: None,
            pinned: None,
            unit: None,
            lines: None,
        };
        let WireEvent::Scroll {
            outcome, refusal, ..
        } = wire_scroll(&refused)
        else {
            panic!("scroll event");
        };
        assert_eq!(outcome, "refused");
        assert_eq!(
            refusal.as_deref(),
            Some("no_scrollable_content"),
            "the refusal's snake_case tag rides the wire"
        );
    }

    /// M52 (ADR-0058) the asker's unit rides beside the unit-scroll
    /// actions — `item`/`page` the ask the distance is priced against
    /// — and M54 (ADR-0060) the value steps' page rides beside the
    /// step species too; the honest absence keeps the stories that
    /// have no unit (the wheel, the app door, the content clamp). The
    /// additive field's rule at version 1: clients that ignore unknown
    /// fields are unaffected.
    #[test]
    fn unit_scroll_events_carry_the_askers_unit() {
        use alacer_core::inspect::{ScrollAction, ScrollOutcome, ScrollRecord};
        use alacer_core::widget::ScrollUnit;
        let item = ScrollRecord {
            action: ScrollAction::ScrollDown,
            outcome: ScrollOutcome::Delivered,
            offset: 40.0,
            requested: Some(40.0),
            target: Some(0xAB_CD),
            pinned: None,
            unit: Some(ScrollUnit::Item),
            lines: None,
        };
        let WireEvent::Scroll { action, unit, .. } = wire_scroll(&item) else {
            panic!("scroll event");
        };
        assert_eq!(action, "scroll_down");
        assert_eq!(
            unit.as_deref(),
            Some("item"),
            "the asker's unit rides the wire (the protocol's snake_case rule)"
        );

        let page = ScrollRecord {
            action: ScrollAction::ScrollUp,
            outcome: ScrollOutcome::Delivered,
            offset: 60.0,
            requested: Some(60.0),
            target: Some(0xAB_CD),
            pinned: None,
            unit: Some(ScrollUnit::Page),
            lines: None,
        };
        let WireEvent::Scroll { unit, .. } = wire_scroll(&page) else {
            panic!("scroll event");
        };
        assert_eq!(
            unit.as_deref(),
            Some("page"),
            "the other unit, same tag rule"
        );

        let step = ScrollRecord {
            action: ScrollAction::Increment,
            outcome: ScrollOutcome::Delivered,
            offset: 100.0,
            requested: None,
            target: Some(0xAB_CD),
            pinned: None,
            unit: Some(ScrollUnit::Page),
            lines: None,
        };
        let WireEvent::Scroll { action, unit, .. } = wire_scroll(&step) else {
            panic!("scroll event");
        };
        assert_eq!(action, "increment");
        assert_eq!(
            unit.as_deref(),
            Some("page"),
            "the step's page rides beside the species (M54, ADR-0060 — the same field, the contract grown)"
        );

        let absent = ScrollRecord {
            action: ScrollAction::ScrollTo,
            outcome: ScrollOutcome::Delivered,
            offset: 100.0,
            requested: Some(100.0),
            target: None,
            pinned: None,
            unit: None,
            lines: None,
        };
        let WireEvent::Scroll { action, unit, .. } = wire_scroll(&absent) else {
            panic!("scroll event");
        };
        assert_eq!(action, "scroll_to");
        assert_eq!(
            unit, None,
            "the app door rides no unit — the honest absence rides null"
        );
    }

    /// M53 (ADR-0059, this cut): the wheel run's notional lines ride
    /// beside the `wheel` action — the coalesced run's magnitude priced
    /// by the platform's line convention (a notch answers 1.0, the
    /// trackpad's pixel runs their honest fraction) — and ride their
    /// honest absence on every story with no wheel run behind it. The
    /// additive field's rule at version 1: clients that ignore unknown
    /// fields are unaffected.
    #[test]
    fn wheel_runs_carry_their_notional_lines() {
        use alacer_core::inspect::{ScrollAction, ScrollOutcome, ScrollRecord};
        let notch = ScrollRecord {
            action: ScrollAction::Wheel,
            outcome: ScrollOutcome::Delivered,
            offset: 40.0,
            requested: Some(40.0),
            target: Some(0xAB_CD),
            pinned: None,
            unit: None,
            lines: Some(1.0),
        };
        let WireEvent::Scroll { action, lines, .. } = wire_scroll(&notch) else {
            panic!("scroll event");
        };
        assert_eq!(action, "wheel");
        assert_eq!(
            lines,
            Some(1.0),
            "one notch prices one line (the platform's own convention)"
        );

        let flick = ScrollRecord {
            action: ScrollAction::Wheel,
            outcome: ScrollOutcome::Delivered,
            offset: 100.0,
            requested: Some(100.0),
            target: Some(0xAB_CD),
            pinned: None,
            unit: None,
            lines: Some(2.5),
        };
        let WireEvent::Scroll { lines, .. } = wire_scroll(&flick) else {
            panic!("scroll event");
        };
        assert_eq!(
            lines,
            Some(2.5),
            "the trackpad's pixel run prices its honest fraction"
        );

        let step = ScrollRecord {
            action: ScrollAction::Decrement,
            outcome: ScrollOutcome::Delivered,
            offset: 60.0,
            requested: None,
            target: Some(0xAB_CD),
            pinned: None,
            unit: None,
            lines: None,
        };
        let WireEvent::Scroll { action, lines, .. } = wire_scroll(&step) else {
            panic!("scroll event");
        };
        assert_eq!(action, "decrement");
        assert_eq!(
            lines, None,
            "the stories with no wheel run ride the absence — never a guessed count"
        );
    }

    /// M51 (ADR-0057, this cut): the content clamp's story rides the
    /// same flat event — the delivered clamp naming the wall it
    /// pinned, the refused non-finite height naming its species, the
    /// ask riding its honest absence on both (a resize is not an ask),
    /// the stamp's honest absence beside them (the app door's rule).
    #[test]
    fn content_clamp_events_carry_the_content_vocabulary() {
        use alacer_core::inspect::{ScrollAction, ScrollOutcome, ScrollRecord, ScrollRefusal};
        let clamped = ScrollRecord {
            action: ScrollAction::ContentClamp,
            outcome: ScrollOutcome::Delivered,
            offset: 200.0,
            requested: None,
            target: None,
            pinned: Some(alacer_core::inspect::RangeBound::Max),
            unit: None,
            lines: None,
        };
        let WireEvent::Scroll {
            action,
            outcome,
            refusal,
            offset,
            requested,
            target,
            pinned,
            unit,
            ..
        } = wire_scroll(&clamped)
        else {
            panic!("scroll event");
        };
        assert_eq!(
            action, "content_clamp",
            "the content's own story rides its own tag (the kill rule's words)"
        );
        assert_eq!(outcome, "delivered");
        assert_eq!(refusal, None, "a delivery names no refusal");
        assert_eq!(offset, 200.0, "the landing the clamp produced");
        assert_eq!(
            requested, None,
            "a resize is not an ask — the honest absence rides the wire"
        );
        assert_eq!(
            target, None,
            "the app door rides no route — the stamp's honest absence"
        );
        assert_eq!(
            pinned.as_deref(),
            Some("max"),
            "the wall the content clamp pinned rides the same vocabulary"
        );
        assert_eq!(
            unit, None,
            "a story with no unit rides the absence (M52, ADR-0058)"
        );

        let refused = ScrollRecord {
            action: ScrollAction::ContentClamp,
            outcome: ScrollOutcome::Refused(ScrollRefusal::NonFiniteContent),
            offset: 100.0,
            requested: None,
            target: None,
            pinned: None,
            unit: None,
            lines: None,
        };
        let WireEvent::Scroll {
            action,
            outcome,
            refusal,
            ..
        } = wire_scroll(&refused)
        else {
            panic!("scroll event");
        };
        assert_eq!(action, "content_clamp");
        assert_eq!(outcome, "refused");
        assert_eq!(
            refusal.as_deref(),
            Some("non_finite_content"),
            "the content's wall rides its own species (the M50 rule at the content door)"
        );
    }

    /// M55 (ADR-0061, this cut): the AT's text write rides the wire as
    /// its own flat event — the whole-value write's shape named by
    /// lengths (the payload itself never riding the wire), the stamp
    /// beside it.
    #[test]
    fn text_write_events_carry_the_write_shape() {
        use alacer_core::inspect::{
            TextWriteAction, TextWriteOutcome, TextWriteRecord, TextWriteSource,
        };
        use alacer_core::widget::TextSelection;
        let rewrite = TextWriteRecord {
            action: TextWriteAction::SetValue,
            outcome: TextWriteOutcome::Delivered,
            replaced: Some(11),
            inserted: Some(21),
            requested: None,
            selection: TextSelection::new(21, 21),
            target: Some(0xAB_CD),
            source: TextWriteSource::AtDoor,
        };
        let WireEvent::TextWrite {
            action,
            outcome,
            refusal,
            replaced,
            inserted,
            requested,
            selection,
            target,
            source,
        } = wire_text_write(&rewrite)
        else {
            panic!("text_write event");
        };
        assert_eq!(
            action, "set_value",
            "the whole-value write rides its own tag"
        );
        assert_eq!(outcome, "delivered");
        assert_eq!(refusal, None, "a delivery names no refusal");
        assert_eq!(
            replaced,
            Some(11),
            "the value the field carried was the consumed span"
        );
        assert_eq!(
            inserted,
            Some(21),
            "the payload's own length, never the payload"
        );
        assert_eq!(
            requested, None,
            "a payload write carries no selection ask — the honest absence"
        );
        assert_eq!(
            (selection.anchor, selection.focus),
            (21, 21),
            "the caret the write left rides beside the shape"
        );
        assert_eq!(target, Some(0xAB_CD), "the stamp names the control");
        assert_eq!(
            source, "at_door",
            "the door the write rode rides beside the shape (M56)"
        );
    }

    /// M55 (ADR-0061, this cut): the selection ask rides the wire with
    /// the ask beside the landing — the forgiving clamp visible as
    /// data, the write's shape riding the honest absence.
    #[test]
    fn text_write_events_carry_the_ask_beside_the_landing() {
        use alacer_core::inspect::{
            TextWriteAction, TextWriteOutcome, TextWriteRecord, TextWriteSource,
        };
        use alacer_core::widget::TextSelection;
        let clamped = TextWriteRecord {
            action: TextWriteAction::SetSelection,
            outcome: TextWriteOutcome::Delivered,
            replaced: None,
            inserted: None,
            requested: Some(TextSelection::new(2, 100)),
            selection: TextSelection::new(2, 11),
            target: Some(0xAB_CD),
            source: TextWriteSource::AtDoor,
        };
        let WireEvent::TextWrite {
            action,
            replaced,
            inserted,
            requested,
            selection,
            ..
        } = wire_text_write(&clamped)
        else {
            panic!("text_write event");
        };
        assert_eq!(action, "set_selection");
        assert_eq!(
            (replaced, inserted),
            (None, None),
            "a selection consumes no span and inserts nothing — the honest absence"
        );
        let ask = requested.expect("the ask rides beside the landing");
        assert_eq!(
            (ask.anchor, ask.focus),
            (2, 100),
            "the ask beside the landing — the M44 rule at the text tier"
        );
        assert_eq!(
            (selection.anchor, selection.focus),
            (2, 11),
            "the forgiving clamp's landing rides beside the ask"
        );
    }

    /// M55 (ADR-0061, this cut): the closed door's refusal rides its
    /// species the way the value tier names it — the same fact,
    /// spoken identically wherever a door names it.
    #[test]
    fn text_write_events_carry_the_closed_doors_species() {
        use alacer_core::inspect::{
            TextWriteAction, TextWriteOutcome, TextWriteRecord, TextWriteRefusal, TextWriteSource,
        };
        use alacer_core::widget::TextSelection;
        let refused = TextWriteRecord {
            action: TextWriteAction::ReplaceSelection,
            outcome: TextWriteOutcome::Refused(TextWriteRefusal::Unavailable),
            replaced: None,
            inserted: None,
            requested: None,
            selection: TextSelection::new(0, 0),
            target: Some(0xAB_CD),
            source: TextWriteSource::AtDoor,
        };
        let WireEvent::TextWrite {
            action,
            outcome,
            refusal,
            ..
        } = wire_text_write(&refused)
        else {
            panic!("text_write event");
        };
        assert_eq!(action, "replace_selection");
        assert_eq!(outcome, "refused");
        assert_eq!(
            refusal.as_deref(),
            Some("unavailable"),
            "the closed door's species rides the way the value tier names it"
        );
    }

    /// M56 (ADR-0062, this cut): the app door's write rides the same
    /// flat event, `source` naming the door — the attribution the
    /// trail reader prices, never reconstructed.
    #[test]
    fn text_write_events_carry_the_app_doors_source() {
        use alacer_core::inspect::{
            TextWriteAction, TextWriteOutcome, TextWriteRecord, TextWriteSource,
        };
        use alacer_core::widget::TextSelection;
        let app_write = TextWriteRecord {
            action: TextWriteAction::SetValue,
            outcome: TextWriteOutcome::Delivered,
            replaced: Some(9),
            inserted: Some(18),
            requested: None,
            selection: TextSelection::new(18, 18),
            target: None,
            source: TextWriteSource::AppDoor,
        };
        let WireEvent::TextWrite {
            source,
            target,
            replaced,
            ..
        } = wire_text_write(&app_write)
        else {
            panic!("text_write event");
        };
        assert_eq!(source, "app_door", "the app door names itself");
        assert_eq!(
            target, None,
            "the app door rode no funnel route — the honest absence"
        );
        assert_eq!(
            replaced,
            Some(9),
            "the write's shape rides beside the source"
        );
    }

    /// M57 (ADR-0063, this cut): the boundary's drop rides its own
    /// flat event — the action's tag beside the species and the aimed
    /// id, never a munged widget-door fact.
    #[test]
    fn route_drop_events_carry_the_boundarys_fact() {
        use alacer_core::inspect::{RouteDropRecord, RouteDropSpecies};
        let aimed_nowhere = RouteDropRecord {
            action: "set_value",
            species: RouteDropSpecies::UnknownTarget,
            target: Some(0xDEAD_BEEF),
        };
        let WireEvent::RouteDrop {
            action,
            species,
            target,
        } = wire_route_drop(&aimed_nowhere)
        else {
            panic!("route_drop event");
        };
        assert_eq!(action, "set_value", "the dropped action's own tag");
        assert_eq!(
            species, "unknown_target",
            "the species names why the boundary refused"
        );
        assert_eq!(
            target,
            Some(0xDEAD_BEEF),
            "the aim the boundary refused, verbatim"
        );

        let payloadless = RouteDropRecord {
            action: "replace_selected_text",
            species: RouteDropSpecies::NoPayload,
            target: Some(0xAB_CD),
        };
        let WireEvent::RouteDrop { species, .. } = wire_route_drop(&payloadless) else {
            panic!("route_drop event");
        };
        assert_eq!(species, "no_payload");
    }

    /// M58 (ADR-0064, this cut): the typing door's edit rides its own
    /// flat event — the shape named by lengths (never the payload), the
    /// landing selection and the driver beside it, the journal's fold
    /// naming the coalesced run. Since M62 (ADR-0068) the event carries
    /// what the door answered — the landed outcome beside its `null`
    /// refusal on every delivered edit, the wall's species beside its
    /// `refused` on the closed field's ask: the value tier's own
    /// vocabulary, additive field values at version 1.
    #[test]
    fn key_edit_events_carry_the_typing_doors_fact() {
        use alacer_core::inspect::{KeyEditAction, KeyEditDriver, KeyEditRecord};
        use alacer_core::widget::TextSelection;
        let typed = KeyEditRecord {
            action: KeyEditAction::Insert,
            driver: KeyEditDriver::Keyboard,
            replaced: 0,
            inserted: 3,
            selection: TextSelection::new(3, 3),
            coalesced: true,
            outcome: alacer_core::inspect::KeyEditOutcome::Landed,
            target: Some(0xAB_CD),
        };
        let WireEvent::KeyEdit {
            action,
            driver,
            outcome,
            refusal,
            replaced,
            inserted,
            selection,
            coalesced,
            target,
        } = wire_key_edit(&typed)
        else {
            panic!("key_edit event");
        };
        assert_eq!(action, "insert", "the field's own editing vocabulary");
        assert_eq!(driver, "keyboard", "the M30 rule at the typing tier");
        assert_eq!(outcome, "landed", "the M58 shape, named additively");
        assert!(refusal.is_none(), "a landing carries no refusal species");
        assert_eq!(replaced, 0, "a typed run consumed no span");
        assert_eq!(inserted, 3, "the run's shape, the fold's total");
        assert_eq!(selection.anchor, 3, "the landing's anchor");
        assert_eq!(selection.focus, 3, "the landing's caret");
        assert!(coalesced, "the flood rule's own bit rides");
        assert_eq!(target, Some(0xAB_CD), "the funnel's stamp beside");

        let chord = KeyEditRecord {
            action: KeyEditAction::WordDeleteForward,
            driver: KeyEditDriver::Keyboard,
            replaced: 6,
            inserted: 0,
            selection: TextSelection::new(2, 2),
            coalesced: false,
            outcome: alacer_core::inspect::KeyEditOutcome::Landed,
            target: None,
        };
        let WireEvent::KeyEdit {
            action, replaced, ..
        } = wire_key_edit(&chord)
        else {
            panic!("key_edit event");
        };
        assert_eq!(
            action, "word_delete_forward",
            "the word chord is never a munged Delete"
        );
        assert_eq!(replaced, 6, "the span the chord consumed");

        // M60 (ADR-0066, this cut): the clipboard's chords ride their
        // own species and driver — the new values are additive field
        // values at version 1, the payload never riding the wire.
        let paste = KeyEditRecord {
            action: KeyEditAction::Paste,
            driver: KeyEditDriver::Clipboard,
            replaced: 7,
            inserted: 4,
            selection: TextSelection::new(4, 4),
            coalesced: false,
            outcome: alacer_core::inspect::KeyEditOutcome::Landed,
            target: Some(0x12_34),
        };
        let WireEvent::KeyEdit {
            action,
            driver,
            replaced,
            inserted,
            ..
        } = wire_key_edit(&paste)
        else {
            panic!("key_edit event");
        };
        assert_eq!(action, "paste", "the chord's own species");
        assert_eq!(driver, "clipboard", "the chord's own driver");
        assert_eq!(replaced, 7, "the selection the paste consumed");
        assert_eq!(inserted, 4, "the payload's length, never the payload");

        let cut = KeyEditRecord {
            action: KeyEditAction::Cut,
            driver: KeyEditDriver::Clipboard,
            replaced: 9,
            inserted: 0,
            selection: TextSelection::new(0, 0),
            coalesced: false,
            outcome: alacer_core::inspect::KeyEditOutcome::Landed,
            target: None,
        };
        let WireEvent::KeyEdit { action, driver, .. } = wire_key_edit(&cut) else {
            panic!("key_edit event");
        };
        assert_eq!(action, "cut", "the cut is never a munged Delete");
        assert_eq!(driver, "clipboard");
    }

    /// M62 (ADR-0068, this cut): the closed field's wall rides its own
    /// species — the ask the availability tier turned away named by its
    /// action and driver, the shape priced at nothing, never a munged
    /// delivered edit.
    #[test]
    fn key_edit_events_carry_the_walls_refusal() {
        use alacer_core::inspect::{KeyEditAction, KeyEditDriver, KeyEditRecord};
        use alacer_core::widget::TextSelection;
        let wall = KeyEditRecord {
            action: KeyEditAction::Paste,
            driver: KeyEditDriver::Clipboard,
            replaced: 0,
            inserted: 0,
            selection: TextSelection::new(2, 2),
            coalesced: false,
            outcome: alacer_core::inspect::KeyEditOutcome::Refused(
                alacer_core::inspect::KeyEditRefusal::Unavailable,
            ),
            target: Some(0xAB_CD),
        };
        let WireEvent::KeyEdit {
            action,
            outcome,
            refusal,
            replaced,
            inserted,
            ..
        } = wire_key_edit(&wall)
        else {
            panic!("key_edit event");
        };
        assert_eq!(action, "paste", "the refused chord keeps its own ask");
        assert_eq!(outcome, "refused", "the wall's own answer");
        assert_eq!(
            refusal.as_deref(),
            Some("unavailable"),
            "the species the value tier names (ADR-0027's own tag)"
        );
        assert_eq!(replaced, 0, "the wall consumed no span");
        assert_eq!(inserted, 0, "the wall carried no payload");
    }

    /// M61 (ADR-0067, this cut): the composing state rides its own flat
    /// event — the shape named by its length (never the payload), the
    /// preedit-local cursor beside it, the funnel's stamp. A new event
    /// kind, additive at version 1 (ADR-0010's rule).
    #[test]
    fn preedit_events_carry_the_composing_states_fact() {
        use alacer_core::inspect::{PreeditAction, PreeditRecord};
        let composing = PreeditRecord {
            action: PreeditAction::Composing,
            shape: 3,
            cursor: Some(2),
            target: Some(0xAB_CD),
        };
        let WireEvent::Preedit {
            action,
            shape,
            cursor,
            target,
        } = wire_preedit(&composing)
        else {
            panic!("preedit event");
        };
        assert_eq!(action, "composing", "the composing state's own phase");
        assert_eq!(shape, 3, "the composition's shape, never its payload");
        assert_eq!(cursor, Some(2), "the platform's preedit-local cursor");
        assert_eq!(target, Some(0xAB_CD), "the funnel's stamp beside");

        let cleared = PreeditRecord {
            action: PreeditAction::Cleared,
            shape: 0,
            cursor: None,
            target: None,
        };
        let WireEvent::Preedit { action, shape, .. } = wire_preedit(&cleared) else {
            panic!("preedit event");
        };
        assert_eq!(action, "cleared", "the end names itself");
        assert_eq!(shape, 0, "the field shows no composing text");
    }

    #[test]
    fn frames_carry_the_damage_verdict_and_regions() {
        use alacer_core::geom::Size;
        use alacer_core::passes::PassKind;
        let summary = FrameSummary {
            number: 7,
            layout_ran: true,
            paint_ran: true,
            a11y_ran: false,
            focus_changed: false,
            damage: alacer_core::Damage::FULL,
            viewport: Size::new(100.0, 80.0),
            a11y_updated: 0,
            a11y_removed: 0,
            total_nanos: 1_000,
            passes: vec![alacer_core::inspect::PassTiming {
                kind: PassKind::Layout,
                ran: true,
                nanos: 400,
            }],
            dirty: Vec::new(),
            dirty_truncated: 0,
        };
        let WireEvent::Frame {
            damage, regions, ..
        } = wire_frame(&summary)
        else {
            panic!("frame event");
        };
        assert_eq!(damage, "full");
        assert_eq!(regions, 1, "full damage is the whole viewport");
    }
}

#[cfg(test)]
mod client_tests {
    /// M30 (ADR-0036, this cut): the embedded standalone client renders
    /// the trail's rider beside the movement — the field the wire
    /// carries is a field the client shows, the attribution a reader of
    /// the log pane answers "who moved focus" with. The client is a
    /// single embedded file; the pin is the file's own fragments, the
    /// same surface route the exit suites ride.
    #[test]
    fn the_client_names_the_rider_beside_the_movement() {
        let page = include_str!("inspector.html");
        for fragment in ["event.rider", "The client names the rider (M30, ADR-0036)"] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the rider: {fragment}"
            );
        }
    }

    /// M31 (ADR-0037, this cut): the embedded standalone client renders
    /// the journal's echo — the edit event the wire now carries lands in
    /// the log pane beside the focus trail, the action and the depths
    /// named, the refusal rendered the way the walk's refusals are. The
    /// pin rides the file's own fragments (the M30 route).
    #[test]
    fn the_client_renders_the_journals_echo() {
        let page = include_str!("inspector.html");
        for fragment in [
            "case \"edit\":",
            "The client renders the journal's echo (M31, ADR-0037)",
        ] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the edit event: {fragment}"
            );
        }
    }

    /// M33 (ADR-0039, this cut): the embedded standalone client renders
    /// the announcements' echo — the announce event the wire now carries
    /// lands in the log pane beside the focus and edit trails, the
    /// politeness and the outcome named, the refusal rendered the way the
    /// walk's refusals are. The pin rides the file's own fragments (the
    /// M31 route).
    #[test]
    fn the_client_renders_the_announcements_echo() {
        let page = include_str!("inspector.html");
        for fragment in [
            "case \"announce\":",
            "The client renders the announcements' echo (M33, ADR-0039)",
        ] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the announce event: {fragment}"
            );
        }
    }

    /// M39 (ADR-0045, this cut): the embedded standalone client renders
    /// the clipboard's echo — the clipboard event the wire now carries
    /// lands in the log pane beside the focus, edit, announce, and key
    /// trails, the action and the outcome named, the refusal rendered
    /// the way the walk's refusals are, the payload's count beside the
    /// delivered answer. The pin rides the file's own fragments (the
    /// M31 route).
    #[test]
    fn the_client_renders_the_clipboards_echo() {
        let page = include_str!("inspector.html");
        for fragment in [
            "case \"clipboard\":",
            "The client renders the clipboard's echo (M39, ADR-0045)",
        ] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the clipboard event: {fragment}"
            );
        }
    }

    /// M37 (ADR-0043, this cut): the embedded standalone client names
    /// the binding — the `name` field the key event now carries lands
    /// in the answer when the consult fired, the log's row labeling
    /// itself so a reader never keeps its own chord → name table. The
    /// pin rides the file's own fragments (the M30 route).
    #[test]
    fn the_client_names_the_binding_beside_the_outcome() {
        let page = include_str!("inspector.html");
        for fragment in ["event.name", "The client names the binding (M37, ADR-0043)"] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the binding's name: {fragment}"
            );
        }
    }

    /// M46 (ADR-0052, this cut): the embedded standalone client renders
    /// the drag's echo — the phase beside the value it found and the
    /// value it landed, the phases never conflating, the flood rule
    /// visible as one row per gesture edge. The pin rides the file's
    /// own fragments (the M30 route).
    #[test]
    fn the_client_renders_the_gestures_echo() {
        let page = include_str!("inspector.html");
        for fragment in [
            "event.phase",
            "The client renders the drag's echo (M46, ADR-0052)",
            // The wall a landing pinned renders beside it (M48, ADR-0054):
            // the bound jump the pointer asked for, readable as data.
            "event.bound",
            "pinned at the ",
        ] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the gesture event: {fragment}"
            );
        }
    }

    /// M50 (ADR-0056, this cut): the embedded standalone client renders
    /// the scroll's echo — the landed offset beside the ask, the wall
    /// the clamp pinned, the refusals rendered the way the walk's
    /// refusals are. The pin rides the file's own fragments (the M46
    /// route).
    #[test]
    fn the_client_renders_the_scrolls_echo() {
        let page = include_str!("inspector.html");
        for fragment in [
            "case \"scroll\":",
            "The client renders the scroll's echo (M50, ADR-0056)",
        ] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the scroll event: {fragment}"
            );
        }
    }

    /// M55 (ADR-0061, this cut): the embedded standalone client renders
    /// the text write's echo — the write's shape named by lengths
    /// beside the landing selection, the refusal rendered the way the
    /// other tiers' refusals are, the payload itself never riding the
    /// wire (the clipboard trail's rule). The pin rides the file's own
    /// fragments (the M50 route).
    #[test]
    fn the_client_renders_the_text_writes_echo() {
        let page = include_str!("inspector.html");
        for fragment in [
            "case \"text_write\":",
            "The client renders the text write's echo (M55, ADR-0061)",
        ] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the text_write event: {fragment}"
            );
        }
    }

    /// M49 (ADR-0055, this cut): the embedded standalone client renders
    /// the funnel's wall — the value event's additive `pinned` field
    /// lands in the log pane beside the clamp span, the ask's own wall
    /// named where a past ask was turned back, the honest absence
    /// rendered exactly as before. The pin rides the file's own
    /// fragments (the M46 route).
    #[test]
    fn the_client_renders_the_walls_echo() {
        let page = include_str!("inspector.html");
        for fragment in [
            "event.pinned",
            "The client renders the value tier's echo (M40, ADR-0046)",
        ] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the pinned wall: {fragment}"
            );
        }
    }
}
