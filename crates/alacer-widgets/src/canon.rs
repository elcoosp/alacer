//! The benchmark canon (M6): the todo and data-table apps as shared
//! builders.
//!
//! The v1 definition of done (architecture §1) benchmarks and documents the
//! framework against the counter / todo / data-table canon. Only the counter
//! existed; the two remaining apps live here — one implementation, two
//! consumers: the examples assemble them behind a window, and the M6 exit
//! suite drives the *same builders* headlessly through the a11y queries.
//!
//! Both apps are view-edition: structure rides the keyed reconciler, dynamic
//! text rides [`RenderBinding`] slots, and every mutation is one write batch
//! ⇒ one scoped re-render ⇒ at most one layout+paint cycle (ADR-6). The
//! re-render is explicit — [`TodoHandles::rerender`] / [`DataTableHandles::
//! rerender`] — because effects never touch widgets (rule 1): the caller
//! wires it to the patch channel (`alacer::platform::PatchChannel`) in a
//! windowed app, or calls it directly between harness interactions in tests.

use alacer_core::flux::{FluxRwSignal, RenderBinding};
use alacer_core::passes::PassPipeline;
use alacer_core::{Flux, GetUntracked as _, Set as _};
use alacer_view::{PatchReport, PropValue, ViewNode, ViewRoot};
use std::cell::RefCell;
use std::rc::Rc;

/// The re-render trigger the canon handlers call after a signal write: in a
/// windowed app it fires the patch channel (one wake ⇒ one scoped re-render,
/// the §13.2 path); headless callers pass `None` and re-render explicitly
/// between interactions ([`TodoHandles::rerender`]).
pub type WakeUp = Option<Rc<dyn Fn()>>;

fn wake_up(wake: &WakeUp) {
    if let Some(wake) = wake {
        wake();
    }
}

/// One todo row's state (the architecture's §5 snippet, verbatim shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoItem {
    /// Whether the item is done.
    pub done: bool,
    /// What to do.
    pub label: String,
}

/// The todo canon's filter — the footer bar's three words (the `TodoMVC`
/// contract's own vocabulary): which items the rows show. The filter is
/// a signal like the todos: one write batch per mutation, the re-render
/// reading it beside the items (ADR-6's shape at the filter tier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoFilter {
    /// Every item, done or open.
    All,
    /// The items still open.
    Active,
    /// The items already done.
    Completed,
}

impl TodoFilter {
    /// The filter's word — the button's own label (the a11y name the
    /// press rides).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
        }
    }

    /// Whether `item` shows under this filter (the row walk's own
    /// predicate — the view's filter and the tests' expectations read
    /// the same one).
    #[must_use]
    pub const fn shows(self, item: &TodoItem) -> bool {
        match self {
            Self::All => true,
            Self::Active => !item.done,
            Self::Completed => item.done,
        }
    }
}

/// The todo canon's assembly: signal, footer binding, and the mounted view.
pub struct TodoHandles {
    /// The todos signal — one write batch per mutation (ADR-6).
    pub todos: FluxRwSignal<Vec<TodoItem>>,
    /// The footer binding: "N items left", updated by the flux bridge.
    pub footer: RenderBinding<String>,
    /// The filter signal: which items the rows show
    /// ([`TodoFilter`]). One write batch per mutation (ADR-6).
    pub filter: FluxRwSignal<TodoFilter>,
    root: ViewRoot,
    wake: WakeUp,
}

impl TodoHandles {
    /// The scoped re-render: rebuilds the view from the current signal
    /// state and patches the retained tree in place (keyed rows, LIS diff).
    /// One call per write batch — the ADR-6 contract's view edition.
    pub fn rerender(&mut self, pipeline: &mut PassPipeline) -> PatchReport {
        let view = todo_view(&self.todos, &self.filter, &self.footer, &self.wake);
        self.root.update(&view, pipeline)
    }

    /// Reads the current todos (test introspection and app logic).
    ///
    /// # Panics
    ///
    /// Panics when the signal's owner is gone (a dropped flux — an app bug).
    pub fn with_todos<R>(&self, f: impl FnOnce(&[TodoItem]) -> R) -> R {
        let todos = self
            .todos
            .try_get_untracked()
            .expect("the todo signal's owner is alive");
        f(&todos)
    }

    /// How many items are not done (the footer memo's value).
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.with_todos(|todos| todos.iter().filter(|item| !item.done).count())
    }

    /// How many items are done (the clear-completed button's own count).
    #[must_use]
    pub fn completed(&self) -> usize {
        self.with_todos(|todos| todos.iter().filter(|item| item.done).count())
    }

    /// Sets the filter — one write batch. The re-render is the caller's
    /// next [`Self::rerender`]; a windowed app's handlers fire the wake
    /// instead.
    pub fn set_filter(&self, filter: TodoFilter) {
        Flux::batch_of(|| self.filter.set(filter));
    }

    /// Removes every done item — one write batch. The write that lands
    /// where the list already is queues nothing: with no completed item
    /// the signal keeps its value (the idle-frame property, A1's rule at
    /// the todo tier).
    pub fn clear_completed(&self) {
        Flux::batch_of(|| {
            let current = self.todos.try_get_untracked().unwrap_or_default();
            if let Some(next) = completed_removed(&current) {
                self.todos.set(next);
            }
        });
    }

    /// Pushes one item — one write batch (ADR-6). The re-render is the
    /// caller's next [`Self::rerender`].
    pub fn add(&self, label: &str) {
        let label = label.trim();
        if label.is_empty() {
            return;
        }
        let item = TodoItem {
            done: false,
            label: label.to_string(),
        };
        Flux::batch_of(|| {
            let mut next = self.todos.try_get_untracked().unwrap_or_default();
            next.push(item);
            self.todos.set(next);
        });
    }

    /// Toggles one item's done flag — one write batch.
    pub fn toggle(&self, index: usize) {
        Flux::batch_of(|| {
            let mut next = self.todos.try_get_untracked().unwrap_or_default();
            if let Some(item) = next.get_mut(index) {
                item.done = !item.done;
            }
            self.todos.set(next);
        });
    }

    /// Removes one item — one write batch.
    pub fn remove(&self, index: usize) {
        Flux::batch_of(|| {
            let mut next = self.todos.try_get_untracked().unwrap_or_default();
            if index < next.len() {
                next.remove(index);
            }
            self.todos.set(next);
        });
    }

    /// The mounted root's node.
    #[must_use]
    pub const fn root_node(&self) -> alacer_core::NodeId {
        self.root.root_id()
    }
}

