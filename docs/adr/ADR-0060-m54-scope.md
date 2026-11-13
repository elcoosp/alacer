# ADR-0060: M54 scope — the value step's page joins the scroll record

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0058's first kill rule — "a consumer needs *the
  step's* unit named — the value step's page is a unit-like distance a
  reader may want attributed the same way — → the record grows the
  unit beside the step species too (the kill rule's words named the
  AT's unit scrolls alone; the step's page rides the absence until a
  consumer prices it)"
- **Companion:** ADR-0058 (the unit field whose absence the step
  rides no more), ADR-0056 (the scroll trail the field rides),
  ADR-0016 (the AT scroll vocabulary whose value step the page
  names), ADR-0010 (the additive wire rule), architecture §13.3, §18
  (the M54 roadmap row)

## Decision

- **The unit rides beside the step species.** The M52 cut gave the
  AT's unit scrolls their unit and left the value steps —
  `Increment`/`Decrement`, the honest value-step for a scrollbar whose
  value *is* the offset (M10, ADR-0016) — riding the absence, the
  kill rule's own words holding the door open: the step's page is a
  unit-like distance, and the consumer the demonstration priced
  cannot say from one record whether the step stepped a page or a
  line. It steps a page — always, by the action vocabulary's own
  contract — and the record names it now. The vocabulary
  (`alacer-widgets/src/list.rs`): the AT action door fills the unit
  beside the step species too — `Some(ScrollUnit::Page)` beside
  `ScrollAction::Increment` and `ScrollAction::Decrement`, the same
  field, the same `name()` discipline (`page`, the protocol's
  `snake_case` rule), never a second magnitude beside the count the
  wheel's run prices (M53, ADR-0059 — the count is the wheel's story,
  the unit is the ask's; the step carries the unit and no count, the
  wheel the count and no unit). The honest absence keeps its other
  stories: the wheel's coalesced run (a run is not an ask — M52's
  own words, and M53 gave it its own magnitude beside the unit's
  seat), the app's own door, and the content's own clamp (ADR-0057)
  are stories with no unit at all.
- **The wire and the client speak it unchanged.** The wire's `unit`
  field already rides every scroll event (M52's additive field) — the
  step's page is the vocabulary's own answer filling the seat the
  null held, no new field, no version bump (the additive rule
  ADR-0010 names: the field's *contract* grew beside its shape, the
  clients that render `null` as absence rendering "by one page"
  unchanged); the embedded client renders the step's page beside the
  landing — "the scroll landed (the offset 100) by one page" — the
  stories with no unit still riding the null.
- **`examples/step_pages.rs` is the consumer demonstration**
  (tracing at `alacer::step_pages`): the real AT action route — the
  increment and the decrement each naming the page they stepped by,
  the reverse step, and the stories with no unit (the wheel's run,
  the app door, the clamp) riding the absence beside them.
- **No new corpus case: the corpus stands at 64.** The field rides
  the route the step already rides — the action route is measured —
  and the unit is one `Copy` enum carried in a record computed once
  per answered ask, inside the budgets the contracts already gate.
  The living documents re-agree at the cut (the README, the register,
  the protocol document's scroll vocabulary, the catalog, the
  CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer prices the *step's own magnitude* apart from the page —
  a scrollbar whose value step is a line or a configurable distance →
  the step grows its distance beside the unit (today the page is the
  step, the vocabulary's own contract).
- A consumer needs the *app door's* ask attributed a unit — a
  programmatic scroll a reader wants priced like the AT's → the door
  grows the unit the app passes (today the handle rides no unit, the
  app door's rule).
- A consumer prices a *horizontal* step — the axis the scroll tier
  does not carry yet → the tier grows the axis beside the unit (the
  kill rule ADR-0056 already pre-registered, still waiting).
