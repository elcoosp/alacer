# ADR-0025: M19 scope — the captured drag (the pointer capture, the last dead convenience, the rsx! diagnostic)

- **Status:** Accepted (2026-11-10)
- **Implements:** the ledger's second feature candidate (the pointer
  capture) plus the surface work the M16–M18 pattern leaves open: the
  one dead convenience API a fresh call-site census still finds, the one
  diagnostic that still discards what it heard, and the living documents
  that drifted again (the README's criterion count)
- **Companion:** ADR-0014 (the input queue and the single-pointer event
  vocabulary the capture rides), ADR-0016 (the refuse-don't-munge
  discipline the release rules extend), ADR-0018 (the resolved-frame
  seam the captured route keeps honoring), ADR-0022/ADR-0023/ADR-0024
  (the surface discipline this continues), ADR-9 (the dependency
  discipline that keeps miette out), architecture §10.1 (the routing
  spec this makes true), §18 (the M19 roadmap row)

## Decision

M18 closed the announcement ledger; two feature candidates remained
(pointer capture, a `Slider`), each a full corpus cycle. M19 takes the
capture — the routing gap both the M12 drag-select and the future
`Slider` fall through: pass 1 hit-tested every event independently, so
a drag that wandered off its target lost its route mid-gesture (the
text selection stopped at the field's edge, and a press-inside /
release-outside button stayed pressed with no release ever arriving) —
while the surface work rides the same cut:

- **The pointer capture joins the dispatch contract**
  (`alacer-core/src/passes.rs`). The rules:
  - **Implicit, single-pointer capture.** Every `Down` holds its
    deepest accepting hit — the target the bubble walk already answers
    first. The event vocabulary carries no pointer id (ADR-0014's
    shape), so one capture slot is the honest width; multi-pointer and
    per-button capture re-open the seam with the id on the event first.
  - **The captured stream rides the ancestor chain.** While a capture
    is held, the gesture's `Move`/`Up`/`Leave` events route through the
    captured node's root→target chain with the same capture→bubble
    phases the hit test walks — only the target comes from the press,
    not from where the pointer is now. The resolved-frame seam
    (ADR-0018) is untouched: `visit_with_phase` still hands the node's
    bounds to the widget.
  - **The release is honest.** The `Up` and the `Leave` end the capture
    on arrival (the widget still sees its event — a pressed button
    answers the release that ends its own gesture); a captured node
    that leaves the arena mid-gesture releases the pointer and falls
    through to the ordinary hit test — no panic, no ghost route.
  - **The wheel and the idle path do not move.** A `Wheel` stays
    hit-tested — it scrolls what is under the pointer, not what the
    press holds (the DOM's shape) — and a frame with no capture in
    flight takes the per-event hit-test path exactly as before: one
    `Option` check on the hot path, nothing else. The M8 storm budget's
    routing shape is untouched.
  - **Introspection and observability.** `PassPipeline::captured()`
    reads the holder (tests, probes, the inspector's input records —
    whose `target` now names the captured node), and the capture and
    its releases trace at the `alacer::capture` target, the workspace's
    per-seam idiom (`alacer::input`, `alacer::editing`,
    `alacer::announce`).
- **The consumer demonstration ships.** `examples/drag_capture.rs`
  drives the seam the way `input_storm.rs` drives the coalescer: a
  custom `DragPanel` probe (a `Widget` with its required role and
  label) records every routed pointer event — press the panel, drag
  *past its edge*, and the log keeps counting while the gesture is
  anywhere in the window; release, wave without pressing, and only the
  moves inside the frame arrive. The observability rides the idiom
  (`tracing` spans at `alacer::capture`, the routed count as a field).
  The contract is pinned by the pipeline's unit tests and the new m19
  exit suite, headlessly.
- **The dead convenience API goes.** `EventCoalescer::pending()` had
  zero call sites workspace-wide — the same class M18 deleted ("a
  convenience nobody uses is a promise the workspace tests without
  exercising"). The coalescer's count answers through the tick and the
  drain, the two routes every consumer actually holds.
- **The weak diagnostic names what it found.** The `rsx!` single-root
  error discarded the offending token ("rsx! expects one root
  element" — for a literal root, a path root, or an empty body alike).
  It now names the token it found and carries the help text the fix
  suggests (`a root is an element like \`column { ... }\``), and the
  sibling error states its `help:` line — the hand-rolled help-text
  discipline (ADR-9 keeps miette out; the text is std, not a framework).
- **The living documents re-agree.** The README still counted "All 53
  criterion cases" one corpus cycle after the M18 bench moved the
  baseline to 54 (`baselines.json`'s `bench_count`); the §10.1 routing
  spec listed "W3C pointer-capture semantics" among what ships before
  any capture existed (the Status paragraph now states the delivered
  contract and names Floem's two-phase capture as the tracked
  evolution); the architecture's ADR count moves 25 → 26 with the
  register's new row; the examples catalog and the README's example
  table carry the demonstration.

### 1. Why this is the milestone

The capture is the cheapest unclaimed ledger candidate and the one the
shipped features already lean on: the M12 drag-select clamps at the
field's edge because the route dies there, and a `Slider` — the last
candidate — is only honest once the drag that sets it keeps routing.
The contract is also deliberately small: no new event variants, no
platform work, no engine work — one field on the pipeline, one branch
in pass 1, and the same phases the hit test already walks. The surface
debts it rides are the M16–M18 pattern's remainder, found by the same
audit-before-the-cut pass.

### 2. The seams that did not move

- **No platform, engine, or a11y work.** The capture is a pipeline
  routing decision; the platform keeps pushing raw events and never
  drains (ADR-0014), the coalescer is untouched, and the a11y sink sees
  nothing new (a capture routes events; it ships no `TreeUpdate`).
- **The DAG is untouched in shape.** The capture state lives on
  `PassPipeline` in `alacer-core`; the example uses the facade's
  existing dev-dependency on `tracing`; `alacer-testing` gains no
  dependency; no published signature changes except the deletion.
- **No miette adoption.** The ADR-9 discipline stands: the diagnostic
  help text is hand-rolled, the refusals trace why, and the chains a
  consumer walks are std `Error::source`, not a diagnostic framework.
- **The editing and announcement contracts are untouched.** A captured
  drag rides the same `EventCtx` the editor already consumes; the
  announcement region, the focus passes, and the pass ledger's shape
  are unchanged.

### 3. What this does not do

- **No `Slider`.** The ledger's remaining candidate stays a candidate;
  it rides a later milestone with its own corpus cycle, and it will
  stand on this one.
- **No per-button or multi-pointer capture, no capture API on
  `Widget`.** The implicit capture is the whole contract this cut; an
  opt-out predicate (beside `accepts_pointer`) or a capture table is
  the ADR's named re-open path, priced when a real gesture needs it.
- **No measurement or blessing in this cut.** The criterion case
  (`input/route_100_captured_moves_1k`) and the example's m1-corpus
  scenario are the milestone's coordinated-corpus remainders: the
  checker fails an uncontracted benchmark and the m1 corpus is
  byte-pinned, so both join at the toolchain run — `cargo bench` +
  `scripts/collect_baseline.py` + the regression gate, and
  `ALACER_BLESS=1 cargo test -p alacer-testing` — per CONTRIBUTING's
  publishing rule. The code half ships complete and green.

## Consequences

- A drag keeps its route: the M12 selection extends past the field's
  edge, a press-inside/release-outside button completes its click
  instead of sticking pressed, and the future slider's thumb follows
  the hand wherever it wanders.
- The idle path pays one `Option` check; the m19 suite pins the
  uncaptured storm's count and the capture's release rules headlessly,
  so CI enforces the contract without a window.
- The examples catalog grows one row (`drag_capture`); the criterion
  corpus stays at 54 until the measurement run publishes the capture
  route's case by the coordinated rule.
- The workspace again carries no public API with zero call sites, and
  no diagnostic that throws away what it heard.
- The architecture's ADR count, the §10.1 routing spec, the README's
  criterion count and example table, and the examples catalog agree
  with the checked-in state.
