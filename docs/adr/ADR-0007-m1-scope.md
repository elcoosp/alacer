# ADR-0007: M1 scope — the view DSL, the pass contract, and the snapshot corpus

- **Status:** Accepted (2026-11-05)
- **Supersedes:** the roadmap's M1 bullet (architecture §18) for the M0 codebase
- **Companion:** ADR-0000 (M0 scope)

## Decision

M1 delivers exactly four things on top of M0, matching the §18 exit criteria
("30 blessed snapshots green; a11y queries pass" + the pass-overhead kill
criterion):

1. **The ten-pass contract, as code (§7.2).** `PassKind` names all ten passes;
   `tick` drives the eight tick passes in contract order (mutate →
   update_focusable → update_focus → style → layout → compose → paint →
   accessibility) behind a per-pass tracing span (`alacer::pass`), with a
   ledger (`FrameReport::passes`) recording ran/skipped + wall time. Pass 1
   (event) gains the full capture→bubble route (§10.1); passes 2–3 implement
   focus candidate recomputation and focus resolution over the `FOCUS`
   currency; pass 5 is a real reason-flag recalc — style diffs against a
   per-node cache invalidate layout in the same frame, no lingering masks.
   The **kill criterion is a test**: on the 10k-node churn workload (1%
   re-styled per frame through mutate→style), pass-system bookkeeping must
   stay under 15% of frame time (median of 40 frames), with a criterion bench
   tracking the frame cost (`benches/pass_overhead.rs`).

2. **The `rsx!` proc-macro lands (ADR-0000's deferral).** New leaf crate
   `alacer-macro` (proc-macro; `syn`/`quote`/`proc-macro2` only — a pure DAG
   leaf: generated code references `::alacer_view` paths at the call site,
   the dioxus-rsx pattern). M1 grammar: elements, props (with `{expr}`
   interpolation in string literals — evaluated per view build, the Dioxus
   model), `key:` (reconciliation identity), `for pat in iter { element }`,
   `if cond { element } else { element }` (both branches mandatory —
   conditional absence waits for scoped re-renders), and string children
   (which are the element's `text` content, §5.2's footer shape). Known
   limitation, documented: an `if` condition that ends in a bare identifier
   must be parenthesized when the parser cannot distinguish it from struct
   syntax (`if (show) { … }`).

3. **The view layer (§6.1, §6.2).** `alacer-view` gains `ViewNode`/`PropValue`
   (the ephemeral view tree), `WidgetRegistry` (element name → typed
   create/patch closures; one downcast per entry via the new required
   `Widget::as_any_mut`), and `ViewRoot` (the reconciler: same element →
   patch in place; different element → subtree replace; all-keyed siblings →
   the peel/map/LIS pipeline with `apply(old, ops) == new` debug-asserted per
   update; mixed siblings → positional pairing). Retained widgets are patched
   in place, never rebuilt; the M0 widget set registers itself as
   `alacer_widgets::default_registry` (the concrete element set lives where
   the widget types live). The §6.4 equivalence gate is mirrored for the
   reconciler: *incremental updates ≡ fresh mounts* on random view churn.

4. **The snapshot corpus + a11y query API (ADR-10).** The blessed corpus grows
   from 2 to 32 committed PNGs (30 new M1 scenarios across widgets, states,
   scroll offsets, themes, and rsx patterns), each run through the real
   pipeline + software raster with the unnamed-input walker active. The a11y
   query API gains the plural builder (`query(role).named(..) →
   exists/count/all/first/bounds`), `press(role, name)` (the *actionable*
   half of §11.5 — dispatches the AccessKit default action through the id
   hash), and `TreeSnapshot::{find_all, contains, by_id, walk}` in core.

## Consequences

- **DAG tightening lands as planned (ADR-9):** widgets drop `alacer-layout`
  and `alacer-text`; text reaches them through `alacer-core::text_seam` (a
  thread-local `TextSeam` installed at assembly sites: facade, platform,
  harness). Widgets are core-only — 11 crates, 31 edges, DAG-checked.
- **The dependency inversion is explicit:** `PropValue::Binding` lets an
  `rsx!` label bind a `RenderBinding<String>` — the flux bridge drives its
  text (§6.1: views re-run, bindings re-render), no view re-run needed for
  the M1 counter canon.
- **Patch accounting is prop-driven:** the reconciler snapshots props per
  mount and reports `patched` only when a *visual* prop value changed;
  handler re-hangs are silent (handlers compare by identity).
- **Positional re-sync is part of the contract:** subtree replaces append to
  the arena, so both the keyed and positional child paths end with
  `reorder_children(parent, logical_order)` — the arena's children always
  equal the view's declared order.
- `alacer-hot` and `alacer-devtools` still wait for M4 (ten → eleven crates;
  twelve at M4).
