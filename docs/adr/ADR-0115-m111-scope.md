# ADR-0115: M111 scope — the auto-complete tier: the field's suggestion contract, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every field vocabulary speaks about the
  suggestions it offers — "this one completes inline, that one lists
  them" — and the tree never said. Through M110 the snapshot said what
  a control *does* (the description, ADR-0099), what an empty field
  *hints* (the placeholder, ADR-0100), the span a value rides (the
  range tier, ADR-0101), the form's ask (the required tier, ADR-0110),
  the form's rejection (the invalid tier, ADR-0111), the collection's
  pick and capacity (ADR-0112/ADR-0113), the member's disclosure (the
  expanded tier, ADR-0114) — and the fact the AT turns into
  "suggestions: inline / list / both" while the user types was the one
  field fact the tree carried nothing for: the AT announced the hint,
  the span, the form facts — and never the suggestion contract. The
  declared-but-unreported gap (the M94 pattern), cashed at the
  auto-complete tier: "a consumer needs the field's suggestion
  contract to read as exactly that, and as an honest absence when the
  field offers none → the tree carries AccessKit's `AutoComplete`
  property (ARIA's `aria-autocomplete`), the doors answering unchanged
  beside the word — the word prices the word and nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0111 (the tier the
  word grows beside — the form's rejection, and now the suggestion
  contract; the kind-mirroring shape the word copies: ARIA's own
  values mirrored so the mapping is one branch with no translation,
  the `InvalidKind` precedent at the enum tier), ADR-0087 (the
  presence rule at the tier: `None` the honest absence — ARIA's
  `none` the absence spelled by `None`, never a guessed mode),
  ADR-9 (the DAG the change never touches), architecture §11 (the
  a11y contract), §17 (the register), §18 (the M111 roadmap row)

## Decision

- **One word, the kind beside it.**
  [`Widget::auto_complete`](crates/alacer-core/src/widget.rs) answers
  `Option<AutoCompleteKind>` — `None` the honest absence (the ADR-0087
  presence rule at the tier: a field that declared nothing announces
  no suggestion contract, never a guessed mode), the kind
  ([`AutoCompleteKind`](crates/alacer-core/src/widget.rs)) riding
  beside the word: `Inline` the field that completes the entry as the
  user types, `List` the field that presents the possible completions,
  `Both` the compound contract — AccessKit's own three enum values
  mirrored, ARIA's fourth (`none`) the absence the `Option` already
  spells. Auto-complete is not a wall: an auto-completed field's doors
  answer exactly as any field's (the word names the suggestion
  contract, it walls nothing — the field stays editable, the AT's
  write landing untouched), so the tier grows no refusal vocabulary
  and prices no behavior beyond the word itself.
- **The producer joins without a seam change.** The editable canon's
  field ships the full shape —
  [`TextInput::auto_complete`](crates/alacer-widgets/src/text_input.rs),
  the builder naming the initial contract, beside
  [`TextInput::set_auto_complete`](crates/alacer-widgets/src/text_input.rs),
  the app's live door through the mutate seam (`None` the honest
  absence restored; the write that lands where the word already is
  queues nothing, the M21 idle-frame property). The word is a field
  word: the value tier's non-editable controls answer nothing (a
  slider reports its value, not its suggestions), so no producer joins
  there.
- **The snapshot carries it; the sink maps it.**
  `TreeNode::auto_complete` rides the same pass-9 snapshot every other
  property rides, and the sink maps AccessKit's `AutoComplete`
  property — one branch beside the expanded word's (ADR-0114's own
  mapping), no patch-tier growth (the word is runtime state the app's
  door moves, never a declarable DSL word).
- **The living documents re-agree at M111 and 0.17.0.** The ADR count
  (a hundred and sixteen), the architecture register (ADR-0115 scopes
  M111), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `auto_complete` row) all name the tier;
  the corpus grew 127 → 128 by the coordinated rule below, and the
  m-suite corpus pins re-agree at 128.

## Consequences

- **The corpus grew 127 → 128 by the coordinated rule — the tier's own
  honest case:** `auto_complete/snapshot_1k_auto_complete` — 1,000
  text fields, the suggestion contract's snapshot price — against the
  same 1,000-node field arena bare (`placeholder/snapshot_1k_bare`,
  the M96 case; the M95 pricing rule — the word is a field word, so
  the honest comparison is fields against fields, the M106/M107
  denominator). The `auto_complete_snapshot_overhead` ratio budget
  ≤ 1.25× (the word rides the snapshot the field rides, the tier's
  price the one `Option` the node carries beside it). The m-suite pins
  re-agree at 128.
- **The AT reads the suggestion contract while the user types.** The
  word is the mapping the platform ATs turn into the field's
  "completions available" — beside the hint the placeholder tier
  already carried, the fact a screen reader answers "editable,
  autocompletion: list" with no extra route, no extra pass. The
  absence is the contract's other half: a field that declared nothing
  reads as ARIA's `none`, never as a guessed mode, so a plain field
  does not start announcing suggestions it does not offer.
- **Consumers get the whole vocabulary for free.** The
  `examples/auto_complete.rs` beat is the shape: the field that lists
  its completions beside the field that completes inline and the
  plain field between them, the AT's write landing on the
  auto-completed field untouched (the word names a contract, it walls
  nothing), the live door moving the word — list, inline, clear — the
  honest absence restored.
