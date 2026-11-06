//! The M4 hot-reload demo (§18 canon, hot edition): one signal, one bound
//! label, one button — plus a "simulate patch" button that swaps the view
//! generation live. The counter keeps counting across the patch: state
//! lives in the flux, code is the only thing a patch replaces (§13.2,
//! ADR-7).
//!
//! Run with `cargo run -p alacer --example hot_reload`. Under the
//! `hot-reload` feature the view generation is wrapped in `subsecond::call`,
//! so the very same example accepts real jump-table patches from the dioxus
//! CLI: `dx serve --hotpatch` — edit the view functions and watch the window
//! re-render sub-second, state intact, no restart.
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer::prelude::*;
use alacer_core::FluxRwSignal;
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::Cell;
use std::rc::Rc;

/// What a hot patch replaces: the view function. `0` is the shipped UI; `1`
/// stands in for "the code the patcher swapped in" — a different layout and
/// a new widget. Under `dx serve --hotpatch` the jump table performs this
/// swap at the binary level instead; the pipeline path is identical.
type Generation = u32;

fn counter_view(
    count: FluxRwSignal<i32>,
    binding: &RenderBinding<String>,
    generation: Generation,
    simulate_patch: Rc<dyn Fn()>,
) -> ViewNode {
    let binding = binding.clone();
    let increment = move || {
        // One write batch per click: the ADR-6 contract at the seam. The
        // same signal serves every generation — state outlives patches.
        alacer_core::flux::Flux::batch_of(|| {
            let current = count.try_get_untracked().unwrap_or_default();
            count.set(current + 1);
        });
    };
    match generation {
        0 => rsx! {
            column { gap: 12.0, padding: 16.0,
                label { binding: binding }
                button { text: "Increment", on_click: increment }
                button { text: "Simulate patch", on_click: move || simulate_patch() }
            }
        },
        _ => rsx! {
            column { gap: 8.0, padding: 24.0,
                label { binding: binding }
                label { text: "patched layout — same signal, same count" }
                button { text: "Still incrementing", on_click: increment }
            }
        },
    }
}

fn main() {
    let mut pipeline = pipeline_default();
    let count = pipeline.flux().signal(0_i32);
    let binding = RenderBinding::new("Clicked 0 times".to_string());

    // The generation slot and the patch channel: the two handles the
    // "simulate patch" button drives (and the subsecond handler fires).
    let generation = Rc::new(Cell::new(0_u32));
    let channel = PatchChannel::new();

    // The view function reads the slot; `mount_hot` wraps it in
    // `subsecond::call` under the `hot-reload` feature.
    let gen_reader = Rc::clone(&generation);
    let gen_writer = Rc::clone(&generation);
    let gen_binding = binding.clone();
    let patch_fire = channel.clone();
    let simulate_patch: Rc<dyn Fn()> = Rc::new(move || {
        // The patch arrives: the swapped generation stands in for the new
        // code a binary patch would install, the channel wakes the loop,
        // and the hook re-renders scoped. The counter never notices.
        gen_writer.set(1);
        patch_fire.fire();
    });
    let generate = move || {
        counter_view(
            count,
            &gen_binding,
            gen_reader.get(),
            Rc::clone(&simulate_patch),
        )
    };

    let mut hot = alacer::hot::mount_hot(&mut pipeline, generate);

    // The flux bridge drives the bound label across generations.
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

    // The patch hook: the loop answers one patch event with one scoped
    // regenerate. A real `dx serve --hotpatch` binary patch takes the same
    // path — register the handler, fire the channel, one regenerate.
    let mut app = AlacerApp::new(
        pipeline,
        "Alacer hot reload (M4)",
        (360.0, 240.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        hot.regenerate(pipeline);
    }));
    alacer::hot::register_patch_handler(&channel);

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
