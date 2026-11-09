# The examples gallery

Every runnable example in the workspace, what each one demonstrates, and the
contract it exercises. All of them run the *real* pipeline — wgpu (or
softbuffer with `ALACER_SOFTWARE=1`), the ten-pass contract, the scene
journal, and AccessKit — never a mock (axiom A4). Headless CI builds every
example (`cargo build -p alacer --examples`) and tests the same code paths
through the software backend.

| Example | Demonstrates | Contract | Run |
|---|---|---|---|
| [`counter`](../crates/alacer/examples/counter.rs) | one signal, one bound label, one button | the M0 canon: masks flow end-to-end, one batch ⇒ one frame (ADR-6) | `cargo run -p alacer --example counter` |
| [`counter_view`](../crates/alacer/examples/counter_view.rs) | the same app written in `rsx!` | the view DSL + in-place reconciliation (M1) | `cargo run -p alacer --example counter_view` |
| [`todo`](../crates/alacer/examples/todo.rs) | the todo canon: input submits on Enter, checkboxes toggle, the footer memo counts what is left | the M6 canon through `alacer_widgets::canon::todo_app` — one write batch ⇒ one scoped re-render (ADR-6, §13.2's one-frame path) | `cargo run -p alacer --example todo` |
| [`data_table`](../crates/alacer/examples/data_table.rs) | the data-table canon: a named grid, an "Add row" seam, cells that renumber on removal | the M6 canon through `alacer_widgets::canon::data_table_app` — a *navigable* AccessKit table (counts + positions) | `cargo run -p alacer --example data_table` |
| [`gallery`](../crates/alacer/examples/gallery.rs) | the whole widget vocabulary in one window: labels, buttons, text input, panels, list, rows, spacers | the M5 tour: signals, bindings, theme, translation-only scrolling | `cargo run -p alacer --example gallery` |
| [`scrolling_list`](../crates/alacer/examples/scrolling_list.rs) | 24 panels in a clipped viewport | translation-only wheel updates — zero measure calls per scroll tick (§8.2) | `cargo run -p alacer --example scrolling_list` |
| [`input_storm`](../crates/alacer/examples/input_storm.rs) | the M8 queue driven from the consumer side: a custom `StormPanel` probe (a `Widget` with its required role and label) records every routed pointer event, *Read the log* mirrors the log into the window, *Reset the log* empties it — wave the pointer and the log grows at most one entry per frame | the M8 coalescing contract: raw events queue, the tick routes one batch, a storm costs one frame (ADR-0014, ADR-0023; the `input_storm_batching` budget; the deterministic fifty-move storm is pinned by the m1 corpus's `input_storm` scenario) | `cargo run -p alacer --example input_storm` |
| [`drag_capture`](../crates/alacer/examples/drag_capture.rs) | the M19 capture driven from the consumer side: a custom `DragPanel` probe records every routed pointer event — press the panel and drag *past its edge*, the log keeps counting while the gesture is anywhere in the window, and the release rides back to the panel; release, then wave without pressing, and only the moves inside the frame arrive | the M19 capture contract: a press implicitly holds its deepest accepting hit, the gesture's Move/Up/Leave stream rides the captured node's ancestor chain, the Up/Leave/death release it, the wheel stays hit-tested, and a frame with no capture routes exactly as before (ADR-0025; pinned by the m19 exit suite and the pipeline's unit tests) | `cargo run -p alacer --example drag_capture` |
| [`text_input`](../crates/alacer/examples/text_input.rs) | caret and selection editing (Shift+arrows, Ctrl+A, Alt/Ctrl+arrows word hops, word-chord deletion, double-click word selection, Ctrl/Cmd+C/X/V clipboard chords, Ctrl/Cmd+Z / Shift+Ctrl/Cmd+Z / Ctrl/Cmd+Y undo/redo), secure mode, Tab focus cycling | the M3 IME contract: UTF-16 end to end, inline preedit + commit, partial paint per keystroke; the M11 selection contract: the range the a11y tree carries and the AT value tier edits (ADR-0017); the M13 word contract: the seam's UAX #29 segmentation and the hop rule (ADR-0019); the M14 clipboard contract: the seam round trip, the single-line paste rule, the refused write that never deletes (ADR-0020); the M15 history contract: the journal in core, the coalescing and reset rules, the selection every step restores (ADR-0021) | `cargo run -p alacer --example text_input` |
| [`undo_redo`](../crates/alacer/examples/undo_redo.rs) | the M15 journal driven directly: *Type hi* appends under the pure-append coalescing rule, *Undo*/*Redo* replay the step with the selection it carries, *Replace all* is the app-write reset, an exhausted journal refuses with an idle frame — no `TextInput` riding along | the M16 demonstration: `EditHistory` as a consumer's vocabulary (`alacer_core::history`) — one edit funnel, one tracing span per route at `alacer::editing`, the introspection reads (`undo_depth`, `can_redo`) live in the window (ADR-0021, ADR-0022) | `cargo run -p alacer --example undo_redo` |
| [`announcements`](../crates/alacer/examples/announcements.rs) | the M18 announcement seam driven from the consumer side: *Save it* announces politely, *Delete it* interrupts assertively, *Speak nothing* refuses honestly — the status line mirrors what the reserved live region carries, read back from the app's own state | the M18 demonstration: the app's voice rides the a11y delta (`alacer_core::announce`) — the handler-reachable queue drains into the same region one frame later (the input-queue shape, ADR-0014), one tracing span per route at `alacer::announce`, and a screen reader speaks the `aria-live` pairing the sink emits (ADR-0024) | `cargo run -p alacer --example announcements` |
| [`slider`](../crates/alacer/examples/slider.rs) | the M20 value tier driven from the consumer side: one `Slider` over 0..=100 — drag the thumb, focus it and press the arrows (Home/End jump to the ends), or let a screen reader write the value; the status line mirrors what the a11y tree carries | the M20 demonstration: the value tier is a widget (ADR-0026) — every door (drag, arrows, the AT's `SetValue`/`Increment`/`Decrement`) rides one funnel that clamps to the range and snaps to the step inside the widget, the value rides the M3 `value()` contract, and a write that lands where the value already is queues no masks | `cargo run -p alacer --example slider` |
| [`progress`](../crates/alacer/examples/progress.rs) | the M21 reported value driven from the consumer side: one bound `Progress` the app writes through both doors — the bridge (the *Step the batch* button's one-batch signal write) and the mutate seam (the lease, the write, the masks on the guard, walked at startup) — with `tracing` spans at `alacer::progress`; the status line mirrors what the a11y tree carries | the M21 demonstration: the value tier's read half (ADR-0027) — a bounded scalar the *app* reports and the AT reads through the M3 `value()` contract, every AT action refused honestly (a reporting widget has no write door), and a write that lands where the value already is queues no masks — the idle-frame property at the app seam | `cargo run -p alacer --example progress` |
| [`hot_reload`](../crates/alacer/examples/hot_reload.rs) | a "simulate patch" button swapping the view generation live | the M4 hot-reload contract: state intact, scoped re-render, one frame (§13.2) | `cargo run -p alacer --example hot_reload` |
| [`inspector_demo`](../crates/alacer/examples/inspector_demo.rs) | the counter app with the Causality inspector riding on it | the M4 inspector: why did this render, who wrote this signal, why is this frame slow (§13.3) | `cargo run -p alacer --example inspector_demo` |

## Where the pixels are proven

Snapshot scenarios for the same pipeline live in
[`crates/alacer-testing/blessed/`](../crates/alacer-testing/blessed/) — 38 blessed PNGs
rendered headlessly by `alacer-testing/tests/m1_exit_criteria.rs`
(36 styled scenarios: labels, wrapped text, buttons in rest/pressed state,
columns, rows, panels, scrolled lists, conditional and nested trees, the
journal's undo/redo window, the input storm's probe panel, the announcement
window, the captured drag's log, the slider window, and the reported value's
window) plus the M0 pair
(`counter_clicked_once`, `list_scrolled_56`) blessed by
`m0_exit_criteria.rs`. If you add an example here, add its scenario there and
bless it:

```sh
ALACER_BLESS=1 cargo test -p alacer-testing
```

## Adding an example

1. Write the example in `crates/alacer/examples/` with a doc comment naming
   the architecture section it demonstrates.
2. Add a row to the table above.
3. If it shows a *new* visual arrangement, add a snapshot scenario to
   `alacer-testing` and bless it — examples are documentation; snapshots are
   the regression net behind it.
4. Handlers write signals in one batch; effects never mutate widgets (axiom
   A1). If your example cannot express itself as `Change` masks, redesign it.
