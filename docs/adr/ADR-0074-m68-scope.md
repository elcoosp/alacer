# ADR-0074: M68 scope — the payloadless chord names its read

- **Status:** Accepted (2026-11-14)
- **Implements:** ADR-0068's second kill rule — a consumer prices the
  payloadless chord (the copy or cut whose selection is empty: the
  chord arrived, the field's door answered, nothing was there to move,
  and the key-edit trail answered nothing — "why did my copy do
  nothing" unanswerable from the field's tier alone) → the wall grows
  the read's species beside the door's, never a double-priced round
  trip
- **Companion:** ADR-0066 (the chords' attribution rule the ask rides),
  ADR-0068 (the wall whose vocabulary the read joins), ADR-0045 (the
  clipboard trail whose one fact the round trip keeps), A7 (the idle
  contract: the note rides the door's own branch), architecture §13.3,
  §18 (the M68 roadmap row)

## Decision

- **The wall grows the read's species.**
  [`KeyEditRefusal`](crates/alacer-core/src/inspect.rs) carries
  `EmptySelection` beside the availability tier's `Unavailable` — the
  open door's read finding nothing to move, the field's own answer as
  data. The ask is named by the door's own match: the action
  vocabulary grows [`KeyEditAction::Copy`] (the chord whose field
  change is nothing — a delivered copy answers no fact, the m60 rule
  standing; the refusal names the chord the door actually answered),
  beside the chords' own `Cut`.
- **One fact per trail, never a double-priced ask.** The round trip
  keeps its one clipboard fact (the `NoSelection` species the M39
  trail already streams); the field's refusal rides the key-edit trail
  beside it — two trails, one fact each, the delivered cut's exact
  shape mirrored at the wall. A delivered copy and a delivered cut
  stream no refusal (the wall prices refusals, never landings).
- **The paste keeps the honest absence.** The refused paste's read is
  the clipboard seam's (the engine never ran, the m60 pin stands): the
  wall prices the field's own read, and a paste's read is not the
  field's.
- **The wire grows additively at version 1.** The `key_edit` event
  carries the new `action` value (`copy`) and the new `refusal`
  species (`empty_selection`) — the additive rule ADR-0010 names;
  clients that ignore unknown tags are unaffected; the embedded client
  renders the refusal row it already renders, the species beside it.
- **No new corpus case.** The refusal rides the editing route the
  key-edit bench already prices (the wall off the delivered hot path,
  the M62 precedent); the corpus stands at 68, the m5/m6 pins
  re-agree unchanged.
- **The demo walks the read beat.** `examples/paste_edits.rs` routes
  the payloadless copy at startup — no selection, the chord arrives,
  the trail label mirrors the refusal the field answered beside the
  round trip's own fact.
