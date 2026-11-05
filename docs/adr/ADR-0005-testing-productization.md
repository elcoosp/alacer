# ADR-0005: The headless harness is the real pipeline (A4/ADR-10)

- **Status:** Accepted (2026-11-05)

## Decision

`alacer-testing` composes the production `PassPipeline` with the software
rasterizer and the thread-local text context — no mocks anywhere. Snapshots
bless to committed PNGs (`ALACER_BLESS=1`) and compare with a per-pixel
channel tolerance plus a 2% pixel-fraction threshold (per-OS font drift must
not rot the corpus).

The a11y query API (`expect_a11y(role, name)`) is the *only* official query
API, and the unnamed-input walker (`assert_all_named`) fails builds — §11.1
and §11.5 as code.

## Consequences

- CI is GPU-less by default and still exercises everything except wgpu
  presentation (compile-verified only).
- Blessed snapshots are committed; regenerating is a deliberate, reviewable
  diff.