/// Builds the todo view for the current signal state: input, keyed rows
/// (checkbox + label + delete) filtered by the filter signal, the footer
/// bar (the filter's three words, clear-completed when there is something
/// to clear), and the bound footer.
fn todo_view(
    todos: &FluxRwSignal<Vec<TodoItem>>,
    filter: &FluxRwSignal<TodoFilter>,
    footer: &RenderBinding<String>,
    wake: &WakeUp,
) -> ViewNode {
    let items = todos.try_get_untracked().unwrap_or_default();
    let filter_state = filter.try_get_untracked().unwrap_or(TodoFilter::All);
    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 12.0_f32);

    // The input: Enter commits — the §5 snippet's `on_submit` seam (M6).
    // First focusable stays the input: the Tab order's head (the M6 exit
    // suite's own walk starts here).
    let submit_todos = *todos;
    let submit_wake = wake.clone();
    root = root.child(
        ViewNode::element("text_input")
            .prop("name", "Add a todo")
            .prop("placeholder", "What needs doing?")
            .prop(
                "on_submit",
                PropValue::TextHandler(Rc::new(RefCell::new(move |text: &str| {
                    let label = text.trim();
                    if label.is_empty() {
                        return;
                    }
                    Flux::batch_of(|| {
                        let mut next = submit_todos.try_get_untracked().unwrap_or_default();
                        next.push(TodoItem {
                            done: false,
                            label: label.to_string(),
                        });
                        submit_todos.set(next);
                    });
                    wake_up(&submit_wake);
                }))),
            ),
    );

    // The rows: keyed by *original* index so the LIS keeps survivors in
    // place and mounts/removes exactly the difference (§6.2), filtered by
    // the filter signal — the handler still names the item's index in the
    // full list, so toggling and deleting stay correct under any filter.
    let rows: Vec<ViewNode> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| filter_state.shows(item))
        .map(|(index, item)| todo_row(index, item, todos, wake))
        .collect();

    root = root.child(
        ViewNode::element("column")
            .prop("gap", 8.0_f32)
            .children(rows),
    );
    root = root.child(todo_footer_bar(todos, filter, wake, filter_state, &items));
    root = root.child(ViewNode::element("label").prop("binding", footer.clone()));
    root
}

/// One todo row: the checkbox (the item's a11y name and `Toggled` state),
/// the label, the delete button — handlers carry the *list* index, not
/// the display one, so they land on the right item under any filter.
fn todo_row(
    index: usize,
    item: &TodoItem,
    todos: &FluxRwSignal<Vec<TodoItem>>,
    wake: &WakeUp,
) -> ViewNode {
    let toggle_todos = *todos;
    let toggle_wake = wake.clone();
    let toggle = PropValue::FlagHandler(Rc::new(RefCell::new(move |checked: bool| {
        Flux::batch_of(|| {
            let mut next = toggle_todos.try_get_untracked().unwrap_or_default();
            if let Some(todo) = next.get_mut(index) {
                todo.done = checked;
            }
            toggle_todos.set(next);
        });
        wake_up(&toggle_wake);
    })));
    let delete_todos = *todos;
    let delete_wake = wake.clone();
    let delete = PropValue::Handler(Rc::new(RefCell::new(move || {
        Flux::batch_of(|| {
            let mut next = delete_todos.try_get_untracked().unwrap_or_default();
            if index < next.len() {
                next.remove(index);
            }
            delete_todos.set(next);
        });
        wake_up(&delete_wake);
    })));
    ViewNode::element("row")
        .key(u64::try_from(index).unwrap_or(u64::MAX))
        .prop("gap", 8.0_f32)
        .child(
            ViewNode::element("checkbox")
                .prop("name", item.label.clone())
                .prop("checked", item.done)
                .prop("on_toggle", toggle),
        )
        .child(ViewNode::element("label").prop("text", item.label.clone()))
        .child(
            ViewNode::element("button")
                .prop("text", format!("Delete {}", item.label))
                .prop("on_click", delete),
        )
}

