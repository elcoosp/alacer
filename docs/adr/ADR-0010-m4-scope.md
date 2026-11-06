# ADR-0010: M4 scope — the hot-reload contract, the Causality inspector, and publish readiness

- **Status:** Accepted (2026-11-06)
- **Implements:** the roadmap's M4 bullet (architecture §18): "DX: subsecond +
  CI hot-patch suite; Causality inspector spike; publish 0.1 + this
  architecture doc"
- **Companion:** ADR-7 (hot reload — adopted-as-is with the template-only
  fallback), ADR-9 (workspace discipline — the 12-crate DAG this lands),
  ADR-0005 / ADR-0008 / ADR-0009 (M1–M3, the infrastructure M4 builds on)

## Decision

M4 spends its budget on the two developer-experience moats the audit
identified — state-preserving hot reload that CI *proves*, and the Causality
inspector that only the framework can build — and closes the release loop.

### 1. The hot-reload contract (§13.2, ADR-7)

`subsecond` is adopted as-is: the facade crate gains an optional dependency
and a `hot-reload` feature; the view layer gains the **generation contract**;
the CI suite pins the framework's half of the deal.

- **The generation contract (`alacer-view::HotView`).** A `HotView` is a
  [`ViewRoot`] plus a swappable view-function generation.
  `HotView::regenerate` re-runs the current generation and reconciles it
  through the existing patch-in-place diff — the scoped re-render §13.2
  requires: structural changes mount/remove exactly the differing subtrees,
  prop changes patch in place, and every touched node enters the frame as
  `Change` masks (the one currency). No world rebuild, no signal churn,
  no dropped widget state.
- **State survives by construction.** All UI-adjacent state lives outside
  the patched code: signals in the flux's reactive `Owner`, dynamic values in
  `RenderBinding` slots, retained widget state (text, focus, scroll) in the
  arena keyed by the id-path identity. A patch swaps *code*; it can never
  drop *state*. The CI suite asserts this over patch churn with interleaved
  mutations.
- **The subsecond wiring (`alacer::hot`, feature `hot-reload`).** The
  example-facing integration wraps the view closure in `subsecond::call`
  (the detour point when a jump-table patch lands), registers a patch
  handler via `subsecond::register_handler` that raises a polled trigger,
  and the run loop answers a trigger with exactly one `HotView::regenerate`
  — "on patch, `RenderEffect`s re-dirty instead of rebuilding the world":
  the reconciler queues masks, the next tick is one layout+paint cycle,
  the frame counters stay at one.
- **The CI hot-patch suite (`alacer-testing/tests/m4_exit_criteria.rs`).**
  Every assertion the architecture demands, run headless against the real
  pipeline: a patched generation renders its *new* value with the *old*
  state intact; node identity is stable across patches (the id-path
  contract); a patch costs exactly one layout+paint cycle and leaves the
  tree idle after; patch churn (10 generations, interleaved clicks and
  typing) ends fully consistent; untouched widgets do not repaint.
  The suite is the kill-switch tripwire: if a patch class ever segfaults or
  drops state, it fails before users feel it.
- **What the suite does not claim.** Jump-table *compilation* is ThinLink /
  `dx serve --hotpatch`'s job — outside cargo's model, so the suite proves
  the framework contract (generation swap ⇒ scoped re-render ⇒ state
  intact ⇒ one frame) and the facade proves the `subsecond::call` /
  `register_handler` wiring compiles and passes through headless (no jump
  table loaded ⇒ a direct call — subsecond's documented degenerate mode).
  The `hot_reload` example is the demo: `cargo run` shows the generation
  swap live with state intact; `dx serve --hotpatch` upgrades it to real
  binary patching with no code change. The ADR-7 fallback (template-only
  reload) is *not* triggered: nothing in the suite is unstable.

### 2. The Causality inspector spike (§13.3)

New crate **`alacer-inspect`** (the DAG's twelfth crate, its own layer
between the harness and the facade), three parts:

- **The observation seam (`alacer-core::inspect`).** Core grows the
  devtools vocabulary: `InspectEvent` (`EdgeActivated`, `SignalWrite`,
  `Input`, `Frame`) and the `InspectObserver` sink.
  `PassPipeline::set_observer` receives frame summaries — the tick's dirty
  map (the mask heatmap §13.3 asks for), the pass ledger, damage, and the
  a11y delta shape. The flux records what only it knows: `bind` registrations
  are the reactive graph's node-facing edges, each effect rerun is an edge
  activation carrying the queued masks, and `Flux::write` is the opt-in
  write-provenance helper ("who wrote this signal"). Capture is off by
  default: an unset observer and a disabled capture flag cost a branch.
- **The recorder.** `alacer-inspect::CausalRecorder` implements the sink
  over bounded ring buffers (frames, edges, writes, inputs — every buffer
  has an eviction policy and a test that proves it, §14.3's rule).
- **The server and the client.** A dependency-light WebSocket server
  (RFC 6455 over `std::net`, `sha1` + `base64` for the handshake, serde
  JSON for the payload) that serves the embedded single-file HTML inspector
  at `GET /` and streams the protocol at `/ws`. The protocol is documented
  in `docs/inspector-protocol.md` — the "documented WS protocol" §13.3
  commits to. The inspector answers the five questions with recorded
  truth: why did this render (edge activations → masks → frame), who wrote
  this signal (provenance entries), who subscribes (the edge table), why is
  this frame slow (per-pass ledger nanos), what did this mask touch (the
  dirty-node heatmap with roles and bounds).

**Spike boundary, stated honestly:** the inspector is read-only capture
plus live protocol — no time-travel replay, no write-provenance backtrace
capture, no memo-subscriber reverse walk beyond the edge table. Those are
post-0.4 work; the WS protocol versions itself so the wire can grow.

### 3. Publish readiness (the "publish 0.1" bullet)

The workspace ships as 0.4.0 with the M4 features; release readiness is
kept verifiable: every manifest carries the crates.io metadata set
(description, keywords, categories, repository), every internal workspace
dependency carries a `version` fallback so the packaged manifests stand
alone (`alacer-x = { path = …, version = "0.4.0" }` — the standard
publish-grade pattern), `cargo package --list` is clean for every crate,
and the CHANGELOG documents the release. Full `cargo package` verification
resolves the packaged manifests against crates.io, so it succeeds in the
standard bottom-up publish order (the DAG's leaves first: `alacer-core`,
`alacer-macro`, then upward) — the order a first `cargo publish` run takes
anyway. The version line started at 0.1 as the roadmap wrote it; the
workspace is already at 0.3, so the M4 release continues that series — the
bullet's substance (a publishable, documented crate set) is what this ADR
closes.

## Consequences

- The workspace is 12 crates; `scripts/check_dag.py` gains the devtools
  layer (`alacer-inspect` → `alacer-core` only; the facade re-exports it).
- `subsecond` is optional and feature-gated; default builds do not link it.
- The inspector's overhead is opt-in: `set_observer` is unset in every
  default path (pipeline, harness, platform), and the causal capture flag
  defaults off — the A7 idle contract and the M1 overhead gate are
  unchanged and re-verified by the existing suites.
- The M4 exit criteria live as tests: `alacer-testing/tests/m4_exit_criteria.rs`
  (the hot-patch contract) and `alacer-inspect`'s suite (capture, eviction,
  protocol). Both are green in CI.
