//! The search-field tier as a consumer's vocabulary (M132, ADR-0136):
//! the field whose purpose is search, said by the tree — the word the
//! AT announces beside the name, the platform's find conventions key
//! off.
//!
//! The beat the example exists to show: the purpose word prices
//! nothing but the word — the declared field edits exactly like the
//! plain one (the typing, the submit, the IME contract unchanged), the
//! tree carries the word beside the name, and a declare/clear write is
//! the app's own declaration flipped in place.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example search_field`.
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

/// The view: the declared search field beside the plain one — the
/// purpose word the tree carries, the submits reading back the way the
/// app wired them.
fn search_field_view(last: &FluxRwSignal<String>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = last.try_get_untracked().unwrap_or_else(|| "none".into());

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The search-field tier (ADR-0136)"));

    // The declared field: the purpose word the AT announces beside the
    // name — "search, edit text" — the find conventions keying off the
    // word the tree carries. The submit rides the field's own Enter
    // door, unchanged.
    let next_last = *last;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("text_input")
            .prop("name", "Mail search")
            .prop("search", true)
            .prop("placeholder", "Type to search the mailbox")
            .prop(
                "on_submit",
                PropValue::TextHandler(Rc::new(RefCell::new(move |query: &str| {
                    Flux::batch_of(|| next_last.set(format!("searched: {query}")));
                    (next_wake.clone())();
                }))),
            ),
    );

    // The plain field: the honest absence beside the word — a field
    // that declared nothing announces no purpose.
    root = root.child(
        ViewNode::element("text_input")
            .prop("name", "Notes")
            .prop("placeholder", "Plain field, no purpose word"),
    );

    // The submits read back the way the app wired them.
    root = root.child(ViewNode::element("label").prop("text", format!("last submit: {current}")));

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

    let view = search_field_view(&last, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer search field",
        (360.0, 220.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = search_field_view(&last, &wake);
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
