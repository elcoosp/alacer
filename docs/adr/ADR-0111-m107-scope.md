# ADR-0111: M107 scope — the invalid tier: the form's rejection, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every form vocabulary speaks when the value
  lands wrong — "this one is not valid as given" — and the tree never
  said. Through M106 the snapshot said what a control *does* (the
  description, ADR-0099), what an empty field *hints* (the
  placeholder, ADR-0100), the span a value rides (the range tier,
  ADR-0101), the number behind the word (the numeric tier, ADR-0103),
  whether the value is the app's (the read-only tier, ADR-0109), the
  form's ask (the required tier, ADR-0110) — and the fact the AT turns
  into "invalid" while the user edits was the one form fact the tree
  carried nothing for: the AT announced the ask, the hint, the span —
  and never the rejection. The declared-but-unreported gap (the M94
  pattern), cashed at the invalid tier: "a consumer needs the form's
  rejection to read as exactly that, and of what kind → the tree
  carries AccessKit's `Invalid` property (ARIA's `aria-invalid`),
  the doors answering unchanged beside the word — the word prices the
  word and nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0110 (the tier the
  word grows beside — the form's ask, and now its rejection; the
  constructor-plus-live-door shape the tier copies), ADR-0087 (the
  presence rule at the tier: `None` the honest absence), ADR-9 (the
  DAG the change never touches), architecture §10.2 (the editing
  tier), §11 (the a11y contract), §17 (the register), §18 (the M107
  roadmap row)

## Decision

- **One word, one property, the kind beside it.**
  [`Widget::invalid`](crates/alacer-core/src/widget.rs) answers
  `Option<InvalidKind>` — `None` the honest absence, the ADR-0087
  presence rule at the tier. The kind is the neutral payload type the
  tree carries ([`InvalidKind`], the ADR-0101 `ValueRange` shape):
  `True` the bare rejection (`aria-invalid="true"`), `Grammar` and
  `Spelling` the prose-error kinds the AT reads natively — ARIA's own
  three values, mirrored so the mapping is one branch with no
  translation. Invalid is not a wall: an invalid control's write doors
  answer exactly as any editable's (the AT reads the rejection *while*
  the user edits, which is the whole point of the word — a tree that
  refused on the word's behalf would price a contract it does not
  own), so the tier grows no refusal vocabulary and prices no behavior
  beyond the word itself.
- **The producers join without a seam change.** `TextInput` ships the
  full shape — the [`invalid(kind)`] builder naming the initial word,
  [`TextInput::set_invalid`](crates/alacer-widgets/src/text_input.rs)
  the app's live door (the caller's `PAINT | A11Y` request paying the
  re-announcement; the write that lands where the word already is
  queues nothing, the M21 idle-frame property; `None` the honest
  absence restored) — and the value tier's controls override beside it
  ([`Slider::invalid`], [`Checkbox::invalid`],
  crates/alacer-widgets/src/slider.rs,
  crates/alacer-widgets/src/checkbox.rs): the trait door already
  answers them, the ADR-0110 producers' shape at the tier.
- **The snapshot carries it; the sink maps it.** `TreeNode::invalid`
  rides the same pass-9 snapshot every other property rides, and the
  sink maps AccessKit's `Invalid` property — one branch beside the
  required flag's (ADR-0110's own mapping), no patch-tier growth (the
  word is runtime state the app's door moves, never a declarable DSL
  word).
- **The living documents re-agree at M107 and 0.17.0.** The ADR count
  (a hundred and twelve), the architecture register (ADR-0111 scopes
  M107), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `invalid` row) all name the tier; the
  corpus grew 120 → 121 by the coordinated rule below. The M107
  campaign also re-agreed the pins earlier cuts trailed: the m-suite
  corpus pins and the README fragments the M102–M106 campaigns left
  behind ("All 118 criterion cases", "M0 through M102", "a hundred and
  seven decision records", m20's gates-len pin) now read the M107 cut's
  truth — the coordinated rule prices the whole chain, not the newest
  suite.

## Consequences

- **The corpus grew 120 → 121 by the coordinated rule:**
  `invalid/snapshot_1k_invalid` — 1,000 invalid fields, the word's
  snapshot price — against the same 1,000-node input arena bare
  (`placeholder/snapshot_1k_bare`, the M96 case), the
  `invalid_snapshot_overhead` ratio budget ≤ 1.25× (the word and its
  kind ride the snapshot the field rides). The m-suite pins re-agree
  at 121.
- **The AT reads the rejection while the user edits.** The word is the
  mapping the platform ATs turn into the form's "invalid" — the
  re-submit's wall stays where it always was, the app's own submit
  handler, because a tree that refused on the word's behalf would
  price a contract it does not own. The kind gives prose-error ATs
  their native surface: `grammar` and `spelling` are words the
  platform may read itself.
- **Consumers get the whole vocabulary for free.** The
  `examples/invalid.rs` beat is the shape: the misspelled card field
  (`spelling`), the out-of-contract amount slider (`true`), the
  mismatched agree checkbox (`grammar`) beside the valid note, the
  AT's write landing on the invalid field untouched, the live door
  moving the word and its kind, `None` restoring the honest absence.
