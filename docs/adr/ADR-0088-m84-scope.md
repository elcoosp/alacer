# ADR-0088: M84 scope — the allocation ledger, the warm-frame budget enforced

- **Status:** Accepted (2026-11-16)
- **Implements:** architecture A7's consequence — "idle CPU = 0%
  (`ControlFlow::Wait`); the warm paint path for a static UI performs
  **zero heap allocations** [budget, CI-enforced via dhat]" — the
  budget's checker, announced from the first cut and shipped by none:
  through M83 the workspace named dhat in its adopted-dependencies
  list, no crate adopted it, and a warm tick's heap movement was
  folklore priced by nothing. The ledger adopts the hook
  ([`dhat::Alloc`](crates/alacer-testing/src/alloc.rs), A2 — steal
  proven cores; own only the composition; the crate is the one the
  claim always named) and grows the budget's vocabulary beside it:
  the armed [`AllocLedger`](crates/alacer-testing/src/alloc.rs)
  window, the [`AllocStats`](crates/alacer-testing/src/alloc.rs)
  snapshots, the [`AllocDelta`](crates/alacer-testing/src/alloc.rs)
  answers ([`measure`](crates/alacer-testing/src/alloc.rs) pricing one
  beat beside the body's own answer)
- **Companion:** ADR-0001 (A1 — the one invalidation currency the
  warm path replays), ADR-0008 (A4 — the headless pipeline the beats
  drive), ADR-0011 (the coordinated rule the corpus grows by), ADR-0022
  (the consumable contract the measured truth ships by), architecture
  §A7, §14.3 (the warm path), §18 (the M84 roadmap row)

## Decision

- **The budget's numbers are measured, never aspirational.** The
  instrument measured first: a static UI's warm tick through M83
  churned **2 blocks / 288 bytes per frame** — not zero, and never
  freed into a ledger that could say so. The source is the pass
  ledger's own bookkeeping: the tick took its buffer
  (`std::mem::take(&mut self.interframe)`), grew it twice (capacity 4
  then 8), handed it into the [`FrameReport`](crates/alacer-core/src/passes.rs),
  and the consumer dropped it — the buffer's capacity died with every
  report, so every warm tick re-paid the take-push cycle. The design's
  intent was reuse from the start (the `interframe` field's own doc);
  the reuse just leaked into the report.
- **The fix keeps the API and keeps the intent.** The tick's ledger
  buffer rides back: the report's
  [`passes`](crates/alacer-core/src/passes.rs) field is the ledger by
  handle — [`std::sync::Arc<Vec<PassRun>>`] — and a warm idle frame
  with timings off answers with the **canonical idle ledger**, built
  once and handed out by [`Arc`] clone (an atomic increment, never an
  allocation). On every cache hit the buffer returns into the pipeline
  (`self.interframe`), capacity intact — the take-push cycle re-pays
  nothing in steady state. A frame where a pass ran, a dispatch-driven
  entry rides, or timings are on carries its own freshly built ledger,
  honestly priced: work frames are the priced path, the warm path is
  the budget's. The measured truth this cut ships: **the warm tick
  allocates zero — to the byte** (the m84 suite and the
  `alloc/warm_frame_zero_alloc` case pin it).
- **The hook is adopted, the composition owned.** dhat's
  [`Alloc`](crates/alacer-testing/src/alloc.rs) is this workspace's
  global allocator wherever a budget beats — a test binary, a bench, an
  example installs it at its own top level, never the framework (a
  consumer's allocator is the consumer's). The composition the ledger
  owns is the vocabulary the suites assert against: [`AllocLedger::arm`](crates/alacer-testing/src/alloc.rs)
  opens one testing-mode window (dhat's one-profiler rule; testing
  mode writes no profile file — the window answers only to the code
  that reads it), [`AllocStats::get`](crates/alacer-testing/src/alloc.rs)
  reads the counters inside it, [`AllocLedger::measure`](crates/alacer-testing/src/alloc.rs)
  brackets one beat, and [`AllocDelta`](crates/alacer-testing/src/alloc.rs)
  answers the movement — [`is_clean`](crates/alacer-testing/src/alloc.rs)
  the budget's word, [`is_balanced`](crates/alacer-testing/src/alloc.rs)
  the churn's, [`net_bytes`](crates/alacer-testing/src/alloc.rs) the
  growth's.
- **The beats pin what is true, priced separately.** The m84 suite
  runs the real pipeline (A4) and pins: the warm tick clean; the pass
  ledger shared by handle (`Arc::ptr_eq` across two warm ticks); eight
  warm ticks leaving the heap where they found it; the observed tick's
  record transient and balanced (the ring at its cap pays its own
  record back by eviction, the dropped count the rule); the raster
  beat's honest scope — the headless harness materializes an owned
  pixmap per call, the buffer the windowed backends hold and reuse is,
  here, the caller's, and the beat's net is the glyph caches' one-time
  warmup, never a per-frame cost.
- **The corpus carries the ledger's pricing.** The budget is a new
  route in the corpus — the corpus grows 86 → 90 by the coordinated
  rule (ADR-0011: bench, gate, and baseline in one cut):
  `alloc/warm_frame_zero_alloc` (the budget's beat, tracking included),
  `alloc/warm_frame_observed` (the priced record beside it),
  `alloc/ledger_overhead` (the hook's own wrapper cost, per
  allocation), `alloc/snapshot_get` (the counters read, per snapshot),
  the m-suite pins re-agreeing at 90.

## Re-open rules (pre-registered)

- a consumer needs the budget at a tier the warm tick does not cover
  (an input frame, a flip frame, a present path the harness cannot
  drive) → the beat joins the suite as its own measured case, priced by
  the same ledger — never a budget asserted without an instrument.
- a consumer prices the ledger's hook cost on a hot path outside the
  suite → the hook stays the test binary's own (the framework never
  installs it); a production build that wants heap accounting grows the
  platform's own vocabulary, priced by its own ADR.
- a consumer needs the counters' shapes beyond the six the stats carry
  → the struct grows the field beside the facts it already names (the
  additive rule), the delta algebra growing beside it — never a second
  ledger type per question.
- a pass count changes and the canonical idle ledger's length follows →
  the cache's length check regenerates the ledger once, the buffer
  riding back as before — never a stale handle served after the
  contract grew.
