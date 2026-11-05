# ADR-0001: Workspace discipline — DAG, pedantic clippy, zero unsafe

- **Status:** Accepted (2026-11-05)
- **Extends:** the architecture's ADR-9

## Decision

- Ten-crate workspace; layer order enforced by `scripts/check_dag.py` in CI
  (allowed-edge whitelist per crate).
- Workspace lints: `clippy::pedantic = deny`, `missing_docs = deny`,
  `unsafe_code = forbid`. **No `#[allow]` anywhere in the codebase.** The
  single float→int cast seam (`alacer-core/src/num.rs`) carries `#[expect]`
  attributes with reasons, so stale expectations fail the build.
- Widgets depends on core + layout + text for M0 (it needs measure specs and
  paint-time text layout); M1 tightens this to core-only re-exports.

## Consequences

- Any new cast must either go through `num.rs` or earn a documented seam.
- The DAG check fails CI on a new upward edge before review can merge it.
