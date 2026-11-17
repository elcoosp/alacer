//! The orientation tier as a consumer's vocabulary (M98, ADR-0102): the
//! single axis a scroll container declares its scrolls to ride — the
//! word the AT's arrows read before they ride — carried by the a11y
//! tree at last. Two lists beside one status line: the vertical list,
//! whose y axis is the constructor's only declared story, and the
//! two-axis list, which declared *both* axes (ADR-0093) and so has no
//! single-axis word to say — the honest absence, ARIA's `undefined`,
//! never a wrong half.
//!
//! The beat the example exists to show: the declaration is the
//! constructor's story and the offset is the state's — scrolling either
//! list moves the offset and leaves the announced axis exactly where
//! the constructor put it, the two facts the AT reads beside each
//! other. Every scroll lands inside the axis word the tree carries.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example orientation
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, List};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{ScrollOrientation, Widget};
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

/// The axis word the tree carries, read back through the trait door —
/// the consumer's own reading of the declaration, the same answer the
/// snapshot and the AT announce.
fn axis_word(widget: &dyn Widget) -> String {
    widget.scroll_orientation().map_or_else(
        || "both axes (undefined)".to_string(),
        |axis: ScrollOrientation| axis.name().to_string(),
    )
}

/// The beats the demo walks before the window opens: the two
/// declarations read back, and one scroll per list — the offset moving
/// under a word that stays.
fn run_beats(
    vertical: &alacer::widgets::ListHandle,
    two_axis: &alacer::widgets::ListHandle,
    status_signal: alacer_core::FluxRwSignal<String>,
) {
    let _route = tracing::trace_span!(target: "alacer::orientation", "beats").entered();
    tracing::info!(
        target: "alacer::orientation",
        vertical = "vertical",
        two_axis = "both axes (undefined)",
        "the declared axes are the constructors' own words (ADR-0102)"
    );

    // Beat one: the vertical list scrolls its declared track — the y
    // axis the tree announced, the offset the state holds.
    vertical.scroll_to(120.0);
    tracing::info!(
        target: "alacer::orientation",
        offset = vertical.offset(),
        "beat 1: the vertical list scrolls its declared axis"
    );

    // Beat two: the two-axis list scrolls its x half — the axis the
    // tree honestly carries no single word for (both axes, undefined).
    two_axis.scroll_to_x(80.0);
    tracing::info!(
        target: "alacer::orientation",
        offset_x = two_axis.offset_x(),
        "beat 2: the two-axis list scrolls its x half — the declared absence holding"
    );
    status_signal.set(
        "declared: vertical beside both axes (undefined) — the offsets moved, the words stayed"
            .to_string(),
    );
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The orientation tier: the declared axis the AT's arrows read (ADR-0102)")
            .boxed(),
    );

    // The vertical list: the y axis is the constructor's only declared
    // story — the tree says `vertical`. The word is read through the
    // trait door before the widget is boxed into the arena (the
    // consumer's side of the fact the tree carries).
    let vertical = List::new(140.0);
    let vertical_handle = vertical.handle();
    let vertical_word = axis_word(&vertical);
    let _ = pipeline.add_child(root, vertical.boxed());
    vertical_handle.set_content_height(600.0);

    // The two-axis list: both axes are the declared story — the tree
    // carries the honest absence, ARIA's `undefined`.
    let two_axis = List::two_axis(320.0, 140.0);
    let two_axis_handle = two_axis.handle();
    let two_axis_word = axis_word(&two_axis);
    let _ = pipeline.add_child(root, two_axis.boxed());
    two_axis_handle.set_content_height(600.0);
    two_axis_handle.set_content_width(800.0);

    // The status line: the declarations read back through the trait
    // door, the consumer's side of the fact the tree carries.
    let status_signal = pipeline.flux_mut().signal(format!(
        "declared: {vertical_word} beside {two_axis_word} — the offsets moved, the words stayed"
    ));
    let status_binding =
        RenderBinding::new(format!("declared: {vertical_word} beside {two_axis_word}"));
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );

    settle(&mut pipeline);
    run_beats(&vertical_handle, &two_axis_handle, status_signal);

    // The window: live play — the same doors the beats rode, driven by
    // buttons. Each press moves an offset; the announced axis word
    // never moves.
    let _ = pipeline.add_child(
        root,
        Button::new("Scroll the vertical list")
            .on_click({
                let handle = vertical_handle.clone();
                move || handle.scroll_to(240.0)
            })
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Button::new("Scroll the two-axis list across")
            .on_click({
                let handle = two_axis_handle.clone();
                move || handle.scroll_to_x(160.0)
            })
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: scroll either list — the offset moves, the declared axis the tree carries stays",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer orientation",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