/// The todo footer bar: the filter's three words (the active one
/// accented) and clear-completed when the completed set is non-empty —
/// the button's own honest absence, never a disabled stub. The footer
/// label rides after it, the root's last child (the binding's
/// declaration order, the mount's own lookup).
fn todo_footer_bar(
    todos: &FluxRwSignal<Vec<TodoItem>>,
    filter: &FluxRwSignal<TodoFilter>,
    wake: &WakeUp,
    filter_state: TodoFilter,
    items: &[TodoItem],
) -> ViewNode {
    let mut bar = ViewNode::element("row").prop("gap", 8.0_f32);
    for word in [TodoFilter::All, TodoFilter::Active, TodoFilter::Completed] {
        let filter_signal = *filter;
        let filter_wake = wake.clone();
        let mut button = ViewNode::element("button").prop("text", word.name()).prop(
            "on_click",
            PropValue::Handler(Rc::new(RefCell::new(move || {
                Flux::batch_of(|| filter_signal.set(word));
                wake_up(&filter_wake);
            }))),
        );
        if filter_state == word {
            button = button.prop(
                "color",
                PropValue::Color(alacer_core::color::Color::new(0x2B, 0x6C, 0xB0, 0xFF)),
            );
        }
        bar = bar.child(button);
    }
    let completed = items.iter().filter(|item| item.done).count();
    if completed > 0 {
        let clear_todos = *todos;
        let clear_wake = wake.clone();
        bar = bar.child(
            ViewNode::element("button")
                .prop("text", format!("Clear completed ({completed})"))
                .prop(
                    "on_click",
                    PropValue::Handler(Rc::new(RefCell::new(move || {
                        Flux::batch_of(|| {
                            let current = clear_todos.try_get_untracked().unwrap_or_default();
                            if let Some(next) = completed_removed(&current) {
                                clear_todos.set(next);
                            }
                        });
                        wake_up(&clear_wake);
                    }))),
                ),
        );
    }
    bar
}

/// The list with every done item removed, `None` when there was nothing
/// to remove — the write that lands where the list already is prices
/// nothing (the idle-frame property, A1's rule at the todo tier). The
/// door and the footer bar's own handler share the one fold.
fn completed_removed(current: &[TodoItem]) -> Option<Vec<TodoItem>> {
    let next: Vec<TodoItem> = current.iter().filter(|item| !item.done).cloned().collect();
    (next.len() != current.len()).then_some(next)
}

/// Assembles the todo canon onto a pipeline (§5's shape, M6 edition): the
/// input commits on Enter, rows toggle and delete through the a11y tree,
/// and the footer binding counts what is left. `wake` (the patch-channel
/// trigger) is what a windowed app passes so handler writes re-render; the
/// exit suite passes `None` and calls [`TodoHandles::rerender`] itself.
///
/// # Panics
///
/// Panics when the view names an unregistered element (a registry bug, not
/// a runtime condition).
#[must_use]
pub fn todo_app(pipeline: &mut PassPipeline, wake: WakeUp) -> TodoHandles {
    let todos = pipeline.flux().signal(Vec::<TodoItem>::new());
    let filter = pipeline.flux().signal(TodoFilter::All);
    let footer = RenderBinding::new("0 items left".to_string());

    let view = todo_view(&todos, &filter, &footer, &wake);
    let root = ViewRoot::mount(&view, pipeline, crate::default_registry());
    let root_id = root.root_id();

    // The footer is the root's last child (the view's declaration order).
    let footer_node = *pipeline
        .arena()
        .children(root_id)
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

    TodoHandles {
        todos,
        footer,
        filter,
        root,
        wake,
    }
}

/// The data-table canon's assembly: a toolbar (the "Add row" seam), the
/// table, the rows signal, and the mounted view.
pub struct DataTableHandles {
    /// The row data — each row is `columns` cell strings. One write batch
    /// per mutation (ADR-6).
    pub rows: FluxRwSignal<Vec<Vec<String>>>,
    /// The selection signal: the selected row's index (the row's
    /// `selected` word, M108's tier). One write batch per mutation (ADR-6).
    pub selected: FluxRwSignal<Option<usize>>,
    /// The sort state: `(column, direction)` while a column sorts, `None`
    /// when the grid keeps its own order. One write batch per mutation.
    pub sort: FluxRwSignal<Option<(usize, alacer_core::SortDirection)>>,
    root: ViewRoot,
    root_node: alacer_core::NodeId,
    /// The header labels (column names).
    headers: Vec<String>,
    wake: WakeUp,
}

impl DataTableHandles {
    /// The scoped re-render (same contract as [`TodoHandles::rerender`]).
    pub fn rerender(&mut self, pipeline: &mut PassPipeline) -> PatchReport {
        let view = data_table_view(
            &self.rows,
            &self.selected,
            &self.sort,
            &self.headers,
            &self.wake,
        );
        self.root.update(&view, pipeline)
    }

    /// Appends one row — one write batch.
    pub fn push_row(&self, cells: Vec<String>) {
        Flux::batch_of(|| {
            let mut next = self.rows.try_get_untracked().unwrap_or_default();
            next.push(cells);
            self.rows.set(next);
        });
    }

    /// Removes one row — one write batch. A selection naming the removed
    /// row clears beside it: a dead row holds no selection (the removed
    /// row's `selected` word has no tree to land in).
    pub fn remove_row(&self, index: usize) {
        Flux::batch_of(|| {
            let mut next = self.rows.try_get_untracked().unwrap_or_default();
            if index < next.len() {
                next.remove(index);
            }
            self.rows.set(next);
            let selected = self.selected.try_get_untracked().unwrap_or_default();
            if selected.is_some_and(|picked| picked == index) {
                self.selected.set(None);
            }
        });
    }

    /// Selects one row — one write batch. `None` clears the selection;
    /// the rows' `selected` word re-announces with the next re-render.
    pub fn select_row(&self, index: Option<usize>) {
        Flux::batch_of(|| self.selected.set(index));
    }

    /// The selection, if a row holds it.
    #[must_use]
    pub fn selected_row(&self) -> Option<usize> {
        self.selected.try_get_untracked().unwrap_or_default()
    }

