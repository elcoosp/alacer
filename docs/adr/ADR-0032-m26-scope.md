# ADR-0032: M26 scope — the directional focus walk: the walk reads the window (the spatial directions join the dispatch contract, the entry contract repaired, the corpus debt the coordinated rule left)

- **Status:** Accepted (2026-11-10)
- **Implements:** the re-open rule ADR-0030 pre-registered — "a walk
  needs an order the tree order cannot express (spatial/D-pad
  navigation) → grow the vocabulary as a second door priced by a real
  UI, never a second pipeline channel" — plus the surface work the
  M16–M25 pattern leaves open: the micro-debt the sweep priced, the
  entry-contract drift the M25 cut documented but its arithmetic did not
  honor, and the corpus debt the M24/M25 coordinated rule described and
  the tree never delivered
- **Companion:** ADR-0030 (the walk the directions aim; the re-open rule
  this ADR cashes), ADR-0031 (the scope that bounds the walk; the entry
  contract this cut repairs), ADR-0028 (the availability tier the
  cycle-time filter serves), ADR-0029 (the plain-help-text rule the
  refusals ride), ADR-0011 (the coordinated corpus rule the routes
  land under), architecture §10.3 (the keyboard contract the walk
  belongs to), §18 (the M26 roadmap row)

## Decision

Every dispatch seam since M19 grew its public vocabulary beside the
keyboard route it shares, and each milestone's re-open rules named the
next vocabulary a real UI would price. The directional walk is the one
the M24 cut named first — a window whose controls form a grid cannot
answer "next" with the tree order, because the row breaks the pre-order
list's stride exactly where the grid's geometry answers — and M26 cashes
that rule without re-inventing the walk:

- **The vocabulary grows inside `FocusMove`.** `Left`, `Right`, `Up`,
  and `Down` join `Next` and `Prev`; the two public doors keep their
  signatures (`focus_move(movement)`, `focus_move_in(scope, movement)`)
  and the walk generalizes behind them (`cycle_focus_in(scope,
  movement)` — the boolean the M25 walk took becomes the movement it
  aims). No new pass, no new mask value, no scheduler change, no second
  door: a directional move is a read-side selection over the same
  pre-order candidate list, the same cycle-time `enabled` filter (M22,
  ADR-0028), the same immediate resolution, and the same `FOCUS | PAINT`
  currency on the two endpoints. One funnel, six directions.
- **The geometry answers, and it is the cheapest honest geometry.** A
  candidate qualifies for a direction when its laid-out bounds lie fully
  beyond the focused node's edge in that direction — no overlap with the
  origin on the axis the move names (`bounds.min_x() >= origin.max_x()`
  for `Right`, and its mirror for the other three). Among the
  qualifiers, the nearest edge distance wins, the cross-axis center
  misalignment breaks ties, and tree order breaks the ties the geometry
  cannot — deterministic, and the same list Tab reads. The bounds are
  what pass 6 wrote: no layout re-run, no measurement, no second
  geometry source. Candidates without bounds (the first frame's
  pre-layout state) cannot answer a directional call and are skipped;
  a focused node without bounds refuses by name.
