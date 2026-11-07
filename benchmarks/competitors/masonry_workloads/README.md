# masonry_workloads — the Masonry side of the beat (M7, ADR-0013)

Idiomatic Masonry implementations of the shared beat workloads
(architecture §14.2 W1/W2/W3), driven headlessly through the public
`RenderRoot` API and measured with criterion by
`scripts/run_comparison.py`.

## The idiomatic mapping (the no-strawman table)

| Workload | Masonry APIs used | The production answer deliberately not taken |
|---|---|---|
| W1 — 10k leaves, 10%/frame churn | `Flex`/`SizedBox` trees, `RenderRoot::edit_base_layer` + `WidgetMut` discipline, `redraw()` | `Portal` / `virtual_scroll` — Masonry's real answer for long lists (scroll-windowed virtualization and widget reuse), which sidesteps the tree-wide mechanism under test |
| W2 — 1k paragraphs, ±5% width jitter | `Label` + the `LineBreaking::WordWrap` property, parley re-breaking on changed constraints | `Portal` (same reason); parley's shaping cache itself is fully in play |
| W3 — 1k animated nodes | `WidgetMut::set_transform` (compose-level translation), `redraw()` | none — compose-level transforms are Masonry's translation-only path; that is the mechanism |

## Scope (ADR-0013 §3 — the asymmetry rule)

One measured frame = the full CPU pass pipeline through
`RenderRoot::redraw()` — update, layout, compose, paint-scene build, and
the accessibility tree update. Vello's rasterization is excluded: it needs
a wgpu device, which the declared rig (headless, no GPU) does not carry.
The exclusion is labeled in `results.json` and is conservative — Masonry's
row measures *less* than a full frame, so a beat won against it is won a
fortiori.

## Provenance

- Masonry is pinned in `Cargo.toml` (`0.4.x`); `Cargo.lock` records the
  exact build. A version bump is a reviewed diff (lockfile + the published
  results + this README).
- The project is standalone (empty `[workspace]`): competitor dependencies
  never enter the Alacer workspace graph, and this crate's silicon is
  measured on the same machine as everything else in `results.json`.
- The selfcheck tests in `src/lib.rs` pin what a bench frame claims: the
  full 10k tree, the re-wrapping labels, the 1k transforms, and the
  accessibility pass running on every measured frame.
