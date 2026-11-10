# ADR-0028: M22 scope — the unavailable control (the availability tier, the refused route's corpus, the micro-debt the sweep priced)

- **Status:** Accepted (2026-11-10)
- **Implements:** the availability tier — the candidate the
  audit-before-the-cut pass priced after M21 emptied the ledger — plus
  the surface work the M16–M21 pattern leaves open: the dead
  introspection accessors the M21 cut's own gate could not see, and the
  two `rsx!` diagnostics still short of the ADR-0025 species
- **Companion:** ADR-0026/ADR-0027 (the value tier whose two-door shape
  the availability tier mirrors), ADR-0016/ADR-0017 (the refuse-don't-
  munge rule the refusals ride), ADR-0025 (the pointer capture the wall
  semantics compose with), ADR-5 (the required role and label the tier
  obeys), ADR-9 (the dependency discipline that keeps the flag a
  hand-rolled `bool`), architecture §11.1 (the contract this widens),
  §18 (the M22 roadmap row)

## Decision

M21 closed the ledger with the value tier complete — writable (M20) and
reportable (M21) — while the other property every real form needs stayed
absent: no widget could *report that it must refuse*. A spent button
looked, read, focused, and answered exactly like a live one; the AT
announced a control the tree claimed was actionable, and the press died
silently. M22 takes the availability tier — the HTML/ARIA `disabled`
vocabulary, the AccessKit `disabled` flag — and delivers it as one
contract:

- **One source of truth on the `Widget` trait.** `enabled()` answers
  whether the control accepts interaction; the default is available, and
  the four actionable controls (button, checkbox, text input, slider)
  carry the flag. The default `focusable()` derivation consults it — an
  unavailable control is never a focus candidate and never a Tab stop,
  so a widget that overrides `enabled` needs no `focusable` override of
  its own. Reporting widgets and decorations are always available.
- **The flag rides the same snapshot and sink.** Pass 9 carries
  `TreeNode.enabled` — no second channel — and the a11y sink maps it
  faithfully onto AccessKit's `disabled` property: the AT reads the
  unavailable control instead of announcing an action the widget will
  refuse. An availability write queues `PAINT | A11Y` like every write,
  so the re-announcement is a node-only delta (the M9 shape).
- **Every door refuses honestly.** The pointer is a wall (the gesture is
  consumed so it never clicks through to an ancestor, it arms nothing,
  and it queues no masks — an idle frame; the HTML `disabled`
  semantics); the keyboard never reaches the control (no focus, and a
  routed activation key is ignored so the framework keeps it); the IME
  is answered with silence; and the AT tier is refused, never munged
  (the M11 rule). Focus never rests on an unavailable control: pass 3's
  sentinel discipline grows one predicate — focus held by a control
  that becomes unavailable does not survive the frame, and the release
  repaints the focus ring exactly like a move.
- **Two doors, the value tier's shape.** The fixed door
  (`.enabled(bool)` at construction, `set_enabled` in place) answers the
  move it made — a write that lands where the flag already is queues
  nothing (the M21 idle-frame property, at the widget seam). The binding
  door (`enabled_binding`, shipped on the button where the reactive form
  rides) tracks a `RenderBinding<bool>` the flux bridge drives — the
  `Progress::bound` rule, at the availability tier. The other three
  controls grow the binding door when a real UI prices it (the ledger's
  re-open rule).
- **The look tells the truth.** An unavailable control paints the theme's
  muted constants — the muted rail and the muted text the disabled state
  borrows — so the pixels and the tree agree; the shipped corpus's
  enabled pixels are untouched (the muted branch is new code, and the
  blessed PNGs stay byte-identical).
- **The consumer demonstration ships.** `examples/disabled.rs` drives
  the bound door from the consumer side — the *Enable sending* checkbox's
  one write batch flips the send button's node through the bridge — and
  walks the mutate seam at startup, where `set_enabled` answers the move
  it made (none: the binding owns the flag), with `tracing` spans at the
  new `alacer::disabled` target naming each route.
- **The corpus grows 56 → 57 by the coordinated rule.** The refused
  press's criterion case `disabled/route_100_refused_presses_1k` — one
  hundred AT presses against a disabled button on the 1,000-node arena —
  is measured (`cargo bench`, 100 samples), published with its baseline
  entry, and gated same-host in `contracts.json`; the bench, the
  baseline, and the gate land together (ADR-0011, CONTRIBUTING's
  publishing rule). Every M-suite pin moves to 57 in the same cut,
  including the m20 and m21 suites' own — the one-cycle drift the
  coordinated rule prices.
- **The m1 corpus grows 36 → 37 scenarios.** The availability window
  joins the styled corpus — the pixels are the regression net behind the
  demonstration.
- **The micro-debt the sweep priced rides the cut.** The M21 gate greps
  the wgpu backend for its dead stats surface while the same-named dead
  accessor still ships in the software renderer — `SoftwareRenderer::
  glyphs_rendered` (zero call sites; the field it read was write-only
  bookkeeping) goes, and the m22 suite extends the source gate to the
  renderer so the species cannot regress there. Two more M18-species
  accessors go: `PassPipeline::clear_observer` and
  `PassPipeline::timings_enabled` (zero call sites; the sibling setters
  they introspect are used), and `ViewRoot::root` — the `root_id`
  duplicate whose doc claimed a caller that does not exist — is deleted
  and the three canon call sites read `root_id()` like everyone else.
- **The weak errors reach the ADR-0025 species' remainder.** The `rsx!`
  fall-through diagnostic — any token that is not an element, `for`,
  `if`, or text lands on a bare syn parse error with no rsx context —
  now names what stood there and the four shapes it could have been;
  the unterminated-`{` text error carries the `help:` the escape hatch
  ships (`{{` for a literal brace).

## Consequences

- The availability vocabulary is core's, not a widget dialect: the
  snapshot, the sink, the focus passes, and the cycle route all read the
  one trait method.
- The two-door shape is demonstrated on the button; the checkbox, text
  input, and slider answer `set_enabled` only. A real UI that needs a
  reactive checkbox re-opens the seam with the binding door priced (the
  ledger's re-open rule).
- The refused route is a published, gated cost: an unavailable control
  must stay in the routing cost class — never a re-layout, never a
  re-materialization.
- The corpus pins moved to 57 together; the next milestone's coordinated
  rule starts from 57.
- The m22 exit suite's source gate covers the software renderer's dead
  accessors — the M21 gate's blind spot is closed where it was found.

## Kill criteria (re-open rules)

- A control needs a *third* availability state (read-only, hydrated,
  loading) → grow the vocabulary as a property on the snapshot node, not
  a second flag (the M18 re-open rule's shape).
- A door needs a shape the refuse-don't-munge rule cannot express (a
  disabled control that must still answer `ScrollTo*`) → re-open the
  seam with the shape as a trait method beside `enabled`.
- The binding door needs to drive more than the button → grow it per
  control the way the value tier grew, one priced demonstration at a
  time.
