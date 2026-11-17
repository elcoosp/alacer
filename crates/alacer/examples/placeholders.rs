//! The placeholder tier as a consumer's vocabulary (M96, ADR-0100): the
//! hint an empty field announces — the grey sentence the canvas has
//! painted since M6, carried by the a11y tree at last. Two fields: the
//! hinted one, whose placeholder the AT reads while the value is empty,
//! and the bare one, whose honest absence announces nothing — never an
//! empty hint.
//!
//! The beat the example exists to show: the *Swap hint* button rewrites
//! the search field's hint through the view's `placeholder` word — the
//! patch tier pays **both** masks the tier costs, the paint the painter
//! draws and the re-announcement the tree carries (the a11y-*also* prop
//! class, ADR-0100's own rule). Type into the field and the value
//! becomes what the AT reads first; the hint stays mounted — the
//! property is the field's own, the painter stops drawing it once the
//! value lands.
//!
//! The app shape is the tri-state canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one scoped
//! re-render (`ViewRoot::update`, the retained tree patched in place).
//!
//! Run with `cargo run -p alacer --example placeholders`.
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

/// The hints the *Swap hint* door rotates through — the search field's
/// placeholder, one sentence per beat.
const HINTS: [&str; 3] = [
    "Find a parts number…",
    "Search by torque spec…",
    "Type a chassis code…",
];

/// The view: the hinted field beside the bare one, the status line
/// reading the current hint back, and the *Swap hint* door — the patch
/// that pays the paint and the re-announcement together.
fn placeholders_view(hint: &FluxRwSignal<usize>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = hint.try_get_untracked().unwrap_or_default();
    let active = HINTS[current % HINTS.len()];

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The placeholder tier (ADR-0100)"));

    // The hinted field: the hint rides the `placeholder` word — the
    // create write at mount, the patch tier's door in place.
    root = root.child(
        ViewNode::element("text_input")
            .prop("name", "Search")
            .prop("placeholder", active),
    );

    // The honest absence beside it: the same role, no hint — the tree
    // announces the absence, never an empty hint.
    root = root.child(ViewNode::element("text_input").prop("name", "Notes"));

    // The status line mirrors what the tree carries for the search
    // field — the app's own truth, one write away.
    root = root.child(ViewNode::element("label").prop("text", format!("Search hint: {active}")));

    // The *Swap hint* door: the placeholder patch — the paint and the
    // re-announcement in one frame, each tier paying its own mask.
    let next_hint = *hint;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("button")
            .prop("text", "Swap hint")
            .prop("description", "Rotates the search field's hint text")
            .prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new(move || {
                    Flux::batch_of(|| {
                        let next =
                            (next_hint.try_get_untracked().unwrap_or_default() + 1) % HINTS.len();
                        next_hint.set(next);
                    });
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
    let hint = pipeline.flux().signal(0_usize);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = placeholders_view(&hint, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer placeholders",
        (360.0, 260.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = placeholders_view(&hint, &wake);
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
