//! The language tier as a consumer's vocabulary (M123, ADR-0127): the
//! language the text is written in, said by the a11y tree at last —
//! AccessKit's `Language` property (the BCP 47 tag the platform AT
//! voices the text with) beside the name, the word every multilingual
//! vocabulary announces as "this label is French" before it speaks the
//! text (the word says the language the app declared; the text's read
//! routes the widget's own — the tier never translates the text). One
//! label under the word beside two honest absences: the worded
//! Bienvenue, the plain Welcome and the plain button — the label that
//! declared no language, never a guessed tag.
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the tag names a fact, it walls nothing (every door
//! answers as ever, the text the widget's own), and the app's own door
//! (`Label::set_language` through the mutate seam) moves the word live —
//! the plain Hello declaring its own tag, the Bienvenue tag withdrawn,
//! the `PAINT | A11Y` request pays the re-announcement — the write that
//! lands where the word already is queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example language
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

/// The language words the tree says for the labels, read back through
/// the snapshot — the same answer the sink maps onto AccessKit's
/// `Language` property: the tag beside the text.
fn language_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let words: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| n.role == Role::Label)
                .map(|n| match &n.language {
                    Some(tag) => format!("{} [{tag}]", n.label),
                    None => n.label.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    words.join(" · ")
}

/// The app's language door: the mutate seam the app's code rides — the
/// tag moved, `PAINT | A11Y` requested on the control, the
/// re-announcement paid for. `None` clears the word.
fn set_label_language(pipeline: &mut PassPipeline, node: alacer_core::NodeId, tag: Option<&str>) {
    let mut guard = pipeline.arena_mut().mutate(node);
    let moved = if let Some(label) = guard.widget_mut().as_any_mut().downcast_mut::<Label>() {
        label.set_language(tag.map(str::to_owned))
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
        Label::new("The language tier: the text's language, said by the tree (ADR-0127)").boxed(),
    );

    // The labels: the declared tag rides the tree from the first frame —
    // the worded greeting under the word, the plain greeting and the
    // plain button the honest absence between them. The word walls
    // nothing: the text is the text it always was.
    let welcome = pipeline.add_child(root, Label::new("Bienvenue").language("fr-FR").boxed());
    let _ = pipeline.add_child(root, Label::new("Welcome").boxed());
    let _ = pipeline.add_child(root, Button::new("Continue").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = language_words(snapshot.as_ref());
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
        target: "alacer::language",
        words = "Bienvenue [fr-FR], Welcome",
        word = "language",
        "beat 1: the declared tag rides the tree — the tag beside the text, the unworded widgets silent"
    );

    // Beat two: the language door — the plain Hello declaring its own
    // tag and the Bienvenue tag withdrawn, the word the tree says
    // following the app's own; then the absence restored (the Hello tag
    // withdrawn, the label the widget it always was).
    set_label_language(&mut pipeline, welcome, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the language door moved the tags",
        language_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::language",
        moved = "Bienvenue → no tag",
        word = "language",
        "beat 2: the app's door moved the tags the tree says — the word follows the app's own"
    );

    let greeting = pipeline.add_child(root, Label::new("Hello").boxed());
    set_label_language(&mut pipeline, greeting, Some("ja-JP"));
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the tags",
        language_words(snapshot.as_ref()),
    ));

    // The window: live play — the worded label says the tag beside the
    // plain ones, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the labels say the language — every door answers as it always did")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer language",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
