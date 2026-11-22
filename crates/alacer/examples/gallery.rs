//! The M5 examples gallery: one window exercising the whole core widget
//! vocabulary — labels, buttons with a bound counter, a text input, a
//! checkbox, a data table, the canvas's paint seam and the image tier's
//! named picture (M82/M138), panels, a scrolling list, rows, spacers —
//! so a new contributor can see everything in one runnable file (M6 adds
//! the canon widgets to the tour).
//!
//! The *entire* static layout is compiled by the `rsx!` macro (§13.1): one
//! declarative tree over the default element registry, no builder chains.
//! The catalog's elements compile to the retained widgets they name —
//! `Label`, `Button`, `TextInput`, `Checkbox`, `Table`, `Canvas`, `Image`,
//! `List`, `Panel`, `Row`, `Column`, `Spacer`, and the table's own
//! `TableRow`, `TableCell`, `ColumnHeader` — through the registry's own
//! factories (§6.1), so the DSL's vocabulary is the widget vocabulary.
//! Every dynamic value rides the one currency: handlers write signals in
//! one batch, the flux bridge queues masks, the next tick is one
//! layout+paint cycle (ADR-6).
//!
//! Run with `cargo run -p alacer --example gallery`.
//! `--dump-a11y-tree` mounts the gallery headlessly, prints the exact
//! tree a screen reader receives (role, name, state — the settled
//! `TreeUpdate`), and exits 0: no winit loop, no window, the §11
//! "accessible by construction" claim made a shareable artifact.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::prelude::*;
use alacer::widgets::theme;
use alacer_core::color::Color;
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Rect, Size};
use alacer_view::{Painter, Pixels, PropValue};
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

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

/// The scrolling list's churn floor: enough rows to scroll and to spot
/// clipping.
const GALLERY_LIST_ITEMS: usize = 20;

/// One list row's height — the panels' height and the content height's
/// unit.
const GALLERY_ROW_PX: f32 = 28.0;

/// The list rows' and the painter's bars cycle the accent ramp.
const GALLERY_RAMP: [Color; 5] = [
    Color::rgb(37, 99, 235),
    Color::rgb(14, 165, 233),
    Color::rgb(22, 163, 74),
    Color::rgb(234, 179, 8),
    Color::rgb(239, 68, 68),
];

/// The data table's inventory (headers are declared beside it in the view).
const GALLERY_INVENTORY: [[&str; 3]; 3] = [
    ["Nails", "12", "A1"],
    ["Screws", "5", "B2"],
    ["Bolts", "40", "C3"],
];

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

