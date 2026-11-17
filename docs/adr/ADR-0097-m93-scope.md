# ADR-0097 — M93 scope: the layout seam's call ledger

Milestone: **M93** (this cut). Parent contract: ADR-0088's re-open
rules standing beside the harness's own charter (A4 — the harness is a
shipped product) and the `measure_calls()` counter the seam has priced
itself with since M1: one number, the whole call story.

## The ask

The layout seam ([`LayoutEngine`], ADR-9's injected stage) answers one
question today: *how many* leaf measures ran since the last reset —
the translation-only probe every scroll milestone priced itself by.
Every other fact of the call — which root, how wide the dirty set, the
viewport the clamp priced against, whether the sync refused — is
invisible: the m-suite asserts call *counts*, and a consumer debugging
their own injected engine (the seam is public on purpose) has no
instrument at all. The alloc ledger's own precedent (A7, M84 — the
instrument the test/bench/example binaries install, never the
framework) names the shape: a recording engine beside the production
one, the ledger the test reads.

## The shape

- **The recording engine** (`alacer_layout::recording::RecordingLayout`):
  a delegating [`LayoutEngine`] — every call rides the wrapped engine
  untouched, and one record lands per call beside it:
  [`LayoutCallRecord`] carries the root asked, the dirty set's size,
  the viewport, and the outcome (delivered or the error's own message
  — the error context rides the record, never a bare `false`).
- **The ledger reader**: `calls()` (the window, drained by the test),
  `clear()`, and the seam's own counters delegated (`measure_calls`,
  `reset_measure_calls`) — the probe the m-suite already asserts
  through keeps answering through the wrapper.
- **The harness door** (`TestHarness::with_layout`): the constructor
  beside `new` that installs a caller-supplied engine — the trail
  door's own shape (`trail`/`trail_with_cap`), so the m-suite drives
  the instrument through the same harness every other suite uses (A4:
  the real pipeline, the real widget set).
- **The wire-up**: exported from `alacer-layout`'s root; the facade
  re-exports the crate whole (`alacer::layout::recording::…`), so a
  consumer's own assembly site installs it exactly where
  `pipeline_default` installs the production host. The instrument is
  *installed by the binary that wants it* — never by the framework
  (the A7 rule the alloc ledger shipped by).
- **The honest-wrapper budget** (the coordinated rule's bench half):
  the wrapper against the bare engine on the same incremental layout
  route — priced as a ratio budget (`recording_wrapper_overhead`, ≤
  1.15×), the wrapper's own cost the contract, the layout work
  dominating the measurement the way the route tier's routes do. The
  m-suite pins re-agree at 101 (M92's pair beside M93's, one cut).

## Exit criteria

- The ledger prices the call story: a dirty set driven through the
  real pipeline lands one record per call — root, dirty size,
  viewport, outcome — read back through `calls()` and cleared by
  `clear()` (the m93 exit suite, through `TestHarness::with_layout`).
- The counters delegate: `measure_calls` through the wrapper equals
  the wrapped engine's own answer (the translation-only probe keeps
  its meaning under the instrument).
- The refusals ride the record: a sync the wrapped engine refuses
  lands as the outcome's named message — the error context the seam's
  `Result` contract ships, carried into the ledger.
- The wrapper is honest: `recording/bare_wrapper_1k_dirty_10pct` vs
  `recording/wrapper_1k_dirty_10pct` — the ratio budget holding the
  instrument's cost at the route's noise floor.
- The corpus grows 97 → 101 by the coordinated rule (M92's pair beside
  M93's, one cut): `recording/wrapper_1k_dirty_10pct` and
  `recording/bare_wrapper_1k_dirty_10pct`, the ratio budget and the
  same-host gates joining the contract set, the m-suite pins
  re-agreeing at 101.
- `examples/layout_probe.rs` drives the beats from the consumer side —
  the LayoutEngine seam's first consumer example (the a11y sink's and
  the canvas's own precedent).

## Re-open rules (pre-registered)

- a consumer needs the *per-node* call story — which nodes went dirty
  per frame — → the record grows the node list beside the count, the
  ledger's shape unchanged (the route tier's additive-field rule).
- a consumer needs the *timing* story — per-call durations — → the
  record grows the duration beside the facts, the ratio budget
  untouched (the instrument prices itself, never the engine).
- a consumer needs the ledger *on the causal log* — layout facts as
  `InspectEvent`s — → the observer route grows the event species, the
  harness ledger staying the test-side instrument (A4's tier rule).
