//! The auto-complete tier as a consumer's vocabulary (M111, ADR-0115):
//! the field's suggestion contract said by the a11y tree at last —
//! AccessKit's `AutoComplete` property (ARIA's `aria-autocomplete`)
//! beside the hint the placeholder tier (ADR-0100) already carried, the
//! word every platform AT turns into "completions available" while the
//! user types. Three fields beside one status line: the field that
//! lists its completions, the field that completes inline, the plain
//! field between them — the honest absence the tier keeps honest
//! (ARIA's `none`, never a guessed mode).
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the suggestion contract names a contract, it walls
//! nothing (the fields stay editable, the AT's write landing
//! untouched), and the app's own door (`TextInput::set_auto_complete`
//! through the mutate seam) moves the word live, the `PAINT | A11Y`
//! request paying the re-announcement — the write that lands where the
//! word already is queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example auto_complete
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, TextInput};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::{PassPipeline, TreeNode};
use alacer_core::widget::{AutoCompleteKind, Role};
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

/// The suggestion contract the tree says for one field, read back
/// through the snapshot — the same answer the sink maps onto
/// AccessKit's `AutoComplete` property: the kind beside the word.
fn field_words(tree: Option<&alacer_core::passes::TreeSnapshot>, name: &str) -> String {
    let field = tree.and_then(|t| t.find(Role::TextInput, name).cloned());
    match field {
        Some(TreeNode {
            auto_complete: Some(kind),
            ..
        }) => {
            let mode = match kind {
                AutoCompleteKind::Inline => "inline",
                AutoCompleteKind::List => "list",
                AutoCompleteKind::Both => "both",
            };
            format!("{name} (completions: {mode})")
        }
        Some(_) => format!("{name} (no contract)"),
        None => format!("{name}: absent"),
    }
}

/// The app's auto-complete door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the field, the
/// re-announcement paid for. `None` clears the word.
fn set_field_auto_complete(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    auto_complete: Option<AutoCompleteKind>,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(field) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        field.set_auto_complete(auto_complete);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The auto-complete tier: the field's suggestion contract, said by the tree (ADR-0115)",
        )
        .boxed(),
    );

    // The fields: the suggestion contract rides the tree from the
    // first frame — the listing field, the inline field, the plain
    // field the honest absence between them.
    let _city = pipeline.add_child(
        root,
        TextInput::new("City")
            .auto_complete(AutoCompleteKind::List)
            .boxed(),
    );
    let street = pipeline.add_child(
        root,
        TextInput::new("Street")
            .auto_complete(AutoCompleteKind::Inline)
            .boxed(),
    );
    let note = pipeline.add_child(root, TextInput::new("Note").boxed());

    // The plain widget beside the fields: a field word, not a widget
    // word — the button inherits the honest absence.
    let _ = pipeline.add_child(root, Button::new("Approve").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = format!(
        "{} · {} · {}",
        field_words(snapshot.as_ref(), "City"),
        field_words(snapshot.as_ref(), "Street"),
        field_words(snapshot.as_ref(), "Note"),
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

    // Beat one: the word is a word, not a wall — the AT's write lands
    // on the auto-completed field exactly as it always did.
    let consumed = pipeline.dispatch_a11y_action(
        snapshot
            .as_ref()
            .and_then(|t| t.find(Role::TextInput, "City"))
            .map(|node| node.id)
            .expect("the field rides the snapshot"),
        alacer_core::widget::A11yAction::SetValue("Paris".into()),
    );
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::auto_complete",
        consumed,
        "beat 1: the AT's write lands unchanged beside the field's contract — the word names a contract, it walls nothing"
    );

    // Beat two: the live door — the app's contract moved, the word the
    // tree says following it (the re-announcement the mutate seam's
    // request paid for); then the honest absence restored.
    set_field_auto_complete(&mut pipeline, street, None);
    set_field_auto_complete(&mut pipeline, note, Some(AutoCompleteKind::Both));
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} · {} · {} — the live door moved the fields' words",
        field_words(snapshot.as_ref(), "City"),
        field_words(snapshot.as_ref(), "Street"),
        field_words(snapshot.as_ref(), "Note"),
    ));
    tracing::info!(
        target: "alacer::auto_complete",
        fields = "Street, Note",
        word = "auto_complete",
        "beat 2: the app's door moved the words the tree says — the contract follows the app's own"
    );
    set_field_auto_complete(&mut pipeline, note, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} · {} · {} — the door cleared the words",
        field_words(snapshot.as_ref(), "City"),
        field_words(snapshot.as_ref(), "Street"),
        field_words(snapshot.as_ref(), "Note"),
    ));

    // The window: live play — the listing field says the contract
    // beside the inline field, the plain field stays the honest
    // absence, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the fields say the word — every door answers as it always did").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer auto_complete",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
