# ADR-0055: M49 scope — the funnel's wall is a named fact

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0053's second kill rule — "a consumer needs
  *which* bound the value pinned when a write was refused at the
  range's edge → grow the refusal's payload beside the species (the
  ask's own wall), never a second action species"
- **Companion:** ADR-0054 (the gesture record's bound the value record
  speaks beside), ADR-0050 (the ask the wall is named against),
  ADR-0049/ADR-0051 (the stamp discipline the wall rides beside),
  ADR-0046 (the value trail whose record grows), ADR-0053 (the kill
  rule this cashes), architecture §13.3, §18 (the M49 roadmap row)

## Decision

- **The wall the ask was turned back by is the write's own fact.**
  `alacer-core/src/inspect.rs`: `ValueRecord` grows `pinned:
  Option<RangeBound>` — the range's edge the funnel pinned the landing
  at (the clamp, the snap, the wall that held a no-op against a past
  ask), beside the action and outcome species — the kill rule's own
  shape: the refusal's payload grown beside the species, never a
  second action species (`ClampToMax` would munge the range's answer
  into an action the AT never asked; the M11 rule's own words at one
  remove). The tags ride ADR-0054's `RangeBound` vocabulary — one
  named-edges concept, spoken identically on the gesture log and the
  value log.
- **The absence is honest in both directions.** A landing mid-range
  and every refusal carry `None` — a write that moved nothing pinned
  nothing. And the ask that wanted the wall exactly (the M47 jumps)
  carries `None` too: the species already names the bound it asked
  for, and a pin would claim the funnel moved an ask it never moved.
- **The door computes it where the knowledge lives.**
  `alacer-widgets/src/slider.rs`: one `pinned_bound(ask)` helper — the
  landed value at an edge *and* the ask somewhere else, the widget's
  own `total_cmp` equality answering both — riding the delivery note
  sites (the AT tier's `SetValue`/`Increment`/`Decrement`, the arrows'
  `step_note`) and answering `None` for the jumps' ask-by-construction
  (`jump_note` carries the computed answer, so the door stays honest
  by arithmetic, not by convention). The read-only `Progress` carries
  the absence at its `no_write_door` refusal — a door-less control
  never pinned a wall.
- **The wire and the client speak it.** `alacer-inspect/src/wire.rs`:
  the `value` event carries the additive `pinned` field; the embedded
  client renders a pinned write as "asked 150, landed 100, pinned at
  the max" beside the M44 clamp span — the wall readable from the log
  pane alone.
- **`examples/pinned_writes.rs` is the consumer demonstration** (tracing
  at `alacer::pinned_writes`): the real AT route and the real key
  route, the asks past both walls, the wall that held, the jump's
  honest absence, the step the wall held, and the two refusals that
  pinned nothing.
- **No new corpus case: the corpus stands at 64.** The routes the wall
  rides are already measured (the value tier's AT and key routes
  under `input/` and the trail benches); the pin is one `Option`
  computed once per answered write, inside the budgets the contracts
  already gate. The living documents re-agree at the cut (the README,
  the register, the protocol document's value vocabulary, the catalog,
  the CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer needs *how far* the ask overshot the wall it pinned → the
  pinned payload grows the overshoot beside the species (the ask and
  the landing already ride the record; the delta is derivable, so the
  growth waits for a consumer that cannot derive it).
- A consumer needs the wall named on a tier whose funnel knows no
  range (the text field's clamps, the list's scroll clamp) → that
  tier's own record grows its own fact beside its species, never the
  value record's (the wall is the asker's story — ADR-0054's rule at
  the next tier).
