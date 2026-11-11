//! The value tier in a view (§6.1, ADR-0026/ADR-0027/ADR-0038): the M20
//! slider and the M21 progress as `rsx!` elements — the value tier's two
//! reactive doors, now view vocabulary like the M6 canon before them.
//!
//! The slider's funnel routes to the app's signal through the
//! `on_change` value handler ([`alacer_view::PropValue::NumberHandler`],
//! the flag/text handlers' rule at a value); the progress reports a
//! [`RenderBinding`] the flux bridge drives — one signal write lands as
//! the label's text, the bar's paint, and the a11y tree's value, the
//! same one-batch currency every view app rides. The drag, the arrow
//! keys, and a screen reader's `SetValue` all land in the same funnel
//! (ADR-0026), and the reported bar refuses every write honestly — a
//! reporting widget has no write door (ADR-0027).
//!
//! Run with `cargo run -p alacer --example value_view`. Drag the slider
//! (or focus it and press the arrows) and watch the bar, the label, and
//! the value the tree carries move together. `ALACER_SOFTWARE=1` forces
//! the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::prelude::*;
use alacer_core::Role;
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::Set as _;

/// The view function: cheap, re-runnable, compiled by `rsx!` into a
/// [`ViewNode`] tree (§6.1). The slider's `on_change` is the value
/// handler prop ([`alacer_view::PropValue::NumberHandler`] — closures
/// with a value payload, the flag/text handlers' rule); the progress's
/// `binding` rides the f32 binding the bridge drives (the value typed by
/// what it carries).
fn value_view(
    value: alacer_core::FluxRwSignal<f32>,
    level: &RenderBinding<f32>,
    text: &RenderBinding<String>,
) -> ViewNode {
    let level = level.clone();
    let text = text.clone();
    rsx! {
        column { gap: 12.0_f32, padding: 16.0_f32,
            label { binding: text }
            slider {
                name: "Volume",
                min: 0.0_f32,
                max: 100.0_f32,
                step: 5.0_f32,
                value: 40.0_f32,
                on_change: alacer_view::PropValue::NumberHandler(std::rc::Rc::new(
                    std::cell::RefCell::new(move |v: f32| {
                        // One write batch per funnel answer: the ADR-6
                        // contract at the seam — the bridge fans the value
                        // out to the bar, the label, and the tree.
                        alacer_core::flux::Flux::batch_of(|| value.set(v));
                    }),
                )),
            }
            progress {
                name: "Level",
                min: 0.0_f32,
                max: 100.0_f32,
                binding: level,
            }
        }
    }
}

/// The arena node of the view-mounted widget with `role` — the
/// assembly-time lookup the bridge needs (the `counter_view` rule: the
/// a11y vocabulary, at assembly time).
fn node_of_role(pipeline: &alacer_core::passes::PassPipeline, role: Role) -> alacer_core::NodeId {
    let arena = pipeline.arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |id, slot| {
        if found.is_none() && slot.widget().role() == role {
            found = Some(id);
        }
    });
    found.expect("the view mounted the value tier")
}

fn main() {
    let _span = tracing::trace_span!(target: "alacer::value_view", "demo").entered();
    let mut pipeline = pipeline_default();

    let value = pipeline.flux().signal(40.0_f32);
    let level = RenderBinding::new(40.0_f32);
    let text = RenderBinding::new("Value: 40".to_string());

    let _root = alacer::mount_view(&value_view(value, &level, &text), &mut pipeline);

    // The flux bridge drives the value tier's report half: one signal
    // write ⇒ the bar's value, the label's text, the tree's answer — the
    // same fan-out the bound label rides, at a value.
    let progress_node = node_of_role(&pipeline, Role::ProgressIndicator);
    pipeline.flux_mut().bind_value(
        value,
        progress_node,
        &level,
        alacer_core::Change::PAINT.union(alacer_core::Change::A11Y),
        |v: &f32| *v,
    );
    let label_node = node_of_role(&pipeline, Role::Label);
    pipeline.flux_mut().bind_value(
        value,
        label_node,
        &text,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |v: &f32| format!("Value: {v:.0}"),
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer value tier (view edition)",
        (360.0, 220.0),
        if std::env::var_os("ALACER_SOFTWARE").is_some() {
            BackendPreference::Softbuffer
        } else {
            BackendPreference::Wgpu
        },
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
