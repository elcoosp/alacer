# ADR-0000: M0 scope — the skeleton milestone

- **Status:** Accepted (2026-11-05)
- **Supersedes:** the audit plan's M0 bullet (architecture §18)

## Decision

M0 delivers exactly: winit + wgpu + reactive_graph + taffy + parley +
accesskit on one window; the counter and the scrolling list render; masks
flow end-to-end; **one write batch ⇒ at most one layout+paint cycle**
(ADR-6 kill-switch), test-enforced. Everything else (rsx! proc-macro,
hot-patch suite, damage regions, IME) is explicitly *not* M0.

## Consequences

- Ten crates, not twelve: `alacer-hot` and `alacer-devtools` wait for M4.
- The view crate ships the reconciliation algorithm without `rsx!` — the
  proc-macro lands with M1 on top of it.
