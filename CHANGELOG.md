# Changelog

All notable changes to **Alacer** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
generated from Conventional Commits by [git-cliff](https://git-cliff.org).

## [Unreleased]

## [0.5.0] — 2026-11-06

### Added

- **M5 — criterion baselines published as contracts (ADR-0011)**: all 38
  criterion benchmark cases across the five benching crates distilled into
  `benchmarks/baselines/baselines.json` by `scripts/collect_baseline.py` —
  median/mean/MAD/sample count per case plus the environment that produced
  them (git commit, dirty flag, toolchain, host fingerprint); the
  architecture's falsifiable gates published as measured **budget
  contracts** (W1 ≥ 2× incremental relayout, measured 14.01×) and one
  **same-host regression gate** per case (fresh median ≤ published × 1.25
  on a matching host fingerprint; foreign hosts skip by policy,
  `--strict-host` for pinned hardware); the uncontracted-benchmark rule
  fails any bench case without a published gate
- **M5 — the perf-contract checker** (`scripts/check_perf_contracts.py`):
  enforces the budgets everywhere (ratios are host-independent), the
  same-host regressions on matching fingerprints, and the contracts ↔
  baseline bijection; exit codes for CI
- **M5 — the perf dashboard** (`scripts/perf_dashboard.py` →
  `benchmarks/dashboard/index.html`): one self-contained HTML file — the
  measured environment, budget contracts with ratios and PASS/FAIL, the
  full baseline table with delta bars, and the four test-enforced gates
  (pass overhead < 15%, TreeUpdate p99 < 0.5 ms, the patch frame budget,
  zero-measure scrolling) each naming its enforcement site; CI
  regenerates and uploads it on main
- **M5 — the examples gallery**: `crates/alacer/examples/gallery.rs` — the
  whole core widget vocabulary (labels, buttons, text input, panels,
  scrolling list, rows, spacers, the shared theme) in one window through
  the canonical write seam (signals in, masks out, one batch one frame);
  `docs/examples.md` catalogs every example with its contract and the
  bless workflow
- **M5 — the egui/Iced migration guides** (`docs/migration/from-egui.md`,
  `from-iced.md`): the two audited reference frameworks mapped
  section-for-section (mental model, app shell, widget mapping,
  reactivity, layout, input and events, accessibility, testing, interop
  limits) with real API on both sides and the version-scope honesty note
- ADR-0011: the M5 scope decision record
- The M5 exit suite (`alacer-testing/tests/m5_exit_criteria.rs`): the
  baseline schema + environment header, the contracts ↔ baseline bijection
  at the pinned corpus size (38), the budgets holding on the published
  data (including the architecture's own 2× W1 value), dashboard
  coverage, catalog coverage, the gallery exercising the widget
  vocabulary through `Flux::batch_of` + `RenderBinding`, and both guides
  carrying the required section set

### Changed

- The workspace ships as 0.5.0
- CI gains the `perf-contracts` job: benches run at reduced sampling, the
  budget contracts are enforced against the published baseline, and the
  dashboard is regenerated and uploaded as an artifact
- `alacer-testing` gains a `serde_json` dev-dependency (the M5 exit suite
  reads the published contracts and baseline)

## [0.4.0] — 2026-11-06

### Added

- **M3 — the a11y contract**: the a11y pass produces `A11yDelta`s (updated /
  removed / focus / initializes); AccessKit `TreeUpdate`s stay dirty-node-only,
  removals surface exactly once, focus travels in the update, editables expose
  `value()`; the §18 gate — incremental TreeUpdate p99 < 0.5 ms @ 1k nodes —
  is test-enforced
- **M3 — partial paint**: the damage model (`alacer-core::damage`, ≤3 regions,
  least-growth merging, NONE/partial/FULL ladder) computed by the paint pass
  from old journal extents + new primitive extents, carried in
  `FrameReport::damage`; software backend repaints regions onto a persistent
  framebuffer, wgpu backend `LoadOp::Load`s an offscreen frame with scissor
  rects; the equivalence fuzz pins partial == full pixel-exactly (Q8)
- **M3 — the IME contract**: UTF-16-native key/IME event vocabulary, the
  `utf16` arithmetic seam (mid-surrogate offsets floor, never slice),
  focus-routed `dispatch_key`/`dispatch_ime` with Tab cycling, per-frame
  `ImeRequest` (purpose + caret anchor), the winit byte→UTF-16 platform seam,
  and the `TextInput` reference widget (caret editing, inline preedit, commit,
  secure mode, focus ring, a11y value)
- `TextInput` widget + the `text_input` example
- ADR-0009: the M3 scope decision record
- **M4 — the hot-reload contract (§13.2, ADR-7)**: `alacer-view::HotView`,
  the view-generation runtime — a patch swaps the generation, the
  reconciler re-renders scoped (patch-in-place, keyed LIS, exact
  differences), and state survives by construction; the facade's
  `hot-reload` feature links `subsecond`, wraps generations in
  `subsecond::call` (the jump-table detour point) and registers the patch
  handler; the platform's `PatchChannel` wakes the winit loop from any
  thread and answers one patch event with exactly one scoped re-render
  (`PlatformEvent::Patch` + `AlacerApp::set_patch_hook`)
- **M4 — the CI hot-patch suite**: `alacer-testing/tests/m4_exit_criteria.rs`
  pins the framework's half of the contract headlessly — a patched
  generation renders its new values with the old state (signal, binding,
  node identity), a patch costs exactly one layout+paint cycle and leaves
  the tree idle (A7), ten-generation churn with interleaved mutations stays
  consistent, untouched subtrees stay out of the patch frame, and the
  unobserved path stays silent (capture off, zero recording); the facade's
  `hot_reload_subsecond` tests pin the subsecond passthrough headless, and
  the `hot_reload` example is the sub-second demo (`dx serve --hotpatch`
  upgrades it to real binary patching, no code change)
- **M4 — the Causality inspector (§13.3)**: the `alacer-core::inspect` seam
  (`InspectEvent`: edge activations, provenance writes, routed inputs,
  frame summaries with the pass ledger + the dirty-node mask heatmap) and
  the `alacer-inspect` crate — `CausalRecorder` over bounded ring buffers
  (every ring's eviction is test-proven, §14.3), a dependency-light RFC
  6455 WebSocket server (`sha1` + `base64`, no async runtime), the
  documented WS protocol (`docs/inspector-protocol.md`, version 1), and the
  embedded single-file HTML client; the five §13.3 questions — why did
  this render, who wrote this signal, who subscribes, why is this frame
  slow, what did this mask touch — answered from recorded truth;
  `Flux::write` / `FluxWriter` is the opt-in write-provenance path;
  `Flux::set_causal_capture` gates activation capture; the protocol suite
  (`alacer-inspect/tests/protocol.rs`) pins the handshake (the RFC 6455
  reference vector), the greeting shape, streaming order, and the controls;
  the `inspector_demo` example is the live demo
- ADR-0010: the M4 scope decision record

### Changed

- `A11ySink::apply` takes a shared `Arc<TreeSnapshot>` (built once per a11y
  run — no per-frame deep clone) and an `&A11yDelta`
- `SceneJournal` gains `layer(node)` — the damage pass reads the layer it is
  about to replace
- Pass 3 notifies focus endpoints (`Widget::set_focused`) and merges their
  PAINT masks same-tick; the tick drains all masks pass 3 merged
- `Widget` gains `on_key_event`, `on_ime_event`, `set_focused`, `ime_purpose`,
  `value` (all defaulted)
- Buttons take keyboard focus on press (click-to-focus, §10.3)
- The workspace is 12 crates (ADR-9's target): `alacer-inspect` joins as
  the devtools layer (`alacer-core` only; `scripts/check_dag.py` enforces
  the order; `alacer-testing` may consume it for the M4 suite)
- `PassPipeline` gains `set_observer` / `clear_observer` — the devtools
  sink; `None` in every default path (the A7 idle contract and the M1
  overhead gate are unchanged and re-verified)
- The workspace ships as 0.4.0 with the full crates.io metadata set on
  every manifest

## [0.3.0] — 2026-11-05



### Added


- Add the Change mask invalidation currency

- Add the single cast seam

- Add shared euclid-backed geometry types

- Add the RGBA color type

- Add id-path prefix hashing

- Add the pointer event vocabulary

- Add typed layout styles and the measure seam

- Add the Widget trait with required role and label

- Add the retained widget arena

- Add the flux bridge over reactive_graph

- Add the scene primitive vocabulary and PaintCtx

- Add the pass contract and PassPipeline

- Wire the core crate public surface

- Add keyed reconciliation

- Add the Alacer-to-Taffy style conversion

- Add the LRU measure cache

- Add torin-style dirty-reason semantics

- Add the persistent Taffy LayoutHost

- Wire the layout crate public surface

- Add the frame-windowed layout cache

- Add the font registry

- Add the zeno glyph rasterizer

- Add the parley facade and thread-local context

- Add the node-layered scene journal

- Add the tiny-skia software rasterizer

- Wire the render crate public surface

- Add the AccessKit sink with incremental TreeUpdates

- Add the Panel decoration

- Add Column, Row and Spacer containers

- Add the Label widget

- Add the Button widget

- Add the scrolling List

- Wire the widget family, theme and BoxedWidget

- Add the event coalescer

- Add the softbuffer present path

- Add the wgpu renderer

- Add the AlacerApp winit driver

- Add the headless TestHarness

- Add the alacer facade crate

- Add u32_to_f32, u64_to_f32 and u128_to_u64 helpers

- Add the EventPhase capture/bubble enum

- Add Widget::focusable and the as_any_mut escape hatch

- Add the FluxRwSignal type alias

- Add the text seam (widgets shape text without alacer-text)

- Add reorder_children, is_live, and drain reporting

- Implement the ten-pass contract with the overhead gate

- Wire text_seam and re-export the M1 pass vocabulary

- Add the ThreadLocalTextSeam adapter

- Add the ViewNode tree and PropValue vocabulary

- Add the WidgetRegistry element table

- Add the ViewRoot in-place reconciler

- Wire the view tree, registry and reconciler modules

- Add the rsx! view DSL proc-macro

- Add in-place setters, as_any_mut and the text seam

- Add style_mut, as_any_mut and Spacer::set_size

- Add in-place setters, as_any_mut and the text seam

- Add style_mut and as_any_mut

- Add set_height, set_color and as_any_mut

- Register the default M0 element set

- Expose the default element registry

- Install the text seam at AlacerApp assembly

- Add the a11y query builder, press, and the text seam

- Expose rsx! and mount_view through the facade

- The root candidate, the bounded publish, and the sync diff

- The cached scene replay


### Benchmarks


- Add arena build/mask/walk benchmarks

- Add flux batch-coalescing and drain benchmarks

- Add reconciliation benchmarks

- Add the incremental vs full relayout benchmark

- Add cold/cached text layout benchmarks

- Add journal commit and replay benchmarks

- Add the §7.2 pass-overhead churn benchmark


### Build


- Scaffold the Alacer workspace

- Pin the initial dependency graph

- Cargo-deny license gate (no AGPL, no unknown licenses)

- Declare the alacer-core crate

- Declare the alacer-view crate

- Declare the alacer-layout crate

- Declare the alacer-text crate

- Declare the alacer-render crate

- Declare the alacer-a11y crate

- Declare the alacer-widgets crate

- Declare the alacer-platform crate

- Declare the alacer-testing crate

- Declare the alacer facade crate

- Register the pass_overhead benchmark target

- Declare the alacer-macro proc-macro crate

- Tighten to core + view (the M1 DAG close-out)

- Add alacer-macro and alacer-view deps

- Add alacer-macro to the facade s dependencies


### Continuous Integration


- Add the crate layering DAG checker

- Add the fmt/clippy/test/doc/deny/dag pipeline

- Add the git-cliff release automation

- Extend the layering for alacer-macro and the M1 tightening

- Ship the counter_view example binary


### Documentation


- Add the counter example

- Add the scrolling list example

- Add the git-cliff configuration

- Add the initial CHANGELOG

- Add the CONTRIBUTING guide

- Add the system architecture document

- Record the M0 scope decision

- Record the workspace discipline decision

- Record the scheduling contract decision

- Record the persistent-Taffy decision

- Record the software-first glyph-seam decision

- Record the testing productization decision

- Record the release automation decision

- Add the counter_view rsx! example

- Record the M1 scope decision

- Add the 0.2.0 release notes

- Update README for the M1 milestone

- Record the M2 scope decision

- The M2 milestone record and the 187-test count


### Fixed


- Treat dead arena slots as not-live during taffy sync

- Repair the broken snapshot corpus — glyph blend, y-convention, stretch, compose


### Miscellaneous


- Ignore build artifacts and editor/OS debris

- Pin the 1.99.1 toolchain with clippy and rustfmt

- Set rustfmt max_width to 100

- Add the Apache-2.0 license text

- Add the MIT license text

- Regenerate the lockfile for the 0.2.0 workspace

- Bless broken pngs

- Readme

- Logo & deny


### Tests


- Add the counter snapshot golden

- Add the scrolled-list snapshot golden

- Add the M0 exit-criteria integration tests

- Implement as_any_mut on the arena bench Node

- Implement as_any_mut on the incremental test Probe

- Implement as_any_mut on the incremental bench TestBox

- Add the M1 exit criteria and 30-scenario corpus

- Add the rsx! end-to-end integration suite

<!-- generated by git-cliff -->
