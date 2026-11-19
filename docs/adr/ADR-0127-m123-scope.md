# ADR-0127: M123 scope — the language tier: the text's language, said by the tree

- **Status:** Accepted (2026-11-19)
- **Implements:** the word every multilingual vocabulary speaks about
  *what language the text speaks* — "this label is French, that one
  Japanese" — and the tree never carried for the text the AT walks to.
  Through M122 the snapshot said what a control *does* (ADR-0099), what
  an empty field *hints* (ADR-0100), the field's line capacity
  (ADR-0126), the form facts, the collection facts, the declared live
  region, the chord the control is invoked with — and the language the
  label's text, the button's name, the field's value speak was a fact
  the app knew and the tree did not: the AT announced the text in
  whatever voice the platform guessed, and a declared tag had no tier —
  the declared-but-unreported gap (the M94 pattern), cashed at the
  language tier: "a consumer needs the AT to voice the text with the
  tag the app declared → the tree carries AccessKit's `Language`
  property (the BCP 47 tag), the string the app declared, the doors
  answering unchanged beside the word — the word prices the word and
  nothing else"
- **Companion:** ADR-0099 (the string-tier precedent the word rides —
  the property is a string, no kind enum to mirror, one branch with no
  translation), ADR-0087 (the presence rule at the tier: `None` the
  honest absence — a widget that declared no language announces none,
  never a guessed tag), ADR-2 (the one-currency discipline the word
  joins — the word rides the `A11Y` mask the snapshot pass already
  pays), ADR-9 (the DAG the change never touches), architecture §11
  (the a11y contract), §17 (the register), §18 (the M123 roadmap row)

## Decision

- **One word, `None` the honest absence.**
  [`Widget::language`](crates/alacer-core/src/widget.rs) answers
  `Option<&str>` — the language the widget's text is written in, the
  BCP 47 tag the platform AT voices the text with ("fr-FR", "ja-JP";
  the property is a string and the tier translates nothing), `None` the
  honest absence (the ADR-0087 presence rule at the tier: a widget that
  declared no language announces none — the text the AT voiced before
  the tier, never a guessed tag). The word prices the word and nothing
  else: a worded widget's doors answer exactly as they always did — the
  word names the language, it walls nothing, the text's read routes the
  widget's own (a tree that refused on the word's behalf would price a
  contract it does not own — the word says the text's language, it
  never translates the text).
- **The producer is the text vocabulary — `Label`, the static text's
  own shape.**
  [`Label::language`](crates/alacer-widgets/src/label.rs) — the builder
  naming the tag from the first frame (the localized greeting, the
  foreign-language quote) — beside
  [`Label::set_language`](crates/alacer-widgets/src/label.rs), the
  app's live door through the mutate seam (`None` restoring the honest
  absence — the tag withdrawn; the write that lands where the word
  already is queues nothing, the M21 idle-frame property). The word is
  a text word: the plain widgets answer nothing (a button reports its
  press, a list its scroll — the language is the text's own), so no
  other producer joins at the cut; other text producers join without a
  seam change (the M107 rule) the day the vocabulary needs them.
- **The snapshot carries it; the sink maps it.** `TreeNode::language`
  rides the pass-9 fill beside the words already there
  (`slot.widget().language()` the same walk every other property
  rides), and the sink's mapping answers with AccessKit's `Language`
  property — the tag the platform AT voices the text with. No
  patch-tier growth (the word is runtime state the app's door moves,
  never a declarable DSL word), no second channel: the property rides
  the same node-only delta as every other property — a language write
  re-announces exactly that node.
- **The living documents re-agree at M123 and 0.17.0.** The ADR count
  (a hundred and twenty-eight), the architecture register (ADR-0127
  scopes M123), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `language` row) all name the
  tier; the corpus grew 142 → 144 by the coordinated rule below, and
  the m-suite corpus pins re-agree at 144.

## Consequences

- **The corpus grew 142 → 144 by the coordinated rule — the M119 pair
  precedent's own shape:** `language/snapshot_1k_labels_language` —
  1,000 labels, the worded arena's snapshot price (the constructor's
  tag, the way a localized greeting mounts) — against
  `language/snapshot_1k_labels_bare`, the same arena bare (the tier's
  own honest pair, the M119 shape; the M95 pricing rule — the word is a
  label word, so the honest comparison is labels against labels). The
  `language_snapshot_overhead` ratio budget ≤ 1.25× (the word rides the
  snapshot the label rides, the tier's price the one bounded copy the
  node carries beside the name), measured 1.099× at the cut.
  The m-suite pins re-agree at 144.
- **The AT voices the text with the tag the app declared.** The word is
  the mapping platform ATs turn into the right voice before they speak
  — beside the text the tree already said, a screen reader walking a
  localized surface reads "Bienvenue" in the voice the app named, and
  a label the app never worded says nothing. The absence is the
  contract's other half: a label that claims no language announces
  none, never a guessed tag, so a plain label does not start carrying
  a tag the app did not declare.
- **Consumers get the whole vocabulary for free.** The
  `examples/language.rs` beat is the shape: the worded Bienvenue under
  the word, the plain Welcome and the plain button the honest absence
  between them, every door answering as it always did — the text the
  text it always was — the language door moving the word — the plain
  Hello declaring its own tag, the Bienvenue tag withdrawn, clear —
  the honest absence restored.
