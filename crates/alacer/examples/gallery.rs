//! The M5 examples gallery: one window exercising the whole core widget
//! vocabulary — labels, buttons with a bound counter, a text input, a
//! checkbox, a data table, panels, a scrolling list, rows, spacers — so a
//! new contributor can see everything in one runnable file (M6 adds the
//! canon widgets to the tour).
//!
//! Every dynamic value rides the one currency: handlers write signals in one
//! batch, the flux bridge queues masks, the next tick is one layout+paint
//! cycle (ADR-6).
//!
//! Run with `cargo run -p alacer --example gallery`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::theme;
use alacer::widgets::{
    BoxedWidget as _, Button, Checkbox, Column, ColumnHeader, Label, List, Panel, Row, Spacer,
    Table, TableCell, TableRow, TextInput,
};
use alacer_core::color::Color;
use alacer_core::flux::RenderBinding;
use alacer_core::style::CrossAxisAlignment;
use reactive_graph::traits::{GetUntracked as _, Set as _};

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
    // a11y contract live.
    let table = pipeline.add_child(root, Table::new("Inventory", 3).boxed());
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
            let _ = pipeline.add_child(row, TableCell::new(*cell, row_index, column).boxed());
        }
    }

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
            "labels · buttons · bindings · text input · checkbox · table · panels · list · rows · spacers",
        )
        .font_size(11.0)
        .color(theme::BORDER)
        .boxed(),
    );

    if let Err(error) =
        alacer::run_window(pipeline, "Alacer gallery", (460.0, 620.0), backend_choice())
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
