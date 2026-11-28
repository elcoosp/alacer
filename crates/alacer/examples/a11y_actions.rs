//! The action tier as a consumer's vocabulary (M156, ADR-0161): the asks a
//! control's door answers, said by the a11y tree — the word the AT reads
//! before it asks. AccessKit's consumer derives "is this node actionable"
//! from the actions a provider declares explicitly, never from the role, so
//! until this tier every Alacer node read as un-actionable on a platform
//! adapter: the door answered, the word never said it would. Now the tree
//! carries the word and the sink maps it onto the platform's action set.
//!
//! The canon under one root: the press on the button, the value tier's
//! three asks on the slider, the text tier's three on the field, and two
//! honest absences — the progress report, whose door refuses every write
//! with a named refusal, and the plain label, which has no door at all.
//!
//! The beat the example exists to show: the word is state-aware, and it
//! prices nothing but the word — the doors answer exactly as they always
//! did. The availability door closing withdraws the button's word and
//! reopens to restore it; the read-only door narrows the field to the one
//! ask it still answers (the selection), the two writes withdrawn. The AT
//! never advertises an ask the widget will refuse.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example a11y_actions
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Progress, Slider, TextInput};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yActions, Role};
use reactive_graph::traits::Set as _;

/// The viewport the demo's beats settle into (the editing canon's shape).
const VIEWPORT_WIDTH: f32 = 520.0;
const VIEWPORT_HEIGHT: f32 = 380.0;

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

/// One control's word the way the AT reads it: the platform asks the sink
/// maps the tree's word onto — or the honest absence, said as nothing.
fn said(word: A11yActions) -> String {
    let asks: Vec<String> = alacer::a11y::advertised_actions(word)
        .map(|ask| format!("{ask:?}"))
        .collect();
    if asks.is_empty() {
        "no ask".to_owned()
    } else {
        asks.join(" + ")
    }
}

/// The words the tree says for the demo's controls, read back through the
/// snapshot — the same word the sink maps onto the platform's action set.
fn action_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let words: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| {
                    matches!(
                        n.role,
                        Role::Button | Role::Slider | Role::TextInput | Role::ProgressIndicator
                    )
                })
                .map(|n| format!("{}: {}", n.label, said(n.actions)))
                .collect()
        })
        .unwrap_or_default();
    words.join(" · ")
}

/// The app's availability door on the button: the mutate seam the app's
/// code rides — the flag moved, `PAINT | A11Y` requested on the control,
/// the re-announcement paid for. Answers whether the write moved the flag.
fn set_button_enabled(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    enabled: bool,
) -> bool {
    let mut guard = pipeline.arena_mut().mutate(node);
    let moved = if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_enabled(enabled)
    } else {
        false
    };
    if moved {
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    moved
}

/// The app's read-only door on the field, through the same seam.
fn set_field_read_only(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    read_only: bool,
) -> bool {
    let mut guard = pipeline.arena_mut().mutate(node);
    let moved = if let Some(field) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        field.set_read_only(read_only)
    } else {
        false
    };
    if moved {
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    moved
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The action tier: the asks each door answers, said by the tree (ADR-0161)")
            .boxed(),
    );

    // The controls: the word rides the tree from the first frame — the
    // press on the button, the value tier on the slider, the text tier on
    // the field, the report and the plain label the honest absence between
    // them. The handlers stay the doors they always were: the word walls
    // nothing.
    let save = pipeline.add_child(
        root,
        Button::new("Save")
            .on_click(|| tracing::info!(target: "alacer::a11y_actions", "Save pressed"))
            .boxed(),
    );
    let _ = pipeline.add_child(root, Slider::new("Volume", 0.0, 100.0).boxed());
    let name = pipeline.add_child(root, TextInput::new("Name").boxed());
    let _ = pipeline.add_child(root, Progress::new("Loading", 0.0, 100.0).boxed());
    let _ = pipeline.add_child(root, Label::new("A plain label").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = action_words(snapshot.as_ref());
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
        target: "alacer::a11y_actions",
        words = "Save: Click · Volume: Increment + Decrement + SetValue · Name: ReplaceSelectedText + SetValue + SetTextSelection · Loading: no ask",
        word = "actions",
        "beat 1: the canon says the asks its doors answer — the word the AT reads before it asks"
    );

    // Beat two: the availability door — the button disabled, its word
    // withdrawn (a closed door is no capability), then restored.
    set_button_enabled(&mut pipeline, save, false);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the availability door closed",
        action_words(snapshot.as_ref())
    ));
    tracing::info!(
        target: "alacer::a11y_actions",
        moved = "Save → no ask",
        word = "actions",
        "beat 2: the availability door withdrew the word — the AT advertises no press the widget will refuse"
    );
    set_button_enabled(&mut pipeline, save, true);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the availability door reopened",
        action_words(snapshot.as_ref())
    ));

    // Beat three: the read-only door — the field's two writes withdrawn,
    // the selection ask it still answers kept, then the field writable
    // again and its whole word restored.
    set_field_read_only(&mut pipeline, name, true);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the read-only door narrowed the field",
        action_words(snapshot.as_ref())
    ));
    tracing::info!(
        target: "alacer::a11y_actions",
        moved = "Name → SetTextSelection",
        word = "actions",
        "beat 3: the read-only door narrowed the word — the asks the field still answers, and nothing beside"
    );
    set_field_read_only(&mut pipeline, name, false);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the read-only door cleared",
        action_words(snapshot.as_ref())
    ));

    // The window: live play — the controls say their asks, every door
    // answering as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: the controls say the asks they answer — every door answers as it always did",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer a11y actions",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
