# ADR-0091: M87 scope — the input-frame allocation budget, the tier the warm tick does not cover

- **Status:** Accepted (2026-11-16)
- **Implements:** ADR-0088's own pre-registered re-open rule cashed:
  "a consumer needs the budget at a tier the warm tick does not cover
  (an input frame, a flip frame, a present path the harness cannot
  drive) → the beat joins the suite as its own measured case, priced
  by the same ledger — never a budget asserted without an
  instrument." Through M86 the ledger's budget had one tier — the
  warm tick of a static UI, pinned clean to the byte (the m84 suite) —
  and an input-driven frame was priced by nothing: the folklore the
  instrument exists to answer. The instrument measured first, and the
  measured truth is the budget this cut pins: an input frame is **not**
  the warm path — a pointer-move frame churns a small, balanced burst
  (6 blocks / ~752 bytes, allocated and freed inside the same tick),
  the dispatch queue's one-time block (~192 bytes) lands on the first
  tracked move frame and is held ever after (the capacity, never
  re-paid), and the warm idle tick beside it stays clean ([`alloc/input_frame_budget`](crates/alacer-testing/benches/alloc.rs),
  the corpus growing 92 → 93 by the coordinated rule)
- **Companion:** ADR-0088 (the ledger the beat rides — the armed
  window, the snapshots, the delta answers), ADR-0011 (the coordinated
  rule the corpus grows by), ADR-0008 (A4 — the headless pipeline the
  beats drive), architecture §A7, §14.3 (the warm path), §18 (the M87
  roadmap row)

## Decision

- **The budget's numbers are measured, never aspirational.** The
  instrument priced the input frame before the budget named it: the
  first tracked move frame carries the dispatch queue's one-time
  block (**+192 bytes net** — the capacity, held after), the
  steady-state move frame churns **6 blocks / ~752 bytes** and retains
  **nothing** (`is_balanced`, net zero to the byte, twenty frames in a
  row), a key frame churns 4 blocks and balances, a click frame 7, and
  the warm idle tick answers clean in the same window (the m84 budget
  re-agreed). The hook's own cost rides separately (`alloc/ledger_overhead`'s
  priced shape): the beat's numbers are the pipeline's, not the
  checker's.
- **The budget the suite pins is the honest shape, not a zero.** The
  warm path keeps its zero-alloc line (A7's own claim); the input
  frame's line is the balanced one — the churn is real, the retention
  is none. Asserting `is_clean` on an input frame would be a budget
  the facts refute; asserting *balanced* is the contract the
  instrument can enforce forever: any frame that leaks (allocations
  that outlive their tick) fails the suite, the bench's honesty check,
  and the example, with the delta that broke it printed. The first
  tracked frame's one-time block is named in the same breath — a
  consumer reading the budget sees the capacity, never a per-frame
  leak.
- **The beat rides the ledger's own tier rules.** The suite (A4) runs
  the real pipeline under the adopted hook, one ledger armed for
  exactly the beats it prices (dhat's one-profiler rule); the bench's
  case primes the one-time block outside its priced window and
  asserts the steady state's balance before the clock (the m84
  bench's honesty-before-the-clock shape); the example prints the
  beats and panics past them. The framework never installs the hook —
  the binary's own allocator, ADR-0088's rule at the new tier.
- **The corpus carries the tier's pricing.** The corpus grows 92 → 93
  by the coordinated rule (ADR-0011: bench, gate, and baseline in one
  cut): `alloc/input_frame_budget` — the steady-state move frame's
  balanced churn priced by the same ledger the suite rides, beside
  `alloc/warm_frame_zero_alloc` (the warm path) and the ledger's own
  priced costs. The m-suite pins re-agree at 93.

## Re-open rules (pre-registered)

- a consumer needs the budget at a tier the harness cannot drive (a
  present path, a real window's flip frame) → the beat joins the
  suite as its own measured case when an instrument can drive it —
  never a budget asserted without an instrument (the rule this cut
  cashed, standing).
- a consumer needs the churn itself shrunk (the 752-byte burst a
  tighter dispatch path could avoid) → the fix prices itself on this
  beat's own baseline first (the measured before/after, ADR-0011's
  rule), the budget's line unchanged — never a tighter path asserted
  without the beat.
- a consumer needs the one-time block's source named in the API (a
  reader pricing the capacity) → the input tier already streams the
  coalescer's facts; a record field naming the capacity is priced by
  a real reader first (ADR-0084's rule) — never a record per frame.
- a pass count or dispatch shape changes and the burst's size moves →
  the suite's asserts are the delta answers' (balanced, net zero),
  shape-independent by construction; the corpus's published median
  moves with the baseline's own regen rule — never a byte count
  pinned where the design's contract is the balance.
