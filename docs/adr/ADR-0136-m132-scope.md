# ADR-0136: M132 scope — the search-field tier: the field whose purpose is search

- **Status:** Accepted (2026-11-19)
- **Implements:** the word every search vocabulary speaks as "this
  field takes search criteria" — and the tree never said. Through
  M131 the snapshot said the named pane (ADR-0135), the instant
  on/off control (ADR-0134), the tool strip (ADR-0133), the bounded
  number field (ADR-0132) — and the text field's *purpose* had no
  word: an app wiring a search box mounted a plain
  [`TextInput`](crates/alacer-widgets/src/text_input.rs), and the AT
  read "edit text" where the vocabulary says "search" — the field
  wearing no purpose the platform's find conventions key off
  (AccessKit's `SearchInput` role is its own fact, the word the
  vocabulary spells beside `TextInput`, ARIA's `role="searchbox"`).
  [`Widget::search_field`](crates/alacer-core/src/widget.rs) carries
  the purpose word — the M100 secure tier's own re-role shape (the
  flag the sink's role branch reads, a declared purpose re-rolling
  the field in the same snapshot every other property rides).
- **Companion:** ADR-0104 (the M100 secure tier — the flag-remap
  shape the purpose word rides, the masked contract it yields to),
  ADR-0126 (the M122 multiline tier — the capacity word that claims
  the composite, one role per node), ADR-0106 (the M102 reveal tier —
  the masked contract's own yield rule), ADR-0094 (the M94 pattern —
  the declared-but-unreported gap the word closes), ADR-2 (the
  one-currency discipline the word rides), ADR-9 (the DAG the change
  never touches), architecture §10.3 (the IME contract the purpose
  never munges), §17 (the register), §18 (the M132 roadmap row)

## Decision

- **One fact, the field's own word.**
  [`Widget::search_field`](crates/alacer-core/src/widget.rs) carries
  the "takes search criteria" fact — the default the honest absence
  (`false`, never a guessed purpose), the editable controls the
  overriders, every other role inheriting the default. The purpose is
  a *tree* word, never an IME flavor: the IME contract
  ([`ImePurpose`](crates/alacer-core/src/passes.rs)) carries the
  platform's composition rules, and search rides it as the ordinary
  text entry it is — no new flavor, no munged purpose.
- **One role per node; the words keep their precedence.** The sink's
  role branch reads the flag in the same snapshot every other property
  rides: the masked contract first (a hidden secured field keeps the
  `PasswordInput` word M100 mapped, the reveal yielding it per M102 —
  a search ask never unmasks the secret), the capacity word claiming a
  field that declared both (a multiline search box keeps the
  `MultilineTextInput` word M122 mapped — AccessKit's vocabulary has
  no composite, the tree never guesses one), and the purpose word
  re-rolling the single-line, non-secured field. **Pre-registered
  re-open rule:** a consumer needing a composite the two words cannot
  name grows the vocabulary beside the branch, priced by a real field
  — never a second role word.
- **The word prices nothing but the word.** A declared field's doors
  answer exactly as they always did — the edit, the selection, the
  clipboard, the journal, the IME contract unchanged. The word's one
  write is the programmatic
  [`TextInput::set_search_field`](crates/alacer-widgets/src/text_input.rs)
  (view reconciliation): report whether the flag moved — the caller
  requests `A11Y` only when this answers `true`, so a write that lands
  where the word already is queues nothing (the M21 idle-frame
  property, at the purpose tier). The `search` prop joins the
  text-input element (create beside the M100 `secure` word's, patch
  reporting the move).
- **The living documents re-agree at M132 and 0.17.0.** The ADR count
  (a hundred and thirty-seven), the architecture register (ADR-0136
  scopes M132), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `search_field` row) all name
  the tier; the m-suite corpus pins re-agree at 144 — the tier grew no
  snapshot route (the word rides the sink's role branch, a flag beside
  the words the field already carries), and its pricing door is the
  widgets crate's own bench, outside the published corpus by
  ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The word's cost is priced in
  [`crates/alacer-widgets/benches/search_field.rs`](crates/alacer-widgets/benches/search_field.rs):
  `search_field/purpose_write_256` — the moving declaration (the
  `bool`'s write the purpose word takes) against
  `search_field/idle_write_256`, the write that lands where the word
  already is (the honest floor of the same write at the widget seam).
  The pair rides outside the published baseline: the coordinated rule
  grows the corpus at the next published cut, never at a widget
  bench's. The m-suite pins re-agree at 144, the m132 suite's pin
  documenting the same shape.
- **The AT names the search box the toolkit always owed it.** A
  screen reader walking a mounted field announces "search, edit text"
  — the purpose beside the name, the find conventions keying off the
  word. The field walls nothing: the edit, the selection, the
  availability, and the IME doors answer exactly as they always did —
  the word names the purpose, it walls nothing.
- **Consumers get the whole vocabulary for free.** The
  `examples/search_field.rs` beat is the shape: the declared field
  beside the plain one (the honest absence riding beside it), the
  submits reading back the way the app wired them — the neighbours
  untouched.
