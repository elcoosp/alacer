//! The numeric tier as a consumer's vocabulary (M99, ADR-0103): the
//! number behind the value's spoken word — the scalar the value tier's
//! funnel holds, carried by the a11y tree at last beside the text the
//! tree always said. Two shapes of one answer: the slider, whose number
//! rides the drag/arrows/AT funnel (aria-valuenow beside the
//! aria-valuetext), and the progress bar, whose number follows the
//! app's report the same way.
//!
//! The beat the example exists to show: the *Snap to max* button writes
//! through the same funnel the drag rides — the clamp puts the value on
//! the wall the tree already carries, and the number and the text move
//! together, one tier, two shapes, never disagreeing.
//!
//! The app shape is the ranges canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one scoped
//! re-render (`ViewRoot::update`, the retained tree patched in place).
//!
//! Run with `cargo run -p alacer --example numeric`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_core::FluxRwSignal;
use alacer_core::flux::Flux;
use alacer_view::{PropValue, ViewNode, ViewRoot};
use alacer_widgets::default_registry;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::rc::Rc;

/// The span the demo declares — the same walls the slider and the
/// progress report, and the wall the *Snap to max* door lands on.
const MIN: f32 = 0.0;
const MAX: f32 = 100.0;
const STEP: f32 = 5.0;

/// The view: the value control, the report beside it, the status line
/// reading the number-and-text pair back the way the tree carries it,
/// and the *Snap to max* door — the write the tier follows.
fn numeric_view(level: &FluxRwSignal<f32>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = level.try_get_untracked().unwrap_or(MIN);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The numeric tier (ADR-0103)"));

    // The value control: the funnel holds the scalar — the AT reads the
    // number beside the text, both landing where the clamp puts them.
    root = root.child(
        ViewNode::element("slider")
            .prop("name", "Volume")
            .prop("min", MIN)
            .prop("max", MAX)
            .prop("step", STEP)
            .prop("value", current)
            .prop(
                "on_change",
                PropValue::NumberHandler(Rc::new(RefCell::new({
                    // The signal is `Copy` — copy it out of the reference
                    // the view builder holds, no clone ceremony (the
                    // pedantic rule the facade's examples answer).
                    let level = *level;
                    let wake = Rc::clone(wake);
                    move |value: f32| {
                        Flux::batch_of(|| level.set(value));
                        (wake.clone())();
                    }
                }))),
            ),
    );

    // The report beside it: the app writes the scalar, the AT reads the
    // number the same door the text rides — the tier's read half.
    root = root.child(
        ViewNode::element("progress")
            .prop("name", "Upload")
            .prop("min", MIN)
            .prop("max", MAX)
            .prop("value", current),
    );

    // The status line mirrors the pair the tree carries: the number
    // (`aria-valuenow`) beside the spoken word (`aria-valuetext`).
    root = root.child(ViewNode::element("label").prop(
        "text",
        format!("now {current}, text \"{current}\" of {MIN}..={MAX}, step {STEP}"),
    ));

    // The *Snap to max* door: the write lands on the wall the tree
    // carries — the number and the text following one funnel together.
    // The signal is `Copy`; copy it out of the view builder's reference.
    let next_level = *level;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("button")
            .prop("text", "Snap to max")
            .prop(
                "description",
                "Writes 100 through the value tier's own funnel",
            )
            .prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new(move || {
                    Flux::batch_of(|| next_level.set(MAX));
                    (next_wake)();
                }))),
            ),
    );

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let level = pipeline.flux().signal(MIN);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = numeric_view(&level, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer numeric", (360.0, 260.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = numeric_view(&level, &wake);
        let _report = root.update(&view, pipeline);
    }));
    if let Err(error) = app.with_patch_channel(channel).run() {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
