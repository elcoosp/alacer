# ADR-0126: M122 scope — the multiline tier: the field's capacity, said by the tree

- **Status:** Accepted (2026-11-19)
- **Implements:** the word every form-and-composer vocabulary speaks
  about *how much the field takes* — "this field wraps onto many
  lines" — and the tree never carried for the field the AT walks to.
  Through M121 the snapshot said what an empty field *hints* (the
  placeholder tier, ADR-0100), the form's ask and the form's rejection
  (ADR-0110/ADR-0111), the field's suggestion contract (ADR-0115), the
  secret entry and its reveal (ADR-0104/ADR-0106), the wall the
  read-only field holds (ADR-0109) — and the capacity the composer's
  notes field, the feedback box, the log viewer take was a fact the
  field's own constructor knew and the tree did not: the field took
  every line the user typed, and the AT announced the hint, the ask,
  the rejection — and never the capacity — the declared-but-unreported
  gap (the M94 pattern), cashed at the multiline tier: "a consumer
  needs the AT to announce the capacity beside the hint → the tree
  carries the field's declared capacity word, the door the app moves,
  the doors answering unchanged beside the word — the word prices the
  word and nothing else"
- **Companion:** ADR-0104/ADR-0106 (the secure tier's own role branch —
  the mapping the word joins: AccessKit carries the field's capacity as
  the `MultilineTextInput` role, there being no separate property to
  mirror, so the tier resolves the role the way the secure tier
  resolves `PasswordInput` — and the secure field's branch wins the
  branch it has carried since M100, the capacity word riding the
  snapshot beside the role it never displaces), ADR-0113 (the flag
  precedent at the tier: one flag, `false` the honest absence, no
  trio-ambiguity for the presence rule to protect — a field that
  declared no capacity announces none, never a guessed "multiline"),
  ADR-2 (the one-currency discipline the word joins — the word rides
  the `A11Y` mask the snapshot pass already pays), ADR-9 (the DAG the
  change never touches), architecture §11 (the a11y contract), §17 (the
  register), §18 (the M122 roadmap row)

## Decision

- **One word, `false` the honest absence.**
  [`Widget::multiline`](crates/alacer-core/src/widget.rs) answers
  `bool` — the field's line capacity, the word the AT reads beside the
  hint ("this field wraps onto many lines"), `false` the honest absence
  (the ADR-0113 flag shape at the tier: a field that declared no
  capacity announces none — the single-line field the AT read before
  the tier, never a guessed "multiline"). The word prices the word and
  nothing else: a worded field's doors answer exactly as they always
  did — the word names the capacity, it walls nothing, the field's
  typing, caret, selection and submit routes the widget's own (a tree
  that refused on the word's behalf would price a contract it does not
  own — the word says the field wraps, it never changes what the field
  takes).
- **The producer is the field vocabulary — `TextInput`, the editable
  word's own shape.**
  [`TextInput::multiline`](crates/alacer-widgets/src/text_input.rs) —
  the builder naming the capacity from the first frame (the composer's
  notes field, the feedback box) — beside
  [`TextInput::set_multiline`](crates/alacer-widgets/src/text_input.rs),
  the app's live door through the mutate seam (`false` restoring the
  honest absence — the capacity withdrawn; the write that lands where
  the word already is queues nothing, the M21 idle-frame property). The
  word is a field word: the plain widgets answer nothing (a button
  reports its press, a list its scroll — the capacity is the field's
  own), so no other producer joins at the cut; other editable
  producers join without a seam change (the M107 rule) the day the
  vocabulary needs them.
- **The snapshot carries it; the sink resolves the role.**
  `TreeNode::multiline` rides the pass-9 fill beside the words already
  there (`slot.widget().multiline()` the same walk every other property
  rides), and the sink's mapping answers with AccessKit's
  `MultilineTextInput` role — the capacity word AccessKit's own
  vocabulary carries (ARIA's `aria-multiline` beside the `textbox`
  role), resolved in the role branch the secure tier opened (M100). No
  patch-tier growth (the word is runtime state the app's door moves,
  never a declarable DSL word), no second channel: a capacity write
  re-announces exactly that node. The secure field's branch wins the
  branch it has carried since M100 — a secured field reads as
  `PasswordInput`, the capacity word riding the snapshot beside the
  role it never displaces; the honest absence resolves nothing — a
  field that declared no capacity reads as the ordinary `TextInput`
  role it always was.
- **The living documents re-agree at M122 and 0.17.0.** The ADR count
  (a hundred and twenty-seven), the architecture register (ADR-0126
  scopes M122), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `multiline` row) all name the
  tier; the corpus grew 141 → 142 by the coordinated rule below, and
  the m-suite corpus pins re-agree at 142.

## Consequences

- **The corpus grew 141 → 142 by the coordinated rule — the M121
  precedent's own shape:** `multiline/snapshot_1k_fields_multiline` —
  1,000 fields, the worded arena's snapshot price (the constructor's
  capacity word, the way a composer mounts its notes field) — against
  the placeholder route's published bare arena
  (`placeholder/snapshot_1k_bare`, the M96 case the tier reuses by the
  M111 precedent; the M95 pricing rule — the word is a field word, so
  the honest comparison is fields against fields, the M108/M121
  denominator shape). The `multiline_snapshot_overhead` ratio budget
  ≤ 1.25× (the word rides the snapshot the field rides, the tier's
  price the one `bool` the node carries beside it), measured
  0.915× at the cut. The m-suite pins re-agree at 142.
- **The AT reads the capacity before it types.** The word is the
  mapping platform ATs turn into "edit text, multiline" beside the
  name — beside the hint the placeholder tier already said, a screen
  reader walking a form reads the capacity before it routes the
  keyboard, and a field the app never worded says nothing. The absence
  is the contract's other half: a field that claims no capacity
  announces none, never a guessed "multiline", so a single-line field
  does not start carrying a capacity the app did not declare.
- **Consumers get the whole vocabulary for free.** The
  `examples/multiline.rs` beat is the shape: the worded notes field
  under the word, the plain title field and the plain label the honest
  absence between them, every door answering as it always did — the
  AT's write door landing where it always landed, the typing door
  taking what it always took — the multiline door moving the word —
  the plain summary field declaring its own capacity, the notes
  capacity withdrawn, clear — the honest absence restored.