/// The gallery's whole static catalog as one `rsx!` tree (§13.1): the
/// styled labels, the bound labels, the buttons, the text input, the
/// checkbox, the data table (the emphasized tier riding its prop), the
/// canvas painter, the named picture, the scrolling list with its rows,
/// the spacer — compiled into `ViewNode`s against the default registry.
#[expect(
    clippy::too_many_lines,
    reason = "the gallery is one declarative walk through the widget vocabulary; splitting it would hide the catalog it exists to show"
)]
fn gallery_view(
    count: &alacer_core::FluxRwSignal<i32>,
    counter_binding: &RenderBinding<String>,
    typed: &alacer_core::FluxRwSignal<String>,
    echo_binding: &RenderBinding<String>,
    toggles: &alacer_core::FluxRwSignal<i32>,
    toggle_binding: &RenderBinding<String>,
    checker: &Arc<[u8]>,
) -> ViewNode {
    let counter_binding = counter_binding.clone();
    let echo_binding = echo_binding.clone();
    let toggle_binding = toggle_binding.clone();
    let count_signal = *count;
    let typed_signal = *typed;
    let toggles_signal = *toggles;
    let checker = Arc::clone(checker);
    // The list's rows: one `(key, color)` pair per row, the ramp cycled
    // over the churn floor — and the content height the scroll seam's
    // contract rides (rows × row height, the builder gallery's own math).
    let rows: Vec<(u64, Color)> = GALLERY_RAMP
        .iter()
        .cycle()
        .take(GALLERY_LIST_ITEMS)
        .enumerate()
        .map(|(index, color)| (u64::try_from(index).unwrap_or(u64::MAX), *color))
        .collect();
    let list_content_height =
        alacer_core::num::u32_to_f32(u32::try_from(GALLERY_LIST_ITEMS).unwrap_or(u32::MAX))
            * GALLERY_ROW_PX;

    rsx! {
        column { padding: 20.0, gap: 14.0,
            label { text: "Alacer gallery", font_size: 24.0, color: theme::TEXT }
            label {
                text: "Every core widget, one window. Type below, click, scroll.",
                font_size: 13.0,
                color: theme::TEXT_MUTED,
            }
            label { binding: counter_binding, font_size: theme::FONT_SIZE }
            row { gap: 10.0,
                button {
                    text: "Increment",
                    color: theme::ACCENT,
                    on_click: move || {
                        // One write batch per click: the ADR-6 contract at
                        // the seam — one layout+paint cycle.
                        alacer_core::flux::Flux::batch_of(|| {
                            let current = count_signal.try_get_untracked().unwrap_or_default();
                            count_signal.set(current + 1);
                        });
                    }
                }
                button {
                    text: "Reset",
                    color: theme::BORDER,
                    text_color: theme::TEXT,
                    on_click: move || {
                        alacer_core::flux::Flux::batch_of(|| count_signal.set(0_i32));
                    }
                }
            }
            label {
                binding: echo_binding,
                font_size: 13.0,
                color: theme::TEXT_MUTED,
            }
            text_input {
                name: "Gallery input",
                placeholder: "type here — IME friendly",
                on_changed: PropValue::TextHandler(Rc::new(RefCell::new(move |value: &str| {
                    alacer_core::flux::Flux::batch_of(|| typed_signal.set(value.to_string()));
                }))),
            }
            label {
                binding: toggle_binding,
                font_size: 13.0,
                color: theme::TEXT_MUTED,
            }
            checkbox {
                name: "Ship in bulk",
                on_toggle: PropValue::FlagHandler(Rc::new(RefCell::new(move |_checked: bool| {
                    alacer_core::flux::Flux::batch_of(|| {
                        let current = toggles_signal.try_get_untracked().unwrap_or_default();
                        toggles_signal.set(current + 1);
                    });
                }))),
            }
            table {
                name: "Inventory",
                columns: 3,
                header_fill: Color::rgb(30, 41, 59),
                table_row {
                    column_header { text: "Item", column: 0 }
                    column_header { text: "Quantity", column: 1 }
                    column_header { text: "Location", column: 2 }
                }
                for (index, cells) in GALLERY_INVENTORY.iter().enumerate() {
                    table_row {
                        for (column, cell) in cells.iter().enumerate() {
                            // The quantity column is the emphasized tier:
                            // header-like data cells, the `strong` prop.
                            table_cell {
                                text: *cell,
                                row: index,
                                column: column,
                                strong: column == 1,
                            }
                        }
                    }
                }
            }
            row { gap: 14.0,
                canvas {
                    name: "Gallery sine strip",
                    width: 240.0,
                    height: 120.0,
                    on_paint: PropValue::Painter(Painter::new(|ctx| {
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
                        // The wave: one filled quad per sample, the bar's
                        // height the sine's share of the rail, the color
                        // cycling the accent ramp — a procedural pattern,
                        // straight from the painter's own math.
                        let sample_count = alacer_core::num::u32_to_f32(
                            u32::try_from(GALLERY_SINE_SAMPLES).unwrap_or(u32::MAX),
                        );
                        let bar_width = size.width / sample_count;
                        let center = size.height * 0.5;
                        let amplitude = size.height * 0.35;
                        for (index, color) in GALLERY_RAMP
                            .iter()
                            .cycle()
                            .take(GALLERY_SINE_SAMPLES)
                            .enumerate()
                        {
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
                    }))
                }
                image {
                    name: "Gallery checker",
                    description: "A 32 by 32 checkerboard generated in memory, four cells of eight pixels",
                    width: 96.0,
                    height: 96.0,
                    pixels: PropValue::Pixels(Pixels::new(
                        checker,
                        GALLERY_CHECKER_PX,
                        GALLERY_CHECKER_PX,
                    ))
                }
            }
            list { height: 120.0, content_height: list_content_height,
                for (index, color) in rows.into_iter() {
                    row { gap: 8.0,
                        panel { height: 28.0, color: color }
                        label {
                            text: "item {index}",
                            font_size: 12.0,
                            color: theme::TEXT_MUTED,
                        }
                    }
                }
            }
            spacer { size: 8.0 }
            label {
                text: "labels · buttons · bindings · text input · checkbox · table · canvas · image · panels · list · rows · spacers",
                font_size: 11.0,
                color: theme::BORDER,
            }
        }
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    // The gallery's dynamics ride signals and bindings: handlers write in
    // one batch, the flux bridge queues masks, the next tick is one
    // layout+paint cycle (ADR-6).
    let count = pipeline.flux().signal(0_i32);
    let counter_binding = RenderBinding::new("Clicked 0 times".to_string());
    let typed = pipeline.flux().signal(String::new());
    let echo_binding = RenderBinding::new("echo: (empty)".to_string());
    let toggles = pipeline.flux().signal(0_i32);
    let toggle_binding = RenderBinding::new("toggled 0 times".to_string());
    let checker: Arc<[u8]> = checker_raster(GALLERY_CHECKER_CELLS, GALLERY_CHECKER_CELL_PX).into();

    // The whole catalog compiles from one rsx! tree and mounts once — the
    // bindings carry every dynamic value, no view re-run needed.
    let view = gallery_view(
        &count,
        &counter_binding,
        &typed,
        &echo_binding,
        &toggles,
        &toggle_binding,
        &checker,
    );
    let _root = alacer::mount_view(&view, &mut pipeline);

    // The flux bridge drives the three bound labels: the targets are the
    // labels the registry mounted, found by the initial text each binding
    // resolves to at assembly time (the a11y query path, at assembly).
    let counter_node = bound_label_node(&pipeline, "Clicked 0 times");
    pipeline.flux_mut().bind_value(
        count,
        counter_node,
        &counter_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );
    let echo_node = bound_label_node(&pipeline, "echo: (empty)");
    pipeline.flux_mut().bind_value(
        typed,
        echo_node,
        &echo_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |value: &String| format!("echo: {}", if value.is_empty() { "(empty)" } else { value }),
    );
    let toggle_node = bound_label_node(&pipeline, "toggled 0 times");
    pipeline.flux_mut().bind_value(
        toggles,
        toggle_node,
        &toggle_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("toggled {n} times"),
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

/// The arena node of the view-mounted label whose resolved name is the
/// binding's initial text — the assembly-time lookup the bridge needs.
fn bound_label_node(
    pipeline: &alacer_core::passes::PassPipeline,
    initial: &str,
) -> alacer_core::NodeId {
    let arena = pipeline.arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |id, slot| {
        if found.is_none()
            && slot.widget().role() == alacer_core::Role::Label
            && slot.widget().label().resolve() == initial
        {
            found = Some(id);
        }
    });
    found.expect("the view mounted the bound label")
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
        .and_then(|any| any.downcast_mut::<alacer_a11y::AccessKitSink>())
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
