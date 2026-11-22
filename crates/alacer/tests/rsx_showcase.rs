//! The §13.1 showcase suite: the flagship consumer examples ride the `rsx!`
//! DSL — the plan's "views that compile-check" mandate, pinned two ways.
//!
//! 1. **The source shape** — `todo.rs` and `gallery.rs` are the launch's
//!    viral DX artifacts, so the suite reads the example sources themselves
//!    and refuses a flagship that fell back to manual builder chains: the
//!    `rsx!` macro must be the whole view tree, and the delegation/
//!    `add_child` vocabularies must be gone from it.
//! 2. **The behavior** — the exact views the examples declare, mirrored
//!    here against the real headless pipeline: the todo canon's keyed `for`
//!    rows, the filter's `if`/`else` swaps (the empty-state hint, the
//!    clear-completed presence), toggles and deletes landing through the
//!    a11y tree — and the gallery's static catalog mounting fully named
//!    through the same tree, the only official query API (ADR-10).

use alacer::prelude::*;
use alacer_core::flux::RenderBinding;
use alacer_testing::TestHarness;
use alacer_widgets::{TodoFilter, TodoItem};
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::rc::Rc;

/// The two example sources the DX mandate names, read from the checkout
/// the test binary was built from.
const FLAGSHIP_EXAMPLES: [(&str, &str); 2] = [
    ("todo.rs", "crates/alacer/examples/todo.rs"),
    ("gallery.rs", "crates/alacer/examples/gallery.rs"),
];

/// The manual vocabularies a §13.1 flagship must not fall back to: the
/// canon builder delegation (`todo_app`) and the builder pipeline's own
/// child-assembly seam (`add_child`) — the DSL replaces both.
const MANUAL_VOCABULARIES: [&str; 2] = ["todo_app(", "pipeline.add_child("];

