//! The tab-panel tier as a consumer's vocabulary (M131, ADR-0135): the
//! named pane the selected page reveals — the tab tier's third word,
//! beside the member (`Tab`, M127) and the strip (`TabList`, M127).
//!
//! The beat the example exists to show: the pane is the content's
//! layout, never a second channel to it — the children keep the roles
//! and labels they already carry, the page switch swaps the pane's
//! word beside its content, and the AT reads strip, pages, and pane as
//! one named shape.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example tab_panel`.
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

/// The pages the demo declares — the strip's span, each its own pane.
const PAGES: [&str; 3] = ["Inbox", "Sent", "Drafts"];

/// The view: the named strip, the named pane, and the pages' words
/// riding one signal — the exclusivity the app's own wiring writes.
fn tab_panel_view(current: &FluxRwSignal<String>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let selected = current
        .try_get_untracked()
        .unwrap_or_else(|| "Inbox".into());

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The tab-panel tier (ADR-0135)"));

    // The named strip: the word the AT announces before it walks the
    // pages, each page its own member — the exclusivity the handler
    // writes (the framework never rewrites widgets it does not own).
    let mut strip = ViewNode::element("tab_list")
        .prop("name", "Mailboxes")
        .prop("gap", 6.0_f32);

    for name in PAGES {
        let page_label = name;
        let next_current = *current;
        let next_wake = Rc::clone(wake);
        strip = strip.child(
            ViewNode::element("tab")
                .prop("name", page_label)
                .prop("selected", page_label == selected)
                .prop(
                    "on_select",
                    PropValue::Handler(Rc::new(RefCell::new(move || {
                        // The app's wiring: select this page, and the
                        // exclusivity rides the re-render — the other
                        // pages' `selected` props recompute from the
                        // signal (the ADR-6 one-batch rule).
                        Flux::batch_of(|| next_current.set(page_label.to_string()));
                        (next_wake.clone())();
                    }))),
                ),
        );
    }
    root = root.child(strip);

    // The named pane: the word the AT announces before it reads the
    // selection's content — the pane's name and body recomputing from
    // the same signal the pages ride.
    let pane_body = format!("the {selected} page's content lives here");
    root = root.child(
        ViewNode::element("tab_panel")
            .prop("name", format!("{selected} content"))
            .prop("gap", 6.0_f32)
            .child(ViewNode::element("label").prop("text", pane_body))
            .child(
                ViewNode::element("button")
                    .prop("text", format!("Open a {selected} message"))
                    .prop(
                        "on_click",
                        PropValue::Handler(Rc::new(RefCell::new(|| {
                            // The pane's children keep their own doors —
                            // the press is the button's, never the pane's.
                        }))),
                    ),
            ),
    );

    // The selection reads back the way the app wired it.
    root =
        root.child(ViewNode::element("label").prop("text", format!("selected page: {selected}")));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let current = pipeline.flux().signal("Inbox".to_string());
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = tab_panel_view(&current, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer tab panel",
        (360.0, 280.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = tab_panel_view(&current, &wake);
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
