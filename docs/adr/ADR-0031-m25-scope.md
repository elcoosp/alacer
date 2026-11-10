# ADR-0031: M25 scope — the focus scope: the walk bounds to a subtree (the modal's focus trap, the scope as the walk's second shape, the micro-debt the sweep priced)

- **Status:** Accepted (2026-11-10)
- **Implements:** the re-open rule ADR-0030 pre-registered — "a consumer
  needs the walk bounded to a subtree (a modal's focus trap) → re-open
  the seam with the scope as a predicate beside `focusable`, not a
  second walk" — plus the surface work the M16–M24 pattern leaves open:
  the micro-debt the sweep priced and the living documents' remainder,
  this time including the release-state pin that missed two sweeps
- **Companion:** ADR-0030 (the walk the scope bounds; the re-open rule
  this ADR cashes), ADR-0028 (the availability tier the scoped walk's
  cycle-time filter serves), ADR-0025 (the diagnostic species the
  refusals join), ADR-0029 (the plain-help-text rule the refusals ride),
  architecture §10.3 (the keyboard contract the walk belongs to), §18
  (the M25 roadmap row)

## Decision

Every dispatch seam since M19 grew its public door beside the keyboard
route it shares, and each milestone's re-open rules named the next
vocabulary a real UI would price. The modal's focus trap is the one the
M24 cut named first — a consumer walking a dialog's form must never
offer the window behind it — and M25 cashes that rule without
re-inventing the walk:

- **The scope is the subtree's root node.** `PassPipeline::focus_move_in`
  takes `scope: NodeId` and `movement: FocusMove` — the cheapest honest
  predicate specialization the priced UI needs. A predicate *argument*
  (the closure shape "beside `focusable`") stays a re-open: no closures
  in the public API, no generic at the door, and a subtree root is
  exactly what a modal app holds (the container it mounted). A trap with
  holes, or a scope over several subtrees, re-opens the seam again with
  that vocabulary — priced by a real UI, never speculatively.
- **One walk, two scopes.** The M24 funnel is untouched:
  `dispatch_key`'s Tab/Shift+Tab fallback and `PassPipeline::focus_move`
  keep their signatures and their behavior. Behind both doors the walk
  generalizes to `cycle_focus_in(scope: Option<NodeId>, reverse)` —
  `None` is the walk Tab has ridden since M3, `Some(root)` filters the
  same candidate list by the same cycle-time `enabled` check (M22,
  ADR-0028) through one ancestor walk (`is_in_scope`). The scope bounds
  the walk, never the candidate list: `focusables()` and `focus()` keep
  answering the whole tree, and a root-scoped walk lands exactly where
  the unscoped door lands (the m25 exit suite pins the two against each
  other).
- **The trap's rules are the walk's rules.** From no focus — or from a
  focus outside the trap — the scoped walk takes the trap's first
  candidate; it wraps at the trap's own ends, skips controls the
  availability tier holds down without a recompute, and resolves
  immediately (key routing sees the landing this event). A move rides
  the one invalidation currency: `FOCUS | PAINT` masks on the two
  endpoints, one frame of work, the frame after idle (A7).
- **The refusal is honest and named.** A dead scope, an empty trap, or a
  trap the availability tier holds down answers `false` and queues
  nothing — an idle frame, focus left wherever the unscoped walk put it,
  never a first-candidate reset (the M11 rule, at the focus seam). The
  refusals ride the `alacer::focus` target with the scope on the span
  and their help as plain diagnostic text — the miette-class `help:`
  sentence, never the miette crate (ADR-0029's rule, which the m24 exit
  suite pins at the manifest).
- **The demonstration walks the trap.** `examples/focus_trap.rs` mounts
  a background control and a modal's form (two inputs, a checkbox, a
  send button), walks the form inside its subtree forward, backward, and
  into the honest refusal while the availability tier holds every
  control in the trap down, then hands the form back — the background
  control never becoming a stop, the unscoped Tab staying the window's
  door, tracing at `alacer::focus_trap`. The m25 exit suite pins the
  delta headlessly: the scoped walk stays in the trap, skips the
  unavailable control, agrees with the unscoped door at the root, costs
  an idle frame on refusal and exactly one frame on a move, and the
  read doors keep answering the whole tree.
