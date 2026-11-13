# The Alacer Inspector Protocol

- **Version:** 1 (`alacer_inspect::wire::PROTOCOL_VERSION`)
- **Status:** Implemented (ADR-0010, architecture §13.3)
- **Transport:** WebSocket (RFC 6455) over TCP; server is
  `alacer_inspect::InspectorServer`
- **Encoding:** UTF-8 JSON text frames; one event per frame

The protocol is the Causality inspector's stable surface: the server streams
what the framework recorded; clients render it. The core vocabulary
(`alacer_core::inspect`) never leaks onto the wire — everything here is the
`alacer-inspect` crate's own, versioned contract.

## Endpoints

| Path | What |
|---|---|
| `GET /` | The embedded single-file HTML client (the standalone inspector app). |
| `GET /ws` | The WebSocket stream (below). Any other path serves a pointer back to `/`. |

The demo (`cargo run -p alacer --example inspector_demo`) binds
`127.0.0.1:9217` and prints its URL; `InspectorServer::bind("127.0.0.1:0", …)`
picks a free port.

## Handshake

Standard RFC 6455. The server responds `101 Switching Protocols` with the
`Sec-WebSocket-Accept` derived per spec. Server frames are never masked;
client frames must be (per RFC). The server answers `ping` with `pong`
(echoing the payload) and echoes a client `close` before closing.

## Server → client

### `hello` — the greeting, exactly once, first

```json
{"kind":"hello","version":1,"app":"inspector-demo"}
```

- `version` — the protocol version; clients gate on it.
- `app` — the recording app's name (the `CausalRecorder::new` label).

### `edge` — a reactive edge activation ("why did this render?")

```json
{"kind":"edge","binding":1,"signal":72057594037927936,"node":7,"masks":"layout+paint"}
```

