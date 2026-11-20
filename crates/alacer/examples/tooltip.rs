//! The tooltip tier as a consumer's vocabulary (M136, ADR-0140): the
//! named help bubble the pointer pair summons — the word the AT
//! announces while the bubble is up.
//!
//! The beat the example exists to show: presence is the app's wiring —
//! the toggle mounts and unmounts the bubble (the reconciler's own
//! shape, no second `visible` flag), the bubble is the help's layout,
//! never a second channel to it (the label inside keeps the word it
//! already carries), and the bubble's word costs the word and nothing
//! else.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example tooltip`.
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

/// The view: the control the bubble helps, the bubble mounted exactly
/// while the signal says up, the reads reading back the way the app
/// wired them.
fn tooltip_view(shown: &FluxRwSignal<bool>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = shown.try_get_untracked().unwrap_or(false);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The tooltip tier (ADR-0140)"));

    // The control the bubble helps: a plain button, the word it
    // carries its own.
    let next_wake = Rc::clone(wake);
    root = root.child(ViewNode::element("button").prop("text", "Copy").prop(
        "on_click",
        PropValue::Handler(Rc::new(RefCell::new(move || {
            (next_wake.clone())();
        }))),
    ));

    // The bubble, present exactly while the app mounts it: the named
    // container the AT announces while it is up, the help text its own
    // child — the bubble their layout, never a second channel to them.
    if current {
        let bubble = ViewNode::element("tooltip")
            .prop("name", "Copy shortcut")
            .child(ViewNode::element("label").prop("text", "Ctrl+C copies the selection"));
        root = root.child(bubble);
    }

    // The toggle: the app's wiring the presence rides.
    let toggle_label = if current { "Hide help" } else { "Show help" };
    let next_shown = *shown;
    let next_wake = Rc::clone(wake);
    root = root.child(ViewNode::element("button").prop("text", toggle_label).prop(
        "on_click",
        PropValue::Handler(Rc::new(RefCell::new(move || {
            Flux::batch_of(|| next_shown.set(!next_shown.get_untracked()));
            (next_wake.clone())();
        }))),
    ));

    // The presence reads back the way the app wired it.
    let status = if current {
        "bubble: up"
    } else {
        "bubble: down"
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
    let shown = pipeline.flux().signal(false);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = tooltip_view(&shown, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer tooltip", (360.0, 240.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = tooltip_view(&shown, &wake);
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