/// The showcase's todo view — the shape `examples/todo.rs` declares, the
/// whole tree compiled by `rsx!`: the committing input, the keyed `for`
/// rows (checkbox + label + delete, keys the *original* list index so the
/// LIS keeps survivors in place under any filter), the footer bar's three
/// filter words (the active one accented through the `if` branch), the
/// clear-completed door's honest absence, and the filter's empty-state
/// swap — `if`/`else` where the rows would be.
#[expect(
    clippy::too_many_lines,
    reason = "the canon is one declarative walk through the todo vocabulary; splitting it would hide what the DSL compiles"
)]
fn todo_view(
    todos: &alacer_core::FluxRwSignal<Vec<TodoItem>>,
    filter: &alacer_core::FluxRwSignal<TodoFilter>,
    footer: &RenderBinding<String>,
    wake: Option<&Rc<dyn Fn()>>,
) -> ViewNode {
    let items = todos.try_get_untracked().unwrap_or_default();
    let filter_state = filter.try_get_untracked().unwrap_or(TodoFilter::All);
    let visible: Vec<(u64, usize, &TodoItem)> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| filter_state.shows(item))
        .map(|(index, item)| (u64::try_from(index).unwrap_or(u64::MAX), index, item))
        .collect();
    let completed = items.iter().filter(|item| item.done).count();
    let filters = [TodoFilter::All, TodoFilter::Active, TodoFilter::Completed];
    let todos_handle = *todos;
    let filter_handle = *filter;
    let wake_owned = wake.cloned();
    let wake_submit = wake_owned.clone();
    let submit_todos = *todos;
    let wake_clear = wake_owned.clone();
    let clear_todos = *todos;

    rsx! {
        column { padding: 16.0, gap: 12.0,
            text_input {
                name: "Add a todo",
                placeholder: "What needs doing?",
                on_submit: alacer_view::PropValue::TextHandler(Rc::new(
                    std::cell::RefCell::new(move |text: &str| {
                        let label = text.trim();
                        if label.is_empty() {
                            return;
                        }
                        alacer_core::flux::Flux::batch_of(|| {
                            let mut next =
                                submit_todos.try_get_untracked().unwrap_or_default();
                            next.push(TodoItem {
                                done: false,
                                label: label.to_string(),
                            });
                            submit_todos.set(next);
                        });
                        if let Some(wake) = &wake_submit {
                            wake();
                        }
                    }),
                )),
            }
            if visible.is_empty() {
                label { text: "Nothing under {filter_state.name()} — add one above" }
            } else {
                column { gap: 8.0,
                    for (key, index, item) in visible.into_iter() {
                        row { key: key, gap: 8.0,
                            checkbox {
                                name: item.label.clone(),
                                checked: item.done,
                                on_toggle: alacer_view::PropValue::FlagHandler({
                                    let todos = todos_handle;
                                    let wake = wake_owned.clone();
                                    Rc::new(std::cell::RefCell::new(move |checked: bool| {
                                        alacer_core::flux::Flux::batch_of(|| {
                                            let mut next =
                                                todos.try_get_untracked().unwrap_or_default();
                                            if let Some(todo) = next.get_mut(index) {
                                                todo.done = checked;
                                            }
                                            todos.set(next);
                                        });
                                        if let Some(wake) = &wake {
                                            wake();
                                        }
                                    }))
                                }),
                            }
                            label { text: item.label.clone() }
                            button {
                                text: "Delete {item.label}",
                                on_click: alacer_view::PropValue::Handler({
                                    let todos = todos_handle;
                                    let wake = wake_owned.clone();
                                    Rc::new(std::cell::RefCell::new(move || {
                                        alacer_core::flux::Flux::batch_of(|| {
                                            let mut next =
                                                todos.try_get_untracked().unwrap_or_default();
                                            if index < next.len() {
                                                next.remove(index);
                                            }
                                            todos.set(next);
                                        });
                                        if let Some(wake) = &wake {
                                            wake();
                                        }
                                    }))
                                }),
                            }
                        }
                    }
                }
            }
            row { gap: 8.0,
                for word in filters.into_iter() {
                    if filter_state == word {
                        button {
                            text: word.name(),
                            color: alacer_core::color::Color::new(0x2B, 0x6C, 0xB0, 0xFF),
                            on_click: alacer_view::PropValue::Handler({
                                let filter = filter_handle;
                                let wake = wake_owned.clone();
                                Rc::new(std::cell::RefCell::new(move || {
                                    alacer_core::flux::Flux::batch_of(|| filter.set(word));
                                    if let Some(wake) = &wake {
                                        wake();
                                    }
                                }))
                            }),
                        }
                    } else {
                        button {
                            text: word.name(),
                            on_click: alacer_view::PropValue::Handler({
                                let filter = filter_handle;
                                let wake = wake_owned.clone();
                                Rc::new(std::cell::RefCell::new(move || {
                                    alacer_core::flux::Flux::batch_of(|| filter.set(word));
                                    if let Some(wake) = &wake {
                                        wake();
                                    }
                                }))
                            }),
                        }
                    }
                }
                if completed > 0 {
                    button {
                        text: "Clear completed ({completed})",
                        on_click: alacer_view::PropValue::Handler(Rc::new(
                            std::cell::RefCell::new(move || {
                                alacer_core::flux::Flux::batch_of(|| {
                                    let current =
                                        clear_todos.try_get_untracked().unwrap_or_default();
                                    let next: Vec<TodoItem> = current
                                        .iter()
                                        .filter(|item| !item.done)
                                        .cloned()
                                        .collect();
                                    if next.len() != current.len() {
                                        clear_todos.set(next);
                                    }
                                });
                                if let Some(wake) = &wake_clear {
                                    wake();
                                }
                            }),
                        )),
                    }
                } else {
                    spacer { size: 0.0 }
                }
            }
            label { binding: footer.clone() }
        }
    }
}

/// The footer label's node — the mounted root's last child (the view's
/// declaration order, the canon assembly's own lookup).
fn footer_node_of(
    root: &alacer_view::ViewRoot,
    harness: &TestHarness,
) -> alacer_core::arena::NodeId {
    *harness
        .pipeline()
        .arena()
        .children(root.root_id())
        .last()
        .expect("the todo view mounts a footer label")
}