- **The micro-debt the sweep priced rides the cut.** The layout
  mapping's dead `fixed_size_of` goes — zero call sites anywhere in the
  workspace, the M24 species' remainder (the accessor that looked like
  surface). The color's dead `to_f32_rgba` goes with it — documented
  for a GPU-uniform upload the wgpu backend never made (the uniform
  path takes the premultiplied u8 route), consumed only by its own
  test. No weak errors remained for the ADR-0025 species to price this
  cut: the M22/M23 diagnostics closed the macro's remainder, and the
  scoped door's refusals are answers, not errors.
- **The living documents re-agree at M25 and 0.16.0.** The README's
  badge and status block ride the pending-cut convention the m7 suite
  itself documents (the successor minor under `[Unreleased]` — the
  0.15.0 release chore left the badge at 0.14.0, incoherent with both
  the manifest and the successor), the status count (M0 through M25),
  the ADR count (thirty-two), the features bullet, the quickstart, and
  the example table; the catalog's row; the migration guides' family
  line (both said 0.14.0 — the same release's remainder); the register
  (31 → 32 with this ADR); the roadmap row; and the m-suite pins, which
  move with the documents in the same cut — including the m7
  release-state pin, whose M22-era literal missed the M23 and M24
  sweeps and made the suite red against a coherent tree: the chain
  cannot drift, and now m7 cannot either.

## Consequences

- The scope is a read-side vocabulary over existing state: no new pass,
  no new mask value, no scheduler change — the scoped door reuses
  `cycle_focus`'s walk through one generalized private method, and the
  ancestor filter costs one parent-chain walk per candidate, on the
  same order as the cycle-time `enabled` check already on the path. The
  hot path carries no new work; the unscoped door's call sites swap
  nothing.
- The corpus does not grow: the demonstration shows no new visual
  arrangement (the catalog's own rule for when an example needs a
  scenario — the trap's paint is the columns and controls the corpus
  already blesses), and the m1 suite's 37-scenario pin stands.
- The perf contracts do not grow either: the focus-move route
  (`input/route_100_focus_moves_1k`) ADR-0030 priced as the
  measurement-run candidate is joined by the scoped route
  (`input/route_100_scoped_moves_1k`), and both join the published
  corpus at one measurement run by the coordinated rule — measured,
  published, and gated at that run (the checker fails an uncontracted
  benchmark; CONTRIBUTING's publishing rule keeps the half-published
  bench out of the tree).
- No new dependency edges: the seam's spans ride the `tracing` core
  every member already resolves, and the help-text discipline stays
  plain diagnostic text (ADR-0029's rule — the miette-class `help:` the
  species ships, never the miette crate).
- The m-suite's document pins now move across six suites (m7 joins
  m20–m24); the next release moves all of them together or the m25
  suite fails loudly.

## Kill criteria (re-open rules)

- A scope needs a shape the subtree root cannot express (a predicate
  over subtrees, a trap with holes, several disjoint traps) → grow the
  scope vocabulary beside `focus_move_in`, priced by a real UI — never
  a second walk or a client-side reimplementation of the candidate
  list.
- A door needs to observe the walk's refusals as data (a status line
  that reports *why* nothing moved inside the trap) → grow the answer
  into a result vocabulary priced by that consumer; until one exists,
  the boolean is the honest shape (ADR-0030's rule stands).
- A keyboard route needs to ride the trap (a modal that holds Tab
  itself) → route the fallback through the scoped door with the trap
  the app declares — priced by a real UI, never a second dispatch
  channel.
