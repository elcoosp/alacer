# ADR-0022: M16 scope — the consumable contract (the consumer surface closes the M15 cut)

- **Status:** Accepted (2026-11-09)
- **Implements:** the consumer surface the editing series (M11–M15) grew
  faster than it could be consumed: the 0.12.0 release cut ADR-0021's
  consequences promised, the error boundary a consumer meets before any
  widget, the workspace's self-descriptions, and the newest vocabulary's
  missing demonstration
- **Companion:** ADR-0021 (the journal this demonstrates), ADR-0020 (the
  clipboard seam whose escape semantics the error messages must not
  contradict), ADR-9 (the dependency discipline that keeps miette out),
  architecture §13.4 (the DX contract), §18 (the M16 roadmap row)

## Decision

M15 completed the editing contract; its ledger (announcements/live
regions, the pointer-capture seam, `supports_*`, a `Slider`) names
feature work that each needs a full corpus cycle — a bench case, a
contract, a baseline re-published from the collecting host. M16 is the
cut that work rides on: the consumable surface, finished in the same
"one contract widens" shape, with the contract this time being the
consumer's:

- **The 0.12.0 cut completes.** The version-bump commit set
  `[workspace.package]` to 0.12.0 and stopped: the twelve internal
  dependency pins in `[workspace.dependencies]` stayed at 0.11.0 and
  `Cargo.lock` still recorded the members at 0.11.0 — a workspace whose
  pins lag its own version cannot publish the coherent 0.12.0 family
  ADR-0021 promised (a facade cut from it would pair with the *old*
  published cores). The pins, the lockfile, and the README's version
  badge join `[workspace.package]` at 0.12.0.
- **The error boundary carries its chain.** `PlatformError` and
  `BackendError` implemented `Error` with an empty body: the
  `EventLoopError`, `OsError`, `RequestDeviceError`, and `wgpu::Error`
  inside them were formatted into the Display and dropped — a consumer
  whose window refuses to start meets the boundary error first and gets
  a dead end. `Error::source` now walks the real chain, and the Display
  messages name the escape hatch they know about (`ALACER_SOFTWARE=1`
  for the softbuffer path when no adapter matches) — help text at the
  seam, hand-rolled.
- **The workspace's self-descriptions catch up with the shipped
  vocabulary.** The widgets crate's manifest description and module
  rustdoc still said "for M0: label, button, column, spacer, and the
  scrolling list" while checkbox, panel, table, text input, and both M6
  canons ship in the same crate; `docs/examples.md`'s PNG count and the
  README's decision-record count follow what the tree actually carries.
- **The history vocabulary gets its consumer demonstration.**
  `examples/undo_redo.rs` drives `EditHistory` directly — record under
  the pure-append rule, undo/redo with the selection each step returns,
  the app-write reset, the honest refusal on an exhausted journal,
  `undo_depth`/`can_redo` reads surfaced in the window — outside the
  nine-contract `text_input` mega-demo the only consumer so far, and a
  blessed snapshot pins the demonstration (the examples catalog's own
  rule: an example arrives with its scenario).

### 1. Why this is the milestone

ADR-0021 §3 wrote the journal's introspection down as "the Causality
surface rides a later seam," and the recorded unblocked seams are each
a milestone of their own shape — announcements need the live-region
vocabulary, the pointer-capture seam needs a gesture state machine, a
`Slider` needs the numeric value tier. None is unblocked by a deferral
the previous ADR wrote down; all of them, however, ship into a surface
that M15's own commit trail left behind: the pending cut is half-done
(the version bump touched one manifest line), the boundary error drops
its chain, the widgets crate describes itself as the M0 set, and the
newest public vocabulary has no runnable demonstration. The cheap cut
is the consumable one: publish-able pins, an error that names its cause
and its escape hatch, docs that match the API, and a journal demo a
consumer can read before they read ADR-0021.

### 2. The seams that did not move

- **No new bench case; the corpus stays 53.** The journal's hot path is
  already contracted (`editing/route_100_undo_redo_1k`); the example
  consumes the same record/undo/redo arithmetic the unit tests pin —
  demonstrating it needs no second measurement.
- **The DAG is untouched in shape.** No new dependency edges: the
  example consumes the facade; the error work changes trait bodies, not
  signatures; the only new dependency in the graph is `tracing` as the
  facade's *dev*-dependency (the workspace already adopts it at core),
  so nothing published moves.
- **The editing contract's rules are untouched.** The journal, the edit
  funnel, and the chords keep their pinned behavior; the example is a
  consumer, not a second definition — it calls `EditHistory`, never
  re-implements a rule.

### 3. What this does not do

- **No crates.io publish.** The architecture's `UNVERIFIED` marker on
  the crate family stays until the first publish actually happens —
  this milestone makes the tree publish-*able*, not published.
- **No miette adoption.** The workspace's dependency discipline (A2,
  ADR-9) ratifies every adopted core in an ADR; a diagnostic-coloring
  core is its own decision. The help text is hand-rolled `Display` +
  `Error::source`, the same discipline `PlatformError` already used.
- **No new lint suppressions.** The error work adds zero
  `#[allow]`/`#[expect]` attributes; the reasoned `#[expect]`s at the
  documented cast seams (ADR-9, `num.rs`) are untouched.
- **No live-region, pointer-capture, or `Slider` vocabulary.** The
  ledger's candidates ride M17+ with their own corpus cycles.

## Consequences

- The workspace publishes 0.12.0 as one coherent family: the facade,
  the cores it rides, and the lockfile agree.
- A failed window start names its cause, its underlying error, and the
  escape hatch it knows about; `Error::source` walks the real chain.
- `cargo doc`, the crate descriptions, and the README describe the
  shipped vocabulary, not the M0 one.
- The journal has a runnable consumer demonstration, blessed with the
  rest of the corpus (32 → 33 PNGs), and the observability idiom
  (tracing at the documented targets) rides the example.
- The next milestone starts from a consumable surface: the ledger's
  feature work lands in a workspace whose cut, docs, errors, and
  demonstrations agree with its own API.
