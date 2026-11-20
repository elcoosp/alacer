//! The image tier as a consumer's vocabulary (M138, ADR-0142): the
//! named picture the AT announces as a picture — the word beside the
//! pixels the app owns.
//!
//! The beat the example exists to show: the picture's name and alt text
//! are the words the app declared (the M95 description word carrying
//! the alt text beside the name), the pixel story stays the app's own
//! (the canvas beside it paints the bars through the paint seam — the
//! word never paints), and the words cost the words and nothing else
//! (the status line reads back the way the app wired them).
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example image`.
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

/// The view: the named picture beside the canvas that paints it, the
/// alt text riding the M95 word, the reads reading back the way the
/// app wired them.
fn image_view(detailed: &FluxRwSignal<bool>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = detailed.try_get_untracked().unwrap_or(false);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The image tier (ADR-0142)"));

    // The named picture: the word the AT announces as a picture — the
    // name and the alt text the app declared, the pixels the canvas
    // beside it paints (the word never paints).
    let alt = if current {
        "Bars per mount, refreshed hourly; click the toggle for the summary"
    } else {
        "Bars per mount, refreshed hourly"
    };
    let picture = ViewNode::element("image")
        .prop("name", "Disk usage chart")
        .prop("description", alt)
        .prop("height", 120.0_f32)
        .child(
            ViewNode::element("canvas")
                .prop("name", "Disk usage chart")
                .prop("on_paint", PropValue::Handler(Rc::new(RefCell::new(|| {})))),
        );
    root = root.child(picture);

    // The app's wiring: one signal, the alt text the word carries
    // moving with it.
    let toggle_label = if current {
        "Shorter alt text"
    } else {
        "Longer alt text"
    };
    let next_detailed = *detailed;
    let next_wake = Rc::clone(wake);
    root = root.child(ViewNode::element("button").prop("text", toggle_label).prop(
        "on_click",
        PropValue::Handler(Rc::new(RefCell::new(move || {
            Flux::batch_of(|| next_detailed.set(!next_detailed.get_untracked()));
            (next_wake.clone())();
        }))),
    ));

    // The words read back the way the app wired them.
    let status = if current {
        "alt text: detailed"
    } else {
        "alt text: brief"
    };
    root = root.child(ViewNode::element("label").prop("text", status));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let detailed = pipeline.flux().signal(false);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = image_view(&detailed, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer image", (360.0, 280.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = image_view(&detailed, &wake);
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
