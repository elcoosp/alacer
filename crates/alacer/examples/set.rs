//! The set tier as a consumer's vocabulary (M119, ADR-0123): the
//! member's place and the collection's size said by the a11y tree at
//! last — AccessKit's `PositionInSet` property (ARIA's
//! `aria-posinset`) beside AccessKit's `SizeOfSet` property (ARIA's
//! `aria-setsize`), the two numbers every platform AT turns into
//! "row 3 of 500" while it walks a collection. One sized collection
//! beside two honest absences: the ledger declaring its span and its
//! rows' places, the archive that declares neither, and the plain
//! widgets the tier keeps honest.
//!
//! The beat the example exists to show: the words price the words and
//! nothing else — a place and a span name facts, they wall nothing
//! (the rows and the table stay put, every door answers as ever), and
//! the app's own doors (`TableRow::set_position_in_set` /
//! `Table::set_size_of_set` through the mutate seam) move the words
//! live — the ledger grows a row, the archive starts its own count,
//! the `PAINT | A11Y` request pays the re-announcement — the write
//! that lands where a word already is queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example set
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, Table, TableCell, TableRow};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::PassPipeline;
use alacer_core::widget::Role;
use reactive_graph::traits::Set as _;

/// The viewport the demo's beats settle into (the editing canon's shape).
const VIEWPORT_WIDTH: f32 = 480.0;
const VIEWPORT_HEIGHT: f32 = 360.0;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let Ok(report) = pipeline.tick(alacer_core::geom::Size::new(
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
        )) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// The collection facts the tree says, read back through the snapshot —
/// the same answer the sink maps onto AccessKit's `PositionInSet` /
/// `SizeOfSet` properties: "row 3 of 500" beside the name.
fn set_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let mut facts: Vec<String> = Vec::new();
    if let Some(t) = tree {
        for node in &t.nodes {
            if node.role == Role::Table {
                match node.size_of_set {
                    Some(size) => facts.push(format!("{} ({size} rows)", node.label)),
                    None => facts.push(format!("{} (unsized)", node.label)),
                }
            }
        }
    }
    facts.join(" · ")
}

/// The app's set doors: the mutate seam the app's code rides — a word
/// moved, `PAINT | A11Y` requested on the row or the table, the
/// re-announcement paid for. `None` clears a word.
fn move_set_words(
    pipeline: &mut PassPipeline,
    row: alacer_core::NodeId,
    table: alacer_core::NodeId,
    place: Option<usize>,
    size: Option<usize>,
) {
    let mut guard = pipeline.arena_mut().mutate(row);
    if let Some(row) = guard.widget_mut().as_any_mut().downcast_mut::<TableRow>()
        && row.set_position_in_set(place)
    {
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    let mut guard = pipeline.arena_mut().mutate(table);
    if let Some(table) = guard.widget_mut().as_any_mut().downcast_mut::<Table>()
        && table.set_size_of_set(size)
    {
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The set tier: the member's place and the collection's size (ADR-0123)").boxed(),
    );

    // The collection facts: the place and the span ride the tree from
    // the first frame — the ledger declaring 500 rows and its first
    // placed entries, the archive and the plain widgets the honest
    // absences between them.
    let ledger = pipeline.add_child(root, Table::new("Ledger", 2).size_of_set(500).boxed());
    let first = pipeline.add_child(ledger, TableRow::new().position_in_set(0).boxed());
    let _ = pipeline.add_child(first, TableCell::new("Entry — placed 0", 0, 0).boxed());
    let second = pipeline.add_child(ledger, TableRow::new().position_in_set(1).boxed());
    let _ = pipeline.add_child(second, TableCell::new("Entry — placed 1", 1, 0).boxed());
    let archive = pipeline.add_child(root, Table::new("Archive", 2).boxed());
    let _ = pipeline.add_child(archive, TableCell::new("Entry — unsized", 0, 0).boxed());
    let _ = pipeline.add_child(root, Label::new("A plain label: no place, no span").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the facts the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = set_words(snapshot.as_ref());
    let status_signal = pipeline.flux_mut().signal(line.clone());
    let status_binding = RenderBinding::new(line);
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );
    tracing::info!(
        target: "alacer::set",
        facts = "Ledger 500, placed 0, placed 1",
        words = "position_in_set, size_of_set",
        "beat 1: the collection facts ride the tree — row 0 of 500 beside the honest absences"
    );

    // Beat two: the live doors — the ledger's third row takes its
    // place, the archive starts its own count, the words the tree says
    // following the app's own; then the absence restored (the archive's
    // count withdrawn, the row left the set).
    let third = pipeline.add_child(ledger, TableRow::new().boxed());
    let _ = pipeline.add_child(third, TableCell::new("Entry — placed 2", 2, 0).boxed());
    move_set_words(&mut pipeline, third, archive, Some(2), Some(120));
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the live doors moved the words",
        set_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::set",
        moved = "third row → placed 2, Archive → 120 rows",
        words = "position_in_set, size_of_set",
        "beat 2: the app's doors moved the words the tree says — the collection follows the app's own"
    );
    move_set_words(&mut pipeline, third, archive, None, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the doors cleared the words",
        set_words(snapshot.as_ref()),
    ));

    // The window: live play — the worded collections say their facts
    // beside the honest absences, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the collections say the facts — every door answers as it always did")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer set",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
