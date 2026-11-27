//! The §6.2 conditional reconciliation price: the `rsx!` `if/else` branch
//! flip on the shipped composition (harness + AccessKit sink) — the
//! subtree the branch replaces mounts and unmounts whole, the same-branch
//! rebuild patches in place and queues nothing.
//!
//! The bench lives in the harness crate beside the `rsx_todo` showcase for
//! the same reasons the showcase does: the whole pipeline is assembled here
//! with the real widget set, and the shape is the consumer one — one `rsx!`
//! tree over the default element registry, keyed rows under a conditional
//! branch. ADR-0007 pre-registered the grammar (`if cond { element } else
//! { element }` — "conditional absence waits for scoped re-renders"); the
//! M136/M137 tiers mount and unmount their presence exactly this way (the
//! M1 conditional-mount precedent, architecture §18). This pair prices the
//! precedent: the flip (branch swap — the 50-row keyed subtree replaced by
//! one fallback label and back) against the idle twin (the same view
//! rebuilt from unchanged state — the reconciler patches in place and the
//! frame stays idle).

use alacer_core::flux::RenderBinding;
use alacer_macro::rsx;
use alacer_testing::TestHarness;
use alacer_view::{ViewNode, ViewRoot};
use alacer_widgets::default_registry;
use alacer_widgets::{TodoFilter, TodoItem};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::rc::Rc;

/// The row count every shape builds: the same keyed-row subtree the
/// `rsx_todo` showcase prices, so the two benches' medians stay comparable.
const ROWS: usize = 50;

/// The conditional view the bench prices: the keyed rows live under one
/// `if/else` branch — `show` mounts the list, `!show` mounts the empty
/// footer the branch falls back to. One signal in, one branch out.
fn conditional_view(
    show: bool,
    todos: &[TodoItem],
    filter: TodoFilter,
    footer: &RenderBinding<String>,
) -> ViewNode {
    let visible: Vec<(u64, usize, &TodoItem)> = todos
        .iter()
        .enumerate()
        .filter(|(_, item)| filter.shows(item))
        .map(|(index, item)| (u64::try_from(index).unwrap_or(u64::MAX), index, item))
        .collect();
    let footer = footer.clone();

    rsx! {
        column { padding: 16.0, gap: 12.0,
            if show {
                column { gap: 8.0,
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
                        }
                    }
                }
            } else {
                label { text: "Nothing to show" }
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

fn bench_rsx_conditional(c: &mut Criterion) {
    let mut group = c.benchmark_group("rsx_conditional");
    group.throughput(Throughput::Elements(
        u64::try_from(ROWS).expect("non-negative"),
    ));

    // The branch flip: each iteration swaps the conditional child — the
    // mounted 50-row keyed subtree unmounts and the fallback label mounts
    // (or the reverse, alternating). One `root.update` + one settle per
    // flip: the reconciler's subtree-replace path end to end. The harness,
    // the mount, and the first settle ride the setup, like the showcase's
    // repatch leg does.
    group.bench_function(BenchmarkId::new("flip", ROWS), |b| {
        let todos = seed(ROWS);
        let footer = RenderBinding::new("0 items left".to_string());
        let mut harness = TestHarness::new(360.0, 900.0);
        let view = conditional_view(true, &todos, TodoFilter::All, &footer);
        let mut root = ViewRoot::mount(&view, harness.pipeline_mut(), default_registry());
        harness.settle();
        let mut show = true;
        b.iter(|| {
            show = !show;
            let next = conditional_view(show, &todos, TodoFilter::All, &footer);
            let report = root.update(&next, harness.pipeline_mut());
            harness.settle();
            black_box(report);
        });
    });

    // The idle twin: the same view rebuilt from unchanged state every
    // iteration — the branch picks the same side, the reconciler patches in
    // place, and the frame stays idle (A1's rule at the view tier). The
    // rebuild is charged honestly: the DSL's own build price rides both
    // legs, so the pair's delta is the flip alone.
    group.bench_function(BenchmarkId::new("idle", ROWS), |b| {
        let todos = seed(ROWS);
        let footer = RenderBinding::new("0 items left".to_string());
        let mut harness = TestHarness::new(360.0, 900.0);
        let view = conditional_view(true, &todos, TodoFilter::All, &footer);
        let mut root = ViewRoot::mount(&view, harness.pipeline_mut(), default_registry());
        harness.settle();
        b.iter(|| {
            let next = conditional_view(true, &todos, TodoFilter::All, &footer);
            let report = root.update(&next, harness.pipeline_mut());
            harness.settle();
            black_box(report);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_rsx_conditional);
criterion_main!(benches);
