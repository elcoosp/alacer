# ADR-0017: M11 scope — the UTF-16 selection model joins the editing and a11y contracts

- **Status:** Accepted (2026-11-09)
- **Implements:** the §11.5 contract's *operable* leg at the text boundary:
  reachable, named, actionable — and now **selectable**, the range an AT
  reads and repositions, not just the value string
- **Companion:** ADR-0016 (the action tier this widens), ADR-5 (the a11y
  contract), architecture §10.3 (the UTF-16 contract unit), §18 (the M11
  roadmap row)

## Decision

M10's "what this does not do" recorded the deferred seam: "no `SetValue`/
text-selection actions (the UTF-16 selection model is still an open seam)".
M11 closes it in the same "join the contract" shape as M8/M9/M10 — one
model, three consumers:

- **The model is one anchor and one caret, in UTF-16 units** — the contract
  unit §10.3 already fixed for the IME. `alacer-core` grows
  `TextSelection { anchor, focus }` (degenerate = the bare caret) and the
  `Widget` trait grows `selection()`, optional like `toggled()`/`table_slot()`
  before it: a default-`None` method, a consuming widget in the same
  milestone, never speculative vocabulary.
- **The editor consumes it.** `TextInput` grows the anchor state and the
  GTK-shaped keyboard contract: Shift+arrows/Home/End extend from the
  anchor, Ctrl/Cmd+A selects all, and a plain arrow over a live selection
  collapses to its near edge before it steps. Every edit consumes the
  selection — typing replaces it, Backspace/Delete clear it, a click
  collapses it, an IME commit is an insertion like any other (one rule, no
  exceptions). The highlight paints from the prefix widths the text seam
  already measures for the caret — the selection costs no new measuring
  machinery.
- **The a11y tree carries it.** Pass 9 copies `selection()` into
  `TreeNode.selection` beside `value`, and `alacer-a11y` maps it onto
  AccessKit's `TextSelection` — anchor and focus as character indexes on
  the value-bearing node itself. AccessKit's full rich-text model points
  `TextPosition`s at `TextRun` children; Alacer's a11y tree is coarse and
  the value lives on the editable node, so the positions point at the node
  that owns the string — a documented simplification, **UNVERIFIED** at
  strict platform adapters, carried honestly like every other unverified
  seam.
- **The AT tier widens faithfully.** The pre-M11 platform munged nothing at
  the *press* tier (M10's work) but dropped the whole value tier. The
  mapping now carries `SetValue` and `ReplaceSelectedText` payloads through
  (`ActionData::Value` → the core action's `String`) and `SetTextSelection`
  as character indexes, dropping payload-less or off-target requests at the
  boundary, traced — the same refuse-don't-munge contract, now about
  *data*. The vocabulary growth costs `A11yAction` its `Copy` (the payload
  rides the action); refusal stays observable exactly because a refused
  action allocates and moves nothing.
- **Composition is a11y-visible.** While a preedit is active the selection
  report drops (`None`): the IME owns the canvas, and reporting a stale
  range through a composition window would lie to the AT. The preedit event
  requests `PAINT | A11Y` so the suppression itself reaches the sink.

### 1. Why this is the milestone

The recorded unblocked seams after M10 were announcements/live regions, the
value tier, and a `Slider`. The value tier is the one that crosses three
subsystems at once — editing (the caret-only input could not even *select*,
so Backspace could not delete a range and Ctrl+A typed an `a`), paint (a
selection nobody sees is a rumor), and a11y (an AT cannot read or
reposition a range the tree does not carry) — which is exactly the shape of
milestone this roadmap ratifies: a contract that one currency carries to
every consumer, with the corpus and the budgets growing by the coordinated
rule. Announcements deserve their own milestone because their delta
semantics (a sink-vocabulary change with priority and interruption rules)
are separable; the selection is not separable from the editing contract it
completes. The alternative candidates (a hit-test spatial index, atlas
eviction) remain single-pass performance repairs — they move a number, not
a contract.

### 2. The seams that did not move

- **The corpus grows by the coordinated rule, nothing else moves.** One
  bench case (`a11y_action/route_100_setvalue_1k` — the value tier keeps
  the action route in the same cost class as a pointer route, payload
  transport included), one regression contract — 48 → 49 cases; the
  M5/M6/M7/M8/M9/M10 pin suites move together.
- **Pass 9's and pass 10's shapes are untouched.** The selection rides the
  existing snapshot; the value tier consumes through the existing
  `PAINT | A11Y` merge; a refused `SetValue` still costs an idle frame
  (test-enforced both directions: the button refuses the value tier, the
  input refuses the press tier).
- **The DAG is untouched.** `TextSelection` lives in `alacer-core`
  (`widget.rs`), so neither the a11y crate nor the widgets learn the word
  "accesskit"; the platform keeps the only adapter dependency (ADR-9).

### 3. What this does not do

- No clipboard (`Copy`/`Cut`/`Paste` are platform-seam actions — arboard is
  not a workspace dependency, and a clipboard milestone owns the
  permission/escape story with it).
- No caret-from-click mapping: a pointer press collapses the selection but
  does not move the caret, because placing a caret from an x-coordinate
  needs the bounds-aware hit-test seam (the widget's resolved bounds do not
  reach `on_pointer_event`). Recorded as the follow-up seam.
- No word-wise movement or double-click word selection (needs the same
  seam), no multi-line selection (the input is single-line), no mouse-drag
  extension (it needs caret-from-click), no `SelectionChanged` announcements
  (the live-region milestone's vocabulary).
- No per-action capability advertisement (AccessKit `supports_actions`):
  refusal is honest and observable today; advertising is a widget-canon
  sweep once more actions exist.
- No `Slider`: the natural `Increment`/`Decrement` consumer arrives with
  the slider, and its value tier is numeric (`NumericValue`), a different
  faithful-mapping shape than the text tier this milestone owns.

## Consequences

- The workspace stays at one milestone per minor cut: the next release
  (0.9.0) carries M11.
- The corpus is 49 cases; the M5–M10 suites pin the number together and the
  M11 suite asserts all six pins plus the published baseline/contract pair.
- The baselines are re-published from the collecting host (the ADR-0016
  procedure): the new case's median rides the same snapshot, and the
  same-host regression gates re-arm against it.
- A screen-reader user can now read *where the caret is*, set the
  selection, replace the value, and type over a range from the AT side —
  and a keyboard user can finally select and range-edit, which the
  caret-only field could not express at any price.
