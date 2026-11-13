# ADR-0059: M53 scope — the wheel's line count joins the scroll record

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0058's second kill rule — "a consumer needs *the
  wheel's line count* named — the coalesced run's notional lines — →
  the record grows the count beside the wheel species (the run is the
  coalescer's story, not a unit ask; today the distance is derivable
  from the ask and the landing)"
- **Companion:** ADR-0056 (the scroll trail the field rides),
  ADR-0058 (the unit field whose absence the wheel rides),
  ADR-0016 (the AT scroll vocabulary whose line convention the run
  prices by), ADR-0010 (the additive wire rule), architecture §10.2,
  §13.3, §18 (the M53 roadmap row)

## Decision

- **The wheel run's notional lines ride beside the wheel species.**
  The M50 trail priced every scroll ask, and M52 gave the AT's unit
  scrolls their unit — but the wheel's coalesced run still streamed
  its answer without the magnitude the pointer asked by. The record
  says "asked 450, landed 400" and a reader who wants the run's own
  story — *how many notches did the wheel turn* — must re-derive it
  from the distance, and only when the clamp never moved the landing
  (a pinned ask's distance is the clamp's, not the run's). The
  consumer the demonstration priced cannot: a status line or an
  inspector client reading one record sees the offsets and cannot say
  whether the run was one notch or a flick. The record vocabulary
  (`alacer-core/src/inspect.rs`): `ScrollRecord` grows an additive
  `lines: Option<f32>` — the coalesced run's notional line count,
  priced by the platform's own convention (one line = 40 px, §10.2 —
  the exact rule the platform's wheel conversion prices `LineDelta`
  with, so a notch answers 1.0 and the trackpad's pixel runs answer
  their honest fraction) — `Some` beside the
  [`ScrollAction::Wheel`] action only. The honest absence on every
  other action: the AT's unit scrolls ask by their unit (M52, the
  unit the distance is priced against — a second magnitude would be
  the same story told twice), the value steps by the page (the step
  is the distance), the app's own door and the content's own clamp
  (ADR-0057) are stories with no wheel run behind them. The
  non-finite run rides the absence too: the ask never became a finite
  scalar, the refusal (`non_finite_ask`) names the wall, and a count
  of a run that never was would be the lie the M44 rule forbids.
- **The door prices the run where the delta is decoded.**
  `alacer-widgets/src/list.rs`: the wheel door computes the count from
  the coalesced delta it already holds — the magnitude priced by the
  same `A11Y_LINE` convention the AT's item scrolls answer one item
  by — before the clamp arithmetic, so a pinned run still names the
  run it was (the count is the ask's story, never the landing's). The
  other doors — the AT actions, the app's handle, the content's
  clamp — fill the absence, the field's contract naming each absence
  as the honest one, never a guessed count reconstructed from the
  distance.
- **The wire and the client speak it.** `alacer-inspect/src/wire.rs`:
  the `scroll` event carries the additive `lines` field at version 1
  (the notional count on the wheel's run, `null` elsewhere — clients
  that ignore unknown fields are unaffected, the additive rule
  ADR-0010 names); the embedded client renders the run beside the
  landing — "the scroll landed (the offset 400) — the run priced 1
  line" — the stories with no run rendering the absence, never a
  guessed notch.
- **`examples/wheel_lines.rs` is the consumer demonstration**
  (tracing at `alacer::wheel_lines`): the real pointer route — the
  one-notch run, the multi-notch run, the trackpad's fractional run,
  the pinned run that names the run it was, the non-finite run's
  refusal, and the stories with no run riding the absence.
- **No new corpus case: the corpus stands at 64.** The field rides
  the route the wheel already rides — the pointer route is measured —
  and the count is one `f32` priced once per answered ask at the door
  that already holds the delta, inside the budgets the contracts
  already gate. The living documents re-agree at the cut (the README,
  the register, the protocol document's scroll vocabulary, the
  catalog, the CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer prices the run's *direction* apart from its magnitude —
  a run that names which way it travelled as its own fact → the
  record grows the direction beside the count (today the direction is
  the ask's own story — `requested` beside the landing — and the
  count prices the magnitude).
- A consumer needs the *trackpad's pixel run* named in its own unit —
  a run the wheel convention does not price honestly (a pixel-exact
  story the 40 px line pricing distorts) → the record grows the
  pixel distance beside the count (the convention's own second
  answer).
- A consumer prices the *keyboard's* scroll ride — the arrow-key
  fallback the focus ring routes — as a scroll fact → the tier grows
  the species the funnel already routes (today the ride stays an
  input fact, the tier boundary the log prices).
