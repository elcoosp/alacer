//! The slider as a consumer's vocabulary (§11.5, ADR-0026): the value
//! tier's widget demonstrated from the consumer side, the way
//! `announcements.rs` demonstrates the announcement seam. A `Slider` —
//! role `slider`, mandatory name, the value riding the M3 `value()`
//! contract — is the whole control: drag the thumb, focus it and press
//! the arrows (Home/End jump to the ends), or let a screen reader write
//! the value through the AT tier (`SetValue`, `Increment`, `Decrement`).
//!
//! The handler is the app's whole side of the contract: it receives the
//! new value and writes one signal — handlers write signals, nothing
//! mutates widgets (axiom A1) — and the status line mirrors it through
//! the same render binding every demo uses. Every write clamps to the
//! range and snaps to the step inside the widget, so the app never
//! re-implements the arithmetic the AT's write just rode.
//!
//! Run with `cargo run -p alacer --example slider`. `ALACER_SOFTWARE=1`
//! forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, Slider};
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::Set as _;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    // The status line's state: the app owns the read, the slider's
    // handler writes it, the binding mirrors it — the only bridge (A1).
    let initial = "Volume: 0".to_string();
    let status_signal = pipeline.flux_mut().signal(initial.clone());
    let status_binding = RenderBinding::new(initial);
    let _ = pipeline.add_child(root, Label::new("The slider (ADR-0026)").boxed());
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |line: &String| line.clone(),
    );

    // The value control: drag, arrows (Home/End), or the AT's write all
    // land here — one funnel inside the widget, one signal write outside.
    let slider = Slider::new("Volume", 0.0, 100.0).on_change(move |value| {
        alacer_core::flux::Flux::batch_of(|| status_signal.set(format!("Volume: {value}")));
    });
    let _ = pipeline.add_child(root, slider.boxed());

    if let Err(error) =
        alacer::run_window(pipeline, "Alacer slider", (420.0, 220.0), backend_choice())
    {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
