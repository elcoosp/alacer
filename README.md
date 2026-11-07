<div align="center">
  <img src="docs/logo.png" alt="Alacer Logo" width="300"/>
  <p>
    <strong>A retained-mode Rust UI framework where reactivity, layout, paint, and accessibility consume one shared invalidation currency — engineered for best-in-class performance and developer experience, with zero-config accessibility.</strong><br/>
    <em>Alacer</em> (Latin for <em>swift, eager, lively</em>) implements a unified per-node change-mask pipeline, so every subsystem sees exactly what changed and nothing more. It combines winit, wgpu, reactive_graph, taffy, parley, and accesskit into a cohesive 12-crate workspace with snapshot testing, criterion benchmarks, both GPU and software rendering backends, a subsecond hot-reload contract, and the Causality inspector.
  </p>
  <p>
    <img src="https://img.shields.io/badge/Rust-1.99%20%7C%202024-000000?style=flat-square&logo=rust" alt="Rust 1.99 / Edition 2024"/>
    <img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue?style=flat-square" alt="License MIT OR Apache-2.0"/>
    <img src="https://img.shields.io/badge/Backend-wgpu%20%2B%20software-6A0DAD?style=flat-square" alt="Backend wgpu + software"/>
    <img src="https://img.shields.io/badge/Crates-12-6F4E37?style=flat-square" alt="12 crates"/>
    <img src="https://img.shields.io/badge/Milestones-M0%20--%20M6-green?style=flat-square" alt="M0 through M6 complete"/>
    <img src="https://img.shields.io/badge/Tests-330%20passing-brightgreen?style=flat-square" alt="330 tests passing"/>
    <img src="https://img.shields.io/badge/A11y-AccessKit-4B32C3?style=flat-square" alt="AccessKit accessibility"/>
    <img src="https://img.shields.io/badge/Layout-Taffy-228B22?style=flat-square" alt="Taffy layout"/>
    <img src="https://img.shields.io/badge/Text-Parley-007ACC?style=flat-square" alt="Parley text"/>
    <img src="https://img.shields.io/badge/Reactivity-reactive__graph-FF4500?style=flat-square" alt="reactive_graph reactivity"/>
    <img src="https://img.shields.io/badge/Hot--reload-subsecond-FF8800?style=flat-square" alt="subsecond hot reload"/>
  </p>
</div>

---

## Status: M0 + M1 + M2 + M3 + M4 + M5 + M6 exit criteria, all green

The repository implements milestone **M0** (the skeleton: winit + wgpu + reactive_graph + taffy + parley + accesskit on one window), milestone **M1** (the view DSL, the pass contract, and the test corpus), milestone **M2** (the invalidation currency: masks + persistent Taffy + torin-style deltas + scene journal replay), milestone **M3** (the a11y contract + partial paint + the IME contract), milestone **M4** (the hot-reload contract on subsecond + the CI hot-patch suite + the Causality inspector + publish readiness), milestone **M5** (the examples gallery + the egui/Iced migration guides + the perf dashboard with criterion baselines published as contracts), and milestone **M6** (the benchmark canon: the todo/data-table widget canon, the W2/W3/W5 shared workloads as gated benches, and the `.alc` record/replay format) of the architecture document in `docs/architecture.md` — completing the §1 definition of done's widget canon.

