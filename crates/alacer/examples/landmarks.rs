//! The landmark tier as a consumer's vocabulary (M143, ADR-0147): the
//! page-structure words the AT walks a document by — the banner, the
//! navigation strip, the main region, the footer.
//!
//! The beat the example exists to show: the words price the words and
//! nothing else — each landmark is a named container the page's
//! furniture rides (the label inside keeping the word it already
//! carries), the furniture never a control (it holds no focus; the
//! controls mounted inside it do), and the vocabulary is AccessKit's
//! own page-structure facts — the landmarks list a screen reader offers
//! for document-wide travel.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example landmarks`.
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

/// The view: the page's furniture, each region named — the banner, the
/// way through, the content, the colophon — and a toggle that mounts
/// and unmounts the navigation strip (the reconciler's own shape, the
/// M1 conditional-mount precedent: presence the app's wiring).
fn landmarks_view(nav_mounted: &FluxRwSignal<bool>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = nav_mounted.try_get_untracked().unwrap_or(false);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    // The banner: the page's opening strip — the masthead the AT's
    // landmarks list walks first, its content keeping the word it
    // already carries.
    root = root.child(
        ViewNode::element("banner")
            .prop("name", "The Alacer Catalog")
            .child(ViewNode::element("label").prop("text", "The Alacer Catalog")),
    );

    // The navigation: the document's way through — present exactly
    // while the app mounts it, the links keeping their own words.
    if current {
        root = root.child(
            ViewNode::element("navigation")
                .prop("name", "Catalog sections")
                .child(ViewNode::element("label").prop("text", "Widgets")),
        );
    }

    // The main: the document's central region — the landmarks list's
    // own destination.
    root = root.child(
        ViewNode::element("main")
            .prop("name", "The catalog")
            .child(ViewNode::element("label").prop("text", "Everything the vocabulary ships")),
    );

    // The content info: the page's closing strip — the colophon the
    // landmarks list walks last.
    root = root.child(
        ViewNode::element("content_info")
            .prop("name", "Catalog colophon")
            .child(ViewNode::element("label").prop("text", "The Alacer Developers")),
    );

    // The toggle: the app's wiring the presence rides.
    let toggle_label = if current { "Hide nav" } else { "Show nav" };
    let next_mounted = *nav_mounted;
    let next_wake = Rc::clone(wake);
    root = root.child(ViewNode::element("button").prop("text", toggle_label).prop(
        "on_click",
        PropValue::Handler(Rc::new(RefCell::new(move || {
            Flux::batch_of(|| next_mounted.set(!next_mounted.get_untracked()));
            (next_wake.clone())();
        }))),
    ));

    // The vocabulary reads back the way the app wired it.
    let status = if current {
        "landmarks: banner, navigation, main, content_info"
    } else {
        "landmarks: banner, main, content_info"
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
    let nav_mounted = pipeline.flux().signal(false);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = landmarks_view(&nav_mounted, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer landmarks",
        (360.0, 420.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = landmarks_view(&nav_mounted, &wake);
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
