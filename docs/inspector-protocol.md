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

### `frame` — a completed frame ("why is this frame slow?", "what did this mask touch?")

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
  inputs 512, focus 512, edits 512); `clear` empties them.
- Recording is opt-in: the pipeline forwards only when an observer is
  installed, and the flux records activations and provenance only when
  `Flux::set_causal_capture(true)`. With both off, the framework's default
  paths pay a branch.

## Versioning

Additive changes bump the minor semantics inside `version`: new event kinds
and new optional fields are compatible; renaming or reshaping existing
fields bumps `version`. Clients must ignore unknown event kinds and unknown
fields.