| Criterion | Evidence |
|---|---|
| **30 blessed snapshots green** | `alacer-testing/tests/m1_exit_criteria.rs` — 30 scenarios through the real pipeline + software raster |
| **A11y queries pass** | the query builder (`query(role).named(..)`), `press` through the AccessKit default action, `find_all`/`walk` |
| **Pass-system overhead < 15% on 10k-node churn** | the §7.2 kill criterion as a test (`pass_overhead_stays_under_the_15_percent_gate_on_10k_node_churn`) + `benches/pass_overhead.rs` |
| **W1 ≥ 2× vs full relayout (measured: ~14×)** | the M2 kill criterion as a test (`alacer-layout/tests/w1_gate.rs`) + the `w1_churn` criterion bench — 10k nodes, 10%/frame style churn |
| **Journal replay paths cached** | `FrameReport.scene` is an `Arc<SceneSnapshot>`: idle and replay-only frames share the last materialization; the paint order is cached between structural changes (`replay_frames_share_the_cached_scene_allocation`) |
| **Torin-style root candidate** | the closest common ancestor of the dirty set bounds each relayout; definite-sized candidates compute standalone, auto-sized ones under their arranger, with the upward walk when a box changes (ADR-0008) |
| **Region-bounded publish** | `publish_bounds` walks only the recomputed region; bounds outside it stay valid by construction |
| **The ten-pass contract, as code** | `PassKind` (§7.2), per-pass tracing spans, the frame ledger, capture→bubble routing (§10.1), focus passes over the `FOCUS` currency |
| **`rsx!` proc-macro** | `alacer-macro` (ADR-0000's M1 deferral, landed): elements, interpolated text, `key:`, `for`, `if/else` |
| **View reconciliation** | `ViewRoot`: patch-in-place, subtree replace, keyed peel→map→LIS with `apply(old, ops) == new` asserted per update |
| **Counter renders; masks flow end-to-end** | `alacer-testing/tests/m0_exit_criteria.rs` |
| **Scrolling list renders** | same suite, wheel-driven, blessed snapshot |
| **One write batch ⇒ at most one layout+paint cycle** | pipeline-level and E2E tests assert exactly 1 (ADR-6 kill-switch) |
| **Idle contract: 0% CPU between interactions** | idle-frame tests assert every pass skips (A7) |
| **Translation-only scrolling: zero measure calls** | `m0_scrolling_list_translates_without_re_measuring` (§8.2) |
| **Zero-config a11y: required role/label at the type level** | `Widget` trait; the CI walker fails unnamed inputs (ADR-5) |
| **Benchmarked** | criterion suites per crate; `cargo bench` (A6) |
| **TreeUpdate p99 < 0.5 ms @ 1k nodes** | the M3 kill criterion as a test (`alacer-testing/tests/m3_exit_criteria.rs`) — incremental `A11yDelta` → `TreeUpdate` production measured over 200 dirty-node-only frames |
| **A11y deltas: removals + focus + values** | `A11yDelta` (updated/removed/focus/initializes); dead widgets surface exactly once; focus rides `TreeUpdate::focus`; editables expose `value()` |
| **Partial paint: the damage ladder** | `FrameReport.damage` — NONE skips the present, ≤3 least-growth-merged regions repaint scissored, FULL on structural frames or ≥75% coverage (`alacer-core::damage`, §9.3) |
| **Partial == full, pixel-exact (Q8)** | the equivalence fuzz: over clicks, typing, CJK IME composition and surrogate-pair commits, the accumulated partial framebuffer equals a fresh full render after every step — both backends implement identical damage semantics |
| **IME contract: UTF-16 end to end** | key/IME routing to the focused widget, Tab cycling, inline preedit + commit via `TextInput`; winit's byte-wise preedit cursor converts at the platform seam (`alacer-platform::keys`); mid-surrogate offsets floor, never slice |
| **`TextInput` widget** | the IME contract's reference implementation: caret editing, secure mode, focus ring, a11y value |
| **Hot-patch contract: state intact, scoped, one frame** | the M4 kill-switch as tests (`alacer-testing/tests/m4_exit_criteria.rs`): a patched generation renders its new values with the old state; node identity stable; exactly one layout+paint cycle per patch; 10-generation churn with interleaved mutations stays consistent; untouched subtrees stay out of the patch frame |
| **`subsecond` wiring** | the facade's `hot-reload` feature: generations wrapped in `subsecond::call`, `register_handler` → patch channel → one scoped re-render (`crates/alacer/tests/hot_reload_subsecond.rs` pins the headless passthrough) |
| **Criterion baselines published as contracts (M5)** | `benchmarks/contracts.json` + `benchmarks/baselines/baselines.json`: all 43 criterion cases published with their environment; the W1 ≥ 2× gate as a measured budget contract (14.37× at republication); a regression gate per case, scoped same-host via the recorded host fingerprint; the uncontracted-benchmark rule fails a bench without a published gate |
| **The perf dashboard (M5)** | `benchmarks/dashboard/index.html` — self-generated, dependency-free; budgets with measured ratios and status, the full baseline table with delta bars, the six test-enforced gates with their enforcement sites |
| **The examples gallery (M5)** | `docs/examples.md` catalogs every example with its contract; `gallery.rs` is the one-window widget tour (`cargo run -p alacer --example gallery`) |
| **Migration guides (M5)** | `docs/migration/from-egui.md` + `docs/migration/from-iced.md` — the same required section set (mental model → interop limits), real API on both sides, the version-scope honesty note; enforced by the exit suite |
| **Causality inspector** | `alacer-inspect`: the `inspect` seam (edge activations, provenance writes, inputs, frame summaries with the pass ledger + mask heatmap), bounded ring buffers with tested eviction, the documented WS protocol (`docs/inspector-protocol.md`, version 1), and the embedded single-file client — the five §13.3 questions answered from recorded truth |
| **The benchmark canon: todo + data-table (M6)** | `Checkbox` and the `Table`/`TableRow`/`TableCell`/`ColumnHeader` family with a *navigable* AccessKit grid (counts + zero-based positions); `alacer_widgets::canon::{todo_app, data_table_app}` — the same builders behind the `todo`/`data_table` examples and the headless exit suite; §1's counter/todo/data-table canon complete |
| **W2/W3/W5 measured as contracts (M6)** | W2 text-dense relayout under ±5% width jitter, W3 the animation frame through the real pipeline + damaged raster, W5 10k-signal/1k-memo batched churn — each a published baseline with a same-host regression gate; the W5 scheduling contract test-enforced at graph scale (one batch ⇒ one rerun, no memo glitches) |
| **No re-wrap oscillation (M6)** | the §8.3 integer wrap rounding adopted in the layout host; `alacer-layout/tests/w2_gate.rs` (integrality, sub-pixel stability, cache-hit-on-revisit) |
| **Record/replay: `.alc` files (M6)** | `alacer-testing::recording` — versioned JSON with semantic steps (press by role + name, typed text, keys, wheel, click); `Session` records + executes in one pass; the committed canonical recording (`crates/alacer-testing/recordings/todo.alc`) replays green in the M6 suite; re-bless with `ALACER_RECORD=1` |

## Quick start

```sh
cargo run -p alacer --example counter          # wgpu window
ALACER_SOFTWARE=1 cargo run -p alacer --example counter   # softbuffer window
cargo run -p alacer --example scrolling_list
cargo run -p alacer --example text_input      # the M3 IME contract live
cargo run -p alacer --example hot_reload      # the M4 hot-reload demo (state intact across patches)
cargo run -p alacer --features hot-reload --example hot_reload
#                                              # … the same demo accepting `dx serve --hotpatch`
cargo run -p alacer --example inspector_demo  # the M4 Causality inspector (open the printed URL)
cargo run -p alacer --example gallery         # the M5 widget gallery (one window, every widget)
cargo run -p alacer --example todo            # the M6 todo canon (the §5 shape, live)
cargo run -p alacer --example data_table      # the M6 data-table canon (a navigable grid)
cargo test --workspace                        # headless: real pipeline + software raster (A4)
cargo bench                                   # criterion suites (A6)
ALACER_RECORD=1 cargo test -p alacer-testing  # re-bless the committed .alc recording (M6)
python3 scripts/collect_baseline.py           # publish the baseline after benching (M5)
python3 scripts/check_perf_contracts.py       # enforce budgets + same-host regressions (M5)
python3 scripts/perf_dashboard.py             # regenerate benchmarks/dashboard/index.html (M5)
```

Headless machines (and CI) run the **same pipeline** through the software backend — no mock renderer, no mock event loop (axiom A4).

## The M0 design, in five points

1. **One currency of change (A1).** Every consumer — layout, paint, accessibility — reads the same per-node `Change { STRUCTURE | LAYOUT | PAINT | A11Y | FOCUS }` masks with a normative ancestor-propagation table. Effects never mutate widgets; they queue masks (the flux bridge, §5.2).
2. **The scheduling contract (ADR-6).** `reactive_graph` `ImmediateEffect` + `batch()`: N synchronous writes inside one batch coalesce into exactly one effect rerun, hence one frame. This is the falsifiable kill-switch, and it is test-enforced at both pipeline and E2E level.
3. **The persistent Taffy tree (ADR-3, §8).** The tree is created once and never cleared; `mark_dirty` + Taffy's per-node caches bound recomputation to the dirty path. Scrolling composes as *translation-only* updates — zero measure calls per scroll tick.
4. **One scene journal, two honest backends (A8, ADR-2).** The paint pass records per-node layers; clean subtrees replay from the journal. wgpu rasterizes quads as rounded-rect SDFs and glyphs from a CPU coverage atlas; tiny-skia + softbuffer blends the *same* bitmaps headlessly.
5. **Accessibility is a type-level contract (ADR-5, §11).** `role()` and `label()` are *required* `Widget` methods; AccessKit node ids are the id-path hashes (one identity with hit-testing, §6.3); updates are dirty-node-only; tests query the a11y tree — reachable, named, actionable.

## The M3 design, in three points (ADR-0009)

1. **The a11y contract is a delta (§11.1).** Pass 9 produces an `A11yDelta` — updated ids, removed ids, focus, initialization — and the sink turns it into an incremental `TreeUpdate`. The snapshot travels as an `Arc` (built once, never deep-cloned per frame); editables expose `value()`; the exit gate measures incremental TreeUpdate p99 < 0.5 ms at 1k nodes (measured: µs-scale).
2. **Partial paint with an escape hatch (§9.3).** The paint pass merges every repainted node's old journal extent and new primitive extent into at most three regions (least-growth merging, clean-room from documented Slint behavior), with `NONE` (skip the present) and `FULL` (structural frames, ≥75% coverage) as the ladder's ends. Both backends implement identical damage semantics — software clears-and-redraws regions on a persistent framebuffer, wgpu `LoadOp::Load`s an offscreen frame texture with scissor rects — and the equivalence fuzz pins them pixel-equal to full renders over a real interaction script (Q8).
3. **The IME contract counts UTF-16 (§10.3).** Keys and IME events route to the focused widget only; Tab cycles; the caret, preedit cursors, and commits all count UTF-16 code units with mid-surrogate offsets flooring left. The platform seam converts winit's byte-wise preedit cursor; `TextInput` is the reference widget; the pipeline derives the frame's `ImeRequest` (purpose + caret anchor) for the windowing system.

## The M4 design, in three points (ADR-0010)

1. **The hot-reload contract is a generation swap (§13.2).** All UI-adjacent state lives outside the patched code — signals in the flux's reactive owner, dynamic values in `RenderBinding` slots, retained widget state in the arena — so a patch swaps *code*, never *state*. The facade's `hot-reload` feature wraps the view generation in `subsecond::call` (the jump-table detour point) and registers a patch handler; the loop answers every patch with exactly one `HotView::regenerate`: the reconciler patches in place and queues masks — one layout+paint cycle, scoped, nothing rebuilt. The CI hot-patch suite pins the contract headlessly; `cargo run --features hot-reload --example hot_reload` under `dx serve --hotpatch` is the sub-second demo.
2. **The Causality inspector answers *why*, not just *what* (§13.3).** `alacer-core::inspect` is the opt-in seam: the pipeline forwards frame summaries (pass ledger, damage, the dirty-node mask heatmap), the flux records edge activations (the masks each signal write queued) and provenance-tagged writes. `alacer-inspect` records them into bounded rings and streams them over a documented WebSocket protocol to the embedded single-file client. Without an observer and with capture off, the default paths pay a branch.
3. **Publish readiness is kept verifiable.** Every manifest carries the crates.io metadata set, the workspace ships as 0.4.0, and the CHANGELOG documents the release (ADR-0010 §3).

## The M5 design, in three points (ADR-0011)

1. **Baselines are contracts, not screenshots of hope.** `scripts/collect_baseline.py` distills every criterion run into `benchmarks/baselines/baselines.json` — all 43 cases with median/mean/MAD/sample count and the environment that produced them (commit, toolchain, host fingerprint). `benchmarks/contracts.json` gates them: the architecture's falsifiable gates as **ratio budgets** (host-independent, enforced everywhere — W1 ≥ 2× measured 14.37× at republication) and one **same-host regression gate** per case (fresh median ≤ published × 1.25 on a matching host fingerprint; foreign hosts skip, pinned hardware enforces with `--strict-host`). `scripts/check_perf_contracts.py` enforces the set and fails on an uncontracted benchmark, so the corpus cannot grow unmeasured.
2. **The perf dashboard publishes the regime.** `scripts/perf_dashboard.py` renders `benchmarks/dashboard/index.html` — one self-contained file, no external assets, no JavaScript: the measured environment, the budget contracts with ratios and PASS/FAIL, the full baseline table with delta bars, and the six test-enforced gates (pass overhead < 15%, TreeUpdate p99 < 0.5 ms, the patch frame budget, zero-measure scrolling, W2 no-re-wrap oscillation, W5 one-rerun-per-batch) each naming its enforcement site. CI re-generates it on main and uploads it as an artifact.
3. **The gallery and the guides are documentation with a regression net.** `gallery.rs` tours the widget vocabulary through the canonical write seam (signals in, masks out, one batch one frame); `docs/examples.md` catalogs every example with its contract; `docs/migration/from-egui.md` and `from-iced.md` map the two audited reference frameworks section-for-section — with an honesty header, real API on both sides, and the interop limits stated plainly. The M5 exit suite (`alacer-testing/tests/m5_exit_criteria.rs`) enforces all of it: baseline schema + environment, the contracts ↔ baseline bijection at the pinned corpus size, budgets holding on the published data, dashboard coverage, catalog coverage, and the guides' required sections.

## The M6 design, in three points (ADR-0012)

1. **The canon completes the definition of done.** The `Checkbox` toggles through click, Space (a first-class key now), and the AccessKit default action, and its checked state rides the neutral snapshot as `ToggleState` → AccessKit's `Toggled`. The data-table family (`Table`/`TableRow`/`TableCell`/`ColumnHeader`) maps onto AccessKit's table roles *with geometry* — row/column counts and zero-based cell positions from the neutral `TableSlot` — so a screen reader navigates a grid, not a pile of roles. The todo and data-table apps live once, in `alacer_widgets::canon`: the examples assemble them behind a window (the patch channel carries the one-wake ⇒ one-scoped-re-render path), and the M6 exit suite drives the same builders headlessly through the a11y queries.
2. **The remaining shared workloads are measured, not promised.** W2 (1k paragraphs, ±5% width jitter), W3 (the animation frame through the real pipeline and the damaged software raster), and W5 (10k signals, 1k memos, batched writes) join the M5 contract regime: published baselines, per-case regression gates, and the corpus pinned at 43 in both suites. The W2 gate is *stability*: the §8.3 integer wrap rounding floors wrap widths to whole pixels (egui's re-wrap-oscillation lesson, adopted as ALGO) and `w2_gate.rs` proves sub-pixel jitter cannot flip a line break. The W5 gate is the ADR-6 contract at graph scale: one write batch ⇒ one effect rerun, memos never glitch.
3. **Bug reports are regression tests.** `alacer-testing::recording` defines the `.alc` format — versioned JSON, semantic steps (`press` by role + name, not pixels) — with `Session` recording and executing in one pass and `replay`/`replay_with` running a committed file back through the real harness. The committed canonical recording of the todo canon replays green in the M6 suite; `ALACER_RECORD=1` re-blesses it the way `ALACER_BLESS=1` re-blesses snapshots.

## The crate DAG (ADR-9)

```text
alacer (facade)
┌──────────────┼──────────────┐
alacer-testing alacer-platform examples
│              │
alacer-widgets ── alacer-view
│              │
alacer-layout alacer-text alacer-render alacer-a11y
└──────────┴──────┬──────┴───────────┘
alacer-macro         alacer-core
(rsx!)               (Change masks · arena · pass system · flux bridge)
        alacer-inspect
        (causal capture · WS protocol · inspector client)
```

Layering is a DAG: no upward dependencies, no cycles (the build system makes cycles impossible; `scripts/check_dag.py` enforces the layer order in CI). Public API surface is confined to `alacer` + leaf crates.

## Repository map

| Path | What |
|---|---|
| `crates/alacer-core` | masks, identity, arena, widget trait, flux bridge, pass system, damage model, UTF-16 seam |
| `crates/alacer-layout` | persistent Taffy host, root-candidate relayout, LRU measure cache, style mapping |
| `crates/alacer-text` | parley facade, font registry, zeno rasterizer, evicting caches |
| `crates/alacer-render` | scene journal, software rasterizer (full + damage-scissored partial), batching |
| `crates/alacer-a11y` | AccessKit sink, incremental `TreeUpdates` |
| `crates/alacer-view` | keyed reconciliation (peel → key map → LIS) |
| `crates/alacer-widgets` | label, button, text input, checkbox, table family, column/row/spacer, list, panel, the canon apps, the default element registry |
| `crates/alacer-platform` | winit loop, wgpu backend (offscreen frame + scissored damage), softbuffer present (persistent framebuffer), key/IME translation, coalescing, the patch channel |
| `crates/alacer-inspect` | the Causality inspector: causal recorder, RFC 6455 server, embedded client, the documented WS protocol |
| `crates/alacer-testing` | headless harness, a11y queries, snapshots, record/replay (`.alc`), M0 + M1 + M3 + M4 + M5 + M6 suites |
| `crates/alacer-macro` | the `rsx!` view DSL compiler (proc-macro leaf) |
| `crates/alacer` | facade: prelude, `rsx!` re-export, `mount_view`, window runner |
| `benchmarks/` | the M5 perf-contract regime: published baselines, the contract set, the generated dashboard |
| `docs/` | architecture document, ADRs, the examples catalog, the migration guides |
| `scripts/` | CI gates: the DAG check, the baseline collector, the contract checker, the dashboard generator |
| `.github/workflows/` | CI + release automation |

## Engineering discipline

- **Lint regime**: clippy `pedantic` **denied** workspace-wide, plus `missing_docs` and `unsafe_code = forbid`. There is no `#[allow]` anywhere; the documented cast seams (`alacer-core/src/num.rs`, the `rsx!` number narrowing in `alacer-view`) carry the single `#[expect]`s with reasons.
- **TDD**: every module grew tests first; **330 tests** across the workspace (unit + integration + doctest), all green locally and in CI.
- **Benchmarks as contracts (A6)**: criterion suites for the flux batch drain, mask propagation, journal replay, incremental-vs-full relayout, the W1/W2/W3/W5 shared workloads, text layout, and the §7.2 pass-overhead churn gate.
- **CI/CD**: fmt + clippy + tests + doc + `cargo-deny` + DAG check on push/PR; the perf-contracts job enforces the published budgets and republishes the dashboard; `git-cliff`-generated changelogs; GitHub Releases on tags (`v*`).
