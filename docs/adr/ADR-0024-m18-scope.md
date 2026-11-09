# ADR-0024: M18 scope — the announced delta (the announcement seam, the last String leaves, the dead conveniences)

- **Status:** Accepted (2026-11-09)
- **Implements:** the ledger's first feature candidate (the a11y
  announcements) plus the surface work the M16/M17 pattern leaves open:
  the last two `String` error leaves, the dead convenience APIs the
  codebase accumulated, and the living documents that drifted again
- **Companion:** ADR-0016 (the refuse-don't-munge discipline this
  extends), ADR-0020 (the clipboard seam whose shape the handler route
  borrows), ADR-0014 (the input queue whose push-now/route-at-the-tick
  shape the handler route borrows), ADR-0022/ADR-0023 (the surface
  discipline this continues), ADR-9 (the dependency discipline that
  keeps miette out), architecture §11.6 (the announcement contract),
  §18 (the M18 roadmap row)

## Decision

M17 made the surface agree with itself; its ledger named three feature
candidates (announcements, pointer capture, a `Slider`), each a full
corpus cycle. M18 takes the first — the announcement seam, the last
M3-era a11y gap: an assistive technology can read and act on the tree,
but an app had no way to *speak* state changes that own no widget — in
the same "one contract widens" shape, while the surface work rides the
same cut:

- **The announcement vocabulary joins core** (`alacer-core/src/announce.rs`).
  The app's voice is a **reserved live-region node** in pass 9's
  snapshot: the same snapshot, the same `A11yDelta`, the same
  `Change::A11Y` currency as every other a11y fact (axiom A1 — no
  second channel). The node is present from the first frame (the root's
  last child, named empty), so announcing never moves the tree's shape —
  one node-only delta, no `STRUCTURE` churn, no ghost removals. The
  politeness is the node's `aria-live` priority
  (`Politeness::Polite`/`Assertive`); the sink maps the ARIA pairing
  (`status` speaks politely, `alert` interrupts) and carries
  `accesskit::Live` on the update. The rules: **last write wins** (one
  region, one spoken fact — never an unbounded queue inside a frame),
  and **refuse, don't munge** (an empty or all-whitespace announcement
  speaks nothing; before a root is mounted there is no tree to attach
  the region to; a refusal queues no masks — an idle frame).
- **Two doors, one destination.** Assembly sites and tests hold the
  pipeline and call `PassPipeline::announce` directly. Event handlers
  cannot hold the pipeline (widgets never touch it — A1), so the
  ambient `alacer_core::announce::announce` fn queues the speech act on
  the calling thread (the clipboard-seam shape, ADR-0020) and the
  dispatching pipeline drains it at the next tick's start (the input
  queue's shape, ADR-0014: push now, the tick routes, one frame). The
  drain takes the batch's last act and clears the rest — no stale act
  leaks into a later tick.
- **The consumer demonstration ships.** `examples/announcements.rs`
  drives the seam the way `undo_redo.rs` drives the journal and
  `input_storm.rs` drives the coalescer: *Save it* announces politely,
  *Delete it* interrupts assertively, *Speak nothing* refuses honestly,
  the status line mirrors what the region carries, and the observability
  rides the workspace's idiom (`tracing` spans at the `alacer::announce`
  target, the politeness and the accept/refuse answer as fields). A
  blessed snapshot pins the demonstration (the m1 corpus grows 32 → 33
  scenarios, 34 → 35 PNGs).
- **The corpus grows by the coordinated rule.** The announcement route
  joins the bench corpus as `announce/route_100_announcements_1k`
  (`alacer-testing/benches/announcements.rs` — one hundred speech acts
  routed on a 1,000-node arena, each `announce` + one tick on the real
  composition): 53 → 54 cases, the baseline published and the
  same-host regression gate added, every M-suite pin moved to 54
  together. No new dependency edge anywhere in the DAG.
- **The last two `String` error leaves grow their chains.** M16 gave
  `PlatformError`/`BackendError` their `source()` discipline and M17
  reached the recording and the layout error; two flattened strings
  remained: `PlatformError::Softbuffer(String)` discarded the
  structured `softbuffer::SoftBufferError`, and
  `BackendError::SurfaceCreation(String)` discarded the (opaque)
  `wgpu::CreateSurfaceError`. Both carry their causes now — a consumer
  walks from the platform error to the present path's real failure, and
  the chain tests pin the downcasts (`SoftBufferError::Unimplemented`
  constructs in a test; wgpu's type is opaque, so its contract pins at
  the type level). `NoAdapter` stays the one honest leaf: a fact about
  the host, not a wrapped failure, and its message still names
  `ALACER_SOFTWARE=1`.
