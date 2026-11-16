//! The inspector in one call (M83): attach before the first frame, print
//! the URL, run the counter — the causal stream watches the masks flow.
//!
//! Run with `cargo run -p alacer --example inspector_attach`, then open
//! the printed URL in a browser (or wire any `docs/inspector-protocol.md`
//! client to the same address). `ALACER_SOFTWARE=1` forces the softbuffer
//! path (VMs, CI, no-GPU machines).

use alacer::inspect::{InspectorOptions, attach_inspector};
use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::{GetUntracked as _, Set as _};

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let count = pipeline.flux().signal(0_i32);
    let binding = RenderBinding::new("Clicked 0 times".to_string());
    let label = pipeline.add_child(root, Label::bound(binding.clone()).boxed());
    let _ = pipeline.add_child(
        root,
        Button::new("Increment")
            .on_click(move || {
                // One write batch per click: the ADR-6 contract at the seam.
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current + 1);
                });
            })
            .boxed(),
    );

    pipeline.flux_mut().bind_value(
        count,
        label,
        &binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );

    // The assembly seam: the recorder into the observer slot, the server
    // bound, the accept loop serving — one call, before the first frame.
    // The default port is the demo canon (127.0.0.1:9217); port 0 is the
    // tests' and parallel apps' door.
    let inspector = attach_inspector(
        &mut pipeline,
        InspectorOptions::new().app("the attach demo"),
    )
    .expect("the default inspector port binds");
    println!("inspector: {} — open it in a browser", inspector.url());

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer inspector",
        (360.0, 200.0),
        backend_choice(),
    ) {
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
