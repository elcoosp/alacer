# ADR-0013: M7 scope — the reproducible rig and the cross-framework beat

- **Status:** Accepted (2026-11-08)
- **Implements:** the two remaining items of the v1 definition of done
  (architecture §1): "a public, reproducible benchmark harness (spec:
  `audit/BENCHMARKS.md` §5)" — the declared rig, the protocol, and the
  publication — and "benchmarks where Alacer beats both the immediate-mode
  DX baseline (egui) and the best audited retained stack on the same
  hardware in at least one shared workload" — the beat, computed same-host
  and published with everything labeled
- **Companion:** ADR-0011 (the perf-contract regime this milestone extends
  without moving its pins), ADR-0012 (M6 — the W1/W2/W3 benches this
  milestone puts on the rig), architecture §14.1/§14.2 (the method and the
  workloads), §18 (the portfolio kill criterion names W1/W2/W3 as the beat
  workloads)

## Decision

M6 completed the widget canon and measured W1/W2/W5/W3 as published,
gated benches. What remained of the v1 definition of done was the *evidence*:
the harness must be declared, reproducible, and honest enough that the beat
claim is a measurement rather than a hope. M7 builds exactly that — the rig,
the competitor harnesses, the runner, and the publication — and computes the
beat from same-host medians.

### 1. The rig, declared once

`scripts/collect_rig.py` writes `benchmarks/rig.json`: the host identity
(os/arch/cpu model/count — the same sha256-truncated fingerprint basis the
M5 baseline uses, so a rig and a baseline can be cross-referenced by
fingerprint), the toolchain, the git state, the *protocol* (5 process runs
per case, median-of-medians estimator, criterion sample size and measurement
window), the backend mode (headless CPU; vsync not applicable — labeled as
such, not omitted), and the dependency versions of every measured framework,
read from the lockfiles. The declared rig is the environment the committed
results were collected on; the §14.2 reference desktop (Vulkan + llvmpipe
parity) remains the portfolio step — rerunning the runner on any machine
regenerates `results.json` in place, which is what "reproducible" means
here: one command, no undocumented knobs.

### 2. The competitors, chosen by the document

- **egui** — the immediate-mode DX baseline, named by §1 and §18. Version
  pinned (0.37), headless CPU path (`Context::run_ui` + `Context::tessellate`
  — egui's real per-frame work: widget walk, layout, text shaping through the
  galley cache, tessellation).
- **Masonry** — the audit's retained reference (the pass-contract design §7
  adopts; the doc's most-cited retained stack). Version pinned (0.4), driven
  through the public `RenderRoot` API: `edit_base_layer` mutations plus
  `redraw()`, which runs the real pass pipeline — update, layout, compose,
  paint-scene build, accessibility tree update.

Both harnesses are **standalone cargo projects** under
`benchmarks/competitors/` (`egui_workloads`, `masonry_workloads`) — not
workspace members. The twelve-crate DAG (ADR-9) and the 43-case corpus pin
(ADR-0011/0012) do not move; competitor silicon stays out of the framework's
dependency graph; each project carries its own lockfile, so a competitor
version bump is a reviewed diff, not a transitive drift.

