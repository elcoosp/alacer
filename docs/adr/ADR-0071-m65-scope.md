# ADR-0071: M65 scope — the route drop names its rider and splits its silence

- **Status:** Accepted (2026-11-14)
- **Implements:** two pre-registered prices — ADR-0063's third kill
  rule (the drops need their rider: a reader asking "which route
  refused my ask" could not read the platform boundary's mapping arms,
  the dispatch funnel's target resolution, and the IME route's silence
  apart from one collapsed record) and ADR-0065's first kill rule (the
  dead focused node's drop prices apart from the never-focused one: the
  committed composition with nowhere to land named the same
  `no_focused_field` species whichever way it had nowhere)
- **Companion:** ADR-0063 (the boundary-drop trail the record rides),
  ADR-0065 (the IME silence the split prices), ADR-0036 (the M30 rider
  rule the attribution generalizes), A7 (the idle contract: the trail's
  single branch when nobody listens is unchanged), architecture §13.3,
  §18 (the M65 roadmap row)

## Decision

- **The record names its rider.**
  [`RouteDropRider`](crates/alacer-core/src/inspect.rs) lives beside the
  drop vocabulary — `Boundary` (the platform's mapping arms, the ask
  never became a core action), `Funnel` (the dispatch door's own
  refusal, the unknown target), `ImeRoute` (the composing event no live
  field held) — one additive `RouteDropRecord::rider` field, the
  attribution declared where the drop resolves (the note site knows its
  own tier; the trail never guesses it from the species, the M30 rule
  at the route tier).
- **The silence splits.** `RouteDropSpecies::NoFocusedField` keeps the
  never-focused half; the focused node's death rides its own
  `RouteDropSpecies::DeadFocus` (`dead_focus`, the wire's `snake_case`
  tag) — the IME route's else branch answers which absence it is, the
  aim riding its honest absence either way (the drop *is* the missing
  field).
- **The wire grows additively at version 1.** The `route_drop` event
  carries the `rider` field and the new species value (the clipboard
  `engine` field's rule — clients that ignore unknown fields are
  unaffected); the embedded client renders the rider beside the species
  ("dropped: unknown_target (aimed at …) — the funnel refused it"); the
  protocol document's section carries the field and the tag.
- **No new corpus case.** The field rides the routes the drops already
  ride and the record grows a `Copy` discriminant on a route no
  criterion case prices (the M57 precedent: the drops ride no measured
  route) — the corpus stands at 68, the m5/m6 pins re-agree unchanged.
- **The demo walks the rider.** `examples/route_drops.rs` walks the
  rider beats: the funnel's drop naming the funnel, the boundary's
  species naming the boundary (pinned by `alacer-platform`'s unit
  tests), the never-focused commit naming the IME route, and the
  committed composition whose field died mid-session naming
  `dead_focus` — the log answering "which route said no, and why did
  the ask have nowhere" from the rows alone.
