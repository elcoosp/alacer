# ADR-0026: M20 scope — the valued widget (the slider, the value tier, the capture case's measurement run)

- **Status:** Accepted (2026-11-10)
- **Implements:** the ledger's last feature candidate (a `Slider`) plus
  the coordinated-corpus remainder M19 left at the measurement run (the
  capture route's criterion case) and the surface work the M16–M19
  pattern leaves open: the corpus-count pins that drift one cycle after
  every bench lands, and the vocabulary self-descriptions that lag the
  shipped widgets
- **Companion:** ADR-0016 (the action tier the value tier answers
  through), ADR-0017 (the value tier the slider joins — `SetValue` was
  an editor contract, now a widget one), ADR-0025 (the pointer capture
  the thumb's drag rides), ADR-0011 (the coordinated rule this cut
  completes for M19's promise), ADR-5 (the required role and label the
  widget obeys), ADR-9 (the dependency discipline that keeps the value
  arithmetic hand-rolled), architecture §11.5 (the action tier this
  widens), §18 (the M20 roadmap row)

## Decision

M19 closed the capture ledger with one candidate remaining and one
measured promise outstanding. M20 takes the `Slider` — the value tier's
widget — and delivers the capture case the same cut:

- **The slider joins the widget vocabulary**
  (`alacer-widgets/src/slider.rs`, `Role::Slider` in core). The rules:
  - **One funnel, four doors.** A drag writes the value from the
    pointer's x against the node's resolved frame (the ADR-0018 seam
    the capture route keeps honoring), the arrow keys step by the step,
    Home/End jump to the ends, and the AT's tier — `SetValue` (parsed,
    then snapped like every write), `Increment`, `Decrement` — answers
    through the same `on_a11y_action` route the editor and the scroll
    area already hold. Every door clamps to the range and snaps to the
    step inside the widget; the app never re-implements the arithmetic.
  - **The masks are the only currency (A1).** A value write requests
    `PAINT | A11Y` — the thumb repaints, the tree re-announces — and a
    write that lands where the value already is queues no masks: the
    idle-frame property the storm budget leans on, at the widget tier.
    A non-finite or non-numeric request is refused honestly (`false`,
    no masks, an idle frame) — never munged into a step, the M11 rule.
  - **The value rides the M3 contract.** `Role::Slider` joins
    `requires_name` (an unnamed value control reads as a bare number —
    ADR-5) and `focusable`; the widget reports its value through
    `value()`, so the a11y tree carries the number the AT reads and
    writes. The a11y sink maps the role faithfully
    (`AccessKitRole::Slider`).
  - **The handlers write signals, nothing else.** `on_change` receives
    the new value; the demo writes one signal through it. No widget
    mutation from handlers, no second channel — the announcement seam's
    discipline, at a control.
- **The consumer demonstration ships.** `examples/slider.rs` drives the
  widget the way `announcements.rs` drives the speech acts: one slider,
  one bound status line mirroring the value the tree carries — drag it,
  focus it and press the arrows, or let a screen reader write it. The
  contract is pinned by the widget's unit tests (the funnel, the
  refusals, the no-op write) and the new m20 exit suite, headlessly.
- **The corpus grows 54 → 55 by the coordinated rule.** The
  M19-promised criterion case `input/route_100_captured_moves_1k` — the
  capture route's cost per move over the storm arena, named in
  ADR-0025's own exit row — is measured (`cargo bench`), published with
  its baseline entry, and gated same-host in `contracts.json`; the
  checker fails an uncontracted benchmark, so the bench, the baseline,
  and the gate land together (ADR-0011, CONTRIBUTING's publishing
  rule). Every M-suite pin moves to 55 in the same commit.
- **The m1 corpus grows 33 → 35 scenarios.** The `drag_capture`
  example's promised blessed scenario (ADR-0025's other coordinated
  remainder) and the slider's own window join the styled corpus — the
  pixels are the regression net behind the demonstrations.
- **The living documents re-agree.** The README's badge and status
  block still said 0.12.0 one version bump after the manifest moved
  (the M16 pattern's remainder); the vocabulary list still omitted the
  canon widgets it ships; the criterion count, the examples catalog,
  the migration guides' snapshot counts, the architecture's ADR count
  (26 → 27 with this register's row), and the CHANGELOG move together.

### 1. Why this is the milestone

The slider is the last unclaimed ledger candidate, and the capture made
it honest: a thumb that follows the hand is only correct once the drag
keeps routing (ADR-0025's own "why"). The contract is deliberately
small: no new event variants, no platform work, no engine work — one
role, one widget, one funnel, and the value-tier mapping the a11y sink
already had the vocabulary for. The surface debts it rides are the
M16–M19 pattern's remainder, found by the same audit-before-the-cut
pass.

### 2. The seams that did not move

- **No platform, engine, or text work.** The value arithmetic is
  hand-rolled (`clamp`, `round`, one `total_cmp` equality — the float
  discipline the workspace already holds); the platform keeps pushing
  raw events; the text seam renders the name it already renders for
  every widget.
- **The DAG is untouched in shape.** `Role::Slider` lives in
  `alacer-core` beside its siblings; the widget lives in
  `alacer-widgets`; `alacer-a11y` gains one match arm and its unit
  test; no published signature changes.
- **No miette adoption.** The refusals are return values the pipeline
  already routes; the help the AT needs is the tree's own value, not a
  diagnostic.
- **The editing and announcement contracts are untouched.** The
  editor's value tier (ADR-0017) is unchanged — the slider joins it as
  a second consumer, never a dialect.

### 3. What this does not do

- **No range or multi-thumb slider, no `onChange` signal binding.** One
  value, one handler — the vocabulary a bounded scalar needs; a range
  pair re-opens the seam with the second thumb priced by a real UI.
- **No keyboard page-step.** The arrows step by `step`; a page-sized
  step (the scroll area's `Increment` unit) is the scroll area's
  contract, not a slider's.
- **No baseline refresh beyond the coordinated rule.** The two
  µs-scale entries whose absolute medians moved on the measuring host
  (`arena_masks/10000`, `flux_batch/1` — a fingerprint collision, not a
  regression; criterion's own previous-run comparison shows no change)
  are re-blessed from this session's full run and disclosed in the PR
  body; every other entry stays as published.

## Consequences

- A screen reader can set and step a slider like a drag can: the value
  tier's vocabulary finally has a widget-shaped consumer, and the tree
  carries the number it writes.
- The capture case's cost is published (the per-move ancestor walk
  over the storm arena), so the M19 contract's price is a number the
  budgets can lean on, not a claim.
- The m1 corpus grows two snapshots; the examples catalog and the
  README's example table grow one row each (`slider`).
- The M-suite pins, the README's count, the migration guides' snapshot
  counts, and the architecture's register agree with the checked-in
  state at 55 cases and 27 records.
- The ledger of feature candidates is empty again: the next feature
  milestone needs a new candidate, priced by the same
  audit-before-the-cut pass.
