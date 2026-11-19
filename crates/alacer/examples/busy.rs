//! The busy tier as a consumer's vocabulary (M117, ADR-0121): the
//! subtree's updating fact said by the a11y tree at last — AccessKit's
//! `Busy` property (ARIA's `aria-busy`) beside the name, the word every
//! platform AT turns into "hold what you would announce here" while the
//! stream is still writing the region. Two containers under the word
//! beside two settled ones: the loading feed and the streaming ledger,
//! the settled archive and the plain scroller — the honest absence
//! between them (the container that is not updating, never a guessed
//! "busy").
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the busy word names the updating fact, it walls
//! nothing (the containers stay put, every door answers as ever), and
//! the app's own door (`List::set_busy` / `Table::set_busy` through the
//! mutate seam) moves the word live — the stream finishes, the archive
//! starts its own refresh, the `PAINT | A11Y` request pays the
//! re-announcement — the write that lands where the word already is
//! queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example busy
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, List, Table};
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

/// The updating facts the tree says for the containers, read back
/// through the snapshot — the same answer the sink maps onto AccessKit's
/// `Busy` property: the word beside the name.
fn busy_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let regions: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| matches!(n.role, Role::ScrollArea | Role::Table))
                .map(|n| {
                    if n.busy {
                        format!("{} (busy)", n.label)
                    } else {
                        format!("{} (settled)", n.label)
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    regions.join(" · ")
}

/// The app's busy door: the mutate seam the app's code rides — the word
/// moved, `PAINT | A11Y` requested on the container, the
/// re-announcement paid for. `false` clears the word.
fn set_region_busy(pipeline: &mut PassPipeline, node: alacer_core::NodeId, busy: bool) {
    let mut guard = pipeline.arena_mut().mutate(node);
    let moved = if let Some(list) = guard.widget_mut().as_any_mut().downcast_mut::<List>() {
        list.set_busy(busy)
    } else if let Some(table) = guard.widget_mut().as_any_mut().downcast_mut::<Table>() {
        table.set_busy(busy)
    } else {
        false
    };
    if moved {
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The busy tier: the subtree's updating fact, said by the tree (ADR-0121)")
            .boxed(),
    );

    // The regions: the updating fact rides the tree from the first
    // frame — the loading feed and the streaming ledger under the word,
    // the settled archive and the plain scroller the honest absence
    // between them.
    let feed = pipeline.add_child(root, List::new(64.0).busy(true).boxed());
    let _ = pipeline.add_child(feed, Label::new("Row 1 — streaming").boxed());
    let ledger = pipeline.add_child(root, Table::new("Ledger", 2).busy(true).boxed());
    let _ = pipeline.add_child(ledger, Label::new("Entry — streaming").boxed());
    let archive = pipeline.add_child(root, Table::new("Archive", 2).boxed());
    let _ = pipeline.add_child(archive, Label::new("Entry — settled").boxed());
    let _scroller = pipeline.add_child(root, List::new(64.0).boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = busy_words(snapshot.as_ref());
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
        target: "alacer::busy",
        regions = "list, Ledger",
        word = "busy",
        "beat 1: the updating fact rides the tree — the word beside the name, the settled regions silent"
    );

    // Beat two: the live door — the stream finished its feed, the
    // archive started its own refresh, the word the tree says following
    // the app's own; then the absence restored (the refresh done, the
    // AT speaks what it held).
    set_region_busy(&mut pipeline, feed, false);
    set_region_busy(&mut pipeline, archive, true);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the live door moved the words",
        busy_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::busy",
        moved = "feed → settled, Archive → busy",
        word = "busy",
        "beat 2: the app's door moved the words the tree says — the updating fact follows the app's own"
    );
    set_region_busy(&mut pipeline, archive, false);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the words",
        busy_words(snapshot.as_ref()),
    ));

    // The window: live play — the worded regions say the word beside
    // the settled ones, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the regions say the word — every door answers as it always did").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer busy",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
