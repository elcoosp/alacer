# ADR-0015: M9 scope — the windowed a11y path joins the incremental contract

- **Status:** Accepted (2026-11-08)
- **Implements:** the §11.1 item 4 contract at the real window: "only dirty
  nodes are re-emitted in the `TreeUpdate` (never a full-tree rebuild per
  frame, the egui cost we refuse)" — delivered by routing the windowed path
  through the pipeline's own sink, not around it
- **Companion:** ADR-5 (the a11y contract this completes at the adapter
  seam), ADR-9 (the lints/DAG regime the downcast hatch respects),
  architecture §11.1 (the incremental a11y update), §18 (the M9 roadmap row)

## Decision

M8 closed the last *input* seam; M9 closes the last *output* seam. The M3
incremental `AccessKitSink` was delivered, tested, and simultaneously inert
in every real window — the exact class of docs-drift-with-a-latent-wiring-
bug that M8 repaired for input. The facade's `pipeline_default` installs the
sink into the pipeline; pass 9 feeds it the per-frame `A11yDelta`; and the
window then ignored all of it: `AlacerApp::request_tree_update` rebuilt a
full `TreeUpdate` from the snapshot on every a11y frame — the egui cost §11.1
refuses — while dropping the M6 vocabulary the sink exists to carry. A
checkbox toggled in a real window reported `Toggled` to nothing; a table
cell reported its grid position to nothing; and the sink's `updates` Vec
grew without bound in every windowed run because nobody drained it.

M9 makes the pipeline's sink the *only* materialization path:

- `alacer-core` gains the type-erased escape hatch (`A11ySink::as_any_mut`,
  the `Widget::as_any_mut` pattern) and `PassPipeline::a11y_sink_mut` — the
  trait stays adapter-free, the DAG untouched.
- `alacer-a11y` factors its builder into one `build_update`, caps produced
  updates in a ring (`UPDATE_RING_CAP = 8` — a dropped stale delta is
  re-established at connect time by the full tree, so the cap bounds memory,
  never correctness), and exposes the protocol surface: `drain_updates` for
  the steady-state frame boundary, `full_tree_update` (pure, same builder)
  for the adapter-activation request.
- `alacer-platform` deletes its snapshot-rebuilding `TreeUpdateBuilder` and
  the `tree_dirty` bookkeeping. `InitialTreeRequested` answers with the
  sink's full tree; every tick is followed by a drain-and-forward of the
  sink's pending deltas; the dead `PlatformEvent::TreeChanged` variant
  becomes the embedder's flush nudge it was always named for.
- The bench pair publishes the ratio: `a11y/full_rebuild_1k` (the pre-M9
  shape) vs `a11y/incremental_1k_dirty_10pct` (the M9 shape), contracted as
  `a11y_incremental_speedup ≥ 2.0×`.

### 1. Why this is the milestone

The invalidation thesis (§4.1) is "one `Change` currency feeding incremental
layout, journal-replay paint, and incremental AccessKit updates" — M9 is the
last leg of that sentence where the production window still ran a different
algorithm than the tests proved. It is also the cheapest honest repair of a
documented-but-undelivered contract, and it is correctness work, not just
performance: the M6 vocabulary (values, toggle state, grid slots) reached
headless queries only; with M9 it reaches the OS. The next tier of a11y work
(actions, announcements, live regions) is unblocked rather than blocked by
the old split-brain.

### 2. The seams that did not move

- **The corpus grows by the coordinated rule, nothing else moves.** One
  bench file, two criterion ids, two regression contracts, one budget —
  45 → 47 cases; the M5/M6/M7/M8 pin suites move together (the M9 suite
  asserts all four).
- **The `A11ySink` trait stays adapter-free.** The escape hatch returns
  `Option<&mut dyn Any>`; a foreign sink downcasts to `None` and the
  platform leaves it alone (test-enforced).
- **The harness path is untouched.** `TestHarness` still reads
  `a11y_snapshot()` for queries; the M9 suite drives the same downcast the
  platform uses, so the suite exercises the mechanism itself, not a
  source-string pin of it.

### 3. What this does not do

- No new a11y actions beyond `DefaultPress`, no announcements or live
  regions, no subtree granularity knobs — the drain protocol is their
  prerequisite, not their implementation.
- No `accesskit` re-export from `alacer-core`: the adapter vocabulary stays
  in `alacer-a11y` and `alacer-platform` (ADR-9).
- No behavioral change for the harness or the record/replay suite: their
  sink is read through the same trait as before.

## Consequences

- The workspace stays 0.7.0; the CHANGELOG documents M9 under Unreleased.
- The corpus is 47 cases; the M5/M6/M7/M8/M9 suites pin the number together.
- The baselines are re-published from the collecting host: the
  `a11y_incremental_speedup` budget joins the two ratio budgets as the
  host-independent gates; the same-host regression rows are gated only on
  the recorded fingerprint.
- `alacer-platform` loses its full-rebuild `TreeUpdateBuilder` and the
  `tree_dirty` flag; `PlatformEvent::TreeChanged` gains a real consumer
  (flush the sink now) after shipping as a producer-less variant since M3.
- The a11y update cost on a real window drops from O(tree) per a11y frame
  to O(dirty) — the M3 p99 gate now governs the windowed path too, and the
  `tree_update_p99_half_ms` budget's "enforced_by" claim is true where it
  matters, not only headless.
