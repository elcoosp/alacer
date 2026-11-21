//! The M6 todo canon (§1's definition of done, §5's shape) as a showcase:
//! the input commits on Enter, rows toggle through the checkbox (the
//! AccessKit `Toggled` state rides the same write batch), rows delete by
//! their own button, the footer bar filters All/Active/Completed (the
//! active filter accented), clear-completed empties the done set, and the
//! footer memo counts what is left — one write batch per interaction, one
//! scoped re-render, at most one layout+paint cycle (ADR-6).
//!
//! The app is `alacer_widgets::canon::todo_app` — the *same* builder the M6
//! exit suite drives headlessly through the a11y queries, now with the
//! filter and clear-completed doors (`TodoHandles::set_filter`,
//! `TodoHandles::clear_completed`) the footer bar's own buttons ride.
//! Every control is reachable, named, and stateful in the a11y tree: the
//! checkboxes announce `Toggled` on/off, the filter words name themselves,
//! and the rows' identities survive the filter (the keyed reconciler
//! moves survivors, it never rewrites them — the handlers carry the list
//! index, not the display one).
//!
//! Try it: type and press Enter, toggle rows, click All/Active/Completed,
//! clear the completed set. Run with `cargo run -p alacer --example todo`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_widgets::todo_app;
use std::rc::Rc;

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The canon: todos + filter signals, footer binding, and the mounted
    // view (state outside the view generation — the ADR-7 state-owner
    // contract). The wake fires the patch channel after every write
    // batch: one wake ⇒ one scoped re-render ⇒ at most one layout+paint
    // cycle.
    let mut handles = todo_app(
        &mut pipeline,
        Some(Rc::new({
            let channel = channel.clone();
            move || channel.fire()
        })),
    );

    // Seed the demo state so the footer bar's words have something to
    // filter: two open, one done — then land it with the same scoped
    // re-render the wake would fire.
    handles.add("Read the architecture's §14.4 ladder");
    handles.add("Run the a11y queries headlessly");
    handles.add("Ship the canon examples");
    handles.toggle(2);
    let _ = handles.rerender(&mut pipeline);

    let mut app = AlacerApp::new(pipeline, "Alacer todo", (360.0, 560.0), backend_choice());
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
