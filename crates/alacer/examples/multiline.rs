//! The multiline tier as a consumer's vocabulary (M122, ADR-0126): the
//! field's line capacity, said by the a11y tree at last — AccessKit's
//! `MultilineTextInput` role (ARIA's `aria-multiline` beside the
//! `textbox` role) beside the name, the word every platform AT
//! announces as "edit text, multiline" before the user types a line
//! (the word says the capacity the app declared; the field's typing
//! doors own the behavior — the tier never changes what the field
//! takes). One field under the word beside two honest absences: the
//! worded Notes, the plain Title and the plain label — the field that
//! declared no capacity, never a guessed "multiline".
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the capacity names a fact, it walls nothing (every
//! door answers as ever, the typing and submit routes the widget's own),
//! and the app's own door (`TextInput::set_multiline` through the
//! mutate seam) moves the word live — the plain Summary declaring its
//! own capacity, the Notes capacity withdrawn, the `PAINT | A11Y`
//! request pays the re-announcement — the write that lands where the
//! word already is queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example multiline
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
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

/// The capacity words the tree says for the fields, read back through
/// the snapshot — the same answer the sink resolves onto AccessKit's
/// `MultilineTextInput` role: the capacity beside the name.
fn multiline_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let words: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| n.role == Role::TextInput)
                .map(|n| {
                    if n.multiline {
                        format!("{} (multiline)", n.label)
                    } else {
                        format!("{} (single-line)", n.label)
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    words.join(" · ")
}

/// The app's multiline door: the mutate seam the app's code rides — the
/// capacity moved, `PAINT | A11Y` requested on the control, the
/// re-announcement paid for. `false` clears the word.
fn set_field_multiline(pipeline: &mut PassPipeline, node: alacer_core::NodeId, multiline: bool) {
    let mut guard = pipeline.arena_mut().mutate(node);
    let moved = if let Some(field) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        field.set_multiline(multiline)
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
        Label::new("The multiline tier: the field's capacity, said by the tree (ADR-0126)").boxed(),
    );

    // The fields: the declared capacity rides the tree from the first
    // frame — the worded notes under the word, the plain title and the
    // plain label the honest absence between them. The handlers stay the
    // doors they always were: the capacity walls nothing.
    let notes = pipeline.add_child(
        root,
        TextInput::new("Notes")
            .multiline()
            .placeholder("What happened today?")
            .on_changed(|_| tracing::info!(target: "alacer::multiline", "Notes edited"))
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        TextInput::new("Title")
            .placeholder("One line, the title's own")
            .on_changed(|_| tracing::info!(target: "alacer::multiline", "Title edited"))
            .boxed(),
    );

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = multiline_words(snapshot.as_ref());
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
        target: "alacer::multiline",
        words = "Notes (multiline), Title (single-line)",
        word = "multiline",
        "beat 1: the declared capacity rides the tree — the word beside the name, the unworded fields silent"
    );

    // Beat two: the multiline door — the plain Title declaring its own
    // capacity and the Notes capacity withdrawn, the word the tree says
    // following the app's own; then the absence restored (the Title
    // capacity withdrawn, the field the single-line widget it always
    // was).
    set_field_multiline(&mut pipeline, notes, false);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the multiline door moved the words",
        multiline_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::multiline",
        moved = "Notes → single-line",
        word = "multiline",
        "beat 2: the app's door moved the capacity the tree says — the word follows the app's own"
    );

    let summary = pipeline.add_child(root, TextInput::new("Summary").boxed());
    set_field_multiline(&mut pipeline, summary, true);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the words",
        multiline_words(snapshot.as_ref()),
    ));

    // The window: live play — the worded field says the capacity beside
    // the plain ones, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the fields say the capacity — every door answers as it always did")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer multiline",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
