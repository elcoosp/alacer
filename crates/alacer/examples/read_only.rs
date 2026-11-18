//! The read-only tier as a consumer's vocabulary (M105, ADR-0109): the
//! "this field is the app's" word said by the a11y tree at last —
//! AccessKit's `ReadOnly` flag (ARIA's `aria-readonly`) beside the
//! value, and the walls every user write door prices behind it. Two
//! fields beside one status line: the locked one, whose tree answer
//! carries the word and whose doors refuse the writes (typing, paste,
//! the AT's `SetValue`) while the reads answer (focus, selection,
//! copy), and the unlocked one beside it, editable as ever.
//!
//! The beat the example exists to show: the app's own read-only door
//! (`TextInput::set_read_only` through the mutate seam) moves the word
//! live — the `PAINT | A11Y` request pays the re-announcement, and the
//! status line mirrors the word the tree says. The availability tier
//! (`enabled(false)`) stays the outer one; the read-only word walls
//! user doors, never the app's own `set_value`.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example read_only
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::{PassPipeline, TreeNode};
use alacer_core::widget::Role;
use reactive_graph::traits::Set as _;

/// The viewport the demo's beats settle into (the editing canon's shape).
const VIEWPORT_WIDTH: f32 = 480.0;
const VIEWPORT_HEIGHT: f32 = 380.0;

/// The value the demo locks in — the app's own fact, the field's
/// content while the read-only word holds.
const LICENSE: &str = "ALACER-105-READ-ONLY";

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

/// The word the tree says for one field, read back through the
/// snapshot — the same answer the sink maps onto AccessKit's flags.
fn tree_word(tree: Option<&alacer_core::passes::TreeSnapshot>, name: &str) -> String {
    let node = tree.and_then(|t| t.find(Role::TextInput, name).cloned());
    match node {
        Some(TreeNode {
            read_only: true,
            value,
            ..
        }) => format!("TextInput (read-only), value \"{value:?}\""),
        Some(TreeNode { value, .. }) => format!("TextInput (editable), value \"{value:?}\""),
        None => format!("{name}: absent"),
    }
}

/// The app's read-only door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the field, the
/// re-announcement paid for.
fn set_read_only(pipeline: &mut PassPipeline, node: alacer_core::NodeId, read_only: bool) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_read_only(read_only);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

/// The app's value door: the write that never asked the field's
/// permission — the tier walls user doors, not the app's.
fn set_value(pipeline: &mut PassPipeline, node: alacer_core::NodeId, value: &str) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_value(value);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The read-only tier: the app's value, said by the tree (ADR-0109)").boxed(),
    );

    // The locked field: mounted through the constructor word, the value
    // the app set through its own door. Typing, paste, cut, undo, redo
    // and the AT's SetValue all land on the wall; focus, selection and
    // copy answer beside it.
    let license = TextInput::new("License").read_only();
    let license_id = pipeline.add_child(root, license.boxed());
    set_value(&mut pipeline, license_id, LICENSE);

    // The editable field beside it: the honest absence — no word, the
    // doors exactly as they always were.
    let draft = TextInput::new("Draft");
    let _ = pipeline.add_child(root, draft.boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads).
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = format!(
        "License: {} · Draft: {}",
        tree_word(snapshot.as_ref(), "License"),
        tree_word(snapshot.as_ref(), "Draft"),
    );
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

    settle(&mut pipeline);

    // Beat one: the unlock — the app's door moves the word, the field
    // answers its doors exactly as any editable's (the re-announcement
    // the mutate seam's request paid for).
    set_read_only(&mut pipeline, license_id, false);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "License: {} · Draft: {} — the unlock moved the word the tree says",
        tree_word(snapshot.as_ref(), "License"),
        tree_word(snapshot.as_ref(), "Draft"),
    ));
    tracing::info!(
        target: "alacer::read_only",
        license = "TextInput (editable)",
        "beat 1: the unlock — the app's own door, the walls gone, the value kept"
    );

    // Beat two: the lock returns — the word rides again, the write
    // doors price the wall, the read doors answer beside it.
    set_read_only(&mut pipeline, license_id, true);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "License: {} · Draft: {} — the lock restored the declared word",
        tree_word(snapshot.as_ref(), "License"),
        tree_word(snapshot.as_ref(), "Draft"),
    ));
    tracing::info!(
        target: "alacer::read_only",
        license = "TextInput (read-only)",
        "beat 2: the lock restored the word — AccessKit's ReadOnly flag, the walls beside it"
    );

    // The window: live play — try typing into the locked field (the
    // wall answers, nothing moves) and into the draft (every door as
    // ever); the status line mirrors the words the tree says.
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: the locked field refuses every write and reads like any other — the draft answers every door",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer read-only",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
