//! The M1 counter (§18 canon, view edition): the same one-signal app as
//! `counter`, written as an `rsx!` view function — structure compiles from
//! the DSL, the label's text flows through a [`RenderBinding`], and the
//! click handler is a plain closure prop.
//!
//! Run with `cargo run -p alacer --example counter_view`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::prelude::*;
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::{GetUntracked as _, Set as _};

/// The view function: cheap, re-runnable, compiled by `rsx!` into a
/// [`ViewNode`] tree (§6.1).
fn counter_view(
    count: alacer_core::FluxRwSignal<i32>,
    binding: &RenderBinding<String>,
) -> ViewNode {
    let binding = binding.clone();
    rsx! {
        column { gap: 12.0, padding: 16.0,
            label { binding: binding }
            button {
                text: "Increment",
                on_click: move || {
                    // One write batch per click: the ADR-6 contract at the seam.
                    alacer_core::flux::Flux::batch_of(|| {
                        let current = count.try_get_untracked().unwrap_or_default();
                        count.set(current + 1);
                    });
                }
            }
        }
    }
}

fn main() {
    let mut pipeline = pipeline_default();
    let count = pipeline.flux().signal(0_i32);
    let binding = RenderBinding::new("Clicked 0 times".to_string());

    let _root = alacer::mount_view(&counter_view(count, &binding), &mut pipeline);

    // The flux bridge drives the bound label: one batch ⇒ one label repaint.
    // The target node is the label the registry mounted — the first label in
    // the arena (the a11y query path, at assembly time).
    let label_node = {
        let arena = pipeline.arena();
        let mut found = None;
        arena.for_each_pre_order(&mut |id, slot| {
            if found.is_none() && slot.widget().role() == alacer_core::Role::Label {
                found = Some(id);
            }
        });
        found.expect("the view mounted a bound label")
    };
    pipeline.flux_mut().bind_value(
        count,
        label_node,
        &binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );

    alacer::run_window(
        pipeline,
        "Alacer counter (view edition)",
        (320.0, 240.0),
        if std::env::var_os("ALACER_SOFTWARE").is_some() {
            BackendPreference::Softbuffer
        } else {
            BackendPreference::Wgpu
        },
    )
    .expect("window runs");
}