    /// Sorts by column — the cycle a column's own control rides: first
    /// ask ascending, second ask descending, third ask clears. A
    /// different column starts its own cycle at ascending. One write
    /// batch.
    pub fn sort_by(&self, column: usize) {
        Flux::batch_of(|| {
            let current = self.sort.try_get_untracked().unwrap_or_default();
            self.sort.set(next_sort_state(current, column));
        });
    }

    /// The sort state, if a column sorts.
    #[must_use]
    pub fn sort_state(&self) -> Option<(usize, alacer_core::SortDirection)> {
        self.sort.try_get_untracked().unwrap_or_default()
    }

    /// The header labels (column names).
    #[must_use]
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// The mounted root's node.
    #[must_use]
    pub const fn root_node(&self) -> alacer_core::NodeId {
        self.root_node
    }
}

/// The data-table view: the toolbar (add, remove selected), the sort
/// bar, the header row (anchored by the sentinel key, each header naming
/// its sort word), the keyed data rows whose cells carry their *display*
/// grid positions, and the selection bar. The geometry the AT walks
/// renumbers under the sort while the keys keep each row's identity.
fn data_table_view(
    rows: &FluxRwSignal<Vec<Vec<String>>>,
    selected: &FluxRwSignal<Option<usize>>,
    sort: &FluxRwSignal<Option<(usize, alacer_core::SortDirection)>>,
    headers: &[String],
    wake: &WakeUp,
) -> ViewNode {
    let data = rows.try_get_untracked().unwrap_or_default();
    let picked = selected.try_get_untracked().unwrap_or_default();
    let sort_state = sort.try_get_untracked().unwrap_or_default();
    let columns = headers.len();
    let display = sorted_display(&data, sort_state);

    // The header row: each header carries the M112 sort word — always
    // present, "none" when this column doesn't sort, so the patch can
    // clear the word it set before (the view states the truth instead of
    // omitting it).
    let header_cells: Vec<ViewNode> = headers
        .iter()
        .enumerate()
        .map(|(column, name)| {
            let word = sort_word_name(sort_state, column);
            ViewNode::element("column_header")
                .prop("text", name.clone())
                .prop("column", usize_to_f32(column))
                .prop("sort", word)
        })
        .collect();
    let header_row = ViewNode::element("table_row")
        // The header sentinel key: the LIS keeps it anchored first while the
        // data rows around it diff (keys are unique: u64::MAX vs 0..n).
        .key(u64::MAX)
        .prop("gap", 8.0_f32)
        .children(header_cells);

    let data_rows: Vec<ViewNode> = display
        .iter()
        .enumerate()
        .map(|(display_index, (original_index, cells))| {
            data_table_row(display_index, *original_index, cells, columns, picked)
        })
        .collect();

    let table = ViewNode::element("table")
        .prop("name", "Inventory")
        .prop("rows", usize_to_f32(data.len()))
        .prop("columns", usize_to_f32(columns))
        .prop("gap", 4.0_f32)
        .child(header_row)
        .children(data_rows);

    ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 12.0_f32)
        .child(data_table_toolbar(rows, selected, wake, picked))
        .child(data_table_sort_bar(sort, headers, wake, sort_state))
        .child(table)
        .child(data_table_selection_bar(selected, wake, picked, &display))
}

/// The toolbar: "Add row" appends a generated row, "Remove selected"
/// removes the row the selection names — one write batch per ask, then
/// the wake. Each button rides the same a11y contract as every control
/// (named, actionable).
fn data_table_toolbar(
    rows: &FluxRwSignal<Vec<Vec<String>>>,
    selected: &FluxRwSignal<Option<usize>>,
    wake: &WakeUp,
    picked: Option<usize>,
) -> ViewNode {
    let add_rows = *rows;
    let add_wake = wake.clone();
    let mut toolbar = ViewNode::element("row").prop("gap", 8.0_f32).child(
        ViewNode::element("button").prop("text", "Add row").prop(
            "on_click",
            PropValue::Handler(Rc::new(RefCell::new(move || {
                Flux::batch_of(|| {
                    let mut next = add_rows.try_get_untracked().unwrap_or_default();
                    let index = next.len();
                    next.push(generated_row(index));
                    add_rows.set(next);
                });
                wake_up(&add_wake);
            }))),
        ),
    );
    if picked.is_some() {
        let remove_rows = *rows;
        let remove_selected = *selected;
        let remove_wake = wake.clone();
        toolbar = toolbar.child(
            ViewNode::element("button")
                .prop("text", "Remove selected")
                .prop(
                    "on_click",
                    PropValue::Handler(Rc::new(RefCell::new(move || {
                        Flux::batch_of(|| {
                            let index = remove_selected.try_get_untracked().unwrap_or_default();
                            if let Some(index) = index {
                                let mut next = remove_rows.try_get_untracked().unwrap_or_default();
                                if index < next.len() {
                                    next.remove(index);
                                }
                                remove_rows.set(next);
                                remove_selected.set(None);
                            }
                        });
                        wake_up(&remove_wake);
                    }))),
                ),
        );
    }
    toolbar
}

