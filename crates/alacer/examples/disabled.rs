//! The availability tier as a consumer's vocabulary (§11.5, ADR-0028):
//! what a form does when a control must refuse. A `Button` the app
//! disables until the form is ready — the AT reads the unavailable
//! control (AccessKit's `disabled` flag), Tab skips it, the pointer is a
//! wall, and the keyboard never reaches it — the refusal is announced
//! before it happens, never discovered.
//!
//! The app's reactive door rides the same `PAINT | A11Y` currency every
//! write rides: the *Enable sending* checkbox writes one signal in one
//! batch; the flux bridge lands the caller's masks on the button's node
//! and the widget paints and reports from the binding (the
//! `Progress::bound` shape, at the availability tier). The mutate seam is
//! walked once at startup — the lease, the write in place, the masks on
//! the guard — where `set_enabled` answers the move it made: the write
//! lands where the binding already has the flag, so it queues nothing —
//! the M21 idle-frame property, demonstrated at the app seam. A
//! fixed-availability control (`.enabled(false)` at construction) is the
//! same contract with the imperative door owning the flag.
//!
//! The workspace's idiom: `tracing` spans at the `alacer::disabled`
//! target, one per route, so the availability write's journey is visible
//! to any subscriber.
//!
//! Run with `cargo run -p alacer --example disabled`. `ALACER_SOFTWARE=1`
//! forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Checkbox, Column, Label};
use alacer_core::flux::{Flux, RenderBinding};
use alacer_core::mask::Change;
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

    // The send button's availability: the app owns the flag, the bridge
    // owns the masks, the widget paints and reports from the binding —
    // the only wiring (A1).
    let send_signal = pipeline.flux_mut().signal(false);
    let send_binding = RenderBinding::new(false);
    let _ = pipeline.add_child(root, Label::new("The availability (ADR-0028)").boxed());
    let send_node = pipeline.add_child(
        root,
        Button::new("Send invite")
            .enabled_binding(send_binding.clone())
            .on_click(|| {})
            .boxed(),
    );
    pipeline.flux_mut().bind_value(
        send_signal,
        send_node,
        &send_binding,
        Change::PAINT.union(Change::A11Y),
        |value: &bool| *value,
    );

    // The status line mirrors what the a11y tree carries — the same
    // render binding every demo uses.
    let status_signal = pipeline
        .flux_mut()
        .signal("Sending: unavailable — the AT reads the flag".to_string());
    let status_binding = RenderBinding::new("Sending: unavailable".to_string());
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        Change::LAYOUT.union(Change::PAINT),
        |line: &String| line.clone(),
    );

    // The bridge door: the checkbox flips availability in one write
    // batch, the span naming the route. The button's node re-announces
    // with the flag — a node-only delta, the same currency every write
    // rides.
    let _ = pipeline.add_child(
        root,
        Box::new(Checkbox::new("Enable sending").on_toggle(move |checked| {
            let _route = tracing::trace_span!(
                target: "alacer::disabled",
                "availability",
                route = "bridge"
            )
            .entered();
            Flux::batch_of(|| {
                send_signal.set(checked);
                status_signal.set(if checked {
                    "Sending: available — the AT reads the flag".to_string()
                } else {
                    "Sending: unavailable — the AT reads the flag".to_string()
                });
            });
        })),
    );

    // The mutate seam: the imperative door, walked once at startup — the
    // lease, the write in place, the write's own masks on the guard. The
    // write lands where the binding already has the flag, so it queues
    // nothing: the door answers the move it made (none) — the idle-frame
    // property, demonstrated at the app seam.
    {
        let _route =
            tracing::trace_span!(target: "alacer::disabled", "availability", route = "mutate")
                .entered();
        let mut guard = pipeline.arena_mut().mutate(send_node);
        if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>()
            && button.set_enabled(false)
        {
            guard.request(Change::PAINT.union(Change::A11Y));
        }
    }

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer availability",
        (420.0, 260.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
