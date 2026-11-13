# ADR-0053: M47 scope — the keyboard's jumps are named facts

- **Status:** Accepted (2026-11-13)
- **Implements:** ADR-0051's second kill rule — "a consumer needs
  Home/End on the log → grow the action vocabulary with a named bound
  species beside `SetValue` — never a munged jump"
- **Companion:** ADR-0051 (the key route whose stamp the jump rides),
  ADR-0046/ADR-0050 (the record vocabulary the jump streams), ADR-0026
  (the value tier whose funnel the bounds share), ADR-0028 (the
  availability tier the keyboard's wall belongs to), architecture
  §13.3, §18 (the M47 roadmap row)

## Decision

- **The jump is its own named species, never a munged write.**
  `alacer-core/src/inspect.rs`: `ValueAction` grows
  `JumpToMin`/`JumpToMax` beside `SetValue` — the wire's `action`
  vocabulary growing the two `snake_case` tags (`jump_to_min`,
  `jump_to_max`) additively at version 1 (the documented field values'
  rule; clients that ignore unknown field values are unaffected). The
  M11 rule's own words stay the law: a jump munged into a `SetValue`
  would collapse the jump's story into a payload that never was.
- **The bounds' door notes its own fact.** `alacer-widgets/src/
  slider.rs`: Home/End route through `jump_note` — the same shared
  route the step tier rides — streaming the jump's species, the
  landing (the funnel's own arithmetic), and the ask beside it: the
  bound *is* the ask, the clamp-and-snap never moving it (M44's
  discipline, the ask living where the knowledge lives).
- **The stamp rides for free.** The M45 key route holds the focused
  control's guard for every routed key; the jump's record carries the
  focused control's AccessKit-stable id like the arrows' rides do —
  one funnel, four named facts (the steps' `Increment`/`Decrement`
  beside the jumps' species).
- **The no-op delivery answers like the landed write it is.** Home at
  the minimum already lands where the value is; the funnel answered
  (the M20 idle-frame property), and the log names it — one fact for
  what was one silence.
- **The corpus stands at 64; the living documents re-agree at the
  cut** (the README, the register, the protocol document's action
  vocabulary, the catalog, the CHANGELOG, the m-suite pins).

## Kill criteria (re-open rules)

- A consumer needs a bound jump the *pointer* asked for (a drag that
  slams an edge) → the M46 gesture record's phases already carry the
  gesture's own story; grow that vocabulary, never the AT tier's.
- A consumer needs *which* bound the value pinned when a write was
  refused at the range's edge → grow the refusal's payload beside the
  species (the ask's own wall), never a second action species.
