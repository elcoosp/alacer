//! The a11y-update ratio benchmark (§11.1, ADR-0015): the pre-M9 windowed
//! shape rebuilt a full `TreeUpdate` from the snapshot on every a11y frame
//! (and dropped the M6 vocabulary doing it); the M9 shape drains the
//! pipeline sink's dirty-node-only delta. The pair is published as the
//! `a11y_incremental_speedup` budget — the incremental contract as a
//! number, same-host-comparable through the regression gates.

use alacer_core::Role;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::passes::{A11yDelta, A11ySink, TreeNode, TreeSnapshot};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;

const NODES: u64 = 1_000;
/// 10% of the tree dirty per frame — the churn the W1 workload set as the
/// house baseline for "steady state" (ADR-0012).
const DIRTY: u64 = 100;

fn snapshot_1k() -> Arc<TreeSnapshot> {
    let capacity = usize::try_from(NODES).unwrap_or_default();
    let mut nodes = Vec::with_capacity(capacity);
    nodes.push(TreeNode {
        id: 0,
        role: Role::Container,
        label: "app".into(),
        bounds: Some(Rect::new(Point::zero(), Size::new(320.0, 480.0))),
        children: (1..NODES).collect(),
        value: None,
        toggled: None,
        table_slot: None,
        selection: None,
        live: None,
    });
    for id in 1..NODES {
        nodes.push(TreeNode {
            id,
            role: Role::Label,
            label: format!("Item {id}"),
            bounds: Some(Rect::new(
                Point::new(0.0, alacer_core::num::u64_to_f32(id) * 16.0),
                Size::new(320.0, 14.0),
            )),
            children: Vec::new(),
            value: None,
            toggled: None,
            table_slot: None,
            selection: None,
            live: None,
        });
    }
    Arc::new(TreeSnapshot {
        nodes,
        root: Some(0),
    })
}

fn bench_tree_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("a11y");
    let snapshot = snapshot_1k();
    let dirty: Vec<u64> = (1..=DIRTY).collect();

    // The pre-M9 shape: one full-tree `TreeUpdate` per a11y frame — every
    // node materialized, every frame, AT connected or not.
    group.bench_function("full_rebuild_1k", |b| {
        let mut sink = alacer_a11y::AccessKitSink::new();
        b.iter(|| {
            sink.apply(
                black_box(Arc::clone(&snapshot)),
                &A11yDelta {
                    initializes: true,
                    ..A11yDelta::empty()
                },
            );
        });
    });

    // The M9 shape: the sink's incremental delta — dirty nodes only, the
    // tree structure untouched, focus riding along.
    group.bench_function("incremental_1k_dirty_10pct", |b| {
        let mut sink = alacer_a11y::AccessKitSink::new();
        sink.apply(
            Arc::clone(&snapshot),
            &A11yDelta {
                initializes: true,
                ..A11yDelta::empty()
            },
        );
        b.iter(|| {
            let mut delta = A11yDelta::empty();
            delta
                .updated
                .extend(black_box(dirty.as_slice()).iter().copied());
            sink.apply(black_box(Arc::clone(&snapshot)), &delta);
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench_tree_update
);
criterion_main!(benches);
