# egui_workloads — the egui side of the beat (M7, ADR-0013)

Idiomatic egui implementations of the shared beat workloads
(architecture §14.2 W1/W2/W3), measured headlessly with criterion and
published by `scripts/run_comparison.py`.

## The idiomatic mapping (the no-strawman table)

| Workload | egui APIs used | The production answer deliberately not taken |
|---|---|---|
| W1 — 10k labels, 10%/frame churn | `Context::run_ui`, `Ui::label`, `RichText::color`, `Context::tessellate` | `ScrollArea::show_rows` row virtualization — egui's real answer for long lists, which sidesteps the tree-wide mechanism under test |
| W2 — 1k paragraphs, ±5% width jitter | `Ui::allocate_ui` (jittered width), `Label` wrapping, egui's galley cache | `ScrollArea` + row virtualization (same reason); the galley cache itself is fully in play |
| W3 — 1k animated nodes | `Ui::painter`, `Painter::rect_filled`, `Color32::gamma_multiply` | none — immediate mode repaints every node by design; that is the mechanism |

## Scope (ADR-0013 §3 — the asymmetry rule)

One measured frame = `run_ui` (widget walk + layout + text shaping through
the galley cache) + `tessellate` (geometry emission). Pixel rasterization is
excluded for every framework in the comparison whose harness omits it,
including egui's own row — and Alacer's W3 row *includes* a damaged software
raster, so the comparison is conservative against Alacer, never against the
competitors.

## Provenance

- egui is pinned in `Cargo.toml` (`0.37.x`); `Cargo.lock` records the exact
  build. A version bump is a reviewed diff (lockfile + the published
  results + this README).
- The project is standalone (empty `[workspace]`): competitor dependencies
  never enter the Alacer workspace graph, and this crate's silicon is
  measured on the same machine as everything else in `results.json`.
- The selfcheck tests in `src/lib.rs` pin what a bench frame claims: the
  full 10k walk, the re-wrapping paragraphs, the 1k paint ops.
