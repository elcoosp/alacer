//! The menu tier as a consumer's vocabulary (M137, ADR-0141): the
//! named strip of menus, the summoned list, and the command entries —
//! the words the AT walks and fires.
//!
//! The beat the example exists to show: the strip names the menus (the
//! word the AT announces before it walks them), the list is present
//! exactly while the app mounts it (presence the app's wiring — the
//! reconciler's own shape, no second `visible` flag), the entries are
//! the tier's only controls (the activation funnel the button tier
//! published — the click, Enter while focused, and the AT's default
//! action landing in one door), and the words cost the words and
//! nothing else (the status line reads back the way the app wired
//! them).
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example menu_bar`.
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

/// The view: the named strip, the summoned list mounted exactly while
/// the signal says open, the entries firing the commands the app wired,
/// the reads reading back the way the tree carries them.
fn menu_view(open: &FluxRwSignal<bool>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = open.try_get_untracked().unwrap_or(false);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The menu tier (ADR-0141)"));

    // The named strip: the menus the app mounts read as its members —
    // the strip naming the set they read as, never a second channel to
    // them.
    let mut strip = ViewNode::element("menu_bar").prop("name", "Main");

    // The summoned list, present exactly while the app mounts it: the
    // named list the AT announces while it is up, the entries its own
    // children — the list their layout, never a second channel to them.
    if current {
        let next_open = *open;
        let next_wake = Rc::clone(wake);
        let close_open = *open;
        let close_wake = Rc::clone(wake);
        let list = ViewNode::element("menu")
            .prop("name", "File")
            .child(ViewNode::element("menu_item").prop("text", "New").prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new(move || {
                    Flux::batch_of(|| next_open.set(false));
                    (next_wake.clone())();
                }))),
            ))
            .child(
                ViewNode::element("menu_item")
                    .prop("text", "Close menu")
                    .prop(
                        "on_click",
                        PropValue::Handler(Rc::new(RefCell::new(move || {
                            Flux::batch_of(|| close_open.set(false));
                            (close_wake.clone())();
                        }))),
                    ),
            );
        strip = strip.child(list);
    }

    // The opener: the control whose wiring summons the list — the
    // has-popup word (M115) naming what it opens, the presence the
    // app's own.
    let toggle_label = if current { "Close File" } else { "Open File" };
    let next_open = *open;
    let next_wake = Rc::clone(wake);
    root = root.child(ViewNode::element("button").prop("text", toggle_label).prop(
        "on_click",
        PropValue::Handler(Rc::new(RefCell::new(move || {
            Flux::batch_of(|| next_open.set(!next_open.get_untracked()));
            (next_wake.clone())();
        }))),
    ));

    root = root.child(strip);

    // The presence reads back the way the app wired it.
    let status = if current {
        "File: open"
    } else {
        "File: closed"
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
    let open = pipeline.flux().signal(false);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = menu_view(&open, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer menu bar",
        (360.0, 240.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = menu_view(&open, &wake);
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