- `binding` — the stable signal→node edge registration that activated.
- `signal` — the signal whose write triggered the rerun (a hashed handle
  identity; stable for the signal's lifetime).
- `node` — the widget node the masks landed on (the arena's node id).
- `masks` — the `Change` bits queued, lowercase bit names joined with `+`,
  in this order: `structure`, `layout`, `paint`, `a11y`, `focus`. Empty
  string means no masks (the edge ran and queued nothing).

### `write` — a provenance-tagged signal write ("who wrote this signal?")

```json
{"kind":"write","signal":72057594037927936,"sequence":4,"origin":"user click"}
```

- `sequence` — monotonic per recorder; ordering is total.
- `origin` — the writer's declared label (`Flux::write` / `FluxWriter::write`).
  Plain `signal.set(..)` writes carry no provenance and produce no event.

### `input` — a routed input event

```json
{"kind":"input","input":"pointer_down","target":9}
```

- `input` — one of `pointer_down`, `pointer_up`, `pointer_move`,
  `pointer_leave`, `wheel`, `key`, `ime`.
- `target` — the widget node the router resolved (deepest hit for pointer
  events, the focused widget for key/IME), or `null` when unrouted.

### `focus` — a focus-move door answered ("why did focus change — or why didn't it?")

```json
{"kind":"focus","movement":"right","rider":"ring","scope":null,"moved":false,"from":null,"to":null,"wrapped":false,"refusal":"nothing_in_direction","node":9,"direction":"right"}
```

- `movement` — the movement the caller asked for: one of `next`,
  `prev`, `left`, `right`, `up`, `down`.
- `rider` — who rode the walk (M30, ADR-0036): `keyboard` (the
  keyboard's Tab/Shift+Tab fallback), `ring` (the directional keys'
  fallback — a D-pad button, a TV remote's ring), or `consumer` (the
  app's own door calls). One funnel, three riders; the field is the
  additive kind the versioning rule admits at version 1.
- `scope` — the subtree root that bounded the walk (the scoped door's
  trap), `null` for the unscoped funnel.
- `moved` — `true` when the walk landed, `false` when it refused.
- `from` / `to` / `wrapped` — the landing's endpoints and the wrap
  (the tree-order move crossed a list end; the entry carries `from`
  `null` and the directional moves never wrap). All three are inert on
  a refusal (`to` `null`, `wrapped` `false`).
- `refusal` — the species on a refused ride: `no_candidates`,
  `dead_scope`, `no_geometry`, `nothing_in_direction`,
  `already_focused`; `null` on a landing.
- `node` — the refusal's node (the dead scope, the geometry-less
  focus, the searched-from node at the edge, the stall's node); `null`
  on a landing.
- `direction` — the empty direction (`nothing_in_direction` only);
  `null` otherwise.

The record streams the moment the door resolves — a refused ride
streams too — so a key event's trail reads `focus` → `input` →
`frame` (the record precedes the input record the dispatch forwards
and the frame summary). Every ride on the funnel streams: the
keyboard's Tab fallback, the app's doors, and the ring's arrows alike
(ADR-0034/ADR-0035), each naming its rider in `rider` (ADR-0036).

### `edit` — a journal action answered ("what did this field's journal do?")

```json
{"kind":"edit","action":"record","coalesced":true,"refused":false,"undo_depth":1,"redo_depth":0}
```

- `action` — the journal action the record carries (M31, ADR-0037):
  `record` (a committed value edit was journaled — the widget's edit
  funnel call), `undo`, `redo`, or `reset` (the whole-value
  replacement's rule — the app's write, never the user's).
- `coalesced` — whether a `record` folded into the journal's tip (the
  pure-append rule, ADR-0021: a typed run is one step). `false` for
  every other action; a coalescing record is visible in the log as a
  depth that never moves.
- `refused` — whether the chord was refused (an exhausted `undo` or an
  empty `redo` branch): the refusal streams too — the M28 honesty, the
  edit tier's own — with `undo_depth`/`redo_depth` naming the journal
  it did not move. Never `true` for `record` or `reset`.
- `undo_depth` / `redo_depth` — the journal's branch depths when the
  fact streamed (the M15 journal's own introspection vocabulary,
  `EditHistory::undo_depth`).

The record streams at the tick's start drain — one frame after the
chord, the announcement drain's currency (the journal queues the fact
on the editing thread, the dispatching pipeline drains it), so an edit
chord's trail reads `edit` → `frame`. Every journal action streams,
applied steps and refused chords alike; the queue behind it is bounded
(the journal's own oldest-first eviction, 256 facts per thread) and
the recorder's `edit` ring holds 512.

### `announce` — a speech act answered ("why did the live region speak — or refuse?")

```json
{"kind":"announce","politeness":"polite","outcome":"delivered","refusal":null}
```

- `politeness` — the `aria-live` priority the act asked for (M33,
  ADR-0039): `polite` (speak at the next break) or `assertive`
  (interrupt now) — the same vocabulary the reserved region carries.
- `outcome` — what the pipeline answered: `delivered` (the act landed
  on the reserved live region — its content and priority ride the
  frame's a11y delta), `superseded` (a later act in the same batch
  replaced this one before the tick drained it — last write wins,
  ADR-0024, the replace rule visible as data instead of a silent
  drop), or `refused` (the act never reached the AT — the M28 honesty
  at the speech-act tier: the fact a trail reader wants is exactly the
  act that never landed).
- `refusal` — the species on a refused act: `empty_text` (an empty or
  all-whitespace text speaks nothing — refused at the push route or at
  the direct door) or `no_root` (no root is mounted — the region has
  no tree to join, refused at the direct door or dropped at the
  drain); `null` otherwise.

The record streams where the answer resolves — the direct door's
answers at the door, the push-side empty refusal riding the thread's
announcement trail to the tick's start drain (the edit trail's shape,
ADR-0037), the drain's batch answers at the tick's start — so a speech
act's trail reads `announce` → `frame`. Every answered act streams,
delivered, replaced, and refused alike; the queue behind it is bounded
(256 facts per thread, oldest-first eviction) and the recorder's
`announce` ring holds 512.

### `key` — a keymap consult answered ("did a shortcut take this key — or why not?")

```json
{"kind":"key","key":"char","character":"s","modifiers":"ctrl","outcome":"fired","name":"save"}
```

- `key` — the logical key the chord carries (M36, ADR-0042): `char`,
  `enter`, `backspace`, `delete`, `tab`, `space`, `escape`,
  `arrow_left`, `arrow_right`, `arrow_up`, `arrow_down`, `home`,
  `end`, `unknown`.
- `character` — the character the chord carries when `key` is `char`
  (the chord's own identity); `null` otherwise.
- `modifiers` — the chord's exact modifier shape, canonical order
  joined with `+` (`shift`, `ctrl`, `alt`, `meta` — the same joined
  rule the `masks` field rides); the empty string when none held. The
  shape is exact: `Ctrl+S` answers `ctrl`, never `ctrl+shift` (a
  different chord, ADR-0041). A primary chord (`Chord::primary`, M38,
  ADR-0044) prints the resolved physical shape the funnel matched —
  Command on Apple platforms, Control elsewhere — never a logical
  label the client would have to interpret.
- `outcome` — what the consult answered: `fired` (a binding ran; the
  funnel answers `Handled` before the framework's fallbacks),
  `fell_through` (a seam answered and named no binding — the key is
  the fallbacks'), or `uninstalled` (no keymap on the window — a
  legitimate application, named instead of guessed).
- `name` — the fired binding's prose name (M37, ADR-0043): what the
  seam's provided `fired_name` answered beside `dispatch` when the
  consult fired — the provided table's binding name or a custom
  seam's own answer; `null` on a miss, an uninstalled window, or a
  seam that names none (the provided method's default — the record
  stays honest about the absence, never guesses). Additive at
  version 1 (the rider field's rule): clients that ignore it are
  unaffected.

The record streams the moment the funnel's consult resolves — before
the fallbacks' focus records, so a key event's trail reads `key` →
`focus` → `input` → `frame`. Only a consult streams: the focused
widget's answer pre-empts the keymap entirely (the M35 precedence),
so a pre-empted key carries its `input` record and no `key` record —
the absent record *is* the precedence. The chord is the fired
binding's identity (one binding per chord, ADR-0041), and since M37
the binding's prose name rides beside it — the funnel capturing the
name before the consult's dispatch, one consult channel still (the
name leg runs no action). The queue behind it is dev-mode and
the recorder's `key` ring holds 512.

### `clipboard` — a clipboard round trip answered ("did the text move — or why not?")

```json
{"kind":"clipboard","action":"paste","outcome":"refused","refusal":"empty_clipboard","engine":null,"read":"non_text","chars":0}
```

- `action` — the round trip the field answered (M39, ADR-0045):
  `copy` (the selection's span written out), `cut` (the span written
  out, then deleted — the deletion only ever followed an accepted
  write), or `paste` (the clipboard's text sanitized through the
  single-line rule and inserted at the caret).
- `outcome` — what the field answered: `delivered` (the payload
  moved) or `refused` (the round trip moved nothing — the M28 honesty
  at the clipboard tier: the fact a trail reader wants is exactly the
  round trip that never landed).
- `refusal` — the species on a refused round trip: `no_selection` (a
  copy or a cut never invents content), `empty_clipboard` (the read
  answered `None` — an empty clipboard, a non-text payload, a
  clipboard this process cannot open — or the payload sanitized to
  nothing), or `write_refused` (the platform refused the write — a cut
  whose write was refused did not delete, the selection surviving a
  clipboard that failed it, ADR-0020's refuse-don't-munge rule);
  `null` on a delivery.
- `engine` — the engine's platform-side species when the round trip's
  refusal was a refused write (M41, ADR-0047, ADR-0045's own re-open
  rule): `unnamed` (the engine answered only its `bool` — the fact
  honest about its absence), `unsupported` (no clipboard subsystem at
  all: a headless box, a sandboxed session), `occupied` (another party
  held the clipboard while the write ran), `conversion` (the payload
  could not be converted to the platform's format), or `os_error` (the
  platform answered with an error the engine cannot classify — its
  description traced engine-side, never guessed onto the wire); `null`
  on a delivery and on every other refusal — a paste reads, a
  no-selection copy never asked the engine. The log answers "which
  clipboard failed it", not only "it failed".
- `read` — the engine's read-side species when the paste refused for
  the read (M42, ADR-0048, ADR-0047's third re-open rule):
  `unnamed` (the engine answered only its `Option` — the fact honest
  about its absence), `unsupported` (no clipboard subsystem at all, or
  the clipboard cannot open), `empty` (the clipboard carries nothing
  the OS counts as content), `non_text` (the clipboard carries
  content, but not text — an image, a custom format), or `os_error`
  (the platform answered with an error the engine cannot classify);
  `null` on a delivery, on the field's own refusal (a payload that
  sanitized to nothing — the read delivered), and on every write-side
  round trip — a copy never asked a read question. The M39
  `empty_clipboard` collapse splits at last: one verdict, the read
  fact beside it answering "did the paste refuse for the read, or for
  the field".
- `chars` — the payload's UTF-8 scalar count when the round trip
  delivered (the span copied, the span cut, the text pasted); zero on
  a refusal. The payload itself never rides the log: a clipboard can
  carry a secret, and the count names the fact, not the content.

The record streams at the tick's start drain — one frame after the
chord, the edit trail's currency (the widget notes the fact where the
round trip resolves, the dispatching pipeline drains it), the drain
running ahead of the edit drain so a cut's trail reads `clipboard` →
`edit` — the causal order the round trip ran: what moved out, then the
deletion it drove. Every round trip streams, delivered and refused
alike; the queue behind it is bounded (256 facts per thread,
oldest-first eviction) and the recorder's `clipboard` ring holds 512.

### `value` — a value-tier write answered ("did the write land — where, or why not?")

```json
{"kind":"value","action":"set_value","outcome":"delivered","refusal":null,"value":42.0,"requested":150.0,"target":43981}
```

- `action` — the value-tier action the AT asked (M40, ADR-0046):
  `set_value` (a payload that parses, then rides the same
  clamp-and-snap funnel every other write rides), `increment`, or
  `decrement` (one step, the funnel's own arithmetic). Since M47
  (ADR-0053, ADR-0051's kill arm) the keyboard's jumps are named facts
  too: `jump_to_min` and `jump_to_max` (the bounds' own species, never
  a munged `SetValue` — the M11 rule's own words), riding the same
  record the AT tier streams, the M45 key route's stamp on the record.
- `outcome` — what the control answered: `delivered` (the value the
  request asked for — a write that lands where the value already was
  is an answered request, not a refusal, ADR-0026's own rule) or
  `refused` (the write moved nothing — the M28 honesty at the value
  tier: the fact a trail reader wants is exactly the write that never
  landed).
- `refusal` — the species on a refused write: `unavailable` (the
  availability tier holds the control down — every door refuses before
  the request is even parsed, ADR-0028), `invalid_payload` (the AT's
  string was not a number — a SetValue never invents a value from
  prose), `non_finite_value` ("inf" parses fine — the funnel carries
  only finite scalars, and the wall is its own fact), or
  `no_write_door` (a reporting widget's read-only tier — every AT
  write refused, never munged into a step or a press, ADR-0027);
  `null` on a delivery.
- `value` — the value the control holds as the event resolves: the
  funnel's clamp-and-snap result on a delivery, the untouched current
  value on a refusal (the tree keeps the value the app wrote). A
  slider's value is the tree's own vocabulary the AT already reads,
  never a secret.
- `requested` — the finite scalar the AT asked, before the control's
  clamp-and-snap moved it (M44, ADR-0050): the ask beside the landing,
  a clamp or a snap the trail could not show visible as data; `null`
  when the ask never became a finite scalar (the unparsable payload,
  the closed door, the non-finite wall — JSON cannot carry an infinite
  ask honestly).
- `target` — the control the action arrived under (M43, ADR-0049,
  ADR-0046's first re-open rule): the AccessKit-stable id the dispatch
  funnel routed — the identity a screen reader's client, an inspector
  pane, or a test correlates, so a multi-control form's trail is
  attributable — the log answering "which control answered" beside
  "did the write land — where, or why not" (two sliders on one page,
  the facts tellable apart).
  The stamp side lives on the funnel: the facts queued beneath a
  routed action carry the id the action arrived under, the widget
  never guessing its own identity. `null` when the write rode no
  routed action — a fact queued outside the funnel's routes carries
  its honest absence. Since M45 (ADR-0051) the keyboard's steps ride
  attributed too, and since M47 (ADR-0053) the jumps ride with them;
  the drag stays off the value log (the flood rule, ADR-0046's second
  kill rule) — the drag's own story rides the `gesture` event instead
  (M46, ADR-0052).

The record streams at the tick's start drain — one frame after the
act, the clipboard trail's currency (the widget notes the fact where
the write resolves, the dispatching pipeline drains it), the drain
running after the edit drain — a value write explains itself; it
deletes nothing. Every value-tier action the AT asks streams, delivered
and refused alike; a question outside the value tier (a press, a
scroll, a text edit) is not a value fact. The queue behind it is
bounded (256 facts per thread, oldest-first eviction) and the
recorder's `value` ring holds 512.


### `gesture` — a drag gesture answered ("where did the drag start, where did it land?")

```json
{"kind":"gesture","phase":"started","from":20.0,"to":null,"target":43981}
{"kind":"gesture","phase":"ended","from":20.0,"to":42.0,"target":43981}
```

The drag's own story (M46, ADR-0052, the price ADR-0046's second kill
rule and ADR-0051's kill arm pre-registered): a consumer needs the
drag on the log — the value tier's trail answers the AT's writes and
the keyboard's steps, but the drag queued nothing, so a trail reader
could not answer "why did the slider's value move while the hand
dragged it" without re-implementing the observation at the gesture
site. The price is the per-gesture record, never per-move: one
event when the press arms the drag, one when the release or the leave
resolves it — one hundred moves inside one gesture stream exactly two
events, the flood rule holding by construction.

- `phase` — the gesture's moment: `started` (the press armed the
  drag) or `ended` (the release or the leave answered it — the leave
  resolving the drag honestly when the capture releases).
- `from` — the value the drag found: the control's own scalar as the
  press landed (the value the drag started *from*). The value tier's
  own vocabulary — the same scalar the AT reads, never a secret.
- `to` — the value the drag left the control holding; `null` on a
  `started` event — the landing is the gesture's answer, not its
  beginning's fact. An `ended` event is self-contained: the beginning
  rides beside the landing, the whole story readable from one line.
- `target` — the control the gesture rides (the funnel's stamp, the
  M43 discipline at the pointer tier): the AccessKit-stable id of the
  node the pointer route resolved — the deepest hit that captured
  (ADR-0025). The stamp side lives on the funnel's pointer route (the
  hit-test route and the captured route hold the guard for the length
  of one routed event's walks); `null` when the fact queued outside a
  routed pointer event (the honest absence). A gesture the
  availability tier's wall never armed (M22, ADR-0028) streams
  nothing at all — the honest absence is the wall's own answer.

The record streams at the tick's start drain — one frame after the
phase, the value trail's currency, the drain running after the value
drain (a gesture explains the value the pointer moved; it streams
beside it). The queue behind it is bounded (256 facts per thread,
oldest-first eviction) and the recorder's `gesture` ring holds 512.

### `frame` — a completed frame

 ("why is this frame slow?", "what did this mask touch?")

```json
{
  "kind":"frame",
  "number":12,
  "layout_ran":true,
  "paint_ran":true,
  "a11y_ran":false,
  "focus_changed":false,
  "damage":"partial",
  "regions":1,
  "a11y_updated":2,
  "a11y_removed":0,
  "total_nanos":184320,
  "passes":[
    {"name":"mutate","ran":false,"nanos":0},
    {"name":"layout","ran":true,"nanos":40120}
  ],
  "dirty":[
    {"node":7,"masks":"layout+paint","role":"label","label":"Clicked 1 times",
     "bounds":[16.0,20.0,120.0,24.0]}
  ],
  "dirty_truncated":0
}
```

- `number` — monotonic per pipeline.
- `damage` — `none` (the present is skipped), `partial` (≤3 regions), or
  `full`; `regions` counts the merged boxes.
- `passes` — the tick's ledger in §7.2 contract order (`mutate`,
  `update_focusable`, `update_focus`, `style`, `layout`, `compose`, `paint`,
  `accessibility`), plus any dispatch-driven `event` passes that preceded
  them. `nanos` is 0 when timings are off.
- `dirty` — the mask heatmap: every dirty node with its role, resolved
  label, and bounds `[x, y, w, h]` (bounds `null` for removed nodes).
  Bounded by the pipeline's heatmap cap (512); `dirty_truncated` counts the
  overflow.
- `a11y_updated` / `a11y_removed` — the a11y delta's sizes (§11.1).

The event trail for a frame (its `write`/`edge`/`input` events) is streamed
immediately *before* that frame's `frame` event.

## Client → server

```json
{"kind":"clear"}
```
Clears the server-side history (rings and latest frame). The client keeps
its own buffer.

```json
{"kind":"bye"}
```
Ends the session; the server echoes `close` and the socket closes. Any
malformed text frame is treated as `bye`.

## Ordering, bounds, and overhead

- Events are delivered in record order; the broadcast queue between the
  observer thread and the session is bounded at 4 096 events and drops the
  oldest (a slow client never back-pressures the app).
- The recorder's rings are bounded (frames 256, edges 1 024, writes 1 024,
  inputs 512, focus 512, edits 512, announces 512, keys 512,
  clipboards 512, values 512, gestures 512); `clear` empties them.
- Recording is opt-in: the pipeline forwards only when an observer is
  installed, and the flux records activations and provenance only when
  `Flux::set_causal_capture(true)`. With both off, the framework's default
  paths pay a branch.

## Versioning

Additive changes bump the minor semantics inside `version`: new event kinds
and new optional fields are compatible; renaming or reshaping existing
fields bumps `version`. Clients must ignore unknown event kinds and unknown
fields.
