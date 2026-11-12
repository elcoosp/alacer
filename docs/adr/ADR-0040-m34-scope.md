# ADR-0040: M34 scope — the backend features are true: each backend compiles with its feature

- **Status:** Accepted (2026-11-12)
- **Implements:** the capability the feature names pre-registered —
  `backend-wgpu` / `backend-softbuffer` forward to optional
  dependencies (`crates/alacer-platform/Cargo.toml`), promising
  single-backend builds the tree never delivered: the platform's
  backend modules, enum, and construction paths compiled
  unconditionally, so `--no-default-features` failed with 151 errors
  and every single-backend combination failed at `cargo check` with
  unresolved crates — a latent breakage CI's `--all-features` runs
  never exercised — cashed as a real capability: "a consumer needs a
  software-only (or GPU-only, or pipeline-only) build → the features
  own their backends, the missing path refuses honestly at startup,
  the matrix is CI-enforced — never a cosmetic toggle"
- **Companion:** ADR-1 (the platform owns the adapter dependencies the
  features gate), ADR-2 (the wgpu primary the preference names),
  ADR-0024 (the softbuffer path the software feature owns), ADR-0022
  (the error-boundary discipline the refusal rides), ADR-9 (the DAG
  the change never touches), architecture §10.1 (the platform seam),
  §18 (the M34 roadmap row)

## Decision

- **Each present path compiles with its feature.** The platform's
  backend modules gate on the features that own them
  (`#[cfg(feature = "backend-softbuffer")] mod softbuffer;`,
  `#[cfg(feature = "backend-wgpu")] mod wgpu_backend;`, their re-exports
  beside them), the `Backend` enum's variants are the compiled
  backends, and the construction paths moved into per-backend builders
  (`build_wgpu_backend`, `build_softbuffer_backend`) that compile with
  their feature — the wgpu instance descriptor, the softbuffer
  framebuffer, and the glyph source (the present paths' shared
  argument) ride the same gate. The present ladder splits into the two
  backends' own functions; the resize path's arms ride the same rule.
  All four combinations now compile warning-free under the CI's
  `-D warnings`: both backends (the default, today's behavior
  unchanged), each alone, and none.
- **A missing path refuses honestly at startup.** With a backend
  compiled out, asking for it returns the typed refusal —
  `PlatformError::BackendUnavailable { requested }` (the preference's
  own name, `BackendPreference::name` the single source): the display
  names the gate that answers it ("no present backend compiled:
  `softbuffer` is gated off — enable backend-wgpu or
  backend-softbuffer"), and the chain ends at the build fact — nothing
  wrapped rides it (the M16 boundary's leaf rule at the build tier; a
  fact about the build, never a panic, never a munged fallback to a
  backend the consumer did not ask for). The facade's `run_window`
  propagates it unchanged.
- **CI keeps the matrix honest.** The features job exercises all four
  combinations: the no-backend build compiles and runs its refusal
  tests, each backend alone compiles, and the facade's software-only
  and pipeline-only builds compile — the capability the consumer
  configures through the facade's forwarding (default both, each
  alone, hot-reload independent). The main jobs' `--all-features`
  shape is unchanged: with both backends compiled the behavior is
  byte-for-byte today's (the cfg discipline compiles the same paths in
  the same order).
- **The living documents re-agree at M34 and 0.17.0.** The README's
  status block, the ADR count (forty-one), the backends bullet (the
  feature matrix a supported shape), the example table's row riding
  ADR-0039's cut; the register; the roadmap row; the m-suite pins
  moving M32 → M34 and 39 → 41 across the chain; and the CHANGELOG
  stages the milestone under `[Unreleased]`.

## Consequences

- The default build's behavior is unchanged: both backends compiled,
  the same construction order, the same present ladder, the same
  Windows smoke shape — the `#[cfg]` discipline compiles identical
  code when both features ride.
- `PlatformError::BackendUnavailable` and `BackendPreference::name`
  are new public vocabulary — pre-1.0, additive, the error boundary's
  documented chain walking to the honest leaf.
- Consumers referencing the gated re-exports (`WgpuRenderer`,
  `SoftbufferPresenter`) under a build without their feature see the
  item's absence — the feature's semantics, not a regression.
- No new dependency edges, no corpus growth: the platform's code
  motion is cfg discipline over paths that already existed, and the
  benches never compiled the gated-out shapes.
- The M16/M18 error-chain unit tests that exercise the gated variants
  gate with them (the no-backend build's 17-test suite carries the
  refusal's own pins), keeping every combination's suite green.

## Kill criteria (re-open rules)

- A platform needs a third present path (a headless pixel sink, a
  remote compositor) → grow the feature beside the two the platform
  already gates, priced by a real backend — never a fourth ungated
  module.
- A consumer needs the refusal to name an escape hatch the display
  cannot carry (a config-file pointer, an env-var override) → grow the
  variant beside `requested`, priced by a real reader — never a
  stringly-typed message.
- A single-backend build needs a backend the features cannot express
  (a runtime backend registry) → re-open ADR-2's primary rule with a
  real consumer — never a dynamic registry the enum's variants cannot
  name.
