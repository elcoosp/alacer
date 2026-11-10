# Alacer — System Architecture

> **Alacer** — Latin for *swift, eager, lively*; the etymon of English *alacrity*: **brisk and cheerful readiness**.
> A retained-mode Rust UI framework whose reactivity, layout, paint, and accessibility consume **one shared invalidation currency** — engineered for best-in-class performance *and* best-in-class developer experience, with zero-config accessibility.

| | |
|---|---|
| **Status** | `architecture/draft-1` — ratified against the 2026-11-05 ecosystem audit |
| **Date** | 2026-11-05 (Europe/Paris) |
| **Evidence base** | `rust-ui-framework-audit-2026-11-05.zip`: 16 repositories verified, cloned, and pinned at exact SHAs; ~705 unique `file:line` citations, every path verified to exist in its pinned clone. Audit codename "Meridian" is superseded by **Alacer** (naming rationale in §2). |
| **Audience** | Core engineers building Alacer; reviewers evaluating its claims; future maintainers |
| **Companion documents** | `VISION.md`, `adr/ADR-1…7`, `audit/INDEX.md`, `audit/MATRIX.md`, `audit/STEAL.md`, `audit/INNOVATE.md`, `audit/PRIMITIVES.md`, `audit/LICENSES.md`, `audit/BENCHMARKS.md` (bundled in the audit ZIP) |

**Citation legend.** Claims about other frameworks are cited to the audited source tree at the pinned SHA, e.g. `crates/gpui/src/window.rs:3743 @ 056a44d` (full SHAs in `audit/INDEX.md`). Cross-framework synthesis lives in `audit/MATRIX.md`; per-framework reports in `audit/<framework>/REPORT.md`. Where the audit could not verify something, the claim is marked **UNVERIFIED** — this document inherits the audit's zero-fabrication policy. Design budgets for Alacer itself are labeled **[budget]** — they are commitments to be CI-enforced, not measurements.

---

## Contents

