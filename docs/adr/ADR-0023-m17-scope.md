# ADR-0023: M17 scope — the demonstrated queue (the coalescer's consumer, the docs that re-agree with the data, the chains completed)

- **Status:** Accepted (2026-11-09)
- **Implements:** the continuation of ADR-0022's consumable-surface work:
  the one core vocabulary with a published budget and no runnable
  demonstration (the M8 coalescer), the living documents that stopped
  agreeing with the checked-in data, and the two error seams the M16
  boundary work did not reach
- **Companion:** ADR-0022 (the consumable contract this continues),
  ADR-0014 (the coalescing contract this demonstrates), ADR-0021 (the
  journal whose demonstration set the example's shape), ADR-9 (the
  dependency discipline that keeps miette out), architecture §10.2 (the
  coalescing contract), §13.4 (the DX contract), §18 (the M17 roadmap row)

## Decision

M16 made the surface consumable; its ledger (announcements/live regions,
the pointer-capture seam, `supports_*`, a `Slider`) names feature work
that each needs a full corpus cycle — a bench case, a contract, a
baseline re-published from the collecting host. None of it is unblocked
by a deferral a previous ADR wrote down, so M17 is the second
consecutive cut that makes the shipped surface agree with itself before
growing it, in the same "one contract widens" shape — this time the
contract being the *demonstration* contract:

- **The M8 coalescer gets its consumer demonstration.**
  `examples/input_storm.rs` drives the frame-batched input contract the
  way `undo_redo.rs` drives the journal: a `StormPanel` probe (a custom
  `Widget` — `role()` and `label()` required, exactly what a consumer
  writes) records every routed pointer event, the *Queue storm* button
  queues fifty synthetic moves through the pipeline's own queue — the
  seam the platform layer pushes into and never drains (ADR-0014) — and
  the *Read the log* route reads the panel's log one frame later, the
  batching latency made visible. The observability rides the workspace's
  idiom: `tracing` spans at the `alacer::input` target with the pushed
  and routed counts as fields. A blessed snapshot pins the demonstration
  (the m1 corpus grows 33 → 34 PNGs; the scenarios grow 31 → 32).
- **The living documents re-agree with the checked-in data.** The README
  still quoted the M9-era headline ratios — 14.76× and 40.9× — while the
  baselines the M13–M15 corpus re-publishes moved them to **14.06×**
  (`w1/full_rebuild / w1/incremental_churn_10pct`) and **40.7×**
  (`input/storm_50_moves_direct / queued`); the architecture's
  zero-fabrication policy applies to the workspace's own README. The
  migration guides self-described as mapping onto "Alacer **0.5**" three
  minors late, and the facade's rustdoc still framed the crate as "the
  M0 dependency spine" while it ships the editing contract, the canons,
  and ten examples — the same class of self-description drift ADR-0022
  fixed in the widgets crate.
- **The error-chain discipline reaches the two seams M16 did not.**
  `Recording::save`/`Recording::load` formatted their `serde_json`
  causes into `String` payloads — a malformed `.alc` (a *bug report*, by
  ADR-10's own definition) dead-ended at a prose string instead of a
  walkable `source()` chain; the cause is now handed to
  `io::Error::other` directly and a unit test walks it. And
  `LayoutError` — the public `LayoutEngine` contract's error type, never
  constructed anywhere in the workspace — documented that "widget ids
  [are] included" while offering no field to carry them: the failing
  node is now data (`node: Option<NodeId>`), not prose, so an engine
  that does fail reports the id a consumer can read.

### 1. Why this is the milestone

The ledger's feature candidates each cost a full corpus cycle and none
is unblocked by a written deferral, but the surface they would land in
had accumulated the same three debts M16 fixed one layer over: a public
vocabulary without a demonstration, documents that lagged their own
data, and an error type that loses its context. The coalescer is the
last core seam with a published budget and no example — the examples
catalog's own rule is that an example arrives with its scenario, and
text, clipboard, and history all have theirs. The README's ratios are
the first numbers a visitor reads, and they were two baseline
re-publishes stale. The cheap cut is the demonstrated one: a storm a
consumer can run, documents that match the checked-in baselines, and
errors a consumer can walk — so the feature milestones land on a
surface that no longer needs a catch-up pass first.

### 2. The seams that did not move

- **No new bench case; the corpus stays 53.** The storm's hot path is
  already contracted (`input/storm_50_moves_direct`,
  `input/storm_50_moves_queued` — the 40.7× budget); the example and
  the scenario consume the same queue the m8 exit suite pins, and a
  demonstration needs no second measurement.
- **The DAG is untouched in shape.** The example consumes the facade;
  the error work changes type bodies (`LayoutError` gains a field,
  `Recording::load` changes a `map_err` body), not signatures' shapes or
  dependency edges. `tracing` rides the facade's *dev*-dependency (the
  ADR-0022 precedent), and `alacer-testing` gains no dependency at all —
  its `io::Error` chain *is* the diagnostic. Nothing published moves
  beyond the documented `LayoutError` field.
- **No miette adoption.** The ADR-9 discipline stands: the help text is
  hand-rolled — the schema-mismatch message names both schemas (the
  honest leaf), the layout error's `Display` names the node it carries,
  and the example's doc comment names the escape hatch
  (`ALACER_SOFTWARE=1`).
- **The editing contract's rules are untouched.** The recording replay's
  panic-on-drift semantics are by design — a recording that no longer
  describes reality is a failing regression test (ADR-10) — and stay;
  only the *load* path's error chain moves. The coalescer's rules
  (moves merge, wheel accumulates, a flip flushes) keep their pinned
  behavior; the example is a consumer, not a second definition.

### 3. What this does not do

- **No ledger feature work.** No live-region, pointer-capture,
  `supports_*`, or `Slider` vocabulary — each rides M18+ with its own
  corpus cycle.
- **No baseline re-publish.** The numbers the README now quotes are the
  ones already checked in `benchmarks/baselines/baselines.json`; M17
  changes no measurement.
- **No crates.io publish.** The family stays publish-*able*, not
  published (the architecture's `UNVERIFIED` marker stands).

## Consequences

- Every core vocabulary with a published budget now has a runnable
  consumer demonstration; the examples catalog grows one row and one
  blessed scenario (31 → 32 scenarios, 33 → 34 PNGs).
- A visitor who reads the README, a consumer who reads the migration
  guides, and `cargo doc` all quote the checked-in baseline's numbers —
  the drift class gets a roadmap row of its own to regress against.
- A malformed `.alc` reports its parse cause as a walkable chain, and a
  layout failure (when an engine produces one) carries its node as
  data.
- The next milestone starts from a surface whose demonstrations,
  numbers, and error chains agree with its own API: the ledger's
  feature work lands without a catch-up pass in front of it.
