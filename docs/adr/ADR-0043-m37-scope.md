# ADR-0043: M37 scope — the binding names itself in the record: the consult's prose name rides

- **Status:** Accepted (2026-11-12)
- **Implements:** the re-open rule ADR-0042 pre-registered on its own
  kill-criteria list — the M36 record carried the chord beside the
  species, and the chord is the fired binding's identity (one binding
  per chord, ADR-0041), but a trail reader that wants the shortcut's
  *prose* name (a settings UI reading the log, an inspector row a
  human labels) had to keep its own chord → name table beside the log
  — cashed as the name leg ADR-0042 pre-registered: "a consumer needs
  the fired binding's prose name in the record → grow the seam beside
  `dispatch` with a provided method the provided table answers and
  custom implementations default — priced by that reader — never a
  second consult channel"
- **Companion:** ADR-0042 (the consult record whose shape this name
  rides), ADR-0041 (the keymap seam whose bindings carry the name),
  ADR-0036 (the rider field whose additive wire rule this field
  reuses), ADR-0033 (the outcome-as-data pattern the record extends),
  ADR-0010 (the protocol's additive versioning), architecture §10.3
  (the keyboard contract), §13.3 (the causal log), §18 (the M37
  roadmap row)

## Decision

- **The seam grows a provided name leg beside `dispatch`.**
  `alacer-core/src/keymap.rs`: [`KeymapSeam::fired_name`] is a
  *provided* method — the provided [`Keymap`] table answers the
  binding its `dispatch` would run (the same match, the same chord
  scan), an implementation whose shortcuts come from somewhere else
  (a settings file re-read on change, a menu bar's accelerator table)
  names its own answer, and one that overrides nothing keeps the
  default `None` — the M35/M36 custom seams keep compiling and
  streaming, unnamed. The answer is the record field's own
  `Option<String>`: an owned name hands the record its field
  directly, and a wrapper seam delegates through its guard, where a
  borrowed `&str` could not escape the `RefCell` that guards its
  table.
- **One consult channel still.** [`consult_name`] is a pure read
  beside [`consult`]: it answers the installed seam's `fired_name`
  and runs no action — the consult's dispatch stays the only door.
  The dispatch funnel asks the name leg *before* the consult, so the
  record names the binding the consult resolves to, never what a
  mid-flight rebind leaves behind — the SelfRenaming seam's own exit
  test pins the ordering (a name read *after* dispatch would answer
  the replacement). The M36 contract is byte-for-byte: `consult`
  answers the three species unchanged, `dispatch_installed` stays the
  fired verdict, and the m36 suite's pins are untouched.
- **One branch for the idle contract.** The capture runs only when an
  observer is installed: the unobserved funnel is byte-for-byte (the
  A7 idle contract, the M31/M33 rule), and the published
  `keymap/dispatch_100_mixed_1k` route measures the unobserved read —
  the name leg never runs on its path.
- **The record grows the name; the wire follows additively.**
  `alacer-core/src/inspect.rs`: [`KeyRecord`] grows `name` beside
  chord and outcome — `Some` only on a fired consult (a miss and an
  uninstalled window name no binding; the funnel clamps the capture),
  `None` when the seam that fired names none. The record trades its
  M36 `Copy` for `Clone` (the name is a heap `String`; the chord
  keeps the by-value ride), and every read site moves from copy to
  clone mechanically. `alacer-inspect`'s wire grows the `key` event's
  `name` field additively at version 1 (the rider field's rule), the
  embedded client renders the name beside the outcome when present,
  and `docs/inspector-protocol.md` documents the field.
- **The corpus stands at 63.** The name leg costs the unobserved
  funnel nothing and rides the published keymap route's consult — no
  hot loop the corpus does not already price (the coordinated rule's
  no-growth arm, the M31/M33/M36 rule): the observer-present path
  prices one owned-`String` answer per *streamed* consult, a
  dev-mode surface bounded by the recorder's rings, not a frame-budget
  liability.
- **The living documents re-agree at M37 and 0.17.0.** The README's
  status block, the ADR count (forty-four), the inspector bullet, the
  quickstart, and the example table (30 rows); the catalog; the
  register; the roadmap; the §13.3 questions table; the m-suite pins
  moving M36 → M37 and 43 → 44 across the chain; the protocol
  document's `key` section; the CHANGELOG.

## Consequences

- The consult costs what the funnel's read always cost on the
  unobserved path — the name leg never runs there. With an observer,
  one pure read (the same chord scan `dispatch` runs) precedes the
  consult, and one owned `String` moves into the record per streamed
  consult.
- A trail reader answers "did a shortcut take this key — or why not —
  and *which* shortcut" from the log alone: the fired binding's prose
  name rides beside the chord and the species, and a wrapper seam
  that forgets the name leg names nothing — visible, never guessed.
- No new dependency edges: the name leg lives in core, the funnel
  streams it, and the recorder, wire, client, example, and suite ride
  the crates they already ride.
- The m-suite's behavioral tests keep passing unchanged — the pins
  move (M36 → M37, 43 → 44) and the behavior does not.

## Kill criteria (re-open rules)

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