- **Space never wraps.** The tree-order moves keep their wrap — the wrap
  is the Tab contract and only the Tab contract. A directional move at
  the window's edge (or at the trap's, under a scope) has no candidate
  fully beyond the edge and answers `false`: no masks, an idle frame,
  focus left where it was, never a first-candidate reset (the M11 rule,
  at the focus seam). The refusals ride the `alacer::focus` target with
  their help as plain diagnostic text — the miette-class `help:`
  sentence, never the miette crate (ADR-0029's rule).
- **The scope and the tier filter before the geometry answers.** The
  scoped door bounds the directional walk exactly as it bounds the
  tree-order one: the candidates are the scope's own before any bound is
  read, so a direction with no candidate inside the trap refuses
  honestly instead of escaping the scope, and the read doors
  (`focusables()`, `focus()`) keep answering the whole tree. The M22
  availability filter runs on the same list before the geometry, so a
  control the app holds down is never a directional stop and the walk
  takes the next qualifying neighbor without a recompute.
- **The entry contract is repaired to what ADR-0031 documented.** From
  no focus, the walk rests on the first candidate — whichever direction
  the caller names, because the walk must rest somewhere before a
  direction can mean anything. From a focus outside the filtered
  candidate list — a scope bounding it, or the tier holding the focused
  control down since pass 2 — ADR-0031 has documented "the walk takes
  the trap's first candidate" since M25, but the M25-era positional
  arithmetic actually landed the list's *second* candidate forward and
  its *last* backward (`unwrap_or(0)` feeding a ±1 step). The document
  wins: both doors land the list's first candidate, both directions,
  from an out-of-list focus alike. The m26 exit suite pins the repaired
  contract, and the m25 suite's generalized-signature pins moved with
  the walk in the same cut.
- **The demonstration walks the window.** `examples/focus_direction.rs`
  mounts a two-by-two grid of buttons and a modal's form, walks the grid
  directionally (the top-right button's `Down` lands the bottom-right —
  the tree's next stop is the bottom-left, which is the seam's reason to
  exist), hits the corner's honest refusal while `Next` still wraps from
  the same spot, and walks the form directionally inside its scope — the
  background grid never a stop, tracing at `alacer::focus_direction`.
  The m26 exit suite pins the delta headlessly on a real two-by-two grid
  through the real layout host: the geometry's answer against the tree's
  next stop, the unavailable neighbor skipped, the corner refusal an
  idle frame while the wrap still lands, the scoped walk staying in the
  trap, the entry contract landing the first candidate from outside the
  list, and one move costing exactly one frame.
- **The micro-debt the sweep priced rides the cut.** The selection's dead
  `is_degenerate` goes — zero call sites anywhere in the workspace, the
  M24 species' remainder (the accessor that looked like surface). The
  color's dead `to_premultiplied_rgba8` goes with it — its only caller
  was its own unit test, the software stamp premultiplies independently
  (`alacer-render`'s coverage stamp), the same species the M25 cut named
  for `to_f32_rgba`: the conversion documented for a consumer that never
  landed. No weak errors remained for the ADR-0025 species to price this
  cut: the focus refusals are answers, not errors.
- **The corpus debt the coordinated rule left rides the cut.** The M24
  and M25 cuts described `input/route_100_focus_moves_1k` and
  `input/route_100_scoped_moves_1k` as having "joined the published
  corpus at the measurement run" — the tree never delivered them: no
  bench, no baseline, no gate, the corpus pinned at 57 either way. The
  drift is repaired the only honest way: the three routes (the two owed
  plus this cut's `input/route_100_directional_moves_1k`) ship in this
  cut's bench file with their regression gates in `contracts.json`, the
  corpus pins move 57 → 60 across the m5–m25 chain in the same commit,
  and the baselines land at this cut's measurement run —
  `cargo bench -p alacer-core --bench input`, then
  `python3 scripts/collect_baseline.py`, then
  `python3 scripts/perf_dashboard.py` — the one step the coordinated
  rule requires before the corpus's own pins agree with the published
  file. Until that run, the pins are the alarm the M5 discipline designs
  them to be: red against a tree whose bench and gates are already in
  place, naming the exact commands in the failure message.
- **The living documents re-agree at M26 and 0.16.0.** The README's
  badge and status block ride the pending-cut convention the m7 suite
  itself documents (the successor minor under `[Unreleased]`), the
  status count (M0 through M26), the ADR count (thirty-three), the
  features bullet, the quickstart, the perf bullet (the corpus count and
  the focus routes' debt named), and the example table; the catalog's
  row; the register (32 → 33 with this ADR); the §10.3 focus paragraph;
  the roadmap row; the facade's shipped-surface rustdoc (which had not
  named the focus seam at all — the M24/M25 remainder); and the m-suite
  pins, which move across the m5–m25 chain in the same cut — the corpus
  pins to 60, the document pins to M26 and thirty-three, and the m25
  suite's walk-signature pins to the generalized form.

## Consequences

- The directional selection is a read-side vocabulary over existing
  state: no new pass, no new mask value, no scheduler change — the walk
  reuses `cycle_focus`'s funnel through one generalized private method,
  and the directional branch costs one bounds read per candidate on the
  same order as the cycle-time `enabled` check already on the path. The
  hot path carries no new work; the tree-order doors swap nothing.
- The keyboard's Tab/Shift+Tab fallback is untouched: the fallback maps
  Shift to `Prev` exactly as before, and no platform key routes to the
  directional moves yet — they are the app's vocabulary (a D-pad, a
  TV remote, a consumer's own arrow-key scheme), priced by the example
  and available the way `focus_move` has been since M24.
- The m1 corpus does not grow: the demonstration shows no new visual
  arrangement (the catalog's own rule for when an example needs a
  scenario — the grid's paint is the rows and buttons the corpus already
  blesses), and the m1 suite's 37-scenario pin stands.
- The perf corpus grows 57 → 60 at this cut's measurement run by the
  coordinated rule — measured, published, and gated at that run (the
  checker fails an uncontracted benchmark; CONTRIBUTING's publishing
  rule keeps the half-published bench out of the tree; the pins are the
  alarm until the baselines land).
- No new dependency edges: the seam's spans ride the `tracing` core
  every member already resolves, and the help-text discipline stays
  plain diagnostic text (ADR-0029's rule — the miette-class `help:` the
  species ships, never the miette crate).
- The m-suite's document pins now move across the m19–m25 suites and the
  corpus pins across the m5–m25 chain in one commit; the next release
  moves all of them together or the m26 suite fails loudly.

## Kill criteria (re-open rules)

- A walk needs an order neither the tree nor the edge-distance heuristic
  can express (a true 2-D grid with occlusion, warping, or per-row
  origin memory) → grow the selection vocabulary beside
  `directional_neighbor`, priced by a real UI — never a second walk or
  a client-side reimplementation of the candidate list.
- A door needs to observe the walk's refusals as data (a status line
  that reports *why* nothing moved — no candidate, no geometry, a dead
  scope) → grow the answer into a result vocabulary priced by that
  consumer; until one exists, the boolean is the honest shape
  (ADR-0031's rule stands).
- A platform key needs to ride a direction (a D-pad button, a TV
  remote's ring) → map the key onto `FocusMove` at the platform's
  key-fallback seam, priced by a real backend — never a second dispatch
  channel.
