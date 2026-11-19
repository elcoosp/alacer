//! The radio tier as a consumer's vocabulary (M124, ADR-0128): the
//! one-of-N control and the named group it reads as — the member the
//! AT hears as "checked" beside the group word and the set tier's own
//! "2 of 3", the exclusivity wired where the framework never reaches:
//! the app's handler clears the siblings through the view's
//! programmatic door, the one deselect door a radio has.
//!
//! The beat the example exists to show: every member's select ask
//! routes through the same door the click and the Space key ride —
//! the moving select lands (the tree re-announces the member), the
//! redundant ask on the already-selected member is the idle frame
//! (nothing moves, nothing fires), and the status line reads the
//! selection back the way the tree carries it.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example radio`.
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

/// The plan set the demo declares — the group's span word ("of 3")
/// and the members' places ride it.
const PLANS: [&str; 3] = ["Standard", "Express", "Overnight"];

/// The member each plan's radio sits at (one-based, the set tier's
/// convention).
fn position(plan: usize) -> f32 {
    (plan + 1) as f32
}

/// The view: the named group the set reads as, the members with their
/// selection facts read live off the signal, and the status line
/// reading the selection back the way the tree carries it.
fn radio_view(plan: &FluxRwSignal<usize>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = plan.try_get_untracked().unwrap_or(0);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The radio tier (ADR-0128)"));

    // The named group: the container the AT announces before it walks
    // the members, the span word its natural carrier.
    let mut group = ViewNode::element("radio_group")
        .prop("name", "Shipping")
        .prop("size", PLANS.len() as f32)
        .prop("gap", 6.0_f32);

    for (index, name) in PLANS.iter().enumerate() {
        // The member: the selection fact read off the signal — the
        // picked member mounts checked, the siblings mount cleared (the
        // exclusivity the app's wiring writes through the view's
        // programmatic door, never a tree-level rewrite).
        let picked = index == current;
        let mut radio = ViewNode::element("radio")
            .prop("name", *name)
            .prop("checked", picked)
            .prop("position", position(index))
            .prop(
                "description",
                "The set's member — select to make it the plan",
            );
        // The select handler: the write the exclusivity rides. The
        // signal is `Copy` — copy it out of the reference the view
        // builder holds, no clone ceremony (the pedantic rule the
        // facade's examples answer).
        let next_plan = *plan;
        let next_wake = Rc::clone(wake);
        radio = radio.prop(
            "on_select",
            PropValue::FlagHandler(Rc::new(RefCell::new(move |selected: bool| {
                if selected {
                    Flux::batch_of(|| next_plan.set(index));
                    (next_wake.clone())();
                }
            }))),
        );
        group = group.child(radio);
    }
    root = root.child(group);

    // The status line mirrors the pair the tree carries: the member's
    // name beside its place in the span — "2 of 3" said by the app.
    let place = position(current);
    root = root.child(ViewNode::element("label").prop(
        "text",
        format!("plan {}: {} of {}", current + 1, PLANS[current], place),
    ));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let plan = pipeline.flux().signal(1_usize);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = radio_view(&plan, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer radio", (360.0, 260.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = radio_view(&plan, &wake);
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
