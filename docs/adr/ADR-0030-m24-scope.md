# ADR-0030: M24 scope — the focus walk joins the dispatch contract (the consumer door over the Tab cycle, the micro-debt the sweep priced)

- **Status:** Accepted (2026-11-10)
- **Implements:** the dispatch-contract candidate the M23 cut left
  private — the Tab/Shift+Tab cycle has walked the candidate list since
  M3, but the walk itself was `cycle_focus`, a private method reachable
  only when the focused widget ignored the key, and the pipeline's
  `focus()`/`focusables()` read doors had no writer a consumer could
  reach without naming a node (`request_focus`) and owning the order
  themselves — plus the surface work the M16–M23 pattern leaves open:
  the micro-debt the sweep priced and the living documents' remainder
- **Companion:** ADR-0014 (the dispatch-contract shape the door joins),
  ADR-0025 (the diagnostic species the funnel's refusals join),
  ADR-0028 (the availability tier the walk's cycle-time filter serves),
  ADR-0029 (the cut whose sweep this one continues), architecture §10.3
  (the keyboard contract the walk belongs to), §18 (the M24 roadmap row)

## Decision

Every dispatch seam since M19 grew a public door the keyboard route and
the consumer's code share — the capture holds its target for any press,
the value tier answers any of its four doors with the same funnel. The
focus walk skipped its own pattern: the traversal was the one dispatch
behavior a consumer could not drive. M24 opens it without re-inventing
it:

- **The walk is a vocabulary, not a loop the caller writes.**
  `FocusMove { Next, Prev }` names the direction; the pipeline's
  `focus_move` is the consumer's door over the walk. The order stays the
  pipeline's business — the pre-order candidate list pass 2 recomputes,
  the availability tier's cycle-time `enabled` filter applied (M22,
  ADR-0028) — so a call site never duplicates a traversal rule the
  pipeline already owns.
- **One funnel, two doors.** `dispatch_key`'s Tab/Shift+Tab fallback now
  routes through `focus_move` — the keyboard and the consumer's code
  land on the same stops because there is one walk, not two copies of
  it. The funnel resolves immediately (the same rule Tab rode: key
  routing must see the new focus this event), wraps at both ends, and
  from no focus takes the first candidate in either direction (the Tab
  contract as shipped, documented at the door).
- **The refusal is honest.** When no candidate exists — an empty arena,
  or every control the availability tier holds down — the door answers
  `false` and queues nothing: no masks, an idle frame, focus left where
  it was. It never munges the walk into a first-candidate reset (the
  M11 rule, at the focus seam). The refusals and the landings ride the
  `alacer::focus` tracing target — the workspace's per-seam idiom, one
  span per door call, the direction on the span.
- **The read side already shipped; the cut says so.**
  `PassPipeline::focus()` (the node holding focus) and
  `PassPipeline::focusables()` (the candidate list) are the walk's read
  doors; `examples/focus_traversal.rs` gives them the consumer that
  reads them — the landing named through the tree's own required
  `label()`, never widget internals.
- **The demonstration walks the beats.** A four-control form (two
  inputs, a checkbox, a send button) steps forward, backward, and into
  the honest refusal while the availability tier holds every control
  down — the mutate seam's write answered, the form handed back before
  the window opens, where Tab and Shift+Tab prove the funnel live. The
  m24 exit suite pins the delta headlessly: the two doors land on the
  same candidates in the same order, the walk skips the unavailable
  control without a recompute, a refusal costs an idle frame, and one
  move costs exactly one frame (the ring repaint lands, the frame after
  is idle).
- **The micro-debt the sweep priced rides the cut.** The flux runtime's
  dead `effect_count` introspection goes — zero call sites anywhere in
  the workspace, the M23 species' remainder (the accessor that looked
  like surface). The harness's dead `spec_of`, `with_text_context`, and
  `text_size` probes go with it — the setup/introspection surface no
  test, bench, or example ever called. No weak errors remained for the
  ADR-0025 species to price this cut: the M22/M23 diagnostics closed the
  macro's remainder, and the focus seam's refusals are answers, not
  errors.
- **The living documents re-agree at M24.** The README's status block
  and count (M0 through M24, thirty-one decision records), the features
  bullet, the quickstart, and the example table; the catalog's row; the
  register (29 → 30 with this ADR, the preamble's stale
  `ADR-0012…ADR-0028` range repaired); the roadmap row; the m1 suite's
  header, which said "36 blessed snapshots" over a corpus its own pin
  counts at 37 — the drift the sweep caught; the m20–m23 suites'
  document pins, which move with the documents in the same cut — the
  chain cannot drift; and this changelog stages the milestone.

## Consequences

- The walk is a read-side vocabulary over existing state: no new pass,
  no new mask value, no scheduler change — the door reuses `cycle_focus`
  (the walk Tab has ridden since M3) and the immediate-resolution rule
  it already shipped. The hot path carries no new work; `dispatch_key`'s
  call sites swap one private call for the same walk behind a span.
- The corpus does not grow: the demonstration shows no new visual
  arrangement (the catalog's own rule for when an example needs a
  scenario — the focus ring's paint already ships), and the m1 suite's
  37-scenario pin stands.
- The perf contracts do not grow either: the focus-move route
  (`input/route_100_focus_moves_1k`) is priced as the next
  measurement-run candidate and joins the published corpus by the
  coordinated rule — measured, published, and gated at one run (the
  checker fails an uncontracted benchmark; CONTRIBUTING's publishing
  rule keeps the half-published bench out of the tree).
- No new dependency edges: the seam's spans ride the `tracing` core
  every member already resolves, and the help-text discipline stays
  plain diagnostic text (ADR-0029's rule — the miette-class `help:` the
  species ships, never the miette crate).
- The m-suite's document pins now move across five suites; the next
  release moves all of them together or the m24 suite fails loudly.

## Kill criteria (re-open rules)

- A walk needs an order the tree order cannot express (spatial/D-pad
  navigation, a reading-order walk) → grow the vocabulary as a second
  door beside `focus_move`, priced by a real UI — never a second
  pipeline channel or a client-side reimplementation of the candidate
  list.
- A consumer needs the walk bounded to a subtree (a modal's focus trap)
  → re-open the seam with the scope as a predicate beside `focusable`
  (the ADR-0028 re-open rule's shape), not a second walk.
- A door needs to observe the walk's refusals as data (a status line
  that reports *why* nothing moved) → grow the answer into a result
  vocabulary priced by that consumer; until one exists, the boolean is
  the honest shape.
