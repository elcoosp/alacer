# ADR-0135: M131 scope — the tab-panel tier: the named pane the selected page reveals

- **Status:** Accepted (2026-11-19)
- **Implements:** the container every tabbed vocabulary speaks as
  "this holds the selected page's content" — and the tree never
  named. Through M130 the snapshot said the instant on/off control
  (ADR-0134), the tool strip (ADR-0133), the bounded number field
  (ADR-0132), the page selector and the strip it reads as
  (ADR-0131), the section title (ADR-0130), the hyperlink
  (ADR-0129), the one-of-N control and its group (ADR-0128) — and
  the strip's *content* still had no word: an app wiring a tab strip
  mounted the pages (the member, M127) inside the named strip (the
  container, M127) and showed the selection's content in a bare box —
  a container wearing no name (the set the AT cannot name is a set it
  cannot walk, the ADR-5 rule the named roles carry) or for a second
  tab list wearing the wrong word (a tab list *names the pages'
  strip*; a tab panel *names the page's content*, and the AT's
  vocabulary names the difference: AccessKit's `TabPanel` role is its
  own fact). [`Role::TabPanel`](crates/alacer-core/src/widget.rs)
  carries the pane word the AT announces before it reads the
  selection's content — the member-and-container shape (ADR-0128)
  grown to its third word: the member ([`Role::Tab`]), the strip
  ([`Role::TabList`]), the pane ([`Role::TabPanel`]).
- **Companion:** ADR-0131 (the M127 tab tier — the member and the
  strip the pane completes; the pane word is the tier's own
  pre-registered third), ADR-0128 (the member-and-container shape the
  pane re-proves: the children keep the words they already carry, the
  container the word that names what they read as), ADR-0005 (the
  ADR-5 required-name rule — an unnamed pane a bare box), ADR-0099
  (the description tier the pane's help text rides), ADR-2 (the
  one-currency discipline the word rides), ADR-9 (the DAG the change
  never touches), architecture §10.3 (the activation parity), §17
  (the register), §18 (the M131 roadmap row)

## Decision

- **One role, the pane's own word.**
  [`Role::TabPanel`](crates/alacer-core/src/widget.rs) carries the
  "holds the selected page's content" fact AccessKit's `TabPanel`
  role spells — not a bare box wearing no name, not a second tab list
  wearing the wrong word: the AT announces the pane's name before it
  reads the content, and the children keep the roles and labels they
  already carry. The sink maps `Role::TabPanel` onto AccessKit's
  `TabPanel`, one branch beside the tab pair's (the M127 `Tab` /
  `TabList` branches this completes), no patch-tier growth — the pane
  carries no property the tree does not already spell. The role joins
  the required-name set (ADR-5) — and never the focusable set: a pane
  is not a control, it holds no focus and answers no activation, the
  content holding the focus the word walls nothing (the M126 heading
  and the M129 toolbar's own rule, at the container tier).
- **The pane is the content's layout, never a second channel to it.**
  The [`TabPanel`](crates/alacer-widgets/src/tab_panel.rs) container
  is a vertical run — the children ride the arena exactly as a
  `Column`'s do, each child keeping its role, its label, and its
  doors: the button presses, the label reads, the pane naming what
  they read as. No door behind the word changes — the tier prices the
  word and nothing else, the content's behaviour the behaviour it
  always was. Which page's content the pane holds is the app's
  wiring: the [`Tab::on_select`](crates/alacer-widgets/src/tab.rs)
  handler and the view's programmatic doors, the exclusivity rule the
  tab tier published (the framework never rewrites widgets it does
  not own, A1). The word's writes are the programmatic
  [`TabPanel::set_name`] and the description tier's
  [`TabPanel::set_description`] (view reconciliation — the write that
  lands where the text already is queues nothing, the M21 idle-frame
  property at the description tier); there is no user door to a
  container's name.
- **The living documents re-agree at M131 and 0.17.0.** The ADR count
  (a hundred and thirty-six), the architecture register (ADR-0135
  scopes M131), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `tab_panel` row) all name the
  tier; the m-suite corpus pins re-agree at 144 — the tier grew no
  snapshot route (the word is the container's own, the shape the
  M124/M127/M129 named containers rode), and its pricing door is the
  widgets crate's own bench, outside the published corpus by
  ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The pane's cost is priced in
  [`crates/alacer-widgets/benches/tab_panel.rs`](crates/alacer-widgets/benches/tab_panel.rs):
  `tab_panel/name_write_256` — the moving rename (the `String`'s
  write the pane's word takes) against `tab_panel/idle_write_256`,
  the write that lands where the word already is (the honest floor of
  the same write at the widget seam). The pair rides outside the
  published baseline: the coordinated rule grows the corpus at the
  next published cut, never at a widget bench's. The m-suite pins
  re-agree at 144, the m131 suite's pin documenting the same shape.
- **The AT names the pane the toolkit always owed it.** A screen
  reader walking a mounted strip announces "tab list, Mailboxes" —
  the pages beside it — and then "tab panel, Inbox content" before it
  reads the selection's body. The pane walls nothing: the content's
  availability, focus, and activation doors answer exactly as they
  always did — the word names what the content reads as, it walls
  nothing.
- **Consumers get the whole vocabulary for free.** The
  `examples/tab_panel.rs` beat is the shape: the named strip, the
  pages riding one signal, the named pane whose word and body
  recompute from the same write — the children keeping their own
  doors, the neighbours untouched.
