# ADR-0119: M115 scope — the has-popup tier: the opener's target, said by the tree

- **Status:** Accepted (2026-11-18)
- **Implements:** the word every menu-and-picker vocabulary speaks
  about what a control opens — "this one opens a menu, that one a
  dialog" — and the tree never said. Through M114 the snapshot said
  what a control *does* (the description, ADR-0099), what an empty
  field *hints* (the placeholder, ADR-0100), the span a value rides
  (the range tier, ADR-0101), the form's ask (the required tier,
  ADR-0110), the form's rejection (the invalid tier, ADR-0111), the
  collection's pick, capacity and disclosure (ADR-0112/ADR-0113/
  ADR-0114), the field's suggestion contract (the auto-complete tier,
  ADR-0115), the sorted column's order (the sort direction tier,
  ADR-0116), the nested member's depth (the level tier, ADR-0117), the
  set's current item (the current tier, ADR-0118) — and the fact the
  AT turns into "opens a menu" while it walks the toolbar was the one
  opener fact the tree carried nothing for: the AT announced the name,
  the currency, the form facts — and never the target. The
  declared-but-unreported gap (the M94 pattern), cashed at the
  has-popup tier: "a consumer needs the opener's target to read as
  exactly that, and as an honest absence when the control opens
  nothing → the tree carries AccessKit's `HasPopup` property (ARIA's
  `aria-haspopup`), the doors answering unchanged beside the word —
  the word prices the word and nothing else"
- **Companion:** ADR-2 (the one-currency discipline the word joins —
  the word rides the `A11Y` mask the snapshot pass already pays),
  ADR-0099/ADR-0100 (the a11y-only prop-class tier the word rides —
  no paint, no refusal, one mapping branch), ADR-0116/ADR-0117/
  ADR-0118 (the tiers the word grows beside — the kind-mirroring shape
  the word copies: AccessKit's own enum values mirrored so the mapping
  is one branch with no translation, the `CurrentKind` precedent at
  the enum tier), ADR-0087 (the presence rule at the tier: `None` the
  honest absence — the attribute the AT read before the tier, never a
  guessed target), ADR-9 (the DAG the change never touches),
  architecture §11 (the a11y contract), §17 (the register), §18 (the
  M115 roadmap row)

## Decision

- **One word, the kind beside it.**
  [`Widget::has_popup`](crates/alacer-core/src/widget.rs) answers
  `Option<HasPopupKind>` — `None` the honest absence (the ADR-0087
  presence rule at the tier: a control that opens nothing announces no
  target, never a guessed one), the kind
  ([`HasPopupKind`](crates/alacer-core/src/widget.rs)) riding beside
  the word: `Menu` the menu button's menu, `Listbox` the combobox's
  pick list, `Tree` the folder browser's tree, `Grid` the date
  picker's cells, `Dialog` the confirm opener's dialog — AccessKit's
  own five enum values mirrored, the attribute's unset state the
  absence the `Option` already spells (ARIA's `true` rides `Menu`, the
  value the platform maps). Has-popup is not a wall: an opener's doors
  answer exactly as any control's (the word names the target, it walls
  nothing — the opener's press and focus routes the widget's own), so
  the tier grows no refusal vocabulary and prices no behavior beyond
  the word itself.
- **The producer joins without a seam change.** The opener widget
  ships the full shape —
  [`Button::has_popup`](crates/alacer-widgets/src/button.rs), the
  builder naming the initial target, beside
  [`Button::set_has_popup`](crates/alacer-widgets/src/button.rs), the
  app's live door through the mutate seam (`None` the honest absence
  restored; the write that lands where the word already is queues
  nothing, the M21 idle-frame property). The word is an opener word:
  the containers and collection members answer nothing (a container
  reports its members' words, a row reports its depth — the opener's
  target is the opener's own fact), so no producer joins there.
- **The snapshot carries it; the sink maps it.** `TreeNode::has_popup`
  rides the same pass-9 snapshot every other property rides, and the
  sink maps AccessKit's `HasPopup` property — one branch beside the
  current word's (ADR-0118's own mapping), no patch-tier growth (the
  word is runtime state the app's door moves, never a declarable DSL
  word).
- **The living documents re-agree at M115 and 0.17.0.** The ADR count
  (a hundred and twenty), the architecture register (ADR-0119 scopes
  M115), the roadmap row (§18), the CHANGELOG's unreleased block, and
  the examples catalog (the `has_popup` row) all name the tier; the
  corpus grew 133 → 134 by the coordinated rule below, and the m-suite
  corpus pins re-agree at 134.

## Consequences

- **The corpus grew 133 → 134 by the coordinated rule — the M111
  precedent's reuse:** `has_popup/snapshot_1k_has_popups` — 1,000
  buttons, the has-popup word's snapshot price — against the same
  1,000-node button arena bare (`current/snapshot_1k_buttons_bare`,
  the M114 case the tier reuses by the M111 precedent; the M95 pricing
  rule — the word is a button word, so the honest comparison is
  buttons against buttons, the M108/M110 denominator shape). The
  `has_popup_snapshot_overhead` ratio budget ≤ 1.25× (the word rides
  the snapshot the button rides, the tier's price the one `Option` the
  node carries beside it). The m-suite pins re-agree at 134.
- **The AT reads the target while it walks the toolbar.** The word is
  the mapping the platform ATs turn into the opener's "opens a menu" —
  beside the name and the currency the M114 word already carried, the
  fact a screen reader answers "Actions, opens a menu" with no extra
  route, no extra pass. The absence is the contract's other half: a
  control that opens nothing reads as the ordinary button it is, never
  as a guessed target, so a plain button does not start announcing a
  target it does not hold.
- **Consumers get the whole vocabulary for free.** The
  `examples/has_popup.rs` beat is the shape: the menu button beside
  the combobox button, the tree toggle, the grid picker and the dialog
  opener, the plain button between them, the AT's press landing on the
  plain button untouched (the word names a target, it walls nothing),
  the live door moving the word — opening menus, switching to dialogs,
  clear — the honest absence restored.
