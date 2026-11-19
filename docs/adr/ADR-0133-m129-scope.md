# ADR-0133: M129 scope — the toolbar tier: the named strip a set of tools reads as

- **Status:** Accepted (2026-11-19)
- **Implements:** the container every editing-and-formatting vocabulary
  speaks as "this strip holds the tools" — and the tree never named.
  Through M128 the snapshot said the bounded number field the AT walks
  by arrows (ADR-0132), the page selector and the strip it reads as
  (ADR-0131), the section title the document outline walks
  (ADR-0130), the hyperlink and its open verb (ADR-0129), the one-of-N
  control and its group (ADR-0128), the set words, the form facts, the
  collection facts — and the tool strip had no word: an app wiring one
  reached for a bare row (a container wearing no name — the set the AT
  cannot name is a set it cannot walk, the ADR-5 rule the named roles
  carry) or for a tab list wearing the wrong word (a tab strip
  *selects pages*; a toolbar *offers tools*, and the AT's vocabulary
  names the difference: AccessKit's `Toolbar` role is its own fact).
  [`Role::Toolbar`](crates/alacer-core/src/widget.rs) carries the
  strip word the AT announces before it walks the tools — the
  tab-list/radio-group shape (the named container, ADR-0128's
  member-and-container discipline) at the tool tier
- **Companion:** ADR-0128 (the member-and-container shape the strip
  re-proves: the members keep the words they already carry, the
  container the word that names the set), ADR-0131 (the M127 tab-list
  half — the same container shape at the page tier, the strip the
  pages' layout, never a second channel to them), ADR-0005 (the ADR-5
  required-name rule — an unnamed strip a bare box), ADR-2 (the
  one-currency discipline the word rides), ADR-9 (the DAG the change
  never touches), architecture §10.3 (the activation parity), §17 (the
  register), §18 (the M129 roadmap row)

## Decision

- **One role, the container's own word.**
  [`Role::Toolbar`](crates/alacer-core/src/widget.rs) carries the
  "reads as a set of tools" fact AccessKit's `Toolbar` role spells —
  not a bare row wearing no name, not a tab strip wearing the wrong
  word: the AT announces the strip's name before it walks the tools,
  and the tools keep the roles and labels they already carry. The sink
  maps `Role::Toolbar` onto AccessKit's `Toolbar`, one branch beside
  the tab-list/radio-group pair's. The role joins the required-name
  set (ADR-5) — and never the focusable set: a toolbar is not a
  control, it holds no focus and answers no activation, the tools
  holding the focus the word walls nothing (the M126 heading's own
  rule, at the container tier).
- **The strip is the tools' layout, never a second channel to them.**
  The [`Toolbar`](crates/alacer-widgets/src/toolbar.rs) container is a
  horizontal run — the children ride the arena exactly as a `Row`'s
  do, each tool keeping its role, its label, and its doors: the button
  presses, the input edits, the strip naming the set they read as. No
  door behind the word changes — the tier prices the word and nothing
  else, the members' behaviour the behaviour it always was. The
  programmatic write ([`Toolbar::set_name`], view reconciliation) is
  the word's one write; there is no user door to a container's name —
  the word is the app's declaration, the label the tree carries.
- **The living documents re-agree at M129 and 0.17.0.** The ADR count
  (a hundred and thirty-four), the architecture register (ADR-0133
  scopes M129), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `toolbar` row) all name the
  tier; the m-suite corpus pins re-agree at 144 — the tier grew no
  snapshot route (the word is the container's own, the shape the
  M124/M127 named containers rode), and its pricing door is the
  widgets crate's own bench, outside the published corpus by
  ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The strip's cost is priced in
  [`crates/alacer-widgets/benches/toolbar.rs`](crates/alacer-widgets/benches/toolbar.rs):
  `toolbar/name_write_256` — the moving rename (the `String`'s write
  the strip's word takes) against `toolbar/idle_write_256`, the write
  that lands where the word already is (the honest floor of the same
  write at the widget seam). The pair rides outside the published
  baseline: the coordinated rule grows the corpus at the next
  published cut, never at a widget bench's. The m-suite pins re-agree
  at 144, the m129 suite's pin documenting the same shape.
- **The AT names the tool strip the toolkit always owed it.** A screen
  reader walking a mounted strip announces "toolbar, Formatting" —
  the set word before the members, the members keeping the words they
  always carried. The strip walls nothing: the tools' availability,
  focus, and activation doors answer exactly as they always did — the
  word names the set the tools read as, it walls nothing.
- **Consumers get the whole vocabulary for free.** The
  `examples/toolbar.rs` beat is the shape: the named strip under the
  word, the tools the members' own controls, the presses reading back
  the way the app wired them — the neighbours untouched.
