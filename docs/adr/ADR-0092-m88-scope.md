# ADR-0092: M88 scope — the bring-into-view door, the scroll ask priced in nodes

- **Status:** Accepted (2026-11-16)
- **Implements:** the complement the scroll tier's own door set always
  named (ADR-0056): the wheel, the AT's unit scrolls and value steps,
  and the app's raw-offset handle all answer asks *priced in offsets*
  — the ask the wheel's route, the AT's action, or the app's arithmetic
  computed before the clamp saw it — while the ask consumers actually
  hold is priced in *nodes* ("this item, this row, this focused
  control"). Through M87 that ask had no door: an app bringing a row
  into view read the container's bounds by hand, priced the offset
  itself, and rode the raw-offset handle with the geometry knowledge
  duplicated at the call site — or skipped the row. The door this cut
  ships is the node-priced ask: [`PassPipeline::scroll_into_view`]
  (crates/alacer-core/src/passes.rs) reads the bounds ledger it already
  owns, computes the minimal offset that lands the aimed node fully
  inside the nearest scrollable ancestor's viewport, and hands the ask
  to the container's own door — [`Widget::bring_into_view`]
  (crates/alacer-core/src/widget.rs), the clamp arithmetic answering
  where the range lives, never the pipeline guessing at a
  widget-private span
- **Companion:** ADR-0056 (the scroll trail the record rides — the
  stamp discipline, the named refusals, the idle contract), ADR-0033
  (the answer-as-data rule the outcome carries, the focus walk's shape
  at the scroll tier), ADR-0011 (the coordinated rule the corpus grows
  by), architecture §8.2 (the translation-only workhorse), §18 (the
  M88 roadmap row)

## Decision

- **The compute lives where the knowledge lives, split exactly in
  two.** The pipeline owns the tree geometry — pass 6's absolute
  composed bounds, the ledger every rider already reads — so the
  pipeline prices the ask: the node's content-space rect is the bounds'
  own difference (container and descendants ride the same compose
  translation, so the difference is invariant under every offset the
  bounds already reflect), the ask is the minimal offset that lands the
  rect fully inside the viewport — above the window scrolls to the top
  edge, below it to the bottom edge, a rect taller than the viewport
  pins its top edge (deterministic, the scroll-into-view convention).
  The container owns the range — its span lives in widget-private
  state — so the container answers: [`Widget::bring_into_view`] applies
  the ask through the same clamp arithmetic every door in
  [`alacer_widgets::List`] answers by (the walls, the named refusals,
  the state moved only on a delivery). Neither half guesses at the
  other's knowledge.
- **The answer is data at every landing, and the refusals never
  reach the tier.** The door answers
  [`ScrollIntoViewOutcome::Landed`] (the container that moved beside
  the container's own clamp answer), [`ScrollIntoViewOutcome::Visible`]
  (the node already fully inside the viewport — nothing to move,
  nothing to ask: no masks, no record, the idle frame the content
  clamp's own rule rides, the M51 idle property at the door's tier),
  or [`ScrollIntoViewOutcome::Refused`] with a named species —
  `unknown_node`, `not_laid_out`, `no_scrollable_ancestor`,
  `no_scroll_door`. The refusals are the *door's* facts, answered as
  typed data alone: a question outside the tier is not a tier fact
  (the ADR-0046 rule the scroll tier rides), so they queue no masks
  and stream no record — the ask that never reached a scroll answer
  never prices one.
- **The landing streams the two ids one ask produced.** A landed ask
  rides the scroll log under the new
  [`ScrollAction::BringIntoView`] species (`bring_into_view`, the
  wire's `snake_case` tag) with the record's stamps naming the
  container whose offset moved (`target`, the M50 discipline) and —
  beside it, additive at version 1 — the node the ask was *for* (the
  new `aimed` field, the M30 attribution rule's shape at the scroll
  tier: every causal fact names who asked and who answered). The
  stamps live in the route (`crate::scroll::ScrollAimGuard` beside the
  wheel route's own), the widget's door never guessing an identity it
  cannot know. `ScrollIntoViewOutcome::Visible` and every refusal
  stream nothing — the log prices the tier's answers, never the
  door's bookkeeping.
- **The currency is the wheel's own.** A delivered ask merges
  `PAINT | A11Y` on the container (§7.2 pass 10, the wheel's merge on
  consumption): compose translates the moved descendants without a
  measure call, the paint pass re-emits their layers, the a11y pass
  re-announces their bounds — one ask, one frame of work, the
  translation-only workhorse doing the moving.
- **The corpus carries the tier's pricing.** The corpus grows 93 → 94
  by the coordinated rule (ADR-0011: bench, gate, and baseline in one
  cut): `scroll/route_100_bring_into_view_1k` — one hundred asks
  alternating between a 1,000-node stack's two ends, each ask riding
  one tick of the real frame currency, so the door's price is the
  scrolled frame's own (compose translation, paint re-emission, a11y
  bounds refresh), never a stand-in. The m-suite pins re-agree at 94.
- **`examples/scroll_into_view.rs` is the consumer demonstration**
  (headless, the real pipeline and the software raster): the visible
  ask's honest no-op, the straddling row's landed ask beside its
  record, the scroll-back, the pixel the offset moved, and the dead
  id's typed refusal.

## Re-open rules (pre-registered)

- a consumer needs the *horizontal* axis — a container that scrolls on
  both axes — → the door grows the axis beside the ask (the vertical
  scalar is the scroll vocabulary's only offset today, ADR-0056's own
  axis rule standing; the ask's x half prices itself on a real reader
  first).
- a consumer needs the *deeper* containers to follow — a node nested
  two scrolls deep asking both — → the walk grows a cascade beside the
  nearest-ancestor answer, each container's ask priced by its own
  clamp, never a recursive ask this door guesses at today.
- a consumer needs the walk's landing to *follow* — the focus walk
  bringing the landed candidate into view — → the composition joins
  the two doors at the call site first (`focus_move` beside
  `scroll_into_view`); a framework-level join is priced by a real
  reader that cannot compose them, never a hidden scroll inside the
  walk (the walk's no-mask refusal stays).
- a consumer needs the AT's own bring-into-view ask — the screen
  reader's scroll-into-view action riding the adapter — → the action
  route grows the rider beside the app door (the M30 rider rule at the
  scroll tier), the record naming who asked, never a second door.