The **no-strawman rule** (§14.1) is enforced structurally: each competitor
workload documents its idiomatic provenance — the exact framework APIs used
and the production answer it deliberately does not take (egui's
`ScrollArea::show_rows` row virtualization; Masonry's `Portal`) — because the
beat workloads pin the *mechanism under test* (tree-wide relayout under
churn, text re-wrap under width jitter, per-frame animation), not the
virtualization strategies both competitors ship for long lists. The README
of each competitor project carries that table; the exit suite pins its
presence.

### 3. The scope-asymmetry rule (conservative against the thesis)

The compared scopes are documented per workload in `results.json` and are
always **conservative against Alacer**: the framework's measured scope is a
subset of what the competitors measure in the same row. W1 compares Alacer's
incremental relayout of the churned frame against egui's *full* immediate
frame and Masonry's *full* pass pipeline. W2 compares incremental text
relayout under ±5% width jitter against egui's re-wrapped tessellated frame
and Masonry's re-wrapped relayout. W3 compares Alacer's animation frame
through the real pipeline *including the damaged software raster* against
egui's and Masonry's frames *without* pixel rasterization (Masonry's vello
raster step needs a GPU device the declared rig does not carry — excluded,
labeled, and conservative: Alacer wins only if it wins while doing extra
work). A claim that survives this asymmetry survives the symmetric
measurement; one that needs the asymmetry reversed is not a claim.

### 4. The protocol and the publication

`scripts/run_comparison.py` is the harness: it builds the competitor
projects, runs each workload's criterion case five times (process-level
runs; the median point estimate per run, then the median of the five — the
§14.1 5-run median protocol), reads the Alacer side from the published
baseline (`w1/incremental_churn_10pct` for `w1_churn_10k`,
`w2/jitter_incremental_1k` for `w2_text_jitter_1k`,
`w3/animation_frame_1k` for `w3_animation_1k` — same-host by fingerprint,
or re-measured with `--remeasure-alacer`), and emits
`benchmarks/comparisons/results.json`:
schema 1, one entry per (workload, framework) with median, run count, scope
note, framework version, and the rig fingerprint; a `beat` object computing,
per workload, whether the Alacer median is lower than *every* competitor's
same-host median. `--validate` checks the committed artifacts cheaply
(schema, fingerprint agreement, run counts, scope notes, the
baselines ↔ results cross-reference) — that is the mode the exit suite and
CI call.

`benchmarks/contracts.json` grows exactly one test-enforced gate —
`comparison_corpus_published`, enforced by the M7 exit suite. Competitor
absolute medians are deliberately **not** regression-gated: ADR-0011's
same-host rule forbids absolute gates on silicon the project does not own,
and a cross-host gate would be exactly the "benchmark theatre" the regime
was built to prevent. The dashboard grows the comparison section
(framework medians per workload, the beat verdict, the advisory badge when
the collecting rig is not the declared reference desktop).

### 5. The executable exit suite

`alacer-testing/tests/m7_exit_criteria.rs` pins the milestone:

- the rig is declared (`rig.json` schema 1, fingerprint agreement between
  rig and results, the 5-run protocol recorded);
- the competitor corpus covers {egui, masonry} × {W1, W2, W3}: standalone
  project, pinned dependency, one `[[bench]]` per workload, the
  no-strawman provenance table present;
- the runner validates the committed artifacts (`--validate` exits green);
- the beat verdict is computable from the committed file: three workloads,
  two competitors each, same-host medians, non-empty scope notes, the
  verdict's won-set ⊆ the workload set;
- the contract gate and the dashboard section publish the comparison;
- the M5/M6 corpus pins do not move (43 workspace cases — M7 adds
  competitor data outside the corpus, never inside it);
- the ADR names the harness pieces, and the workspace ships as 0.7.0.

## Consequences

- The workspace ships as **0.7.0**; the CHANGELOG documents the release.
- No new workspace crates; the DAG check, the 43-case corpus, the M5/M6
  suites, and the snapshot corpus are untouched by construction.
- `benchmarks/rig.json` and `benchmarks/comparisons/results.json` are
  committed data, reviewed like the baselines; the dashboard regenerates
  from them.
- The first published comparison is collected on the declared rig of
  record — a 2-vCPU headless CI-class Linux host (the fingerprint names it)
  — with `advisory: true` semantics for anything the reference desktop
  would re-measure; the harness, not the hardware, is the deliverable.
- The beat claim, if green, closes the §1 definition of done; the remaining
  v1 items are portfolio claims (the reference-rig cross-framework beat on
  shared hardware) that this harness makes runnable anywhere.
- egui and masonry versions move independently of the workspace; the
  version bump is a three-edit change (competitor lockfile, results,
  README provenance table) enforced by the exit suite's corpus check.
