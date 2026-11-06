# ADR-0011: M5 scope — the examples gallery, the migration guides, and baselines as contracts

- **Status:** Accepted (2026-11-06)
- **Implements:** the roadmap's M5 bullet (architecture §18): "Community:
  examples gallery, egui/Iced migration notes, perf dashboards" — exit
  criteria: "criterion baselines published as contracts"
- **Companion:** ADR-0005 / ADR-0010 (testing productization, the M4
  devtools layer), ADR-6 (the scheduling contract the benches measure),
  ADR-0008 (the W1 gate the budget contract publishes)

## Decision

M5 turns the framework's first outward-facing milestone: everything before
it proved the architecture *to the repository*; M5 makes the repository
provable *from outside* — runnable in one file, portable from two
frameworks, and measurable against published numbers that fail loudly.

### 1. The examples gallery

The widget vocabulary becomes a one-file tour and a catalog:

- **`crates/alacer/examples/gallery.rs`** — one window exercising every core
  widget: styled labels, buttons with a bound counter (the ADR-6 seam, one
  write batch per click), the `TextInput` (the M3 IME reference widget),
  panels, a translation-only scrolling `List` (§8.2), rows and spacers, the
  shared `theme` vocabulary. It is written the way the rules demand —
  handlers write signals, effects queue masks — so it doubles as the
  idiomatically-correct starting template.
- **`docs/examples.md`** — the catalog: every example with what it
  demonstrates, which architecture contract it exercises, and its run
  command. The M5 exit suite enforces coverage: a new example file without
  a catalog row fails `cargo test`. The blessed snapshot corpus
  (30 scenarios, ADR-0005) remains the pixel-level regression net behind
  the gallery.

### 2. The egui/Iced migration notes

`docs/migration/from-egui.md` and `docs/migration/from-iced.md` map the two
audited reference points onto Alacer — egui (immediate mode, the DX
baseline) and Iced (the closest retained relative) — each with the same
required section set (mental model, app shell, widget mapping, reactivity,
layout, input and events, accessibility, testing, interop limits), enforced
by the M5 suite so a port cannot skip the hard sections. Both carry an
explicit version-scope honesty note (architecture §20's rules apply to
documentation too), real API on both sides of every mapping table, and
links to the runnable examples that demonstrate the mapped seam. The
guides state the interop limits plainly: no immediate escape hatch, no
`Task`/command type, one window, themes as constants.

### 3. Criterion baselines published as contracts

The exit criterion, implemented as three artifacts and one mechanism:

- **The published baseline (`benchmarks/baselines/baselines.json`).** One
  entry per criterion benchmark case — all 38 across the five benching
  crates — distilled from real `cargo bench` runs by
  `scripts/collect_baseline.py`: criterion median, mean, median absolute
  deviation, sample count, plus the environment that produced them (git
  commit, dirty flag, toolchain, host fingerprint). The fingerprint
  (os/arch/cpu model/count, sha256-truncated) is the honesty anchor for
  every absolute number.
- **The contract set (`benchmarks/contracts.json`).** Two kinds of gate:
  - **Budgets** — the architecture's falsifiable gates expressed as median
    ratios over published benches: the W1 ≥ 2× incremental-relayout gate
    (§18 M2, ADR-0008) measured at **14.01×** on the published baseline.
    Ratios are host-independent, so they are enforced *everywhere* — local
    runs and CI.
  - **Regressions** — one gate per benchmark case (a bijection with the
    published baseline is enforced by the exit suite): fresh median ≤
    published median × (1 + 25% tolerance). Scoped `same-host`: absolute
    nanoseconds only compare on the recorded host fingerprint. On a foreign
    host — a CI runner — they are reported as SKIPPED, not failed;
    `--strict-host` turns the skip into a failure for pinned-hardware cron
    runners. Absolute-time contracts across different silicon would be
    noise theatre, and the roadmap's "published as contracts" is read
    honestly: the *ratio* budgets are the portable contracts, the absolute
    gates are the same-host tripwires, and criterion's own previous-run
    comparison remains the first-line per-machine regression detector.
- **The mechanism.** `scripts/check_perf_contracts.py` compares fresh
  criterion data (or a `--candidate` baseline-format file) against the
  published baseline: budgets always, same-host regressions on matching
  fingerprints, and an *uncontracted benchmark* rule — a bench case that
  runs without a published gate fails the check, so the corpus cannot grow
  unmeasured. `scripts/perf_dashboard.py` renders
  `benchmarks/dashboard/index.html` — a self-contained, dependency-free
  dashboard (no external assets, no JavaScript): the environment header,
  the budget contracts with measured ratios and PASS/FAIL, the full
  baseline table with delta bars, and the four test-enforced gates the
  criterion model cannot express (pass overhead < 15%, TreeUpdate p99 <
  0.5 ms, the patch frame budget, zero-measure scrolling) each with its
  enforcement site. The dashboard is committed and regenerated whenever the
  baseline changes; the exit suite asserts every contract is published on
  it.

### 4. The executable exit suite

`alacer-testing/tests/m5_exit_criteria.rs` pins the milestone as tests:
the baseline's schema and environment header; the contracts ↔ baseline
bijection at the pinned corpus size (38, mirroring the M1 suite's
30-scenario pin); the budgets holding on the published data (including the
architecture's own 2× W1 value); the dashboard publishing every contract;
the catalog covering every example; the gallery exercising the widget
vocabulary through the ADR-6 write seam; and both migration guides
carrying the full required section set. Like every milestone before it,
the exit criteria are green in CI or the milestone did not ship.

## Consequences

- The workspace ships as **0.5.0**; the CHANGELOG documents the release.
- CI gains a `perf-contracts` job: benches run (reduced sampling), budgets
  are enforced, and the dashboard is regenerated and uploaded as an
  artifact on main. Same-host regression gates fire on maintainer machines
  and pinned-hardware cron runners, not on CI's fleet — stated here so the
  skip is a policy, not an oversight.
- Adding a benchmark case now requires three coordinated edits (bench,
  contracts.json gate, baseline regeneration), all enforced by the M5
  suite and the checker's uncontracted-benchmark rule.
- The published baseline records this repository's reference host; it is
  data, not a promise — the budgets are the promise.
- The gallery, the guides, and the dashboard are documentation with a
  regression net: each is covered by the exit suite, so none can rot
  silently.
