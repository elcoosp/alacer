//! The toolbar tier as a consumer's vocabulary (M129, ADR-0133): the
//! named strip a set of tools reads as — the word the AT announces
//! before it walks the tools, the tools the members' own controls.
//!
//! The beat the example exists to show: the strip is the tools'
//! layout, never a second channel to them — the buttons keep the
//! roles and labels they already carry, the status line reading the
//! presses back the way the app wired them, and the strip's word
//! costing the word and nothing else.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example toolbar`.
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

/// The tools the demo declares — the strip's span, each its own
/// control.
const TOOLS: [&str; 3] = ["Bold", "Italic", "Underline"];

/// The view: the named strip the tools read as, the presses reading
/// back through the signal the way the app wired them.
fn toolbar_view(last: &FluxRwSignal<String>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = last.try_get_untracked().unwrap_or_else(|| "none".into());

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The toolbar tier (ADR-0133)"));

    // The named strip: the container the AT announces before it walks
    // the tools, each tool its own control — the strip their layout,
    // never a second channel to them.
    let mut strip = ViewNode::element("toolbar")
        .prop("name", "Formatting")
        .prop("gap", 6.0_f32);

    for name in TOOLS {
        let tool_label = name;
        let next_last = *last;
        let next_wake = Rc::clone(wake);
        strip = strip.child(ViewNode::element("button").prop("text", name).prop(
            "on_click",
            PropValue::Handler(Rc::new(RefCell::new(move || {
                Flux::batch_of(|| next_last.set(tool_label.to_string()));
                (next_wake.clone())();
            }))),
        ));
    }
    root = root.child(strip);

    // The presses read back the way the app wired them.
    root =
        root.child(ViewNode::element("label").prop("text", format!("last tool used: {current}")));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let last = pipeline.flux().signal("none".to_string());
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = toolbar_view(&last, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer toolbar", (360.0, 240.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = toolbar_view(&last, &wake);
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