- **The dead convenience APIs go.** Three public items had zero call
  sites — `PassPipeline::has_queued_input` (the platform pushes and
  never drains; the coalescer's count answers through the tick),
  `run_hot_window_with_channel` (the hot-reload contract's consumers
  wire `mount_hot` + the patch hook themselves, exactly what the
  example and the CI suite do), and the `registry()` accessors on
  `HotView`/`ViewRoot` (extend a registry where you own it — at the
  mount call). Deleting is the honest maintenance: a convenience nobody
  uses is a promise the workspace tests without exercising.
- **The living documents re-agree.** The architecture's Contents line
  still counted "20 decision records" (24 files on disk — the M18
  register row makes it 25); the migration guides still said "30
  canonical scenarios"/"30 blessed PNG snapshots" two corpus cycles
  after the counts moved (33 scenarios / 35 PNGs); the journal's
  `SceneBatches` doc claimed "the GPU backend consumes this shape
  directly" while the wgpu backend never touches it (it is the bench
  and test materialization shape — the doc now says so); the core
  crate's module doc summarized the pass contract as six passes
  (the ten-pass order is the contract); the README credited
  `counter_view` with "keyed reconciliation" while the example carries
  no keys (the reconciliation it demonstrates is the in-place patch).
  The same class of self-description drift ADR-0022/ADR-0023 fixed,
  found by the same audit-before-the-cut pass.

### 1. Why this is the milestone

The announcement seam is the last M3-era a11y gap and the cheapest
ledger candidate: pointer capture and a `Slider` each need new gesture
machinery, while an announcement needs only a node the snapshot already
knows how to carry — and the sink's live-region mapping is a property,
not a protocol. The surface debts it rides are the exact pattern M16
and M17 established: a feature milestone lands on a surface that can
publish, describe itself, and hand consumers walkable errors — and the
two `String` leaves plus three dead conveniences were the remainder of
exactly that audit. The cheap cut is the announced one: a voice a
consumer can run, errors a consumer can walk to the end of, and no
promise the workspace never exercises.

### 2. The seams that did not move

- **No platform or engine work.** The region lives in the snapshot the
  pass already builds; the platform's adapter drain forwards whatever
  the sink emits, unchanged. AccessKit's `Live` property is a node
  attribute — no protocol work, no adapter negotiation.
- **The DAG is untouched in shape.** `announce.rs` is a core module
  (no new dependency edges); the sink's mapping is a function of types
  both crates already hold; the platform error change swaps a `String`
  for a type its own Cargo.toml already carries; `alacer-testing`
  gains no dependency at all. `tracing` rides the facade's
  *dev*-dependency (the ADR-0022 precedent). Nothing published moves
  beyond the two error variants' payloads (0.x semver, documented in
  the CHANGELOG).
- **No miette adoption.** The ADR-9 discipline stands: the help text is
  hand-rolled — the refusal traces name why (empty speaks nothing, no
  root to join), the `NoAdapter` leaf still names `ALACER_SOFTWARE=1`,
  and the chains a consumer walks are std `Error::source`, not a
  diagnostic framework.
- **The editing contract's rules are untouched.** The announcement
  region shares no state with the editor; a speech act queues no masks
  beyond `A11Y` on its own node, and focus never moves for a speech
  act (the delta's focus field rides unchanged).

### 3. What this does not do

- **No pointer capture and no `Slider`.** The ledger's remaining
  candidates stay candidates; each rides a later milestone with its own
  corpus cycle.
- **No widgets-side announcement API.** The region is the app's, not a
  widget's: no `Widget::announce`, no container-level live region — an
  app that wants a spoken counter announces through the seam; a widget
  that changes state already rides the delta.
- **No crates.io publish.** The family stays publish-*able*, not
  published (the architecture's `UNVERIFIED` marker stands).

## Consequences

- The app's voice is a first-class a11y fact: a screen reader attached
  to a real window hears `status`/`alert` updates through the same
  `TreeUpdate` stream as every widget change; headless CI asserts the
  same delta through the harness (the m18 exit suite).
- The examples catalog grows one row and one blessed scenario (32 → 33
  scenarios, 34 → 35 PNGs); the bench corpus grows one case by the
  coordinated rule (53 → 54, baseline + contract published, every
  M-suite pin moved together).
- The error-chain discipline is complete: no public error type in the
  workspace flattens a structured cause into prose. A failed window
  start, a failed present path, a failed surface creation, and a
  malformed recording all walk to their real causes.
- The workspace carries no public API with zero call sites; the
  convenience layer matches what the examples and the suites actually
  wire.
- The architecture's ADR count, the migration guides' snapshot counts,
  the journal's materialization-shape claim, the core pass-order
  rustdoc, and the README's example attributions agree with the checked
  in state — audited before the cut, by the same pass that found M17's
  drift.
