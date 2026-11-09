//! The reported value as a consumer's vocabulary (§11.5, ADR-0027): the
//! value tier's read half demonstrated from the consumer side, the way
//! `slider.rs` demonstrates the write tier. A `Progress` — role
//! `progress_indicator`, mandatory name, the value riding the M3
//! `value()` contract — is the whole control: the app reports, the AT
//! reads, and nobody else writes.
//!
//! Both of the app's doors ride the same `PAINT | A11Y` currency:
//!
//! - **the bridge door** — the *Step the batch* button writes one signal
//!   in one batch; the flux bridge lands the caller's masks on the bar's
//!   node and the widget paints and reports from the binding;
//! - **the mutate seam** — the imperative door the pipeline's pass-4
//!   queue exists for: take the arena lease, write in place, request the
//!   write's own masks on the guard (Masonry `WidgetMut` discipline).
//!   The window's *Commit* button drives it; the m21 exit suite drives
//!   it headlessly.
//!
//! The workspace's idiom: `tracing` spans at the `alacer::progress`
//! target, one per route, so the write's journey is visible to any
//! subscriber. Every write that lands where the value already is queues
//! no masks — the idle-frame property, at the app seam.
//!
//! Run with `cargo run -p alacer --example progress`. `ALACER_SOFTWARE=1`
//! forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, Progress};
use alacer_core::flux::RenderBinding;
use alacer_core::mask::Change;
use reactive_graph::traits::{GetUntracked as _, Set as _};

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

    // The bar's state: the app owns the number, the bridge owns the masks,
    // the widget reads the binding — the only wiring (A1).
    let batch_signal = pipeline.flux_mut().signal(0.0_f32);
    let batch_binding = RenderBinding::new(0.0_f32);
    let _ = pipeline.add_child(root, Label::new("The progress (ADR-0027)").boxed());
    let progress = Progress::bound("Batch", 0.0, 100.0, batch_binding.clone());
    let progress_node = pipeline.add_child(root, progress.boxed());
    pipeline.flux_mut().bind_value(
        batch_signal,
        progress_node,
        &batch_binding,
        Change::PAINT.union(Change::A11Y),
        |value: &f32| *value,
    );

    // The status line mirrors what the a11y tree carries — the same
    // render binding every demo uses.
    let status_signal = pipeline.flux_mut().signal("Batch: 0 of 100".to_string());
    let status_binding = RenderBinding::new("Batch: 0 of 100".to_string());
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        Change::LAYOUT.union(Change::PAINT),
        |line: &String| line.clone(),
    );

    // The bridge door: one write batch per click, the span naming the
    // route. The bar clamps its paint; the report is the app's number.
    let step = {
        move || {
            let _route =
                tracing::trace_span!(target: "alacer::progress", "report", route = "bridge")
                    .entered();
            alacer_core::flux::Flux::batch_of(|| {
                let current = batch_signal.try_get_untracked().unwrap_or_default();
                let next = (current + 10.0).min(100.0);
                batch_signal.set(next);
                status_signal.set(format!("Batch: {next} of 100"));
            });
        }
    };
    let _ = pipeline.add_child(
        root,
        alacer_widgets::Button::new("Step the batch")
            .on_click(step)
            .boxed(),
    );

    // The mutate seam: the imperative door, walked once at startup (the
    // demo's first 10%) — the lease, the write, the masks on the guard.
    {
        let _route =
            tracing::trace_span!(target: "alacer::progress", "report", route = "mutate").entered();
        let mut guard = pipeline.arena_mut().mutate(progress_node);
        if let Some(bar) = guard.widget_mut().as_any_mut().downcast_mut::<Progress>()
            && bar.set_value(10.0)
        {
            guard.request(Change::PAINT.union(Change::A11Y));
        }
    }

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer progress",
        (420.0, 240.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
