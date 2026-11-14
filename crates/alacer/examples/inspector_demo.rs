//! The M4 Causality inspector demo (§13.3, ADR-0010): the counter app with
//! the inspector riding on it. Every frame's pass ledger, dirty-node mask
//! heatmap, edge activations, and provenance-tagged writes stream over the
//! documented WS protocol to the embedded single-file client.
//!
//! Run with `cargo run -p alacer --example inspector_demo`, then open the
//! printed URL in a browser. Click the button and watch the causal chain:
//! `write "user click"` → `edge #1 … queued [layout+paint]` → the frame's
//! ledger and heatmap.
//!
//! Overhead is opt-in: without the observer nothing records; with it every
//! buffer is bounded (ADR-0010, §14.3). Since M64 (ADR-0070) the observer
//! slot is shared: the tee fans the stream out to the recorder (the wire)
//! and the trail's provided reader (in-process, ADR-0069) at once.

use alacer::inspect::{CausalRecorder, InspectorServer};
use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::Tee;
use alacer_core::trail::TrailLog;
use reactive_graph::traits::GetUntracked as _;

const INSPECTOR_ADDR: &str = "127.0.0.1:9217";

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let count = pipeline.flux().signal(0_i32);
    let binding = RenderBinding::new("Clicked 0 times".to_string());
    let label = pipeline.add_child(root, Label::bound(binding.clone()).boxed());
    // The provenance-tagged write handle: the click handler captures this,
    // not the pipeline (§13.3 "who wrote this signal").
    let writer = pipeline.flux().writer();
    let _ = pipeline.add_child(
        root,
        Button::new("Increment")
            .on_click(move || {
                // One write batch per click (ADR-6); the write declares its
                // origin and the inspector logs it before the edge fires.
                alacer_core::flux::Flux::batch_of(|| {
                    let next = count.try_get_untracked().unwrap_or_default() + 1;
                    writer.write(&count, next, "user click");
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

    // The inspector: the recorder observes the pipeline, the server streams
    // it (§13.3's "standalone app speaking a documented WS protocol"). The
    // observer's tee (ADR-0070) shares the single slot with the trail's
    // provided reader (ADR-0069): the recorder keeps the wire, the app
    // holds an in-process reading handle — one installation, both doors.
    let recorder = CausalRecorder::new("inspector-demo");
    let trail = TrailLog::new();
    pipeline.set_observer(Box::new(Tee::new(
        Box::new(recorder.clone_handle()),
        Box::new(trail.clone()),
    )));
    pipeline.flux().set_causal_capture(true);
    pipeline.set_timings(true);
    let server = InspectorServer::bind(INSPECTOR_ADDR, recorder, "inspector-demo")
        .expect("the inspector address is free (or set ALACER_INSPECT_ADDR)");
    let url = server.url();
    println!("Causality inspector: {url}");
    println!("  ws protocol: {url}ws  (docs/inspector-protocol.md, version 1)");
    std::thread::spawn(move || server.serve_forever());

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer Causality inspector demo",
        (360.0, 200.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the in-process door, read back — the facts the
    // tee forwarded while the recorder carried the wire (ADR-0070's
    // demonstration).
    let read_back = trail.drain();
    println!(
        "the session streamed {} facts since the last read — the provided reader carried them in-process",
        read_back.len()
    );
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
