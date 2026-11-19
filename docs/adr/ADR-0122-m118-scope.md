# ADR-0122: M118 scope — the routing-outcome doors: the harness's keyboard and IME doors answer what the pipeline answered, as data

- **Status:** Accepted (2026-11-19)
- **Implements:** the one routing question a test could not ask
  directly. Through M117 the harness's keyboard and IME doors
  dispatched and settled — and dropped the verdict on the floor
  (`let _ = self.pipeline.dispatch_key(event)`): a test that wanted
  to know whether the field took the key, the read-only wall refused
  it, or the funnel fell through had to smuggle the answer out
  through effects — the value that moved, the trail that streamed,
  the focus that landed — the effects the doors already produce,
  priced per assertion. The pipeline always answered as data (the
  M27 rule at the dispatch seam):
  [`EventResponse`](crates/alacer-core/src/widget.rs) — `Handled`
  the widget (or the keymap or a framework fallback) consumed the
  key, `Ignored` it bubbled on — the return value the funnel already
  computed, discarded at the door. Cashed at the routing-outcome
  doors: "a test needs the routing the pipeline answered → the doors
  carry the funnel's own verdict to the test, the dispatch
  byte-for-byte the dispatch they always ran — the door prices the
  answer and nothing else"
- **Companion:** ADR-10 (the testing productization the doors
  sharpen — the harness is the product, and the doors' answers are
  its assertions' vocabulary), ADR-0033 (the M27 outcome-as-data
  rule the doors copy at the harness tier — the dispatch already
  answered as data; the doors stop discarding it), ADR-0041/ADR-0042
  (the keymap funnel the verdict rides — the consult's
  `dispatch_installed` verdict and the focus-cycling fallback are
  the `Handled` half of the answer, exactly as dispatched),
  ADR-0109 (the read-only wall the verdicts name — a refused edit is
  a consumed key: the field heard its own key, the wall answered it,
  and the verdict says so), ADR-9 (the DAG the change never
  touches), architecture §13 (the DX contract), §17 (the register),
  §18 (the M118 roadmap row)

## Decision

- **The doors answer the funnel's own verdict.**
  [`TestHarness::key`](crates/alacer-testing/src/lib.rs),
  [`TestHarness::press_key`](crates/alacer-testing/src/lib.rs) and
  [`TestHarness::ime`](crates/alacer-testing/src/lib.rs) return the
  `EventResponse` their dispatch computed — `Handled` the focused
  widget (or the keymap, or the Tab cycling, or the directional
  fallback) consumed the key, `Ignored` it bubbled on.
  [`TestHarness::type_text`](crates/alacer-testing/src/lib.rs)
  returns one verdict per dispatched character, in order — the field
  that took the characters, the wall that consumed its refusals
  (ADR-0109), and the widget that takes no text each name
  themselves. No new verdict vocabulary: the enum the pipeline has
  answered since M0 is the answer.
- **The doors price nothing but the answer.** The dispatch they rode
  is byte-for-byte the dispatch they always ran — the settle still
  follows, the `# Panics` contract stands — the verdict is the
  return value the funnel already computed, never a second route.
  The corpus keeps its count at the M117 cut: no case joins and no
  case leaves, the door prices no frame work.
- **The replay driver records steps, not verdicts.** The recording
  tier's contract is what-you-did-is-what-replays; the routing
  verdicts were not recorded facts, so the replay's door calls read
  unchanged (the returned verdicts discarded at the replay's match
  arms, the steps the only recorded truth).
- **The living documents re-agree at M118 and 0.17.0.** The ADR count
  (a hundred and twenty-three), the architecture register (ADR-0122
  scopes M118), the roadmap row (§18), the CHANGELOG's unreleased
  block, and the examples catalog (the `key_routing` row) all name
  the doors; the corpus pin holds at 137, and the m-suite corpus
  pins re-agree at 137.

## Consequences

- **The corpus kept at 137.** The doors price no frame work — the
  verdict is the return value the dispatch already computed — so no
  criterion case joins and no case leaves. The `m118` suite pins the
  corpus at the M117 cut's count (the pin-holds assertion), the only
  suite in the m-suite whose corpus assertion asserts *stability*
  rather than growth; the coordinated rule's next tier grows the
  corpus from 137.
- **The routing question is a direct, deterministic answer.** The
  assertion the harness's own m-suite reached for through effects —
  the value that moved, the wall that held — now reads off the door:
  the stray key `Ignored`, the cycling's Tab `Handled`, the typed
  run `Handled` per character, the button's rejections `Ignored` per
  character, the read-only wall `Handled` and empty. A regression
  that flips a widget's consumption of a key flips the verdict, not
  a side effect two asserts away.
- **Consumers get the whole vocabulary for free.** The
  `examples/key_routing.rs` beat is the shape: the stray key the
  empty funnel falls through, the Tab the cycling takes, the
  characters a focused field lands, the characters a plain button
  takes no interest in, the wall that consumes its refusals while
  landing nothing, and the IME route's verdict beside them — every
  door answering what the pipeline answered, as data.
