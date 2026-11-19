//! The link tier as a consumer's vocabulary (M125, ADR-0129): the
//! hyperlink control and its activation verb — the "opens something"
//! fact the tree carries (`Role::Link`, AccessKit's own `Link` role),
//! the open act the app's navigation rides (the handler), the
//! keyboard's verb the link's own (Enter; Space stays the scroller's
//! key the framework hands on).
//!
//! The beat the example exists to show: every activation routes
//! through the same door the click and the AT's ask ride — the status
//! line reading the destination back the way the app named it, the
//! navigation the app's own (the tree says "opens", never where).
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example link`.
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

/// The view: the two links the document canon mounts (the described
/// one beside the plain one), the status line reading the opened
/// destination back the way the app named it.
fn link_view(opened: &FluxRwSignal<&'static str>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = opened.try_get_untracked().unwrap_or("nothing yet");

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The link tier (ADR-0129)"));

    // The described link: the destination the name is too short to
    // carry rides the description word (M95, ADR-0099).
    let mut root = root.child(
        ViewNode::element("link")
            .prop("name", "Read the architecture")
            .prop(
                "description",
                "Opens the design axioms and the pass contract",
            )
            .prop(
                "on_open",
                PropValue::Handler(Rc::new(RefCell::new({
                    let opened = *opened;
                    let wake = Rc::clone(wake);
                    move || {
                        Flux::batch_of(|| opened.set("the architecture"));
                        (wake.clone())();
                    }
                }))),
            ),
    );

    // The plain link: the honest absence beside the description tier —
    // the name the only word the tree says.
    root = root.child(
        ViewNode::element("link")
            .prop("name", "API reference")
            .prop(
                "on_open",
                PropValue::Handler(Rc::new(RefCell::new({
                    let opened = *opened;
                    let wake = Rc::clone(wake);
                    move || {
                        Flux::batch_of(|| opened.set("the API reference"));
                        (wake.clone())();
                    }
                }))),
            ),
    );

    // The status line mirrors the activation the tree carries: the
    // destination the app's navigation rode, said by the app.
    root = root.child(ViewNode::element("label").prop("text", format!("opened {current}")));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let opened = pipeline.flux().signal("nothing yet");
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = link_view(&opened, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer link", (360.0, 220.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = link_view(&opened, &wake);
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
