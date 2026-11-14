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
  ADR-0016), `increment`/`decrement` (the honest value-step for a
  scrollbar whose value *is* the offset — one page), or `scroll_to`
  (the app's own programmatic door, no route behind it) — and since M51 the content's own story rides the same vocabulary: `content_clamp` (ADR-0057, the price ADR-0056's third kill rule pre-registered — the clamp `set_content_height` answers when the content shrank under the offset; the clamp is the content's story, not the ask's). Since M52 the unit scrolls name their asker's unit in the event's `unit` field (ADR-0058).
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
dispatch funnel's ask aimed at an id no tree node carries, and — since
M59 (ADR-0065) — the IME event no live field held (a committed
composition with nowhere to land) — stream one flat record each. A drop never rides a widget-door record (no door ever
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
  live field held — no field has focus, or the focused node has gone;
  the `action` rides the event's own tag `ime_preedit`/`ime_commit`,
  and `target` rides its honest absence — the boundary had no aim to
  name).
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
double-priced edit); the AT's and app's writes ride
`text_write` — the doors tellable apart on one log.

```json
{
  "kind": "key_edit",
  "action": "insert",
  "driver": "keyboard",
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
kinds are unaffected.


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