/// The gallery's bound labels ride `RenderBinding`s the flux bridge
/// drives; the finder names the initial text the binding resolves to at
/// assembly time (the CI simulation's own resolved-name rule).
fn bound_label_node(harness: &TestHarness, initial: &str) -> Option<alacer_core::arena::NodeId> {
    let mut found = None;
    harness
        .pipeline()
        .arena()
        .for_each_pre_order(&mut |id, slot| {
            if found.is_none()
                && slot.widget().role() == Role::Label
                && slot.widget().label().resolve() == initial
            {
                found = Some(id);
            }
        });
    found
}

/// The source shape: the flagships ride the DSL, the manual vocabularies
/// are gone from them. The test that keeps §13.1 from regressing.
#[test]
fn the_flagship_examples_ride_the_rsx_dsl() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    for (name, relative) in FLAGSHIP_EXAMPLES {
        let source = std::fs::read_to_string(format!("{manifest}/../../{relative}"))
            .unwrap_or_else(|error| panic!("{name} is unreadable: {error}"));
        assert!(
            source.contains("rsx! {"),
            "{name} no longer rides the rsx! DSL — §13.1's showcase mandate \
             is the whole view tree compiled by the macro"
        );
        for vocabulary in MANUAL_VOCABULARIES {
            assert!(
                !source.contains(vocabulary),
                "{name} fell back to the manual `{vocabulary}` vocabulary — \
                 the flagship's view tree is the DSL's, not the builder's"
            );
        }
    }
}

