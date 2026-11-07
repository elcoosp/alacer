//! The M6 todo canon (§1's definition of done, §5's shape): the input
//! commits on Enter, rows toggle through the checkbox, the footer memo
//! counts what is left — one write batch per interaction, one scoped
//! re-render, at most one layout+paint cycle (ADR-6).
//!
//! The app is `alacer_widgets::canon::todo_app` — the *same* builder the M6
//! exit suite drives headlessly through the a11y queries. The reactive
//! re-render rides the patch channel: handlers write the signal and fire
//! the channel; the loop answers with exactly one scoped re-render (the
//! same one-frame path a hot patch takes, §13.2).
//!
//! Run with `cargo run -p alacer --example todo`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_widgets::todo_app;
use std::rc::Rc;

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The canon: signal + footer binding + mounted view (state outside the
    // view generation — the ADR-7 state-owner contract). The wake fires the
    // patch channel after every write batch: one wake ⇒ one scoped
    // re-render ⇒ at most one layout+paint cycle.
    let mut handles = todo_app(
        &mut pipeline,
        Some(Rc::new({
            let channel = channel.clone();
            move || channel.fire()
        })),
    );

    let mut app = AlacerApp::new(pipeline, "Alacer todo", (360.0, 480.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        handles.rerender(pipeline);
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
