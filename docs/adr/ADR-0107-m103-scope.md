# ADR-0107: M103 scope — the time seam: the clock the platform owns becomes a seam

- **Status:** Accepted (2026-11-18)
- **Implements:** the one external effect with no seam cashed — the
  clock. Two contracts read a clock and both hardcoded
  `std::time::Instant::now`: the platform's click chain, whose own M13
  test block carried the pre-registered gap since ADR-0019 ("the time
  bound rides `Instant` and is not sleep-tested; the distance bound
  is") — the double-click window was the one input contract CI never
  priced; and nothing else, because every other external fact had
  already grown its seam (the clipboard, ADR-0020; the text services,
  ADR-9; the theme, ADR-0079; the OS settings, ADR-0080). The clock
  was the last hardcoded read — cashed as a real capability: "a test
  needs the double-click window priced deterministically, a consumer
  needs elapsed time its harness controls → the reading site holds one
  [`TimeSeam`], the canon clock answers real time, a manual clock
  answers exactly what the test advanced it to — never a sleep, never
  a hope that the scheduler agrees"
- **Companion:** ADR-2 (the one-currency discipline the read joins —
  a clock read queues nothing, it prices what the frame already did),
  ADR-0019 (the click chain's own doc — "the platform owns the clock,
  the widget never does" — now names *which* clock, ADR-0107), ADR-9
  (the DAG the change never touches: the seam lives in core, the
  canon and deterministic impls beside it, the platform only holds a
  box), architecture §10.3 (the platform seam), §7.2 (the pass ledger
  the reads price), §18 (the M103 roadmap row)

## Decision

- **One trait, one read.** [`TimeSeam`](crates/alacer-core/src/time.rs)
  answers `now_nanos(&self) -> u64` — monotonic nanoseconds since the
  clock's own anchor, never backwards within one installed clock.
  Deliberately minimal: durations are the caller's arithmetic
  (differences of two reads), the shape both consumers already ride —
  the chain's window bound compares deltas, the pass ledger prices
  `end − start`. No sleep, no deadline, no wall-calendar read: a wall
  date is a platform fact the OS owns (the setting seam's tier), and a
  waiting framework is not this framework's shape — the winit loop
  parks, it never polls a clock.
- **The provided implementations ride the provided-implementation
  rule.** [`MonotonicClock`](crates/alacer-core/src/time.rs) is the
  canon: real monotonic time off `std::time::Instant`, anchored at its
  construction, the default build's shape
  (`ClickCounter::default` rides it). [`ManualClock`] is the
  deterministic seam: a shared `Arc<AtomicU64>` reading, `advance`
  writing — a clone shares the one reading, so a test advances through
  its handle while the consumer reads through theirs (the
  shared-handle shape the recorder handles ride). A frozen clock
  prices every duration exactly zero; an advanced one prices exactly
  what the advance wrote.
- **The click chain rides the seam.** The platform's
  [`ClickCounter`](crates/alacer-platform/src/lib.rs) holds
  `Box<dyn TimeSeam>` beside its count, prices
  `now − then ≤ DOUBLE_CLICK_NANOS` (the window's own width in the
  seam's unit — one constant, never a second clock-shaped fact), and
  constructs with the canon clock, byte-for-byte today's behavior. The
  M13 block's gap closes the honest way: four tests install a
  [`ManualClock`] and price the window exactly — a press 200 ms later
  chains, 500 ms later starts fresh, exactly 400 ms chains (the ≤
  bound), and the triple counts before a stale chain resets — no
  sleeps, and the distance bound's tests stand unchanged beside them.
- **The seam is public vocabulary.** `TimeSeam`, `MonotonicClock` and
  `ManualClock` ship in core's prelude (the `KeymapSeam`/`SettingSeam`
  placement) and the facade re-exports the prelude — app code that
  needs elapsed time holds one seam and never calls `Instant::now`,
  the axiom the chain's own doc names ("the widget never owns the
  clock") extended to the app's own code.
- **No trail tier.** A clock read prices what the frame already did —
  it writes nothing, queues nothing, and carries no payload a
  devtools surface needs (the pass ledger and the chain's click
  counts are the named data). The seam has no record because the seam
  has no side effect.
- **The living documents re-agree at M103 and 0.17.0.** The README's
  status block, the ADR count (a hundred and nine), the architecture
  register (ADR-0107 scopes M103), the roadmap row (§18), the
  CHANGELOG's unreleased block, and the examples catalog (the
  `time_seam` row) all name the seam; the corpus grew 114 → 116 by the
  coordinated rule below.

## Consequences

- **The corpus grew 114 → 116 by the coordinated rule:**
  `click_chain/seamed_chain_1k` — 1,000 chained presses through the
  canon clock, the shipped default's per-press price (the clock read
  is the chain's own cost) — beside `click_chain/manual_chain_1k`, the
  same storm through the deterministic clock, the seam indirection's
  floor. The pair's ratio is the vDSO read's own price, not the
  seam's, so no ratio budget rides it: both cases publish regression
  gates, the per-case medians the perf contracts hold. The m-suite
  pins re-agree at 116.
- **The window is finally a contract CI enforces.** The M13 gap was
  pre-registered debt: the chain's time bound could silently regress
  (a flipped comparison, a wrong constant) and every test would stay
  green. The four deterministic tests pin it at the nanos.
- **Consumers get deterministic time for free.** The
  `examples/time_seam.rs` beat is the shape: inject a `ManualClock`,
  advance exactly 1.5 s, and the tree says `1.500 s elapsed` — the
  same code a wall-clock app rides, exact in every CI run.
