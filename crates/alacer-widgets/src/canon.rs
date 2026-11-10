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

/// The todo canon's assembly: signal, footer binding, and the mounted view.
pub struct TodoHandles {
    /// The todos signal — one write batch per mutation (ADR-6).
    pub todos: FluxRwSignal<Vec<TodoItem>>,
    /// The footer binding: "N items left", updated by the flux bridge.
    pub footer: RenderBinding<String>,
    root: ViewRoot,
    footer_node: alacer_core::NodeId,
    wake: WakeUp,
}

impl TodoHandles {
    /// The scoped re-render: rebuilds the view from the current signal
    /// state and patches the retained tree in place (keyed rows, LIS diff).
    /// One call per write batch — the ADR-6 contract's view edition.
    pub fn rerender(&mut self, pipeline: &mut PassPipeline) -> PatchReport {
        let view = todo_view(&self.todos, &self.footer, &self.wake);
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

    /// The footer label's arena node (the binding's target; assembly-site
    /// introspection).
    #[must_use]
    pub const fn footer_node(&self) -> alacer_core::NodeId {
        self.footer_node
    }

    /// The mounted root's node.
    #[must_use]
    pub const fn root_node(&self) -> alacer_core::NodeId {
        self.root.root_id()
    }
}

/// Builds the todo view for the current signal state: input, keyed rows
/// (checkbox + label + delete), and the bound footer.
fn todo_view(
    todos: &FluxRwSignal<Vec<TodoItem>>,
    footer: &RenderBinding<String>,
    wake: &WakeUp,
) -> ViewNode {
    let items = todos.try_get_untracked().unwrap_or_default();
    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 12.0_f32);

    // The input: Enter commits — the §5 snippet's `on_submit` seam (M6).
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

    // The rows: keyed by index so the LIS keeps survivors in place and
    // mounts/removes exactly the difference (§6.2).
    let rows: Vec<ViewNode> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
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
        })
        .collect();

    root = root.child(
        ViewNode::element("column")
            .prop("gap", 8.0_f32)
            .children(rows),
    );
    root = root.child(ViewNode::element("label").prop("binding", footer.clone()));
    root
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
    let footer = RenderBinding::new("0 items left".to_string());

    let view = todo_view(&todos, &footer, &wake);
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
        root,
        footer_node,
        wake,
    }
}

/// The data-table canon's assembly: a toolbar (the "Add row" seam), the
/// table, the rows signal, and the mounted view.
pub struct DataTableHandles {
    /// The row data — each row is `columns` cell strings. One write batch
    /// per mutation (ADR-6).
    pub rows: FluxRwSignal<Vec<Vec<String>>>,
    root: ViewRoot,
    root_node: alacer_core::NodeId,
    /// The header labels (column names).
    headers: Vec<String>,
    wake: WakeUp,
}

impl DataTableHandles {
    /// The scoped re-render (same contract as [`TodoHandles::rerender`]).
    pub fn rerender(&mut self, pipeline: &mut PassPipeline) -> PatchReport {
        let view = data_table_view(&self.rows, &self.headers, &self.wake);
        self.root.update(&view, pipeline)
    }

    /// Reads the current rows (test introspection).
    ///
    /// # Panics
    ///
    /// Panics when the signal's owner is gone (a dropped flux — an app bug).
    pub fn with_rows<R>(&self, f: impl FnOnce(&[Vec<String>]) -> R) -> R {
        let rows = self
            .rows
            .try_get_untracked()
            .expect("the rows signal's owner is alive");
        f(&rows)
    }

    /// Appends one row — one write batch.
    pub fn push_row(&self, cells: Vec<String>) {
        Flux::batch_of(|| {
            let mut next = self.rows.try_get_untracked().unwrap_or_default();
            next.push(cells);
            self.rows.set(next);
        });
    }

    /// Removes one row — one write batch.
    pub fn remove_row(&self, index: usize) {
        Flux::batch_of(|| {
            let mut next = self.rows.try_get_untracked().unwrap_or_default();
            if index < next.len() {
                next.remove(index);
            }
            self.rows.set(next);
        });
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

/// Builds the data-table view: the toolbar (the "Add row" seam), the
/// header row (anchored by the sentinel key), and keyed data rows whose
/// cells carry their grid positions.
fn data_table_view(
    rows: &FluxRwSignal<Vec<Vec<String>>>,
    headers: &[String],
    wake: &WakeUp,
) -> ViewNode {
    let data = rows.try_get_untracked().unwrap_or_default();
    let columns = headers.len();

    // The toolbar: "Add row" appends a generated row — one write batch,
    // then the wake. The button rides the same a11y contract as every
    // control (named, actionable).
    let add_rows = *rows;
    let add_wake = wake.clone();
    let add_button = ViewNode::element("button").prop("text", "Add row").prop(
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
    );

    let header_cells: Vec<ViewNode> = headers
        .iter()
        .enumerate()
        .map(|(column, name)| {
            ViewNode::element("column_header")
                .prop("text", name.clone())
                .prop("column", usize_to_f32(column))
        })
        .collect();
    let header_row = ViewNode::element("table_row")
        // The header sentinel key: the LIS keeps it anchored first while the
        // data rows around it diff (keys are unique: u64::MAX vs 0..n).
        .key(u64::MAX)
        .prop("gap", 8.0_f32)
        .children(header_cells);

    let data_rows: Vec<ViewNode> = data
        .iter()
        .enumerate()
        .map(|(row_index, cells)| {
            let cells_view: Vec<ViewNode> = cells
                .iter()
                .take(columns)
                .enumerate()
                .map(|(column, text)| {
                    ViewNode::element("table_cell")
                        .prop("text", text.clone())
                        .prop("row", usize_to_f32(row_index))
                        .prop("column", usize_to_f32(column))
                })
                .collect();
            ViewNode::element("table_row")
                .key(u64::try_from(row_index).unwrap_or(u64::MAX - 1))
                .prop("gap", 8.0_f32)
                .children(cells_view)
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
        .child(
            ViewNode::element("row")
                .prop("gap", 8.0_f32)
                .child(add_button),
        )
        .child(table)
}

/// One deterministic generated row (the "Add row" seam's content).
fn generated_row(index: usize) -> Vec<String> {
    let letter = char::from(b'A' + u8::try_from(index % 26).unwrap_or(25));
    let shelf = 1 + (index % 9);
    vec![
        format!("Part {index}"),
        format!("{}", 1 + (index * 3) % 97),
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
    let headers: Vec<String> = vec![
        "Item".to_string(),
        "Quantity".to_string(),
        "Location".to_string(),
    ];

    let view = data_table_view(&rows_signal, &headers, &wake);
    let root = ViewRoot::mount(&view, pipeline, crate::default_registry());
    let root_node = root.root_id();

    DataTableHandles {
        rows: rows_signal,
        root,
        root_node,
        headers,
        wake,
    }
}

#[cfg(test)]
mod tests {
    use super::{DataTableHandles, TodoHandles, data_table_app, todo_app};
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
}
