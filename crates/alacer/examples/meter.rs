//! The meter tier as a consumer's vocabulary (M133, ADR-0137): the
//! gauge whose reading is a measurement, said by the tree — the word
//! the AT announces as a gauge, not the progress tier's task word.
//!
//! The beat the example exists to show: the gauge rides the value
//! tier's own words — the M3 spoken reading, the M97 declared scale,
//! the M99 number behind the spoken one — and the app writes the
//! reading through the door every reported value takes: the reactive
//! bridge. One signal write fans out (one write batch ⇒ one frame) to
//! the bound gauge the bridge drives, the label the patch tier
//! re-renders, and the tree's answer.
//!
//! Run with `cargo run -p alacer --example meter`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_core::FluxRwSignal;
use alacer_core::flux::{Flux, RenderBinding};
use alacer_core::widget::Role;
use alacer_view::{PropValue, ViewNode, ViewRoot};
use alacer_widgets::default_registry;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::rc::Rc;

/// The view: the slider the app's write door rides, the bound gauge
/// the bridge drives, the reading the label mirrors — the measurement
/// the tree announces beside it.
fn meter_view(
    reading: &FluxRwSignal<f32>,
    gauge: &RenderBinding<f32>,
    wake: &Rc<dyn Fn()>,
) -> ViewNode {
    let current = reading.try_get_untracked().unwrap_or(0.0);
    let gauge = gauge.clone();

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The meter tier (ADR-0137)"));

    // The declared scale: the slider the app's write door rides — drag
    // it (or let a screen reader write it) and the funnel routes to one
    // signal, the bridge's own door.
    let next_reading = *reading;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("slider")
            .prop("name", "Simulated disk fill")
            .prop("min", 0.0_f32)
            .prop("max", 100.0_f32)
            .prop(
                "on_change",
                PropValue::NumberHandler(Rc::new(RefCell::new(move |value: f32| {
                    Flux::batch_of(|| next_reading.set(value));
                    (next_wake.clone())();
                }))),
            ),
    );

    // The bound gauge: the reading the bridge drives — a measurement on
    // the scale the app declared, the AT announcing "Disk usage, gauge"
    // with the M3 spoken reading beside the M97 scale and the M99
    // number. Never a progress bar wearing the wrong word: nothing is
    // *becoming done* here — the number is how much there is. A rebuild
    // re-hangs the same binding slot (the bound label's rule, at the
    // value tier).
    root = root.child(
        ViewNode::element("meter")
            .prop("name", "Disk usage")
            .prop("min", 0.0_f32)
            .prop("max", 100.0_f32)
            .prop("binding", gauge),
    );

    // The reading the app mirrors from its own state — the same number
    // the tree carries.
    root = root.child(ViewNode::element("label").prop("text", format!("reading: {current:.0}")));

    root
}

/// The assembly-time lookup the bridge needs (the `counter_view` rule:
/// the a11y vocabulary, at assembly time).
fn gauge_of(pipeline: &alacer_core::passes::PassPipeline, role: Role) -> alacer_core::NodeId {
    let arena = pipeline.arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |id, slot| {
        if found.is_none() && slot.widget().role() == role {
            found = Some(id);
        }
    });
    found.expect("the view mounted the meter tier")
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let reading = pipeline.flux().signal(62.0_f32);
    let gauge = RenderBinding::new(62.0_f32);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = meter_view(&reading, &gauge, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    // The flux bridge drives the gauge's reading: one signal write ⇒
    // the gauge's PAINT | A11Y masks (the M21 report half, at the meter
    // tier) — the same fan-out the bound progress rides, at a gauge.
    let gauge_node = gauge_of(&pipeline, Role::Meter);
    pipeline.flux_mut().bind_value(
        reading,
        gauge_node,
        &gauge,
        alacer_core::Change::PAINT.union(alacer_core::Change::A11Y),
        |v: &f32| *v,
    );

    let mut app = AlacerApp::new(pipeline, "Alacer meter", (360.0, 220.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = meter_view(&reading, &gauge, &wake);
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
