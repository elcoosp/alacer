//! The multiselectable tier as a consumer's vocabulary (M109, ADR-0113):
//! the collection's capacity said by the a11y tree at last — AccessKit's
//! `Multiselectable` property (ARIA's `aria-multiselectable`) beside the
//! member picks the selected tier (ADR-0112) already carried, the word
//! every platform AT turns into "you can pick more than one" before the
//! user ever reaches the rows. A mini pair of tables beside one status
//! line: the multi-pick table, the single-pick table — and the plain
//! containers beside them both, the honest absence the tier keeps
//! honest.
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the multiselectable table's doors answer exactly as
//! they always did (the word names capacity, it walls nothing), and the
//! app's own door (`Table::set_multiselectable` through the mutate
//! seam) moves the word live, the `PAINT | A11Y` request paying the
//! re-announcement — the write that lands where the word already is
//! queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example multiselectable
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

/// The words the tree says for one container, read back through the
/// snapshot — the same answers the sink maps onto AccessKit's
/// properties: the capacity word beside the member pick word.
fn container_words(
    tree: Option<&alacer_core::passes::TreeSnapshot>,
    role: Role,
    name: &str,
) -> String {
    let node = tree.and_then(|t| t.find(role, name).cloned());
    match node {
        Some(TreeNode {
            multiselectable: true,
            selected,
            ..
        }) => format!(
            "{name} (multi-pick, first row: {})",
            match selected {
                Some(true) => "selected",
                Some(false) => "unpicked",
                None => "no word",
            }
        ),
        Some(_) => format!("{name} (single-pick)"),
        None => format!("{name}: absent"),
    }
}

/// The app's multiselectable door: the mutate seam the app's code
/// rides — the word moved, `PAINT | A11Y` requested on the container,
/// the re-announcement paid for.
fn set_table_multiselectable(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    multiselectable: bool,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(table) = guard.widget_mut().as_any_mut().downcast_mut::<Table>() {
        table.set_multiselectable(multiselectable);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The multiselectable tier: the collection's capacity, said by the tree (ADR-0113)",
        )
        .boxed(),
    );

    // The collections: the capacity rides the tree from the first frame
    // — the multi-pick table (its first row the pick, the selected
    // tier's word beside it), the single-pick table the honest absence.
    let leads = pipeline.add_child(root, Table::new("Leads", 2).multiselectable(true).boxed());
    let picked = pipeline.add_child(leads, TableRow::new().selected(true).boxed());
    let _ = pipeline.add_child(picked, TableCell::new("Ada", 0, 0).boxed());
    let _ = pipeline.add_child(picked, TableCell::new("640", 0, 1).boxed());
    let themes = pipeline.add_child(root, Table::new("Themes", 1).boxed());
    let theme_row = pipeline.add_child(themes, TableRow::new().boxed());
    let _ = pipeline.add_child(theme_row, TableCell::new("Dusk", 0, 0).boxed());

    // The plain widget beside the collections: a container word, not a
    // widget word — the button inherits the honest absence.
    let _ = pipeline.add_child(root, Button::new("Approve").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = format!(
        "{} · {}",
        container_words(snapshot.as_ref(), Role::Table, "Leads"),
        container_words(snapshot.as_ref(), Role::Table, "Themes"),
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
    // on the button beside the multiselectable table exactly as it
    // always did.
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
        target: "alacer::multiselectable",
        consumed,
        "beat 1: the press route answers unchanged beside the container's word — the word names capacity, it walls nothing"
    );

    // Beat two: the live door — the app's capacity moved, the word the
    // tree says following it (the re-announcement the mutate seam's
    // request paid for); then the honest absence restored.
    set_table_multiselectable(&mut pipeline, themes, true);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} · {} — the live door moved the Themes' word",
        container_words(snapshot.as_ref(), Role::Table, "Leads"),
        container_words(snapshot.as_ref(), Role::Table, "Themes"),
    ));
    tracing::info!(
        target: "alacer::multiselectable",
        table = "Themes",
        word = "multiselectable",
        "beat 2: the app's door moved the word the tree says — the capacity follows the app's contract"
    );
    set_table_multiselectable(&mut pipeline, themes, false);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} · {} — the door cleared the word",
        container_words(snapshot.as_ref(), Role::Table, "Leads"),
        container_words(snapshot.as_ref(), Role::Table, "Themes"),
    ));

    // The window: live play — the multi-pick table says the capacity
    // beside the member pick, the single-pick table stays the honest
    // absence, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: the multi-pick table says the word — every door answers as it always did",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer multiselectable",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
