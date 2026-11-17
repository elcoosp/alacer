# ADR-0096 — M92 scope: the `#[derive(Widget)]` form

Milestone: **M92** (this cut). Parent contract: architecture §19 Q5,
whose own words pre-registered this cut as the standing option:
"Required a11y methods won't poison ergonomics … answered by the
shipped shape: required trait methods since M0 (no derive yet); gather
feedback at 0.1, the `#[derive(Widget)]` form stays the option."

## The ask

The trait's required pair has stood since M0 on purpose: the a11y
contract starts at `role()`, the name at `label()`, the patching hatch
at `as_any_mut()` — no derive could be trusted with the AT's first
questions until the contract had proven itself (the Q5 bet). The 0.17.0
feedback the option asked for is in: hand-written widgets write the
same three lines on every type — the role, the name, the `{ self }`
upcast — and the three lines are the exact boilerplate the option
existed to retire. The derive grows into `alacer-macro` beside the
`rsx!` compiler it already hosts: the attribute speaks the vocabulary
the whole framework prints, and the expansion is the required pair and
*nothing else* — every other method rides the trait's own default, so a
derived widget is exactly as honest as the hand-written one beside it.
No derive existed until the defaults could be trusted; the defaults
have ninety-one milestones of tests pinning them, and now the derive
shortens the hand, never the contract.

## The shape

- **The attribute grammar** (`#[widget(...)]`): `role = "…"` — required,
  the vocabulary `Role::name`'s own (`button`, `container`, `text_input`,
  `scroll_area`, … the words the snapshot's role field and the CI
  walker's diagnostics print). `label = "…"` or `label = from_children` —
  required too: the string form rides [`LabelSource::text`], the keyword
  rides [`LabelSource::FromChildren`], the container aggregation contract
  Column and Table declare themselves. Silence is never the derive's
  default — an unnamed widget is the a11y walker's offender (§11.1), so
  the derive refuses to mint one.
- **The expansion** — the required pair and nothing else: `role()` from
  the vocabulary word, `label()` from the name form, `as_any_mut()` in
  its documented `{ self }` shape. The `live_region` word is refused with
  its reason (the pipeline builds the region itself, ADR-0024 — a widget
  claiming the reserved role would fight the pipeline's own node);
  unknown words are refused with the vocabulary named; duplicates,
  non-strings, enums, and generics are refused with the limitation named
  (the ADR-0025 diagnostic species — the error names what stood where,
  and the fix rides the message).
- **The wire-up** (the serde pattern, one name two namespaces): the
  derive is re-exported from the facade root and the prelude beside the
  trait it implements — `use alacer::prelude::*` imports both, and
  `#[derive(Widget)]` resolves in the macro namespace while the trait
  resolves in the type namespace, exactly as `serde::Serialize` has
  always shipped. Generated code references `::alacer_core` paths at
  the call site (the crate's own dioxus-rsx pattern: the macro is a
  pure DAG leaf, no internal dependency, so the derive cannot drift
  from the trait it implements).
- **The zero-overhead budget** (the coordinated rule's bench half): the
  a11y snapshot pass over a 1,000-node derived arena against the same
  1,000-node hand-written arena — the same roles, the same names, the
  same trees — priced as a *ratio* budget (`derive_zero_overhead`, ≤
  1.10×), the host-independent contract class the M5 design shipped.
  The m-suite pins re-agree at 101.

## Exit criteria

- The derived pair lands in the a11y snapshot: a derived container and
  a derived button mount through the real pipeline, lay out like any
  other widget, and answer `find(ROLE).named(...)` by the attribute's
  own words (the m92 exit suite; `assert_all_named` passes — the derive
  mints no unnamed widgets).
- The two name forms answer their own contracts: the string is
  [`LabelSource::text`], the keyword is [`LabelSource::FromChildren`] —
  no third form exists.
- The defaults ride the trait: availability, value, toggle, IME, and
  the focus derivation reading the role — asserted directly on the
  derived types.
- The `Any` hatch reaches the derived type: the arena mutate guard
  downcasts through the generated `{ self }` upcast (the §6.1 patching
  discipline).
- The AT's ask meets the honest refusal: the derived button is
  reachable and named, and the press is refused by the trait's own
  default answer — never munged, never dropped (the M11 rule).
- The corpus grows 97 → 101 by the coordinated rule (M92's pair beside
  M93's, one cut): `derive/snapshot_1k_derived` and
  `derive/snapshot_1k_handwritten`, the ratio budget and the same-host
  gates joining the contract set, the m-suite pins re-agreeing at 101.
- `examples/derive_widget.rs` drives the beats from the consumer side.

## Re-open rules (pre-registered)

- a consumer needs the *behavioral* overrides generated too — a
  `paint = path` or `value = expr` entry — → the attribute grows the
  entry beside the pair, the expansion delegating to the named path;
  never a second derive.
- a consumer needs generics — a derived wrapper over `T` — → the
  expansion grows the impl generics the attribute cannot express yet,
  the bounds the trait's own contract names.
- a consumer needs the *view DSL* to mint derived widgets —
  `rsx! { my_widget { ... } }` over a derived type — → the registry
  grows the registration the elements already ride, the derive
  untouched.
