# ADR-0003: Persistent Taffy via mark_dirty + engine caches

- **Status:** Accepted (2026-11-05)
- **Extends:** the architecture's ADR-3 (the Q1 spike, resolved early)

## Decision

The Taffy tree is created once and **never cleared**. Incrementality is
expressed through `mark_dirty` on the persistent tree; Taffy's per-node
caches bound recomputation to the dirty path — the torin
ancestor-candidate semantics (§8.2 item 3) expressed through the engine's
own cache, with no second bookkeeping to keep coherent.

## Evidence

- Scrolling: translation-only in the compose pass; the M0 test asserts
  **zero** measure calls for a scroll tick.
- The mini-W1 criterion bench (100/1k/10k nodes) tracks the incremental-vs-
  full gap from day one; the ≥2× W1 gate remains M2's.
- §8.4 equivalence: a seeded property test compares the churned incremental
  engine against a fresh rebuild over identically-constructed arenas.
