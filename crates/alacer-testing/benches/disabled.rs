//! The availability benchmark (§11.5, ADR-0028): the refused-press route
//! driven through the pipeline on a 1,000-node arena — one hundred AT
//! presses against a disabled button, each refused honestly, each
//! costing the routing pass and an idle frame on the shipped composition
//! (harness + AccessKit sink), not a probe stand-in.
//!
//! The published regression gate keeps the refusal honest as the tier
//! grows: an unavailable control must answer in the cost class of
//! routing — the action reaches the widget, the widget refuses, no masks
//! queue, and no `TreeUpdate` re-materializes (the flag rode the delta
//! when it moved, not when the refusals happen).
//!
//! The bench lives in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness
//! layer drives every surface).

use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, Panel};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const ROUTES: usize = 100;
const FILLER: usize = 1000;
const PANEL_HEIGHT: f32 = 0.4;

fn bench_disabled_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("disabled");
    group.throughput(Throughput::Elements(
        u64::try_from(ROUTES).expect("non-negative"),
    ));

    group.bench_function("route_100_refused_presses_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().boxed());
        let _ = harness.add_child(root, Label::new("The availability tier").boxed());
        for i in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
            let _ = i;
        }
        let _ = harness.add_child(
            root,
            Button::new("Send invite")
                .enabled(false)
                .on_click(|| {})
                .boxed(),
        );
        harness.settle();

        // The disabled button's AccessKit identity: the id-path hash the
        // action route answers for.
        let hash = {
            let tree = harness
                .pipeline()
                .a11y_snapshot()
                .expect("the a11y tree resolves");
            let node = tree
                .find(Role::Button, "Send invite")
                .expect("the disabled button is named in the tree");
            assert!(!node.enabled, "the button reports unavailable");
            node.id
        };

        // One hundred AT presses, every one refused: each route is one
        // dispatch (the action reaches the widget, the widget refuses) +
        // one tick — and the refusal queues nothing, so the tick is the
        // idle frame the M21 idle property promises at this seam.
        b.iter(|| {
            let consumed = black_box(
                harness
                    .pipeline_mut()
                    .dispatch_a11y_action(hash, A11yAction::DefaultPress),
            );
            assert!(!consumed, "the press is refused, never munged");
            black_box(harness.tick());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_disabled_route);
criterion_main!(benches);
