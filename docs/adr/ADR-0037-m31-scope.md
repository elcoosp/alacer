# ADR-0037: M31 scope — the journal rides the causal log: the edit trail

- **Status:** Accepted (2026-11-12)
- **Implements:** the hook the journal's own docs pre-registered — "the
  inspector may *someday* surface the depth" (`history.rs`, the eviction
  seam's note; `undo_depth`, "the inspector's future vocabulary") —
  cashed as a real reader: "a status line, an inspector trail, or a test
  needs the field's journal history → every `EditHistory` action streams
  one record the tick drains to the observer — never a second journal,
  never a re-implementation at the chord site"
- **Companion:** ADR-0034 (the causal trail the edit tier joins), ADR-0036
  (the attribution rule whose shape the record rides), ADR-0021 (the
  journal whose actions stream), ADR-0024 (the announcement drain whose
  shape the facts ride), ADR-0020 (the clipboard-seam shape the
  thread-local queue borrows), ADR-0010 (the protocol's additive
  versioning rule the wire event rides), architecture §13.3 (the
  questions table the echo answers), §18 (the M31 roadmap row)

## Decision

The causal log answers "why did this render", "who wrote this signal",
and — since M28 — "why did focus change". The editing contract's own
history had no route: an undo chord, a refused chord, the app's reset
resolved inside the journal and left no trace a trail reader could
route on, while the journal's own docs carried the open hook ("the
inspector may *someday* surface the depth"). M31 cashes the hook:

- **The journal speaks for itself.** `EditHistory::{record, undo, redo,
  reset}` stream one `alacer_core::inspect::EditRecord` the moment the
  action resolves — the `EditAction` (`Record`/`Undo`/`Redo`/`Reset`,
  `name()` the wire's `snake_case` tag), the `coalesced` flag (a
  `record` folded into the tip — the pure-append rule visible as a
  depth that never moves), the `refused` flag (an exhausted undo or an
  empty redo branch: the refusal streams too, the depths naming the
  journal it did not move — the M28 honesty, the edit tier's own), and
  the `undo_depth`/`redo_depth` the action left. The facts queue on the
  editing thread (`alacer::history` tracing beside them) — handlers
  cannot reach the pipeline (axiom A1), so the journal queues on the
  calling thread and the dispatching pipeline drains at the tick's
  start, the announcement drain's shape (ADR-0024) and the
  clipboard-seam's thread-local discipline (ADR-0020).
- **The trail is dev-mode and bounded, like everything on the seam.**
  The queue notes a fact only while the trail is enabled — one branch
  per action otherwise, the A7 idle contract — and
  `PassPipeline::set_observer` enables it when a devtools observer
  installs. The bound is the journal's own eviction rule (§14.3)
  applied to its echo: `EDIT_TRAIL_CAP` (256 facts per thread, oldest
  dropped first), so an action queued where no pipeline will ever drain
  still terminates. The drain forwards every fact as
  `InspectEvent::Edit` — never collapsed, a journal's history — and
  empties the queue whether an observer listens or not: a fact streams
  once, an absent reader just never sees it.
- **The wire carries the echo, additively.** `WireEvent::Edit`
  (`action`, `coalesced`, `refused`, `undo_depth`, `redo_depth` — the
  focus event's flat shape, `EditAction::name` the single source) — a
  new event kind, compatible by the protocol's own versioning rule
  ("new event kinds and new optional fields are compatible"),
  `PROTOCOL_VERSION` stays 1. The recorder grows the bounded edit ring
  beside its frames/edges/writes/inputs/focus (512, the §14.3
  discipline, `CausalRecorder::edits()` the read door), and the
  embedded client renders the echo in the log pane beside the focus
  trail — the action and the depths named, the refusal rendered the way
  the walk's refusals are.
- **The demonstration drives the journal with the observer riding.**
  `examples/edit_trail.rs` walks the M15 journal's own routes — a typed
  run's coalescing facts, the applied undo/redo, the app-write reset,
  the exhausted journal's refusal — the app mirroring its own
  introspection while the *trail* label mirrors what the log streamed,
  with tracing at `alacer::edit_trail`, the workspace's per-seam idiom.
  The m31 exit suite pins the contract headlessly: the real pipeline, a
  real `TextInput` driven by real key events, the typed run's one fact
  per keystroke, the refused chord streaming with no masks, the AT's
  `SetValue` reset, and the no-observer idle contract.
- **The living documents re-agree at M31 and 0.17.0.** The README's
  status block (M0 through M32, this cut and M32 together), the ADR
  count (thirty-nine), the inspector bullet, the quickstart, and the
  example table; the catalog's row; the register (37 → 39 with this ADR
  and ADR-0038, the ADR-0000 row joining at last); the §13.3 questions
  table; the roadmap row; the protocol document's `edit` section; the
  m-suite pins moving M30 → M32 and 37 → 39 across the chain; and the
  CHANGELOG stages the milestone under `[Unreleased]`.

## Consequences

- The journal's rules are untouched: the coalescing, reset, and bounded
  rules of ADR-0021 answer exactly as before, the m15 suite's pins
  stand, and `record`'s signature is unchanged — the note is the
  journal's own side effect, priced at one branch when the trail is
  off.
- `EditRecord` and `EditAction` are new public core vocabulary — the
  growth the M27/M28 cuts priced the same way: pre-1.0, additive, the
  wire carrying the flat shape.
- The hot path carries no new work by default: the disabled trail costs
  one branch per journal action, the drain runs only while an observer
  is installed, and the published corpus prices the editing routes as
  shipped.
- The perf corpus stands at 62: the echo rides no hot loop the corpus
  does not already price (the journal's actions ride the published
  editing routes), the coordinated rule's no-growth arm (no new
  criterion case, no baseline move, the m5 bijection green as pinned).
- The m1 corpus does not grow: the demonstration shows no new visual
  arrangement (the trail label rides the journal's own window shape),
  and the m1 suite's 37-scenario pin stands.
- No new dependency edges: the echo is core's own vocabulary, the wire
  event is `alacer-inspect`'s, and the spans ride the `tracing` core
  every member already resolves.

## Kill criteria (re-open rules)

- A consumer needs the echo's *content* (the edit's text, the
  selection each step restored) → grow the record beside `EditRecord`
  (the ADR-0034 rule stands), priced by a real reader — never a second
  journal.
- A consumer needs per-field attribution (which of several inputs
  streamed the fact) → grow the record beside the depths, priced by a
  real multi-field reader — never a stringly-typed tag.
- A client needs the echo grouped or filtered by action (an edit pane
  per field, a replay control) → grow the client's rendering beside the
  event the wire already carries (ADR-0010's additive rule), priced by
  that client — never a second wire vocabulary.
