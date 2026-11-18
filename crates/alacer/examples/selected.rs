//! The selected tier as a consumer's vocabulary (M108, ADR-0112): the
//! collection's pick said by the a11y tree at last — AccessKit's
//! `Selected` property (ARIA's `aria-selected`) beside the grid
//! position the M6 slot already carried, the word every platform AT
//! turns into "selected" while the user walks the rows. A mini data
//! table beside one status line: the picked row, the row standing
//! unpicked beside it — and the wordless row beside them both, the
//! honest absence the tier keeps honest.
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the selected row's doors answer exactly as they
//! always did (the word names state, it walls nothing), and the app's
//! own door (`TableRow::set_selected` / `TableCell::set_selected`
//! through the mutate seam) moves the word live, the `PAINT | A11Y`
//! request paying the re-announcement — the write that lands where the
//! word already is queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example selected
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Table, TableCell, TableRow};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::PassPipeline;
use alacer_core::widget::Role;
use reactive_graph::traits::Set as _;

/// The viewport the demo's beats settle into (the data-table canon's shape).
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

/// The word the tree says for one row, read back through the snapshot —
/// the same answer the sink maps onto AccessKit's `Selected` property.
/// The row is a nameless container (the cells carry the content), so
/// the walk reads the rows in tree order beside their worded siblings.
fn row_word(tree: Option<&alacer_core::passes::TreeSnapshot>, index: usize) -> String {
    let rows = tree
        .map(|t| {
            t.walk(None)
                .into_iter()
                .filter(|n| n.role == Role::TableRow)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let word = rows
        .get(index)
        .map(|row| match row.selected {
            Some(true) => "selected",
            Some(false) => "unpicked",
            None => "no word",
        })
        .unwrap_or("absent");
    format!("row {} ({word})", index + 1)
}

/// The app's selected door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the row, the
/// re-announcement paid for.
fn set_row_selected(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    selected: Option<bool>,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(row) = guard.widget_mut().as_any_mut().downcast_mut::<TableRow>() {
        row.set_selected(selected);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The selected tier: the collection's pick, said by the tree (ADR-0112)").boxed(),
    );

    // The selectable table: the word rides the tree from the first
    // frame — row one the pick, row two standing unpicked beside it,
    // row three belonging to no selection collection (the honest
    // absence the AT reads as ARIA's `undefined`).
    let table = pipeline.add_child(root, Table::new("Leads", 2).boxed());
    let picked = pipeline.add_child(table, TableRow::new().selected(true).boxed());
    let _ = pipeline.add_child(picked, TableCell::new("Ada", 0, 0).boxed());
    let _ = pipeline.add_child(picked, TableCell::new("640", 0, 1).boxed());
    let unpicked = pipeline.add_child(table, TableRow::new().selected(false).boxed());
    let _ = pipeline.add_child(unpicked, TableCell::new("Bo", 1, 0).boxed());
    let _ = pipeline.add_child(unpicked, TableCell::new("128", 1, 1).boxed());
    let _ = pipeline.add_child(table, TableRow::new().boxed());

    // The plain widget beside the collection: a member word, not a
    // widget word — the button inherits the honest absence.
    let _ = pipeline.add_child(root, Button::new("Approve").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = format!(
        "{} · {} · {}",
        row_word(snapshot.as_ref(), 0),
        row_word(snapshot.as_ref(), 1),
        row_word(snapshot.as_ref(), 2),
    );
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

    // Beat one: the word is a word, not a wall — the AT's press lands
    // on the button beside the selected table exactly as it always did.
    let consumed = pipeline.dispatch_a11y_action(
        snapshot
            .as_ref()
            .and_then(|t| t.find(Role::Button, "Approve"))
            .map(|node| node.id)
            .expect("the button rides the snapshot"),
        alacer_core::widget::A11yAction::DefaultPress,
    );
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::selected",
        consumed,
        "beat 1: the press route answers unchanged beside the collection's word — the word names state, it walls nothing"
    );

    // Beat two: the live door — the app's selection moved, the word the
    // tree says following it (the re-announcement the mutate seam's
    // request paid for); then the honest absence restored.
    set_row_selected(&mut pipeline, unpicked, Some(true));
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} · {} · {} — the live door moved row two's word",
        row_word(snapshot.as_ref(), 0),
        row_word(snapshot.as_ref(), 1),
        row_word(snapshot.as_ref(), 2),
    ));
    tracing::info!(
        target: "alacer::selected",
        row = "2",
        word = "selected",
        "beat 2: the app's door moved the word the tree says — the pick follows the app's selection"
    );
    set_row_selected(&mut pipeline, unpicked, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} · {} · {} — the door cleared the word",
        row_word(snapshot.as_ref(), 0),
        row_word(snapshot.as_ref(), 1),
        row_word(snapshot.as_ref(), 2),
    ));

    // The window: live play — the picked row says the word beside its
    // cells, the wordless row stays the honest absence, every door
    // answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the picked row says the word — every door answers as it always did")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer selected",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
