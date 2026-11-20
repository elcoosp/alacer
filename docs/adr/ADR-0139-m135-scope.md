# ADR-0139: M135 scope — the tree tier: the hierarchical collection the AT walks by disclosure and depth

- **Status:** Accepted (2026-11-20)
- **Implements:** the words the hierarchical collection speaks — and
  the tree never said. Through M134 the snapshot said the dialog tier
  (ADR-0138), the meter tier (ADR-0137), the search field
  (ADR-0136), the named pane (ADR-0135) — and the *flat* collections
  had the words: the M6 table grid (rows × columns), the M108 picks,
  the M110 disclosures, the M113 depths, the M119 places and spans.
  The *hierarchical* collection — the file explorer, the category
  outline, the nested navigation — had no word of its own: an app
  mounting one reached for the table's grid (a grid's geometry is
  rows × columns; a tree's is parentage — the wrong geometry wearing
  the wrong word) or for a bare list whose members could say nothing
  about the branches they stood in. [`Role::Tree`]
  (crates/alacer-core/src/widget.rs) carries the collection word and
  [`Role::TreeItem`](crates/alacer-core/src/widget.rs) the member
  word AccessKit's own roles spell — the member-and-container shape
  (ADR-0128) grown to the hierarchical tier.
- **Companion:** ADR-0128 (the member-and-container discipline the
  pair grows), ADR-0112/0114/0117/0123 (the M108/M110/M113/M119 words
  the members ride — the pick, the disclosure, the depth, the place
  and span), ADR-0113/0121/0124 (the M109/M117/M120 container words
  the collection rides — the capacity, the updating word, the
  declared region), ADR-0099 (the M95 description tier — the help
  text beside the name), ADR-0138 (the M134 dialog tier — the
  named-container precedent the tier follows), ADR-5 (the
  required-name rule the collection and its members join), ADR-2 (the
  one-currency discipline the change never leaves), ADR-9 (the DAG
  the change never touches), architecture §7.2 pass 9 (the snapshot
  the words ride), §17 (the register), §18 (the M135 roadmap row)

## Decision

- **Two words, the collection's own.** [`Role::Tree`]
  (crates/alacer-core/src/widget.rs) carries the "reads as a
  hierarchical collection" fact — AccessKit's own `Tree` role spells
  it — and [`Role::TreeItem`](crates/alacer-core/src/widget.rs) the
  "reads as a branch or a leaf" fact its `TreeItem` role spells. Both
  join the required-name set (an unnamed collection reads as a bare
  box, an unnamed member a bare mark — the ADR-5 rule the named roles
  carry) and neither joins the focusable set (a tree and its items
  are not controls: they hold no focus and answer no activation — the
  M126/M129 container rule, at the tree tier; the controls the app
  mounts inside the items hold the focus). The roles are *tree*
  words, never layout flavors: the AT announces the collection before
  it walks the branches, the platform's expansion conventions keying
  off the words. **Pre-registered re-open rule:** a consumer needing
  member-level focus and activation (the walk stopping on items, the
  AT's default action opening a branch) grows it beside the funnel,
  priced by a real tree — never a second walk, and never by mutating
  what these words mean.
- **The hierarchy is the arena's own nesting, never a second
  structure.** The producers ship without a seam change:
  [`Tree`](crates/alacer-widgets/src/tree.rs) and
  [`TreeItem`](crates/alacer-widgets/src/tree.rs) — a member's
  children ride the arena exactly a row's cells do (nested items
  under a branch, the app's own controls under either), each child
  keeping its role, its label, and its doors. No door behind the
  words changes — the tier prices the words and nothing else; the
  members' behaviour is the behaviour it always was.
- **Every word the members say is a tier the tree already published,
  never a second vocabulary.** The pick rides
  [`Widget::selected`](crates/alacer-core/src/widget.rs) (M108,
  ADR-0112), the disclosure [`Widget::expanded`] (M110, ADR-0114), the
  depth [`Widget::level`] (M113, ADR-0117), the place
  [`Widget::position_in_set`] (M119, ADR-0123); on the container the
  span rides [`Widget::size_of_set`] (M119), the capacity
  [`Widget::multiselectable`] (M109, ADR-0113), the updating word
  [`Widget::busy`] (M117, ADR-0121), the declared region
  [`Widget::live`] (M120, ADR-0124), beside the name and the
  description tier's word (M95, ADR-0099). The words' writes are the
  doors those tiers published — each reporting the move, the write
  that lands where the word already is queueing nothing (the M21
  idle-frame property, at the tree tier). One role per node: a
  `TableRow` keeps the grid word it published, a bare list member the
  honest absence — the tree's words are the tree's own.
- **The living documents re-agree at M135 and 0.17.0** (the README's
  status block, register line, widget vocabulary, and example counts;
  the catalog; the register; the roadmap; the m-suite pins moving
  M134 → M135 across the chain; the CHANGELOG). The corpus pin
  re-agrees at 144 — the tier grows no snapshot route (the words are
  the container's and member's own, the shape the M124/M127/M129/
  M131/M134 named containers rode), and its pricing door is the
  widgets crate's own bench ([`benches/tree.rs`]
  (crates/alacer-widgets/benches/tree.rs), `tree/name_write_256`
  against `tree/idle_write_256`), outside the published corpus by
  ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The words' cost is priced in
  [`crates/alacer-widgets/benches/tree.rs`]
  (crates/alacer-widgets/benches/tree.rs): `tree/name_write_256` — the
  moving rename (the `String`'s write the member's word takes) against
  `tree/idle_write_256`, the write that lands where the word already
  is (the honest floor of the same write at the widget seam, the M21
  idle path priced). The pair rides outside the published baseline:
  the coordinated rule grows the corpus at the next published cut,
  never at a widget bench's. The m-suite pins re-agree at 144, the
  m135 suite's pin documenting the same shape.
- **The AT walks the branches at last.** A screen reader walking a
  mounted tree announces the collection, then the members — each
  member's pick, disclosure, depth and place reading from the words
  the tiers already mapped, the platform's expansion conventions
  keying off the collection word. The words wall nothing: the
  members' doors answer exactly as they always did, and the focus the
  tree's content needs stays the walk the framework already publishes.
- **Consumers get the whole vocabulary for free.** The
  `examples/tree.rs` beat is the shape: the named collection with its
  nested members (the branch open, the leaf picked, the app's own
  controls riding the doors they always carried), the reads reading
  back the way the app wired them — the neighbours untouched.
