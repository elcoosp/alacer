# ADR-0014: M8 scope — frame-batched input, the coalescer joins the scheduling contract

- **Status:** Accepted (2026-11-08)
- **Implements:** the §10.2 contract as wired architecture: "input storms
  cost one frame of work, not N events" — delivered through the pipeline
  queue, not the platform loop
- **Companion:** ADR-2 (the scheduling contract this extends to the input
  half), ADR-6 (the one-write-batch effect contract whose currency input
  now shares), architecture §10.1/§10.2 (routing and coalescing), §14.4
  (the degradation ladder's "coalesce remaining flux work" rung)

## Decision

M7 closed the v1 definition of done. The first post-v1 architectural item
is not a new widget or a new pass — it is the one place where the workspace
still routes *around* its own scheduling contract. The §10.2 coalescer
existed and was unit-tested, but the platform drained it at the end of
every winit callback: with one event per callback, it structurally never
coalesced in the live loop. The property was tested and simultaneously
inert — docs drift with a latent wiring bug. The same seam carried two
smaller defects: a redundant double `take_pending_masks` whose net effect
is `needs_present |= handled` (dead branch in disguise), and a wheel
mapping that dropped the x axis on the floor.

M8 moves the coalescer into `alacer-core` (`input::EventCoalescer`) and
makes it a field of the `PassPipeline`: the platform *queues* pointer
events and never drains; `tick` drains the queue at its start — before the
ledger is taken, so the batch's event passes are the frame's own ledger
entries. Input now enters through the same one-frame currency as signal
writes (axiom A1): a storm of fifty moves costs one coalesced routing, one
event pass, one frame. Keyboard and IME stay direct dispatches (they are
focus-routed and low-frequency; §10.2 is a pointer discipline). The wheel
mapping carries both axes (`LineDelta` scaled at 40 px/line, `PixelDelta`
as-is), and the batching ratio is published as a criterion pair plus a
budget contract:

- `input/storm_50_moves_queued` — 50 events queued, one tick (the M8 shape):
  **7.32 µs** median;
- `input/storm_50_moves_direct` — 50 direct dispatches + one tick (the
  pre-M8 shape): **299.21 µs** median;
- the `input_storm_batching` budget pins direct ÷ queued ≥ 2.0× — measured
  40.9× on the published baseline's host class.

### 1. Why this is the milestone

The harness, the canon, the comparison, and the perf-contract regime are
built; the widget vocabulary grows one seam at a time. What cannot grow
*correctly* on top of the pre-M8 wiring is the next interaction tier: hover
states, drags, sliders, and scrollbar tracking all read pointer streams
whose per-event dispatch cost and per-event semantic visibility must be
bounded first. Batched input is also the cheapest honest repair of a
documented-but-undelivered contract — the exact class of drift this
workspace treats as a defect (§20's honesty rules).

### 2. The seams that did not move

- **The corpus grows by the coordinated rule, nothing else moves.** One
  bench file, two criterion ids, two regression contracts, one budget —
  43 → 45 cases; the M5/M6/M7 pin suites move together (the M8 suite
  asserts all three).
- **The pass contract is unchanged.** No new `PassKind`; the event pass
  stays dispatch-driven, its runs land in the same ledger, and the idle
  contract (A7) is untouched — a queue drained at tick start costs nothing
  when empty.
- **The platform stays thin (ADR-1).** `window_event` maps and pushes;
  `queue_pointer` sets `needs_present`, and the existing
  RedrawRequested → `render_frame` → `tick` path consumes the batch. The
  double-drain block is deleted, not refactored.

### 3. What this does not do

- No hover/enter-leave synthesis, no pointer capture, no drag state
  machine — the batched queue is their prerequisite, not their
  implementation.
- No horizontal scrolling at the widget level: `List` still consumes `.y`
  only; the event now *carries* `.x` so widgets can opt in.
- No change to `A11yAction` breadth, virtualization, or the animation
  ladder — each is a candidate M9 item, unblocked rather than delivered
  here.

## Consequences

- The workspace stays 0.7.0; the CHANGELOG documents M8 under Unreleased.
- The corpus is 45 cases; the M5/M6/M7/M8 suites pin the number together.
- The baselines are re-published from the same host class (fingerprint
  `ee37c3efc5995352`, dirty tree): the w1 budget re-measures at **14.76×**
  (gate ≥ 2.0×), the w3 reference row at 24.2 ms — the committed
  cross-framework comparison (`results.json`) is untouched, as ADR-0013's
  declared rig of record intends.
- `alacer-platform` loses its `coalesce` module and its `EventCoalescer`
  re-export; the type lives at `alacer_core::input::EventCoalescer`. The
  DAG check is unaffected (platform already depended on core).
- The `docs/architecture.md` §18 roadmap table gains the M6/M7 rows it was
  missing (the README claimed them; the table had drifted) and an M8 row.
