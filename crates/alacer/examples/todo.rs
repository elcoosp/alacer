//! The M6 todo canon (§1's definition of done, §5's shape) as a §13.1
//! showcase: the *entire* view tree is compiled by the `rsx!` macro — the
//! committing input, the keyed `for` rows over the filtered items (the
//! checkboxes announcing `Toggled`, the rows' identities surviving the
//! filter through their original-list keys), the `if`/`else` blocks for the
//! filter state (the empty-state hint where the rows would be), the footer
//! bar's three filter words with the active one accented through its own
//! `if`, and the clear-completed door's honest absence — one write batch
//! per interaction, one scoped re-render, at most one layout+paint cycle
//! (ADR-6).
//!
//! The state rides the canon's own vocabulary — [`TodoItem`] and
//! [`TodoFilter`] — while the view is the example's own: the *same* words
//! the M6 exit suite drives through `alacer_widgets::canon::todo_app`, now
//! written the way a consumer app writes views. Every control is reachable,
//! named, and stateful in the a11y tree: the checkboxes announce `Toggled`
//! on/off, the filter words name themselves, and the rows' identities
//! survive the filter (the keyed reconciler moves survivors, it never
//! rewrites them — the handlers carry the list index, not the display one).
//!
//! Try it: type and press Enter, toggle rows, click All/Active/Completed,
//! clear the completed set. Run with `cargo run -p alacer --example todo`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer::prelude::*;
use alacer_core::flux::RenderBinding;
use alacer_view::PropValue;
use alacer_widgets::{TodoFilter, TodoItem};
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::rc::Rc;

/// The wake seam's shape: the patch-channel trigger the handlers fire after
/// every write batch (the windowed app's door; the headless suites pass
/// `None` and re-render themselves).
type Wake = Option<Rc<dyn Fn()>>;

/// The accent the active filter word rides — the canon's own blue.
const FILTER_ACCENT: alacer_core::color::Color =
    alacer_core::color::Color::new(0x2B, 0x6C, 0xB0, 0xFF);

/// The todo view — the whole tree compiled by `rsx!` from the current
/// signal state (views are cheap trees, rebuilt per re-render, §6.1): the
/// input, the keyed `for` rows, the footer bar, the bound footer.
#[expect(
    clippy::too_many_lines,
    reason = "the canon is one declarative walk through the todo vocabulary; splitting it would hide the DSL showcase it exists to be"
)]
fn todo_view(
    todos: &alacer_core::FluxRwSignal<Vec<TodoItem>>,
    filter: &alacer_core::FluxRwSignal<TodoFilter>,
    footer: &RenderBinding<String>,
    wake: Option<&Rc<dyn Fn()>>,
) -> ViewNode {
    let items = todos.try_get_untracked().unwrap_or_default();
    let filter_state = filter.try_get_untracked().unwrap_or(TodoFilter::All);
    // The rows: keyed by *original* index so the LIS keeps survivors in
    // place and mounts/removes exactly the difference (§6.2) — the handler
    // carries the list index, not the display one, so toggling and deleting
    // stay correct under any filter.
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
                on_submit: PropValue::TextHandler(Rc::new(RefCell::new(move |text: &str| {
                    let label = text.trim();
                    if label.is_empty() {
                        return;
                    }
                    alacer_core::flux::Flux::batch_of(|| {
                        let mut next = submit_todos.try_get_untracked().unwrap_or_default();
                        next.push(TodoItem {
                            done: false,
                            label: label.to_string(),
                        });
                        submit_todos.set(next);
                    });
                    if let Some(wake) = &wake_submit {
                        wake();
                    }
                }))),
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
                                on_toggle: PropValue::FlagHandler({
                                    let todos = todos_handle;
                                    let wake = wake_owned.clone();
                                    Rc::new(RefCell::new(move |checked: bool| {
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
                                on_click: PropValue::Handler({
                                    let todos = todos_handle;
                                    let wake = wake_owned.clone();
                                    Rc::new(RefCell::new(move || {
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
                            color: FILTER_ACCENT,
                            on_click: PropValue::Handler({
                                let filter = filter_handle;
                                let wake = wake_owned.clone();
                                Rc::new(RefCell::new(move || {
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
                            on_click: PropValue::Handler({
                                let filter = filter_handle;
                                let wake = wake_owned.clone();
                                Rc::new(RefCell::new(move || {
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
                        on_click: PropValue::Handler(Rc::new(RefCell::new(move || {
                            alacer_core::flux::Flux::batch_of(|| {
                                let current = clear_todos.try_get_untracked().unwrap_or_default();
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
                        }))),
                    }
                } else {
                    spacer { size: 0.0 }
                }
            }
            label { binding: footer.clone() }
        }
    }
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // State outside the view generation — the ADR-7 state-owner contract.
    // The wake fires the patch channel after every write batch: one wake ⇒
    // one scoped re-render ⇒ at most one layout+paint cycle.
    let wake: Wake = Some(Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    }));
    let todos = pipeline.flux().signal(Vec::<TodoItem>::new());
    let filter = pipeline.flux().signal(TodoFilter::All);
    let footer = RenderBinding::new("0 items left".to_string());

    // Mount the DSL's tree, then bind the footer: the root's last child is
    // the footer label (the view's declaration order, the canon assembly's
    // own lookup).
    let mut root = alacer::mount_view(
        &todo_view(&todos, &filter, &footer, wake.as_ref()),
        &mut pipeline,
    );
    let footer_node = *pipeline
        .arena()
        .children(root.root_id())
        .last()
        .expect("the todo view mounts a footer label");
    pipeline.flux_mut().bind_value(
        todos,
        footer_node,
        &footer,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |todos: &Vec<TodoItem>| {
            let remaining = todos.iter().filter(|item| !item.done).count();
            format!("{remaining} items left")
        },
    );

    // Seed the demo state so the footer bar's words have something to
    // filter: two open, one done — then land it with the same scoped
    // re-render the wake would fire.
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
    let _ = root.update(
        &todo_view(&todos, &filter, &footer, wake.as_ref()),
        &mut pipeline,
    );

    let mut app = AlacerApp::new(pipeline, "Alacer todo", (360.0, 560.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let _ = root.update(
            &todo_view(&todos, &filter, &footer, wake.as_ref()),
            pipeline,
        );
    }));
    if let Err(error) = app.with_patch_channel(channel).run() {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