/// The sort bar: one button per column, its label naming the column and
/// the cycle's current answer — the state the header's own `sort` word
/// announces beside it. The cycle is the door's own
/// ([`DataTableHandles::sort_by`]): ascending, descending, clear.
fn data_table_sort_bar(
    sort: &FluxRwSignal<Option<(usize, alacer_core::SortDirection)>>,
    headers: &[String],
    wake: &WakeUp,
    sort_state: Option<(usize, alacer_core::SortDirection)>,
) -> ViewNode {
    let mut bar = ViewNode::element("row").prop("gap", 8.0_f32);
    for (column, name) in headers.iter().enumerate() {
        let sort_signal = *sort;
        let sort_wake = wake.clone();
        let word = match sort_state {
            Some((sorted, alacer_core::SortDirection::Ascending)) if sorted == column => {
                format!("Sort {name} (ascending)")
            }
            Some((sorted, _)) if sorted == column => format!("Sort {name} (descending)"),
            _ => format!("Sort {name}"),
        };
        bar = bar.child(ViewNode::element("button").prop("text", word).prop(
            "on_click",
            PropValue::Handler(Rc::new(RefCell::new(move || {
                Flux::batch_of(|| {
                    let current = sort_signal.try_get_untracked().unwrap_or_default();
                    sort_signal.set(next_sort_state(current, column));
                });
                wake_up(&sort_wake);
            }))),
        ));
    }
    bar
}

/// The display order: the rows as seeded while no column sorts, the
/// sorted order otherwise — numeric cells compare by value ("2" before
/// "10"), text cells by their string. The rows keep their own indices:
/// the LIS *moves* the survivors instead of rewriting them (§6.2's
/// identity rule at the sorted tier).
fn sorted_display(
    data: &[Vec<String>],
    sort_state: Option<(usize, alacer_core::SortDirection)>,
) -> Vec<(usize, &Vec<String>)> {
    let mut display: Vec<(usize, &Vec<String>)> = data.iter().enumerate().collect();
    if let Some((column, direction)) = sort_state {
        let descending = direction == alacer_core::SortDirection::Descending;
        let numeric = data.iter().all(|row| {
            row.get(column)
                .is_some_and(|cell| cell.trim().parse::<u64>().is_ok())
        });
        display.sort_by(|a, b| {
            let order = match (a.1.get(column), b.1.get(column)) {
                (Some(a_cell), Some(b_cell)) if numeric => {
                    let a_number = a_cell.trim().parse::<u64>().unwrap_or(0);
                    let b_number = b_cell.trim().parse::<u64>().unwrap_or(0);
                    a_number.cmp(&b_number)
                }
                (Some(a_cell), Some(b_cell)) => a_cell.cmp(b_cell),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            };
            if descending { order.reverse() } else { order }
        });
    }
    display
}

/// One data row: its cells at the *display* grid positions (the geometry
/// the AT walks, renumbered under the sort) and the row's `selected`
/// word, always present — `false` states the absence so the patch can
/// clear the `true` it set before (M108's tier at the view edition).
fn data_table_row(
    display_index: usize,
    original_index: usize,
    cells: &[String],
    columns: usize,
    picked: Option<usize>,
) -> ViewNode {
    let cells_view: Vec<ViewNode> = cells
        .iter()
        .take(columns)
        .enumerate()
        .map(|(column, text)| {
            ViewNode::element("table_cell")
                .prop("text", text.clone())
                .prop("row", usize_to_f32(display_index))
                .prop("column", usize_to_f32(column))
        })
        .collect();
    ViewNode::element("table_row")
        .key(u64::try_from(original_index).unwrap_or(u64::MAX - 1))
        .prop("gap", 8.0_f32)
        .prop("selected", picked == Some(original_index))
        .children(cells_view)
}

/// The selection bar: one button per row, its label naming the row's own
/// first cell — pressing selects the row (pressing the selected row's
/// button clears the selection), and the row announces the `selected`
/// word beside it. The buttons live outside the grid: the table's
/// declared geometry stays the cells' own (the frozen contract the exit
/// suite pins), the controls the toolbar's neighbors.
fn data_table_selection_bar(
    selected: &FluxRwSignal<Option<usize>>,
    wake: &WakeUp,
    picked: Option<usize>,
    display: &[(usize, &Vec<String>)],
) -> ViewNode {
    let mut bar = ViewNode::element("row").prop("gap", 8.0_f32);
    for (original_index, cells) in display {
        let select_signal = *selected;
        let select_wake = wake.clone();
        let row_name = cells.first().cloned().unwrap_or_default();
        let word = if picked == Some(*original_index) {
            format!("Deselect {row_name}")
        } else {
            format!("Select {row_name}")
        };
        let index = *original_index;
        bar = bar.child(ViewNode::element("button").prop("text", word).prop(
            "on_click",
            PropValue::Handler(Rc::new(RefCell::new(move || {
                Flux::batch_of(|| {
                    let current = select_signal.try_get_untracked().unwrap_or_default();
                    let next = if current == Some(index) {
                        None
                    } else {
                        Some(index)
                    };
                    select_signal.set(next);
                });
                wake_up(&select_wake);
            }))),
        ));
    }
    bar
}

/// The sort cycle's next state: first ask ascending, second ask
/// descending, third ask clears; a different column starts its own cycle
/// at ascending. The door and the sort bar's handler share the one fold.
fn next_sort_state(
    current: Option<(usize, alacer_core::SortDirection)>,
    column: usize,
) -> Option<(usize, alacer_core::SortDirection)> {
    match current {
        Some((current_column, alacer_core::SortDirection::Ascending))
            if current_column == column =>
        {
            Some((column, alacer_core::SortDirection::Descending))
        }
        Some((current_column, _)) if current_column == column => None,
        _ => Some((column, alacer_core::SortDirection::Ascending)),
    }
}

/// The header's sort word for `column` — `"ascending"`/`"descending"`
/// while it sorts, `"none"` otherwise (the word the a11y sink maps onto
/// AccessKit's `SortDirection`, M112).
fn sort_word_name(
    sort_state: Option<(usize, alacer_core::SortDirection)>,
    column: usize,
) -> &'static str {
    match sort_state {
        Some((sorted, alacer_core::SortDirection::Ascending)) if sorted == column => "ascending",
        Some((sorted, _)) if sorted == column => "descending",
        _ => "none",
    }
}

