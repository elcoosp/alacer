//! The M0 scrolling list (§18): 24 panels in a clipped viewport; wheel events
//! compose as translation-only updates — zero measure calls per scroll.
//!
//! Run with `cargo run -p alacer --example scrolling_list`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{Column, List, Panel};
use alacer_core::color::Color;

const ITEM_HEIGHT: f32 = 28.0;
const ITEM_COUNT: usize = 24;

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().padding(12.0).boxed());

    let list = List::new(200.0);
    let handle = list.handle();
    let list_node = pipeline.add_child(root, list.boxed());

    let colors = [
        Color::rgb(239, 68, 68),
        Color::rgb(59, 130, 246),
        Color::rgb(34, 197, 94),
        Color::rgb(234, 179, 8),
    ];
    for color in colors.iter().cycle().take(ITEM_COUNT) {
        let _ = pipeline.add_child(list_node, Panel::new(ITEM_HEIGHT, *color).boxed());
    }
    handle
        .set_content_height(f32::from(u16::try_from(ITEM_COUNT).unwrap_or(u16::MAX)) * ITEM_HEIGHT);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer scrolling list",
        (280.0, 320.0),
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
