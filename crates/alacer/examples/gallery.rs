//! The M5 examples gallery: one window exercising the whole core widget
//! vocabulary — labels, buttons with a bound counter, a text input, a
//! checkbox, a data table, the canvas's paint seam and the image tier's
//! named picture (M82/M138), panels, a scrolling list, rows, spacers —
//! so a new contributor can see everything in one runnable file (M6 adds
//! the canon widgets to the tour).
//!
//! Every dynamic value rides the one currency: handlers write signals in one
//! batch, the flux bridge queues masks, the next tick is one layout+paint
//! cycle (ADR-6).
//!
//! Run with `cargo run -p alacer --example gallery`.
//! `--dump-a11y-tree` mounts the gallery headlessly, prints the exact
//! tree a screen reader receives (role, name, state — the settled
//! `TreeUpdate`), and exits 0: no winit loop, no window, the §11
//! "accessible by construction" claim made a shareable artifact.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::theme;
use alacer::widgets::{
    BoxedWidget as _, Button, Canvas, Checkbox, Column, ColumnHeader, Image, Label, List, Panel,
    Row, Spacer, Table, TableCell, TableRow, TextInput,
};
use alacer_a11y::AccessKitSink;
use alacer_core::color::Color;
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::style::CrossAxisAlignment;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::collections::{HashMap, HashSet};

/// The image tier's in-memory raster shape: 4 cells of 8 px make the
/// 32×32 checkerboard the gallery presents.
const GALLERY_CHECKER_CELLS: u32 = 4;
const GALLERY_CHECKER_CELL_PX: u32 = 8;
const GALLERY_CHECKER_PX: u32 = GALLERY_CHECKER_CELLS * GALLERY_CHECKER_CELL_PX;

/// The sine strip's sample count: the bars the canvas painter draws.
const GALLERY_SINE_SAMPLES: usize = 48;

/// The dump path's settle budget: at most this many headless ticks before
/// the tree is taken as it stands (the a11y demos' settle shape — the
/// first tick builds the full tree, the next reports idle).
const GALLERY_DUMP_SETTLE_TICKS: usize = 16;

/// The dump path's viewport: the same window shape the gallery opens
/// live, so the settled tree is the one the real window would hand over.
const GALLERY_DUMP_VIEWPORT: Size = Size::new(480.0, 780.0);

/// Builds the gallery's procedural raster: a `cells × cells` checkerboard
/// of `cell_px` squares, premultiplied RGBA8, row-major — every cell
/// opaque (a = 255), so the premultiplied bytes equal the straight ones,
/// the byte contract the image primitive documents.
fn checker_raster(cells: u32, cell_px: u32) -> Vec<u8> {
    let edge = cells * cell_px;
    let mut bytes = Vec::with_capacity((edge * edge * 4) as usize);
    for y in 0..edge {
        for x in 0..edge {
            let light = ((x / cell_px) + (y / cell_px)).is_multiple_of(2);
            let (r, g, b) = if light { (31, 122, 77) } else { (24, 24, 30) };
            bytes.extend_from_slice(&[r, g, b, 255]);
        }
    }
    bytes
}

