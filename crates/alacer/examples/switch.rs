//! The switch tier as a consumer's vocabulary (M130, ADR-0134): the
//! instant on/off control — the fact the AT announces beside the name,
//! the binary pair the tree carries (`On` or `Off`, never the mixed
//! summary a checkbox resolves).
//!
//! The beat the example exists to show: the toggle lands in one funnel —
//! a click, the Space key while focused, the AT's default action all
//! fire the same handler with the same new flag — and the status line
//! reads the on/off facts back the way the tree carries them. The
//! switches' state rides one signal each; the loop answers with exactly
//! one scoped re-render (`ViewRoot::update`, the retained tree patched
//! in place).
//!
//! Run with `cargo run -p alacer --example switch`.
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

/// The toggles the demo declares — each its own control, the on/off
/// fact the app's wiring owns.
const CONTROLS: [&str; 3] = ["Wi-Fi", "Bluetooth", "Airplane mode"];

/// The view: the switches with their on/off facts read live off the
/// signal, and the status line reading them back the way the tree
/// carries them.
fn switch_view(flags: &FluxRwSignal<Vec<bool>>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let flags_now = flags.try_get_untracked().unwrap_or_else(|| vec![false; 3]);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The switch tier (ADR-0134)"));

    for (index, name) in CONTROLS.iter().enumerate() {
        // The switch: the on/off fact read off the signal — the binary
        // pair the tree announces, never a summary. The toggle handler
        // writes the app's own flag back: the exclusivity of the state
        // is the app's wiring, the tree saying what the app wrote.
        let next_flags = *flags;
        let next_wake = Rc::clone(wake);
        root = root.child(
            ViewNode::element("switch")
                .prop("name", *name)
                .prop("on", flags_now[index])
                .prop(
                    "description",
                    "An instant toggle — the flip lands now, not on submit",
                )
                .prop(
                    "on_toggle",
                    PropValue::FlagHandler(Rc::new(RefCell::new(move |on: bool| {
                        // One write batch: the flag read, the index
                        // written, the signal set — the scheduling
                        // contract collapses it into one re-render.
                        Flux::batch_of(move || {
                            let mut flags_now = next_flags.try_get_untracked().unwrap_or_default();
                            if let Some(slot) = flags_now.get_mut(index) {
                                *slot = on;
                            }
                            next_flags.set(flags_now);
                        });
                        (next_wake.clone())();
                    }))),
                ),
        );
    }

    // The status line mirrors the facts the tree carries: each
    // control's name beside the on/off the funnel landed.
    let status = CONTROLS
        .iter()
        .zip(flags_now.iter())
        .map(|(name, on)| format!("{name}: {}", if *on { "on" } else { "off" }))
        .collect::<Vec<_>>()
        .join(" · ");
    root = root.child(ViewNode::element("label").prop("text", status));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal (the flags vector), outside the view
    // generation (the ADR-7 state-owner contract). The wake fires the
    // patch channel after every write batch: one wake ⇒ one scoped
    // re-render.
    let flags = pipeline.flux().signal(vec![true, false, false]);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = switch_view(&flags, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer switch", (360.0, 240.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = switch_view(&flags, &wake);
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
