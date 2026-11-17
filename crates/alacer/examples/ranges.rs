//! The range tier as a consumer's vocabulary (M97, ADR-0101): the
//! declared span every value-tier write funnels through — the walls the
//! drag, the arrows, and the AT's own acts land inside — carried by the
//! a11y tree at last. Two reports beside one control: the slider, whose
//! walls *and* step the AT reads (the span the snap arithmetic
//! declares), and the progress bar, whose walls ride and whose step is
//! the honest absence — a report declares where its value lives, not
//! how far one act moves it.
//!
//! The beat the example exists to show: the *Snap to max* button writes
//! through the same funnel the drag rides — the clamp puts the value on
//! the wall the tree already carries, and the tree's truth follows the
//! pixels exactly. Every write lands inside the announced span; nothing
//! else in the frame pays for it.
//!
//! The app shape is the placeholders canon's own: handlers write one
//! signal and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example ranges`.
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

/// The view: the ranged slider, the report whose walls ride beside it,
/// the status line reading the span back the way the tree carries it,
/// and the *Snap to max* door — the write the range constrains.
fn ranges_view(level: &FluxRwSignal<f32>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = level.try_get_untracked().unwrap_or(MIN);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The range tier (ADR-0101)"));

    // The ranged control: walls and step ride the `min`/`max`/`step`
    // words — the span the AT reads beside the value, the span every
    // write lands inside.
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

    // The report beside it: walls ride, the step is the honest absence —
    // the tree announces the span, never a guessed default.
    root = root.child(
        ViewNode::element("progress")
            .prop("name", "Upload")
            .prop("min", MIN)
            .prop("max", MAX)
            .prop("value", current),
    );

    // The status line mirrors what the tree carries for the slider —
    // the app's own truth, one write away.
    root = root.child(ViewNode::element("label").prop(
        "text",
        format!("Level: {current} of {MIN}..={MAX}, step {STEP}"),
    ));

    // The *Snap to max* door: the write lands on the wall the tree
    // carries — the clamp's answer and the announced span agreeing.
    // The signal is `Copy`; copy it out of the view builder's reference.
    let next_level = *level;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("button")
            .prop("text", "Snap to max")
            .prop("description", "Writes 100 through the range's own funnel")
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

    let view = ranges_view(&level, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer ranges", (360.0, 260.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = ranges_view(&level, &wake);
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
