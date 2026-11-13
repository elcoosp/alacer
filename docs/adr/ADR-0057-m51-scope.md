# ADR-0057: M51 scope — the content-side clamp joins the causal log

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0056's third kill rule — "a consumer needs the
  content-side clamp named — `set_content_height` silently re-clamping
  the offset when the content shrinks — → the door grows its own fact
  (the clamp is the content's story, not the ask's; today it rides
  outside the log)"
- **Companion:** ADR-0056 (the scroll trail whose record the fact
  rides), ADR-0055 (the kill rules the scroll tier cashed), ADR-0054
  (the `RangeBound` vocabulary the wall speaks), ADR-0050 (the ask the
  landing is named against), ADR-0046 (the value trail's shape the
  scroll trail rides), ADR-0010 (the additive wire rule), architecture
  §13.3, §18 (the M51 roadmap row)

## Decision

- **The content's door grows its own fact.** The scroll doors — the
  wheel, the AT's unit scrolls and value steps, the app's own
  programmatic scroll — stream their asks since M50, but the content's
  door stayed silent: `ListHandle::set_content_height` re-clamped the
  offset when the content shrank under it and said nothing — the one
  scroll answer no ask produced, invisible on the log the other doors
  feed. The record vocabulary
  (`alacer-core/src/inspect.rs`): `ScrollAction` grows `ContentClamp`
  (the content's own story — the door with no ask, `scroll_to`'s
  no-route rule at the content's door) and `ScrollRefusal` grows
  `NonFiniteContent` (the ask tier's `non_finite_ask` wall, at the
  content's door). The delivered record carries the landing beside the
  wall it pinned — `pinned: Option<RangeBound>`, ADR-0054's one
  named-edges concept spoken identically wherever a landing names it,
  never a second action species (the kill rule's own words) — with
  `requested: None` (a resize is not an ask; the honest absence the
  field's contract names) and `target: None` (the app door rides no
  route, the stamp rule's shape).
- **The wall computes where the knowledge lives.**
  `alacer-widgets/src/list.rs`: `set_content_height` computes the
  re-clamp through the same `pinned_offset_bound` arithmetic the ask
  doors answer by (the ask the clamp judged is the offset it found —
  the landing at the new edge *and* the held offset somewhere else,
  the widget's `total_cmp` equality answering both), streams one
  record when the offset moved, and queues nothing when it did not —
  growth never moves an offset, a shrink the span still covers never
  lands elsewhere: the M20 idle-frame property at the content's own
  door.
- **The micro-debt the record's design priced is cashed with it.** A
  non-finite content height rode the clamp arithmetic and spread `NaN`
  into the offset *and* the stored span alike — the list scrolled to
  nothing, permanently, with no error anywhere (the exact corruption
  the M50 trail exposed at the wheel door, still open at the content's
  own door). The door now refuses the height before the arithmetic:
  the state keeps the height and offset it held, the refusal
  (`non_finite_content`) is a named fact on the log, and the next good
  ask answers like nothing happened.
- **The wire and the client speak it.** `alacer-inspect/src/wire.rs`:
  the `scroll` event's action vocabulary grows the `content_clamp` tag
  and its refusal vocabulary the `non_finite_content` tag at version 1
  (clients that ignore unknown strings are unaffected — the additive
  rule ADR-0010 names); the embedded client renders the content's
  sentence — "the content clamp landed the offset 200, pinned at the
  max" — beside the refusals, the log pane answering "why did the
  offset move — or why didn't it" from the content's side alone.
- **`examples/content_clamp.rs` is the consumer demonstration**
  (tracing at `alacer::content_clamp`): the real app door — the
  handle's resize — walking the clamp's stage, the pinned landing, the
  resizes that queue nothing, the non-finite refusal with the state
  intact, and the wall spoken identically on both stories; the live
  window's *Shrink the content* / *Grow the content* buttons ride the
  same door.
- **No new corpus case: the corpus stands at 64.** The door rides no
  route and runs outside the frame's hot paths — a resize happens
  after structural changes, not per frame — and the record is one
  `Copy` struct computed once per answered resize, inside the budgets
  the contracts already gate. The living documents re-agree at the cut
  (the README, the register, the protocol document's scroll
  vocabulary, the catalog, the CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer needs the offset the clamp *found* named beside the
  landing — the held offset the content shrank under — → the record
  grows the fact beside the action (the story is derivable from the
  trail's prior records today; the growth waits for a consumer that
  cannot derive it).
- A consumer needs the content clamp attributed to a control the way
  the routes stamp the asks — the resize the app fires from code
  outside the handle's reach — → the door grows a route of its own
  (the app's handle knows no node id today; the stamp's absence is the
  honest one).
- A consumer needs the same fact for a *growing* content — a viewport
  that shrinks under a static content (the window resized) — → the
  door grows the viewport's story beside the content's (the viewport
  is fixed at construction today; no resize ever re-answers it).
