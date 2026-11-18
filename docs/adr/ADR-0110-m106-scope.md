# ADR-0110: M106 scope — the required tier: the form contract's word, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every form vocabulary speaks before the
  submit fires — "this one must be filled" — and the tree never said.
  Through M105 the snapshot said what a control *does* (the
  description, ADR-0099), what an empty field *hints* (the
  placeholder, ADR-0100), the span a value rides (the range tier,
  ADR-0101), the number behind the word (the numeric tier, ADR-0103),
  whether the value is the app's (the read-only tier, ADR-0109) — and
  the ask the AT turns into "required" on every platform was the one
  form fact the tree carried nothing for: the AT announced the field,
  the hint, the span, and never the contract. The declared-but-
  unreported gap (the M94 pattern), cashed at the required tier: "a
  consumer needs the form's contract to read as exactly that → the
  tree carries AccessKit's `Required` flag (ARIA's `aria-required`),
  the doors answering unchanged beside the word — the word prices the
  word and nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the flag rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0109 (the
  constructor-plus-live-door shape the tier copies: the word the
  builder names, the door the app moves), ADR-0087 (the presence rule
  at the tier: `false` the honest absence), ADR-9 (the DAG the change
  never touches), architecture §10.2 (the editing tier), §11 (the
  a11y contract), §18 (the M106 roadmap row)

## Decision

- **One word, one flag, doors unchanged.**
  [`Widget::required`](crates/alacer-core/src/widget.rs) answers
  `bool` — `false` the honest absence, the ADR-0087 presence rule at
  the tier. Required is not a wall: a required field's write doors
  answer exactly as any editable's (the empty submit is the app's own
  contract — the AT reads the declaration *before* it asks, which is
  the whole point of the word), so the tier grows no refusal
  vocabulary and prices no behavior beyond the flag itself.
- **The producers join without a seam change.** `TextInput` ships the
  full shape — the [`required()`] builder naming the initial state,
  [`TextInput::set_required`](crates/alacer-widgets/src/text_input.rs)
  the app's live door (the caller's `PAINT | A11Y` request paying the
  re-announcement; the write that lands where the word already is
  queues nothing, the M21 idle-frame property) — and the value tier's
  controls override beside it ([`Slider::required`],
  [`Checkbox::required`], crates/alacer-widgets/src/slider.rs,
  crates/alacer-widgets/src/checkbox.rs): the trait door already
  answers them, the ADR-0104 re-open rule's shape at the producers.
- **The snapshot carries it; the sink maps it.** `TreeNode::required`
  rides the same pass-9 snapshot every other property rides, and the
  sink maps AccessKit's `Required` flag — one branch beside the
  read-only flag's (ADR-0109's own mapping), no patch-tier growth
  (the word is runtime state the app's door moves, never a declarable
  DSL word).
- **The living documents re-agree at M106 and 0.17.0.** The ADR count
  (a hundred and eleven), the architecture register (ADR-0110 scopes
  M106), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `required` row) all name the tier; the
  corpus grew 119 → 120 by the coordinated rule below.

## Consequences

- **The corpus grew 119 → 120 by the coordinated rule:**
  `required/snapshot_1k_required` — 1,000 required fields, the word's
  snapshot price — against the same 1,000-node input arena bare
  (`placeholder/snapshot_1k_bare`, the M96 case), the
  `required_snapshot_overhead` ratio budget ≤ 1.25× (the bool rides
  the snapshot the field rides). The m-suite pins re-agree at 120.
- **The AT reads the contract before it asks.** The word is the
  mapping the platform ATs turn into the form check; the empty
  submit's wall stays where it always was — the app's own submit
  handler — because a tree that refused on the word's behalf would
  price a contract it does not own.
- **Consumers get the whole vocabulary for free.** The
  `examples/required.rs` beat is the shape: the required card field,
  amount slider and agree checkbox beside the optional note, the AT's
  write landing on the required field untouched, the live door moving
  the word.
