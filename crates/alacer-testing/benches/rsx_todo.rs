//! The §13.1 showcase benchmark: the `rsx!` todo view's three prices on the
//! shipped composition (harness + AccessKit sink) — the DSL's runtime tree
//! build, the first mount, and the per-interaction repatch — the exact
//! paths `examples/todo.rs` rides.
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness layer
//! drives every surface), and because the showcase view is the consumer
//! shape: one `rsx!` tree over the default element registry, keyed rows,
//! handlers capturing their own signal copies.

use alacer_core::flux::RenderBinding;
use alacer_macro::rsx;
use alacer_testing::TestHarness;
use alacer_view::{ViewNode, ViewRoot};
use alacer_widgets::default_registry;
use alacer_widgets::{TodoFilter, TodoItem};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::rc::Rc;

/// The row count every shape builds: enough keyed rows for the LIS to
/// matter, few enough for the mount to stay the unit under test.
const ROWS: usize = 50;

/// The todo view the showcase ships: the committing input, the keyed
/// `for` rows, the footer bar — one `rsx!` tree over the current state.
fn todo_view(todos: &[TodoItem], filter: TodoFilter, footer: &RenderBinding<String>) -> ViewNode {
    let visible: Vec<(u64, usize, &TodoItem)> = todos
        .iter()
        .enumerate()
        .filter(|(_, item)| filter.shows(item))
        .map(|(index, item)| (u64::try_from(index).unwrap_or(u64::MAX), index, item))
        .collect();
    let footer = footer.clone();

    rsx! {
        column { padding: 16.0, gap: 12.0,
            text_input { name: "Add a todo", placeholder: "What needs doing?" }
            for (key, _index, item) in visible.into_iter() {
                row { key: key, gap: 8.0,
                    checkbox {
                        name: item.label.clone(),
                        checked: item.done,
                        on_toggle: alacer_view::PropValue::FlagHandler(Rc::new(
                            std::cell::RefCell::new(move |_checked: bool| {}),
                        )),
                    }
                    label { text: item.label.clone() }
                    button {
                        text: "Delete {item.label}",
                        on_click: alacer_view::PropValue::Handler(Rc::new(
                            std::cell::RefCell::new(move || {}),
                        )),
                    }
                }
            }
            label { binding: footer }
        }
    }
}

/// The bench's seed: `rows` open todos, numbered labels.
fn seed(rows: usize) -> Vec<TodoItem> {
    (0..rows)
        .map(|index| TodoItem {
            done: false,
            label: format!("todo {index}"),
        })
        .collect()
}

fn bench_rsx_todo(c: &mut Criterion) {
    let mut group = c.benchmark_group("rsx_todo");
    group.throughput(Throughput::Elements(
        u64::try_from(ROWS).expect("non-negative"),
    ));

    // The DSL's runtime price: the view function compiles the tree from
    // the current state — this is what every scoped re-render spends
    // before the reconciler sees anything.
    group.bench_function(BenchmarkId::new("build", ROWS), |b| {
        let todos = seed(ROWS);
        let footer = RenderBinding::new("0 items left".to_string());
        b.iter(|| {
            let view = todo_view(black_box(&todos), TodoFilter::All, &footer);
            black_box(view);
        });
    });

    // The first frame: build + mount over the default registry + one
    // settle — the price `examples/todo.rs` pays once at assembly.
    group.bench_function(BenchmarkId::new("mount", ROWS), |b| {
        let todos = seed(ROWS);
        let footer = RenderBinding::new("0 items left".to_string());
        b.iter(|| {
            let mut harness = TestHarness::new(360.0, 900.0);
            let view = todo_view(&todos, TodoFilter::All, &footer);
            let root = ViewRoot::mount(&view, harness.pipeline_mut(), default_registry());
            harness.settle();
            black_box(root.root_id());
        });
    });

    // The per-interaction price: one label edit lands as one scoped
    // re-render — build the swapped view, reconcile in place, settle one
    // frame. State rides the harness; nothing re-mounts.
    group.bench_function(BenchmarkId::new("repatch", ROWS), |b| {
        let mut harness = TestHarness::new(360.0, 900.0);
        let todos = seed(ROWS);
        let footer = RenderBinding::new("0 items left".to_string());
        let view = todo_view(&todos, TodoFilter::All, &footer);
        let mut root = ViewRoot::mount(&view, harness.pipeline_mut(), default_registry());
        harness.settle();
        b.iter(|| {
            let mut edited = seed(ROWS);
            edited[ROWS / 2].label = "the edited todo".to_string();
            let next = todo_view(black_box(&edited), TodoFilter::All, &footer);
            let report = root.update(&next, harness.pipeline_mut());
            harness.settle();
            black_box(report);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_rsx_todo);
criterion_main!(benches);
