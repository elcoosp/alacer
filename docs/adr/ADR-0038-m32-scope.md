# ADR-0038: M32 scope — the value tier joins the view registry

- **Status:** Accepted (2026-11-12)
- **Implements:** the re-open rule the M6 canon's registry work left
  open — "a widget vocabulary the `rsx!` DSL cannot express is a
  second-class citizen of its own toolkit → the value tier's widgets
  register as view elements, priced by a real view app — never a
  widget-only tier beside a view-only canon"
- **Companion:** ADR-0026 (the slider whose funnel the element rides),
  ADR-0027 (the progress whose report the element carries), ADR-0009
  (the testing-productization discipline the registry tests ride), the
  M6 canon's `checkbox`/`text_input` registrations (the create/patch
  shape this ADR extends), ADR-0037 (the milestone this cut shares),
  architecture §6.1 (the registry seam the elements join), §18 (the
  M32 roadmap row)

## Decision

The value tier has been widget vocabulary since M20/M21 — the slider's
one funnel and the progress's honest report — but the view registry
knew nothing of them: `default_registry` listed the M0 and M6 canons,
so a `rsx!` view could not carry a slider or a progress, the hot-reload
path could not patch one, and the view edition of every value-tier app
fell back to manual assembly. M32 closes the gap:

- **The value tier registers.** `slider` and `progress` join
  `alacer_widgets::default_registry` with the create/patch pairs every
  canon element rides: create reads `name`/`min`/`max`/`step`/`value`/
  `on_change`/`accent` (slider) and `name`/`min`/`max`/`value`/`fill`
  or `binding` (progress); patch takes what has an in-place writer —
  the name and the value — and everything else is create-time. The
  patch answers are the widgets' own: the slider's programmatic write
  clamps and snaps inside the widget (the funnel every door rides),
  fires no handler, and reports "visual" only when the funnel moved the
  value (the widget's own `total_cmp` equality — never `f32::eq`);
  the progress's `set_value` bool *is* the M21 idle-frame property, a
  write that lands where the value already is queueing no masks.
- **The prop vocabulary grows the two types the tier needs.**
  `PropValue::NumberHandler` carries the funnel's value
  (`SharedNumberHandler`, `on_change` — the flag/text handlers' rule at
  a value; the patcher's handler-churn exemption covers it: re-hanging
  the same handler repaints nothing); `PropValue::NumberBinding` rides
  the f32 `RenderBinding` (`binding: my_f32_binding` — the value typed
  by what it carries, `From<RenderBinding<f32>>` the same rule the text
  binding rode). A bound progress's patch re-hangs the same binding
  slot idempotently — never a visual change, the bound label's rule at
  the value tier.
- **The bridge still owns the value.** The element's binding carries
  the number; the signal→node edge stays the app's assembly-time
  `bind_value` call (the `counter_view` rule — the arena lookup, the
  masks the caller names). The view DSL expresses the *shape*; the
  flux bridge expresses the *reactivity*; neither grows a second
  channel (axiom A1).
- **The demonstration is the view edition.** `examples/value_view.rs`
  mounts the M20 slider and the M21 progress from one `rsx!` view: the
  slider's `on_change` routes the funnel to one signal, the bridge fans
  the write out to the bar, the label, and the tree's answer. The rsx
  suite pins the contract headlessly: both elements reachable and
  valued through the a11y tree from the first frame, a real drag
  heard by the view handler in gesture order, the rebuild patching in
  place (zero churn), the bound progress keeping the binding's number.
  The registry suite pins the create/patch semantics: the funnel's
  clamp-and-snap at create and patch, the idle frame on a no-op write,
  the binding's idempotent re-hang.
- **The living documents re-agree at M32 and 0.17.0.** The README's
  status block (M0 through M32), the ADR count (thirty-nine), the
  widgets usage bullet, and the example table; the catalog's row; the
  register (37 → 39 with this ADR and ADR-0037, the ADR-0000 row
  joining at last); the roadmap row; the m-suite pins moving
  M30 → M32 and 37 → 39 across the chain; and the CHANGELOG stages the
  milestone under `[Unreleased]`.

## Consequences

- The widget vocabulary is unchanged: the elements register the
  widgets that exist — no new widget code, no new a11y contract, the
  M20/M22 pins stand untouched.
- `PropValue` grows two variants — a breaking change to exhaustive
  matches, the growth the M6 flag/text handlers priced the same way:
  pre-1.0, in-tree matches moved with the cut (the patcher's
  handler-churn exemption, the wire's serializer), and the workspace's
  own match sites are the only ones (the registry test's introspection
  matches on `PropValue::Text` carry a wildcard arm, the hot suite's
  rule).
- The hot path carries no new work: the registry lookup is the same
  map the canon elements ride, the patch answers are the widgets' own
  bools, and the published corpus prices the view patch routes as
  shipped.
- The perf corpus stands at 62: the elements ride the published view
  routes (the M32 demonstration shows no new frame shape), the
  coordinated rule's no-growth arm (no new criterion case, no baseline
  move, the m5 bijection green as pinned).
- The m1 corpus does not grow: the demonstration shows no new visual
  arrangement (the elements paint the widgets' own shapes), and the
  m1 suite's 37-scenario pin stands.
- No new dependency edges: the prop variants are `alacer-view`'s (the
  foundation layer, no widget types), the element registrations are
  `alacer-widgets`' (the layer that owns the widget types — the DAG's
  own shape), and the demonstration is the facade's.

## Kill criteria (re-open rules)

- A view app needs the bridge wired by the DSL (a `bind:` prop naming
  the signal, the arena lookup automated) → grow the mount beside
  `PropValue::NumberBinding`, priced by a real view app — never a
  second binding channel.
- A widget needs a value-carrying prop the two types cannot express
  (a handler with a richer payload, a multi-value widget) → grow the
  vocabulary beside `PropValue`, priced by a real widget — never a
  stringly-typed prop.
- A consumer needs the element set to differ from the widget set (an
  extended registry with app-defined value widgets) → the registry's
  own `register` door already answers; grow the docs, never a second
  registry.
