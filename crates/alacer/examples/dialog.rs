//! The dialog tier as a consumer's vocabulary (M134, ADR-0138): the
//! window the modal vocabulary speaks as "this is a dialog", said by
//! the tree — the word the AT announces before it walks the
//! conversation.
//!
//! The beat the example exists to show: the window word prices
//! nothing but the word — the controls inside the dialog keep the
//! words and doors they always carried (the field edits, the submit
//! reads back the way the app wired it), the plain panel beside it is
//! the honest absence, and the modal *semantics* — the walk bounded to
//! the window's subtree while it is up — ride the focus scope the M25
//! tier published (`PassPipeline::focus_move_in`), never a second
//! channel: the word names the window, the walk bounds it.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example dialog`.
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

/// The view: the named window with its conversation, the submit
/// reading back the way the app wired it — the bare panel beside it
/// the honest absence.
fn dialog_view(last: &FluxRwSignal<String>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = last.try_get_untracked().unwrap_or_else(|| "none".into());

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The dialog tier (ADR-0138)"));

    // The named window: the dialog word the AT announces before it
    // walks the conversation — the field and the submit riding the
    // doors they always carried, the window naming what they read as.
    let next_last = *last;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("dialog")
            .prop("name", "Confirm publish")
            .prop("description", "The file goes live")
            .child(
                ViewNode::element("text_input")
                    .prop("name", "File name")
                    .prop("placeholder", "What ships"),
            )
            .child(ViewNode::element("button").prop("text", "Publish").prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new(move || {
                    Flux::batch_of(|| next_last.set("published".into()));
                    (next_wake.clone())();
                }))),
            )),
    );

    // The reads read back the way the app wired them.
    root = root.child(ViewNode::element("label").prop("text", format!("last act: {current}")));

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

    let view = dialog_view(&last, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer dialog", (360.0, 220.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = dialog_view(&last, &wake);
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