/// The todo canon as a view: filter, toggle, delete, clear — every beat
/// landing through the a11y tree, the footer count riding the flux bridge.
#[expect(
    clippy::too_many_lines,
    reason = "one walk through the whole canon beats four partial ones: the walk is the story"
)]
#[test]
fn the_rsx_todo_view_filters_toggles_and_clears_through_the_a11y_tree() {
    let mut harness = TestHarness::new(360.0, 560.0);
    let todos = harness
        .pipeline_mut()
        .flux_mut()
        .signal(Vec::<TodoItem>::new());
    let filter = harness.pipeline_mut().flux_mut().signal(TodoFilter::All);
    let footer = RenderBinding::new("0 items left".to_string());
    let view = todo_view(&todos, &filter, &footer, None);
    let mut root = alacer::mount_view(&view, harness.pipeline_mut());
    harness.settle();

    // The empty state speaks before the first todo: the `if` branch's own
    // honest absence.
    let _found = harness.expect_a11y(Role::Label, "Nothing under All — add one above");
    let footer_node = footer_node_of(&root, &harness);
    harness.pipeline_mut().flux_mut().bind_value(
        todos,
        footer_node,
        &footer,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |todos: &Vec<TodoItem>| {
            let remaining = todos.iter().filter(|item| !item.done).count();
            format!("{remaining} items left")
        },
    );

    // Seed the canon's demo state: two open, one done — one write batch,
    // one scoped re-render.
    alacer_core::flux::Flux::batch_of(|| {
        todos.set(vec![
            TodoItem {
                done: false,
                label: "Read the architecture's §14.4 ladder".to_string(),
            },
            TodoItem {
                done: false,
                label: "Run the a11y queries headlessly".to_string(),
            },
            TodoItem {
                done: true,
                label: "Ship the canon examples".to_string(),
            },
        ]);
    });
    let next = todo_view(&todos, &filter, &footer, None);
    let _report = root.update(&next, harness.pipeline_mut());
    harness.settle();

    // The rows mounted keyed: three checkboxes named by their items, the
    // footer counting what is left.
    let _found = harness.expect_a11y(Role::CheckBox, "Read the architecture's §14.4 ladder");
    let _found = harness.expect_a11y(Role::CheckBox, "Run the a11y queries headlessly");
    let _found = harness.expect_a11y(Role::CheckBox, "Ship the canon examples");
    let _found = harness.expect_a11y(Role::Label, "2 items left");

    // The Completed filter: the `for` shrinks to the one done row, the
    // clear-completed door is present (one completed item), and the open
    // rows are gone from the tree.
    harness.press(Role::Button, "Completed");
    let next = todo_view(&todos, &filter, &footer, None);
    let _report = root.update(&next, harness.pipeline_mut());
    harness.settle();
    let _found = harness.expect_a11y(Role::CheckBox, "Ship the canon examples");
    let _found = harness.expect_a11y(Role::Button, "Clear completed (1)");
    assert!(
        harness
            .pipeline()
            .a11y_snapshot()
            .and_then(|tree| tree.find(Role::CheckBox, "Run the a11y queries headlessly"))
            .is_none(),
        "an open row survived the Completed filter"
    );

    // Toggling the last done row through its own checkbox empties the
    // filter: the `if`/`else` swaps the rows column for the empty-state
    // hint, and the clear-completed door yields to its honest absence.
    harness.press(Role::CheckBox, "Ship the canon examples");
    let next = todo_view(&todos, &filter, &footer, None);
    let _report = root.update(&next, harness.pipeline_mut());
    harness.settle();
    let _found = harness.expect_a11y(Role::Label, "Nothing under Completed — add one above");
    assert!(
        harness
            .pipeline()
            .a11y_snapshot()
            .and_then(|tree| tree.find(Role::Button, "Clear completed (1)"))
            .is_none(),
        "the clear-completed door survived an empty completed set"
    );

    // Back to All: the keyed rows remount, the delete door takes one row,
    // and the footer recounts.
    harness.press(Role::Button, "All");
    let next = todo_view(&todos, &filter, &footer, None);
    let _report = root.update(&next, harness.pipeline_mut());
    harness.settle();
    let _found = harness.expect_a11y(Role::CheckBox, "Read the architecture's §14.4 ladder");
    harness.press(Role::Button, "Delete Run the a11y queries headlessly");
    let next = todo_view(&todos, &filter, &footer, None);
    let _report = root.update(&next, harness.pipeline_mut());
    harness.settle();
    let _found = harness.expect_a11y(Role::Label, "2 items left");
    assert!(
        harness
            .pipeline()
            .a11y_snapshot()
            .and_then(|tree| tree.find(Role::CheckBox, "Run the a11y queries headlessly"))
            .is_none(),
        "the deleted row survived its own button"
    );

    // One more toggle through the checkbox: the completed set is non-empty
    // again, the clear door returns, and pressing it empties the done set —
    // the rows' identities surviving the whole walk (the LIS's survivors).
    harness.press(Role::CheckBox, "Read the architecture's §14.4 ladder");
    let next = todo_view(&todos, &filter, &footer, None);
    let _report = root.update(&next, harness.pipeline_mut());
    harness.settle();
    harness.press(Role::Button, "Clear completed (1)");
    let next = todo_view(&todos, &filter, &footer, None);
    let _report = root.update(&next, harness.pipeline_mut());
    harness.settle();
    let _found = harness.expect_a11y(Role::Label, "1 items left");
    assert!(
        harness
            .pipeline()
            .a11y_snapshot()
            .and_then(|tree| tree.find(Role::CheckBox, "Read the architecture's §14.4 ladder"))
            .is_none(),
        "the cleared row survived the clear"
    );
}

