# ADR-0137: M133 scope — the meter tier: the gauge whose reading is a measurement

- **Status:** Accepted (2026-11-19)
- **Implements:** the word every gauge vocabulary speaks as "this is
  how much there is" — and the tree never said. Through M132 the
  snapshot said the search field (ADR-0136), the named pane
  (ADR-0135), the instant on/off control (ADR-0134), the tool strip
  (ADR-0133), the bounded number field (ADR-0132) — and the scalar
  *measurement* had no word: an app wiring a disk-usage gauge, a
  battery reading, a capacity headroom reached for the progress bar
  wearing the wrong word (a progress bar's value is *task progress* —
  how far some task has come toward done; a meter's value is a
  *measurement* on a scale the app declares — how much of something
  there is, ARIA's `role="meter"` beside its `role="progressbar"`),
  or for a bare label carrying no value words at all. AccessKit's
  `Meter` role is its own fact — the word the AT's gauge conventions
  key off. [`Role::Meter`](crates/alacer-core/src/widget.rs) carries
  the gauge word — the value tier's own vocabulary, the M3 `value()`
  spoken word, the M97 range word, the M99 numeric word beside it
  (never a second value language), the M21 progress rule's own shape
  at the meter tier: a read-only report, the write vocabulary the
  app's.
- **Companion:** ADR-0027 (the M21 progress tier — the read-only
  report shape the meter rides, the idle-frame property at the app
  seam), ADR-0026 (the M20 value tier — the funnel the value words
  ride), ADR-0101/ADR-0103 (the M97/M99 range and numeric words — the
  vocabulary the gauge speaks), ADR-0136 (the M132 search-field tier —
  the one-word shape the tier follows), ADR-5 (the required-name rule
  the gauge joins), ADR-2 (the one-currency discipline the write
  rides), ADR-9 (the DAG the change never touches), architecture
  §7.2 pass 9 (the snapshot the word rides), §17 (the register), §18
  (the M133 roadmap row)

## Decision

- **One word, the gauge's own.**
  [`Role::Meter`](crates/alacer-core/src/widget.rs) carries the
  "reads as a gauge" fact — AccessKit's own `Meter` role spells it,
  ARIA's `role="meter"` beside its `role="progressbar"` names the
  difference. The gauge joins the required-name set (an unnamed meter
  reads as a bare number — the ADR-5 rule the named roles carry) and
  never the focusable set (a reading is not a control: it holds no
  focus and answers no activation — the M126 heading's own rule at
  the value tier's read half). The role is a *tree* word, never a
  paint flavor: the AT announces "gauge" beside the name, the value
  conventions key off the word.
- **The reading rides the value tier's own words.** No new value
  language: the M3 `value()` contract carries the spoken reading, the
  M97 `value_range` word carries the scale the app declared, the M99
  `numeric_value` word carries the number behind the spoken one — the
  sink maps the role one branch beside the progress indicator's, and
  the M97/M99 properties it already sets generically carry the
  reading (no second branch, no patch-tier growth).
- **The word prices the word and the reading, nothing else.** The
  producer ships without a seam change:
  [`Meter`](crates/alacer-widgets/src/meter.rs) — the read-only
  report shape (ADR-0027) at the gauge tier: the name, the declared
  scale (a degenerate one panics at construction — the invariants
  every read relies on), the reading from wherever the app writes it.
  Two doors land in the same `PAINT | A11Y` currency every report
  takes: a [`RenderBinding`] the flux bridge drives (one write batch,
  the caller's masks) and `set_value` for the mutate seam — the
  answer the `bool` that says whether the reading moved, so a write
  that lands where the reading already is queues nothing (the M21
  idle-frame property, at the meter tier); a non-finite request is
  refused honestly (`false`, never munged). The AT tier is read-only
  by construction: a measurement has no write door, every action
  refused honestly (the M11 rule; the M40 `no_write_door` species
  streams it). The `meter` element joins the registry (create/patch
  beside the progress element's: `name`/`min`/`max`/`value`/`fill`,
  the `binding` prop the bridge owns, the patch reporting the move).
- **The word never munges the neighbours.** One role per node: a
  `Progress` keeps the progress tier's word, a `Slider` the value
  control's, a button the press's — the gauge's word is the gauge's
  own, and the honest absence rides beside it (a field that declared
  nothing announces no gauge). **Pre-registered re-open rule:** a
  consumer needing the gauge's scale to answer the AT's write tier
  (an editable meter) grows the write door beside the funnel, priced
  by a real control — never a slider wearing the gauge word.
- **The living documents re-agree at M133 and 0.17.0** (the README's
  status block, register line, and widget table; the catalog; the
  register; the roadmap; the m-suite pins moving M132 → M133 across
  the chain; the CHANGELOG). The corpus pin re-agrees at 144 — the
  tier grows no snapshot route (the gauge rides the sink's role
  branch and the value words M3/M97/M99 already carry), and its
  pricing door is the widgets crate's own bench
  ([`benches/meter.rs`](crates/alacer-widgets/benches/meter.rs),
  `meter/value_write_256` against `meter/idle_write_256`), outside
  the published corpus by ADR-0131's rule.

## Consequences

- **The published corpus stays at 144 — the pricing door is the
  widget's own.** The reading's cost is priced in
  [`crates/alacer-widgets/benches/meter.rs`](crates/alacer-widgets/benches/meter.rs):
  `meter/value_write_256` — the moving write (the reported value's
  write the reading takes) against `meter/idle_write_256`, the write
  that lands where the reading already is (the honest floor of the
  same write at the widget seam, the M21 idle path priced). The pair
  rides outside the published baseline: the coordinated rule grows
  the corpus at the next published cut, never at a widget bench's.
  The m-suite pins re-agree at 144, the m133 suite's pin documenting
  the same shape.
- **The AT names the gauge the toolkit always owed it.** A screen
  reader walking a mounted gauge announces the meter word with the
  scale and the number — `aria-valuemin`/`aria-valuemax`/
  `aria-valuenow` beside the name, the gauge conventions keying off
  the word. The gauge walls nothing: the neighbours keep the words
  they published, and the app's write doors answer exactly as they
  always did — the word names the measurement, it walls nothing.
- **Consumers get the whole vocabulary for free.** The
  `examples/meter.rs` beat is the shape: the slider's funnel routed
  to one signal, the bound gauge the bridge drives, the reading the
  app mirrors from its own state — the neighbours untouched.
