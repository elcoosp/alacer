//! The expanded tier as a consumer's vocabulary (M110, ADR-0114): the
//! collection member's disclosure said by the a11y tree at last —
//! AccessKit's `Expanded` property (ARIA's `aria-expanded`) beside the
//! picks the selected tier (ADR-0112) already carried, the word every
//! platform AT turns into "expanded" / "collapsed" while the user walks
//! the rows. A folder table beside one status line: the row standing
//! open, the row standing collapsed, the wordless row between them —
//! the trio the honest shape (`None` the member that opens and closes
//! not, never a guessed `false`).
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the disclosure names state, it walls nothing (the
//! rows' press and focus routes the widget's own), and the app's own
//! door (`TableRow::set_expanded` through the mutate seam) moves the
//! word live, the `PAINT | A11Y` request paying the re-announcement —
//! the write that lands where the word already is queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example expanded
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Table, TableCell, TableRow};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::{PassPipeline, TreeNode};
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

/// The disclosure word the tree says for the snapshot's first table row,
/// read back through the snapshot — the same answer the sink maps onto
/// AccessKit's `Expanded` property: the trio the honest shape.
fn row_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let row = tree.and_then(|t| t.nodes.iter().find(|n| n.role == Role::TableRow).cloned());
    match row {
        Some(TreeNode {
            expanded: Some(true),
            ..
        }) => "Archive (open)".to_string(),
        Some(TreeNode {
            expanded: Some(false),
            ..
        }) => "Archive (collapsed)".to_string(),
        Some(_) => "Archive (no word)".to_string(),
        None => "Archive: absent".to_string(),
    }
}

/// The app's expanded door: the mutate seam the app's code rides — the
/// word moved, `PAINT | A11Y` requested on the row, the
/// re-announcement paid for. `None` clears the word.
fn set_row_expanded(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    expanded: Option<bool>,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(row) = guard.widget_mut().as_any_mut().downcast_mut::<TableRow>() {
        row.set_expanded(expanded);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The expanded tier: the collection member's disclosure, said by the tree (ADR-0114)",
        )
        .boxed(),
    );

    // The collection: the disclosure rides the tree from the first
    // frame — the row standing open, the row standing collapsed, the
    // wordless row the honest absence between them.
    let folders = pipeline.add_child(root, Table::new("Folders", 1).boxed());
    let archive = pipeline.add_child(folders, TableRow::new().expanded(true).boxed());
    let _ = pipeline.add_child(archive, TableCell::new("Archive", 0, 0).boxed());
    let drafts = pipeline.add_child(folders, TableRow::new().expanded(false).boxed());
    let _ = pipeline.add_child(drafts, TableCell::new("Drafts", 1, 0).boxed());
    let spam = pipeline.add_child(folders, TableRow::new().boxed());
    let _ = pipeline.add_child(spam, TableCell::new("Spam", 2, 0).boxed());

    // The plain widget beside the collection: a member word, not a
    // widget word — the button inherits the honest absence.
    let _ = pipeline.add_child(root, Button::new("Approve").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = row_words(snapshot.as_ref());
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
    // on the button beside the expanded row exactly as it always did.
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
        target: "alacer::expanded",
        consumed,
        "beat 1: the press route answers unchanged beside the member's word — the word names state, it walls nothing"
    );

    // Beat two: the live door — the app's disclosure moved, the word
    // the tree says following it (the re-announcement the mutate seam's
    // request paid for); then the collapsed word, then the honest
    // absence restored.
    set_row_expanded(&mut pipeline, archive, Some(false));
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(row_words(snapshot.as_ref()));
    tracing::info!(
        target: "alacer::expanded",
        row = "Archive",
        word = "expanded",
        "beat 2: the app's door moved the word the tree says — the disclosure follows the app's contract"
    );
    set_row_expanded(&mut pipeline, archive, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the word",
        row_words(snapshot.as_ref())
    ));

    // The window: live play — the folder table says the disclosure
    // beside the member pick, the wordless row stays the honest
    // absence, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the folder rows say the word — every door answers as it always did")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer expanded",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
