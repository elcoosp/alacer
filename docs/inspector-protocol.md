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
(echoing the payload) and echoes a client `close` before closing. The
server sends no pings of its own, and an unsolicited client `pong` ends
the session — the server treats it as the goodbye it echoes a close for
(the same shape `bye` takes; a client that pongs re-connects cleanly).

## Server → client

### `hello` — the greeting, exactly once, first

```json
{"kind":"hello","version":1,"app":"inspector-demo","facts":41,"gate":null}
```

- `version` — the protocol version; clients gate on it.
- `app` — the recording app's name (the `CausalRecorder::new` label).
- `facts` — the recorder's total fact count at greeting time (since
  M140). Every fact the observer has streamed since the recorder's
  first, evictions included in the count and clears included too: the
  count never moves backward. The cursor source a reconnecting client
  asks its replay with — the number this field named at the client's
  *first* connect is the `after` its later replay ask carries.
  Additive at version 1: clients that ignore unknown fields are
  unaffected.
- `gate` — the live-stream gate this session starts under (since
  M145). The gate a previous session's `subscribe` ask left behind,
  persisted across reconnects: `null` is the whole stream (the shape
  every pre-M142 session rode — a fresh server greets it, and so does
  the session after a bare restore ask), a list gates the session's
  fan-out to the named tiers, an empty list is the mute (the stream
  goes quiet while the rings keep recording). The names are the
  `subscribe` ask's own vocabulary — the stats rows' tier names — in
  the event vocabulary's order, so the greeting's gate and a panel's
  later ask cannot drift apart. A reconnecting panel reads its own ask
  back instead of re-asking blind; re-asking is still welcome, and the
  ask's shape is what the next session greets with. The gate is the
  *session's* fan-out, never the recorder's capture: the rings keep
  recording every tier, and the asks' answers (replay, stats) ride
  whole regardless. Additive at version 1: clients that ignore unknown
  fields are unaffected.

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
{"kind":"edit","action":"record","coalesced":true,"refused":false,"undo_depth":1,"redo_depth":0,"target":43981}
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
- `target` — the field the journal serves (M66, ADR-0072): the
  AccessKit-stable id the routed event arrived under — the M43 stamp
  rule at the journal tier, the key route's and the AT action route's
  guard filling what the journal cannot know. `null` when the journal
  answered outside any route (the app's direct rides — the honest
  absence); two journals tellable apart on one log.

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
{"kind":"value","action":"set_value","outcome":"delivered","refusal":null,"value":100.0,"requested":150.0,"target":43981,"pinned":"max"}
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
- `pinned` — the bound the funnel pinned the landing at (M49,
  ADR-0055, ADR-0053's second kill rule): the range's own edge
  (`min`, `max`) a past ask was turned back by — the clamp, the snap,
  the wall that held a no-op against a past ask — the ask's own wall
  beside the species, never a second action species; `null` when the
  ask wanted the wall exactly (the M47 jumps — the species already
  names the bound it asked for), on every landing mid-range, and on
  every refusal (a write that moved nothing pinned nothing). The
  additive field the versioning rule admits at version 1 — clients
  that ignore unknown fields are unaffected.

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
{"kind":"gesture","phase":"started","from":20.0,"to":null,"target":43981,"bound":null}
{"kind":"gesture","phase":"ended","from":20.0,"to":100.0,"target":43981,"bound":"max"}
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
- `bound` — the bound the landing pinned (M48, ADR-0054, ADR-0053's
  first kill rule): the range's own edge (`min`, `max`) the drag's
  landing sat at — the bound jump the *pointer* asked for, named on
  the drag's own record; `null` on a mid-track landing and on every
  `started` event (the beginning is not the wall's fact: the landing,
  and so the bound, is the gesture's answer). The additive field the
  versioning rule admits at version 1 — clients that ignore unknown
  fields are unaffected.

The record streams at the tick's start drain — one frame after the
phase, the value trail's currency, the drain running after the value
drain (a gesture explains the value the pointer moved; it streams
beside it). The queue behind it is bounded (256 facts per thread,
oldest-first eviction) and the recorder's `gesture` ring holds 512.

### `scroll` — a scroll ask answered ("did the list scroll — where, or why not?")

```json
{"kind":"scroll","action":"wheel","outcome":"delivered","refusal":null,"offset":400.0,"requested":450.0,"target":43981,"pinned":"max","unit":null,"lines":1.0}
{"kind":"scroll","action":"scroll_down","outcome":"refused","refusal":"no_scrollable_content","offset":0.0,"requested":40.0,"target":43981,"pinned":null,"unit":null,"lines":null}
{"kind":"scroll","action":"content_clamp","outcome":"delivered","refusal":null,"offset":200.0,"requested":null,"target":null,"pinned":"max","unit":null,"lines":null}
{"kind":"scroll","action":"scroll_down","outcome":"delivered","refusal":null,"offset":140.0,"requested":140.0,"target":43981,"pinned":null,"unit":"page","lines":null}
```

The scroll tier's own story (M50, ADR-0056, the price ADR-0055's
second kill rule pre-registered): a consumer needs the wall named on a
tier whose funnel knows no range — the list's scroll clamp — so the
tier grows its own record beside its species, never the value record's
(the wall is the asker's story — ADR-0054's rule at the next tier).
The scroll doors — the wheel, the AT's unit scrolls and value steps
(ADR-0016: the scrollbar's value *is* the offset), the app's own
programmatic scroll — answered in silence: a wheel past the content's
end clamped without a word, a wheel over a list whose content fits the
viewport moved nothing unexplained. The record prices every answer:
the landed offset beside the ask, the wall the clamp pinned, the
refusals named.

- `action` — the scroll-tier action the ask rode (M50, ADR-0056):
  `wheel` (a notch the coalescer accumulated upstream, §10.2),
  `scroll_up`/`scroll_down` (the AT's scroll command, traveling by the
  requested unit — one item is one line, one page the viewport,
  ADR-0016), `scroll_left`/`scroll_right` (M91/ADR-0095, additive: the
  AT's horizontal vocabulary beside the vertical pair — the same unit
  contract on the x axis, one page the viewport's *width*),
  `increment`/`decrement` (the honest value-step for a
  scrollbar whose value *is* the offset — one page), or `scroll_to`
  (the app's own programmatic door, no route behind it; since M90 the
  door's x half — `scroll_to_x` — rides the same species, the x story
  on the event's `horizontal` field) — and since M51 the content's own story rides the same vocabulary: `content_clamp` (ADR-0057, the price ADR-0056's third kill rule pre-registered — the clamp `set_content_height` answers when the content shrank under the offset; the clamp is the content's story, not the ask's). Since M52 the unit scrolls name their asker's unit in the event's `unit` field (ADR-0058). And since M88 the geometry-driven ask rides the same vocabulary: `bring_into_view` (ADR-0092 — the pipeline's door prices the minimal offset from the bounds ledger and hands it to the container's own clamp; the event's `aimed` field names the node the ask was for).
- `horizontal` (M89/ADR-0093, additive): the horizontal half of the ask's own story — `{outcome, refusal, offset, requested, pinned}`, the same flat rule per axis; `null` on the stories with no horizontal half (the vertical unit scrolls ride their vertical vocabulary untouched; the M91 horizontal pair's own story rides here, the vertical fields beside it the identity answer).
- `outcome` — what the scroll area answered: `delivered` (the offset
  the clamp produced — a landing the clamp moved, the ask answered, or
  the wall held) or `refused` (the ask moved nothing — the M28
  honesty at the scroll tier).
- `refusal` — the refusal species on a refused ask; `null` otherwise:
  `no_scrollable_content` (the range has no span — the content fits
  the viewport, no scroll exists to deliver), `non_finite_ask` (the
  funnel carries only finite asks — the value record's own rule; the
  door refuses before the arithmetic, the state keeping the offset the
  ask found), or — since M51 — `non_finite_content` (the content's
  own wall, ADR-0057: a non-finite height rode the clamp arithmetic
  and spread `NaN` into the offset and the stored span alike — the
  corruption class the M50 trail exposed at the wheel door, closed
  at the content's own door; the state keeps the height and offset
  it held).
- `offset` — the offset the scroll area holds as the event resolves:
  the clamp's result on a delivery, the untouched offset on a refusal
  (the tree keeps the offset the app set). A scroll area's offset is
  the tree's own vocabulary the AT already reads, never a secret.
- `requested` — the finite offset the ask wanted, before the clamp
  moved it (the M44 rule at the scroll tier) — the ask beside the
  landing, a clamp the trail could not show visible as data; `null`
  when the ask never became a finite scalar (`non_finite_ask` — JSON
  cannot carry an infinite ask honestly). The additive field the
  versioning rule admits at version 1 — clients that ignore unknown
  fields are unaffected.
- `target` — the control the ask rode (the routes' stamp, the M43
  discipline at the scroll tier): the AccessKit-stable id the wheel
  route resolved (the deepest hit under the hover) or the AT action
  routed; `null` when the ask rode no route (the app's own handle
  door — the honest absence; the content clamp rides the same absence,
  M51/ADR-0057).
- `pinned` — the wall the clamp pinned the landing at (ADR-0055's
  second kill rule): the range's own edge (`min`, `max`) a past ask
  was turned back by; `null` when the ask wanted the wall exactly (the
  landing *is* the ask), on every landing mid-range, and on every
  refusal (a write that moved nothing pinned nothing).
- `unit` — the asker's unit on the AT's unit scrolls and value steps
  (M52, ADR-0058, the price ADR-0056's first kill rule
  pre-registered; M54, ADR-0060, the price ADR-0058's first kill
  rule pre-registered): `item` or `page` beside the
  `scroll_up`/`scroll_down` actions, the ask the distance is priced
  against, and the value step's page rides beside the step species
  too — `page` beside `increment`/`decrement` (the step's page is a
  unit-like distance a reader may want attributed the same way; the
  field's contract grew beside its shape, no new field, no version
  bump); `null` on the stories with no unit (the wheel's coalesced
  run — the `lines` field prices it its own magnitude instead — the
  app's own door, the content's own clamp). The additive field the
  versioning rule admits at version 1 — clients that ignore unknown
  fields are unaffected.
- `lines` — the wheel run's notional line count (M53, ADR-0059,
  the price ADR-0058's second kill rule pre-registered): the
  coalesced run's magnitude priced by the platform's own convention
  — one line = 40 px (§10.2), the rule the platform's wheel
  conversion prices `LineDelta` with — so a notch answers `1.0` and
  the trackpad's pixel runs answer their honest fraction, beside the
  `wheel` action only; `null` on every other action and on the
  non-finite run (`non_finite_ask` — a count of a run that never was
  would be the lie the additive rule forbids). The additive field
  the versioning rule admits at version 1 — clients that ignore
  unknown fields are unaffected.
- `aimed` — the node the ask was *for* (M88, ADR-0092): the
  AccessKit-stable id of the node the `bring_into_view` ask aimed at,
  beside the event's `target` (the container whose offset moved) —
  the two ids one ask produced, the aimed one verbatim (the
  route-drop trail's rule); `null` on every ask that aimed no node —
  the wheel, the AT's unit scrolls and value steps, the app's
  raw-offset door, the content's clamp. The additive field the
  versioning rule admits at version 1 — clients that ignore unknown
  fields are unaffected.

The record streams at the tick's start drain — one frame after the
ask, the gesture trail's currency, the drain running after the gesture
drain (a scroll explains the viewport the wheel moved; it streams
beside the gestures). The queue behind it is bounded (256 facts per
thread, oldest-first eviction) and the recorder's `scroll` ring holds
512.

### `text_write` — an AT text write answered

 ("did the screen reader rewrite this field — what shape, where did
 the caret land, or why not?")

```json
{
  "kind":"text_write",
  "action":"set_value",
  "outcome":"delivered",
  "refusal":null,
  "replaced":11,
  "inserted":21,
  "requested":null,
  "selection":{"anchor":21,"focus":21},
  "target":7312465530086738955
}
```

One event per AT text write (M55, ADR-0061, the price ADR-0046's second
kill rule pre-registered): the `SetValue`, `ReplaceSelectedText`, or
`SetTextSelection` a screen reader asked at a field's write door —
the write's shape named by lengths and never the payload (a field's value
can carry a secret — the clipboard trail's rule, ADR-0045), the landing
selection beside the shape, the refusals named. The fields:

- `action` — the text-write action the ask rode: `set_value` (the
  whole value replaced — the journal resetting beneath it,
  ADR-0021's rule), `replace_selection` (the selection's span
  replaced, like a paste the reader made), `set_selection` (the
  selection moved directly).
- `outcome` — `delivered` or `refused`.
- `refusal` — the refusal species on a refused write:
  `unavailable` (the availability tier's closed door, M22/ADR-0028 —
  the same fact the value tier names); `null` otherwise.
- `replaced` — the span the write consumed, in UTF-16 units of the
  field's value (the whole value for a `set_value`, the selection's
  span for a `replace_selection`); `null` when the write consumed no
  span (the selection ask, every refusal).
- `inserted` — the payload's own length, in UTF-16 units — the
  payload itself never riding the wire (a field's value can carry a
  secret, the clipboard trail's rule); `null` when the write carried
  no payload (the selection ask, every refusal).
- `requested` — the selection the AT asked for, before the forgiving
  read side floored or clamped it (the M44 rule at the text tier):
  `{"anchor":…,"focus":…}` — the ask beside the landing, a clamp the
  trail could not show visible as data; `null` on the writes that ask
  with a payload (their shape rides `replaced`/`inserted`) and on
  every refusal.
- `selection` — the selection the field carries as the event
  resolves: the caret the write left, or the field's own untouched
  selection on a refusal. A field's selection is the tree's own
  vocabulary the AT already reads, never a secret.
- `target` — the control the write rode (the funnel's stamp, the M43
  discipline at the editing tier): the AccessKit-stable id the AT
  action routed; `null` when the fact rode no route.
- `source` — the door the write rode (M56, ADR-0062, the M30
  attribution rule at the text tier): `at_door` for the AT's ask at
  the action door, `app_door` for the app's own write — the
  imperative `set_value` door the mutate seam calls and the view
  reconciliation's patches ride (the door the journal's reset echo
  already streamed beside). The trail reader's answer to "did the
  screen reader rewrite this field, or did the app?" — named where
  the record is built, never reconstructed.

The event streams at the tick's start drain — one frame after the
write, the scroll trail's currency, the drain running after the scroll
drain (a text write explains the field the reader rewrote; it streams
beside the scrolls). The queue behind it is bounded (256 facts per
thread, oldest-first eviction) and the recorder's `text_write` ring
holds 512. The additive event the versioning rule admits at version 1
— clients that ignore unknown kinds are unaffected.

### `route_drop` — an ask dropped before any widget's door

The boundary route's own refusal vocabulary (M57, ADR-0063, the price
ADR-0061's third kill rule pre-registered): the asks the boundary
dropped before any widget's door — the action no widget contract
answers, the `SetValue`/`ReplaceSelectedText` whose payload the request
never carried, the `SetTextSelection` aimed at another node, the
dispatch funnel's ask aimed at an id no tree node carries, the IME
event no live field held (a committed composition with nowhere to land,
M59/ADR-0065) — and, since M65 (ADR-0071), the record names the rider
beside the species — the route that dropped the ask, the platform
boundary's mapping arms, the dispatch funnel's refusal, and the IME
route's silence tellable apart on one flat record each, the dead
focused node's drop priced apart from the never-focused one. A drop never rides a widget-door record (no door ever
answered, and a munged one would be the lie the kill rule refuses): the
action is named by its stable `snake_case` tag — the platform
vocabulary's own name for what arrived (`A11yAction::name` answers the
same tags for the actions the two vocabularies share) — the species
naming why the boundary refused beside it, and the aimed id riding
verbatim, never re-resolved against the tree.

```json
{
  "kind": "route_drop",
  "action": "set_value",
  "species": "unknown_target",
  "rider": "funnel",
  "target": 3735928559
}
```

Fields:

- `action` — the dropped action's stable tag (`set_value`, `focus`,
  `show_context_menu`, …): the ask the boundary refused, never the
  core action it did not become.
- `species` — why the boundary dropped the ask: `unknown_action` (no
  consumer in the framework's vocabulary — mapping it onto a press
  would be a correctness bug, ADR-0016's rule), `no_payload` (a
  `SetValue`/`ReplaceSelectedText` whose payload the request did not
  carry), `off_target_selection` (a `SetTextSelection` aimed at
  another node — the single-node selection contract, ADR-0017),
  `unknown_target` (the dispatch funnel's ask aimed at an id no tree
  node carries), `no_focused_field` (M59, ADR-0065: the IME event no
  live field held — *no field has focus*; the `action` rides the
  event's own tag `ime_preedit`/`ime_commit`, and `target` rides its
  honest absence — the boundary had no aim to name), `dead_focus`
  (M65, ADR-0071: the field that held focus has gone — the focused
  node died, and the composition names *why* it had nowhere, never the
  never-focused one the pre-M65 collapse munged it into; the same
  honest absence on `target`).
- `rider` — which route dropped the ask (M65, ADR-0071): `boundary`
  (the platform's mapping arms — the ask never became a core action),
  `funnel` (the dispatch funnel's own refusal — the unknown target),
  or `ime_route` (the IME event no live field held — both halves of
  the split silence). The attribution is declared where the drop
  resolves; a trail reader answers "which route said no" from the row
  alone.
- `target` — the id the request aimed at, verbatim: the identity the
  AT's client correlates against the tree it believes exists; `null`
  when the boundary had no aim to name.

The event streams at the tick's start drain — one frame after the
drop, the text-write trail's currency, the drain running after the
text-write drain (a drop explains the ask the boundary refused; it
streams beside the doors' facts). The queue behind it is bounded (256
facts per thread, oldest-first eviction) and the recorder's
`route_drop` ring holds 512. The additive event the versioning rule
admits at version 1 — clients that ignore unknown kinds are
unaffected.

### `key_edit` — a typing-door edit answered ("what did the typist change in this field?")

The typing door's own record (M58, ADR-0064, the price ADR-0062's
first kill rule pre-registered): the edits the keyboard's keys and the
IME's commits landed at a field — a typed character, the unit the
backspace consumed, the word the chord took, the composition the IME
committed — stream one flat record each. The journal's `edit` event
streams the depths the edit left; this record rides beside it with the
change's *shape*: named by lengths (`replaced`/`inserted`, UTF-16
units) — never the payload (a field's value can carry a secret, the
clipboard trail's rule, ADR-0045) — the landing selection beside it,
the driver who wrote it, and the journal's own fold. The flood rule is
the journal's fold: a coalesced typed run lands as one record whose
shape is the run's (N keystrokes in one pure-append run = one row) —
an edit the journal kept as its own step pushes its own record, and
the drivers' records never merge. Since M60
(ADR-0066) the clipboard chords' field changes stream here too —
`paste`/`cut` behind the `clipboard` driver, the round trip keeping
the clipboard trail's fact (one fact per trail, never a
double-priced edit); since M68 (ADR-0074) the payloadless chord's
read streams here too — `copy` beside the chords, the empty
selection the open door found naming its own refusal, the round
trip keeping its fact (one fact per trail, the delivered chord's
two-fact shape mirrored at the wall); since M62 (ADR-0068) the
record names what the door answered — the landed edit or the wall's
refusal (the routed edit key a disabled field ignored, the IME's
commit a closed field turned away, the ask named by the open door's
own match, the window the availability tier's own — the M22
eviction bounds it, the boundary's post-eviction silence the
route's own); the AT's and app's writes ride `text_write` — the
doors tellable apart on one log.

```json
{
  "kind": "key_edit",
  "action": "insert",
  "driver": "keyboard",
  "outcome": "landed",
  "refusal": null,
  "replaced": 0,
  "inserted": 3,
  "selection": { "anchor": 3, "focus": 3 },
  "coalesced": true,
  "target": 3735928559
}
```

Fields:

- `action` — what the typing door did: `insert` (a typed character, an
  IME's committed composition), `backspace` (the unit or selection
  before the caret), `delete` (the unit or selection after it),
  `word_delete_back` / `word_delete_forward` (the Ctrl/Alt word
  chords, ADR-0019 — never a munge onto the unit deletes they mirror),
  `paste` / `cut` (the clipboard chords' field changes, M60, ADR-0066 —
  never a munge onto the insert and delete they mirror).
- `driver` — who drove the edit: `keyboard` (the keyboard's own keys),
  `ime` (the committed composition), `clipboard` (the paste and cut
  chords, M60, ADR-0066) — the M30 attribution rule at the typing tier.
- `outcome` — what the door answered: `landed` (the edit the shape
  prices, the M58 record) or `refused` (the closed field's wall,
  M62, ADR-0068 — the value trail's own refusal vocabulary, ADR-0046,
  at the typing tier). A refusal prices the *ask*: the action and
  driver the open door would have answered, the shape priced at
  nothing, the selection the untouched one — never a munged delivered
  edit.
- `refusal` — the refusal's species when the door refused:
  `unavailable` (the availability tier's wall, M22, ADR-0028 — the
  tag the AT tier's closed door streams, ADR-0027);
  `empty_selection` (the read's wall, M68, ADR-0074 — the payloadless
  copy or cut, the open door's read finding nothing to move); `null`
  on a landing.
- `replaced` — the span the edit consumed, in UTF-16 units of the
  field's value; `0` on a pure insertion.
- `inserted` — the payload's own length, in UTF-16 units; `0` on a
  pure deletion. A coalesced typed run's record carries the run's
  total.
- `selection` — the selection the field carries as the edit resolves:
  the caret the edit left, anchor collapsed onto it (the M11 edit
  rule).
- `coalesced` — whether the journal folded the edit into its tip: the
  flood rule's own bit, the fold named on the record.
- `target` — the control the edit rode (the funnel's stamp, the M43
  rule at the typing tier — the key route's and, since M58, the IME
  route's own guard); `null` when the fact rode no route (the app's
  direct mutate seam).

The event streams at the tick's start drain — one frame after the
edit, the route-drop drain's currency, the drain running after the
route-drop drain (a key edit explains the field the typist changed; it
streams beside the doors' facts and the boundary's). The queue behind
it is bounded (256 facts per thread, oldest-first eviction) and the
recorder's `key_edit` ring holds 512. The additive event the
versioning rule admits at version 1 — clients that ignore unknown
kinds are unaffected; the M62 `outcome`/`refusal` fields are
additive fields on the same event (the protocol's own rule), never a
new kind.


### `preedit` — a composing state answered ("what is the field showing while the IME composes?")

The composing state's own record (M61, ADR-0067, the price ADR-0064's
first kill rule pre-registered): the composition the field shows while
the IME owns the canvas — the inline shape named by its UTF-16 length
(`shape`) — never the payload (a field's value can carry a secret,
ADR-0011's rule at the composition tier; the secret-entry field
suppresses the preedit echo at the platform, the trail keeps the same
discipline) — the platform's preedit-local cursor beside it
(`cursor`, a position inside the preedit, never a field offset: the
composition lands nowhere), and the funnel's stamp. One live session
rides as one record: N composition frames supersede the session's
record in place (the flood rule at the composing tier — never an echo
per composition frame), a cleared state pushes its own fact, and a
commit's landing rides `key_edit` beside it (one fact per trail —
the trails tellable apart: a cleared state with no commit beside it
reads as the cancelled session). A new event kind, additive by the
protocol's own rule — clients that ignore unknown kinds unaffected
(ADR-0010). The recorder's `preedit` ring holds 512.

```json
{
  "kind": "preedit",
  "action": "composing",
  "shape": 3,
  "cursor": 2,
  "target": 3735928559
}
```

Fields:

- `action` — what the composing door answered: `composing` (the live
  session's inline shape), `cleared` (the composing state ended with no
  landing — the honest `0`; the commit would have ridden `key_edit`).
- `shape` — the composition's inline shape, in UTF-16 units of the
  preedit text; `0` on a cleared state.
- `cursor` — the platform's cursor inside the preedit, in UTF-16 units
  of the preedit text — preedit-local, never a field offset; `null`
  when the platform named none.
- `target` — the control the composition rode (the funnel's stamp, the
  M43 rule at the composing tier); `null` when the fact rode no route.

### `a11y_echo` — the adapter's full-tree echo

 ("why did the app just push the whole tree, and how big was it?")

```json
{
  "kind":"a11y_echo",
  "nodes":42
}
```

The a11y tier's own fact (M67, ADR-0073, the price M59's kill rule
pre-registered): an assistive technology connected or reconnected —
`InitialTreeRequested` — and the sink answered with the whole resolved
tree, the same node builder the deltas use. The tree is named by its
node count; the payload itself never rides the wire (a resolved tree
carries the app's labels, and a label can carry a secret, the clipboard
trail's rule at the a11y tier). `nodes` is never `0`: an activation
before the first frame resolves no tree, and no fact streams for it
(the honest absence — the trail never invents a size).

The event streams at the tick's start drain, *ahead of the frame's
routes* — a connect predates the asks the frame carries, and the echo
explains the tree those asks answer into (the echo reads first, the
routes beside it). The queue behind it is bounded (256 facts per
thread, oldest-first eviction) and the recorder's `a11y_echo` ring
holds 512. A connect is rare — one fact per build, the log quiet
between connects (the flood rule at a tier whose events are rare). The
additive event the versioning rule admits at version 1 — clients that
ignore unknown kinds are unaffected.

### `action` — an activation ask answered

 ("did the AT's press land — or why not, and who pressed?")

```json
{
  "kind":"action",
  "ask":"press",
  "target":42,
  "outcome":"refused",
  "refusal":"unavailable",
  "rider":"action_route"
}
```

The action tier's own fact (M69, ADR-0075, the tier's own kill rule): the press — the default accessibility action the platform's action route aims at a control — answered the dispatch funnel's bare `bool`, whose `false` collapsed the two facts a trail reader wants apart. The record names the ask's answer: `delivered`, or `refused` with the species beside it — `unavailable` when the availability tier's wall held the control down (the tree's one `enabled` truth, computed at the route, never guessed at a widget-private call site), `no_door` when no door answered the ask. The aimed id rides verbatim; a landing carries `refusal: null`. Since M70 (ADR-0076) the record names its rider: `action_route` for the AT's ask (the funnel's own note site) or `keyboard` for the focused control's Space/Enter activation (the M10 parity — the button's GTK release rule, the checkbox's toggle-on-press), the additive field the versioning rule admits at version 1. One fact per trail: the asks whose tiers already price themselves — the value tier's writes, the scroll tier's asks, the text-write tier's rewrites — stream no action fact. The event streams at the tick's start drain (the preedit drain's shape), one frame after the ask resolved; the queue behind it is bounded (256 facts per thread, oldest-first eviction) and the recorder's `action` ring holds 512. The additive event the versioning rule admits at version 1 — clients that ignore unknown kinds are unaffected.

### `frame_refusal` — a frame refused to paint

 ("why did nothing paint?")

```json
{
  "kind":"frame_refusal",
  "species":"layout_refused",
  "node":9
}
```

The frame tier's own fact (M71, ADR-0077, the tier's own kill rule): the frame itself — the tier every other tier's work rides — answered a log line and a skipped frame when it went unpainted; "why did nothing paint?" was unanswerable from the log alone. The record names the stage that refused: `layout_refused` (the tick's layout stage answered `LayoutError` — the engine seam's own failure or the no-root wiring bug, ADR-0023's error shape), `render_failed` (the backend's renderer refused a draw the frame asked for), `partial_repaint_failed` (the damage regions' scissored draw failed; the framebuffer survives), `framebuffer_lost` (the software path's persistent framebuffer could not be built), `present_failed` (the pixels were drawn and the present refused — the frame the user never saw). The failing node rides verbatim (the arena's own id vocabulary, ADR-0023's rule — the engine named it, the consumer resolves it against its own arena); a present-tier failure is the window's fact, not a widget's, and carries `node: null`. The record rides *beside* the refusal's own door — the tick still answers `Err(LayoutError)`, the ladder still skips the frame — never instead of it; the engines' prose summaries stay on the error types (the payload rule's shape: a fact, not a message). One fact per refusal, never per frame: the frames that painted stream no refusal fact (the frame summary prices those, the flood rule at the frame tier). The event streams at the tick's start drain, one frame after the frame that refused; the queue behind it is bounded (256 facts per thread, oldest-first eviction) and the recorder's `frame_refusal` ring holds 512. The additive event the versioning rule admits at version 1 — clients that ignore unknown kinds are unaffected.

### `view_patch` — a view patch resolved

 ("what did my re-render change — and how much?")

```json
{
  "kind":"view_patch",
  "created":1,
  "removed":0,
  "patched":0,
  "moved":0
}
```

The view tier's own fact (M72, ADR-0078, the tier's own kill rule): the view reconciliation — the tier every re-render rides — answered a `PatchReport` the imperative caller read and dropped (the mount's build streamed nothing at all), and the log carried no fact. The record carries the reconciler's four counts beside each other: `created` (subtree mounts — the build's opening count), `removed`, `patched` (a visual prop applied), `moved` (LIS non-members, §6.2). The counts are the fact, named by their lengths — the payload rule's shape: the props a patch applied never ride, the text they carry can. One fact per patch that moved something, never per frame: the frames that re-render nothing stream none, and the no-op re-render — a diff that moved nothing — streams none too (the idle-frame property, the m32 canon's own pre-registered rule; the flood rule at the view tier). The record rides *beside* the door's own `PatchReport` — `update` still answers the report it always answered — never instead of it. The event streams at the tick's start drain, one frame after the re-render that asked for the patch; the queue behind it is bounded (256 facts per thread, oldest-first eviction) and the recorder's `view_patch` ring holds 512. The additive event the versioning rule admits at version 1 — clients that ignore unknown kinds are unaffected.

### `setting` — an OS setting flip landed

 ("what does the vocabulary consult now?", "when did the OS move the facts?")

```json
{
  "kind":"setting",
  "dark":true,
  "reduced_motion":false,
  "high_contrast":false,
  "accent":null,
  "reduced_transparency":false,
  "repainted":42
}
```

The setting tier's own fact (M74, ADR-0080, ADR-0079's pre-registered re-open rule): the OS's dark-mode fact existed nowhere in the pipeline — a widget that wanted it guessed or polled — and the fact is a seam now, installed by the platform adapter and pushed through the door. The record carries the dark-mode fact beside the door that landed it: `dark` is the stable, documented field value — the bool *is* the fact. Since M76 (ADR-0082, ADR-0080's second pre-registered re-open rule) the event grows the broadcast's node count beside the fact — `repainted` is the additive field the versioning rule admits at version 1, the lengths rule's shape (the count names the flip's reach, the ids never riding), computed where the broadcast resolves and never re-counted at the reader; a tree that grows later never rewrites a queued record, and the no-op flip streams no event, so no count. Since M77 (ADR-0083, ADR-0080's first pre-registered re-open rule) the event grows the reduced-motion fact beside the dark one — `reduced_motion` is the additive field the versioning rule admits at version 1, one seam and one record carrying the facts the vocabulary consults beside each other, never a second event per fact; the seam answers the fact once per paint pass into `PaintCtx`, the door's flip check comparing both facts. Since M79 (ADR-0085, ADR-0083's first pre-registered re-open rule cashed a second time) the event grows the high-contrast fact beside the two — `high_contrast` is the additive field the versioning rule admits at version 1, one seam and one record carrying the facts the vocabulary consults beside each other, never a second event per fact; the seam answers the fact once per paint pass into `PaintCtx`, the door's flip check comparing all three facts. Since M81 (ADR-0087, ADR-0083's first pre-registered re-open rule cashed a third time, grown by ADR-0086's own rule beside it) the event grows the accent-color fact beside the three — `accent` is the additive field the versioning rule admits at version 1, the fact riding its presence: the documented `#RRGGBBAA` hex when the platform reports an accent, `null` when it reports none — the honest absence, never a guessed color; one seam and one record carrying the facts the vocabulary consults beside each other, never a second event per fact; the seam answers the fact once per paint pass into `PaintCtx`, the door's flip check comparing all four facts, and the composition resolves its accent roles by it beside the three (ADR-0086's own rule). Since M154 (ADR-0158, the same rule cashed a fourth time) the event grows the reduced-transparency fact beside the four — `reduced_transparency` is the additive field the versioning rule admits at version 1, one seam and one record carrying the facts the vocabulary consults beside each other, never a second event per fact; the fact answers the ask (`false` is effects allowed, the canon answer the default build resolves), the seam answers the fact once per paint pass into `PaintCtx`, the door's flip check comparing all five facts, and the composition resolves its translucency roles by it beside the four (ADR-0086's own rule). One fact per flip that landed, never per frame: a flip that lands where the fact already is (both facts) queues no masks and no facts (the idle-frame property, A1's rule at the setting tier), and the frames that paint under the facts stream no setting fact of their own — the frame summary prices those. The fact never munges the palette: the roles the widgets borrowed re-resolve from the installed palette exactly as they were. The record rides *beside* the door's `bool` answer (`PassPipeline::set_settings` answers whether the flip landed) never instead of it. The event streams at the tick's start drain (the setting drain's arm runs just ahead of the theme drain, so a frame's trail reads the fact the palette resolved beside → the palette that fact painted into), one frame after the door answered; the queue behind it is bounded (256 facts per thread, oldest-first eviction) and the recorder's `setting` ring holds 512. The additive event the versioning rule admits at version 1 — clients that ignore unknown kinds are unaffected.

### `theme` — a theme swap landed

 ("which palette is live now?", "when did the whole window repaint?", "how much did the swap repaint?")

```json
{
  "kind":"theme",
  "palette":"midnight",
  "repainted":42
}
```

The theme tier's own fact (M73, ADR-0079): the palette the widgets borrow for their un-overridden paint roles — compile-time constants through 0.16, no door answering "make it dark" at all — is a seam now, and the swap answers on the log. The record carries the installed `ThemeSeam`'s own `name()` tag beside the door that landed it: `palette` is the stable, documented field value (`"canon"` for the provided default, app-defined tags for every custom palette). The tag is the fact — the payload rule's shape: the log names the palette, the paint pass resolves it, the palette's colors never ride the wire. Since M78 (ADR-0084, ADR-0082's second pre-registered re-open rule reaching the theme tier) the event grows the broadcast's node count beside the tag — `repainted` is the additive field the versioning rule admits at version 1, the lengths rule's shape at the theme tier (the count names the swap's reach, the ids never riding), computed where the swap's broadcast resolves and never re-counted at the reader, the composed swap naming the same reach the flip's broadcast resolved (one flip, one count). One fact per swap that landed, never per frame: a swap that lands where the palette already is queues no masks and no facts (the idle-frame property, A1's rule at the theme tier), and the frames that paint the new palette stream no theme fact of their own — the frame summary prices those. The record rides *beside* the door's `bool` answer (`PassPipeline::set_theme` answers whether the swap landed) never instead of it. The event streams at the tick's start drain (the theme drain's arm runs after the view-patch drain, so a frame's trail reads routes → tiers → the palette those routes painted into), one frame after the door answered; the queue behind it is bounded (256 facts per thread, oldest-first eviction) and the recorder's `theme` ring holds 512. Since M75 the tag's vocabulary carries the composition door's swaps (ADR-0081): a `SettingComposer` the app installed through `PassPipeline::set_composer` resolves the palette the OS's flips compose, the composed swap streaming the composer's own `name()` tag (app-defined — the *Duet* demo streams `"duet"`) behind the flip's own `setting` event, the equality check against the effective palette holding unchanged. The additive event the versioning rule admits at version 1 — clients that ignore unknown kinds are unaffected.

### `cursor` — a hover's resolved cursor fact

 ("why did the pointer's shape move — which face named it?")

```json
{
  "kind":"cursor",
  "style":"grab",
  "source":"widget",
  "node":42,
  "role":"canvas"
}
```

The cursor tier's own fact (M153, ADR-0157): the pointer's hover shape existed nowhere in the log — M152's resolution door answered the platform's per-frame ask, and the churn's only observer was the platform's own debug line — and the trail is a tier now, the fact queued where the answer resolves. The record carries the style the hover resolves to beside the source that named it: `style` is the vocabulary's own `name()` tag (the stable, documented field value — `"pointer"`, `"text"`, `"grab"`, the CSS `cursor` words underscored), and `source` names the resolution's two honest answers — `"widget"` when a widget's hook named the shape, `"default"` when nothing accepted the hover (the honest absence, the absence named as a source, never guessed onto a widget that did not answer). The widget source rides the ids the tree event's vocabulary speaks — `node` (the id-path hash's `u64`) and `role` (the semantic role word) — both `null` when the default answered; the label never rides (the lengths rule — a hover is not a payload, and the log is a devtools surface, not a shadow tree). One fact per resolved *change*, never per frame: a style that did not move is not a fact (the flood rule at the cursor tier — the door coalesces against the last noted answer, the compare the still hover's whole cost), and the disabled trail queues nothing (the A7 idle contract's one branch per resolution). The event streams at the tick's start drain (the cursor drain's arm runs after the clipboard drain, the clipboard drain's shape — the fact queues where the hover resolves, one frame after the change), and the queue behind it is bounded (256 facts per thread, oldest-first eviction) and the recorder's `cursors` ring holds 512. The additive event the versioning rule admits at version 1 — clients that ignore unknown kinds are unaffected.

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
  "degradation_step":"none",
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
- `degradation_step` — the degradation ladder's step (architecture §14.4):
  what a frame over budget gave up — `none` (every pass ran inside the
  budget; the ladder is disarmed by default and every frame reports it),
  `drop_input` (the input drain blew its share — the batch's remainder
  stays queued and rides the next frame's drain, never a lost event),
  `force_full_paint` (a behind-schedule paint collapsed a partial
  present to one full frame), or `skip_a11y` (the frame was past the
  budget's halfway line when the a11y pass came due — the dirty updates
  deferred, their masks re-queued, the next frame lands the delta; focus
  moves, announcements, first frames, and structural churn never defer).
  The additive field the versioning rule admits at version 1 — clients
  that ignore unknown fields are unaffected.
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

### `stats` — the rings' occupancy answer (M141)

```json
{"kind":"stats","facts":1280,"rings":[{"name":"broadcast","held":96,"cap":4096,"evicted":12},{"name":"edges","held":1024,"cap":1024,"evicted":0},{"name":"frames","held":256,"cap":256,"evicted":1024}]}
```

The answer to the `stats` control: the recorder's fact count beside one
row per ring, the stream's own bound (`broadcast`) first, the trail rings
in the event vocabulary's order, `frames` last. The rows shown above are
elided — the real answer carries all twenty-four rows, exactly the rings
the bounds sentence below names.

- `facts` — the recorder's total fact count at the ask (the same
  monotonic count the greeting carries and the replay ask's cursor
  rides).
- `rings[].name` — the ring's wire name (`broadcast`, `edges`, `writes`,
  `inputs`, `focus`, `edits`, `announces`, `keys`, `clipboards`,
  `values`, `gestures`, `scrolls`, `text_writes`, `route_drops`,
  `key_edits`, `preedits`, `a11y_echoes`, `actions`, `frame_refusals`,
  `view_patches`, `themes`, `settings`, `cursors`, `frames`).
- `rings[].held` — the entries the ring holds right now.
- `rings[].cap` — the bound the ring is held to (the bounds sentence
  below is this row's contract).
- `rings[].evicted` — the entries the cap has evicted so far — the named
  loss. The counter survives `clear` (a reset empties the contents, it
  does not un-name the loss).

The rows tell a panel reading a replay burst which trails are complete
(`held` < `cap`, `evicted` 0) and which are capped (`held` == `cap`,
`evicted` climbing) — the burst's completeness is data, not a guess.
Additive at version 1: a new event kind, clients that ignore unknown
kinds are unaffected.

### `tree` — the resolved tree's shape (the dump ask's answer)

```json
{"kind":"tree","root":1,"nodes":[{"node":1,"role":"container","label":"window","bounds":[0.0,0.0,800.0,600.0],"enabled":true,"children":[2]},{"node":2,"role":"button","label":"Save","bounds":[16.0,20.0,80.0,24.0],"enabled":true,"children":[]}]}
```

The answer to the `tree_dump` control: the resolved tree's shape as the
a11y pass last answered it — the snapshot the observer's tree door
retained, one event, identities as plain integers. Like `stats`, the
event prices no ring: it is the ask's answer, never a captured fact,
never a streamed one (the live stream stays exactly as it was; a panel
that wants the shape asks).

- `root` — the root node's id, when a tree stands; `null` before the
  first a11y pass resolved one (the honest absence, never an invented
  shape).
- `nodes[]` — the nodes in pre-order (the tree walk order), the frame
  heatmap's own vocabulary grown the parentage and the availability the
  walk needs: `node` (the id-path hash), `role` (the wire's
  `snake_case` role word, the same vocabulary the heatmap and the
  `stats` rows speak), `label` (the resolved accessible name), `bounds`
  (`[x, y, w, h]` when laid out, `null` before the layout pass placed
  the node), `enabled` (the availability the AT reads — the `disabled`
  property's inverse), and `children` (the child ids in tree order).
- The content keys are absent, not empty: a field's `value`, its
  `selection`, the `placeholder` and the `description` beside them have
  no key in the shape — the secret-carrying rule at the dump tier, the
  rule the `clipboard`, `announce`, and `text_write` tiers already
  speak (payloads and contents never ride the wire; the labels are the
  pipeline's own resolved names, the same words the `frame` event's
  heatmap carries).

The shape is *state*, not history: `clear` empties the rings and leaves
the tree standing (a panel that asks again reads the shape the last
frame resolved), and a `replay` burst never carries a `tree` event. The
look reads without disturbing (the replay ask's own rule): no ring
drains, the broadcast queue is untouched. Additive at version 1 (the
`Replay` ask's precedent): a server older than the ask closes the
session on the unknown control, the documented unknown-frame behavior.

### `hit` — the point ask's answer (M146)

```json
{"kind":"hit","asked":[30.0,30.0],"landed":true,"miss":null,"ancestry":[{"node":1,"role":"container","label":"window","bounds":[0.0,0.0,800.0,600.0],"enabled":true,"children":[2]},{"node":2,"role":"button","label":"Save","bounds":[16.0,20.0,80.0,24.0],"enabled":true,"children":[]}]}
```

The answer to the `hit_ask` control: which node the asked point
touches — the geometry query over the same resolved snapshot the
`tree_dump` ask answers from, one event. The walk's landed-or-refused
shape (the `focus` event's own):

- `asked` — the point the ask carried, `[x, y]` in the tree's logical
  pixels — the answer echoes the ask (the replay cursor's own rule:
  the ask the panel reads the answer against).
- `landed` — whether the walk landed (`true`) or refused (`false`).
- `miss` — the refusal species, `null` on a landing: `no_tree` (before
  the first a11y pass resolved a tree), `no_geometry` (nodes stand,
  none carry bounds — the layout pass never wrote the geometry),
  `outside` (geometry stands, the point touches no node),
  `malformed` (the asked point is not a finite coordinate — the core
  door's own refusal; on the wire a non-finite ask is already the
  malformed frame's close, JSON cannot carry NaN or infinity).
- `ancestry` — the walk's path, root-first, the hit node last; each
  node the `tree` event's own node vocabulary (one shape, the two
  asks' doors, the two unable to drift apart), empty on a miss. The
  land rule is the paint order's own: among the nodes containing the
  point, the last in pre-order wins — the tree walk order is the paint
  order, later siblings composite on top, and the answer names what
  the pixels would answer. Containment is the geometry's own
  half-open convention (the min edges in, the max edges out — the
  convention the damage tests and the scroll clamps speak).
- The content keys are absent, not empty: the `tree` ask's
  secret-carrying rule at the point tier — the shape answers, the
  contents (a field's value, its selection, the placeholder and
  description beside them) never.
- Like `stats` and `tree`, the event prices no ring: the ask's
  answer, never a captured fact, never a streamed one. The ask reads
  without disturbing (the dump ask's rule): no ring drains, the
  broadcast queue is untouched, and the answer is the *state*. A
  `clear` emptied the rings but not the shape the ask answers from.
  Additive at version 1 (the `Replay` ask's precedent): a server
  older than the ask closes the session on the unknown control, the
  documented unknown-frame behavior.

### `find` — the word ask's answer (M147)

```json
{"kind":"find","query":"save","role":"button","matches":[{"node":2,"role":"button","label":"Save","bounds":[16.0,20.0,80.0,24.0],"enabled":true,"children":[]}]}
```

The answer to the `find_ask` control: the nodes whose resolved names
carry the query — the resolved snapshot's query over the same state
the `tree_dump` ask answers from, one event:

- `query` — the query the ask carried — the echo the panel reads the
  matches against (the `hit` answer's own rule).
- `role` — the role word the ask named, `null` the whole vocabulary.
- `matches` — the nodes whose resolved names carry the query,
  case-insensitive substring (the a11y query API's own rule), in tree
  order, each node the `tree` event's own node vocabulary. An empty
  list is the honest absence — no name carried the needle, never a
  refusal (the replay burst's empty answer, at the name tier). The
  empty needle names every resolved name — the ask's every-node
  answer.
- The query reads the resolved names — never the values, the
  selections, the hints: the secret-carrying rule at the name tier,
  the content keys absent from the shape, not empty.
- Like `stats`, `tree` and `hit`, the event prices no ring: the ask's
  answer, never a captured fact, never a streamed one. The ask reads
  without disturbing (the dump ask's rule): no ring drains, the
  broadcast queue is untouched, the answer is the *state*. Additive at
  version 1 (the `Replay` ask's precedent): a server older than the
  ask closes the session on the unknown control, the documented
  unknown-frame behavior.

### `region` — the region ask's answer (M148)

```json
{"kind":"region","asked":[0.0,0.0,120.0,60.0],"matches":[{"node":1,"role":"container","label":"window","bounds":[0.0,0.0,800.0,600.0],"enabled":true,"children":[2]},{"node":2,"role":"button","label":"Save","bounds":[16.0,20.0,80.0,24.0],"enabled":true,"children":[]}]}
```

The answer to the `region_ask` control: which nodes the asked rect
touches — the point ask's plural sibling over the same resolved
snapshot the `tree_dump` ask answers from, one event:

- `asked` — the rect the ask carried, `[x, y, width, height]` in the
  tree's logical pixels — the answer echoes the ask (the `hit`
  answer's own rule).
- `matches` — the nodes whose bounds intersect the rect, in tree
  order (pre-order — the paint order's own: later siblings composite
  on top, and the matches name the composite order the pixels
  answer), each node the `tree` event's own node vocabulary. The
  touch rule is the geometry's own intersection convention (the
  convention the damage tests speak). An empty list is the honest
  absence — no tree resolved yet, no bounds written, or the region
  touching nothing standing — never a refusal (the `find` answer's
  empty answer, at the region tier). A rect naming no positive area
  — inverted or empty extents — touches nothing among them: the same
  honest absence, never a guessed normalization.
- The rect's coordinates are the tree's own logical pixels (the
  bounds quadruple's frame). A rect whose coordinates are not finite
  cannot ride JSON — serde renders NaN and infinity as `null`, the
  array parse refuses — so a non-finite ask is a malformed control
  frame, the documented report-and-close behavior (the `hit_ask`
  rule, at the region tier).
- The content keys are absent, not empty: the `tree` ask's
  secret-carrying rule at the region tier — the shape answers, the
  contents (a field's value, its selection, the placeholder and
  description beside them) never.
- Like `stats`, `tree`, `hit` and `find`, the event prices no ring:
  the ask's answer, never a captured fact, never a streamed one. The
  ask reads without disturbing (the dump ask's rule): no ring drains,
  the broadcast queue is untouched, the answer is the *state*.
  Additive at version 1 (the `Replay` ask's precedent): a server
  older than the ask closes the session on the unknown control, the
  documented unknown-frame behavior.

### `path` — the path ask's answer (M149)

```json
{"kind":"path","asked":2,"landed":true,"miss":null,"ancestry":[{"node":1,"role":"container","label":"window","bounds":[0.0,0.0,800.0,600.0],"enabled":true,"children":[2]},{"node":2,"role":"button","label":"Save","bounds":[16.0,20.0,80.0,24.0],"enabled":true,"children":[]}]}
```

The answer to the `path_ask` control: where the asked node stands —
the ancestry the node stands in, the point ask's landed-or-refused
shape at the address tier, over the same resolved snapshot the
`tree_dump` ask answers from, one event:

- `asked` — the node id the ask carried — the answer echoes the ask
  (the `hit` answer's own rule). The id is the one the `tree`, the
  `hit`, the `find` and the `region` answers already name: the ask
  family composing.
- `landed` — whether the chain answered (`true`) or the ask was
  refused (`false`).
- `miss` — the refusal species, `null` on a landing: `no_tree`
  (before the first a11y pass resolved a tree), `no_node` (a tree
  stands; the asked id names nothing in it).
- `ancestry` — the asked node's chain, root-first, the asked node
  last (the root's ancestry the root alone); each node the `tree`
  event's own node vocabulary, empty on a miss. The chain is the
  parentage the resolved tree carries — the walk `hit_at` descends
  with, answered for the id the panel already holds.
- The content keys are absent, not empty: the `tree` ask's
  secret-carrying rule at the address tier — the shape answers, the
  contents (a field's value, its selection, the placeholder and
  description beside them) never.
- Like `stats`, `tree`, `hit`, `find` and `region`, the event prices
  no ring: the ask's answer, never a captured fact, never a streamed
  one. The ask reads without disturbing (the dump ask's rule): no
  ring drains, the broadcast queue is untouched, the answer is the
  *state*. Additive at version 1 (the `Replay` ask's precedent): a
  server older than the ask closes the session on the unknown
  control, the documented unknown-frame behavior.

### `descendants` — the descendant ask's answer (M150)

```json
{"kind":"descendants","asked":2,"landed":true,"miss":null,"beneath":[{"node":3,"role":"button","label":"Save","bounds":[16.0,20.0,80.0,24.0],"enabled":true,"children":[]},{"node":4,"role":"label","label":"Status","bounds":[16.0,60.0,80.0,24.0],"enabled":true,"children":[]}]}
```

The answer to the `descendants_ask` control: what stands beneath the
asked node — the subtree, the path ask's landed-or-refused shape at
the beneath tier, over the same resolved snapshot the `tree_dump` ask
answers from, one event:

- `asked` — the node id the ask carried — the answer echoes the ask
  (the `path` answer's own rule). The id is the one the `tree`, the
  `hit`, the `find`, the `region` and the `path` answers already name:
  the ask family composing.
- `landed` — whether the subtree answered (`true`) or the ask was
  refused (`false`).
- `miss` — the refusal species, `null` on a landing: `no_tree`
  (before the first a11y pass resolved a tree), `no_node` (a tree
  stands; the asked id names nothing in it) — the path ask's species
  vocabulary shared, one species list two doors.
- `beneath` — the subtree under the asked node, tree order (pre-order,
  the paint order's own); each node the `tree` event's own node
  vocabulary, empty on a miss — and on a leaf's landing, the honest
  absence (nothing stands beneath, never a refusal). The asked node
  itself never rides: it is the anchor the ask already names.
- The content keys are absent, not empty: the `tree` ask's
  secret-carrying rule at the beneath tier — the shape answers, the
  contents (a field's value, its selection, the placeholder and
  description beside them) never.
- Like `stats`, `tree`, `hit`, `find`, `region` and `path`, the event
  prices no ring: the ask's answer, never a captured fact, never a
  streamed one. The ask reads without disturbing (the dump ask's
  rule): no ring drains, the broadcast queue is untouched, the answer
  is the *state*. Additive at version 1 (the `Replay` ask's
  precedent): a server older than the ask closes the session on the
  unknown control, the documented unknown-frame behavior.

### `siblings` — the sibling ask's answer (M151)

```json
{"kind":"siblings","asked":3,"landed":true,"miss":null,"beside":[{"node":4,"role":"label","label":"Status","bounds":[16.0,60.0,80.0,24.0],"enabled":true,"children":[]}]}
```

The answer to the `siblings_ask` control: what stands in the parent's
row beside the asked node — the parent's other children, the
descendant ask's landed-or-refused shape at the beside tier, over the
same resolved snapshot the `tree_dump` ask answers from, one event:

- `asked` — the node id the ask carried — the answer echoes the ask.
  The id is the one the family's answers already name: the ask family
  composing.
- `landed` — whether the row answered (`true`) or the ask was refused
  (`false`).
- `miss` — the refusal species, `null` on a landing: `no_tree`
  (before the first a11y pass resolved a tree), `no_node` (a tree
  stands; the asked id names nothing in it) — the family's species
  vocabulary shared, one species list three doors.
- `beside` — the parent's other children, tree order (pre-order, the
  paint order's own); each node the `tree` event's own node
  vocabulary, empty on a miss — and on the root's or an only child's
  landing, the honest absence (the asked node stands beside nothing,
  never a refusal). The asked node itself never rides: it is the
  anchor the ask already names.
- The content keys are absent, not empty: the `tree` ask's
  secret-carrying rule at the beside tier — the shape answers, the
  contents never.
- Like `stats`, `tree`, `hit`, `find`, `region`, `path` and
  `descendants`, the event prices no ring: the ask's answer, never a
  captured fact, never a streamed one. The ask reads without
  disturbing (the dump ask's rule): no ring drains, the broadcast
  queue is untouched, the answer is the *state*. Additive at version 1
  (the `Replay` ask's precedent): a server older than the ask closes
  the session on the unknown control, the documented unknown-frame
  behavior.

## Client → server

```json
{"kind":"clear"}
```
Clears the server-side history — the rings (the latest frame and the
resolved tree stand: state, not history). The client keeps its own
buffer.

```json
{"kind":"replay"}
```
Asks the server to replay the facts its rings hold — the durable per-tier
history a reconnecting panel never sees through the shared broadcast queue
(whose contents a first session's drain already took). The answer is one
burst of the rings' events, oldest first per trail, in the event
vocabulary's order (§ above, the kinds' own sequence), the frames last.
The ask drains nothing: the broadcast queue is untouched, the rings keep
their contents, so facts the session saw live may repeat in the burst —
the client folds the duplicates. An empty history answers an empty burst
(the session simply keeps serving). Additive at version 1: servers older
than the ask close the session on the unknown control, the documented
unknown-frame behavior — a client that needs the ask can probe or reconnect.

```json
{"kind":"replay","after":41}
```
The replay ask with a cursor (M140): only the facts recorded *after*
fact number 41 answer the burst — the history at or below the cursor
stays silent, so a panel that already carried the first 41 facts asks
with the number its first greeting's `facts` field named and the rings
answer with the delta, not the whole worth. The field is optional and
defaults to absent — the bare ask replays the rings whole, exactly as
shipped. A cursor above every recorded fact answers an empty burst (the
honest absence); the session keeps serving either way. Facts the client
streamed live above its cursor may still repeat in the burst — the
folding contract stands, the client owning the dedup (the pre-registered
rule's own words: priced by that client).

```json
{"kind":"stats"}
```
Asks the server what its rings hold right now (M141): the answer is one
`stats` event — the fact count beside one row per ring — so a panel
reading a replay burst knows what the history *cannot* carry. The look
reads without disturbing (the replay ask's own rule at the stats tier):
no ring drains, the broadcast queue is untouched, the live stream beside
the answer continues untouched. Additive at version 1 (the `Replay`
ask's precedent): a server older than the ask closes the session on the
unknown control, the documented unknown-frame behavior.

```json
{"kind":"subscribe","tiers":["frames","edges"]}
```
Asks the server to gate this session's live stream to the named tiers
(M142): the facts whose ring the ask names answer the session's fan-out
from this frame's drain onward, the unasked tiers staying silent. The
ask is the *session's* fan-out, never the recorder's capture — the rings
keep recording every tier (the replay and stats asks answer whole either
way), so the gate drops nothing the rings hold: the panel's own silence,
never a loss (the eviction counters name the losses the caps cause; a
muted tier is not one of them). The names are the `stats` answer's row
vocabulary — the twenty-two trail rings (`edges`, `writes`, `inputs`,
`focus`, `edits`, `announces`, `keys`, `clipboards`, `values`,
`gestures`, `scrolls`, `text_writes`, `route_drops`, `key_edits`,
`preedits`, `a11y_echoes`, `actions`, `frame_refusals`, `view_patches`,
`themes`, `settings`, `frames`), one list the two doors read, the two
unable to drift apart. `broadcast` is deliberately absent — the stream
is the medium, never a tier of itself — and a name outside the
vocabulary is a malformed control frame: the session's documented
report-and-close behavior (the unknown control's own rule, at the name
tier). The field is optional and defaults to absent:

```json
{"kind":"subscribe"}
```
The bare ask restores the whole stream — the shape every pre-M142
session rode. An empty list is the mute:

```json
{"kind":"subscribe","tiers":[]}
```
The live stream goes quiet for this session while the rings keep
recording — the replay burst's reading panel's ask (the facts recorded
during the silence answer through the replay ask, the stats rows naming
them as they climb). The asks' own answers are never gated: the events
no ring captures (the `stats` occupancy answer among them) ride whole —
the ask is a live-stream ask, never an answer-silencer. Since M145 the
ask's shape is what the server remembers (ADR-0149): the gate persists
across reconnects, every later session starting under it and reading it
back from the greeting's `gate` field — a fresh server greets the whole
stream, the bare ask's restore and the mute persisting exactly as the
asks named them. Additive at
version 1 (the `Replay` ask's precedent): a server older than the ask
closes the session on the unknown control, the documented
unknown-frame behavior.

```json
{"kind":"tree_dump"}
```
Asks the server for the resolved tree's shape — the a11y pass's latest
snapshot, retained by the recorder through the observer's tree door.
The answer is one `tree` event: the roles, the resolved names, the
bounds, the availability, and the parentage a panel walks; the contents
the secret-carrying rule keeps off the wire have no key in the shape.
The ask reads without disturbing (the replay ask's own rule): no ring
drains, the broadcast queue is untouched, and the answer is the *state*
— a `clear` emptied the rings but not the shape, and before the first
a11y pass resolves a tree the answer is the honest empty shape
(`root` null, no nodes). Additive at version 1 (the `Replay` ask's
precedent): a server older than the ask closes the session on the
unknown control, the documented unknown-frame behavior.

```json
{"kind":"hit_ask","point":[120.5,84.0]}
```
Asks the server which node the named point touches (M146) — the point
ask riding the same retained snapshot the `tree_dump` ask answers
from. The answer is one `hit` event: the walk's landed-or-refused
shape, the ancestry the walk descended through (root-first, the hit
node last), the refusal a named species. The coordinates are the
tree's own logical pixels (the bounds quadruple's frame). A point
whose coordinates are not finite cannot ride JSON — serde renders NaN
and infinity as `null`, the array parse refuses — so a non-finite ask
is a malformed control frame, the documented report-and-close
behavior. The ask reads without disturbing (the dump ask's own rule).
Additive at version 1 (the `Replay` ask's precedent): a server older
than the ask closes the session on the unknown control, the documented
unknown-frame behavior.

```json
{"kind":"find_ask","query":"save","role":"button"}
```
Asks the server for the nodes whose resolved names carry the query
(M147) — the word ask riding the same retained snapshot the
`tree_dump` ask answers from. The answer is one `find` event: the
matches in tree order, the ask echoed beside the answer. The query is
case-insensitive substring over the resolved names — never the values
the secret-carrying rule keeps off the wire. The `role` field is
optional and defaults to absent — the bare ask answers the whole
vocabulary:

```json
{"kind":"find_ask","query":"save"}
```
A word outside the roles' list (`Role`'s own `snake_case` words, the
`tree` event's role vocabulary) is a malformed control frame: the
session's documented report-and-close behavior (the unknown control's
own rule, at the name tier). Additive at version 1 (the `Replay` ask's
precedent): a server older than the ask closes the session on the
unknown control, the documented unknown-frame behavior.

```json
{"kind":"region_ask","rect":[120.0,84.0,200.0,120.0]}
```
Asks the server which nodes the named rect touches (M148) — the region
ask riding the same retained snapshot the `tree_dump` ask answers from,
the point ask's plural sibling. The answer is one `region` event: the
touched nodes in tree order (the paint order's own), the ask echoed
beside the answer, an empty list the honest absence. The rect is
`[x, y, width, height]` in the tree's logical pixels (the bounds
quadruple's frame); a rect naming no positive area — inverted or empty
extents — touches nothing (the same honest absence, never a guessed
normalization). A rect whose coordinates are not finite cannot ride
JSON — serde renders NaN and infinity as `null`, the array parse
refuses — so a non-finite ask is a malformed control frame, the
documented report-and-close behavior. The ask reads without disturbing
(the dump ask's own rule). Additive at version 1 (the `Replay` ask's
precedent): a server older than the ask closes the session on the
unknown control, the documented unknown-frame behavior.

```json
{"kind":"path_ask","node":2}
```
Asks the server where the named node stands (M149) — the path ask
riding the same retained snapshot the `tree_dump` ask answers from, the
point ask's shape at the address tier. The answer is one `path` event:
the ancestry the node stands in (root-first, the asked node last), the
ask echoed beside the answer, the refusal a named species (`no_tree`
before the first tree resolves, `no_node` for an id naming nothing
standing). The id is the one the dump, the hit, the find and the
region answers already name — the ask family composing (the word ask's
matches feeding the walk). The ask reads without disturbing (the dump
ask's own rule). Additive at version 1 (the `Replay` ask's precedent):
a server older than the ask closes the session on the unknown control,
the documented unknown-frame behavior.

```json
{"kind":"descendants_ask","node":2}
```
Asks the server what stands beneath the named node (M150) — the
descendant ask riding the same retained snapshot the `tree_dump` ask
answers from, the path ask's shape at the beneath tier. The answer is
one `descendants` event: the subtree in tree order (the asked node the
anchor the ask names, never its own answer's row), the ask echoed
beside the answer, the refusal a named species (`no_tree` before the
first tree resolves, `no_node` for an id naming nothing standing), a
leaf's landing the honest absence. The id is the one the dump, the
hit, the find, the region and the path answers already name — the ask
family composing. The ask reads without disturbing (the dump ask's own
rule). Additive at version 1 (the `Replay` ask's precedent): a server
older than the ask closes the session on the unknown control, the
documented unknown-frame behavior.

```json
{"kind":"siblings_ask","node":3}
```
Asks the server what stands in the parent's row beside the named node
(M151) — the sibling ask riding the same retained snapshot the
`tree_dump` ask answers from, the descendant ask's shape at the beside
tier. The answer is one `siblings` event: the parent's other children
in tree order (the asked node the anchor the ask names, never its own
answer's row), the ask echoed beside the answer, the refusal a named
species (`no_tree` before the first tree resolves, `no_node` for an id
naming nothing standing), the root's landing the honest absence — the
root stands beside nothing. The id is the one the family's answers
already name — the ask family composing. The ask reads without
disturbing (the dump ask's own rule). Additive at version 1 (the
`Replay` ask's precedent): a server older than the ask closes the
session on the unknown control, the documented unknown-frame behavior.

```json
{"kind":"bye"}
```
Ends the session; the server echoes `close` and the socket closes. Any
malformed text frame is treated as `bye`.

## Ordering, bounds, and overhead

- Events are delivered in record order; the broadcast queue between the
  observer thread and the session is bounded at 4 096 events and drops the
  oldest (a slow client never back-pressures the app). A `subscribe` ask
  gates the session's fan-out to the named tiers (M142) — the gate
  applying to the facts the session delivers from the ask's drain
  onward, the capture and the asks' answers untouched. Since M145 the
  gate is the server's memory (ADR-0149): the last session's ask is the
  shape every later session starts under, one panel's worth — a fresh
  server has none.
- The recorder's rings are bounded (frames 256, edges 1 024, writes 1 024,
  inputs 512, focus 512, edits 512, announces 512, keys 512,
  clipboards 512, values 512, gestures 512, scrolls 512, text_writes 512,
  route_drops 512, key_edits 512, preedits 512, a11y_echoes 512, actions
  512, frame_refusals 512, view_patches 512, themes 512, settings 512,
  cursors 512);
  `clear` empties them; `replay` reads them without disturbing; `stats`
  answers their occupancy, caps, and evictions without disturbing either.
  The `stats` answer's row list is this sentence's own list — the two
  cannot drift apart — and the `subscribe` ask's tier vocabulary is the
  same list (the trail names only; `broadcast` the stream's own bound).
- Recording is opt-in: the pipeline forwards only when an observer is
  installed, and the flux records activations and provenance only when
  `Flux::set_causal_capture(true)`. With both off, the framework's default
  paths pay a branch.

## Versioning

Additive changes bump the minor semantics inside `version`: new event kinds
and new optional fields are compatible; renaming or reshaping existing
fields bumps `version`. Clients must ignore unknown event kinds and unknown
fields.
