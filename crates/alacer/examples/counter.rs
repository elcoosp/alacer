//! The M0 counter (§18): one signal, one bound label, one button — masks
//! flowing end-to-end on one window.
//!
//! Run with `cargo run -p alacer --example counter`. One write batch per
//! click; the scheduling contract (ADR-6) guarantees one layout+paint cycle.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

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

    if let Err(error) =
        alacer::run_window(pipeline, "Alacer counter", (360.0, 200.0), backend_choice())
    {
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