/// One deterministic generated row (the "Add row" seam's content).
fn generated_row(index: usize) -> Vec<String> {
    let letter = char::from(b'A' + u8::try_from(index % 26).unwrap_or(25));
    let shelf = 1 + (index % 9);
    vec![
        format!("Part {index}"),
        (1 + (index * 3) % 97).to_string(),
        format!("{letter}{shelf}"),
    ]
}

/// `usize` → `f32` for view props (the documented cast-seam shape; every
/// prop number is far below 2^24).
fn usize_to_f32(value: usize) -> f32 {
    alacer_core::num::u32_to_f32(u32::try_from(value).unwrap_or(u32::MAX))
}

/// Assembles the data-table canon onto a pipeline: `rows` seeds the initial
/// data; the headers are the canon's three columns (Item, Quantity,
/// Location).
///
/// # Panics
///
/// Panics when the view names an unregistered element (a registry bug, not
/// a runtime condition).
#[must_use]
pub fn data_table_app(
    pipeline: &mut PassPipeline,
    rows: Vec<Vec<String>>,
    wake: WakeUp,
) -> DataTableHandles {
    let rows_signal = pipeline.flux().signal(rows);
    let selected = pipeline.flux().signal(Option::<usize>::None);
    let sort = pipeline
        .flux()
        .signal(Option::<(usize, alacer_core::SortDirection)>::None);
    let headers: Vec<String> = vec![
        "Item".to_string(),
        "Quantity".to_string(),
        "Location".to_string(),
    ];

    let view = data_table_view(&rows_signal, &selected, &sort, &headers, &wake);
    let root = ViewRoot::mount(&view, pipeline, crate::default_registry());
    let root_node = root.root_id();

    DataTableHandles {
        rows: rows_signal,
        selected,
        sort,
        root,
        root_node,
        headers,
        wake,
    }
}

#[cfg(test)]
mod tests {
    use super::{DataTableHandles, TodoFilter, TodoHandles, data_table_app, todo_app};
    use alacer_core::Role;
    use alacer_core::arena::{Arena, NodeId};
    use alacer_core::geom::Size;
    use alacer_core::mask::Change;
    use alacer_core::passes::{
        A11yDelta, A11ySink, LayoutEngine, LayoutError, PassPipeline, SceneJournal, TreeSnapshot,
    };
    use rustc_hash::FxHashMap;
    use std::sync::Arc;

    /// A no-op layout engine: the canon tests assert the *a11y metadata*
    /// (roles, names, toggled state), which the snapshot builds without
    /// bounds. The full-pipeline integration lives in the M6 exit suite.
    struct ProbeLayout;
    impl LayoutEngine for ProbeLayout {
        fn sync_and_layout(
            &mut self,
            _arena: &mut Arena,
            _root: NodeId,
            _dirty: &FxHashMap<NodeId, Change>,
            _viewport: Size,
        ) -> Result<(), LayoutError> {
            Ok(())
        }
        fn measure_calls(&self) -> u64 {
            0
        }
        fn reset_measure_calls(&mut self) {}
    }

    struct ProbeJournal;
    impl SceneJournal for ProbeJournal {
        fn commit_layer(&mut self, _: NodeId, _: Vec<alacer_core::scene::Primitive>) {}
        fn drop_layer(&mut self, _: NodeId) {}
        fn layer(&self, _: NodeId) -> Option<Vec<alacer_core::scene::Primitive>> {
            None
        }
        fn snapshot(&self, _: &[NodeId]) -> alacer_core::scene::SceneSnapshot {
            alacer_core::scene::SceneSnapshot::default()
        }
        fn retain(&mut self, _: &[NodeId]) {}
        fn clear(&mut self) {}
        fn stats(&self) -> alacer_core::passes::JournalStats {
            alacer_core::passes::JournalStats::default()
        }
    }

    /// A recording sink: keeps the last resolved tree for queries.
    #[derive(Default)]
    struct RecordingSink {
        last: Option<Arc<TreeSnapshot>>,
    }
    impl A11ySink for RecordingSink {
        fn apply(&mut self, snapshot: Arc<TreeSnapshot>, _delta: &A11yDelta) {
            self.last = Some(snapshot);
        }
        fn resolved_tree(&self) -> Option<&TreeSnapshot> {
            self.last.as_deref()
        }
    }

    fn probe_pipeline() -> PassPipeline {
        PassPipeline::new(
            alacer_core::Flux::new(),
            Box::new(ProbeLayout),
            Box::new(ProbeJournal),
            Box::new(RecordingSink::default()),
        )
    }

    /// A probe text seam: deterministic metrics, no glyphs — the canon
    /// widgets paint and measure through the seam, and the widget layer
    /// cannot reach `alacer-text` (the DAG), so tests install this stub.
    struct ProbeSeam;
    impl alacer_core::text_seam::TextSeam for ProbeSeam {
        fn measure(
            &self,
            spec: &alacer_core::style::TextSpec,
            max_width: Option<f32>,
        ) -> alacer_core::geom::Size {
            let per_char = 7.0;
            let width = alacer_core::num::u32_to_f32(
                u32::try_from(spec.text.chars().count()).unwrap_or(u32::MAX),
            ) * per_char;
            let wrapped = max_width.is_some_and(|max| width > max && max > 0.0);
            let lines = if wrapped { 2.0 } else { 1.0 };
            alacer_core::geom::Size::new(width.min(max_width.unwrap_or(width)), 16.0 * lines)
        }