#[expect(
    clippy::too_many_lines,
    reason = "the gallery is one declarative walk through the widget vocabulary; splitting it would hide the catalog it exists to show"
)]
fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(
        Column::new()
            .padding(20.0)
            .gap(14.0)
            .align(CrossAxisAlignment::Stretch)
            .boxed(),
    );

    // Labels: the styled text vocabulary — title, body, muted caption.
    let _ = pipeline.add_child(
        root,
        Label::new("Alacer gallery")
            .font_size(24.0)
            .color(theme::TEXT)
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Label::new("Every core widget, one window. Type below, click, scroll.")
            .font_size(13.0)
            .color(theme::TEXT_MUTED)
            .boxed(),
    );

    // Buttons + a bound label: the scheduling contract live. One write batch
    // per click coalesces into exactly one layout+paint cycle (ADR-6).
    let count = pipeline.flux().signal(0_i32);
    let counter_binding = RenderBinding::new("Clicked 0 times".to_string());
    let counter_label = pipeline.add_child(
        root,
        Label::bound(counter_binding.clone())
            .font_size(theme::FONT_SIZE)
            .boxed(),
    );
    let actions = pipeline.add_child(root, Row::new().gap(10.0).boxed());
    let _ = pipeline.add_child(
        actions,
        Button::new("Increment")
            .colors(theme::ACCENT, theme::ACCENT_PRESSED)
            .on_click(move || {
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current + 1);
                });
            })
            .boxed(),
    );
    let _ = pipeline.add_child(
        actions,
        Button::new("Reset")
            .colors(theme::BORDER, theme::BORDER)
            .text_color(theme::TEXT)
            .on_click(move || {
                alacer_core::flux::Flux::batch_of(|| count.set(0_i32));
            })
            .boxed(),
    );

    pipeline.flux_mut().bind_value(
        count,
        counter_label,
        &counter_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );

    // The text input: the IME contract's reference widget (UTF-16 caret,
    // inline preedit, commit). Its echo rides a signal too — effects never
    // mutate widgets; they queue masks (axiom A1).
    let typed = pipeline.flux().signal(String::new());
    let echo_binding = RenderBinding::new("echo: (empty)".to_string());
    let echo_label = pipeline.add_child(
        root,
        Label::bound(echo_binding.clone())
            .font_size(13.0)
            .color(theme::TEXT_MUTED)
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        TextInput::new("Gallery input")
            .placeholder("type here — IME friendly")
            .on_changed(move |value: &str| {
                alacer_core::flux::Flux::batch_of(|| typed.set(value.to_string()));
            })
            .boxed(),
    );
    pipeline.flux_mut().bind_value(
        typed,
        echo_label,
        &echo_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |value: &String| format!("echo: {}", if value.is_empty() { "(empty)" } else { value }),
    );

    // The checkbox (M6): click, Space, or the a11y default action all toggle
    // it; the count rides a signal in one batch.
    let toggles = pipeline.flux().signal(0_i32);
    let toggle_binding = RenderBinding::new("toggled 0 times".to_string());
    let toggle_label = pipeline.add_child(
        root,
        Label::bound(toggle_binding.clone())
            .font_size(13.0)
            .color(theme::TEXT_MUTED)
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Checkbox::new("Ship in bulk")
            .on_toggle(move |_checked| {
                alacer_core::flux::Flux::batch_of(|| {
                    let current = toggles.try_get_untracked().unwrap_or_default();
                    toggles.set(current + 1);
                });
            })
            .boxed(),
    );
    pipeline.flux_mut().bind_value(
        toggles,
        toggle_label,
        &toggle_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("toggled {n} times"),
    );

    // The data table (M6): headers + positioned cells, the navigable-grid
    // a11y contract live. The header row rides the builder's own fill, and
    // one emphasized data cell shows the strong tier (the builder
    // vocabulary the canon's `header_fill`/`strong` carry).
    let table = pipeline.add_child(
        root,
        Table::new("Inventory", 3)
            .header_fill(Color::rgb(30, 41, 59))
            .boxed(),
    );
    let header_row = pipeline.add_child(table, TableRow::new().boxed());
    for (column, name) in ["Item", "Quantity", "Location"].iter().enumerate() {
        let _ = pipeline.add_child(header_row, ColumnHeader::new(*name, column).boxed());
    }
    for (row_index, row_data) in [
        ["Nails", "12", "A1"],
        ["Screws", "5", "B2"],
        ["Bolts", "40", "C3"],
    ]
    .iter()
    .enumerate()
    {
        let row = pipeline.add_child(table, TableRow::new().boxed());
        for (column, cell) in row_data.iter().enumerate() {
            let cell = TableCell::new(*cell, row_index, column);
            // The quantity column is the emphasized tier: header-like
            // data cells, the `strong` builder's own styling.
            let cell = if column == 1 { cell.strong(true) } else { cell };
            let _ = pipeline.add_child(row, cell.boxed());
        }
    }

    // The canvas (M82) + the image tier (M138): the app's own paint
    // closure and the app's own pixels, live in the canonical consumer
    // example. The painter draws a procedural sine strip through
    // `push_quad` (no signals, no assets — the painter's own math), the
    // image presents a 32×32 checkerboard generated in memory through
    // `Image::pixels` — both primitives the isolated harness tests prove,
    // now riding the same scene journal as the core widget set above.
    let raster_row = pipeline.add_child(root, Row::new().gap(14.0).boxed());
    let _ = pipeline.add_child(
        raster_row,
        Canvas::new("Gallery sine strip")
            .size(240.0, 120.0)
            .painter(|ctx| {
                let size = ctx.size();
                let palette = ctx.palette();
                // The rail: a recessed surface the wave rides.
                ctx.push_quad(alacer_core::scene::Quad {
                    rect: Rect::new(Point::new(0.0, 0.0), size),
                    color: palette.surface,
                    corner_radius: 8.0,
                    border_color: palette.border,
                    border_width: 1.0,
                    clip: None,
                });
                // The wave: one filled quad per sample, the bar's height
                // the sine's share of the rail, the color cycling the
                // accent ramp — a procedural pattern, straight from the
                // painter's own math.
                let sample_count = alacer_core::num::u32_to_f32(
                    u32::try_from(GALLERY_SINE_SAMPLES).unwrap_or(u32::MAX),
                );
                let bar_width = size.width / sample_count;
                let center = size.height * 0.5;
                let amplitude = size.height * 0.35;
                let ramp = [
                    Color::rgb(37, 99, 235),
                    Color::rgb(14, 165, 233),
                    Color::rgb(22, 163, 74),
                    Color::rgb(234, 179, 8),
                    Color::rgb(239, 68, 68),
                ];
                for (index, color) in ramp.iter().cycle().take(GALLERY_SINE_SAMPLES).enumerate() {
                    let scaled =
                        alacer_core::num::u32_to_f32(u32::try_from(index).unwrap_or(u32::MAX));
                    let t = scaled / sample_count;
                    let share = (t * std::f32::consts::TAU).sin() * 0.5 + 0.5;
                    let bar_height = amplitude * share * 2.0;
                    ctx.push_quad(alacer_core::scene::Quad::filled(
                        Rect::new(
                            Point::new(scaled * bar_width + 1.0, center - bar_height * 0.5),
                            Size::new((bar_width - 2.0).max(1.0), bar_height),
                        ),
                        *color,
                    ));
                }
            })
            .boxed(),
    );

    // The image tier (M138): the named picture the AT announces as a
    // picture, its raster generated in memory (no embedded asset needed)
    // and passed through `Image::pixels` — opaque premultiplied RGBA8,
    // the byte contract the scene journal's image primitive spells, the
    // crisp `Nearest` default sampling it up.
    let checker: std::sync::Arc<[u8]> =
        checker_raster(GALLERY_CHECKER_CELLS, GALLERY_CHECKER_CELL_PX).into();
    let _ = pipeline.add_child(
        raster_row,
        Image::new("Gallery checker")
            .description("A 32 by 32 checkerboard generated in memory, four cells of eight pixels")
            .width(96.0)
            .height(96.0)
            .pixels(checker, GALLERY_CHECKER_PX, GALLERY_CHECKER_PX)
            .boxed(),
    );

    // The scrolling list: translation-only wheel updates, zero measure calls
    // per scroll tick (§8.2). Panel + label rows cycle the accent ramp.
    let list = List::new(120.0);
    let handle = list.handle();
    let list_node = pipeline.add_child(root, list.boxed());
    let ramp = [
        Color::rgb(37, 99, 235),
        Color::rgb(14, 165, 233),
        Color::rgb(22, 163, 74),
        Color::rgb(234, 179, 8),
        Color::rgb(239, 68, 68),
    ];
    // The list's churn floor: enough rows to scroll and to spot clipping.
    let items: usize = 20;
    for (index, color) in ramp.iter().cycle().take(items).enumerate() {
        let row = pipeline.add_child(list_node, Row::new().gap(8.0).boxed());
        let _ = pipeline.add_child(row, Panel::new(28.0, *color).boxed());
        let _ = pipeline.add_child(
            row,
            Label::new(format!("item {index}"))
                .font_size(12.0)
                .color(theme::TEXT_MUTED)
                .boxed(),
        );
    }
    handle.set_content_height(
        alacer_core::num::u32_to_f32(u32::try_from(items).unwrap_or(u32::MAX)) * 28.0,
    );

    // Spacer: the flexible gap vocabulary.
    let _ = pipeline.add_child(root, Spacer::new(8.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "labels · buttons · bindings · text input · checkbox · table · canvas · image · panels · list · rows · spacers",
        )
        .font_size(11.0)
        .color(theme::BORDER)
        .boxed(),
    );

    // The virality path: `--dump-a11y-tree` never opens a window — the
    // gallery mounts, settles headlessly, and prints the tree a screen
    // reader receives (the §11 contract as a copy-pasteable artifact).
    if std::env::args().any(|arg| arg == "--dump-a11y-tree") {
        dump_a11y_tree(&mut pipeline);
        return;
    }

    if let Err(error) =
        alacer::run_window(pipeline, "Alacer gallery", (480.0, 780.0), backend_choice())
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

