# ADR-0041: M35 scope — the keymap seam: the app's shortcuts ride the dispatch contract

- **Status:** Accepted (2026-11-12)
- **Implements:** the keyboard's window-level half, which until now no
  consumer could name without smuggling it through widget handlers —
  an application's actions (save, find, export) must answer wherever
  the focus sits, and the funnel the M24/M29 milestones built carried
  only the framework's own fallbacks (Tab cycling, the directional
  ring), so a chord like `Ctrl+S` routed to the focused input, found
  no widget vocabulary, and died — cashed as a real seam: "an app
  needs window-level actions the focused widget does not own → one
  installed [`KeymapSeam`] the dispatch funnel consults when the
  focused widget did not answer, before the framework's fallbacks —
  never per-widget handler plumbing, never a second dispatch channel"
- **Companion:** ADR-0020 (the clipboard seam whose thread-local
  install discipline this seam rides), ADR-0024 (the announcement
  drain whose optional-seam precedent this seam's total read follows),
  ADR-0030 (the focus walk the Tab fallback rides), ADR-0035 (the
  directional fallback the keymap's precedence stands above),
  ADR-9 (the DAG the change never touches), architecture §10.3 (the
  keyboard contract), §18 (the M35 roadmap row)

## Decision

- **One seam, installed at assembly, consulted by the funnel.** The
  vocabulary lives in `alacer-core/src/keymap.rs` — [`Chord`] (one
  logical `Key` plus the exact `Modifiers` shape it answers, built with
  `new`/`with_modifiers` and the `control`/`shift`/`alt`/`meta`
  builders), [`KeymapSeam`] (the trait whose `dispatch` runs the bound
  action and answers whether the key is the keymap's now), [`Keymap`]
  (the provided table: one binding per chord, `bind` replacing in
  place so the settings story is a move and never a shadow, `unbind`
  returning the chord to the framework's fallbacks, `names` the
  introspection a settings read-back walks), and the thread-local
  install trio (`install`, `is_installed`, `dispatch_installed`) — the
  clipboard seam's discipline (ADR-0020), because handlers cannot
  reach the pipeline (axiom A1) and a widget-internal shortcut table
  would fork the dispatch contract the same way a widget-internal
  clipboard would fork the editing one.
- **The precedence is the funnel's own order, made three-tier.** The
  focused widget's vocabulary answers first (a focused input's Space
  must type — the M29 rule the m35 suite pins at the keymap tier too),
  the app's shortcuts answer second (the explicit beats the implicit:
  an app that binds `Tab` in a modal means it, and a bound plain `Tab`
  holds focus while the unbound `Shift+Tab` — a different chord —
  still walks backward), and the framework's fallbacks stand third
  exactly where the app stayed silent (Tab cycling, M24; the
  directional ring, M29). The consult needs no focused widget at all:
  window-level vocabulary answers wherever the focus sits, and a
  shortcut is focus-neutral — the fired action queues no masks of its
  own, the M21 idle-frame property at the app seam.
- **The press only, and the fired action names itself.** A chord
  matches pressed events (`Chord::matches`): a release is never a
  second firing and a held key's auto-repeat is one run per physical
  press (the GTK activation rule the buttons carry, at the window
  tier). The fired binding traces its name on the `alacer::keymap`
  target (`tracing::debug!`, the house full-path macro idiom), so a
  `RUST_LOG` reader sees *which* shortcut answered, not merely that
  one did.
- **The seam is optional, so the read is total.** A window with no
  shortcuts is a legitimate application, not a wiring bug:
  `dispatch_installed` answers `false` when nothing is installed and
  the funnel falls through honestly (the M11 rule at the dispatch
  seam) — no panic door, the clipboard's `with_clipboard` rule
  inverted for an optional seam, the A7 idle contract unchanged.
- **The corpus grows by the coordinated rule.** The keymap's mixed
  route joins the published corpus (`keymap/dispatch_100_mixed_1k`:
  one thousand events alternating a miss and a mid-table hit against
  one hundred bindings, through the funnel's own
  `dispatch_installed` read), gated in `benchmarks/contracts.json`
  and measured in `benchmarks/baselines/baselines.json` in the same
  cut (bench, gate, and baseline together — the corpus pins moving
  62 → 63 across the m5–m30 chain; the entry measured on the
  handoff host, the baseline header still naming the last full
  regeneration).
- **The living documents re-agree at M35 and 0.17.0.** The README's
  status block, the ADR count (forty-two), and the example table (28
  rows); the catalog; the register; the roadmap; the m-suite pins
  moving M34 → M35 and 41 → 42 across the chain; the CHANGELOG.

## Consequences

- The dispatch funnel's keyboard order is fixed and pinned: focused
  widget → keymap → Tab → arrows. A chord the keymap consumes costs
  one linear scan over the table's owned bindings (the published
  route prices it); an unbound chord costs the same scan and the
  same `false` the widget answer's absence already carried.
- No new dependency edges: the seam lives in core, the funnel
  consults it, the example and the suite ride the facade and the
  harness they already ride.
- The m-suite's behavioral tests keep passing unchanged — the
  uninstalled window is today's behavior byte-for-byte (the m35
  suite's no-seam test pins the idle contract directly).

## Kill criteria (re-open rules)

- A consumer needs the shortcut's firings in the causal log (an
  inspector pane that replays which chord answered) → grow the
  record beside the input vocabulary the funnel already streams —
  `InspectEvent`'s additive rule, priced by a real reader — never a
  second trail.
- A consumer needs a chord the exact `Modifiers` shape cannot name
  (platform-conditional primaries, key-position chords) → grow the
  vocabulary beside `Chord`, priced by a real table — never a
  stringly-typed chord the funnel cannot compare.
- A consumer needs the table to change the *tree's* answer (a
  shortcut that moves focus or edits a widget) → ride the doors the
  funnel already names (`focus_move`, the mutate seam) from inside
  the action — never a bypass of the pass contract the action's
  masks already flow through.