        fn layout_glyphs(
            &self,
            _text: &str,
            _font_size: f32,
            _max_width: Option<f32>,
        ) -> alacer_core::text_seam::GlyphRun {
            alacer_core::text_seam::GlyphRun {
                width: 0.0,
                height: 16.0,
                glyphs: Vec::new(),
            }
        }

        /// The probe's segmentation: alphanumeric runs are word-like,
        /// everything else separates — deterministic for the ASCII probe
        /// values; the real tables ride the engine (alacer-text).
        fn word_segments(&self, text: &str) -> Vec<alacer_core::text_seam::WordSegment> {
            let mut segments = Vec::new();
            let (mut start, mut word_like, mut units) = (0_usize, false, 0_usize);
            for ch in text.chars() {
                let like = ch.is_alphanumeric();
                if units > 0 && like != word_like {
                    segments.push(alacer_core::text_seam::WordSegment {
                        start,
                        end: units,
                        word_like,
                    });
                    start = units;
                }
                word_like = like;
                units += ch.len_utf16();
            }
            if units > start {
                segments.push(alacer_core::text_seam::WordSegment {
                    start,
                    end: units,
                    word_like,
                });
            }
            segments
        }
    }

    fn tick(pipeline: &mut PassPipeline) {
        // The canon tests run before the widget paint pass, which reaches
        // text through the seam — install the probe stub (the real seam is
        // `alacer_text::ThreadLocalTextSeam` at assembly sites).
        if !alacer_core::text_seam::is_installed() {
            alacer_core::text_seam::install(std::rc::Rc::new(ProbeSeam));
        }
        pipeline
            .tick(Size::new(400.0, 400.0))
            .expect("probe layout always succeeds");
    }

    #[test]
    fn todo_canon_adds_toggles_and_counts() {
        let mut pipeline = probe_pipeline();
        let mut handles: TodoHandles = todo_app(&mut pipeline, None);
        handles.add("write docs");
        handles.add("ship it");
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);

        assert_eq!(handles.remaining(), 2);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        assert!(snapshot.contains(Role::CheckBox, "write docs"));
        assert!(snapshot.contains(Role::CheckBox, "ship it"));
        assert!(snapshot.contains(Role::Button, "Delete write docs"));
        let checkbox = snapshot
            .find(Role::CheckBox, "write docs")
            .expect("row box");
        assert_eq!(
            checkbox.toggled,
            Some(alacer_core::ToggleState::Off),
            "unchecked at creation"
        );

