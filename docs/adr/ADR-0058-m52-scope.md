# ADR-0058: M52 scope — the asker's unit joins the scroll record

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0056's first kill rule — "a consumer needs *the
  asker's unit* named — one item or one page — on the AT's unit
  scrolls → the record grows the unit beside the action species (the
  distance is derivable from the ask and the landing today; the growth
  waits for a consumer that cannot derive it)"
- **Companion:** ADR-0056 (the scroll trail the field rides),
  ADR-0057 (the content clamp whose absence the field respects),
  ADR-0046 (the value trail's shape the scroll trail rides),
  ADR-0016 (the AT scroll vocabulary whose unit the field names),
  ADR-0010 (the additive wire rule), architecture §13.3, §18 (the M52
  roadmap row)

## Decision

- **The asker's unit rides beside the action species.** The M50 trail
  priced every scroll ask, but the AT's unit scrolls streamed their
  answer without the unit they asked by — one item or one page, the
  distance the ask is priced against, derivable only by re-deriving
  the widget's own unit arithmetic (`A11Y_LINE` against the viewport)
  from the outside. The consumer the demonstration priced cannot: a
  status line or an inspector client reading one record sees "asked
  140, landed 140" and cannot say whether the asker meant one item or
  half a page. The record vocabulary
  (`alacer-core/src/inspect.rs`): `ScrollRecord` grows an additive
  `unit: Option<ScrollUnit>` — `Some` beside the
  `ScrollUp`/`ScrollDown` actions (ADR-0016's unit vocabulary, the
  AccessKit shape core already mirrors), the honest absence on every
  other action: the wheel's notch is a coalesced run, not a unit ask;
  the value step's page is the step, not a unit scroll; the app's own
  door and the content's own clamp (ADR-0057) are stories with no
  unit at all. `ScrollUnit` grows its `name()` — `item`/`page`, the
  protocol's `snake_case` rule, the same naming discipline the action
  species and the refusal species ride.
- **The doors fill it where the ask is known.**
  `alacer-widgets/src/list.rs`: the AT action door fills the unit on
  the two unit-scroll species (the ask the door already decoded), the
  wheel door, the app door, and the content door ride the absence —
  the field's contract names each absence as the honest one, never a
  guessed unit reconstructed from the distance.
- **The wire and the client speak it.** `alacer-inspect/src/wire.rs`:
  the `scroll` event carries the additive `unit` field at version 1
  (`item`/`page` on the unit scrolls, `null` elsewhere — clients that
  ignore unknown fields are unaffected, the additive rule ADR-0010
  names); the embedded client renders the asker's unit beside the
  landing — "the scroll landed (the offset 140) by one page" — the
  stories with no unit rendering the absence, never a guessed edge.
- **`examples/scroll_units.rs` is the consumer demonstration**
  (tracing at `alacer::scroll_units`): the real AT action route — the
  item scroll and the page scroll with their units beside the
  landings, the reverse ask, the step's honest absence, the wheel's,
  and the app door's.
- **No new corpus case: the corpus stands at 64.** The field rides the
  routes the asks already ride — the action route is measured — and
  the unit is one `Copy` enum carried in a record computed once per
  answered ask, inside the budgets the contracts already gate. The
  living documents re-agree at the cut (the README, the register, the
  protocol document's scroll vocabulary, the catalog, the CHANGELOG,
  the m-suite pins).

## Kill criteria (re-open rules)

- A consumer needs the *step's* unit named — the value step's page is
  a unit-like distance a reader may want attributed the same way — →
  the record grows the unit beside the step species too (the kill
  rule's words named the AT's unit scrolls alone; the step's page
  rides the absence until a consumer prices it).
- A consumer needs the wheel's *line count* named — the coalesced
  run's notional lines — → the record grows the count beside the
  wheel species (the run is the coalescer's story, not a unit ask;
  today the distance is derivable from the ask and the landing).
- A consumer needs a *character* or *pixel* unit — an AT asking by a
  unit the vocabulary does not carry — → `ScrollUnit` grows the
  species beside `Item`/`Page` (the AccessKit shape core mirrors
  carries exactly the two today; the growth waits for the platform's
  own).
