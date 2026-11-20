# ADR-0141: M137 scope — the menu tier: the named strip of menus, the summoned list, and the command entries, said by the tree

- **Status:** Accepted (2026-11-20)
- **Implements:** the words the command vocabulary speaks. Through
  M136 the snapshot said the tooltip tier (ADR-0140), the tree tier
  (ADR-0139), the dialog tier (ADR-0138), the meter tier (ADR-0137),
  the search field (ADR-0136), the named pane (ADR-0135) — and the
  command vocabulary had two shapes that were not the menu's: the tool
  strip (M129, ADR-0133 — a horizontal run of tools always on) and the
  has-popup word (M115, ADR-0119 — the *opener's* "this opens a
  menu"). The menu itself had no word of its own: an app wiring a menu
  bar reached for a [`Toolbar`](crates/alacer-widgets/src/toolbar.rs)
  wearing the wrong word (a toolbar *offers tools*, a menu bar *names
  the menus the commands live in*), for a
  [`Dialog`](crates/alacer-widgets/src/dialog.rs) wearing the wrong
  transient (a dialog is a modal window the walk bounds itself into; a
  menu is a transient list the opener summons), or for a bare
  [`Button`](crates/alacer-widgets/src/button.rs) wearing no entry word
  at all (the command the AT cannot name is a command it cannot fire —
  the ADR-5 rule the named roles carry). [`Role::MenuBar`],
  [`Role::Menu`] and [`Role::MenuItem`]
  (crates/alacer-core/src/widget.rs) carry the words AccessKit's own
  `MenuBar`, `Menu` and `MenuItem` roles spell — the
  member-and-container shape (ADR-0128) grown to the command tier, its
  first three-word tier (the strip, the summoned list, the entry).
- **Companion:** ADR-0128 (the member-and-container discipline the
  words grow), ADR-0133/0134/0135/0138/0139/0140 (the
  M129/M130/M131/M134/M135/M136 named-container precedents the tier
  follows — the tool strip, the instant control, the pane, the window,
  the collection, the bubble), ADR-0119 (the has-popup word — the
  *opener's* fact the menu never replaces), ADR-0016 (the M10 action
  tier the entry's default action rides), ADR-0028 (the M22
  availability tier the entry's doors answer), ADR-5 (the required-name
  rule the words join), ADR-2 (the one-currency discipline the change
  never leaves), ADR-9 (the DAG the change never touches),
  architecture §7.2 pass 9 (the snapshot the words ride), §17 (the
  register), §18 (the M137 roadmap row)

## Decision

- **Three words, the vocabulary's own.** [`Role::MenuBar`] carries the
  strip word ("reads as a set of menus"), [`Role::Menu`] the summoned
  list ("reads as a list of commands"), [`Role::MenuItem`] the entry
  ("fires one command") — AccessKit's own roles spell each. All three
  join the required-name set (an unnamed strip, list or entry reads as
  a bare box or a bare mark — the ADR-5 rule the named roles carry).
  The strip and the list never join the focusable set (a container is
  not a control: it holds no focus and answers no activation — the
  M126/M129 container rule, at the menu tier; the entries hold the
  focus); the entry joins it (a menu item *is* a control — the walk
  lands it, the funnel answers). The menu's word is *not* the toolbar's
  worn twice (a toolbar offers tools, a menu bar names menus) and *not*
  the dialog's worn down (a dialog is a modal window the walk bounds
  itself into; a menu is a transient list the opener summons, asking no
  scope and answering no walk). **Pre-registered re-open rule:** a
  consumer needing the checkable or radio entry species grows them
  beside the funnel, priced by a real menu — never a second walk, and
  never by mutating what these words mean.
- **The strip and the list are the entries' layout, never a second
  channel to them.** [`MenuBar`](crates/alacer-widgets/src/menu.rs) —
  a horizontal run, the shape a strip is;
  [`Menu`](crates/alacer-widgets/src/menu.rs) — a vertical run, the
  shape a list is. The children ride the arena exactly a toolbar's and
  a pane's do, each child keeping its role, its label, and its doors:
  the entries fire, the labels read, the containers naming what they
  read as. No door behind the words changes — the tier prices the
  words and nothing else.
- **Presence is the app's wiring, never a widget flag.** Whether the
  list is up is the view's own mount and unmount — the reconciler's
  shape since M1, the tooltip tier's presence rule (ADR-0140) at the
  menu tier. A `visible` flag would be a second truth the tree would
  have to guess at: the snapshot says what is mounted, and what is
  mounted is what the app declared this frame. The strip's and the
  list's one programmatic write is `set_name` (view reconciliation);
  the entry's writes are `set_label`, `set_enabled` (the M22
  availability write reporting the move — the write that lands where
  the flag already is queues nothing, the M21 idle-frame property),
  and `set_on_click`.
- **The entry's funnel is the button tier's own, never a second
  activation vocabulary.** The click on release, Enter while focused —
  the M10 parity (Space beside it, the button's own rule) — and the
  AT's default action the door answers: one funnel, the doors the
  `Button` already publishes (`on_a11y_action` refusing honestly when
  unavailable, the M11 rule; the activation naming itself on the M70
  rider when the keyboard fires it). The tier grows no new action
  species, no new property class.
- **The living documents re-agree at M138 and 0.17.0** (the README's
  status block, register line, widget vocabulary, and example counts;
  the catalog; the register; the roadmap; the m-suite pins moving
  M136 → M138 across the chain; the CHANGELOG). The corpus pin
  re-agrees at 144 — the tier grows no snapshot route (the words are
  the containers' and the entry's own, the shape the M124/M127/M129/
  M131/M134/M135/M136 named containers rode), and its pricing door is
  the widgets crate's own bench
  ([`benches/menu_bar.rs`](crates/alacer-widgets/benches/menu_bar.rs),
  `menu_bar/name_write_256` against `menu_bar/idle_write_256`),
  outside the published corpus by ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The words' cost is priced in
  [`crates/alacer-widgets/benches/menu_bar.rs`]
  (crates/alacer-widgets/benches/menu_bar.rs): `menu_bar/name_write_256`
  — the moving rename (the `String`'s write the strip's word takes)
  against `menu_bar/idle_write_256`, the write that lands where the
  word already is (the honest floor of the same write at the widget
  seam, the M21 idle path priced). The pair rides outside the
  published baseline: the coordinated rule grows the corpus at the
  next published cut, never at a widget bench's. The m-suite pins
  re-agree at 144, the m137 suite's pin documenting the same shape.
- **The AT walks and fires the commands, at last.** A screen reader
  encountering a mounted menu bar announces the strip, walks the menus
  it names, announces the summoned list, and lands the entries — the
  command vocabulary the words could not say before — and every child
  reads as itself, the words walling nothing: the controls around the
  vocabulary answer exactly as they always did.
- **Consumers get the whole vocabulary for free.** The
  `examples/menu_bar.rs` beat is the shape: the named strip, the
  summoned list mounted exactly while the signal says open (presence
  the app's wiring), the entries firing the commands the app wired,
  the reads reading back the way the tree carries them — the
  neighbours untouched.
