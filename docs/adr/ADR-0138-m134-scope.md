# ADR-0138: M134 scope — the dialog tier: the window the modal vocabulary speaks as "this is a dialog"

- **Status:** Accepted (2026-11-19)
- **Implements:** the word every modal vocabulary speaks as "this is
  a dialog" — and the tree never said. Through M133 the snapshot said
  the meter tier (ADR-0137), the search field (ADR-0136), the named
  pane (ADR-0135), the instant on/off control (ADR-0134) — and the
  modal window had no word: an app wiring a confirmation, a settings
  sheet, a file picker mounted the conversation in a bare panel (a
  container wearing no name — the set the AT cannot name is a set it
  cannot walk, the ADR-5 rule the named roles carry) or for a second
  tab panel wearing the wrong word (a tab panel names *the selected
  page's content*; a dialog names *the window the conversation lives
  in*, and the AT's vocabulary names the difference: AccessKit's
  `Dialog` role is its own fact). [`Role::Dialog`]
  (crates/alacer-core/src/widget.rs) carries the window word the AT
  announces before it walks the conversation — the named-container
  shape (ADR-0128) grown to the modal tier.
- **Companion:** ADR-0135 (the M131 tab-panel tier — the named-
  container shape the tier grows), ADR-0031 (the M25 focus scope — the
  walk the modal semantics ride, never a second channel), ADR-0099
  (the M95 description tier — the help text beside the name),
  ADR-0128 (the member-and-container discipline the word rides),
  ADR-0137 (the M133 meter tier — the one-word shape the tier
  follows), ADR-5 (the required-name rule the window joins), ADR-2
  (the one-currency discipline the change never leaves), ADR-9 (the
  DAG the change never touches), architecture §7.2 pass 9 (the
  snapshot the word rides), §17 (the register), §18 (the M134 roadmap
  row)

## Decision

- **One word, the window's own.**
  [`Role::Dialog`](crates/alacer-core/src/widget.rs) carries the
  "reads as a modal window" fact — AccessKit's own `Dialog` role
  spells it. The window joins the required-name set (an unnamed
  window reads as a bare box — the ADR-5 rule the named roles carry)
  and never the focusable set (a window is not a control: it holds no
  focus and answers no activation — the M126/M129 container rule, at
  the modal tier; the controls inside it hold the focus). The role is
  a *tree* word, never a layout flavor: the AT announces "dialog"
  beside the name before it walks the conversation, the platform's
  modal conventions keying off the word.
- **The window is the content's layout, never a second channel to
  it.** The producer ships without a seam change:
  [`Dialog`](crates/alacer-widgets/src/dialog.rs) — the children ride
  the arena exactly as a [`TabPanel`](crates/alacer-widgets/src/tab_panel.rs)'s
  do (a vertical run, the shape a conversation is), each child
  keeping its role, its label, and its doors: the field edits, the
  button presses, the window naming what they read as. No door behind
  the word changes — the tier prices the word and nothing else; the
  members' behaviour is the behaviour it always was. The word's
  writes are the programmatic `set_name` (view reconciliation — the
  word is the app's declaration, never a user door) and the
  description tier's `set_description` (M95, ADR-0099 — the write
  that lands where the text already is queues nothing, the M21
  idle-frame property at the description tier).
- **The modal semantics ride the walk, never a second channel.**
  Bounding the walk to the window's subtree while it is up is the
  focus scope the M25 tier published
  ([`PassPipeline::focus_move_in`](crates/alacer-core/src/passes.rs),
  ADR-0031) — the consumer's own door, one walk, the same candidate
  rules, the wrap, the refusal that queues no masks. The word names
  the window; the walk bounds it. **Pre-registered re-open rule:** a
  consumer needing the window to own the modal semantics itself (the
  walk auto-scoped while it is up) grows the scoping beside the
  funnel, priced by a real window — never a second walk.
- **The word never munges the neighbours.** One role per node: a
  `TabPanel` keeps the pane word it published (ADR-0135), a bare
  panel the honest absence, the root container the canon's word —
  the window's word is the window's own. The `dialog` element joins
  the registry (create/patch beside the tab-panel element's: `name`,
  the description word, the style words), and the sink maps the role
  one branch beside the tab panel's — no patch-tier growth.
- **The living documents re-agree at M134 and 0.17.0** (the README's
  status block, register line, and widget table; the catalog; the
  register; the roadmap; the m-suite pins moving M133 → M134 across
  the chain; the CHANGELOG). The corpus pin re-agrees at 144 — the
  tier grows no snapshot route (the word is the container's own, the
  shape the M124/M127/M129/M131 named containers rode), and its
  pricing door is the widgets crate's own bench
  ([`benches/dialog.rs`](crates/alacer-widgets/benches/dialog.rs),
  `dialog/name_write_256` against `dialog/idle_write_256`), outside
  the published corpus by ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The word's cost is priced in
  [`crates/alacer-widgets/benches/dialog.rs`](crates/alacer-widgets/benches/dialog.rs):
  `dialog/name_write_256` — the moving rename (the `String`'s write
  the window's word takes) against `dialog/idle_write_256`, the write
  that lands where the word already is (the honest floor of the same
  write at the widget seam, the M21 idle path priced). The pair rides
  outside the published baseline: the coordinated rule grows the
  corpus at the next published cut, never at a widget bench's. The
  m-suite pins re-agree at 144, the m134 suite's pin documenting the
  same shape.
- **The AT names the window the toolkit always owed it.** A screen
  reader walking a mounted dialog announces "dialog" beside the name
  before it walks the conversation — the modal conventions keying off
  the word. The window walls nothing: the members' doors answer
  exactly as they always did, and the walk the modal semantics need
  is the walk the M25 tier already published — the word names the
  window, it walls nothing.
- **Consumers get the whole vocabulary for free.** The
  `examples/dialog.rs` beat is the shape: the named window with its
  conversation (the field and the submit riding the doors they always
  carried), the reads reading back the way the app wired them — the
  neighbours untouched.
