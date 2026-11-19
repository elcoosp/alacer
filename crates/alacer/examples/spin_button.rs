//! The spin-button tier as a consumer's vocabulary (M128, ADR-0132):
//! the bounded number field the AT walks by arrows — the value the M3
//! contract carries, the span the M97 range word announces, the number
//! the M99 numeric word beside it, the discrete step the arrows and
//! the steppers speak.
//!
//! The beat the example exists to show: every write — an arrow key, a
//! stepper click, the AT's `Increment`/`SetValue` — routes through the
//! same clamp-and-snap funnel the constructor mounted, the moving step
//! lands (the tree re-announces the field), the arrow held at a wall
//! is the idle frame (nothing moves, nothing fires), and the status
//! line reads the value back the way the tree carries it.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example spin_button`.
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

/// The view: the named field the AT walks by arrows, mounted from the
/// signal, and the status line reading the value back the way the tree
/// carries it.
fn spin_button_view(quantity: &FluxRwSignal<f32>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = quantity.try_get_untracked().unwrap_or(1.0);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The spin-button tier (ADR-0132)"));

    // The field: the bounded number the doors write — the value read
    // live off the signal, the step the arrows and steppers speak, the
    // help text the tree carries beside the name.
    let next_quantity = *quantity;
    let next_wake = Rc::clone(wake);
    let field = ViewNode::element("spin_button")
        .prop("name", "Quantity")
        .prop("min", 1.0_f32)
        .prop("max", 12.0_f32)
        .prop("step", 1.0_f32)
        .prop("value", current)
        .prop("description", "How many, one to twelve")
        .prop(
            "on_change",
            PropValue::NumberHandler(Rc::new(RefCell::new({
                // The signal is `Copy` — copy it out of the reference
                // the view builder holds, no clone ceremony (the
                // pedantic rule the facade's examples answer).
                let wake = Rc::clone(&next_wake);
                move |value: f32| {
                    Flux::batch_of(|| next_quantity.set(value));
                    (wake.clone())();
                }
            }))),
        );
    root = root.child(field);

    // The value the tree carries, read back the way the app wired it.
    root = root.child(ViewNode::element("label").prop("text", format!("in the cart: {current}")));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let quantity = pipeline.flux().signal(1.0_f32);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = spin_button_view(&quantity, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer spin button",
        (360.0, 240.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = spin_button_view(&quantity, &wake);
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
