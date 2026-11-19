//! The live tier as a consumer's vocabulary (M120, ADR-0124): the
//! declared live region said by the a11y tree at last — AccessKit's
//! `Live` property (ARIA's `aria-live`) beside the name, the word every
//! platform AT turns into "announce what lands in this region" while
//! the app streams it. The politeness is the announcement seam's own
//! vocabulary: polite regions wait their turn, assertive regions
//! interrupt. Two containers under the word beside two honest
//! absences: the polite feed and the assertive ledger, the settled
//! archive and the plain scroller — the container that declared no
//! region, never a guessed politeness.
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the declared region names the region the AT watches,
//! it walls nothing (the containers stay put, every door answers as
//! ever, the imperative `announce` channel the channel it always was),
//! and the app's own door (`List::set_live` / `Table::set_live` through
//! the mutate seam) moves the word live — the feed's region withdrawn,
//! the archive declaring its own, the `PAINT | A11Y` request pays the
//! re-announcement — the write that lands where the word already is
//! queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example live
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, List, Table};
use alacer_core::announce::Politeness;
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

/// The declared regions the tree says for the containers, read back
/// through the snapshot — the same answer the sink maps onto AccessKit's
/// `Live` property: the politeness beside the name.
fn live_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let regions: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| matches!(n.role, Role::ScrollArea | Role::Table))
                .map(|n| match &n.live {
                    Some(Politeness::Polite) => format!("{} (polite)", n.label),
                    Some(Politeness::Assertive) => format!("{} (assertive)", n.label),
                    None => format!("{} (undeclared)", n.label),
                })
                .collect()
        })
        .unwrap_or_default();
    regions.join(" · ")
}

/// The app's live door: the mutate seam the app's code rides — the word
/// moved, `PAINT | A11Y` requested on the container, the
/// re-announcement paid for. `None` clears the word.
fn set_region_live(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    politeness: Option<Politeness>,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    let moved = if let Some(list) = guard.widget_mut().as_any_mut().downcast_mut::<List>() {
        list.set_live(politeness)
    } else if let Some(table) = guard.widget_mut().as_any_mut().downcast_mut::<Table>() {
        table.set_live(politeness)
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
        Label::new("The live tier: the declared region, said by the tree (ADR-0124)").boxed(),
    );

    // The regions: the declared region rides the tree from the first
    // frame — the polite feed and the assertive ledger under the word,
    // the settled archive and the plain scroller the honest absence
    // between them.
    let feed = pipeline.add_child(root, List::new(64.0).live(Politeness::Polite).boxed());
    let _ = pipeline.add_child(feed, Label::new("Row 1 — streamed politely").boxed());
    let ledger = pipeline.add_child(
        root,
        Table::new("Ledger", 2).live(Politeness::Assertive).boxed(),
    );
    let _ = pipeline.add_child(ledger, Label::new("Entry — interrupted for").boxed());
    let archive = pipeline.add_child(root, Table::new("Archive", 2).boxed());
    let _ = pipeline.add_child(archive, Label::new("Entry — settled").boxed());
    let _scroller = pipeline.add_child(root, List::new(64.0).boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = live_words(snapshot.as_ref());
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
        target: "alacer::live",
        regions = "list (polite), Ledger (assertive)",
        word = "live",
        "beat 1: the declared regions ride the tree — the politeness beside the name, the undeclared containers silent"
    );

    // Beat two: the live door — the feed's region withdrawn and the
    // archive declaring its own assertive region, the word the tree
    // says following the app's own; then the absence restored (the
    // archive's region withdrawn, the container the widget it always
    // was).
    set_region_live(&mut pipeline, feed, None);
    set_region_live(&mut pipeline, archive, Some(Politeness::Assertive));
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the live door moved the words",
        live_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::live",
        moved = "list → undeclared, Archive → assertive",
        word = "live",
        "beat 2: the app's door moved the words the tree says — the declared region follows the app's own"
    );
    set_region_live(&mut pipeline, archive, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the words",
        live_words(snapshot.as_ref()),
    ));

    // The window: live play — the worded regions say the word beside
    // the undeclared ones, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the regions say the word — every door answers as it always did").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer live",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
