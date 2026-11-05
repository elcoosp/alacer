//! The M3 text input (§10.3): the IME contract live on one window — click a
//! field, type, watch the caret; with an IME running (fcitx5/ibus on Linux),
//! compose preedit inline and commit. Tab cycles focus; the focus ring and
//! the IME request follow.
//!
//! Run with `cargo run -p alacer --example text_input`. Partial paint is on:
//! every keystroke repaints only the damaged regions (§9.3).
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let _ = pipeline.add_child(root, Label::new("Who's greeting?").boxed());
    let _ = pipeline.add_child(
        root,
        TextInput::new("First name")
            .placeholder("Type here…")
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        TextInput::new("Last name").placeholder("Tab here…").boxed(),
    );
    let _ = pipeline.add_child(root, TextInput::new("Passphrase").secure().boxed());

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer text input",
        (360.0, 280.0),
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
