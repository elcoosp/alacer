# ADR-0130: M126 scope — the heading tier: the section title the document outline walks

- **Status:** Accepted (2026-11-19)
- **Implements:** the word every document vocabulary speaks about its
  structure — "this names a section" — and the tree never carried.
  Through M125 the snapshot said the hyperlink and its open verb
  (ADR-0129), the one-of-N control and the group it reads as
  (ADR-0128), the set words, the form facts, the collection facts, the
  declared live region, the chord, the capacity, the language — and
  the section title had no role: an app wiring a document outline
  reached for a plain label, and the AT walked the page as one flat
  run of text — the rotor's quick-nav, the outline every reader skims
  by, had nothing to land on. The tier ships the role:
  `Role::Heading` carrying the "names a section" fact AccessKit's own
  `Heading` role spells, the depth in the outline riding the level
  word the M113 tier published — the control prices the word and
  nothing else
- **Companion:** ADR-0017 (the M113 level tier — the depth word the
  heading's outline rides, never a second depth vocabulary), ADR-0005
  (the ADR-5 required-name rule an unnamed heading answers — an
  unnamed section title reads as bare styled text, the outline the AT
  cannot walk), ADR-0087 (the honest-absence presence rule at the
  tier — a heading that declared no depth announces none, never a
  guessed level), ADR-0001 (the one-currency discipline the word
  joins), ADR-9 (the DAG the change never touches), architecture §10.2
  (the a11y contract), §17 (the register), §18 (the M126 roadmap row)

## Decision

- **One role, one word.**
  [`Role::Heading`](crates/alacer-core/src/widget.rs) carries the
  "names a section" fact AccessKit's `Heading` role spells — not a
  bold label wearing bigger glyphs: the AT announces a heading as
  structure, and the outline is the app's
  ([`Heading`](crates/alacer-widgets/src/heading.rs), the producer
  whose text *is* the name). The depth rides
  [`Widget::level`](crates/alacer-core/src/widget.rs), the M113
  tier's own word (AccessKit's `Level` property, zero-based beside
  ARIA's one-based `aria-level`): the heading's natural carrier, never
  a second depth vocabulary, the sink's existing mapping serving the
  tier with no new branch. The role joins the required-name set
  (ADR-5): an unnamed heading reads as bare styled text — the outline
  the AT cannot walk.
- **A heading is not a control.** The tier prices the word and
  nothing else: a heading holds no focus (the role's own answer to
  `focusable`), answers no activation, and the AT's presses land on
  the widgets the section contains — the word names structure, it
  walls nothing, the section's content reading as it always did. The
  write is the app's: [`Heading::set_level`](crates/alacer-widgets/src/heading.rs)
  the one depth write (view reconciliation, `None` restoring the
  honest absence — the flat title, never a guessed level), the write
  that lands where the depth already is queueing nothing (the M21
  idle-frame property, at the heading tier).
- **The snapshot carries it; the sink maps it.** The pass-9 fill needs
  no new field — the role is the fact and the depth rides the
  `TreeNode::level` word M113 published — and the sink maps
  `Role::Heading` onto AccessKit's `Heading` role, one branch beside
  the link's. No patch-tier growth beyond the DSL door the widget set
  shares (the `heading` element: `name`, `level`, `language`,
  `color`, `font_size`), no second channel: the level write
  re-announces exactly the heading's node.
- **The living documents re-agree at M126 and 0.17.0.** The ADR count
  (a hundred and thirty-two), the architecture register (ADR-0130
  scopes M126), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `heading` row) all name the
  tier; the m-suite corpus pins re-agree at 144 — the tier grew no
  snapshot route (the role is the fact), and its pricing door is the
  widgets crate's own bench, outside the published corpus by
  ADR-0128's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The heading's cost is priced in
  [`crates/alacer-widgets/benches/heading.rs`](crates/alacer-widgets/benches/heading.rs):
  `heading/level_write_route_256` — the moving write (the compare, the
  state write, the moved answer) against `heading/idle_write_256`, the
  M21 idle path (the write that lands where the depth already is,
  answering `false` and queueing nothing). The pair rides outside the
  published baseline: the coordinated rule grows the corpus at the
  next published cut, never at a widget bench's. The m-suite pins
  re-agree at 144, the m126 suite's pin documenting the same shape.
- **The AT walks the outline the toolkit always owed it.** A screen
  reader walking a mounted document announces "heading, Installation"
  and the quick-nav lands section by section — the depth the level
  word carries, the name the ADR-5 contract requires. A title that
  declared no depth announces the honest absence, never a guessed
  level.
- **Consumers get the whole vocabulary for free.** The
  `examples/heading.rs` beat is the shape: the outline under the word
  — the root title the honest absence beside, the nested sections
  riding their depths — the deepening write the one mask the outline
  costs, the neighbours untouched.
