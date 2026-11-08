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
| [`counter_view`](../crates/alacer/examples/counter_view.rs) | the same app written in `rsx!` | the view DSL + keyed reconciliation (M1) | `cargo run -p alacer --example counter_view` |
| [`todo`](../crates/alacer/examples/todo.rs) | the todo canon: input submits on Enter, checkboxes toggle, the footer memo counts what is left | the M6 canon through `alacer_widgets::canon::todo_app` — one write batch ⇒ one scoped re-render (ADR-6, §13.2's one-frame path) | `cargo run -p alacer --example todo` |
| [`data_table`](../crates/alacer/examples/data_table.rs) | the data-table canon: a named grid, an "Add row" seam, cells that renumber on removal | the M6 canon through `alacer_widgets::canon::data_table_app` — a *navigable* AccessKit table (counts + positions) | `cargo run -p alacer --example data_table` |
| [`gallery`](../crates/alacer/examples/gallery.rs) | the whole widget vocabulary in one window: labels, buttons, text input, panels, list, rows, spacers | the M5 tour: signals, bindings, theme, translation-only scrolling | `cargo run -p alacer --example gallery` |
| [`scrolling_list`](../crates/alacer/examples/scrolling_list.rs) | 24 panels in a clipped viewport | translation-only wheel updates — zero measure calls per scroll tick (§8.2) | `cargo run -p alacer --example scrolling_list` |
| [`text_input`](../crates/alacer/examples/text_input.rs) | caret editing, secure mode, Tab focus cycling | the M3 IME contract: UTF-16 end to end, inline preedit + commit, partial paint per keystroke | `cargo run -p alacer --example text_input` |
| [`hot_reload`](../crates/alacer/examples/hot_reload.rs) | a "simulate patch" button swapping the view generation live | the M4 hot-reload contract: state intact, scoped re-render, one frame (§13.2) | `cargo run -p alacer --example hot_reload` |
| [`inspector_demo`](../crates/alacer/examples/inspector_demo.rs) | the counter app with the Causality inspector riding on it | the M4 inspector: why did this render, who wrote this signal, why is this frame slow (§13.3) | `cargo run -p alacer --example inspector_demo` |

## Where the pixels are proven

Snapshot scenarios for the same pipeline live in
[`crates/alacer-testing/blessed/`](../crates/alacer-testing/blessed/) — 32
blessed PNGs rendered headlessly by `alacer-testing/tests/m1_exit_criteria.rs`
(30 styled scenarios: labels, wrapped text, buttons in rest/pressed state,
columns, rows, panels, scrolled lists, conditional and nested trees) plus the
M0 pair (`counter_clicked_once`, `list_scrolled_56`) blessed by
`m0_exit_criteria.rs`. If you add an example
here, add its scenario there and bless it:

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
