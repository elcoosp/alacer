# ADR-0131: M127 scope — the tab tier: the page selector and the named strip it reads as

- **Status:** Accepted (2026-11-19)
- **Implements:** the control every preferences-and-mail vocabulary
  speaks as "one page of many, shown" — and the tree never carried.
  Through M126 the snapshot said the section title the document
  outline walks (ADR-0130), the hyperlink and its open verb
  (ADR-0129), the set words, the form facts, the collection facts, the
  declared live region, the chord, the capacity, the language — and
  the tab strip had no tier: an app wiring one reached for a row of
  buttons (the wrong shape — a button's activation is a *press* with
  no after-state, a tab's activation *selects the page*) or for bare
  labels wearing no selection fact at all. The tier ships two roles
  beside one machinery: `Role::Tab` carrying the "selects a page" fact
  AccessKit's own `Tab` role spells, and `Role::TabList` the named
  strip the set reads as — the radio tier's own member-and-container
  shape (ADR-0128) at the page tier
- **Companion:** ADR-0112 (the M108 selected tier — the word the
  tab's selection rides, never a second selection vocabulary),
  ADR-0128 (the member-and-container shape the strip re-proves: the
  user's doors select and never deselect, the exclusivity the app's
  wiring, the redundant ask the M21 idle frame), ADR-0005 (the ADR-5
  required-name rule — an unnamed page a bare mark, an unnamed strip
  a bare box), ADR-0023 (the availability wall every door answers
  behind), ADR-0076 (the keyboard activation naming itself — the
  Space-select streams the `Keyboard` rider), ADR-2 (the one-currency
  discipline the activation joins), ADR-9 (the DAG the change never
  touches), architecture §10.3 (the activation parity), §17 (the
  register), §18 (the M127 roadmap row)

## Decision

- **Two roles, one machinery.**
  [`Role::Tab`](crates/alacer-core/src/widget.rs) carries the
  "selects a page" fact AccessKit's `Tab` role spells — not a button
  wearing the strip's clothes: the AT announces a tab as a page
  selector, and the selection rides
  [`Widget::selected`](crates/alacer-core/src/widget.rs), the M108
  tier's own word (AccessKit's `Selected` property): the tab's natural
  carrier, never a second selection vocabulary, the sink's existing
  mapping serving the tier with no new branch.
  [`Role::TabList`](crates/alacer-core/src/widget.rs) is the named
  strip the set reads as — the AT announces the strip's name before it
  walks the pages, the container the pages' horizontal layout (a
  [`TabList`](crates/alacer-widgets/src/tab.rs) is a row, never a
  second channel to its children). Both roles join the required-name
  set (ADR-5).
- **Three doors, one selection, one contract.** A click, the Space key
  while focused, and the AccessKit default action all *select* — the
  selected page never deselects itself ([`Tab::select`], the radio
  tier's funnel at the page tier). A redundant ask is consumed and
  moves nothing: no masks, no handler — the M21 idle frame. The
  exclusivity is the app's wiring: the [`Tab::on_select`] handler is
  where the strip's writer clears the siblings (the framework never
  rewrites widgets it does not own, A1); the one deselect door is the
  programmatic write ([`Tab::set_selected`], view reconciliation) —
  programmatic state, never a user door, the checkbox's own rule. The
  Space-select streams one `ActionRecord` with `Keyboard` beside it
  (M70, ADR-0076), answered only when the selection moved. The
  availability tier guards every door (M22, ADR-0028): an unavailable
  tab refuses, never munges (the M11 rule).
- **The snapshot carries it; the sink maps it.** The pass-9 fill needs
  no new field — the selection rides the `TreeNode::selected` word
  M108 published — and the sink maps `Role::Tab` onto AccessKit's
  `Tab` role and `Role::TabList` onto AccessKit's `TabList`, two
  branches beside the radio pair's. No patch-tier growth beyond the
  DSL door the widget set shares (the `tab` element: `name`,
  `selected`, `on_select`, `description`, `accent`; the `tab_list`
  element: `name`), no second channel: the select re-announces exactly
  the tab's node.
- **The living documents re-agree at M127 and 0.17.0.** The ADR count
  (a hundred and thirty-two), the architecture register (ADR-0131
  scopes M127), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `tab` row) all name the tier;
  the m-suite corpus pins re-agree at 144 — the tier grew no snapshot
  route (its word is the selected tier's), and its pricing door is the
  widgets crate's own bench, outside the published corpus by
  ADR-0128's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The tab's cost is priced in
  [`crates/alacer-widgets/benches/tab.rs`](crates/alacer-widgets/benches/tab.rs):
  `tab/select_route_256` — the moving select (the state write, the
  mask request, the handler call) against `tab/redundant_ask_256`, the
  M21 idle path (the consumed ask that moves nothing, queues nothing,
  fires nothing). The pair rides outside the published baseline: the
  coordinated rule grows the corpus at the next published cut, never
  at a widget bench's. The m-suite pins re-agree at 144, the m127
  suite's pin documenting the same shape.
- **The AT names the page the toolkit always owed it.** A screen
  reader walking a mounted strip announces "tab, Inbox, selected" —
  the page word the vocabulary reserves for the selector, the
  selection the selected word carries. The wall is the contract's
  other half: an unavailable tab reads as the muted text it is, every
  door refusing before the request is parsed.
- **Consumers get the whole vocabulary for free.** The
  `examples/tab.rs` beat is the shape: the named strip under the word,
  the pages mounting selected and unselected from the signal, the
  select handler writing the one write the exclusivity rides, the
  status line reading the selection back the way the tree carries it —
  the neighbours untouched.
