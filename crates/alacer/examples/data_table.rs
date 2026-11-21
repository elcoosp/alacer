//! The M6 data-table canon (§1's definition of done) as a showcase: a
//! named grid with header cells and positioned data cells, sortable
//! columns (each header announces its `aria-sort` word — ascending,
//! descending, none — while the cells' *display* geometry renumbers),
//! row selection (the row's AccessKit `Selected` word beside a named
//! control per row), an "Add row" seam, and rows that renumber correctly
//! when removed — the a11y tree carries a *navigable* table (row/column
//! counts, cell positions), not just roles.
//!
//! The app is `alacer_widgets::canon::data_table_app` — the same builder
//! the M6 exit suite drives headlessly. The sort doors
//! (`DataTableHandles::sort_by` — first ask ascending, second descending,
//! third clears) and the selection doors (`DataTableHandles::select_row`)
//! are what the toolbar's own buttons ride. The sort is numeric-aware:
//! the Quantity column orders 5 before 12, the way a reader expects.
//!
//! Try it: click "Sort Quantity" twice to cycle ascending → descending →
//! clear, select a row and remove it, add rows. Run with
//! `cargo run -p alacer --example data_table`. `ALACER_SOFTWARE=1` forces
//! the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_widgets::data_table_app;
use std::rc::Rc;

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The canon: seeded inventory rows, headers from the builder, and the
    // mounted view (toolbar + sort bar + table + selection bar).
    let mut handles = data_table_app(
        &mut pipeline,
        vec![
            vec!["Nails".to_string(), "12".to_string(), "A1".to_string()],
            vec!["Screws".to_string(), "5".to_string(), "B2".to_string()],
            vec!["Bolts".to_string(), "40".to_string(), "C3".to_string()],
            vec!["Washers".to_string(), "3".to_string(), "D4".to_string()],
        ],
        Some(Rc::new({
            let channel = channel.clone();
            move || channel.fire()
        })),
    );

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer data table",
        (460.0, 620.0),
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
