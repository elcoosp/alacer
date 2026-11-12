# ADR-0042: M36 scope — the keymap rides the causal log: the consult's answer as data

- **Status:** Accepted (2026-11-12)
- **Implements:** the re-open rule ADR-0041 pre-registered and cashed
  on its own kill-criteria list — the M35 funnel consult answered a
  bare `bool` whose `false` collapsed three different facts (a fired
  shortcut, a table that named no binding, a window that never
  installed one), so the shortcut's firings left no trace a trail
  reader could route on and "why didn't my shortcut fire?" was
  unanswerable from the log — cashed as one record beside the input
  vocabulary the funnel already streams: "a consumer needs the
  shortcut's firings in the causal log → grow the record beside the
  input vocabulary the funnel already streams (`InspectEvent`'s
  additive rule, priced by a real reader) — never a second trail"
- **Companion:** ADR-0041 (the keymap seam whose consult this record
  rides), ADR-0033 (the outcome-as-data pattern this consult's answer
  follows — the M27 rule at the keymap seam), ADR-0034/ADR-0037/
  ADR-0039 (the focus, edit, and announce trails whose shape the
  shortcut trail extends), ADR-0036 (the rider field whose additive
  wire rule this event's fields reuse), ADR-0010 (the protocol's
  additive versioning), architecture §10.3 (the keyboard contract),
  §13.3 (the causal log), §18 (the M36 roadmap row)

## Decision

- **The consult answers as data.** The vocabulary lives in
  `alacer-core/src/keymap.rs` — [`KeymapConsult`] (`Uninstalled`,
  `FellThrough`, `Fired`; `fired()` the verdict accessor, `name()` the
  wire's `snake_case` tag) and [`consult`], the funnel's read as data:
  it runs the installed seam's dispatch exactly as
  `dispatch_installed` always did and answers the species the read
  resolved to. `dispatch_installed` stays the fired verdict — the M35
  contract unchanged, the m35 suite's no-seam pins untouched — and the
  consult is its full answer. The seam handle leaves the thread-local
  before `dispatch` runs (the handle is an `Rc`), so a fired action
  that re-installs mutates its own thread's table without re-entering
  the borrow — the read's re-entrancy shape, made explicit by the
  species' own test.
- **One record per funnel consult, streamed where it resolves.**
  `alacer-core/src/inspect.rs` grows [`KeyRecord`] (the chord beside
  the outcome) and `InspectEvent::Key`; `PassPipeline::dispatch_key`
  streams one record the moment its consult resolves — before the
  fallbacks' focus records, so a key event's trail reads keymap →
  focus → input → frame. Only a consult streams: the focused widget's
  answer pre-empts the keymap entirely (the M35 precedence), so a
  pre-empted key carries its input record and no key record — the
  absent record *is* the precedence, visible in the trail. The record
  carries the chord (`Chord::with_modifiers` — the event's exact key
  and modifier shape) because one binding per chord (the M35
  invariant) makes the chord the fired binding's identity: the record
  answers "which shortcut" without a second naming channel, and the
  fired action's own `alacer::keymap` tracing event keeps carrying the
  table's prose name (the M35 rule, unchanged).
- **The chord rides the record by value.** `Key` and `Chord` gain
  `Copy` — every variant carries its data by value (a `char` and unit
  variants; two small fields) — so the record is `Copy` like
  [`FocusRecord`] before it and the causal log copies the chord the
  way the consult already carried it. No signature changes: the
  derives are additive and the compare semantics are untouched.
- **The observation surface grows additively.** The recorder grows the
  bounded key ring beside its rings (`CausalRecorder::keys()`, 512 —
  the §14.3 eviction discipline), the wire grows the `key` event
  additively at version 1 (the rider field's rule: the logical key's
  tag, the character beside it, the modifier shape in canonical order,
  the species), the embedded client renders the echo beside the focus
  and edit trails, and `docs/inspector-protocol.md` documents the
  event.
- **The corpus stands at 63.** The record rides the published keymap
  route's consult — no hot loop the corpus does not already price (the
  coordinated rule's no-growth arm, the M31/M33 rule): the consult is
  the `dispatch_installed` read the `keymap/dispatch_100_mixed_1k`
  route already measures, and the record streams only when an observer
  is installed (one branch, the A7 idle contract). The m36 suite pins
  the no-observer contract directly.
- **The living documents re-agree at M36 and 0.17.0.** The README's
  status block, the ADR count (forty-three), and the example table (29
  rows); the catalog; the register; the roadmap; the §13.3 questions
  table; the m-suite pins moving M35 → M36 and 42 → 43 across the
  chain; the protocol document's `key` section; the CHANGELOG.

## Consequences

- The consult costs what the funnel's read always cost: the same
  linear scan over the table's owned bindings, the same `false` on a
  miss — and one branch for the record when no observer is installed.
  The unobserved window is today's behavior byte-for-byte (the m36
  suite pins the idle contract directly).
- A trail reader answers "did a shortcut take this key — or why not"
  from the log alone: the fired chord's identity, the table's
  fall-through, and the uninstalled window are three named species,
  never one collapsed `false`.
- No new dependency edges: the consult and the record live in core,
  the funnel streams them, and the recorder, wire, client, example,
  and suite ride the crates they already ride.
- The m-suite's behavioral tests keep passing unchanged — the pins
  move (M35 → M36, 42 → 43) and the behavior does not.

## Kill criteria (re-open rules)

- A consumer needs the fired binding's *prose name* in the record
  (a settings UI reading the log, not the table) → grow the seam
  beside `dispatch` with a provided method the provided table answers
  and custom implementations default — priced by that reader — never
  a second consult channel.
- A consumer needs a chord the exact `Modifiers` shape cannot name
  (platform-conditional primaries, key-position chords) → grow the
  vocabulary beside `Chord`, priced by a real table — never a
  stringly-typed chord the funnel cannot compare (ADR-0041's rule
  stands).
- A consumer needs the table to change the *tree's* answer (a
  shortcut that moves focus or edits a widget) → ride the doors the
  funnel already names (`focus_move`, the mutate seam) from inside
  the action — never a bypass of the pass contract the action's masks
  already flow through (ADR-0041's rule stands).
- A client needs the rides grouped or filtered by outcome → grow the
  client's rendering beside the event the wire already carries
  (ADR-0010's additive rule), priced by that client.
