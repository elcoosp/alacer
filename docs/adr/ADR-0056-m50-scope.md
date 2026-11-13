# ADR-0056: M50 scope — the scroll's story joins the causal log

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0055's second kill rule — "a consumer needs the
  wall named on a tier whose funnel knows no range (the text field's
  clamps, the list's scroll clamp) → that tier's own record grows its
  own fact beside its species, never the value record's (the wall is
  the asker's story — ADR-0054's rule at the next tier)"
- **Companion:** ADR-0055 (the kill rule this cashes), ADR-0054 (the
  `RangeBound` vocabulary the wall rides), ADR-0052 (the pointer tier's
  per-gesture discipline the stamp follows), ADR-0050 (the ask the
  landing is named against), ADR-0049/ADR-0051 (the stamp discipline),
  ADR-0046 (the value trail whose shape the scroll trail rides),
  ADR-0016 (the AT scroll vocabulary the doors answer), architecture
  §13.3, §18 (the M50 roadmap row)

## Decision

- **The scroll tier's own record is the fact.** The scroll doors —
  the wheel, the AT's unit scrolls and value steps (ADR-0016: the
  scrollbar's value *is* the offset), the app's own programmatic
  scroll — answered the dispatch contract in silence: a wheel past the
  content's end clamped without a word, a wheel over a list whose
  content fits the viewport moved nothing unexplained. The record
  vocabulary (`alacer-core/src/inspect.rs`): `ScrollRecord` grows beside
  the trail family — `action` (`ScrollAction`: `Wheel`, `ScrollUp`,
  `ScrollDown`, `Increment`, `Decrement`, `ScrollTo`), `outcome`
  (`ScrollOutcome::Delivered`, `Refused(ScrollRefusal)`), the landed
  `offset` beside the finite `requested` ask (the M44 shape at the
  scroll tier), the `target` the routes stamp (the M43/M45/M46
  discipline at the scroll tier), and `pinned: Option<RangeBound>` —
  ADR-0054's one named-edges concept, spoken identically wherever a
  landing names it, never a second action species (the kill rule's own
  words). The refusals: `no_scrollable_content` (the range has no span
  — no scroll exists to deliver) and `non_finite_ask` (the funnel
  carries only finite asks — the value record's own rule).
- **The trail is the value trail's shape (ADR-0046).**
  `alacer-core/src/scroll.rs`: the thread trail — bounded at 256,
  oldest-first eviction, `note_scroll` at the door that knows the
  answer, the dispatching pipeline draining at the tick's start into
  `InspectEvent::Scroll`, dev-mode and silent while nobody listens (the
  A7 idle contract: one branch per ask).
- **The doors compute the wall where the knowledge lives.**
  `alacer-widgets/src/list.rs`: one `answer_scroll_ask` helper answers
  by the clamp arithmetic all three doors share, one `pinned_offset_bound`
  computes the wall (the landing at an edge *and* the ask somewhere
  else, the widget's own `total_cmp` equality answering both), and each
  door notes its own action species at its own site — the wheel in
  `on_pointer_event`, the AT asks in `on_a11y_action` (the actions
  outside the scroll tier — the press, the value writes — stay refused
  and silent: a question outside the tier is not a tier fact, the
  ADR-0046 rule), the app's handle in `ListHandle::scroll_to` (no route
  — the stamp's honest absence).
- **The micro-debt the trail's design exposed is cashed with it.** A
  non-finite wheel delta rode the clamp arithmetic and spread `NaN`
  into the scroll state — the list scrolled to nothing, permanently,
  with no error anywhere. The door now refuses the ask before the
  arithmetic: the state keeps the offset the ask found, the refusal
  (`non_finite_ask`) is a named fact on the log, and the next good ask
  answers like nothing happened.
- **The routes hold the stamp.** `alacer-core/src/passes.rs`: the wheel
  route holds one `ScrollTargetGuard` for the length of one routed
  wheel (the deepest hit under the hover — the wheel is the only
  pointer ask the scroll tier answers), the action route holds one
  beside the value stamp (the id the AT action arrived under), the
  drain rides after the gesture drain (a scroll explains the viewport
  the wheel moved; it streams beside the gestures).
- **The wire and the client speak it.** `alacer-inspect/src/wire.rs`:
  the `scroll` event carries the action, the outcome, the refusal
  species, the landed `offset` beside the additive `requested` and
  `pinned` fields at version 1 (clients that ignore unknown fields are
  unaffected); the embedded client renders the ask's sentence — "asked
  450, landed 400, pinned at the max" — beside the refusals, the log
  pane answering "did the list scroll — where, or why not" alone.
- **`examples/scroll_trail.rs` is the consumer demonstration** (tracing
  at `alacer::scroll_trail`): the real pointer route and the real AT
  route, the walls, the ask that wanted the wall exactly, the fitted
  list's refusal, the non-finite ask refused before it corrupts, and
  the app's own door.
- **No new corpus case: the corpus stands at 64.** The routes the ask
  rides are already measured (the wheel route under `input/` and the
  trail drains inside the tick); the record is one `Copy` struct
  computed once per answered ask, inside the budgets the contracts
  already gate. The living documents re-agree at the cut (the README,
  the register, the protocol document's scroll vocabulary, the catalog,
  the CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer needs *the asker's unit* named — one item or one page —
  on the AT's unit scrolls → the record grows the unit beside the
  action species (the distance is derivable from the ask and the
  landing today; the growth waits for a consumer that cannot derive
  it).
- A consumer needs the horizontal axis — a list that scrolls on both
  axes — → the record grows the axis beside the action (the vertical
  ask is the only ask today; the wheel's x delta is ignored by the
  door, an absence the log does not price yet).
- A consumer needs the content-side clamp named — `set_content_height`
  silently re-clamping the offset when the content shrinks — → the
  door grows its own fact (the clamp is the content's story, not the
  ask's; today it rides outside the log).
