# ADR-0012: M6 scope — the benchmark canon, the shared workloads, and record/replay

- **Status:** Accepted (2026-11-07)
- **Implements:** the remaining items of the v1 definition of done
  (architecture §1): "a widget set covering the **counter/todo/data-table
  canon**"; the shared workloads W2/W3/W5 of §14.2 as measured, gated
  benches; and §13.4's record/replay — bug reports that are also regression
  tests
- **Companion:** ADR-0005 / ADR-0011 (testing productization, the perf-
  contract regime the new benches join), ADR-0008 (the layout discipline W2
  exercises), ADR-6 (the scheduling contract W5 re-measures at graph scale)

## Decision

M0–M5 proved the pipeline, the contract, the DX, and the outward story.
What remained of the v1 definition of done was the *canon*: the framework
is benchmarked and documented against counter / todo / data-table, and only
the counter existed. M6 completes the canon, wires the remaining shared
workloads into the M5 contract regime, and gives bug reports a file format.

### 1. The widget canon: checkbox and data table

The todo and data-table apps need two widget families the M0 set does not
carry:

- **`Checkbox`** — a toggle control: role `CheckBox`, click toggles, the
  Space key toggles while focused, the AccessKit default action toggles,
  and the checked state travels in the a11y tree. The neutral vocabulary
  grows a `ToggleState` (`Off`/`On`/`Mixed`) carried on the snapshot node;
  `alacer-a11y` maps it onto AccessKit's `Toggled` property. Keyboard
  toggling adds `Key::Space` to the key vocabulary (the platform seam maps
  winit's `NamedKey::Space`).
- **`Table` / `TableRow` / `TableCell` / `ColumnHeader`** — the data-table
  canon as retained composition: a `Table` is a column container carrying
  its grid geometry (rows × columns, kept by the widget's row-count API);
  `TableRow` is a row container; `TableCell` and `ColumnHeader` are named
  text cells. The neutral vocabulary grows a `TableSlot` (`Table{rows,
  columns}` / `Header{column}` / `Cell{row, column}`); `alacer-a11y` maps
  it onto AccessKit's table attributes (`table_row_count`,
  `table_column_count`, `table_column_index`, `table_cell_position`,
  `table_row_index`) — positions, not just roles, so a real screen reader
  can navigate the grid.

Both families obey the standing contracts unchanged: required role + label
(ADR-5 — the CI walker now fails unnamed checkboxes too), masks requested
through `EventCtx` only (A1 — a toggle requests `PAINT | A11Y`, never a
direct a11y write), and `Key::Space`/`DefaultPress` behavior test-enforced.

The default element registry grows the matching view elements (`checkbox`,
`text_input`, `table`, `table_row`, `table_cell`, `column_header`), and
`TextInput` gains `on_submit` — the Enter-to-commit seam the todo canon
needs (the architecture's own §5 snippet writes `input { on_submit: … }`).

### 2. The canon apps, shared between example and exit suite

`alacer-widgets::canon` hosts the two canonical apps as builder functions
over a pipeline: `todo_app` (the §5 shape — input + rows + a memo-driven
"items left" footer, rows rebuilt through the keyed view reconciler) and
`data_table_app` (headers + rows through the new table widgets). The
examples (`todo.rs`, `data_table.rs`) assemble them behind a window; the
M6 exit suite drives the *same builders* headlessly through the a11y
queries. One canon, one implementation, two consumers — the example is the
documentation, the exit suite is the regression net.

Reactive structure (rows appearing and disappearing) rides the view
reconciler: the canon builders return a `ViewRoot` and a re-render entry
point, and the M6 contract is the same one-frame rule as ADR-6 — one write
batch ⇒ one scoped re-render ⇒ at most one layout+paint cycle.

### 3. W2, W3, W5: the remaining shared workloads, measured and gated

The M5 regime (contracts.json + published baseline + dashboard) absorbs
the three workloads §14.2 defines but M5 did not measure:

- **W2 — text-dense relayout** (`alacer-layout/benches/w2_text.rs`): 1k
  text leaves under a ±5% width jitter, incremental relayout through the
  persistent engine, plus the full-relayout denominator. The gate is
  *stability*, not a ratio: jitter must not oscillate the wrap (§8.3's
  integer wrap rounding — adopted in this milestone: the layout host
  rounds text wrap widths to whole device pixels, so sub-pixel container
  jitter cannot flip a line break back and forth). The no-oscillation
  property is test-enforced (`alacer-layout/tests/w2_gate.rs`); the bench
  cases join the regression-gate corpus.
- **W3 — animation loop** (`alacer-testing/benches/w3_animation.rs`): 1k
  nodes, per-frame translation (scroll offset advance) + paint churn
  (opacity/color fade on 10% of leaves) through the *real* ten-pass
  pipeline and the software rasterizer with damage scissoring — the bench
  lives in the harness crate because that is where the whole pipeline is
  assembled (the DAG's harness layer drives every surface). Same-host
  regression gate; the 120 fps @ 1k nodes budget stays a reference-rig
  number, published as data, not promised from CI silicon.
- **W5 — signal-graph churn** (`alacer-core/benches/w5_signal_graph.rs`):
  10k signals, 1k memos, batched writes → one effect pass → mask drain.
  The gate is the scheduling contract at graph scale: one write batch ⇒ at
  most one rerun per batch scope, and memos never glitch (test-enforced in
  the M6 suite: every memo reads consistent values mid-batch).

Each new bench case gets its contracts.json regression gate; the corpus
grows from 38 to 51; the M5 suite's corpus pin and the dashboard are
regenerated in the same change (the coordinated three-edit rule of
ADR-0011, extended by one pin).

### 4. Record/replay: bug reports that are also regression tests (§13.4)

`alacer-testing` gains the `.alc` recording format (versioned JSON):
a header (schema, viewport, created) plus ordered steps — semantic steps
(`press` by role + name, key presses, typed text, wheel) rather than raw
coordinates, so a recording survives layout drift that a pixel click would
not. `Recording` builds from harness calls, saves and loads `.alc` files,
and replays deterministically through the real harness methods (every step
settles, the A7 idle contract unchanged).

The M6 suite replays a committed recording of the todo canon
(`crates/alacer-testing/recordings/todo.alc`) on a fresh harness and
asserts the final a11y state — the recorded interaction *is* the
regression test. Round-trip determinism (record → save → load → identical
steps) is test-enforced.

### 5. The executable exit suite

`alacer-testing/tests/m6_exit_criteria.rs` pins the milestone:

- the checkbox contract: click, Space, and the AccessKit default action
  all toggle; `Toggled` reaches the snapshot and the AccessKit update; the
  CI walker passes on a checkbox-bearing tree;
- the data-table contract: the grid is reachable and named through the
  a11y queries (table → rows → cells), and the AccessKit update carries
  table roles *with positions*;
- the todo canon end-to-end: type + submit adds a row, the a11y press
  toggles it, the footer memo reads the right count, delete removes the
  row — all through the shared builder;
- the data-table canon end-to-end through its builder;
- record/replay: round-trip determinism, the committed todo recording
  replays green, and a recording of a fresh session matches the committed
  one's shape;
- the workload corpus: the contracts ↔ baseline bijection at 51, the W2
  no-oscillation gate, the W5 one-rerun gate, and the dashboard publishing
  every new gate.

## Consequences

- The workspace ships as **0.6.0**; the CHANGELOG documents the release.
- The benchmark corpus spans **six** crates (core, layout, render, text,
  view, testing) — the M5 language "five benching crates" updates with the
  corpus pin.
- `alacer-testing` gains `serde`/`serde_json` runtime dependencies (the
  `.alc` format) and a `[[bench]]` target (W3), with `criterion` as a
  dev-dependency.
- The neutral a11y vocabulary (`TreeNode`) gains two optional fields;
  `Widget` gains two defaulted trait methods (`toggled`, `table_slot`) —
  no existing widget breaks.
- The counter/todo/data-table canon of §1's definition of done is
  complete; the remaining v1 items (cross-framework beat on shared
  hardware, the reproducible harness rig) are portfolio-level claims that
  need the declared reference rig, not new code — the M6 benches make them
  runnable anywhere.
- Record/replay steps are semantic by design: coordinate-level clicks
  remain available (the harness keeps `click`), but the canonical format
  records intent, so recordings outlive cosmetic layout changes.
