# ADR-0065: M59 scope — the IME's boundary drops join the route-drop vocabulary

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0063's first kill rule — "a consumer prices the
  *platform's* other silences — the adapter activation's timing, the
  a11y sink's full-tree fallbacks, the present path's drops — on the
  log → the tier that resolves them grows its own record beside its
  route" — the reader prices one of them now: an IME event no live
  field held (no field has focus, or the focused node has gone)
  dropped with only a consumed `Ignored` verdict — "my IME committed;
  where did the text go" was unanswerable from the log alone. The
  route tier resolves this silence, so the route's own vocabulary
  grows the species; the keyboard's keys are not a silence (the
  keymap consult streams its `KeyRecord` whoever holds focus), the
  IME route was the one that swallowed whole events
- **Companion:** ADR-0063 (the route-drop record the species joins —
  the M57 vocabulary, one record per drop), ADR-0010 (the additive
  wire rule), architecture §13.3, §18 (the M59 roadmap row), ADR-0064
  (the M58 cut this rides beside — the IME route's other half: the
  funnel stamp the delivered commit now carries)

## Decision

- **The route names its other silence.** `dispatch_ime`
  (`crates/alacer-core/src/passes.rs`) routed one event and answered
  `Ignored` when no live field held focus — the committed composition
  vanished with nothing on the log (the M36 rule's own shape, at the
  IME route). The event joins the route-drop trail
  (`crates/alacer-core/src/route_drop.rs`): one `RouteDropRecord` —
  the event named by its stable `snake_case` tag
  (`ime_preedit`/`ime_commit`, the boundary's own name for what
  arrived, never a munge onto the widget action it did not become),
  the species naming why the boundary refused
  (`no_focused_field`), and the aimed id riding its honest absence
  (`None` — the boundary had no aim to name; the drop *is* the
  missing field, and a re-resolved aim would be a lie). A delivered
  commit never rides a drop record: the widget's door answered, the
  M57 rule's honest absence holds.
- **No new trail, no new wire kind.** The record rides the route-drop
  trail the M57 cut built (bounded 256, the tick's start drain, the
  text-write trail's shape) — the tier that resolves the silence
  already owns the stream, and a second trail for one species would
  be two logs for one route. The wire's `route_drop` event carries
  the species as data (the additive rule ADR-0010 names:
  `no_focused_field` is a new *value* of a documented field, not a
  new kind — `PROTOCOL_VERSION` stays 1, clients that render the
  species string render the echo unchanged). The embedded client's
  prose reads "dropped: no_focused_field" — no client change.
- **The example grows the beat.** `examples/route_drops.rs` walks a
  fourth beat — the committed composition no field held, beside the
  delivered asks and the unknown targets — one log carrying the
  boundary's refusals and its silences tellable apart.
- **No new corpus case: the corpus stands at 65.** The note rides the
  dispatch route the events already take, outside the frame's hot
  loops (the drops are the cold path; one branch per dropped event
  while nobody listens — the A7 idle contract, the coordinated
  rule's no-growth arm, the M57 precedent). The living documents
  re-agree at the cut (the README's species lists, the register, the
  protocol document's `route_drop` vocabulary, the CHANGELOG).

## Kill criteria (re-open rules)

- A consumer prices the *dead focused node's* drop apart from the
  never-focused one (two species where one stands) → the vocabulary
  splits the species behind a real reader's ask, never a
  speculative arm.
- A consumer prices the *adapter activation's timing* or the *a11y
  sink's full-tree fallbacks* on the log → the tier that resolves
  them grows its own record beside its route (the kill rule's own
  words), priced by a real reader, never a boundary-wide echo
  channel.
- A consumer needs the *dropped composition's shape* — how large the
  text the IME carried was — → the record grows the length behind an
  explicit opt-in seam (the clipboard trail's own re-open rule),
  never a default-on wire field.
