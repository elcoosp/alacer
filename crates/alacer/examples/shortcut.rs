//! The shortcut tier as a consumer's vocabulary (M121, ADR-0125): the
//! chord the keymap rides, said by the a11y tree at last — AccessKit's
//! `KeyboardShortcut` property (ARIA's `aria-keyshortcuts`) beside the
//! name, the word every platform AT announces as "Save — Control+S"
//! before the app's keymap ever dispatches (the word says the chord the
//! app named; the keymap seam owns the dispatch — ADR-0043, the tier
//! never binds a key). Two controls under the word beside two honest
//! absences: the worded Save and Print, the plain Close and the plain
//! label — the control that declared no chord, never a guessed binding.
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the chord names a fact, it walls nothing (every press
//! answers as ever, the keymap's dispatch the dispatch it always was),
//! and the app's own door (`Button::set_keyboard_shortcut` through the
//! mutate seam) moves the word live — the export chord declared on a
//! plain control, the save chord withdrawn, the `PAINT | A11Y` request
//! pays the re-announcement — the write that lands where the word
//! already is queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example shortcut
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::PassPipeline;
use alacer_core::widget::Role;
use reactive_graph::traits::Set as _;

/// The viewport the demo's beats settle into (the editing canon's shape).
const VIEWPORT_WIDTH: f32 = 480.0;
const VIEWPORT_HEIGHT: f32 = 360.0;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let Ok(report) = pipeline.tick(alacer_core::geom::Size::new(
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
        )) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// The chords the tree says for the buttons, read back through the
/// snapshot — the same answer the sink maps onto AccessKit's
/// `KeyboardShortcut` property: the chord beside the name.
fn shortcut_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let words: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| n.role == Role::Button)
                .map(|n| match &n.keyboard_shortcut {
                    Some(chord) => format!("{} ({chord})", n.label),
                    None => format!("{} (no chord)", n.label),
                })
                .collect()
        })
        .unwrap_or_default();
    words.join(" · ")
}

/// The app's shortcut door: the mutate seam the app's code rides — the
/// chord moved, `PAINT | A11Y` requested on the control, the
/// re-announcement paid for. `None` clears the word.
fn set_button_chord(pipeline: &mut PassPipeline, node: alacer_core::NodeId, chord: Option<&str>) {
    let mut guard = pipeline.arena_mut().mutate(node);
    let moved = if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_keyboard_shortcut(chord.map(str::to_owned))
    } else {
        false
    };
    if moved {
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The shortcut tier: the chord the keymap rides, said by the tree (ADR-0125)")
            .boxed(),
    );

    // The controls: the declared chord rides the tree from the first
    // frame — the worded save and print under the word, the plain close
    // and the plain label the honest absence between them. The handlers
    // stay the doors they always were: the chord walls nothing.
    let save = pipeline.add_child(
        root,
        Button::new("Save")
            .keyboard_shortcut("Control+S")
            .on_click(|| tracing::info!(target: "alacer::shortcut", "Save pressed"))
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Button::new("Print")
            .keyboard_shortcut("Control+Shift+P")
            .on_click(|| tracing::info!(target: "alacer::shortcut", "Print pressed"))
            .boxed(),
    );
    let export = pipeline.add_child(
        root,
        Button::new("Export")
            .on_click(|| tracing::info!(target: "alacer::shortcut", "Export pressed"))
            .boxed(),
    );
    let _ = pipeline.add_child(root, Button::new("Close").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = shortcut_words(snapshot.as_ref());
    let status_signal = pipeline.flux_mut().signal(line.clone());
    let status_binding = RenderBinding::new(line);
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );
    tracing::info!(
        target: "alacer::shortcut",
        chords = "Save (Control+S), Print (Control+Shift+P)",
        word = "keyboard_shortcut",
        "beat 1: the declared chords ride the tree — the chord beside the name, the unworded controls silent"
    );

    // Beat two: the shortcut door — the plain Export declaring its own
    // chord and the Save chord withdrawn, the word the tree says
    // following the app's own; then the absence restored (the Export
    // chord withdrawn, the control the widget it always was).
    set_button_chord(&mut pipeline, export, Some("Control+E"));
    set_button_chord(&mut pipeline, save, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the shortcut door moved the chords",
        shortcut_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::shortcut",
        moved = "Export → Control+E, Save → no chord",
        word = "keyboard_shortcut",
        "beat 2: the app's door moved the chords the tree says — the word follows the app's own"
    );
    set_button_chord(&mut pipeline, export, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the chords",
        shortcut_words(snapshot.as_ref()),
    ));

    // The window: live play — the worded controls say the chord beside
    // the plain ones, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the controls say the chord — every door answers as it always did")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer shortcut",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
