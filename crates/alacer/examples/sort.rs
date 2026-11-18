//! The sort direction tier as a consumer's vocabulary (M112, ADR-0116):
//! the sorted column's order said by the a11y tree at last — AccessKit's
//! `SortDirection` property (ARIA's `aria-sort`) beside the position the
//! M6 slot already carried, the word every platform AT turns into
//! "sorted ascending" while it walks the rows. Four headers beside one
//! status line: the column that sorts ascending, the column that sorts
//! descending, the column that sorts by an order ARIA names no word for,
//! the plain header between them — the honest absence the tier keeps
//! honest (ARIA's `none`, never a guessed direction).
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the sort word names an order, it walls nothing (the
//! headers stay put, the button beside them answers its press as ever),
//! and the app's own door (`ColumnHeader::set_sort` through the mutate
//! seam) moves the word live, the `PAINT | A11Y` request paying the
//! re-announcement — the write that lands where the word already is
//! queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example sort
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, ColumnHeader, Label};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::{PassPipeline, TreeNode};
use alacer_core::widget::{Role, SortDirection};
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

/// The order the tree says for one header, read back through the
/// snapshot — the same answer the sink maps onto AccessKit's
/// `SortDirection` property: the kind beside the word.
fn header_words(tree: Option<&alacer_core::passes::TreeSnapshot>, name: &str) -> String {
    let header = tree.and_then(|t| t.find(Role::ColumnHeader, name).cloned());
    match header {
        Some(TreeNode {
            sort_direction: Some(kind),
            ..
        }) => {
            let order = match kind {
                SortDirection::Ascending => "ascending",
                SortDirection::Descending => "descending",
                SortDirection::Other => "other",
            };
            format!("{name} (sorted {order})")
        }
        Some(_) => format!("{name} (no order)"),
        None => format!("{name}: absent"),
    }
}

/// The app's sort door: the mutate seam the app's code rides — the word
/// moved, `PAINT | A11Y` requested on the header, the re-announcement
/// paid for. `None` clears the word.
fn set_header_sort(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    sort_direction: Option<SortDirection>,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(header) = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<ColumnHeader>()
    {
        header.set_sort(sort_direction);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

/// The status line's whole read: the words the tree says for the four
/// headers, joined — the consumer's side of the fact the AT reads, the
/// same answer the sink maps onto AccessKit's `SortDirection` property.
fn all_header_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    format!(
        "{} · {} · {} · {}",
        header_words(tree, "Price"),
        header_words(tree, "Name"),
        header_words(tree, "Added"),
        header_words(tree, "Notes"),
    )
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The sort direction tier: the column's order, said by the tree (ADR-0116)")
            .boxed(),
    );

    // The headers: the order rides the tree from the first frame — the
    // ascending column, the descending column, the other column, the
    // plain header the honest absence between them.
    let _price = pipeline.add_child(
        root,
        ColumnHeader::new("Price", 0)
            .sort(SortDirection::Ascending)
            .boxed(),
    );
    let name = pipeline.add_child(
        root,
        ColumnHeader::new("Name", 1)
            .sort(SortDirection::Descending)
            .boxed(),
    );
    let _added = pipeline.add_child(
        root,
        ColumnHeader::new("Added", 2)
            .sort(SortDirection::Other)
            .boxed(),
    );
    let notes = pipeline.add_child(root, ColumnHeader::new("Notes", 3).boxed());

    // The plain widget beside the headers: a header word, not a widget
    // word — the button inherits the honest absence.
    let _ = pipeline.add_child(root, Button::new("Resort").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = all_header_words(snapshot.as_ref());
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
    // sorted columns answers its press exactly as it always did.
    let consumed = pipeline.dispatch_a11y_action(
        snapshot
            .as_ref()
            .and_then(|t| t.find(Role::Button, "Resort"))
            .map(|node| node.id)
            .expect("the button rides the snapshot"),
        alacer_core::widget::A11yAction::DefaultPress,
    );
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::sort",
        consumed,
        "beat 1: the AT's press lands unchanged beside the sorted columns — the word names an order, it walls nothing"
    );

    // Beat two: the live door — the app's column order moved, the word
    // the tree says following it (the re-announcement the mutate seam's
    // request paid for); then the honest absence restored.
    set_header_sort(&mut pipeline, name, Some(SortDirection::Ascending));
    set_header_sort(&mut pipeline, notes, Some(SortDirection::Descending));
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the live door moved the headers' words",
        all_header_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::sort",
        headers = "Name, Notes",
        word = "sort_direction",
        "beat 2: the app's door moved the words the tree says — the order follows the app's own"
    );
    set_header_sort(&mut pipeline, notes, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the words",
        all_header_words(snapshot.as_ref()),
    ));

    // The window: live play — the sorted columns say the order beside
    // the plain header, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the headers say the word — every door answers as it always did").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer sort",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
