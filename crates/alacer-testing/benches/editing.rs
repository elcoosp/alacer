//! The editing benchmarks (§10.3): the pointer tier (ADR-0018), the word
//! tier (ADR-0019), and the clipboard tier (ADR-0020) routed through the
//! pipeline on a 1,000-node arena — each measured on the shipped
//! `TextInput`, not a probe stand-in.
//!
//! The published regression gates keep the editing tiers honest as the
//! contract grows: placing a caret must stay in the cost class of routing
//! a pointer event (the full pre-order walk, the resolved frame, the
//! hit-test inversion), moving a word must stay in the cost class of
//! moving a caret, and a clipboard round trip must stay in the cost class
//! of a key edit — a seam write or read, a sanitization, and a mask,
//! never a re-layout of the arena or a re-materialization of its a11y
//! tree.
//!
//! The benches live in the harness crate because that is where the whole
//! pipeline is assembled with the real widget set (the DAG's harness
//! layer drives every surface).

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Panel, TextInput};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

const EVENTS: usize = 100;
const FILLER: usize = 1000;
const PANEL_HEIGHT: f32 = 0.4;

/// The input's bounds, read through the a11y tree (§11.5 — the same query
/// a headless test targets through), then the click x positions spread
/// across the field's inner width so every placement moves the caret.
fn click_xs(harness: &TestHarness) -> (Vec<f32>, f32) {
    let bounds = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, "Search"))
        .and_then(|node| node.bounds)
        .expect("the input has resolved bounds");
    let y = bounds.origin.y + bounds.size.height * 0.5;
    let inner = bounds.size.width - 8.0;
    let step = inner / (alacer_core::num::usize_to_f32(EVENTS) - 1.0);
    let xs = (0..EVENTS)
        .map(|i| bounds.origin.x + 4.0 + step * alacer_core::num::usize_to_f32(i))
        .collect();
    (xs, y)
}

fn bench_editing_caret_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("editing");
    group.throughput(Throughput::Elements(
        u64::try_from(EVENTS).expect("non-negative"),
    ));

    group.bench_function("route_100_caret_clicks_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().boxed());
        // The input is the first child; the 1,000-node filler follows it —
        // the routing walk pays for the whole arena on every queued event.
        let _ = harness.add_child(root, TextInput::new("Search").boxed());
        for _ in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
        }
        harness.settle();
        let (xs, y) = click_xs(&harness);

        b.iter(|| {
            for x in &xs {
                harness.queue_pointer(PointerEvent::Down {
                    position: Point::new(*x, y),
                    button: MouseButton::Left,
                    click_count: 1,
                });
                harness.queue_pointer(PointerEvent::Up {
                    position: Point::new(*x, y),
                    button: MouseButton::Left,
                });
            }
            let report = harness.tick();
            black_box(report);
        });
    });

    group.finish();
}

/// An Alt-arrow press (the word chords; ADR-0019) — the exact event the
/// platform delivers when the user moves word-wise.
fn alt_arrow(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            alt: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

fn bench_editing_word_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("editing");
    group.throughput(Throughput::Elements(
        u64::try_from(EVENTS).expect("non-negative"),
    ));

    group.bench_function("route_100_word_moves_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().boxed());
        let node = harness.add_child(root, TextInput::new("Search").boxed());
        for _ in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
        }
        harness.settle();
        harness.pipeline_mut().request_focus(node);
        harness.settle();
        // A multi-word value: every press has a word to hop. The presses
        // alternate direction so the caret oscillates and no press ever
        // parks at a clamp doing nothing.
        harness.type_text("hello brave new world of incremental editing ");

        let right = alt_arrow(Key::ArrowRight);
        let left = alt_arrow(Key::ArrowLeft);
        b.iter(|| {
            for i in 0..EVENTS {
                let event = if i % 2 == 0 { &right } else { &left };
                let _ = harness.pipeline_mut().dispatch_key(event);
            }
            let report = harness.tick();
            black_box(report);
        });
    });

    group.finish();
}

/// A Ctrl press (the clipboard chords' command set on Linux/Windows;
/// Meta is the same tier on macOS — ADR-0020) — the exact event the
/// platform delivers when the user copies, cuts, or pastes.
fn ctrl(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

fn bench_editing_clipboard_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("editing");
    group.throughput(Throughput::Elements(
        u64::try_from(EVENTS).expect("non-negative"),
    ));

    group.bench_function("route_100_clipboard_ops_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().boxed());
        let node = harness.add_child(root, TextInput::new("Search").boxed());
        for _ in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
        }
        harness.settle();
        harness.pipeline_mut().request_focus(node);
        harness.settle();
        // A multi-word value with no line breaks: each cycle selects all,
        // copies the selection into the seam, and pastes over the
        // selection — the value round-trips through the seam and stays
        // stable, so every event does real work (a selection write, a
        // seam write, a seam read, an insertion) and none of them grows
        // the field across iterations.
        harness.type_text("hello brave new world of incremental editing ");

        let select_all = ctrl(Key::Char('a'));
        let copy = ctrl(Key::Char('c'));
        let paste = ctrl(Key::Char('v'));
        b.iter(|| {
            for i in 0..EVENTS {
                let event = match i % 3 {
                    0 => &select_all,
                    1 => &copy,
                    _ => &paste,
                };
                let _ = harness.pipeline_mut().dispatch_key(event);
            }
            let report = harness.tick();
            black_box(report);
        });
    });

    group.finish();
}

/// A Shift+Ctrl press (the redo chord; ADR-0021) — the exact event the
/// platform delivers when the user replays an undone edit.
fn ctrl_shift(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            shift: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

fn bench_editing_history_route(c: &mut Criterion) {
    let mut group = c.benchmark_group("editing");
    group.throughput(Throughput::Elements(
        u64::try_from(EVENTS).expect("non-negative"),
    ));

    group.bench_function("route_100_undo_redo_1k", |b| {
        let mut harness = TestHarness::new(800.0, 600.0);
        let root = harness.mount(Column::new().boxed());
        let node = harness.add_child(root, TextInput::new("Search").boxed());
        for _ in 0..FILLER {
            let _ = harness.add_child(
                root,
                Panel::new(PANEL_HEIGHT, alacer_widgets::theme::PANEL).boxed(),
            );
        }
        harness.settle();
        harness.pipeline_mut().request_focus(node);
        harness.settle();
        // One coalesced typing step fills the field: every undo/redo pair
        // then replays it — a full-span string surgery, a selection
        // restore, and the value+selection a11y delta — and the field
        // oscillates between the two states, so every chord does real
        // work and none of them grows the journal across iterations.
        harness.type_text("hello brave new world of incremental editing ");

        let undo = ctrl(Key::Char('z'));
        let redo = ctrl_shift(Key::Char('z'));
        b.iter(|| {
            for i in 0..EVENTS {
                let event = if i % 2 == 0 { &undo } else { &redo };
                let _ = harness.pipeline_mut().dispatch_key(event);
            }
            let report = harness.tick();
            black_box(report);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_editing_caret_route,
    bench_editing_word_route,
    bench_editing_clipboard_route,
    bench_editing_history_route
);
criterion_main!(benches);
