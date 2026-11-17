# ADR-0098 — M94 scope: the checkbox grows the mixed state

Milestone: **M94** (this cut). Parent contract: the Widget trait's own
toggle contract (§11.2) and the a11y sink's M6 mapping standing beside
the vocabulary's declared-but-unproduced state: `ToggleState::Mixed` —
"partially checked (tri-state parents)" — mapped onto AccessKit's
`Toggled::Mixed` in the same cut, and carried by no widget since.

## The ask

The tree has always been able to announce a mixed toggle; the widget
vocabulary has never been able to produce one. The select-all parent —
the classic tri-state summary, "some of the children are" — was a story
the a11y walker could carry and the widget set could not tell: a
consumer composing a group had only the binary pair, so the parent
either lied (`checked: true` while half the children were not) or
dropped the semantics entirely. The state the vocabulary declares is
the state the widget must produce — the M92 derive rule at the toggle
tier: the required pair the trait names, the widget answers.

## The shape

- **The tri-state door** (`Checkbox::state(ToggleState)`): the builder
  beside `checked(bool)` — the binary door keeps its own contract, the
  tri-state door mounts the summary. `toggle_state()` and `is_mixed()`
  are the read side (test introspection and app logic); `toggled()`
  answers the full vocabulary the tree announces.
- **The resolution rule** (the cut's own decision): the mixed state is
  a *summary*, never a third bit. Every user door — the pointer's
  click, the keyboard's Space, the AT's default press — resolves a
  mixed summary **fully checked**: the handler receives the resolved
  boolean, the tree announces `On`, and the resolved control toggles
  like any checkbox after it. No `FnMut(ToggleState)` handler, no
  cycle, no protocol the app must interpret — the one answer every
  toolkit's tri-state parent gives, priced where the route is known.
- **The programmatic door** (`Checkbox::set_state(ToggleState)`): the
  view-reconciliation path writes the summary verbatim — the app's
  select-all recomputation lands where the patch tier drives it, fires
  no handler, and the write tier queues its own masks (the m22
  discipline).
- **The painter**: the mixed box fills with the accent and draws the
  horizontal dash — two deterministic quads beside the check mark's
  strokes, the same font-free discipline the snapshot corpus ships by.
  The look still tells the truth the tree carries (the M22 rule at the
  paint tier).
- **The view DSL** (`state:` word): `"on"` / `"off"` / `"mixed"` ride
  the checkbox element's create and patch — unknown words refused by
  absence (the patcher applies only the props it understands, the
  canvas painter's own rule); the binary `checked` flag keeps its
  contract beside the word.
- **The tri-state budget** (the coordinated rule's bench half): the
  a11y snapshot pass over a 1,000-node tri-state arena against the
  same 1,000-node binary arena — priced as a ratio budget
  (`toggle_tri_state_overhead`, ≤ 1.10×), the dash the only difference
  the pipeline prices. The m-suite pins re-agree at 103 (M92's pair
  beside M93's beside M94's, one cut).

## Exit criteria

- The mixed summary lands in the a11y snapshot: a tri-state parent
  mounted through the real pipeline announces `Mixed` through the only
  official query API, the binary children announcing their pair beside
  it (the m94 exit suite).
- The user doors resolve the summary: the AT's press, the pointer's
  click, and the keyboard's Space each land a mixed parent fully
  checked — the handler seeing the resolved boolean, the second press
  toggling the resolved bit honestly.
- The resolution costs its masks: the funnel's `PAINT | A11Y` land and
  the passes run; the programmatic write re-announces through the same
  masks without firing a handler.
- The painter tells the tri-state truth: the dash, the check, and the
  empty box pinned in pixels (`tri_state_group.png`, the blessed
  corpus).
- The view lands the word: `checkbox { state: "mixed" }` mounts the
  summary and patches it in place on re-render (the view tier's
  `update`, the patch report naming the diff).
- The wrapper is honest: `toggle/snapshot_1k_tri_state` vs
  `toggle/snapshot_1k_binary` — the ratio budget holding the summary's
  cost at the snapshot route's noise floor.
- The corpus grows 101 → 103 by the coordinated rule (M94's pair
  beside M92's and M93's, one cut): the ratio budget and the same-host
  gates joining the contract set, the m-suite pins re-agreeing at 103.
- `examples/tri_state.rs` drives the beats from the consumer side —
  the select-all parent recomputing from the tasks, the resolution
  cascading in one batch, the patch channel answering with one scoped
  re-render.

## Re-open rules (pre-registered)

- a consumer needs the *group* semantics priced by the framework — the
  parent recomputed from children automatically — → the composition
  grows a helper beside the canon builders (the todo canon's own
  shape), the widget staying the vocabulary's producer and the app
  staying the owner of the flags (A1: handlers write signals).
- a consumer needs other controls to carry the vocabulary (a
  tri-state menu item, a tree row's summary) → the trait contract
  already answers them (`toggled()` is not checkbox-shaped), the
  producers joining without a seam change.
- a consumer prices the resolution rule's *other* arm — the first
  press resolving to unchecked — → the rule is one `match` arm at the
  widget's funnel, changed where the route is known, the suite's door
  tests re-pinning the answer beside it.
