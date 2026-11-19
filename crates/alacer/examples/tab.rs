//! The tab tier as a consumer's vocabulary (M127, ADR-0131): the page
//! selector and the named strip it reads as — the selection the
//! selected tier's word announces (M108, ADR-0112), the exclusivity
//! wired where the framework never reaches: the app's handler clears
//! the siblings through the view's programmatic door, the one
//! deselect door a tab has.
//!
//! The beat the example exists to show: every page's select ask routes
//! through the same door the click and the Space key ride — the
//! moving select lands (the tree re-announces the page), the redundant
//! ask on the already-selected page is the idle frame (nothing moves,
//! nothing fires), and the status line reads the selection back the
//! way the tree carries it.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example tab`.
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

/// The pages the demo declares — the strip's span, the selection one
/// of them carries.
const PAGES: [&str; 3] = ["Inbox", "Sent", "Archive"];

/// The view: the named strip the pages read as, the selection facts
/// read live off the signal, and the status line reading the selection
/// back the way the tree carries it.
fn tab_view(page: &FluxRwSignal<usize>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = page.try_get_untracked().unwrap_or(0);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The tab tier (ADR-0131)"));

    // The named strip: the container the AT announces before it walks
    // the pages, the selection facts mounted from the signal — the
    // picked page mounts selected, the siblings mount unselected (the
    // exclusivity the app's wiring writes through the view's
    // programmatic door, never a tree-level rewrite).
    let mut strip = ViewNode::element("tab_list")
        .prop("name", "Mailboxes")
        .prop("gap", 6.0_f32);

    for (index, name) in PAGES.iter().enumerate() {
        let picked = index == current;
        let mut tab = ViewNode::element("tab")
            .prop("name", *name)
            .prop("selected", picked)
            .prop(
                "description",
                "The strip's page — select to make it the one shown",
            );
        // The select handler: the write the exclusivity rides. The
        // signal is `Copy` — copy it out of the reference the view
        // builder holds, no clone ceremony (the pedantic rule the
        // facade's examples answer).
        let next_page = *page;
        let next_wake = Rc::clone(wake);
        tab = tab.prop(
            "on_select",
            PropValue::FlagHandler(Rc::new(RefCell::new(move |selected: bool| {
                if selected {
                    Flux::batch_of(|| next_page.set(index));
                    (next_wake.clone())();
                }
            }))),
        );
        strip = strip.child(tab);
    }
    root = root.child(strip);

    // The page the strip's content shows — the selection the tree
    // carries, read back the way the app wired it.
    root =
        root.child(ViewNode::element("label").prop("text", format!("showing: {}", PAGES[current])));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let page = pipeline.flux().signal(0_usize);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = tab_view(&page, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer tab", (360.0, 240.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = tab_view(&page, &wake);
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
