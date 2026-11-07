//! The M6 data-table canon (§1's definition of done): a named grid with
//! header cells and positioned data cells, an "Add row" seam, and rows that
//! renumber correctly when removed — the a11y tree carries a *navigable*
//! table (row/column counts, cell positions), not just roles.
//!
//! The app is `alacer_widgets::canon::data_table_app` — the same builder
//! the M6 exit suite drives headlessly. The "Add row" button writes the
//! rows signal in one batch and fires the patch channel; the loop answers
//! with one scoped re-render (ADR-6's one-frame contract).
//!
//! Run with `cargo run -p alacer --example data_table`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_widgets::data_table_app;
use std::rc::Rc;

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The canon: seeded inventory rows, headers from the builder, and the
    // mounted view (toolbar + table).
    let mut handles = data_table_app(
        &mut pipeline,
        vec![
            vec!["Nails".to_string(), "12".to_string(), "A1".to_string()],
            vec!["Screws".to_string(), "5".to_string(), "B2".to_string()],
            vec!["Bolts".to_string(), "40".to_string(), "C3".to_string()],
        ],
        Some(Rc::new({
            let channel = channel.clone();
            move || channel.fire()
        })),
    );

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer data table",
        (420.0, 420.0),
        backend_choice(),
    );
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
