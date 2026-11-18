# ADR-0109: M105 scope — the read-only tier: the field whose value is the app's, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the declared-but-unreported word the write walls
  priced before the tree could say it. Since M22 (ADR-0028) the
  availability tier named the control that refuses *every* door
  including focus; since M55 (ADR-0061) the AT's write door streamed
  its refusals; since M62 (ADR-0068) the typing door priced its walls
  as named species — and the field every word-processing app ships
  (the license key, the computed total, the log the app owns) had no
  word: it reads, selects, copies and walks its caret like any other
  field while refusing only the writes, and the tree said nothing. The
  AT announced an editable it could not edit — the M94 pattern's gap
  ("the declared-but-unreported fact"), cashed at the read-only tier:
  "a consumer needs the field whose value the app owns to read as
  exactly that → the tree carries AccessKit's `ReadOnly` flag
  (ARIA's `aria-readonly`), the write doors refuse with the named
  species the trail already speaks, the read doors answer beside the
  wall — never a munged edit, never a guessed word"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the flag rides the `A11Y` mask the snapshot pass already pays),
  ADR-0028 (the availability tier the read-only wall sits inside —
  the outer wall, never a second availability), ADR-0061/ADR-0068
  (the refusal vocabularies the tier grows by one species each),
  ADR-0087 (the presence rule at the tier: `false` the honest
  absence), ADR-9 (the DAG the change never touches: the word is
  core vocabulary, the flag a widgets-level flag, the mapping the
  sink's own), architecture §10.2 (the editing tier), §11 (the a11y
  contract), §18 (the M105 roadmap row)

## Decision

- **One word, one flag.** [`Widget::read_only`]
  (crates/alacer-core/src/widget.rs) answers `bool` — `false` the
  honest absence, the ADR-0087 presence rule at the tier. The word is
  a *declaration about doors*, not a style: it says which walls the
  doors already price, it does not mute the field, and it is
  constructor-immutable no more than the reveal word is — the app's
  live door ([`TextInput::set_read_only`],
  crates/alacer-widgets/src/text_input.rs) moves it, the caller's
  `PAINT | A11Y` request paying the re-announcement (the mutate seam's
  shape, the M102 precedent).
- **The snapshot carries it; the sink maps it.**
  `TreeNode::read_only` rides the same pass-9 snapshot every other
  property rides, and the sink maps AccessKit's `ReadOnly` flag —
  one branch beside the disabled flag's (ADR-0028's own mapping), no
  patch-tier growth (the word is runtime state the app's door moves,
  never a declarable DSL word).
- **The walls are the doors' own vocabulary.** The typing door's
  classifier already names the edit asks (M62's `refused_action`);
  a read-only field routes them to the new
  [`KeyEditRefusal::ReadOnly`] species — typing, deletes, the word
  chords, paste, cut — while the classifier names nothing for the
  caret's walk, the selection's chords and the copy chord, which
  pass beside the wall. The IME's commit streams the same species
  beside the IME driver (the availability wall's shape, the inner
  tier's species). The AT's write door (`SetValue`,
  `ReplaceSelectedText`) refuses before arithmetic with
  [`TextWriteRefusal::ReadOnly`]; the selection door
  (`SetTextSelection`) answers — a read-only field reads. The
  journal's chords (undo, redo) are value writes the pre-registered
  wire vocabulary did not name: [`KeyEditAction::Undo`] and
  [`KeyEditAction::Redo`] join the action set (`undo` / `redo`, the
  additive wire rule ADR-0010 names), so the wall prices them like
  every other ask. The submit's Enter is a value write too (fire,
  then clear) and asks nothing by the M62 rule — a read-only field
  consumes it and moves nothing.
- **The availability tier stays the outer one.** A field both
  disabled and read-only never reaches the read-only wall: the
  unavailable control holds no focus (M22's own derivation), the key
  route's boundary answers first (the M59/M62 shape), and the
  read-only word rides the tree beside the availability's. One
  concept per tier, the species naming the tier that answered.
- **The app's door never asked.** `set_value` is the app's write —
  the tier walls *user* doors, not the app's; a locked field's value
  is exactly what the app last wrote, the rule the disabled field's
  value already rode.
- **The living documents re-agree at M105 and 0.17.0.** The ADR
  count (a hundred and ten), the architecture register (ADR-0109
  scopes M105), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `read_only` row) all name the
  tier; the corpus grew 118 → 119 by the coordinated rule below, and
  the published baseline re-agrees with its own pins (the M103/M104
  cases the baseline file had trailed — see Consequences).

## Consequences

- **The corpus grew 118 → 119 by the coordinated rule:**
  `read_only/snapshot_1k_read_only` — 1,000 read-only fields, the
  word's snapshot price — against the same 1,000-node input arena
  bare (`placeholder/snapshot_1k_bare`, the M96 case), the
  `read_only_snapshot_overhead` ratio budget ≤ 1.25× (the bool rides
  the snapshot the field rides). The m-suite pins re-agree at 119.
- **The published baseline re-agrees with its own pins.** The
  baseline file had trailed the corpus: `bench_count` published 114
  while the contracts gated 118 — the M103 clock cases
  (`click_chain/seamed_chain_1k`, `click_chain/manual_chain_1k`) and
  the M104 timing cases (`pass_overhead/timed_idle_10k`,
  `pass_overhead/seamed_timed_idle_10k`) were gated but never
  published, so every corpus-pin suite at 118 failed against the
  file it read. The four cases join the baseline beside the tier's
  own, measured medians from this host's fresh criterion data — the
  uncontracted-benchmark rule's mirror (no *ungated* case, and no
  *unpublished* gate either).
- **The AT stops announcing an editable that cannot edit.** The
  word is the mapping the platform ATs read; the walls stream their
  named species on the trails the devtools already speak.
- **Consumers get the whole vocabulary for free.** The
  `examples/read_only.rs` beat is the shape: a locked license field
  beside an editable draft, the status line mirroring the words the
  tree says, the unlock door moving the word live.