/// The gallery's static catalog as a view: the whole core vocabulary in
/// one `rsx!` tree, mounting fully named through the a11y tree — labels,
/// bound labels, buttons, the text input, the checkbox, the table (headers
/// and cells, the emphasized tier riding its prop), the canvas, the image,
/// the list with its rows, the spacer.
#[test]
fn the_rsx_gallery_catalog_mounts_named_and_reachable() {
    let mut harness = TestHarness::new(480.0, 780.0);
    let count = harness.pipeline_mut().flux_mut().signal(0_i32);
    let counter_binding = RenderBinding::new("Clicked 0 times".to_string());
    let typed = harness.pipeline_mut().flux_mut().signal(String::new());
    let echo_binding = RenderBinding::new("echo: (empty)".to_string());
    let toggles = harness.pipeline_mut().flux_mut().signal(0_i32);
    let toggle_binding = RenderBinding::new("toggled 0 times".to_string());
    let checker: std::sync::Arc<[u8]> = std::sync::Arc::from(vec![7_u8; 32 * 32 * 4].as_slice());
    let view = gallery_view(
        &count,
        &counter_binding,
        &typed,
        &echo_binding,
        &toggles,
        &toggle_binding,
        &checker,
    );
    let _root = alacer::mount_view(&view, harness.pipeline_mut());
    harness.settle();

    // Reachable and named through the a11y tree — the only query API.
    let _found = harness.expect_a11y(Role::Label, "Alacer gallery");
    let _found = harness.expect_a11y(Role::Button, "Increment");
    let _found = harness.expect_a11y(Role::Button, "Reset");
    let _found = harness.expect_a11y(Role::TextInput, "Gallery input");
    let _found = harness.expect_a11y(Role::CheckBox, "Ship in bulk");
    let _found = harness.expect_a11y(Role::Table, "Inventory");
    let _found = harness.expect_a11y(Role::ColumnHeader, "Item");
    let _found = harness.expect_a11y(Role::ColumnHeader, "Quantity");
    let _found = harness.expect_a11y(Role::TableCell, "Nails");
    let _found = harness.expect_a11y(Role::TableCell, "12");
    let _found = harness.expect_a11y(Role::Canvas, "Gallery sine strip");
    let _found = harness.expect_a11y(Role::Image, "Gallery checker");
    let _found = harness.expect_a11y(Role::Label, "item 3");
    harness.assert_all_named();

    // The bound labels found by their initial texts: the bridge's
    // assembly-time targets.
    assert!(
        bound_label_node(&harness, "Clicked 0 times").is_some(),
        "the catalog mounted the counter label"
    );
    assert!(
        bound_label_node(&harness, "echo: (empty)").is_some(),
        "the catalog mounted the echo label"
    );
    assert!(
        bound_label_node(&harness, "toggled 0 times").is_some(),
        "the catalog mounted the toggle label"
    );

    // One click through the tree: the write batch lands without a view
    // re-run — the static catalog's handlers ride the same batch currency
    // while the bindings carry every dynamic value.
    harness.press(Role::Button, "Increment");
    harness.settle();
    let _found = harness.expect_a11y(Role::Button, "Increment");
}