        // Toggle + re-render: the checked state re-announces.
        handles.toggle(0);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        assert_eq!(handles.remaining(), 1, "one item done");
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        assert_eq!(
            snapshot
                .find(Role::CheckBox, "write docs")
                .expect("row box")
                .toggled,
            Some(alacer_core::ToggleState::On),
            "the checkbox's a11y state follows the signal"
        );
    }

    #[test]
    fn todo_canon_removes_rows() {
        let mut pipeline = probe_pipeline();
        let mut handles = todo_app(&mut pipeline, None);
        handles.add("a");
        handles.add("b");
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        handles.remove(0);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        handles.with_todos(|todos| {
            assert_eq!(todos.len(), 1);
            assert_eq!(todos[0].label, "b");
        });
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        assert!(
            !snapshot.contains(Role::CheckBox, "a"),
            "row gone from a11y"
        );
        assert!(snapshot.contains(Role::CheckBox, "b"));
    }

    #[test]
    fn todo_add_ignores_blank_labels() {
        let mut pipeline = probe_pipeline();
        let mut handles = todo_app(&mut pipeline, None);
        handles.add("   ");
        let _ = handles.rerender(&mut pipeline);
        handles.with_todos(|todos| assert!(todos.is_empty(), "blank labels never mount"));
    }

    #[test]
    fn data_table_canon_mounts_a_grid() {
        let mut pipeline = probe_pipeline();
        let mut handles: DataTableHandles = data_table_app(
            &mut pipeline,
            vec![
                vec!["Nails".into(), "12".into(), "A1".into()],
                vec!["Screws".into(), "5".into(), "B2".into()],
            ],
            None,
        );
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);

        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        let table = snapshot.find(Role::Table, "Inventory").expect("the table");
        assert_eq!(
            table.table_slot,
            Some(alacer_core::TableSlot::Table {
                rows: 2,
                columns: 3
            }),
            "the table's declared geometry"
        );
        assert!(snapshot.contains(Role::ColumnHeader, "Quantity"));
        let cell = snapshot.find(Role::TableCell, "Nails").expect("cell");
        assert_eq!(
            cell.table_slot,
            Some(alacer_core::TableSlot::Cell { row: 0, column: 0 })
        );

        // Push a row: the geometry updates with the re-render.
        handles.push_row(vec!["Bolts".into(), "3".into(), "C3".into()]);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        let table = snapshot.find(Role::Table, "Inventory").expect("table");
        assert_eq!(
            table.table_slot,
            Some(alacer_core::TableSlot::Table {
                rows: 3,
                columns: 3
            })
        );
        let cell = snapshot.find(Role::TableCell, "Bolts").expect("new cell");
        assert_eq!(
            cell.table_slot,
            Some(alacer_core::TableSlot::Cell { row: 2, column: 0 })
        );

        // Remove a row: cells renumber.
        handles.remove_row(0);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        let cell = snapshot
            .find(Role::TableCell, "Screws")
            .expect("shifted cell");
        assert_eq!(
            cell.table_slot,
            Some(alacer_core::TableSlot::Cell { row: 0, column: 0 }),
            "cells renumber after removal"
        );
    }

    #[test]
    fn todo_canon_filters_rows_and_clears_completed() {
        let mut pipeline = probe_pipeline();
        let mut handles = todo_app(&mut pipeline, None);
        handles.add("write docs");
        handles.add("ship it");
        handles.toggle(0);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);

        // The Active filter: the open item only — the done row leaves the
        // tree (a filtered-out row is not an invisible one), the footer
        // binding still counts everything.
        handles.set_filter(TodoFilter::Active);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        assert!(
            !snapshot.contains(Role::CheckBox, "write docs"),
            "the done row is outside the Active filter"
        );
        assert!(
            snapshot.contains(Role::CheckBox, "ship it"),
            "the open row shows"
        );
        assert_eq!(handles.remaining(), 1, "the footer counts the full list");

        // The Completed filter mirrors it.
        handles.set_filter(TodoFilter::Completed);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        assert!(snapshot.contains(Role::CheckBox, "write docs"));
        assert!(!snapshot.contains(Role::CheckBox, "ship it"));

        // Clear completed: the done item leaves the list entirely (one
        // write batch), and the Completed filter's view empties with it.
        handles.clear_completed();
        handles.with_todos(|todos| assert_eq!(todos.len(), 1, "the done item is gone"));
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        assert!(
            !snapshot.contains(Role::CheckBox, "write docs"),
            "the cleared row is gone from the tree"
        );

        // The no-op clear prices nothing: the list is where the write
        // lands, the signal keeps its value.
        handles.clear_completed();
        handles.with_todos(|todos| assert_eq!(todos.len(), 1));

        // Back to All: the survivors show.
        handles.set_filter(TodoFilter::All);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        assert!(snapshot.contains(Role::CheckBox, "ship it"));
    }

    #[test]
    fn todo_canon_toggle_works_under_a_filter() {
        let mut pipeline = probe_pipeline();
        let mut handles = todo_app(&mut pipeline, None);
        handles.add("a");
        handles.add("b");
        handles.set_filter(TodoFilter::Active);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        // Both rows show under Active; toggling the second one lands on
        // the right item — the handlers carry the list index, not the
        // display one.
        handles.toggle(1);
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        assert!(
            !snapshot.contains(Role::CheckBox, "b"),
            "the toggled row left the Active filter"
        );
        assert!(snapshot.contains(Role::CheckBox, "a"));
        handles.with_todos(|todos| {
            assert!(!todos[0].done && todos[1].done, "the right item toggled");
        });
    }

    #[test]
    fn data_table_canon_sorts_selects_and_renumbers() {
        let mut pipeline = probe_pipeline();
        let mut handles: DataTableHandles = data_table_app(
            &mut pipeline,
            vec![
                vec!["Nails".into(), "12".into(), "A1".into()],
                vec!["Screws".into(), "5".into(), "B2".into()],
            ],
            None,
        );
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);

        // Selection: the row's `selected` word rides the snapshot.
        handles.select_row(Some(1));
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        let selected_rows = snapshot
            .nodes
            .iter()
            .filter(|node| node.selected == Some(true))
            .count();
        assert_eq!(selected_rows, 1, "one row holds the selection");

        // Sort by Quantity ascending: the numeric order — 5 before 12 —
        // with the cells' *display* geometry renumbered and the rows'
        // identity kept by their keys.
        handles.sort_by(1);
        assert_eq!(
            handles.sort_state(),
            Some((1, alacer_core::SortDirection::Ascending))
        );
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        let screws = snapshot
            .find(Role::TableCell, "Screws")
            .expect("the lighter row first");
        assert_eq!(
            screws.table_slot,
            Some(alacer_core::TableSlot::Cell { row: 0, column: 0 }),
            "5 sorts before 12 — the display geometry renumbers"
        );
        let nails = snapshot
            .find(Role::TableCell, "Nails")
            .expect("the heavier row second");
        assert_eq!(
            nails.table_slot,
            Some(alacer_core::TableSlot::Cell { row: 1, column: 0 })
        );
        let header = snapshot
            .find(Role::ColumnHeader, "Quantity")
            .expect("the sorted header");
        assert_eq!(
            header.sort_direction,
            Some(alacer_core::SortDirection::Ascending),
            "the header announces the sort word"
        );
        let other_header = snapshot
            .find(Role::ColumnHeader, "Item")
            .expect("the unsorted header");
        assert_eq!(
            other_header.sort_direction, None,
            "the unsorted header announces no sort"
        );

        // The second ask descending, the third clears — the cycle a
        // column's own control rides.
        handles.sort_by(1);
        assert_eq!(
            handles.sort_state(),
            Some((1, alacer_core::SortDirection::Descending))
        );
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        let nails = snapshot
            .find(Role::TableCell, "Nails")
            .expect("descending flips the order");
        assert_eq!(
            nails.table_slot,
            Some(alacer_core::TableSlot::Cell { row: 0, column: 0 })
        );
        handles.sort_by(1);
        assert_eq!(handles.sort_state(), None, "the third ask clears");
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
        let snapshot = pipeline.a11y_snapshot().expect("a11y tree built");
        let nails = snapshot
            .find(Role::TableCell, "Nails")
            .expect("the seeded order restored");
        assert_eq!(
            nails.table_slot,
            Some(alacer_core::TableSlot::Cell { row: 0, column: 0 }),
            "no sort: the grid keeps its own order"
        );

        // Removing the selected row clears the selection beside it: a
        // dead row holds no selection.
        handles.select_row(Some(0));
        handles.remove_row(0);
        assert_eq!(
            handles.selected_row(),
            None,
            "the selection followed the row out"
        );
        let _ = handles.rerender(&mut pipeline);
        tick(&mut pipeline);
    }
}
