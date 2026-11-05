# ADR-0002: reactive_graph ImmediateEffect + batch — the scheduling contract

- **Status:** Accepted (2026-11-05)
- **Extends:** the architecture's ADR-6

## Decision

The flux bridge uses `ImmediateEffect` (synchronous, feature `effects`) with
`batch()` coalescing. The contract — *one write batch ⇒ at most one
layout+paint cycle* — is asserted at three levels:

1. flux unit: 5 writes in one batch → exactly 1 effect rerun;
2. pipeline: batch → exactly one layout pass and one paint pass;
3. E2E: a synthetic click → one layout cycle (`m0_exit_criteria.rs`).

reactive_graph's `BATCH` is process-global, so tests that depend on effect
timing serialize through `flux::batch_test_lock()`.

## Kill-switch

If effect scheduling cannot drive paint batching within budget, GPUI-style
draw-level entity tracking becomes the paint consumer, keeping
reactive_graph for data (architecture §5.2). The M0 numbers clear the bar.
