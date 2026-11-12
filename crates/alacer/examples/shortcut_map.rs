//! The M35 keymap seam driven from the consumer side: window-level
//! shortcuts installed at assembly and fired through the real dispatch —
//! `Ctrl+Up` / `Ctrl+Down` step the count wherever the focus sits, `Home`
//! resets it, and *Swap chords* rebinds the pair live (the settings
//! story: a rebind moves in place, the displaced action never fires
//! again). The status label mirrors what the table answered, read back
//! from the app's own state.
//!
//! Run with `cargo run -p alacer --example shortcut_map`. Set
//! `RUST_LOG=alacer::keymap=debug` to name every chord as it fires;
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU
//! machines).

use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::events::{Key, KeyEvent};
use alacer_core::flux::RenderBinding;
use alacer_core::keymap::{Chord, Keymap, KeymapSeam};
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::rc::Rc;

/// The app's table behind interior mutability, wrapped in the seam: the
/// trait is the funnel's only contract, so a live-rebinding table rides
/// unchanged (the m35 suite's own shape).
struct Shared(Rc<RefCell<Keymap>>);

impl KeymapSeam for Shared {
    fn dispatch(&self, event: &KeyEvent) -> bool {
        self.0.borrow().dispatch(event)
    }
}

fn main() {
    let mut pipeline = alacer::pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let count = pipeline.flux().signal(0_i32);
    let binding = RenderBinding::new("Shortcuts fired: 0".to_string());
    let label = pipeline.add_child(root, Label::bound(binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        count,
        label,
        &binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Shortcuts fired: {n}"),
    );

    // The app's shortcuts, named like the actions they run: every fired
    // chord traces its own name on the seam's target, and the status
    // label mirrors the count the table moved.
    let table = Rc::new(RefCell::new(Keymap::new()));
    table
        .borrow_mut()
        .bind(Chord::new(Key::ArrowUp).control(), "increment", move || {
            tracing::info!(target: "alacer::shortcut_map", "the increment chord fired");
            alacer_core::flux::Flux::batch_of(|| {
                let current = count.try_get_untracked().unwrap_or_default();
                count.set(current + 1);
            });
        });
    table.borrow_mut().bind(
        Chord::new(Key::ArrowDown).control(),
        "decrement",
        move || {
            tracing::info!(target: "alacer::shortcut_map", "the decrement chord fired");
            alacer_core::flux::Flux::batch_of(|| {
                let current = count.try_get_untracked().unwrap_or_default();
                count.set(current - 1);
            });
        },
    );
    table
        .borrow_mut()
        .bind(Chord::new(Key::Home), "reset", move || {
            tracing::info!(target: "alacer::shortcut_map", "the reset chord fired");
            alacer_core::flux::Flux::batch_of(|| count.set(0));
        });
    alacer_core::keymap::install(Rc::new(Shared(Rc::clone(&table))));

    // The settings story, live: the same chords rebind to the horizontal
    // pair — the table moves in place, the displaced actions are gone,
    // and the label answers where the chords went.
    let swap_table = Rc::clone(&table);
    let _ = pipeline.add_child(
        root,
        Button::new("Swap chords (Ctrl+Up/Down → Ctrl+Left/Right)")
            .on_click(move || {
                let mut table = swap_table.borrow_mut();
                table.unbind(&Chord::new(Key::ArrowUp).control());
                table.unbind(&Chord::new(Key::ArrowDown).control());
                table.bind(
                    Chord::new(Key::ArrowLeft).control(),
                    "increment (left)",
                    move || {
                        alacer_core::flux::Flux::batch_of(|| {
                            let current = count.try_get_untracked().unwrap_or_default();
                            count.set(current + 1);
                        });
                    },
                );
                table.bind(
                    Chord::new(Key::ArrowRight).control(),
                    "decrement (right)",
                    move || {
                        alacer_core::flux::Flux::batch_of(|| {
                            let current = count.try_get_untracked().unwrap_or_default();
                            count.set(current - 1);
                        });
                    },
                );
                drop(table);
                tracing::info!(target: "alacer::shortcut_map", "the chords swapped: the horizontal pair answers now");
            })
            .boxed(),
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer shortcut map",
        (420.0, 200.0),
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