/// The gallery catalog view — the shape `examples/gallery.rs` declares.
#[expect(
    clippy::too_many_lines,
    reason = "the catalog is one declarative walk through the widget vocabulary; splitting it would hide what the DSL compiles"
)]
fn gallery_view(
    count: &alacer_core::FluxRwSignal<i32>,
    counter_binding: &RenderBinding<String>,
    typed: &alacer_core::FluxRwSignal<String>,
    echo_binding: &RenderBinding<String>,
    toggles: &alacer_core::FluxRwSignal<i32>,
    toggle_binding: &RenderBinding<String>,
    checker: &std::sync::Arc<[u8]>,
) -> ViewNode {
    let counter_binding = counter_binding.clone();
    let echo_binding = echo_binding.clone();
    let toggle_binding = toggle_binding.clone();
    let count_signal = *count;
    let typed_signal = *typed;
    let toggles_signal = *toggles;
    let rows: Vec<(u64, alacer_core::color::Color)> = RAMP
        .iter()
        .cycle()
        .take(GALLERY_ITEMS)
        .enumerate()
        .map(|(index, color)| (u64::try_from(index).unwrap_or(u64::MAX), *color))
        .collect();
    let checker = checker.clone();

    rsx! {
        column { padding: 20.0, gap: 14.0,
            label { text: "Alacer gallery", font_size: 24.0, color: alacer_widgets::theme::TEXT }
            label {
                text: "Every core widget, one window. Type below, click, scroll.",
                font_size: 13.0,
                color: alacer_widgets::theme::TEXT_MUTED,
            }
            label { binding: counter_binding }
            row { gap: 10.0,
                button {
                    text: "Increment",
                    color: alacer_widgets::theme::ACCENT,
                    on_click: move || {
                        alacer_core::flux::Flux::batch_of(|| {
                            let current = count_signal.try_get_untracked().unwrap_or_default();
                            count_signal.set(current + 1);
                        });
                    }
                }
                button {
                    text: "Reset",
                    color: alacer_widgets::theme::BORDER,
                    text_color: alacer_widgets::theme::TEXT,
                    on_click: move || {
                        alacer_core::flux::Flux::batch_of(|| count_signal.set(0_i32));
                    }
                }
            }
            label {
                binding: echo_binding,
                font_size: 13.0,
                color: alacer_widgets::theme::TEXT_MUTED,
            }
            text_input {
                name: "Gallery input",
                placeholder: "type here — IME friendly",
                on_changed: alacer_view::PropValue::TextHandler(Rc::new(
                    std::cell::RefCell::new(move |value: &str| {
                        alacer_core::flux::Flux::batch_of(|| typed_signal.set(value.to_string()));
                    }),
                )),
            }
            label {
                binding: toggle_binding,
                font_size: 13.0,
                color: alacer_widgets::theme::TEXT_MUTED,
            }
            checkbox {
                name: "Ship in bulk",
                on_toggle: alacer_view::PropValue::FlagHandler(Rc::new(
                    std::cell::RefCell::new(move |_checked: bool| {
                        alacer_core::flux::Flux::batch_of(|| {
                            let current = toggles_signal.try_get_untracked().unwrap_or_default();
                            toggles_signal.set(current + 1);
                        });
                    }),
                )),
            }
            table { name: "Inventory", columns: 3, header_fill: alacer_core::color::Color::rgb(30, 41, 59),
                table_row {
                    column_header { text: "Item", column: 0 }
                    column_header { text: "Quantity", column: 1 }
                    column_header { text: "Location", column: 2 }
                }
                for (index, cells) in GALLERY_INVENTORY.iter().enumerate() {
                    table_row {
                        for (column, cell) in cells.iter().enumerate() {
                            table_cell {
                                text: *cell,
                                row: index,
                                column: column,
                                strong: column == 1,
                            }
                        }
                    }
                }
            }
            row { gap: 14.0,
                canvas {
                    name: "Gallery sine strip",
                    width: 240.0,
                    height: 120.0,
                    on_paint: alacer_view::PropValue::Painter(alacer_view::Painter::new(|ctx| {
                        let size = ctx.size();
                        ctx.push_quad(alacer_core::scene::Quad::filled(
                            alacer_core::geom::Rect::new(
                                alacer_core::geom::Point::new(0.0, 0.0),
                                size,
                            ),
                            alacer_core::color::Color::rgb(31, 122, 77),
                        ));
                    }))
                }
                image {
                    name: "Gallery checker",
                    description: "A 32 by 32 checkerboard generated in memory",
                    width: 96.0,
                    height: 96.0,
                    pixels: alacer_view::PropValue::Pixels(alacer_view::Pixels::new(
                        checker.clone(),
                        32,
                        32,
                    ))
                }
            }
            list { height: 120.0, content_height: 560.0,
                for (index, color) in rows.into_iter() {
                    row { gap: 8.0,
                        panel { height: 28.0, color: color }
                        label {
                            text: "item {index}",
                            font_size: 12.0,
                            color: alacer_widgets::theme::TEXT_MUTED,
                        }
                    }
                }
            }
            spacer { size: 8.0 }
            label {
                text: "labels · buttons · bindings · text input · checkbox · table · canvas · image · panels · list · rows · spacers",
                font_size: 11.0,
                color: alacer_widgets::theme::BORDER,
            }
        }
    }
}

/// The gallery's accent ramp — the list rows cycle it.
const RAMP: [alacer_core::color::Color; 5] = [
    alacer_core::color::Color::rgb(37, 99, 235),
    alacer_core::color::Color::rgb(14, 165, 233),
    alacer_core::color::Color::rgb(22, 163, 74),
    alacer_core::color::Color::rgb(234, 179, 8),
    alacer_core::color::Color::rgb(239, 68, 68),
];

/// The list's row count in the catalog view.
const GALLERY_ITEMS: usize = 20;

/// The catalog table's inventory (the original gallery's cells).
const GALLERY_INVENTORY: [[&str; 3]; 3] = [
    ["Nails", "12", "A1"],
    ["Screws", "5", "B2"],
    ["Bolts", "40", "C3"],
];
