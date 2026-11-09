# ADR-0027: M21 scope — the reported value (the progress indicator, the mutate seam's demonstration, the micro-debt the audit priced)

- **Status:** Accepted (2026-11-10)
- **Implements:** the value tier's read half — the candidate the
  audit-before-the-cut pass priced after M20 emptied the ledger — plus
  the surface work the M16–M20 pattern leaves open: the mutate seam's
  missing consumer demonstration, the corpus-count pins that drift one
  cycle after every bench lands, and the living documents whose
  self-descriptions lag the shipped family (the migration guides' 0.12.0
  line among them)
- **Companion:** ADR-0026 (the value tier's write half — the slider this
  read half mirrors), ADR-0017 (the value tier the report joins as a
  second consumer), ADR-0011 (the coordinated rule this cut completes),
  ADR-6 (the scheduling contract the app write rides), ADR-5 (the
  required role and label the report obeys), ADR-9 (the dependency
  discipline that keeps the read-only arithmetic hand-rolled),
  architecture §11.5 (the value tier this widens), §18 (the M21 roadmap
  row)

## Decision

M20 closed the ledger with the value tier writable and its read half
unclaimed. M21 takes the reported value — a bounded scalar the *app*
writes and the AT reads — and delivers it as a widget:

- **The progress indicator joins the widget vocabulary**
  (`alacer-widgets/src/progress.rs`, `Role::ProgressIndicator` in core).
  The rules:
  - **Two app doors, one currency.** The bound door rides the flux
    bridge the reactive text rides: one write batch, the caller's masks
    (`PAINT | A11Y`), the widget painting and reporting from the
    binding. The imperative door rides the mutate seam the pass-4 queue
    exists for: the app takes the arena lease, writes in place, and
    requests the write's own masks on the guard — and `set_value`
    answers the move it made (`bool`), so a write that lands where the
    value already is queues nothing. The idle-frame property the storm
    budget leans on, at the app seam.
  - **The AT tier is read-only.** Every action — `SetValue`,
    `Increment`, `Decrement`, `DefaultPress` — is refused honestly
    (`false`, no masks): a reporting widget has no write door, and a
    refusal is never munged into a step or a press (the M11 rule). The
    tree keeps the number the app wrote.
  - **The report rides the M3 contract.** `Role::ProgressIndicator`
    joins `requires_name` (an unnamed report reads as a bare number —
    ADR-5) and stays out of the focusable set (a report holds no
    focus); the widget reports through `value()`, so the a11y tree
    carries the number. The a11y sink maps the role faithfully
    (`AccessKitRole::ProgressIndicator`).
  - **The report is the app's voice.** A bound bar reads the binding
    live — the `Label::bound` rule, no widget-side rewrite — while the
    paint clamps into the range (the bar is a bounded control's body);
    the fixed door's writes clamp into the range and refuse non-finite
    requests.
- **The consumer demonstration ships.** `examples/progress.rs` drives
  both doors the way `slider.rs` drives the funnel: the *Step the batch*
  button writes one signal per click (the bridge door), the demo's
  startup write walks the mutate seam (the lease, the write, the masks
  on the guard), tracing spans at the new `alacer::progress` target name
  each route, and the status line mirrors what the a11y tree carries.
  The mutate seam — pass 4's queue and the `WidgetMut` mask-on-drop —
  gets its first consumer demonstration; until now only the pipeline's
  own tests walked it.
- **The corpus grows 55 → 56 by the coordinated rule.** The value-write
  route's criterion case `progress/route_100_value_writes_1k` — one
  hundred app-driven write batches through the bound bridge on the
  1,000-node storm arena — is measured (`cargo bench`, 100 samples),
  published with its baseline entry, and gated same-host in
  `contracts.json`; the checker fails an uncontracted benchmark, so the
  bench, the baseline, and the gate land together (ADR-0011,
  CONTRIBUTING's publishing rule). Every M-suite pin moves to 56 in the
  same cut, including the m20 suite's own — the one-cycle drift the
  coordinated rule prices.
- **The m1 corpus grows 35 → 36 scenarios.** The reported value's window
  joins the styled corpus — the pixels are the regression net behind the
  demonstration.
- **The micro-debt the audit priced rides the cut.** Two dead
  introspection surfaces go: `PassPipeline::has_observer` (a predicate
  on an `Option` with zero call sites — the M18 species) and the wgpu
  backend's dead stats surface (`glyphs_rendered`,
  `instance_frames_skipped`, `atlas_clears`, and the atlas `clears`
  counter under them — counters nobody reads; the inspector consumes
  core's `FrameSummary`, not the backend's private tallies). No error
  leaf needed work: every `Result` in the workspace already carries its
  cause.
- **The living documents re-agree.** The migration guides still scoped
  themselves "onto Alacer **0.12.0**" one release after the family moved
  (the M16 pattern's remainder, again); the README's criterion count,
  vocabulary, and status block, the catalog, the architecture's register
  (27 → 28 with this record) and roadmap, and the CHANGELOG move
  together.

### 1. Why this is the milestone

The value tier's read half is the smallest honest completion of the
ledger M20 emptied: the AT can write a value, but an app had no
widget-shaped way to *report* one — long work either borrowed a label or
reached for the editor's write vocabulary. The seam gap the audit found
is real: pass 9 runs only on the `A11Y` mask, so an app-driven value
write must carry the currency or the tree goes stale, and the mutate
seam that carries it had no consumer demonstration at all. The contract
is deliberately small: no new event variants, no platform work, no
engine work, no new dependency edges — one role, one widget, two doors
the pipeline already routes, and the micro-debt the same
audit-before-the-cut pass priced.

### 2. The seams that did not move

- **No platform, engine, or text work.** The read-only arithmetic is
  hand-rolled (`clamp`, one `total_cmp` equality — the float discipline
  the workspace already holds); the platform keeps pushing raw events;
  the text seam renders the name it already renders for every widget.
- **The DAG is untouched in shape.** `Role::ProgressIndicator` lives in
  `alacer-core` beside its siblings; the widget lives in
  `alacer-widgets`; `alacer-a11y` gains one match arm and its unit
  test; no published signature changes; `scripts/check_dag.py` sees no
  new edges.
- **No miette adoption.** The refusals are return values the pipeline
  already routes; the help the AT needs is the tree's own value, not a
  diagnostic (ADR-0026's rule, held). The workspace's error convention —
  structured causes with escape-hatch help text at the seams that know
  one — is untouched; this cut found no error leaf that needed work.
- **The editing, announcement, and slider contracts are untouched.** The
  editor's value tier (ADR-0017) and the slider's funnel (ADR-0026) are
  unchanged — the report joins them as the read half, never a dialect.

### 3. What this does not do

- **No editable or animated progress.** A report with a write door is
  the editor's or the slider's contract; an animation is a signal
  cadence the app owns, not a widget feature.
- **No registry element.** The bar is app-driven (the bridge or the
  mutate seam), not prop-driven — the slider's precedent: the
  demonstration is widget-direct, and a `progress` rsx element is
  priced only when a real view needs one.
- **No baseline refresh beyond the coordinated rule.** The fifty-five
  published medians are the 2026-11-09 run untouched; the new case's
  entry is measured on the cut's own host and disclosed in the PR body
  (the fingerprint matches the published host class, so the same-host
  gate compares honestly).

## Consequences

- An app can report bounded state — a batch, an upload, a scan — in the
  same tree vocabulary the AT already reads, with the same idle-frame
  economy every other write holds.
- The mutate seam (pass 4's queue, the `WidgetMut` mask-on-drop) has a
  consumer demonstration and a tracing target, so the imperative door is
  documented by example instead of by test alone.
- The m1 corpus grows one snapshot; the examples catalog and the
  README's example table grow one row each (`progress`).
- The M-suite pins, the README's count, the catalog's counts, the
  migration guides' counts, and the architecture's register agree with
  the checked-in state at 56 cases and 28 records.
- The ledger is empty again: the next feature milestone needs a new
  candidate, priced by the same audit-before-the-cut pass.