/// The `--dump-a11y-tree` path: mount (already done), settle headlessly,
/// downcast the pipeline's sink to the concrete adapter the facade
/// installed (the platform drain's own [`A11ySink::as_any_mut`] seam),
/// take the settled full-tree update, and print it — role, name, state,
/// in the update's own child order — then exit 0. No winit loop to park:
/// the window is never opened, so the teardown is immediate.
///
/// The printed artifact is what a screen reader is handed on first
/// connection: the whole gallery — labels, buttons, the bound counter,
/// the text input, the checkbox, the data table, the canvas, the image,
/// the list — named and role'd, straight from the produced update.
fn dump_a11y_tree(pipeline: &mut alacer_core::passes::PassPipeline) {
    for _ in 0..GALLERY_DUMP_SETTLE_TICKS {
        let report = pipeline
            .tick(GALLERY_DUMP_VIEWPORT)
            .expect("the gallery lays out headlessly");
        if report.is_idle() {
            break;
        }
    }
    let sink = pipeline
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<AccessKitSink>())
        .expect("pipeline_default installs the AccessKitSink");
    let Some(update) = sink.full_tree_update() else {
        eprintln!("alacer: the a11y sink never settled a tree — nothing to dump");
        std::process::exit(1);
    };
    println!(
        "a11y tree dump: {} nodes across {} produced update(s)",
        update.nodes.len(),
        sink.update_count()
    );
    // The update is rendered through plain std data, so this example
    // names no accesskit type — the facade stays adapter-independent
    // (ADR-0015) while the dump stays honest to the produced update.
    let mut lines: HashMap<u64, String> = HashMap::new();
    let mut children: HashMap<u64, Vec<u64>> = HashMap::new();
    for (id, node) in &update.nodes {
        let name = match node.label() {
            Some(name) => format!(" name={name:?}"),
            None => " name=(unnamed)".to_string(),
        };
        let mut states: Vec<String> = Vec::new();
        if let Some(toggled) = node.toggled() {
            states.push(format!("toggled={toggled:?}"));
        }
        for (set, flag) in [
            (node.is_disabled(), "disabled"),
            (node.is_selected().unwrap_or_default(), "selected"),
            (node.is_expanded().unwrap_or_default(), "expanded"),
            (node.is_required(), "required"),
        ] {
            if set {
                states.push(flag.to_string());
            }
        }
        let state = if states.is_empty() {
            String::new()
        } else {
            format!(" state={}", states.join("|"))
        };
        lines.insert(id.0, format!("[{}] {:?}{name}{state}", id.0, node.role()));
        children.insert(id.0, node.children().iter().map(|child| child.0).collect());
    }
    // The root is the one node nobody names as a child — settled from the
    // child sets, no guessing at id magnitudes (the ids are hashes, so
    // "smallest id" is not the root).
    let child_ids: HashSet<u64> = children.values().flatten().copied().collect();
    let roots: Vec<u64> = lines
        .keys()
        .copied()
        .filter(|id| !child_ids.contains(id))
        .collect();
    if roots.len() != 1 {
        eprintln!(
            "alacer: the settled tree has {} roots — refusing to guess",
            roots.len()
        );
        std::process::exit(1);
    }
    print_a11y_subtree(&lines, &children, roots[0], 0);
}

/// Prints one node and its descendants from the plain-data shape
/// [`dump_a11y_tree`] built — the readable, copy-pasteable traversal.
fn print_a11y_subtree(
    lines: &HashMap<u64, String>,
    children: &HashMap<u64, Vec<u64>>,
    id: u64,
    depth: usize,
) {
    if let Some(line) = lines.get(&id) {
        println!("{}{}", "  ".repeat(depth), line);
    }
    for child in children.get(&id).into_iter().flatten() {
        print_a11y_subtree(lines, children, *child, depth + 1);
    }
}
