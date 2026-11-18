//! The level tier as a consumer's vocabulary (M113, ADR-0117): the
//! nested member's depth said by the a11y tree at last — AccessKit's
//! `Level` property (zero-based, beside ARIA's one-based `aria-level`)
//! beside the grid position the M6 slot already carried, the word every
//! platform AT turns into "level 2" while it walks the rows. Four rows
//! beside one status line: the root-level row, the level-1 row, the
//! level-2 row, the flat row between them — the honest absence the
//! tier keeps honest (the flat row, never a guessed `0`).
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the level word names a depth, it walls nothing (the
//! rows stay put, the button beside them answers its press as ever),
//! and the app's own door (`TableRow::set_level` through the mutate
//! seam) moves the word live, the `PAINT | A11Y` request paying the
//! re-announcement — the write that lands where the word already is
//! queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example level
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, TableRow};
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

/// The depths the tree says for the rows, read back through the
/// snapshot in walk order — the same answer the sink maps onto
/// AccessKit's `Level` property: the zero-based count beside the word.
/// Rows are nameless members (the label resolves from their children),
/// so the read walks the snapshot's rows by position.
fn row_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let depths: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| n.role == Role::TableRow)
                .map(|n| match n.level {
                    Some(depth) => format!("row (level {depth})"),
                    None => "row (flat)".to_owned(),
                })
                .collect()
        })
        .unwrap_or_default();
    depths.join(" · ")
}

/// The app's level door: the mutate seam the app's code rides — the
/// word moved, `PAINT | A11Y` requested on the row, the
/// re-announcement paid for. `None` clears the word.
fn set_row_level(pipeline: &mut PassPipeline, node: alacer_core::NodeId, level: Option<usize>) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(row) = guard.widget_mut().as_any_mut().downcast_mut::<TableRow>() {
        row.set_level(level);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The level tier: the member's depth, said by the tree (ADR-0117)").boxed(),
    );

    // The rows: the depth rides the tree from the first frame — the
    // root-level row, the level-1 row, the level-2 row, the flat row
    // the honest absence between them.
    let _top = pipeline.add_child(root, TableRow::new().level(0).boxed());
    let group = pipeline.add_child(root, TableRow::new().level(1).boxed());
    let _leaf = pipeline.add_child(root, TableRow::new().level(2).boxed());
    let flat = pipeline.add_child(root, TableRow::new().boxed());

    // The plain widget beside the rows: a member word, not a widget
    // word — the button inherits the honest absence.
    let _ = pipeline.add_child(root, Button::new("Collapse all").boxed());

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

    // Beat one: the word is a word, not a wall — the button beside the
    // nested rows answers its press exactly as it always did.
    let consumed = pipeline.dispatch_a11y_action(
        snapshot
            .as_ref()
            .and_then(|t| t.find(Role::Button, "Collapse all"))
            .map(|node| node.id)
            .expect("the button rides the snapshot"),
        alacer_core::widget::A11yAction::DefaultPress,
    );
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::level",
        consumed,
        "beat 1: the AT's press lands unchanged beside the nested rows — the word names a depth, it walls nothing"
    );

    // Beat two: the live door — the app's depth moved, the word the
    // tree says following it (the re-announcement the mutate seam's
    // request paid for); then the honest absence restored.
    set_row_level(&mut pipeline, group, Some(2));
    set_row_level(&mut pipeline, flat, Some(1));
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the live door moved the rows' words",
        row_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::level",
        rows = "Group, Flat",
        word = "level",
        "beat 2: the app's door moved the words the tree says — the depth follows the app's own"
    );
    set_row_level(&mut pipeline, flat, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the words",
        row_words(snapshot.as_ref()),
    ));

    // The window: live play — the nested rows say the depth beside the
    // flat row, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the rows say the word — every door answers as it always did").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer level",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