1. [Thesis](#1-thesis) — the quadrant, in one sentence and four numbers
2. [Name](#2-name) — why Alacer
3. [Design axioms](#3-design-axioms) — eight falsifiable commitments
4. [System overview](#4-system-overview) — layers, crates, and the life of a frame
5. [State & flux — the reactive core](#5-state--flux--the-reactive-core)
6. [Structure — views, reconciliation, identity](#6-structure--views-reconciliation-identity)
7. [Widget core — arena and the pass contract](#7-widget-core--arena-and-the-pass-contract)
8. [Layout — persistent Taffy + torin-style incrementality](#8-layout--persistent-taffy--torin-style-incrementality)
9. [Rendering — one scene journal, two honest backends](#9-rendering--one-scene-journal-two-honest-backends)
10. [Input & focus](#10-input--focus)
11. [Accessibility — a type-level contract](#11-accessibility--a-type-level-contract)
12. [Platform integration](#12-platform-integration)
13. [DX architecture](#13-dx-architecture) — the best-in-class developer experience plan
14. [Performance architecture](#14-performance-architecture) — the best-in-class performance plan
15. [What Alacer will not build](#15-what-alacer-will-not-build)
16. [Moats](#16-moats) — three defensible differentiators
17. [ADR register](#17-adr-register) — 33 decision records
18. [Roadmap & kill criteria](#18-roadmap--kill-criteria)
19. [Risks & open questions](#19-risks--open-questions)
20. [Provenance, licensing, and honesty](#20-provenance-licensing-and-honesty)
21. [Glossary](#21-glossary)
22. [Appendix: evidence index](#22-appendix-evidence-index)

---

## 1. Thesis

> **Alacer is a retained-mode Rust UI framework in which reactivity, layout, paint, and accessibility consume one shared invalidation currency — delivering Zed-class responsiveness, Leptos-class reactivity ergonomics, and zero-config accessibility: the quadrant no audited framework occupies.**

The audit of 16 Rust UI frameworks (2026-11-05, pinned SHAs) produced four numbers that justify this project's existence:

| # | Verified finding | Consequence |
|---|---|---|
| **4** | Fine-grained reactivity was independently re-implemented four times (Leptos `reactive_graph`, Dioxus signals, Freya 0.4 core, Floem's crate — `audit/MATRIX.md` §3). One of them is MIT-licensed, framework-independent, and complexity-tested. | Writing a fifth reactive core is negative-value work. We adopt, extend, and upstream. |
| **5** | Five custom layout engines exist (Masonry, torin, morphorm, Slint's water-filling solver, Makepad's Walk) while Taffy — which won the flexbox/grid API war — is cleared and rebuilt *every frame* by the most layout-serious desktop app in the ecosystem (`crates/gpui/src/window.rs:3743` @ `056a44d`). | The industry's missing piece is not another solver; it is **incremental relayout over a persistent tree**. We build exactly that, once. |
| **2** | Two of the most popular "hobby-scale" stacks ship **zero** accessibility (Iced and Floem, grep-verified; iced's a11y issue #282 has been open since 2020). | Zero-config accessibility is the single largest verified differentiation slot in the ecosystem. |
| **1** | Dioxus's `subsecond` hot-patcher became ecosystem infrastructure: Dioxus, GPUI, Leptos, and Freya all hot-reload through it. | State-preserving hot reload is table stakes we adopt for free — and make CI-grade. |

**The market failure Alacer exploits:** every audited stack optimizes one vertex of the triangle *performance × ergonomics × accessibility* and treats the others as a tax. GPUI buys editor-class speed with per-frame rebuilds and a Zed-sized platform investment; egui buys unmatched tool-DX with immediate-mode ceilings (a11y nodes built lazily during the UI walk, unbounded glyph/state caches — `crates/epaint/src/text/glyph_atlas.rs:381-395`, `crates/egui/src/util/id_type_map.rs:573-601` @ `fc9d66d`); Iced and Floem buy approachability by dropping a11y entirely. Nobody composes. Composition is the product.

**Definition of done for v1** (traceable to §18): a widget set covering the counter/todo/data-table canon; benchmarks where Alacer beats both the immediate-mode DX baseline (egui) and the best audited retained stack on the same hardware in at least one shared workload; an accessibility CI gate that fails builds on unnamed widgets; and a public, reproducible benchmark harness (spec: `audit/BENCHMARKS.md` §5).

---

## 2. Name

**Alacer** (*ah-LAH-ker*), Latin adjective: *swift, quick, lively, eager*. It is the root of English **alacrity** — "brisk and cheerful readiness" — which is precisely the product contract: **swift** (performance) and **cheerful** (developer experience) are one property, not a trade-off.

Selection was evidence-driven, in the audit's spirit:

- **crates.io** (checked live, 2026-11-05): `alacer` is **unregistered**. Nearly every other candidate was taken and rejected on collision grounds: `ferrite` (an image-loading crate), `keel` (an active "Keel Engine"), `pronghorn`, `iridium`, `flywheel`, `stator`, `escapement`, `carina`, `fovea`, `saccade`, `tessera`, `vivace`, `legato`, `cairn`, `heddle` — all occupied.
- **GitHub** (checked live): 21 repos named `alacer`, none a software framework (max 7★, no Rust project).
- **Crate family:** `alacer`, `alacer-core`, `alacer-macro`, `alacer-view`, `alacer-layout`, `alacer-render`, `alacer-text`, `alacer-a11y`, `alacer-widgets`, `alacer-platform`, `alacer-inspect`, `alacer-testing` (availability to be re-verified at first publish — `UNVERIFIED` until done).
- The audit's internal codename "Meridian" is retired; all ADRs and audit references carry over unchanged.

---

## 3. Design axioms

Each axiom is a commitment with a falsifiable consequence. "Evidence" is the audited mechanism that motivates it.

### A1 — One currency of change
Every cross-cutting consumer — layout, paint, accessibility, devtools — consumes the same per-node `Change` masks. No subsystem invents private invalidation.
*Evidence of the wound:* GPUI clears its entire Taffy tree every frame and hides the cost behind entity caches (`crates/gpui/src/window.rs:3743`, `taffy.rs:55-60`); Freya 0.4 regressed to full repaint in its rewrite (`// TODO: Use incremental rendering`, `crates/freya-core/src/render_pipeline.rs:58`); Iced's damage-based redraw exists only on its software path — its wgpu `present()` accepts no damage parameter (`wgpu/src/window/compositor.rs:230-264` vs `tiny_skia/src/window/compositor.rs:149-185`).
*Consequence:* a feature that cannot express itself as a mask is rejected or redesigned. Devtools read the same masks (§13.3).

### A2 — Steal proven cores; own only the composition
We write **no subsystem for which a maintained, permissively-licensed, production-proven crate exists.** Adopted: `reactive_graph` (reactivity), `taffy` (flex/grid), torin's algorithm (incremental relayout), `parley`/`fontique`/`swash` (text), `accesskit`, `winit`, `wgpu`, `subsecond` (hot reload). Ported-as-algorithm: Slint's partial renderer. Owned: the **composition** — masks, passes, journals, and the DX/tooling above them.
*Evidence:* four reactive reimplementations; five layout engines; Masonry's pass contract is reusable as a design; `audit/STEAL.md` classifies all 20 steals (IDEA/ALGO/CODE) with licenses.

### A3 — Accessibility is a type-level invariant, not a feature
A widget that cannot be named cannot be compiled. Role and label source are required trait methods (Masonry precedent: `masonry_core/src/core/widget.rs:424-435` @ `dc49687`); test queries route through the accessibility tree (egui kittest precedent); a debug-mode CI walker fails builds on unnamed widgets.
*Consequence:* the a11y gap map (Iced/Floem/Blinc = zero) is structurally impossible in Alacer, by construction rather than by discipline.

### A4 — The test harness mirrors production
Headless testing runs the **real** pass pipeline with a software rasterizer — the design that four teams independently converged on (masonry_testing, freya-testing, egui kittest, floem headless; `audit/STEAL.md` #9). No mock event loop, no mock renderer for tests.
*Consequence:* every feature must be headless-testable before merge; GPU-less CI is the default, not a downgrade.

### A5 — Never fork core infrastructure
Iced pins three in-org forks by rev (winit, cosmic-text, cryoglyph — `Cargo.toml:206,207,249` @ `8ff48c4`); Floem rides a Lapce winit fork (`Cargo.toml:65` @ `1351ffb`). Both pay documented supply-chain and merge pain. KAS, by contrast, adopted a winit platform feature upstream within days (`kas-core/src/data_transfer.rs:4-7` @ `98d1419`).
*Consequence:* patches go upstream; a fork requires an ADR with a sunset plan (§17, ADR-1/ADR-9).

### A6 — Budgets are contracts, not aspirations
Floem commits event-path criterion baselines *inside a normative spec* — ~2.25 ns cached hit-test, ~390-410 ns pointer dispatch (`src/event/README.md:334-412` @ `1351ffb`). We extend the practice to build time, binary size, memory, and frame phases: budgets live in CI, regressions are red.
*Consequence:* performance debates end with a diff to a baseline file, not an opinion.

### A7 — Sustained speed, not sprint speed
Cheetahs win benchmarks; the 10,000th frame wins users. Immediate mode is a treadmill — correct and simple, but it re-pays full cost every frame; Alacer is a **flywheel** — work done once (measurement, shaping, journal entries, a11y nodes) is retained and re-used until invalidated.
*Evidence:* egui's whole-UI repaint each frame vs GPUI's `Entity::cached` subtree replay making per-frame element-tree rebuilds viable at editor scale (`crates/gpui/src/view.rs:552-568` @ `056a44d`); Masonry's debug-mode purity panics that catch cache-coherence bugs structurally (`masonry_core/src/passes/layout.rs:72-99`).
*Consequence:* idle CPU = 0% (`ControlFlow::Wait`); the warm paint path for a static UI performs **zero heap allocations** [budget, CI-enforced via dhat].

### A8 — The software path is first-class
One scene journal, consumed by both the GPU backend and the software backend. Slint's partial renderer already wraps both its Skia and software renderers — proof the pattern works (`internal/core/partial_renderer.rs:273` @ `0b1c00d`); Iced's GPU/software damage asymmetry is the cautionary tale.
*Consequence:* CI, screenshots, headless tests, and low-end machines exercise the identical scene representation the GPU sees.

---

## 4. System overview

### 4.1 Layers

```mermaid
flowchart TB
    subgraph APP["Application"]
        UI["rsx! views + Signal/Memo/Effect state"]
    end
    subgraph FLUX["flux — reactivity (adopted: reactive_graph)"]
        G["dependency graph · push-Check / pull-Update"]
        E["effect queue · batch() · owners"]
    end
    subgraph CUR["invalidation currency — the heart"]
        M["Change { LAYOUT · PAINT · A11Y } per-node masks"]
    end
    subgraph CORE["core — retained widget tree (owned)"]
        A["tree arena · WidgetIds · passes"]
        P["event → focus → mutate → style → layout → compose → paint → a11y"]
    end
    subgraph SURF["surface"]
        L["layout: persistent Taffy + torin-style deltas"]
        S["scene journal (append-only paint ops)"]
        R["renderers: wgpu | tiny-skia+softbuffer"]
        T["text: parley + fontique + swash + GPU glyph atlas"]
        K["accesskit TreeUpdates"]
        W["winit · muda · rfd · arboard · tray-icon"]
    end
    UI -->|"write batch"| G
    G --> E -->|"masks, never direct mutation"| M
    M --> P
    M --> L
    P --> S
    S --> R
    P --> T
    M --> K
    W -->|"window events"| P
```

ASCII fallback:

```
┌────────────────────────────────────────────────────────────────┐
│ application: rsx! views · Signal/Memo/Effect · owners          │
└───────────────┬────────────────────────────────────────────────┘
                │ write batch (coalesced)
┌───────────────▼────────────────────────────────────────────────┐
│ flux: reactive_graph — push-Check/pull-Update, effect queue    │
└───────────────┬────────────────────────────────────────────────┘
                │ Change{Layout,Paint,A11y} masks (merged upward)
┌───────────────▼──────────────┐   ┌─────────────────────────────┐
│ core: arena + pass system    │   │ layout: persistent Taffy    │
│ event→focus→mutate→style→    │◄──┤ + torin-style incremental   │
│ layout→compose→paint→a11y    │   │ deltas, LRU measure cache   │
└───────┬───────────┬──────────┘   └─────────────────────────────┘
        │           │
        ▼           ▼
┌───────────────────┐ ┌───────────────────────────────────────────┐
│ scene journal     │ │ accessibility: accesskit incremental      │
│ (append-only ops) │ │ TreeUpdates from dirty nodes              │
└─────┬───────┬─────┘ └───────────────────────────────────────────┘
      │       │
      ▼       ▼
┌───────────┐ ┌──────────────┐   ┌───────────────────────────────┐
│ wgpu      │ │ tiny-skia +  │   │ platform: winit · muda · rfd  │
│ (primary) │ │ softbuffer   │   │ arboard · tray-icon           │
└───────────┘ └──────────────┘   └───────────────────────────────┘
```

### 4.2 Crate workspace

Layering is a DAG, enforced in CI (no upward deps, no cycles, facade-only re-exports). Public API surface is confined to `alacer` + leaf crates; internals are free to change (ADR-9).

| Crate | Responsibility | Key dependencies (all MIT/Apache) | LOC budget [budget] |
|---|---|---|---|
| `alacer` | Facade: prelude, re-exports, docs, feature flags; hot-reload runtime (the `subsecond` integration + patch contract live here, not in a separate crate) | all below, `subsecond` (feature-gated) | ≤ 3k |
| `alacer-macro` | The `rsx!` view-DSL compiler → `alacer_view::ViewNode` builder trees | `proc-macro2`, `quote`, `syn` | ≤ 2k |
| `alacer-inspect` | Causality inspector: causal recorder, WebSocket wire protocol, embedded client (feature-gated at the facade) | `alacer-core`, serde, sha1, base64 | ≤ 15k |
| `alacer-testing` | Headless harness, a11y queries, snapshots, record/replay, fuzz targets | `alacer` + tiny-skia | ≤ 20k |
| `alacer-widgets` | Widget library (button, label, text input, list, table, scroll…) | `alacer-core` | ≤ 30k |
| `alacer-platform` | winit integration, frame-batched input queueing (M8: the coalescer lives in core; the platform pushes and never drains), window present over wgpu/softbuffer | `winit`, `wgpu`, `softbuffer`, `accesskit_winit` | ≤ 15k |
| `alacer-a11y` | AccessKit contract, incremental TreeUpdates, id-path NodeIds | `accesskit` | ≤ 8k |
| `alacer-text` | parley/fontique/swash façade, shaping caches, glyph atlas | `parley`, `fontique`, `swash` | ≤ 12k |
| `alacer-render` | Scene journal, wgpu + tiny-skia backends, damage/partial paint | `wgpu`, `tiny-skia`, `softbuffer` | ≤ 35k |
| `alacer-layout` | Persistent Taffy host, torin-style incremental engine, measure cache | `taffy`, `euclid` | ≤ 10k |
| `alacer-view` | rsx! proc-macro, view tree, keyed reconciliation, id-path hashing | `syn`, `quote`, `alacer-core` | ≤ 15k |
| `alacer-core` | Widget trait + arena + pass system + Change masks + focus/IME contract | `euclid`, `tracing` | ≤ 25k |

Dependency weight is a design goal: the audit's build-cost proxies show lockfiles of 599–1,576 unique crates for peer stacks (`audit/BENCHMARKS.md` §2). Alacer's own-code budget is ~190k LOC — deliberately larger than a toy, an order of magnitude smaller than Makepad's 1.85M-LOC vendoring monorepo — and its *direct* dependency list is the twelve crates above.

### 4.3 The life of a frame

Design budgets for a 120 Hz frame (8.33 ms) on the reference rig (§14.2), mid-size app (1k nodes, one region dirty) **[budget]**:

| Phase | Budget | Mechanism |
|---|---|---|
| Input coalescing | ≤ 0.3 ms | compatible-event merge, wheel accumulation (§10.2) |
| Flux: effect drain to fixpoint | ≤ 0.5 ms | push-Check/pull-Update; 1-bit coalescing; batch() |
| Reconciliation (view diff → mutations) | ≤ 1.0 ms | peel → key map → LIS; only dirty scopes |
| Layout (incremental) | ≤ 2.0 ms | common-ancestor re-measure + translation-only updates |
| Paint (journal append) | ≤ 2.5 ms | dirty subtrees only; cached subtrees replayed |
| Accessibility TreeUpdate | ≤ 0.5 ms (p99 @ 1k nodes) | dirty-node incremental updates |
| Present | remainder | damage-aware on both backends |

Idle contract: no timers, no polling — `ControlFlow::Wait`; 0% CPU between interactions [budget]. Violations of any phase budget are CI failures against committed criterion/tracing baselines (A6), not release-note apologies.

---

## 5. State & flux — the reactive core

### 5.1 Decision: adopt `reactive_graph`, extend, upstream

Alacer's reactivity is **Leptos's `reactive_graph` crate (MIT) as a dependency — not a vendored copy, not a re-implementation.** It is the only Rust reactivity engine that is (a) framework-independent, (b) runtime-agnostic via `any_spawner` (tokio, glib, wasm-bindgen, async-executor — `any_spawner/Cargo.toml:12-20, 33-39` @ `7d656a4`), and (c) complexity-tested (2,170 test LOC including amortized-regression tests, of 17,794 source LOC). Three other frameworks converged on the same design — Dioxus's signals/generational-box, Freya's 0.4 core, Floem's crate — independently re-deriving it (ADR-6). A fifth hand-rolled core would repeat the ecosystem's most expensive duplication.

Core semantics we inherit (all cited at `7d656a4`):

| Mechanism | What it buys us | Where verified |
|---|---|---|
| **push-Check / pull-Update** propagation | glitch suppression; recomputation only toward actual dirty sources | `reactive_graph/src/computed/inner.rs:69-79, 81-97, 106-124` |
| **Order-preserving `FxIndexSet` edges** | effects observe dependencies in registration order ("the inner effect assumes the outer has already run" — `shift_remove` preserves it) | `src/graph/sets.rs:1-13, 84-93` |
| **1-bit effect channel** (`AtomicBool` + `AtomicWaker`) | N synchronous signal writes coalesce into **one** wake/rerun | `src/channel.rs:17-21, 51-56, 61-77` |
| **WriteGuard notifies on drop** | notifications deferred until the mutable borrow ends — no torn reads | `src/signal/guards.rs:354-368` |
| **`batch()`** | nested-safe, ordered `IndexSet` replay at scope end | `src/effect/immediate.rs:150-183, 210-255` |
| **Owner tree** with amortized-O(1) pruning + regression tests; pause/resume cascades | leak-safe teardown; disposal order children → cleanups → arena | `src/owner.rs:327-351`, `:546-559`, `owner.rs:661-708` (tests); `owner.rs:476-515` (pause/resume); `src/owner.rs:533-543` (structure) |
| **`RenderEffect`** | synchronous first run *during rendering*; per-dynamic-slot render bindings; re-dirty on hot patch | `src/effect/render_effect.rs:22-37, 175-216` |

### 5.2 What Alacer adds (and gives upstream)

1. **Memo cycle detection.** Verified gap: `reactive_graph` has none (grep clean; only an `ImmediateEffect` recursion warning at `immediate.rs:352-354`). A memo cycle today misbehaves silently. We contribute a dev-only DFS-over-edges detector at memo creation — the same class of check Masonry applies to measure purity. *Kill-switch note (ADR-6):* if effect scheduling cannot drive paint batching within M0's budget, GPUI-style draw-level entity tracking (`crates/gpui/src/app.rs:1459-1466, 1468-1495` @ `056a44d`) becomes the documented fallback for paint, keeping `reactive_graph` for data.
2. **Borrow-location diagnostics.** Port Floem's `TrackedRefCell` (MIT): in debug builds, signal cells record the `Location` of every borrow, turning "already borrowed" panics into a two-location diff (`reactive/src/signal.rs:29-37, 118-120` @ `1351ffb`).
3. **The mask bridge (this is the composition).** Effects in Alacer do not mutate widgets. Every effect drain ends by applying `Change` masks to arena nodes; widget mutation happens only inside the pass system (§7). This is the single rule that makes Pillar 3 (§4.1) possible: *reactivity is upstream of invalidation, never tangled with it.*
4. **A scheduling contract, test-enforced:** *one write batch ⇒ at most one layout+paint cycle* (plus an animation-tick frame when springs are active). Freya is the counterexample we refuse to become: its memos/effects are async task loops with arrival-order scheduling and no cycle detection (`crates/freya-core/src/lifecycle/memo.rs:106-116`, `effect.rs:48-203` @ `0b13171`).

### 5.3 Public API surface (what users touch)

```rust
use alacer::prelude::*;

#[derive(Clone, PartialEq)]
struct Todo { done: bool, label: String }

fn app() -> View {
    let todos = use_signal(Vec::<Todo>::new);        // Signal<Vec<Todo>>
    let remaining = use_memo(move ||                 // Memo<usize>, tracked automatically
        todos.with(|t| t.iter().filter(|x| !x.done).count()));

    rsx! {
        input { placeholder: "Add a todo…",
                on_submit: move |text| todos.write().push(Todo { done: false, label: text }) }
        for (i, todo) in todos.read().iter().enumerate() {   // fine-grained: per-item bindings
            todo_row { key: i, todo, on_toggle: move |_| todos.write()[i].done ^= true }
        }
        footer { "{remaining} items left" }                  // re-renders this slot only
    }
}
```

Ergonomic targets, borrowed from the audited winners: `Copy` signal handles over generational arenas (Dioxus `packages/generational-box` semantics — thread-local arenas, generation counters, `Owner` teardown, `packages/generational-box/src/unsync.rs:38-46`, `entry.rs:31-59` @ `7dfd07e`); dynamic per-run dependency tracking (no manual `unwrap`-style declarations); hooks-rule violations caught with actionable panics (Freya's HOOKS_ERROR pattern, `lifecycle/base.rs:9-107` @ `0b13171`).

**Why not the alternatives (decided, with evidence):**
- *Elm-style whole-app re-run* (Masonry/Xilem): every message re-runs `app_logic` and diffs the whole view tree (`xilem/src/driver.rs:256-293` @ `dc49687`) — a tax that grows with app size, paid even when one label changes.
- *Immediate-mode no-reactivity* (egui): unmatched for tools, but it forces whole-UI repaint and structurally complicates a11y/damage (§9, §11).
- *Entities + draw-level tracking only* (GPUI): cheap and correct for redraws, but no memo-level data reuse; adopted as the paint-consumer fallback, not the core (ADR-6).

---

## 6. Structure — views, reconciliation, identity

### 6.1 View layer: ephemeral builders, retained widgets

Alacer separates what is *declared* from what *lives*:

- **View tree** — ephemeral, re-created for dirty scopes only: a typed tree produced by `rsx!` (proc-macro builders; no external DSL — §15).
- **Widget tree** — retained, arena-owned: created once, mutated in place; widgets are never re-created on state change (Xilem/Masonry bridge precedent: view diff emits `WidgetMut` mutations; `xilem_masonry/src/pod.rs:21-59`, `masonry_root.rs:45-71` @ `dc49687`).

This hybrid is the audited sweet spot: GPUI proves per-frame element trees need entity caches to survive editor scale (`crates/gpui/src/element.rs:10-14`, `view.rs:552-568`); Masonry proves retained widgets + ephemeral views give a11y and IME a structural home. Reconciliation cost is bounded by scoping: only scopes dirtied by the flux layer re-run their view closures (the Leptos insight — each dynamic slot is its own binding — applied to a widget world).

### 6.2 Keyed reconciliation: peel → key map → LIS

For collections, Alacer ports the diff pipeline Dioxus hardened (Apache/MIT, `packages/vdom/src/diff/iterator.rs:231-352` @ `7dfd07e`):

1. **Prefix/suffix peel** — match runs with equal keys at both ends (`iterator.rs:250-267`).
2. **Key map** — remaining nodes resolved through a hash map of keys (debug-assert: keys unique — `iterator.rs:241-254`).
3. **Longest-increasing-subsequence** — LIS members are "stable": they do not move; everything else is the minimal mutation set (`iterator.rs:271-346`). Floem's LCS-style keyed diff (`src/view/mod.rs:398-480` @ `1351ffb`) is the cross-check ALGO reference.

Anti-goal: Iced's default — index-based `TypeId`-tag diffing where a mismatch recreates whole subtree state (`core/src/widget/tree.rs:62-73` @ `8ff48c4`), with keying available only as opt-in `keyed_column` (`widget/src/keyed.rs:1-42`). Keying is the default in Alacer's list widgets; index-pairing remains an explicit, documented opt-out for append-mostly lists.

### 6.3 Identity: one hash, four consumers

Every element gets an **id path** from the view structure; each path prefix carries a pre-folded hash (GPUI design, `crates/gpui/src/element.rs:212-236` @ `056a44d`). That single hash value serves as:

| Consumer | Role of the hash |
|---|---|
| Element-state slot | persistence key across re-renders (GPUI `(GlobalElementId, TypeId)` pattern, `window.rs:1044`) |
| Hit-testing identity | the same node id reported by the spatial index (§10.1) |
| AccessKit `NodeId` | stable across frames without extra mapping tables (GPUI: `element.rs:253-255`) |
| Hot-reload template key | template signals keyed by `file:line:col` (Dioxus global-signal interning, `packages/signals/src/global/mod.rs:120-170` @ `7dfd07e`) |

One identity system for state, input, accessibility, and tooling kills an entire class of drift bugs — the a11y tree can never disagree with the hit-test tree about what a node *is*.

### 6.4 Correctness regime for the diff

Dioxus is the only audited project fuzzing its reconciler; we adopt the practice wholesale (IDEA+CODE): a mock-tree **oracle** (`packages/oracle/src/lib.rs:1-9`) + libFuzzer structured-mutation harness asserting *incremental result ≡ fresh rebuild* (`packages/fuzz/README.md:1-18`), plus a miri stress suite (`packages/vdom/tests/miri_stress.rs`). The same harness pattern later guards the invalidation engine (§8.4) and the scene journal (§9.2).

---

## 7. Widget core — arena and the pass contract

### 7.1 Arena

Widgets live in a `tree_arena`-style arena (Masonry's standalone `tree_arena` crate, Apache-2.0): safe nested-`Vec` nodes with a parents map as the **default**; an O(1) hash-map arena exists upstream but ships with 11 `unsafe` blocks and is self-described as "not yet fully tested" (`tree_arena/src/lib.rs:7-75`, `tree_arena_unsafe.rs:8` @ `dc49687`). Alacer ships safe-by-default; the fast arena lands only behind a feature flag *after* passing the fuzz equivalence suite (ADR-9). `WidgetId` = `NonZeroU64` from a global atomic counter, attributed on attach (`masonry_core/src/core/widget.rs:34-44`).

All tree mutation flows through guard types that **merge invalidation flags upward on Drop** (Masonry `WidgetMut` discipline, `masonry_core/src/core/widget_mut.rs:22-43`) — combined with GPUI's lease-and-panic re-entrancy protection (`entity_map.rs:164-177, 296-307` @ `056a44d`): mutating a widget mid-update is a panic with the two contending locations named.

### 7.2 The pass contract

Ten named passes, each explicit, ordered, and independently traceable (Masonry's pass system as contract — `masonry_core/src/doc/pass_system.md:26-41, 133-147` @ `dc49687`; ~3,600 pass LOC in Masonry validates the scale). Every cross-cutting concern becomes a testable, debuggable stage instead of scattering through callbacks:

| # | Pass | Responsibility | Flag consumed / produced |
|---|---|---|---|
| 1 | `event` | target widget via spatial index + z-order; capture → bubble along precomputed immutable paths (Floem's 400-line normative spec, `src/event/README.md:24-152, 154-242` @ `1351ffb`) | consumes input; emits masks |
| 2 | `update_focusable` | recompute focus candidates after tree changes | `Change::Structure` |
| 3 | `update_focus` | resolve active focus; emit platform IME area (Masonry sentinel pattern, `render_root.rs:47`) | focus diffs |
| 4 | `mutate` | drain queued `WidgetMut` closures; flags merge on Drop | `Change::*` |
| 5 | `style` | recalc with reason flags — Floem's Chromium-grade `StyleReasonFlags` (SELECTOR/ANIMATION/TRANSITION/INHERITED_CHANGE…, `src/style/recalc.rs:1-69` @ `1351ffb`) to fixpoint | `Change::Layout`/`Paint` |
| 6 | `layout` | measure/layout split (Masonry `LenReq` model, `layout/len_req.rs`); 10-slot LRU measurement cache per widget (`layout/measurement_cache.rs:7-94`); **debug purity panics** (`passes/layout.rs:72-99`) | `Change::Layout` → geometry |
| 7 | `compose` | scroll offsets, transforms; translation-only updates (torin `post_measure` pattern, §8.2) | geometry → final rects |
| 8 | `paint` | append scene ops to the journal; skip non-dirty subtrees; replay cached ranges | `Change::Paint` |
| 9 | `accessibility` | build incremental AccessKit updates from dirty nodes only (`passes/accessibility.rs:16-60` pattern) | `Change::A11y` |
| 10 | `action` | propagate typed `Action`s to ancestors, then to the app driver (`pass_system.md:99-103`) | side-channel, never masks |

*Pass-overhead kill criterion:* if pass-system bookkeeping exceeds 15% of frame time on the 10k-node churn workload (tracing spans, M1 gate), collapse to a 3-pass pipeline — the masks and the journal survive; the taxonomy shrinks.

---

## 8. Layout — persistent Taffy + torin-style incrementality

### 8.1 The decision (ADR-3)

- **Taffy stays** — flex + grid, multi-org maintained (GPUI pins `=0.15.0`; Floem 0.9.2; Slint delegates flexbox to taffy 0.15.1; blitz uses `stylo_taffy` — `audit/MATRIX.md` §4). Building a sixth engine is exactly the duplicated work A2 forbids.
- **The tree persists** — never cleared between frames. The anti-pattern is specific and verified: GPUI clears its whole `TaffyTree` after every draw (`window.rs:3743`, `taffy.rs:55-60`) and hides the cost behind entity caches — a tax on every general-purpose app that uses its stack.
- **Incrementality is ported from torin (MIT)** — Freya's 3.7k-LOC engine proves partial relayout works over an engine seam (`crates/torin/src/torin.rs:44-242`, `measure.rs:59-154` @ `0b13171`).
- **Masonry's measurement discipline is adopted as contract** — measure/layout split, per-widget LRU cache, debug purity panics.

### 8.2 The incremental algorithm (ported semantics, cited at `0b13171`)

1. **Dirty-reason map** — per-node `DirtyReason` (torin: `None | InnerLayout | Reorder` at `torin.rs:94-107`; Alacer extends with `Style` and `Text`).
2. **Invalidation walk** — `invalidate_with_reason` walks dependants and stops at ascendants that do not need re-measure (`torin.rs:197-242`).
3. **Root candidate** — the **closest common ancestor** of dirty nodes bounds the re-measure region (`torin.rs:44-92`, `measure()` at 289-330).
4. **Translation-only updates** — if only position changed (ancestor moved, size identical), `recursive_translate` repositions without re-measuring (`measure.rs:59-154`). This is the workhorse: scrolling a 10k-row list touches translations, not measurements.
5. **Text is a node concern** — text/font changes mark only the text node dirty in Taffy (Floem pattern: `taffy.mark_dirty(node)`, `src/views/label.rs:214-218` @ `1351ffb`), never restyling the tree.

### 8.3 Numeric discipline

- Layout computes in `f32`; one documented rounding point at `compose`, per axis, in device pixels (single-snapping policy). 
- egui's hard-won cache lesson is adopted as ALGO: **integer wrap-width rounding** in the galley cache kills the re-wrap feedback loop class (`crates/epaint/src/text/galley_cache.rs:41-64`, citing issues #4927/#4928/#5084/#5163 @ `fc9d66d`).
- Masonry's debug purity panic is enabled from day one: if a forced re-measure disagrees with the cached size, the build panics at the guilty widget (`passes/layout.rs:72-99`).

### 8.4 Verification

The §6.4 fuzz harness is mirrored for layout: *incremental relayout ≡ full relayout* on random style/structure churn. Coherence bugs — the known cost of incremental everything (`audit/INNOVATE.md` I4) — are caught by fuzzing + purity panics + snapshot tests, not by users.

**Q1 kill-switch (ADR-3):** a 3-day spike decides whether Taffy's API tolerates ancestor-candidate relayout without forking. If not, torin itself becomes the engine (it is `no_std`-capable and euclid-only, `torin/src/lib.rs:2`) with Taffy kept as an optional compatibility layer.

---

## 9. Rendering — one scene journal, two honest backends

### 9.1 Scene journal (GPUI design, Apache-2.0 CODE)

Paint produces an append-only **scene journal** (GPUI's `scene.rs` architecture, `crates/gpui/src/scene.rs:39-53, 151-163, 172-180` @ `056a44d`):

- per-kind primitive vectors (quads, text runs, images, shadows, borders) enabling same-kind batching for instanced draws (`batches()`);
- an ordered `paint_operations` journal with `finish()` sorting by painter order — replay is deterministic and cache-friendly;
- rounded corners and shadows as SDF quad properties (`scene.rs:574-586`) — no polygon explosion for the two most common effects.

Because the journal is a *record*, it is also a *replay*: cached subtrees re-emit their recorded ranges (GPUI `Entity::cached` replay semantics, `view.rs:552-568`, `scene.rs:141-149` — with its documented constraint respected: caching requires definite sizes, `view.rs:252-256`).

### 9.2 Two backends, one journal (ADR-2)

| Backend | Role | Notes |
|---|---|---|
| **wgpu** | primary | consensus substrate (7/10 audited stacks; Masonry 28, Iced 30, egui 30, Blitz 30, Floem 27, Blinc 26 — Cargo.locks verified) |
| **tiny-skia + softbuffer** | headless/CI/low-end/VM | the test rig *is* this backend (A4); Floem proves a 4-backend `Renderer` trait with honest fallback (`renderer/src/lib.rs:103-150`, `src/paint/renderer.rs:66-83` @ `1351ffb`) |

Vello is **tracked, not load-bearing** (at Masonry HEAD it was removed as a direct dep and survives via `imaging_vello` — API churn is real, `audit/PRIMITIVES.md` P2). Skia may arrive later as an optional external backend (Floem's anyrender-skia precedent), never in core: its C++ build tax is exactly the cold-build cost we budget against (`audit/BENCHMARKS.md` §2).

### 9.3 Partial paint: damage with an escape hatch

- **Damage model:** the union of `Change::Paint` rects, merged into at most **three** DirtyRegion boxes with least-growth overflow merging, old+new rects on geometry change — Slint's algorithm, ported **clean-room** from documented behavior (its code is GPL/commercial; `audit/LICENSES.md` classifies it IDEA/ALGO-only): `internal/core/partial_renderer.rs:273` fixes the ≤3-rect budget, `:290-318` merges overflow by least growth, `:604-618` adds old+new screen rects on geometry change @ `0b1c00d`.
- **Overflow → full frame.** When damage exceeds the region budget or is untrackable (backdrop-blur-style effects, Slint `:726-735`), Alacer presents the whole frame. Correctness first; partial paint is an optimization with a defined fallback, never a risk.
- **Per-item paint trackers** (Slint's `PropertyTracker` slabs, `:748-763`) are a stretch goal behind Q4 — the journal-replay path alone is expected to carry most of the win, exactly as GPUI's does.

### 9.4 Text

- **parley + fontique + swash** as the default stack (ADR-4): Linebender-family convergence (Masonry 0.8, Floem 0.7, Slint migrating to 0.12) means bugfixes flow from a shared ecosystem.
- **Glyph atlas:** cryoglyph-lineage GPU pipeline (glyphon's descendant, Apache/MIT) with 4-way subpixel binning (GPUI precedent, `gpui_linux/src/linux/text_system.rs` + `text_system.rs:50-53` @ `056a44d`).
- **Caches evict.** Where egui's atlas "grows in height as needed" forever (`glyph_atlas.rs:381-395`) and `IdTypeMap` is never auto-collected (`id_type_map.rs:573-601`), Alacer's shaping/galley caches use frame-windowed eviction (egui's own `FrameCache` scheme, `cache/frame_cache.rs:37-44`) with dhat-enforced RSS budgets (§14.3).
- Per-OS platform shapers (GPUI's `PlatformTextSystem` trait design) remain the documented post-1.0 escape hatch for per-platform text quality — behind the `TextLayout` trait seam, not a fork.

---

## 10. Input & focus

### 10.1 Routing as a normative spec

Event routing ships as a **written spec with committed performance baselines** — Floem's model, adopted end to end (`src/event/README.md:24-152, 154-242` for the normative dispatch spec; `:334-412` for the committed baselines @ `1351ffb`): z-index-aware hit-testing; capture → bubble phases; precomputed immutable event paths (no per-event tree walks); W3C pointer-capture semantics (pending → active); synthetic click-after-pointer-up. Baselines: ~2.25 ns cached hit-test, ~390-410 ns flat-tree dispatch, ~100-120 ns per depth level.

**Status:** the capture half of this spec is M19's delivery (ADR-0025): a press implicitly holds its deepest accepting hit — single-pointer, because the event vocabulary carries no pointer id (ADR-0014's shape) — and the gesture's `Move`/`Up`/`Leave` stream routes through the captured node's ancestor chain with the same capture→bubble phases; the `Up`/`Leave` and the target's death release the capture, the wheel stays hover-routed, and a frame with no capture in flight is the per-event hit test exactly as before. Floem's pending→active two-phase capture and the precomputed event paths remain the tracked evolution.

- **Spatial index:** a `GridF64` 100 px-cell index over element boxes (Floem's understory backends, `src/view/storage.rs:104, 130-143` @ `1351ffb`) plus a 2-entry exact-float-bits hit-test cache, cleared on layout/tree changes (`src/event/path.rs:17-24`) — the Blink-inspired micro-optimization, kept honest by the spec's baselines.

### 10.2 Coalescing and pacing

Adopt Freya's event-buffer discipline (`crates/ragnarok/src/event_buffer.rs:10-69` @ `0b13171`): compatible consecutive events merge; `MouseMove` coalesces on equal button state (`platform.rs:169-178`); same-direction wheel deltas accumulate (`platform.rs:179-198`); the release deadline derives from the monitor's refresh interval in mHz (`crates/freya-winit/src/window.rs:754-763`). Result: input storms cost one frame of work, not N events. **Status:** delivered by M8 (ADR-0014) — the coalescer lives on the `PassPipeline` (`alacer_core::input::EventCoalescer`); the platform queues and never drains; `tick` routes the coalesced batch at its start, and the `input_storm_batching` budget publishes the ratio.

### 10.3 Focus, IME, shortcuts

- **Focus** is a pair of first-class passes (§7.2), not an afterthought on the event path. Since M24 the walk is public vocabulary (`FocusMove`, one `focus_move` door), since M25 it bounds to a subtree (`focus_move_in`, the modal's focus trap), and since M26 it reads the window too: the four directional moves select the candidate the laid-out bounds name — fully beyond the focused node's edge in the direction, nearest edge distance first, cross-axis misalignment breaking ties — never wrapping, refusing honestly at the edge, the scope and the availability tier filtering before the geometry answers (ADR-0030/ADR-0031/ADR-0032).
- **IME contract** (I2): preedit styling, cursor-rect sync, purpose hints, commit semantics — semantics copied from GPUI's `EntityInputHandler` (UTF-16 ranges, marked text) and egui's platform-quirk table including the Windows-Korean preedit workaround (`egui-winit/src/lib.rs:741-763, 795-806` @ `fc9d66d`). Where winit falls short on Wayland, contribute `zwp_text_input_v3` handling upstream (smithay-client-toolkit), *never* fork (A5).
- **Shortcuts:** GPUI's action/keymap system (typed actions, multi-key sequences) as the IDEA steal; actions flow through pass 10, so keymaps are inspectable and conflict-checkable.
- **Drag & drop** and clipboard data-transfer formats land post-0.1 (winit's data-transfer extension is the seam — KAS precedent, `kas-core/src/data_transfer.rs:4-7`).

---

## 11. Accessibility — a type-level contract

The audit's sharpest finding is a market failure: AccessKit is the de-facto standard (8/10 Tier-1 use it or route through the webview), yet Iced and Floem ship **zero** a11y, and iced's issue #282 has been open since 2020 (`audit/MATRIX.md` §6; `audit/INNOVATE.md` I1). Alacer treats that as the opening.

### 11.1 The contract (ADR-5)

1. **Required widget methods.** The `Widget` derive requires `role` and a label source — Masonry's required-method design (`masonry_core/src/core/widget.rs:424-435` @ `dc49687`) expressed as `#[derive(Widget)] #[widget(role = Button)]` so the friction lands in one line, not one trait impl.
2. **Debug-mode CI walker.** A test-only pass walks the widget tree and fails the build if any widget lacks role/label or exposes unnamed input controls — novel in the ecosystem, cheap to build (egui's kittest already panics on unnamed inputs *at test time*; we move the check to *compile/test gate* time, `crates/egui_kittest/README.md` @ `fc9d66d`).
3. **Identity sharing.** AccessKit `NodeId` = id-path hash = hit-test id (§6.3) — the a11y tree is the same tree, not a parallel one.
4. **Incremental updates.** TreeUpdates built from dirty nodes only (Freya's `accessibility/dirty_nodes.rs` @ `0b13171`), inside pass 9 — never a full-tree rebuild per frame (egui emits a full `TreeUpdate` every end-pass, `crates/egui/src/context.rs:3182-3207` @ `fc9d66d`; we do not inherit that cost).
5. **Tests drive the a11y tree.** The official query API — `find(ROLE_BUTTON).named("Save")` — routes through AccessKit nodes with settle-or-error semantics (kittest: 8,131 LOC, `crates/egui_kittest/src/lib.rs:69-80`): a test can only pass if the widget is *reachable, named, and actionable* by a screen reader. Testing and accessibility reinforce each other; neither can rot silently.
6. **Actions are routed, never munged** (M10, ADR-0016). The platform maps each AccessKit `ActionRequest` onto exactly one `A11yAction` variant (`Click`, `Increment`, `Decrement`, `ScrollUp/Down` with the request's `ScrollUnit`); pass 10 delivers it and the widget refuses what it does not implement — refusal queues no masks and fires nothing, so an AT is never silently lied to. The canon consumes the tier: the list answers scroll/step actions translation-only, and keyboard parity holds because a focused button answers Space/Enter like a pointer press-and-release.
7. **The selection is tree vocabulary, not widget trivia** (M11, ADR-0017). The focused editable exposes anchor and caret in the §10.3 UTF-16 contract unit (`Widget::selection()` → `TreeNode.selection` → AccessKit `TextSelection` on the value-bearing node), and the value tier rides the same faithful mapping: `SetValue`, `ReplaceSelectedText`, and `SetTextSelection` carry their payload or are dropped at the boundary — a `SetValue` without data never becomes a press, and a refused value action costs an idle frame. While a preedit composes, the report drops: the IME owns the canvas, and a stale range would lie to the AT.
8. **The unavailable control is tree vocabulary** (M22, ADR-0028). One
   `enabled()` source of truth on the `Widget` trait — the default
   `focusable()` derivation consults it, so an unavailable control is
   never a Tab stop and never a focus candidate; pass 3's sentinel
   discipline evicts focus from a control that becomes unavailable; the
   pointer is a wall (the gesture never clicks through), the keyboard
   and the IME are answered with silence, and every AT action is
   refused, never munged (the M11 rule). The flag rides pass 9's
   snapshot — `TreeNode.enabled` — and the sink maps it faithfully onto
   AccessKit's `disabled` property, so the AT reads the refusal before
   it happens; a write that lands where the flag already is queues no
   masks.

### 11.2 Budgets

TreeUpdate p99 < 0.5 ms at 1k nodes (M3 gate); a11y pass cost is a committed tracing baseline like every other phase; and since M10 the action route is a published same-host regression gate (`a11y_action/route_100_actions_1k`) — routing an AT action stays in the cost class of routing a pointer event, never an a11y re-materialization; since M11 the same holds for the value tier's payload transport (`a11y_action/route_100_setvalue_1k`). *Kill-switch:* if the budget breaks frame targets, scope updates to lazy-subtree mode — the type-level contract stays; the update granularity adapts.

---

## 12. Platform integration

**winit, un-forked** (ADR-1): Masonry, Freya, egui, Slint and blitz ride stock winit; `accesskit_winit`, `muda`, and friends assume its types — ecosystem gravity is free interop. Alacer isolates the platform behind a thin `Platform` trait (GPUI's `crates/gpui/src/platform.rs` design) from day one so a future custom loop (display-link pacing, layer-shell) never breaks the public API.

Platform services, all permissive-licensed, all adopted not written (audit PRIMITIVES P8): menus via **muda** (Slint ships a 1,112-LOC integration; Dioxus and Floem also use it), dialogs via **rfd**, clipboard via **arboard** (+ `smithay-clipboard` on Linux), tray via **tray-icon**, file watching via **notify**, window handles via **raw-window-handle**.

The line we do not cross: GPUI's 7 per-OS platform crates and ~100k LOC of platform code are a Zed-scale investment (`audit/ADR-1`); Alacer buys its wins (IME quality, display-link pacing) via upstream contribution, and buys nothing else until users demand it.

---

## 13. DX architecture

Best-in-class DX is a *subsystem*, specified and budgeted like rendering. Six pillars:

### 13.1 Views that compile-check: `rsx!` with real diagnostics

Proc-macro builders, no external DSL (§15). Slint's DSL tooling is the gold standard — and the cautionary investment: a 91,430-LOC compiler with 66 passes plus a 36,427-LOC LSP (`internal/compiler/passes/`, `tools/lsp` @ `0b1c00d`). Alacer steals the *diagnostics patterns*, not the compiler: miette-style labeled spans on `rsx!` errors, "did you mean" for style properties, and error texts written like Masonry's hint strings ("Did you forget to use `with_action_widget`?", `xilem/src/driver.rs:317-322` @ `dc49687`).

### 13.2 State-preserving hot reload that CI proves

`subsecond` is adopted as-is (ADR-7): jump-table fn-pointer patching with state preserved in generational-box owners — already shared infrastructure for Dioxus, GPUI, Leptos, and Freya (`crates/gpui/src/hot_reload.rs:11-40` @ `056a44d`; Leptos `Cargo.toml:62-64, 116-125` @ `7d656a4`). Alacer's contribution is the **contract + the proof**:

- all UI-adjacent state lives in owners that survive `subsecond::call` frames (Dioxus precedent);
- on patch, `RenderEffect`s re-dirty instead of rebuilding the world (Leptos pattern, `render_effect.rs:175-216`) — **scoped re-render, not Dioxus's `SchedulerMsg::AllDirty`** (`packages/vdom/src/virtual_dom.rs:798-803` @ `7dfd07e`);
- a **CI hot-patch suite**: every example is hot-patched in CI and asserted to render the new value with state intact. Nobody in the audited ecosystem does this; subsecond's admitted edges (thread-local resets, segfault class — in-code warnings at `packages/subsecond/subsecond/src/lib.rs:88-93`; issues #5317/#4872; Freya's child-scope breakage #2304) are precisely what such a suite catches.
- *Fallback (ADR-7):* if subsecond cannot pass the suite by M4, ship template-only hot reload (the `leptos_hot_reload` AST-diff pattern, MIT) and re-evaluate at 1.0.

### 13.3 The Causality inspector

Every audited devtools answers "what does the tree look like?"; none answers "**why**" (Dioxus's most-requested missing tooling; `audit/INNOVATE.md` I5). Because Alacer owns both the reactive graph and the journals, it can answer causally — the moat is that only the framework can build this:

| Question | Answered from |
|---|---|
| "Why did this render?" | reactive_graph edge activations leading to the node's mask |
| "Who wrote this signal?" | write provenance recorded per `Signal` (dev-mode) |
| "Who subscribes to this memo?" | graph reverse edges |
| "Why is this frame slow?" | per-phase tracing spans + frame journal (GPUI ships a 3,542-LOC profiler journal as precedent, `profiler/journal.rs`) |
| "What did this mask touch?" | mask heatmap overlaid on the widget tree |

Overhead is feature-gated and sampled; the inspector is a standalone app (Freya's devtools-app precedent, `crates/freya-devtools/src/server.rs:20-90` @ `0b13171`) speaking a documented WS protocol.

### 13.4 Testing as a product (`alacer-testing`, day one)

The four-way convergence (masonry_testing / freya-testing / kittest / floem headless) is adopted and extended (I6):

- **Headless = production pipeline + software raster** (A4): synthetic pointer/key/IME/a11y events, time control (`animate_ms` — Masonry, `masonry_testing/src/harness.rs:73-130` @ `dc49687`), PNG snapshots GPU-free with `BLESS` re-blessing and oxipng size caps.
- **A11y-driven queries** as the only official API (§11.1) — including automatic panics on unnamed input widgets.
- **Record/replay**: user interactions recorded to `.alc` files replayable in `cargo test` (Iced's `.ice` precedent, `examples/todos/tests/carl_sagan.ice` @ `8ff48c4`) — bug reports that are also regression tests.
- **Baselines as contracts**: committed event-latency and phase-time criterion baselines (Floem precedent) — the harness fails on regression before users feel it.
- **Fuzz targets**: reconciler, incremental layout, and journal equivalence (§6.4, §8.4).
- **Hot-patch suite** (§13.2).
- Snapshot hygiene: per-OS thresholds (kittest's `kittest.toml` pattern) so cross-platform font differences don't rot the corpus.

### 13.5 Zero-to-first-pixel: budgets a user feels

Publish and CI-enforce (I7; nothing in the audited ecosystem publishes compile-time budgets):

| Metric | Budget [budget] | Anchor |
|---|---|---|
| Cold build, counter example | < 60 s | reference rig; no Skia-class C++ in the dep tree |
| Incremental rebuild | < 1 s | ThinLink-style object patching if needed (Dioxus's <500 ms claim is UNVERIFIED — we commit to the weaker number) |
| Binary size (stripped) | < 10 MB | against peer stacks' lockfile weight (`audit/BENCHMARKS.md` §2) |
| Idle RSS | < 40 MB | with evicting caches (§9.4) |
| Example gallery | counter ≈ 30 LOC, todo ≈ 70 LOC | Dioxus counter = 21, Freya = 48, Floem = 80, Xilem todo = 169 (`audit/MATRIX.md` §7) |

### 13.6 Docs and migration

A book written against the benchmark canon (counter / todo / data-table); migration notes from egui (state where each `Memory`-ism lives now) and Iced (message → signal mapping); an examples gallery doubling as the storybook (Makepad's storybook app is the UX precedent).

---

## 14. Performance architecture

### 14.1 The method (before the numbers)

The audit's dynamic phase was honest about its limits: no GPU, no toolchain — so it fabricated nothing and left a harness spec (`audit/BENCHMARKS.md` §5). Alacer adopts that spec as its own performance constitution: shared workloads, fixed rig, 5-run medians, maintainer-reviewed idiomatic examples per compared framework (no strawmen), criterion for CPU paths, tracing spans for phase attribution, dhat for allocations, everything labeled.

### 14.2 Workloads and targets

| # | Workload (BENCHMARKS §5) | Target [budget] | Mechanism under test |
|---|---|---|---|
| W1 | 10k-node tree build + 10%/frame style churn | ≥ 5× vs full relayout by M2 gate (≥ 2× minimum to keep the design) | persistent Taffy + ancestor candidates (§8) |
| W2 | text-dense relayout: 1k paragraphs, width jitter ±5% | stable frame times; no re-wrap oscillation | text-node-only dirty marks + integer wrap rounding (§8.3) |
| W3 | animation loop, 1k nodes, position/opacity @ vsync | 120 fps @ 1k nodes on the reference rig | journal replay + translation-only updates (§8.2, §9.1) |
| W4 | input-to-paint latency | p99 ≤ 2 frames @ 120 Hz | coalescing (§10.2) + phase budgets (§4.3) |
| W5 | signal-graph churn: 10k signals, 1k memos | one rerun per write batch; no glitching | push-Check/pull-Update + 1-bit coalescing (§5.1) |

**Reference rig** (declared once, kept stable): a specified Linux desktop (Vulkan) plus an llvmpipe software run for parity; same winit version across competitors; vsync locked.

### 14.3 Memory discipline

- Idle RSS < 40 MB [budget]; dhat profiles run in CI on the W1/W3 workloads.
- **Warm-path allocation budget: zero heap allocations** for a static UI's steady-state frame (paint replay + present) [budget]; the masks/journal design makes this checkable, and any new feature that breaks it needs an ADR.
- Every cache has an eviction policy *and a test that proves it* — the ecosystem's unbounded-cache club (egui's atlas and `IdTypeMap`; cited issues #3689) stays at zero members.
- Hot-path maps use `FxHash`-class hashing only where profiling justifies it (Makepad measured SipHash at ~4% of main-thread encode samples, `platform/hal/src/lib.rs:13-15` @ `8049ae2` — the lesson is "measure then swap", not "swap everywhere").

### 14.4 Degradation ladder (backpressure)

When a frame can't fit its budget: (1) coalesce remaining flux work into the next frame (already the default); (2) merge damage regions and present one larger region; (3) present full frame; (4) *never* skip the a11y or focus passes — latency may degrade, correctness may not. This ladder is documented, ordered, and each rung is observable in the frame journal.

### 14.5 Scheduling model

Main thread owns the frame loop, the arena, and paint (retained tree mutation is main-thread by contract — Makepad documents the same reality honestly, `widgets/core/src/widget_tree.rs:14-17` @ `8049ae2`). Async work (resources, IO) runs on any_spawner's executors off-thread and lands in the frame via signals — the effect channel coalesces their wakeups. Parallel layout/paint of independent subtrees is an explicit non-goal for v1 (§15) — sequencing risk is not worth pre-1.0 complexity when W1–W5 targets are reachable single-threaded.

---

## 15. What Alacer will not build

Scope discipline is architecture. Each non-goal cites who owns it and why we lose nothing:

| Non-goal | Reason (evidence-based) |
|---|---|
| **No external DSL** | Slint's DSL moat rests on a 91k-LOC compiler + 36k-LOC LSP; competing there doubles scope for a moat we don't need. Proc-macro builders + stolen diagnostics patterns cover the need. |
| **No webview runtime** | Dioxus owns it; the memory cost and non-native a11y seams are verified (`audit/STEAL.md` anti-steals). |
| **No web/mobile in the first 6 months** | Desktop-first; wasm/mobile = M4+ stretch behind the platform matrix (I8). GPUI's iOS a11y absence is the cautionary tale — we won't ship platforms we can't make accessible. |
| **No embedded/MCU story** | Slint's software renderer is 15 years of company work. We keep a first-class *desktop* software path only (A8), which incidentally serves CI/headless. |
| **No 3D / game-engine ambitions** | Bevy adjacency is a trap; wgpu composability is the escape hatch. |
| **No TUI** | Out of the audited problem space (ratatui owns it). |
| **No runtime CSS engine** | Typed Rust style properties with Floem-style reason-flag recalc (§7.2 pass 5); selectors/parity with Stylo is a multi-year spec, and no audited native stack ships a full engine either. |
| **No vendored monorepo** | Makepad's 1.85M LOC / 205 vendored crates buys independence at the price of audit impossibility and bus-factor ≈ 1 (squashed history). We buy the same independence with small, permissive deps (A2, A5). |

---

## 16. Moats

Three differentiators, each hard to copy and disproportionately funded (the audit requires ≥ 3, each evidenced):

1. **The invalidation composition (§4.1, Pillar 3).** One `Change` currency feeding incremental layout (torin-on-Taffy), journal-replay paint, and incremental AccessKit updates. A competitor matching it must rework their layout engine (all Taffy users), their renderer (egui, Iced), or their reactivity (GPUI) — and Freya 0.4 shows what churn without this discipline does (`render_pipeline.rs:58`). We fund it as the core (~40% of M1–M3 effort), and it is kill-proof: the engine transfers to any widget set.
2. **Zero-config accessibility at the type level (§11).** Competitors can bolt on AccessKit; making it *required* breaks their APIs — a cost incumbents demonstrably defer (iced #282, open 6 years). We fund the docs, the quirks tables, the CI gate, and the a11y-driven test API that no solo project matches.
3. **Causality devtools (§13.3).** "Why did this render / who wrote / who subscribes" requires owning the reactive core's semantics *and* the renderer's journals — a composition only the framework itself can build. GPUI's profiler journal and Iced's message replay prove the appetite; nobody owns the whole causal chain.

---

## 17. ADR register

ADRs 1–7 were ratified with the audit bundle (`adr/ADR-*.md`); each carries its own kill-switch. ADRs 8–11 are authored here to complete the architecture; ADR-0012…ADR-0031 scope milestones M6–M25 as they shipped, and ADR-0032 scopes M26 (see `docs/adr/`).

| ADR | Decision | Status | Kill-switch (falsifiable) |
|---|---|---|---|
| **ADR-1** | Windowing: stock winit behind a thin `Platform` trait; contribute upstream, never fork | Accepted | winit blocks a launch-critical feature (e.g. sub-frame pacing) > 2 months → patch crate, not fork |
| **ADR-2** | GPU: wgpu primary + tiny-skia/softbuffer behind one scene journal; Vello tracked, not load-bearing; Skia optional later | Accepted | journal replay on wgpu misses W3 budget by M3 → re-evaluate GPUI-style per-OS pipelines |
| **ADR-3** | Layout: persistent Taffy tree + torin-style incremental port + Masonry measurement discipline | Accepted (Q1-gated) | 3-day spike: Taffy API resists ancestor-candidate relayout → adopt torin as the engine, Taffy optional |
| **ADR-4** | Text: parley + fontique + swash behind a `TextLayout` trait; cryoglyph-lineage atlas; egui cache lessons as ALGO | Accepted | blocking CJK/complex-script parley bug with no upstream bandwidth → swap impl to cosmic-text |
| **ADR-5** | A11y: AccessKit-only, required methods via derive, id-path NodeIds, dirty-node updates, a11y-driven tests | Accepted | AccessKit can't express a needed platform → evaluate Makepad's bespoke stack (interop UNVERIFIED — audit before reuse) |
| **ADR-6** | Reactivity: adopt `reactive_graph`; upstream memo-cycle detection; TrackedRefCell diagnostics | Accepted | effect scheduling can't drive paint batching in M0 → GPUI-style draw-level tracking for paint, reactive_graph for data |
| **ADR-7** | Hot reload: adopt subsecond as-is + CI hot-patch suite + scoped re-render contract | Accepted | CI suite fails by M4 (segfault class) → template-only hot reload (leptos_hot_reload pattern) |
| **ADR-8** *(new)* | **Identity & reconciliation:** id-path prefix hashing as the single identity across element-state, hit-testing, AccessKit NodeIds, and hot-reload keys; keyed peel→map→LIS diffing; fuzz-equivalence gate | Accepted | equivalence fuzz finds a class LIS cannot express minimally → fall back to key-map move semantics, keep fuzz gate |
| **ADR-9** *(new)* | **Workspace & discipline:** 12-crate DAG (§4.2) with facade-only re-exports; `missing_docs` + `unsafe_code` deny (Iced precedent, `Cargo.toml:253-254` @ `8ff48c4`); zero unsafe outside platform crates; `cargo-deny` license gate (no AGPL, no unknown licenses); upstream-first patch policy | Accepted | DAG friction measurably slows feature work > 1 sprint → consolidate to 6 crates, re-vote |
| **ADR-10** *(new)* | **Testing productization:** headless-with-real-pipeline as default `cargo test`; a11y-driven queries as the only official API; baselines-as-contracts; fuzz targets; hot-patch suite | Accepted | it *is* the validation (I6) — scope shrinks only via roadmap gates, not by removing the harness |
| **ADR-11** *(new)* | **Versioning & stability:** 0.x semver; `alacer` + widget crates follow semver-style care from 0.1 (deprecations, migrate notes); internals (`alacer-core`, `-view`, `-render`) may break at minors with CHANGELOG; MSRV pinned; renaming the audit codename to Alacer is recorded as the first entry | Accepted | community pain signals (issue-labeled `stability`) > 10/month → adopt 1.0-RC freeze earlier |
| **ADR-0012** | M6 scope: widget canon + record/replay + W2/W3/W5 published benches | Accepted | canon can't hold the a11y contract end-to-end → narrow the canon to what can |
| **ADR-0013** | M7 scope: the reproducible bench rig + cross-framework comparison beat | Accepted | competitor absolute medians gated → benchmark theatre (never done) |
| **ADR-0014** | M8 scope: frame-batched input — the coalescer joins the scheduling contract | Accepted | per-event dispatch semantics change observable widget behavior → route the batch per event behind a queue-depth knob |
| **ADR-0015** | M9 scope: the windowed a11y path joins the incremental contract — the pipeline's sink is the only `TreeUpdate` materialization path | Accepted | a platform adapter needs a shape the sink cannot express → re-open the seam with the shape as a trait method |
| **ADR-0016** | M10 scope: the a11y action tier joins the dispatch contract — faithful `ActionRequest` mapping, the `Increment`/`Decrement`/`Scroll*` vocabulary, keyboard-operable buttons | Accepted | an action class needs a shape pass 10 cannot express → grow the vocabulary behind the same refuse-don't-munge contract |
| **ADR-0017** | M11 scope: the UTF-16 selection model joins the editing and a11y contracts — anchor+caret in the contract unit, selection-consuming edits, the `SetValue`/`ReplaceSelectedText`/`SetTextSelection` tier mapped faithfully | Accepted | an AT needs a selection shape the value-bearing node cannot express → grow the snapshot vocabulary behind the same refuse-don't-munge contract |
| **ADR-0018** | M12 scope: the pointer joins the editing contract — the resolved frame reaches `on_pointer_event`, the text seam inverts the caret's own prefix widths (`hit_test`), a click places the caret and a drag extends the selection from the anchor | Accepted | a pointer gesture needs a mapping the event/text seams cannot express → grow the contract behind the same `PAINT \| A11Y` currency (frame on `EventCtx`, `hit_test` on `TextSeam`) |
| **ADR-0019** | M13 scope: the word joins the editing contract — the text seam carries the UAX #29 segmentation (`word_segments`, required — no honest default without the tables), core fixes one hop rule, the editor hops word runs (Alt/Ctrl+arrows), deletes the hop's span (Ctrl/Alt+Backspace/Delete), and a double-click selects the word under it (`PointerEvent::Down` grows the platform-computed `click_count`) | Accepted | a word gesture needs a mapping the event/text seams cannot express → grow the contract behind the same `PAINT \| A11Y` currency (segmentation on `TextSeam`, the chain on `PointerEvent::Down`) |
| **ADR-0020** | M14 scope: the clipboard joins the editing contract — the `ClipboardSeam` carries the OS round trip (no provided implementation; the engine rides arboard in `alacer-platform`), core fixes one paste rule (line breaks collapse to spaces), the editor answers Ctrl/Cmd+C/X/V with the M11 selection rules, a refused write never deletes, and refused chords cost an idle frame | Accepted | a clipboard shape the seam cannot express → grow the contract behind the same refuse-don't-munge rule (the seam in `alacer-core`, the engine in `alacer-platform`) |
| **ADR-0021** | M15 scope: the undo stack joins the editing contract — the `EditHistory` journal in core carries the inverse of every committed edit (bounded, the pure-append coalescing rule, the whole-value reset), the editor routes every edit through one journaling funnel and answers Ctrl/Cmd+Z, Shift+Ctrl/Cmd+Z, and Ctrl/Cmd+Y with the M11 selection semantics, and an exhausted or reset journal refuses with no masks | Accepted | an undo gesture needs a mapping the edit funnel cannot express → grow the contract behind the same refuse-don't-munge rule (the journal in `alacer-core`, the funnel in the editor) |
| **ADR-0022** | M16 scope: the consumable contract — the 0.12.0 cut completes (the internal dependency pins and the lockfile ride `[workspace.package]`), `PlatformError`/`BackendError` carry their `source()` chains and name the escape hatch they know, the widgets crate's self-description matches the shipped vocabulary, and the M15 journal gets its consumer demonstration (`examples/undo_redo.rs`, blessed with the m1 corpus) | Accepted | a feature milestone lands in a surface that cannot publish, mis-describes itself, or drops its error chains → fix the surface first; the ledger's candidates (announcements, pointer capture, a `Slider`) each need a full corpus cycle (see `docs/adr/ADR-0022-m16-scope.md`) |
| **ADR-0023** | M17 scope: the demonstrated queue — the M8 coalescer gets its consumer demonstration (`examples/input_storm.rs`, a custom `StormPanel` probe blessed with the m1 corpus, tracing at `alacer::input`), the living documents re-agree with the checked-in baselines (the README's 14.06×/40.7×, the migration guides' 0.12.0 self-description, the facade's shipped-surface rustdoc), and the error-chain discipline reaches `Recording::load` (the parse cause walks) and `LayoutError` (the failing node is data, not prose) | Accepted | a feature milestone lands in a surface whose demonstrations, numbers, or error chains lag its own API → fix the surface first; the ledger's candidates (announcements, pointer capture, a `Slider`) each need a full corpus cycle (see `docs/adr/ADR-0023-m17-scope.md`) |
| **ADR-0024** | M18 scope: the announced delta — the a11y announcement seam joins the invalidation currency (a reserved live-region node in pass 9's snapshot, polite/assertive `aria-live`, the handler-reachable queue drained at the tick, refuse-don't-munge, `examples/announcements.rs` blessed with the m1 corpus, tracing at `alacer::announce`), the last two `String` error leaves carry their causes (`PlatformError::Softbuffer` ← `softbuffer::SoftBufferError`, `BackendError::SurfaceCreation` ← `wgpu::CreateSurfaceError`), the dead convenience APIs go (`PassPipeline::has_queued_input`, `run_hot_window_with_channel`, the registry accessors), and the living documents re-agree (the ADR count, the migration guides' snapshot counts, the journal's materialization-shape claim, the facade's pass-order rustdoc) | Accepted | the region is a second invalidation channel or announcing moves the tree's shape → collapse it into the widget vocabulary; a speech act needs a shape the sink cannot express → re-open the seam with the shape as a property on the snapshot node (see `docs/adr/ADR-0024-m18-scope.md`) |
| **ADR-0025** | M19 scope: the captured drag — the pointer capture joins the dispatch contract (a press implicitly holds its deepest accepting hit, the gesture's Move/Up/Leave stream rides the captured node's ancestor chain wherever the pointer goes, the Up/Leave and the target's death release it, the wheel stays hit-tested, and a frame with no capture routes exactly as the hit test always did), the surface work rides the same cut (the dead `EventCoalescer::pending` goes — zero call sites, the M18 pattern — and the `rsx!` single-root diagnostic names what it found with its help text), and the living documents re-agree (the README's 53-case criterion count finally quotes the 54-case baseline, the §10.1 routing spec states the capture it ships, `examples/drag_capture.rs` demonstrates the contract) | Accepted | a gesture needs a route the single capture slot cannot express (multi-pointer, per-button capture) → grow the event vocabulary with the pointer id first, then the capture table; a widget needs to refuse or steal the implicit capture → re-open the seam with the opt-out as a `Widget` predicate beside `accepts_pointer` (see `docs/adr/ADR-0025-m19-scope.md`) |
| **ADR-0026** | M20 scope: the valued widget — the slider joins the widget vocabulary (a `Role::Slider` that is requires-name and focusable and carries its bounded value through the M3 `value()` contract; one funnel with four doors — drag against the resolved frame, arrows and Home/End, the AT's `SetValue`/`Increment`/`Decrement` through the existing action route — with every write clamped to the range and snapped to the step inside the widget, non-numeric and non-finite writes refused honestly, a no-op write queues no masks), `examples/slider.rs` is the consumer demonstration, the corpus grows 54 → 55 by the coordinated rule (the M19-promised `input/route_100_captured_moves_1k` measured, published, and gated at this run), the m1 corpus grows 33 → 35 (the drag-capture and slider scenarios), and the living documents re-agree (the README's badge, count, vocabulary, and status block, the catalog, the migration guides, the CHANGELOG) | Accepted | a range or multi-thumb slider needs a vocabulary one value cannot express → re-open the seam with the second thumb priced by a real UI; a value write needs a shape the funnel cannot express → grow the contract behind the same refuse-don't-munge rule (see `docs/adr/ADR-0026-m20-scope.md`) |
| **ADR-0027** | M21 scope: the reported value — the progress indicator joins the value tier as its read half (a `Role::ProgressIndicator` that is requires-name and *not* focusable and reports the app's bounded scalar through the M3 `value()` contract; two app doors landing the same `PAINT \| A11Y` currency — the binding bridge with the caller's masks, and the mutate seam whose `set_value` answers the move it made so a write that lands where the value already is queues nothing; every AT action refused honestly — a reporting widget has no write door, the M11 rule), `examples/progress.rs` is the consumer demonstration (the mutate seam's first one), the corpus grows 55 → 56 by the coordinated rule (the value-write route's `progress/route_100_value_writes_1k` measured, published, and gated at this run), the m1 corpus grows 35 → 36 (the reported value's window), the micro-debt the audit priced rides the cut (the dead `PassPipeline::has_observer` and the wgpu backend's dead stats surface go), and the living documents re-agree (the migration guides' 0.12.0 self-description among them — the M16 pattern's remainder) | Accepted | a report needs a write door (an editable progress) → that is the editor's or the slider's contract, not a reporter's — grow the vocabulary there; a value report needs a shape the bridge cannot express → re-open the seam with the shape as a property on the binding contract (see `docs/adr/ADR-0027-m21-scope.md`) |
| **ADR-0028** | M22 scope: the unavailable control — the availability tier joins the widget contract (one `Widget::enabled` source of truth; the default `focusable()` derivation consults it; pass 3 evicts focus from a control that becomes unavailable; the pointer is a wall, the keyboard and IME are answered with silence, and every AT action is refused honestly, never munged), the flag rides pass 9's snapshot (`TreeNode.enabled`) and the sink maps it onto AccessKit's `disabled` property, two doors land the same `PAINT \| A11Y` currency (the fixed `set_enabled` that answers the move it made; the button's `enabled_binding` bridge door), `examples/disabled.rs` is the consumer demonstration, the corpus grows 56 → 57 by the coordinated rule (the refused route's `disabled/route_100_refused_presses_1k` measured, published, and gated at this run), the m1 corpus grows 36 → 37, the micro-debt the sweep priced rides the cut (the software renderer's dead `glyphs_rendered` — the M21 gate's own blind spot — plus `PassPipeline::clear_observer`/`timings_enabled` and the `ViewRoot::root` duplicate), and the `rsx!` diagnostics reach the ADR-0025 species' remainder | Accepted | a control needs a third availability state → grow the vocabulary as a snapshot property, not a second flag; a door needs a shape refuse-don't-munge cannot express → re-open the seam with the shape as a trait method beside `enabled` (see `docs/adr/ADR-0028-m22-scope.md`) |
| **ADR-0029** | M23 scope: the frame's ledger — the scene journal becomes legible to its consumers: a required `SceneJournal::stats()` carrying the `JournalStats` vocabulary (commits, drops, live layers) with `PassPipeline::journal_stats()` as the pipeline's one read door, `examples/frame_ledger.rs` as the consumer demonstration (tracing at `alacer::frame_ledger`), the m1 corpus's broken trio repaired (the `list_scrolled_120`/`400` scenarios never fed `set_content_height`, so the scroll clamp folded every offset to zero and three distinct scroll positions blessed one identical image — the scenarios grow the range and feed the clamp; the next bless run re-blesses them distinct), and the micro-debt the sweep priced rides the cut (the journal's dead inherent accessors replaced by the seam, the todo canon's dead `footer_node`, the `rsx!` body-cardinality diagnostic names its cardinality with `help:` text, the inspector's accept loop reports the session errors it swallowed through the adopted `tracing` core — no new dependency enters the graph) | Accepted | a ledger read per frame on the hot path → that is the damage report's job; grow `FrameReport`, never a second channel (the re-open rule's shape) |
| **ADR-0030** | M24 scope: the focus walk joins the dispatch contract — Tab's private cycle becomes public vocabulary: a `FocusMove` direction (`Next`/`Prev`) and one `PassPipeline::focus_move` door the keyboard's Tab/Shift+Tab fallback and the consumer's code both ride (one funnel, two doors — the order never duplicated at the call site), the walk skips unavailable controls (the M22 cycle-time `enabled` check), wraps at both ends, resolves immediately (key routing sees the new focus this event), and refuses honestly when no candidate exists (no masks, an idle frame, never a first-candidate reset), with `tracing` spans at `alacer::focus` and `examples/focus_traversal.rs` as the consumer demonstration; the micro-debt the sweep priced rides the cut (the flux runtime's dead `effect_count` introspection, the harness's dead `spec_of`/`with_text_context`/`text_size` probes), and the living documents re-agree at M24 (the README's status block, count, features bullet, quickstart, and table; the catalog; the register; the roadmap; the m1 header's snapshot count that lagged its own pin; the m20–m23 suites' document pins; the CHANGELOG) | Accepted | a walk needs an order the tree order cannot express (spatial/D-pad navigation) → grow the vocabulary as a second door priced by a real UI, never a second pipeline channel; a consumer needs the walk bounded to a subtree (a modal's focus trap) → re-open the seam with the scope as a predicate beside `focusable` (the ADR's re-open rule) |
| **ADR-0031** | M25 scope: the focus walk bounds to a subtree — ADR-0030's pre-registered re-open rule (the modal's focus trap) ships as one `PassPipeline::focus_move_in(scope, movement)` door beside the unscoped funnel, the scope the subtree's root node, the walk generalized behind both doors (`cycle_focus_in` — the same candidate list, the M22 availability filter, the wrap, the immediate resolution; the scope bounds the walk, never the candidate list), refusals honest and named with plain help text on the `alacer::focus` target (ADR-0029's rule), `examples/focus_trap.rs` as the consumer demonstration, the micro-debt the sweep priced riding the cut (the dead `fixed_size_of`, the dead `to_f32_rgba`), and the living documents re-agree at M25 and 0.16.0 — the m7 release-state pin re-joining the chain (it missed the M23/M24 sweeps) | Accepted | a scope needs a shape the subtree root cannot express → grow the scope vocabulary beside `focus_move_in`, priced by a real UI, never a second walk (see `docs/adr/ADR-0031-m25-scope.md`) |
| **ADR-0032** | M26 scope: the directional focus walk — ADR-0030's pre-registered re-open rule (spatial/D-pad navigation) ships as four `FocusMove` directions (`Left`/`Right`/`Up`/`Down`) riding the same two doors and the same one walk: the tree-order moves keep the pre-order list and their wrap, the directional moves read the bounds pass 6 wrote (`directional_neighbor` — fully beyond the focused node's edge in the direction, nearest edge distance first, cross-axis center misalignment breaking ties, then tree order), the availability tier filters before the geometry answers, the scope bounds the directional walk exactly as it bounds the tree-order one, space never wraps (the window's edge is the walk's end) and the refusals are honest and named on the `alacer::focus` target (ADR-0029's rule), the entry contract is repaired to what ADR-0031 has documented since M25 (from no focus — or a focus outside the filtered list — the walk takes the list's first candidate; the M25-era arithmetic landed the second), `examples/focus_direction.rs` as the consumer demonstration (tracing at `alacer::focus_direction`), the micro-debt the sweep priced riding the cut (the selection's dead `is_degenerate`, the color's dead `to_premultiplied_rgba8` whose only caller was its own test — the M25 species), and the corpus debt the M24/M25 coordinated rule left rides the cut: the two promised focus routes join the published corpus beside the directional route at this cut's measurement run (the corpus pins move 57 → 60 across the m5–m25 chain in the same commit) | Accepted | a walk needs an order neither the tree nor the edge-distance heuristic can express (a true 2-D grid with occlusion or warping) → grow the selection vocabulary beside `directional_neighbor`, priced by a real UI, never a second pipeline channel; a door needs to observe the walk's refusals as data → grow the answer into a result vocabulary priced by that consumer (ADR-0031's rule stands) (see `docs/adr/ADR-0032-m26-scope.md`) |

---

## 18. Roadmap & kill criteria

Six months, ordered, each gate falsifiable (evolved from VISION.md; M-names align with the audit's plan).

| # | Weeks | Milestone | Exit criteria | Kill criteria |
|---|---|---|---|---|
| M0 | 1–2 | Skeleton: winit + wgpu + reactive_graph + taffy + parley + accesskit on one window | counter + scrolling list render; masks flow end-to-end | reactive_graph cannot drive redraw batches without Glut-style full re-runs (> 1 rebuild per write batch) → pivot Pillar 1 per ADR-6 |
| M1 | 3–6 | Widget arena + pass contract; `alacer-testing` headless crate from day one | 30 blessed snapshots green; a11y queries pass | pass-system overhead > 15% of frame time on 10k nodes (tracing) → collapse to 3 passes |
| M2 | 7–12 | **The invalidation currency**: masks + persistent Taffy + torin-style deltas + scene journal replay | W1 ≥ 2× vs full relayout; journal replay paths cached | W1 < 2× → abandon incremental layout, keep journals (GPUI model) |
| M3 | 13–18 | A11y contract + a11y-driven queries + partial paint on GPU path + IME contract | TreeUpdate p99 < 0.5 ms @ 1k nodes; fcitx5/ibus CI matrix green | a11y cost breaks frame budget → lazy-subtree a11y updates only |
| M4 | 19–24 | DX: subsecond + CI hot-patch suite; Causality inspector spike; publish 0.1 + this architecture doc | sub-second hot reload demo with state intact; inspector demo | subsecond unstable in CI (segfault class) → template-only hot reload per ADR-7 |
| M5 | cont. | Community: examples gallery, egui/Iced migration notes, perf dashboards | criterion baselines published as contracts | — |
| M6 | cont. | Widget canon + record/replay + W2/W3/W5 published benches (ADR-0012) | todo/data-table canons end-to-end through the a11y tree; committed recording replays green; W5 one-rerun-per-batch at graph scale | — |
| M7 | cont. | The reproducible rig + cross-framework beat (ADR-0013) | declared rig, no-strawman competitor harnesses, 5-run-median results committed, beat computed same-host (W1 won; W2/W3 published as lost, scoped) | competitor absolute medians gated → benchmark theatre (never done) |
| M8 | cont. | Frame-batched input: the coalescer joins the scheduling contract (ADR-0014) | a 50-move storm routes one coalesced event (probe + counter + ledger agree); the `input_storm_batching` budget ≥ 2× | per-event dispatch semantics change observable widget behavior → route the batch per event behind a queue-depth knob |
| M9 | cont. | The windowed a11y path joins the incremental contract (ADR-0015): the pipeline's `AccessKitSink` is the only `TreeUpdate` materialization path | the window drains dirty-only deltas with the full M6 vocabulary (values, toggle state, grid slots); the activation response is the sink's own full tree; the `a11y_incremental_speedup` budget ≥ 2×; corpus 45 → 47 | a platform adapter needs a shape the sink cannot express → re-open the seam with the shape as a trait method |
| M10 | cont. | The a11y action tier joins the dispatch contract (ADR-0016): faithful `ActionRequest` mapping, the `Increment`/`Decrement`/`Scroll*` vocabulary, keyboard-operable buttons | the list canon scrolls from the AT side (translation-only, zero measure calls); a non-implemented action is refused with no masks and no effects (never munged into a press); a focused button activates on Space/Enter with GTK release semantics; the `a11y_action/route_100_actions_1k` gate is published; corpus 47 → 48 | an action class needs a shape pass 10 cannot express → grow the vocabulary behind the same refuse-don't-munge contract |
| M11 | cont. | The UTF-16 selection model joins the editing and a11y contracts (ADR-0017): anchor+caret in the contract unit, selection-consuming edits, the `SetValue`/`ReplaceSelectedText`/`SetTextSelection` tier mapped faithfully | a focused input selects with Shift+arrows/Home/End and Ctrl+A and paints the highlight; the sink's `TreeUpdate` carries the selection and drops it while a preedit composes; a refused value action costs an idle frame (button and input both refuse honestly); the `a11y_action/route_100_setvalue_1k` gate is published; corpus 48 → 49 | an AT needs a selection shape the value-bearing node cannot express → grow the snapshot vocabulary behind the same refuse-don't-munge contract |
| M12 | cont. | The pointer joins the editing contract (ADR-0018): the resolved frame reaches `on_pointer_event`, the text seam grows the bounds-aware `hit_test`, a click places the caret at the hit point and a press-drag extends the selection from the anchor | a click inside the field places the caret at the hit offset (edges clamp, ties break to the cluster start); a drag selects and the release keeps the range; an unpressed move is hover, never selection; the drag's selection reaches the AT in the drag's own direction (anchor at the press) and drops while a preedit composes; the `editing/route_100_caret_clicks_1k` gate is published; corpus 49 → 50 | a pointer gesture needs a mapping the event/text seams cannot express → grow the contract behind the same `PAINT \| A11Y` currency (frame on `EventCtx`, `hit_test` on `TextSeam`) |
| M13 | cont. | The word joins the editing contract (ADR-0019): the text seam grows the required UAX #29 segmentation (`word_segments` — no default without the tables), core fixes the hop rule, the editor maps Alt/Ctrl+arrows to word hops with the M11 anchor semantics, word chords delete the hop's span, and a double-click selects the word under the hit | Alt/Ctrl+Left/Right hop word runs (a plain press collapses a live selection first, Shift extends from the anchor); a chord deletes exactly the span the caret would hop; a double-click on a word selects it in the click's own reading order, on a separator it collapses (never wanders); the word selection rides the a11y delta unchanged; the `editing/route_100_word_moves_1k` gate is published; corpus 50 → 51 | a word gesture needs a mapping the event/text seams cannot express → grow the contract behind the same `PAINT \| A11Y` currency (segmentation on `TextSeam`, the chain on `PointerEvent::Down`) |
| M14 | cont. | The clipboard joins the editing contract (ADR-0020): the `ClipboardSeam` carries the OS round trip beside the text seam (no provided implementation — the engine rides arboard in `alacer-platform`, opened lazily), core fixes the single-line paste rule (`paste_text` — every line break pastes as one space), and the editor answers Ctrl/Cmd+C/X/V with the M11 selection rules | Ctrl/Cmd+C writes the selection's exact UTF-16 span and edits nothing; Ctrl/Cmd+X writes it and deletes the range; Ctrl/Cmd+V inserts the sanitized clipboard text, consuming an active selection first; an empty clipboard and a refused write are honest no-ops — Handled, no masks, an idle frame — and a refused cut never deletes; the chords are inert while a preedit composes; the paste rides the a11y delta unchanged; the `editing/route_100_clipboard_ops_1k` gate is published; corpus 51 → 52 | a clipboard shape the seam cannot express → grow the contract behind the same refuse-don't-munge rule (the seam in `alacer-core`, the engine in `alacer-platform`) |
| M15 | cont. | The undo stack joins the editing contract (ADR-0021): the `EditHistory` journal joins the editing rules beside `paste_text` and `word_hop` (pure UTF-16 arithmetic — the replaced span, the removed and inserted text, the selection on both sides; the pure-append coalescing rule; the whole-value reset; the bounded, evicting capacity), the editor routes every committed edit through one journaling funnel, and answers Ctrl/Cmd+Z, Shift+Ctrl/Cmd+Z, and Ctrl/Cmd+Y with the M11 selection semantics | a typed run undoes as one step (a deletion, a paste, a cut, an IME commit are their own); an undo restores the selection the edit consumed and the AT reads it unchanged; a whole-value replacement resets the journal (the app's write is never undone); an exhausted journal is consumed with no masks — an idle frame; the chords are inert while a preedit composes; the `editing/route_100_undo_redo_1k` gate is published; corpus 52 → 53 | an undo gesture needs a mapping the edit funnel cannot express → grow the contract behind the same refuse-don't-munge rule (the journal in `alacer-core`, the funnel in the editor) |
| M16 | cont. | The consumable contract (ADR-0022): the 0.12.0 cut completes — the internal dependency pins and the lockfile ride `[workspace.package]`'s version — `PlatformError`/`BackendError` carry their `source()` chains with escape-hatch help text at the seams that know one, the widgets crate's self-description matches the shipped vocabulary, and the M15 journal gets its consumer demonstration (`examples/undo_redo.rs`: the pure-append rule, the reset rule, the honest refusal, the introspection reads, tracing at `alacer::editing`) | `cargo metadata` resolves the 0.12.0 family as one cut; a failed window start walks a real `source()` chain and the no-adapter message names `ALACER_SOFTWARE=1`; `cargo doc` and the manifest descriptions describe the shipped widgets, not the M0 set; the example compiles in CI and its blessed snapshot pins it; corpus 53 unchanged, PNGs 32 → 33 | a feature milestone lands in a surface that cannot publish, mis-describes itself, or drops its error chains → fix the surface first; the ledger's candidates (announcements, pointer capture, a `Slider`) each need a full corpus cycle |
| M17 | cont. | The demonstrated queue (ADR-0023): the M8 coalescer gets its consumer demonstration (`examples/input_storm.rs` — a custom `StormPanel` probe recording every routed pointer event, the log read back through the one-batch mirror, tracing at `alacer::input`), the living documents re-agree with the checked-in data (the README quotes the baselines' 14.06× and 40.7×, the migration guides map onto 0.12.0, the facade's rustdoc describes the shipped surface), and the error chains complete: a malformed `.alc` walks to its parse cause, and `LayoutError` carries the failing node as data | the example compiles in CI and its blessed snapshot pins it (the m1 corpus grows 31 → 32 scenarios, 33 → 34 PNGs); the README's ratios match `baselines.json`'s medians; a malformed `.alc`'s `source()` downcasts to the `serde_json` cause; `LayoutError.node` is `Option<NodeId>`, not prose; corpus 53 unchanged, no new dependency edges | a feature milestone lands in a surface whose demonstrations, numbers, or error chains lag its own API → fix the surface first; the ledger's candidates (announcements, pointer capture, a `Slider`) each need a full corpus cycle |
| M18 | cont. | The announced delta (ADR-0024): the a11y announcement seam joins the invalidation currency — a reserved live-region node in pass 9's snapshot (present from the first frame, the root's last child, `aria-live` polite/assertive with the ARIA `status`/`alert` pairing), `PassPipeline::announce` for assembly sites and the ambient `announce` queue for handlers (the input queue's shape: push now, the tick routes one frame later, last write wins), the refusals honest (empty text, no root), `examples/announcements.rs` as the consumer demonstration; the surface completes: the last two `String` error leaves carry their causes, the dead convenience APIs (`has_queued_input`, `run_hot_window_with_channel`, the registry accessors) go, and the living documents re-agree | the region rides one node-only delta (the example compiles in CI and its blessed snapshot pins it, the m1 corpus grows 32 → 33 scenarios, 34 → 35 PNGs); an assertive act re-roles to `alert` and a refusal queues no masks; the softbuffer and surface-creation chains downcast to their structured causes; corpus 53 → 54 by the coordinated rule (baseline + regression contract published); no new dependency edges | the region is a second invalidation channel or announcing moves the tree's shape → collapse it into the widget vocabulary; a speech act needs a shape the sink cannot express → re-open the seam with the shape as a property on the snapshot node |
| M19 | cont. | The captured drag (ADR-0025): the pointer capture joins the dispatch contract — a press implicitly holds its deepest accepting hit, the gesture's Move/Up/Leave stream routes through the captured node's ancestor chain (capture→bubble, the same phases), the Up/Leave and the target's death release it, the wheel stays hover-routed, a frame with no capture is the hit test exactly as before, `PassPipeline::captured` introspects, tracing at `alacer::capture`, and `examples/drag_capture.rs` is the consumer demonstration; the surface work rides the cut (the dead `EventCoalescer::pending` goes, the `rsx!` single-root diagnostic names its token) and the living documents re-agree | the drag past the target's frame still routes and the m19 exit suite pins it headlessly; the captured release completes the armed button's click; a wheel during a capture scrolls the hover; the uncaptured storm routes by the hit test alone (the count gate); no new dependency edges; the criterion case `input/route_100_captured_moves_1k` and the example's blessed-corpus scenario join at the measurement run by the coordinated rule (the checker fails an uncontracted benchmark — CONTRIBUTING's publishing rule) | a gesture needs a route the single capture slot cannot express → grow the event vocabulary with the pointer id first, then the capture table (the ADR's re-open rule) |
| M20 | cont. | The valued widget (ADR-0026): the slider joins the widget vocabulary — `Role::Slider` in core (requires-name, focusable, the M3 `value()` contract), one funnel with four doors (drag against the resolved frame, arrows and Home/End, the AT's `SetValue`/`Increment`/`Decrement` through the existing action route) with every write clamped and snapped inside the widget, refusals honest (non-numeric, non-finite, `DefaultPress` — never munged), a no-op write queues no masks, and the a11y sink maps the role faithfully; `examples/slider.rs` is the consumer demonstration; the corpus grows 54 → 55 (the M19-promised capture case measured, published, and gated at this run) and the m1 corpus grows 33 → 35 (the drag-capture and slider scenarios blessed); the living documents re-agree (the README's badge, count, vocabulary, and status block; the catalog; the migration guides; the CHANGELOG) | the AT's write, a drag, and an arrow land in the same value (the m20 exit suite pins the tier headlessly); a non-numeric `SetValue` is refused with an idle frame; the hover writes nothing (the idle count gate); the capture case carries a real 100-sample median and its same-host gate; no new dependency edges | a range or multi-thumb slider needs a vocabulary one value cannot express → re-open the seam with the second thumb priced by a real UI (the ADR's re-open rule) |
| M21 | cont. | The reported value (ADR-0027): the progress indicator joins the value tier as its read half — `Role::ProgressIndicator` in core (requires-name, not focusable, the M3 `value()` contract), two app doors landing the same `PAINT \| A11Y` currency (the binding bridge with the caller's masks; the mutate seam whose `set_value` answers the move it made — a write that lands where the value already is queues nothing), refusals honest (every AT action — a reporting widget has no write door, never munged), and the a11y sink maps the role faithfully; `examples/progress.rs` is the consumer demonstration (the mutate seam's first one, tracing at `alacer::progress`); the corpus grows 55 → 56 (the value-write route's criterion case measured, published, and gated at this run) and the m1 corpus grows 35 → 36 (the reported value's window blessed); the micro-debt the audit priced rides the cut (the dead `PassPipeline::has_observer` and the wgpu backend's dead stats surface go) and the living documents re-agree (the migration guides' 0.12.0 self-description among them; the README; the catalog; the register; the CHANGELOG) | the app's write, the bridge's write, and the tree's read land in the same value (the m21 exit suite pins the tier headlessly); a same-value write costs an idle frame (the count gate); every AT action is refused with the value unchanged; the value-write case carries a real 100-sample median and its same-host gate; no new dependency edges | a report needs a write door → that is the editor's or the slider's contract, not a reporter's — grow the vocabulary there (the ADR's re-open rule) |
| M22 | cont. | The unavailable control (ADR-0028): the availability tier joins the widget contract — one `Widget::enabled` source of truth (the four actionable controls carry it), the default `focusable()` derivation consults it (an unavailable control is never a Tab stop and never a focus candidate), pass 3's sentinel discipline evicts focus from a control that becomes unavailable, every door refuses honestly (the pointer is a wall that never clicks through, the keyboard and IME are answered with silence, every AT action refused — never munged, the M11 rule), the flag rides pass 9's snapshot (`TreeNode.enabled`) and the a11y sink maps it faithfully onto AccessKit's `disabled` property, two doors land the same `PAINT \| A11Y` currency (the fixed `set_enabled` answering the move it made — a write that lands where the flag already is queues no masks; the button's `enabled_binding` bridge door), the look tells the truth (the muted paint the theme constants borrow — the shipped corpus's enabled pixels untouched), `examples/disabled.rs` is the consumer demonstration (tracing at `alacer::disabled`); the corpus grows 56 → 57 (the refused route's criterion case measured, published, and gated at this run) and the m1 corpus grows 36 → 37 (the availability window blessed); the micro-debt the sweep priced rides the cut (the software renderer's dead `glyphs_rendered` — the M21 gate's own blind spot — plus the dead `PassPipeline::clear_observer`/`timings_enabled` and the `ViewRoot::root` duplicate with its lying doc) and the weak errors reach the ADR-0025 species' remainder (the `rsx!` fall-through names the token; the unterminated-`{` help names the escape) and the living documents re-agree (the README's count, bullet, table, and status block; the catalog; the register; the roadmap; the migration guides' snapshot counts; the CHANGELOG) | the AT reads the flag and every door refuses (the m22 exit suite pins the tier headlessly); a refused gesture costs an idle frame (the count gate); focus never rests on an unavailable control; the refused route carries a real 100-sample median and its same-host gate; no new dependency edges | a control needs a third availability state → grow the vocabulary as a snapshot property, not a second flag; a door needs a shape the refuse-don't-munge rule cannot express → re-open the seam with the shape as a trait method beside `enabled` (the ADR's re-open rule) |
| M23 | cont. | The frame's ledger (ADR-0029): the scene journal becomes legible to its consumers — a required `SceneJournal::stats()` carrying the `JournalStats` vocabulary (commits, drops, live layers), `PassPipeline::journal_stats()` as the pipeline's one read door, and `examples/frame_ledger.rs` as the consumer demonstration (tracing at `alacer::frame_ledger`); the blessed trio repaired (the `list_scrolled_120`/`400` scenarios never fed `set_content_height`, so the scroll clamp folded every offset to zero and three distinct scroll positions blessed one identical image — the scenarios grow the range and feed the clamp from the laid-out children, and the next bless run re-blesses them distinct, CONTRIBUTING's diff-is-the-review rule); the micro-debt the sweep priced rides the cut (the journal's dead inherent accessors replaced by the seam, the todo canon's dead `footer_node`, the `rsx!` body-cardinality diagnostic names its cardinality, the inspector's swallowed session errors reported through the adopted `tracing` core); and the living documents re-agree at 0.14.0 (the README's badge, status block, quickstart, perf bullet, and table; the catalog; the register; the roadmap; the migration guides' family line; the widgets manifest's shipped vocabulary; the m-suite pins move together) | one ticker write re-commits exactly one layer while the static layer replays untouched and nothing churns (the m23 exit suite pins the delta headlessly); idle frames write no ledger entries; the three blessed list scenarios are pairwise-distinct images; no new dependency edges, no corpus growth (the demonstration shows no new visual arrangement) | a ledger read per frame on the hot path → that is the damage report's job; grow `FrameReport`, never a second channel; a journal shape the `JournalStats` vocabulary cannot express → grow the struct as a snapshot field, never a second read channel (the ADR's re-open rule) |
| M24 | cont. | The focus walk joins the dispatch contract (ADR-0030): Tab's private cycle becomes public vocabulary — a `FocusMove` direction (`Next`/`Prev`) and one `PassPipeline::focus_move` door the keyboard's Tab/Shift+Tab fallback and the consumer's code both ride (one funnel, two doors — the walk's order never duplicated at the call site), the walk skips unavailable controls (the M22 cycle-time `enabled` check), wraps at both ends, resolves immediately (key routing sees the new focus this event), and refuses honestly when no candidate exists (no masks, an idle frame, never a first-candidate reset — the M11 rule, at the focus seam), `PassPipeline::focus()` stays the one read door, `examples/focus_traversal.rs` is the consumer demonstration (tracing at `alacer::focus_traversal`); the micro-debt the sweep priced rides the cut (the flux runtime's dead `effect_count` introspection, the harness's dead `spec_of`/`with_text_context`/`text_size` probes) and the living documents re-agree at M24 (the README's status block, count, features bullet, quickstart, and table; the catalog; the register; the roadmap; the m1 header's snapshot count that lagged its own pin; the m20–m23 suites' document pins; the CHANGELOG) | the walk lands where Tab lands, in the same order (the m24 exit suite pins the two doors against each other headlessly); an unavailable control is never a stop and the walk skips it without a recompute; a refusal with no candidate costs an idle frame; one move costs one frame (the ring repaint lands, the frame after is idle); no new dependency edges, no corpus growth (the demonstration shows no new visual arrangement — the catalog's own rule), and the focus-move route's criterion case joins at the measurement run by the coordinated rule (the checker fails an uncontracted benchmark — CONTRIBUTING's publishing rule) | a walk needs an order the tree order cannot express (spatial/D-pad navigation) → grow the vocabulary as a second door priced by a real UI, never a second pipeline channel; a consumer needs the walk bounded to a subtree (a modal's focus trap) → re-open the seam with the scope as a predicate beside `focusable` (the ADR's re-open rule) |
| M25 | cont. | The focus scope (ADR-0031): the walk bounds to a subtree — ADR-0030's pre-registered re-open rule ships as `PassPipeline::focus_move_in(scope, movement)`, the modal's focus trap as the real UI that prices it: the scope is the subtree's root node (the cheapest honest predicate specialization — no closures in the public API, no second walk), the scoped door rides the same generalized walk as Tab and `focus_move` (`cycle_focus_in`, one funnel, two scopes — the pre-order candidate list, the M22 cycle-time `enabled` filter, the wrap at both ends, the immediate resolution), the scope bounds the walk never the candidate list (`focusables()` and `focus()` keep answering the whole tree, the unscoped door stays the keyboard's funnel), refusals honest and named (a dead scope, an empty trap, a trap the availability tier holds down — `false`, no masks, an idle frame, plain help text on the `alacer::focus` target, ADR-0029's rule), `examples/focus_trap.rs` is the consumer demonstration (tracing at `alacer::focus_trap`); the micro-debt the sweep priced rides the cut (the layout mapping's dead `fixed_size_of`, the color's dead `to_f32_rgba` documented for a GPU-uniform consumer that never landed) and the living documents re-agree at M25 and 0.16.0 (the README's badge and status block riding the pending-cut convention, count, features bullet, quickstart, and table; the catalog; the migration guides' family line; the register; the roadmap; the m7 release-state pin re-joining the chain after missing the M23/M24 sweeps; the m20–m24 suites' document pins; the CHANGELOG) | the scoped walk stays in the trap and lands where the unscoped door lands at the root (the m25 exit suite pins both headlessly); an unavailable control inside the trap is never a stop; every refusal costs an idle frame and never munges focus; one move costs one frame (the ring repaint lands, the frame after is idle); no new dependency edges, no corpus growth (the demonstration shows no new visual arrangement — the catalog's own rule); the focus routes (`input/route_100_focus_moves_1k` from M24's cut, `input/route_100_scoped_moves_1k` from this one) join the published corpus at the measurement run by the coordinated rule (the checker fails an uncontracted benchmark — CONTRIBUTING's publishing rule) | a scope needs a shape the subtree root cannot express (a predicate over subtrees, a trap with holes) → grow the scope vocabulary beside `focus_move_in`, priced by a real UI, never a second walk; a door needs to observe the walk's refusals as data → grow the answer into a result vocabulary priced by that consumer (ADR-0030's rule stands) |
| M26 | cont. | The directional focus walk (ADR-0032): the walk reads the window, not only the tree — ADR-0030's pre-registered re-open rule (spatial/D-pad navigation) ships as four `FocusMove` directions (`Left`/`Right`/`Up`/`Down`) riding the same two doors (`focus_move`, `focus_move_in`) and the same one walk (`cycle_focus_in(scope, movement)`): the tree-order moves keep the pre-order list and their wrap, the directional moves read the bounds pass 6 wrote (`directional_neighbor` — a candidate qualifies fully beyond the focused node's edge in the direction, the nearest edge distance wins, cross-axis center misalignment breaks ties, then tree order), the M22 availability filter runs before the geometry answers, the scope bounds the directional walk exactly as it bounds the tree-order one, space never wraps (the window's edge is the walk's end — an edge refusal queues no masks, an idle frame), the refusals are honest and named on the `alacer::focus` target (ADR-0029's rule), and the entry contract is repaired to what ADR-0031 has documented since M25 — from no focus, or from a focus outside the filtered candidate list, the walk takes the list's first candidate, both directions (the M25-era arithmetic landed the second forward and the last backward), `examples/focus_direction.rs` is the consumer demonstration (tracing at `alacer::focus_direction`); the micro-debt the sweep priced rides the cut (the selection's dead `is_degenerate`, the color's dead `to_premultiplied_rgba8` whose only caller was its own test — the M25 species) and the corpus debt the M24/M25 coordinated rule left rides the cut: the promised focus routes (`input/route_100_focus_moves_1k`, `input/route_100_scoped_moves_1k`) join the published corpus beside the directional route (`input/route_100_directional_moves_1k`) at this cut's measurement run, the corpus pins moving 57 → 60 across the m5–m25 chain in the same commit, and the living documents re-agree at M26 and 0.16.0 (the README's badge and status block riding the pending-cut convention, count, features bullet, quickstart, perf bullet, and table; the catalog; the register; the roadmap; the §10.3 focus paragraph; the m-suite pins; the CHANGELOG) | the directional walk lands the geometry's answer, not the tree's next stop (the m26 exit suite pins the two against each other on a real two-by-two grid); an unavailable neighbor is skipped and the far column's control lands; a corner refusal costs an idle frame and never wraps while `Next` still wraps; the scoped directional walk stays in the trap; the entry contract lands the first candidate from outside the list; one move costs one frame (the ring repaint lands, the frame after is idle); no new dependency edges, no m1 corpus growth (the demonstration shows no new visual arrangement — the catalog's own rule) | a walk needs an order neither the tree nor the edge-distance heuristic can express (a true 2-D grid with occlusion or warping) → grow the selection vocabulary beside `directional_neighbor`, priced by a real UI, never a second pipeline channel; a door needs to observe the walk's refusals as data → grow the answer into a result vocabulary priced by that consumer (ADR-0031's rule stands) |

**Portfolio kill criterion (the whole thesis):** if by end of M3 the shared-harness benchmarks (W1, W2, W3) show **no workload where Alacer beats both egui (the DX baseline) and the best audited retained stack on the same hardware** — or the a11y CI gate cannot be maintained under feature velocity — then "composition beats specialization" is falsified. Pre-registered pivots: (a) extract the invalidation engine as a Masonry/egui backend crate; (b) reduce scope to an a11y + testing supercrate for egui/Iced. Killing a thesis cheaply is cheaper than marketing a failed one.

---

## 19. Risks & open questions

Each open question carries its cheapest validation experiment (VISION.md Q1–Q6, restated with owners):

| # | Assumption | Risk | Cheapest validation |
|---|---|---|---|
| Q1 | Taffy can host torin-style ancestor-candidate relayout externally | High — Pillar 3 | 3-day spike over a persistent `TaffyTree`; measure relayout calls/frame (Floem's text-node `mark_dirty` works out-of-tree — encouraging precedent) |
| Q2 | reactive_graph can drive paint scheduling without a second scheduler | Medium | M0 metric (ADR-6 kill-switch); GPUI draw-tracking is the documented fallback |
| Q3 | subsecond is CI-grade for a non-Dioxus framework | Medium | I3 CI hot-patch suite in M4; template-only fallback documented |
| Q4 | Per-item paint trackers on a GPU path without Slint's rewrite cost | Medium-High | journal-replay-only partial paint first; add trackers only if fill rate still dominates |
| Q5 | Required a11y methods won't poison ergonomics | Low-Medium | ship `#[derive(Widget)] #[widget(role = …)]` early; gather feedback at 0.1 |
| Q6 | A maintained bench rig stays honest across winit versions | Low | the rig is M0 infrastructure; baselines-as-contract from day 1 (Floem precedent) |
| Q7 | *(new)* reactive_graph's maintainer set stays responsive | Medium | upstream-first contributions from M0 build goodwill; the crate is small and MIT — a fork under our license is the cheap last resort (explicitly exempted from A5's "never fork" by ADR-6's fork-path note) |
| Q8 | *(new)* tiny-skia partial paint matches GPU damage semantics exactly | Medium | the equivalence fuzz (§9.3) runs against both backends from M2 |

---

## 20. Provenance, licensing, and honesty

**Adopted (dependencies, all MIT and/or Apache-2.0 — verified in `audit/LICENSES.md`):** `reactive_graph`, `generational-box` (via signal handles), `taffy`, `parley`/`fontique`/`swash`, `accesskit` (+ `accesskit_winit`), `winit`, `wgpu`, `tiny-skia`, `softbuffer`, `subsecond`, `muda`, `rfd`, `arboard`, `tray-icon`, `notify`, `raw-window-handle`, `euclid`, `tracing`, `criterion`, `dhat`.

**Ported as algorithms (IDEA/ALGO):** torin's incremental relayout semantics (MIT code exists — we port onto Taffy, adapting); Slint's partial-renderer damage algorithm (**clean-room from documented behavior** — its source is GPL/commercial; provenance note required per file); Floem's event-spec structure and `TrackedRefCell` (MIT, CODE); GPUI's scene-journal and id-hash designs (Apache-2.0, CODE); egui's integer wrap-rounding lesson (ALGO); Iced's damage ladder and record/replay (ALGO/IDEA); Blinc's statechart/spring primitive patterns (IDEA — maturity audited first, 560★ new project).

**Clean composition (owned, original):** the `Change` mask currency and its three consumers; the pass contract; the a11y type contract + CI walker; the Causality inspector; the CI hot-patch suite; the budgets-as-contracts regime.

**Honesty rules inherited from the audit:** no benchmark numbers exist in this document — every target is labeled **[budget]** and unmeasured until the harness produces it; every cross-framework claim carries a pinned-SHA citation; residual unknowns are marked **UNVERIFIED** (Makepad a11y interop; Dioxus ThinLink timing; GitHub star counts scraped from HTML for 6 repos; Slint generated-code compile times). License posture: Alacer ships `MIT OR Apache-2.0`; `cargo-deny` enforces no-AGPL/no-unknown-license deps; GPL sources are never copied.

---

## 21. Glossary

| Term | Meaning |
|---|---|
| **Alacer** | this framework; Latin *swift, eager* (§2) |
| **Flux** | the reactive layer around `reactive_graph`: signals, memos, effects, owners |
| **Change mask** | per-node bitflags `LAYOUT | PAINT | A11Y` — the single invalidation currency (A1) |
| **Scene journal** | append-only, ordered record of paint operations; the replayable source of truth for both renderers (§9.1) |
| **Pass** | one explicit, ordered tree traversal of the pass contract (§7.2) |
| **Owner** | reactive_graph scope that owns effects/values and disposes them in order; survives hot-patch frames (§5.1, §13.2) |
| **RenderEffect** | a synchronous-first-run effect bound to one dynamic render slot (§5.1) |
| **id path / prefix hash** | structural identity of an element; its hash doubles as hit-test id and AccessKit `NodeId` (§6.3) |
| **Peel → key map → LIS** | Dioxus-derived keyed reconciliation pipeline (§6.2) |
| **Purity panic** | debug-mode check that a cached measurement equals a forced re-measure (Masonry pattern, §8.3) |
| **Translation-only update** | repositioning without re-measure when sizes are unchanged (torin pattern, §8.2) |
| **Baselines-as-contracts** | committed criterion/perf numbers whose regression fails CI (A6) |
| **[budget]** | a design commitment, CI-enforced, not yet a measurement |
| **UNVERIFIED** | a claim the audit could not check in its environment; carried honestly forward |

---

## 22. Appendix: evidence index

Where each architectural decision meets its evidence (audit bundle paths):

| § | Topic | Primary evidence |
|---|---|---|
| 1 | Thesis numbers | `audit/MATRIX.md` §3–6; `audit/INDEX.md` |
| 3 | Axioms A1–A8 | GPUI `window.rs:3743`; Freya `render_pipeline.rs:58`; Iced `wgpu/src/window/compositor.rs:230-264` vs `tiny_skia/…/compositor.rs:149-185`; Masonry `widget.rs:424-435`; Floem `event/README.md:334-412`; egui `glyph_atlas.rs:381-395`, `id_type_map.rs:573-601`; Iced `Cargo.toml:206, 207, 249` (forks), `Cargo.toml:253-254` (lint deny); Floem `Cargo.toml:65` |
| 5 | Reactivity | `audit/leptos/REPORT.md` §B; `reactive_graph/src/{graph/sets.rs, channel.rs, computed/inner.rs, owner.rs, effect/render_effect.rs}`; Freya `lifecycle/{memo,effect}.rs`; Floem `reactive/src/signal.rs`; GPUI `app.rs:1459-1466, 1468-1495` |
| 6 | Views & identity | `audit/xilem-masonry/REPORT.md` §A; `audit/dioxus/REPORT.md` §B/D; GPUI `element.rs:212-255`; Dioxus `diff/iterator.rs`, `packages/oracle`, `packages/fuzz` |
| 7 | Arena & passes | `audit/xilem-masonry/REPORT.md` §A; `masonry_core/src/doc/pass_system.md`; `tree_arena`; `widget_mut.rs:22-43`; GPUI `entity_map.rs` |
| 8 | Layout | `audit/freya/REPORT.md` §C (torin); Masonry `layout/{measurement_cache,len_req}.rs`, `passes/layout.rs:72-99`; Floem `views/label.rs:214-218`; egui `galley_cache.rs:41-64` |
| 9 | Rendering & text | GPUI `scene.rs`; `audit/slint/REPORT.md` §D (`partial_renderer.rs`); Floem `renderer/src/lib.rs:103-150`; `audit/PRIMITIVES.md` P2/P4; ADR-2/ADR-4 |
| 10 | Input | Floem `src/event/README.md`; `understory` grid index; Freya `ragnarok/src/event_buffer.rs`; egui `egui-winit/src/lib.rs:741-763, 795-806` |
| 11 | Accessibility | `audit/INNOVATE.md` I1; Masonry `widget.rs:424-435`, `passes/accessibility.rs`; Freya `accessibility/dirty_nodes.rs`; egui `egui_kittest`; ADR-5 |
| 12 | Platform | ADR-1; `audit/PRIMITIVES.md` P1/P8; KAS `data_transfer.rs:4-7` |
| 13 | DX | `audit/dioxus/REPORT.md` §G (subsecond, oracle); Leptos `render_effect.rs:175-216`; Freya `freya-testing`; Masonry `masonry_testing`; Iced `.ice`; `audit/INNOVATE.md` I3/I5/I6/I7 |
| 14 | Performance | `audit/BENCHMARKS.md` §2–5; Floem baselines; Makepad `platform/hal/src/lib.rs:13-15`, `widget_tree.rs:14-17` |
| 15 | Non-goals | VISION.md; `audit/STEAL.md` anti-steals; Slint compiler LOC; Makepad `Cargo.toml:458-465` |
| 16 | Moats | `audit/INNOVATE.md` I1/I4/I5; VISION.md moats |
| 18 | Roadmap | VISION.md M0–M5; `audit/BENCHMARKS.md` §5 |

*Every citation resolves to a verified `file:line` at a pinned SHA recorded in `audit/INDEX.md`. Where the audit said `UNVERIFIED`, this document says `UNVERIFIED`. Nothing here is believed; everything here is cited, budgeted, or falsifiable.*

---

*Alacer · architecture draft 1 · 2026-11-05 · from audit to spine in one document.*
