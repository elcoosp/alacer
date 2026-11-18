# ADR-0108: M104 scope — the pipeline's pass timings ride the time seam

- **Status:** Accepted (2026-11-18)
- **Implements:** the timed tick's own clock cashed — the seam ADR-0107
  shipped at the platform's click chain now rides the pipeline's pass
  ledger too. The M1 gate instrumentation (architecture §7.2, the
  `set_timings` door) priced every pass against a hardcoded
  `Instant::now` — nine read sites, the one remaining hardcoded clock
  after M103 — so the ledger the benches gate against, the overhead
  test asserts, and the frame report carries was wall noise no test
  could assert exactly and no demo could print twice. Cashed as a
  real capability: "a test needs the pass ledger priced exactly, a
  harness needs the timed frame deterministic → the pipeline holds one
  [`TimeSeam`], the [`set_clock`](crates/alacer-core/src/passes.rs)
  door installs it beside `set_timings`, and every read the timed path
  pays answers the installed clock — the fallback anchor still answers
  real time, byte-for-byte the pre-seam tick"
- **Companion:** ADR-0107 (the seam's own shape — one read, monotonic
  nanos, durations as differences), ADR-0011 (the perf contracts the
  new budget rides), ADR-2 (the idle-frame property the timed path
  keeps — a timed tick is still a skip-priced frame, never work), ADR-9
  (the DAG: the door lives in core, no crate moved), architecture §7.2
  (the ten-pass contract's instrumentation), §18 (the M104 roadmap row)

## Decision

- **One door beside the instrumentation's own.**
  [`PassPipeline::set_clock`](crates/alacer-core/src/passes.rs)
  installs `Box<dyn TimeSeam>`; every timing read — the six
  method-level sites (the three dispatch passes, the tick's total, the
  layout work-read, `run_pass`) and the three closure-level sites
  (the compose, paint and a11y work-reads) — answers
  `read_clock_nanos`: the installed seam's monotonic nanos, or the
  fallback anchor's elapsed when nothing is installed. The anchor is
  the pipeline's own construction instant; only differences of reads
  are priced, so the deltas equal the deltas the direct
  `Instant::now` reads answered, from one fixed origin — the default
  build's behavior is byte-for-byte the pre-seam tick's.
- **The timed frame is now exact data, not wall noise.** A stepping
  seam clock (every read one step past the last) prices every pass a
  whole step: two idle ticks of the same tree price identically —
  `a_seam_clock_prices_two_ticks_identically` asserts the ledgers
  equal, as data. A frozen [`ManualClock`] prices the whole frame zero
  — the timed skip is still a skip. `set_clock` replaces: the newest
  install wins, one door to swap a deterministic clock in and the canon
  real time back out. And with no seam installed the fallback still
  measures — `without_a_seam_the_tick_still_measures_wall_time` keeps
  the shipped shape honest.
- **The read-count structure is kept, not invented.** The due-gated
  passes (layout, compose, paint, a11y) read only when due — a skipped
  pass prices zero *and pays no read*; the always-on passes
  (mutate, focusables, focus, style) read start and end through
  `run_pass` regardless. The tests assert whole steps and same-class
  determinism (idle against idle), never a fixed read count — the
  ledger's shape stays free to evolve, the seam's determinism is what
  pins.
- **No patch-tier growth, no trail tier.** The clock is instrumentation
  state the door swaps — nothing declarable, nothing the view DSL
  names (the M98/M99 rule at the timing tier); and a clock read
  prices what the frame already did — no side effect, no record, no
  devtools surface (the ADR-0107 rule at the pipeline tier).
- **The living documents re-agree at M104 and 0.17.0.** The README's
  status block and criterion count (a hundred and eighteen cases), the
  architecture register (ADR-0108 scopes M104) and roadmap row (§18),
  the CHANGELOG's unreleased block, and the examples catalog (the
  `frame_clock` row) all name the door; the corpus grew 116 → 118 by
  the coordinated rule below.

## Consequences

- **The corpus grew 116 → 118 by the coordinated rule:**
  `pass_overhead/timed_idle_10k` — the timed idle tick on the 10,000-node
  static tree, the shipped fallback's own price (the skips the ledger
  records, the per-frame rebuild the timed path pays instead of the
  idle ledger's handle, the real reads) — beside
  `pass_overhead/seamed_timed_idle_10k`, the same tick with the
  deterministic clock behind the door. The
  `timed_tick_seam_overhead` ratio budget (≤ 1.25×) holds the seam's
  indirection at the fallback's own class; both cases publish
  regression gates beside it. The m-suite pins re-agree at 118.
- **The M1 gate's ledger became assertable.** The overhead test and
  the benches gated on medians; the *structure* (which pass priced
  what) was unassertable against wall noise. With the seam installed
  the ledger is a table of exact values — the `examples/frame_clock.rs`
  beat prints it, byte-identical every run.
- **Deterministic pricing rides the same door consumers get.** The
  harness's timed frames, the inspector's frame trail and a consumer's
  own `set_timings` diagnostics all answer the installed clock — one
  seam, one door, the same shape the click chain rides.
