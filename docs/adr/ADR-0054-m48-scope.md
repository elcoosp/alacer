# ADR-0054: M48 scope — the drag's wall is a named fact

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0053's first kill rule — "a consumer needs a
  bound jump the *pointer* asked for (a drag that slams an edge) →
  the M46 gesture record's phases already carry the gesture's own
  story; grow that vocabulary, never the AT tier's"
- **Companion:** ADR-0052 (the gesture record whose vocabulary grows),
  ADR-0053 (the kill rule this cashes), ADR-0049/ADR-0051 (the stamp
  discipline the bound rides beside), ADR-0026 (the value tier whose
  range owns the walls), architecture §13.3, §18 (the M48 roadmap row)

## Decision

- **The wall the pointer pinned is the drag's own fact.**
  `alacer-core/src/inspect.rs`: `GestureRecord` grows `bound:
  Option<RangeBound>` — the range's edge the `Ended` record's landing
  pinned (`min`/`max`), the ADR-0053 kill rule's answer on the
  vocabulary the rule named. `RangeBound` (`Min`/`Max`) is the one
  named-edges vocabulary, its `snake_case` tags riding the wire's
  documented-field-values rule additively at version 1 (clients that
  ignore unknown fields are unaffected). A bound jump munged into the
  AT tier's species would collapse the pointer's story into a door it
  never rode — the kill rule's own words.
- **The beginning is not the wall's fact.** Every `Started` record and
  every mid-track `Ended` record carries the honest absence: the
  landing, and so the bound, is the gesture's answer — a `None` names
  "no wall" as truthfully as a `Some` names the edge.
- **The drag's door computes it where the knowledge lives.**
  `alacer-widgets/src/slider.rs`: the release and the leave route
  their `Ended` records through one `landed_bound()` helper — the
  widget's own `total_cmp` equality against the range it owns, never a
  bound guessed from the landing by the trail's readers. The stamp
  rides for free (the M46 pointer route's guard), the flood rule
  untouched (the moves still stream nothing — the wall rides the
  per-gesture record, never a per-move fact).
- **The wire and the client speak it.** `alacer-inspect/src/wire.rs`:
  the `gesture` event carries the additive `bound` field; the embedded
  client renders a pinned landing as "…, pinned at the max" and a
  mid-track landing exactly as before — the wall readable from the
  log pane alone.
- **`examples/drag_bounds.rs` is the consumer demonstration** (tracing
  at `alacer::drag_bounds`): the real pointer route, the slam at the
  max, the leave at the min, the mid-track absence, the wall's
  silence.
- **No new corpus case: the corpus stands at 64.** The route the wall
  rides is already measured (`gestures/route_100_drag_moves_1k`); the
  bound is one `Option` computed once per gesture, inside the budget
  the contract already gates. The living documents re-agree at the cut
  (the README, the register, the protocol document's gesture
  vocabulary, the catalog, the CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer needs the *wheel* or the *chords* to ask for a bound →
  the ask's own tier grows its fact beside its species, never the
  gesture record's (the ask's wall is the asker's story).
- A consumer needs *how far* the ask overshot the wall it pinned →
  the bound's payload grows the overshoot beside the species, never a
  second bound species.
