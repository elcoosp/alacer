# ADR-0044: M38 scope — the primary chord: the platform's primary modifier as chord vocabulary

- **Status:** Accepted (2026-11-12)
- **Implements:** the re-open rule ADR-0043 pre-registered on its own
  kill-criteria list (ADR-0042's before it) — a chord the exact
  `Modifiers` shape cannot name: an app's most common intent is "the
  platform's primary accelerator" (`Ctrl+S` on Linux/Windows, `Cmd+S`
  on macOS), but a chord's shape is *physical*, so the app had to fork
  its table per platform — or reach for a stringly-typed chord the
  funnel cannot compare, the rule ADR-0041 refused — cashed as the
  vocabulary growth beside `Chord` the rule pre-registered: "grow the
  vocabulary beside `Chord`, priced by a real table — never a
  stringly-typed chord the funnel cannot compare"
- **Companion:** ADR-0041 (the keymap seam whose table this chord
  rides), ADR-0043 (the consult record whose chord prints the
  resolution), ADR-0010 (the protocol's additive versioning —
  untouched here: the wire carries the resolved physical shape the
  record already carried), architecture §10.3 (the keyboard
  contract), §18 (the M38 roadmap row)

## Decision

- **One builder, resolved at build time.**
  `alacer-core/src/keymap.rs`: [`Chord::primary`] records the physical
  flag the platform's primary accelerator modifier carries on this
  build — Command on Apple platforms, Control elsewhere — the same
  resolution [`Modifiers::primary`] reads (the input's command-chord
  convention). No new field, no logical flag, no match-time platform
  context: the chord's shape *is* the event's physical shape, so the
  exact-shape match (the M35 invariant) is untouched and the funnel's
  compare is the compare it always was.
- **The resolution is the chord's identity.** One binding per chord
  (the M35 invariant) resolves with the chord: off Apple platforms a
  `primary()` chord *is* a `control()` chord — binding the physical
  shape over the primary one moves in place, never shadows — and on
  Apple platforms the two are different chords, free for different
  actions. The aliasing never widens: a `Cmd+S` event off Apple
  platforms is a different shape, the consult falls through honestly,
  and the record carries the chord the funnel *asked* about (the meta
  shape), unnamed.
- **The drift is fixed, not the flag.** `Chord::control`'s doc claimed
  "Control (Linux/Windows) or Command (macOS)" — a claim the physical
  shape never kept (an Apple Command event carries `meta`), and its
  second paragraph named a `primary()` builder that did not exist.
  The doc now pins the physical `control` flag and points at
  [`Chord::primary`] for the platform's primary; the builder's own
  rustdoc carries the resolution rules and the identity consequences.
- **The wire is untouched.** The `key` event's `modifiers` field
  prints the record's chord — the resolved physical shape — which is
  the truth the wire always carried; `docs/inspector-protocol.md`
  says so in the `modifiers` bullet (the resolved physical shape a
  primary chord resolves to). Version 1, additive nothing.
- **The corpus stands at 63.** The resolution happens at bind time —
  a `cfg!` branch, no match-time cost, no new hot loop (the
  coordinated rule's no-growth arm, the M31/M33/M36/M37 rule): the
  published `keymap/dispatch_100_mixed_1k` route measures the same
  scan it always measured.
- **The living documents re-agree at M38 and 0.17.0.** The README's
  status block, the ADR count (forty-five), the quickstart, and the
  example table (31 rows); the catalog; the register; the roadmap;
  the m-suite pins moving M37 → M38 and 44 → 45 across the chain;
  the protocol document's `modifiers` sentence; the CHANGELOG.

## Consequences

- An app writes one table — `Chord::new(Key::Char('s')).primary()` —
  and it answers the platform's own accelerator everywhere the
  workspace compiles; the per-platform fork (or the stringly-typed
  chord) is gone.
- The chord's `Debug` shape and the causal record print the resolved
  physical flags: a trail reader on macOS sees `meta`, on Linux
  `control` — the physical truth the funnel matched, never a logical
  label the wire would have to interpret.
- No new dependency edges, no wire change, no corpus growth: the
  builder lives beside the other flag builders, the record already
  carried the chord, and the bench route measures the same scan.
- The m-suite's behavioral tests keep passing unchanged — the pins
  move (M37 → M38, 44 → 45) and the behavior does not.

## Kill criteria (re-open rules)

- A consumer needs a chord the resolved primary *still* cannot name
  (key-position chords, per-layout chords) → grow the vocabulary
  beside `Chord`, priced by a real table — never a stringly-typed
  chord the funnel cannot compare (ADR-0041's rule stands).
- A consumer needs the table to change the *tree's* answer (a
  shortcut that moves focus or edits a widget) → ride the doors the
  funnel already names (`focus_move`, the mutate seam) from inside
  the action — never a bypass of the pass contract the action's masks
  already flow through (ADR-0041's rule stands).
- A client needs the rides grouped or filtered by outcome → grow the
  client's rendering beside the event the wire already carries
  (ADR-0010's additive rule), priced by that client.
