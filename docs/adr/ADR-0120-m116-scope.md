# ADR-0120: M116 scope — the role-description tier: the role a custom control is read as, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every custom-control vocabulary speaks
  about what a widget is *read as* — "this slider is a star rating,
  that button is a wizard page" — and the tree never said. Through
  M115 the snapshot said what a control *does* (the description,
  ADR-0099), what an empty field *hints* (the placeholder, ADR-0100),
  the span a value rides (the range tier, ADR-0101), the form's ask
  (the required tier, ADR-0110), the form's rejection (the invalid
  tier, ADR-0111), the collection's pick, capacity and disclosure
  (ADR-0112/ADR-0113/ADR-0114), the field's suggestion contract (the
  auto-complete tier, ADR-0115), the sorted column's order (the sort
  direction tier, ADR-0116), the nested member's depth (the level
  tier, ADR-0117), the set's current item (the current tier,
  ADR-0118), the opener's target (the has-popup tier, ADR-0119) — and
  the fact the AT turns into "star rating" while it walks the form was
  the one specialization fact the tree carried nothing for: the AT
  announced the generic role — "slider", "button" — and never the
  read-as one, even where the app mounted a specialized control. The
  declared-but-unreported gap (the M94 pattern), cashed at the
  role-description tier: "a consumer needs the custom control to read
  as exactly the role it implements, and as the generic role when it
  claims no specialization → the tree carries AccessKit's
  `RoleDescription` property (ARIA's `aria-roledescription`), the doors
  answering unchanged beside the word — the word prices the word and
  nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0118/ADR-0119 (the
  tiers the word grows beside — the single-string shape the word
  copies where the tier carries no kind: the description precedent at
  the string tier, no enum mirrored because AccessKit's
  `RoleDescription` *is* a string), ADR-0087 (the presence rule at the
  tier: `None` the honest absence — the generic role the AT read
  before the tier, never a guessed specialization), ADR-9 (the DAG the
  change never touches), architecture §11 (the a11y contract), §17
  (the register), §18 (the M116 roadmap row)

## Decision

- **One word, the generic role beside it.**
  [`Widget::role_description`](crates/alacer-core/src/widget.rs)
  answers `Option<&str>` — `None` the honest absence (the ADR-0087
  presence rule at the tier: a control that claims no specialization
  is read as the generic role it already was, never a guessed one),
  the word riding beside the role the snapshot's `Role` already
  spells: the star rating behind an ordinary slider-shaped control,
  the wizard page behind an ordinary button, the checklist step behind
  a plain field. No kind enum mirrors this tier — AccessKit's
  `RoleDescription` is a string property (the description precedent,
  ADR-0099, at the string tier), so the mapping is one branch with no
  translation and no vocabulary to get wrong. Role-description is not
  a wall: a custom control's doors answer exactly as any control's
  (the word names the read-as role, it walls nothing — the control's
  press and focus routes the widget's own), so the tier grows no
  refusal vocabulary and prices no behavior beyond the word itself.
- **The producers join without a seam change.** The custom-control
  widgets ship the full shape —
  [`Button::role_description`](crates/alacer-widgets/src/button.rs),
  the builder naming the initial read-as role, beside
  [`Button::set_role_description`](crates/alacer-widgets/src/button.rs),
  the app's live door through the mutate seam (`None` the honest
  absence restored; the write that lands where the word already is
  queues nothing, the M21 idle-frame property) — and
  [`Slider::role_description`](crates/alacer-widgets/src/slider.rs),
  the constructor word the canonical specialization rides ("star
  rating", the ARIA document's own example shape), the M107 join rule:
  the slider joins without a seam change, its value funnel the doors
  it always was. The word is a custom-control word: the containers and
  collection members answer nothing (a container reports its members'
  words, a header reports its order — the read-as role is the
  specialized control's own fact), so no producer joins there.
- **The snapshot carries it; the sink maps it.**
  `TreeNode::role_description` rides the same pass-9 snapshot every
  other property rides, and the sink maps AccessKit's
  `RoleDescription` property — one branch beside the description
  word's (ADR-0099's own mapping), no patch-tier growth (the word is
  runtime state the app's door moves, never a declarable DSL word).
- **The living documents re-agree at M116 and 0.17.0.** The ADR count
  (a hundred and twenty-one), the architecture register (ADR-0120
  scopes M116), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `role_description` row) all
  name the tier; the corpus grew 134 → 135 by the coordinated rule
  below, and the m-suite corpus pins re-agree at 135.

## Consequences

- **The corpus grew 134 → 135 by the coordinated rule — the M111
  precedent's reuse:** `role_description/snapshot_1k_role_descriptions`
  — 1,000 buttons, the role-description word's snapshot price —
  against the same 1,000-node button arena bare
  (`current/snapshot_1k_buttons_bare`, the M114 case the tier reuses
  by the M111 precedent; the M95 pricing rule — the word is a button
  word, so the honest comparison is buttons against buttons, the
  M108/M110 denominator shape). The
  `role_description_snapshot_overhead` ratio budget ≤ 1.25× (the word
  rides the snapshot the button rides, the tier's price the one
  `Option<String>` the node carries beside it). The m-suite pins
  re-agree at 135.
- **The AT reads the specialization while it walks the form.** The
  word is the mapping the platform ATs turn into the control's
  "star rating" — beside the name and the generic role the snapshot
  already carried, the fact a screen reader answers "Rating, star
  rating, slider" with no extra route, no extra pass. The absence is
  the contract's other half: a control that claims no specialization
  reads as the ordinary role it is, never as a guessed one, so a plain
  slider does not start announcing a specialization it does not hold.
- **Consumers get the whole vocabulary for free.** The
  `examples/role_description.rs` beat is the shape: the star-rating
  slider beside the plain slider, the wizard-page button beside the
  plain button, the AT's press landing on the plain button untouched
  (the word names a read-as role, it walls nothing), the live door
  moving the word — specializing, rewording, clear — the honest
  absence restored.
