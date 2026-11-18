//! The scroll position tier as a consumer's vocabulary (M101, ADR-0105):
//! the scroll container's resolved position and extent — where the
//! doors landed and the walls they clamp to — carried by the a11y tree
//! at last. Two lists beside one status line: the scrolled list, whose
//! declared span is live and whose tree answer is AccessKit's
//! `ScrollY`/`ScrollYMin`/`ScrollYMax` (the arithmetic the platform ATs
//! announce as "scrolled 140 of 0..460"), and the bare list, whose
//! content fits its viewport — the honest absence, the same wall every
//! scroll door answers `no_scrollable_content` with.
//!
//! The beat the example exists to show: the AT's own ask (`ScrollDown`,
//! one page — the funnel a screen reader's unit scroll rides) lands the
//! offset, the door's `PAINT | A11Y` currency pays the re-announcement,
//! and the position the tree carries is the landing beside the wall it
//! clamps to. The wheel scrolls natively beside it; the app's own
//! handle door moves the same state and rides the next paid pass.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example scroll_position
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, List};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yAction, Role, ScrollPosition, ScrollUnit, Widget};
use reactive_graph::traits::Set as _;

/// The viewport the demo's beats settle into (the scroll canon's shape).
const VIEWPORT_WIDTH: f32 = 480.0;
const VIEWPORT_HEIGHT: f32 = 380.0;

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

/// The position the tree carries, read back through the trait door —
/// the consumer's own reading of the fact, the same answer the snapshot
/// maps onto AccessKit's scroll properties. Each axis rides its own
/// honesty; the wall names the extent the doors clamp to.
fn position_word(widget: &dyn Widget) -> String {
    widget.scroll_position().map_or_else(
        || "the absence is honest (nothing scrolls)".to_string(),
        |position: ScrollPosition| {
            let vertical = position.vertical.map_or_else(
                || "no vertical span".to_string(),
                |axis| format!("{} of 0..{}", axis.offset, axis.max),
            );
            match position.horizontal {
                Some(axis) => format!("{vertical} · x {} of 0..{}", axis.offset, axis.max),
                None => vertical,
            }
        },
    )
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The scroll position tier: where the doors landed, the walls they clamp to (ADR-0105)",
        )
        .boxed(),
    );

    // The scrolled list: the span live (600 content over a 140
    // viewport), the offset the doors move — the tree says the position.
    let inbox = List::new(140.0);
    let inbox_handle = inbox.handle();
    let inbox_id = pipeline.add_child(root, inbox.boxed());
    inbox_handle.set_content_height(600.0);

    // The bare list beside it: the content fits the viewport — the span
    // is empty, no scroll exists, so no position exists.
    let archive = List::new(140.0);
    archive.handle().set_content_height(100.0);
    let archive_id = pipeline.add_child(root, archive.boxed());

    // The status line: the positions read back through the trait door
    // (the arena holds the boxed widgets), the consumer's side of the
    // fact the tree carries.
    let line = format!(
        "Inbox: {} · Archive: {}",
        position_word(pipeline.arena().widget(inbox_id)),
        position_word(pipeline.arena().widget(archive_id)),
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

    settle(&mut pipeline);

    // Beat: the AT's ask — one page down the Inbox through the action
    // funnel (the door a screen reader's unit scroll rides). The
    // consumed ask merges `PAINT | A11Y` on the container, the
    // re-announcement the position rides.
    if let Some(node) = pipeline
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::ScrollArea, "Inbox").map(|node| node.id))
    {
        let _ = pipeline.dispatch_a11y_action(node, A11yAction::ScrollDown(ScrollUnit::Page));
    }
    settle(&mut pipeline);
    let landed = position_word(pipeline.arena().widget(inbox_id));
    status_signal.set(format!(
        "Inbox: {landed} · Archive: {}",
        position_word(pipeline.arena().widget(archive_id)),
    ));
    tracing::info!(
        target: "alacer::scroll_position",
        inbox = %landed,
        archive = "the absence is honest",
        "the AT's page landed; the tree carries the landing beside the wall it clamps to (ADR-0105)"
    );

    // The window: live play — the same doors the beats rode, driven by
    // input. The wheel pays the a11y currency natively; the handle ask
    // moves the state the next paid pass re-announces.
    let _ = pipeline.add_child(
        root,
        Button::new("Scroll the Inbox by a handle ask")
            .on_click({
                let handle = inbox_handle.clone();
                move || handle.scroll_to(280.0)
            })
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: wheel the lists — the position the tree carries follows the doors that pay for it",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer scroll position",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
