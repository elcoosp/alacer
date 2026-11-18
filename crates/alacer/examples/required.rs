//! The required tier as a consumer's vocabulary (M106, ADR-0110): the
//! form contract's declared word said by the a11y tree at last —
//! AccessKit's `Required` flag (ARIA's `aria-required`) beside the
//! value, the word every platform AT turns into "required" before the
//! user ever reaches the submit. A mini form beside one status line:
//! the required card field, the required amount slider, the required
//! agree checkbox — and the optional note beside them, the honest
//! absence the tier keeps honest.
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the required field's doors answer exactly as any
//! editable's (required is not a wall), and the app's own door
//! (`TextInput::set_required` through the mutate seam) moves the word
//! live, the `PAINT | A11Y` request paying the re-announcement.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example required
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Checkbox, Column, Label, Slider, TextInput};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::{PassPipeline, TreeNode};
use alacer_core::widget::Role;
use reactive_graph::traits::Set as _;

/// The viewport the demo's beats settle into (the editing canon's shape).
const VIEWPORT_WIDTH: f32 = 480.0;
const VIEWPORT_HEIGHT: f32 = 420.0;

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

/// The word the tree says for one control, read back through the
/// snapshot — the same answer the sink maps onto AccessKit's flags.
fn tree_word(tree: Option<&alacer_core::passes::TreeSnapshot>, role: Role, name: &str) -> String {
    let node = tree.and_then(|t| t.find(role, name).cloned());
    match node {
        Some(TreeNode { required: true, .. }) => format!("{name} (required)"),
        Some(_) => format!("{name} (optional)"),
        None => format!("{name}: absent"),
    }
}

/// The app's required door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the field, the
/// re-announcement paid for.
fn set_required(pipeline: &mut PassPipeline, node: alacer_core::NodeId, required: bool) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_required(required);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The required tier: the form contract, said by the tree (ADR-0110)").boxed(),
    );

    // The form's required controls: the word rides the tree from the
    // first frame, the doors answering exactly as they always did.
    let card = TextInput::new("Card").required();
    let card_id = pipeline.add_child(root, card.boxed());
    let _ = pipeline.add_child(root, Slider::new("Amount", 0.0, 100.0).required().boxed());
    let _ = pipeline.add_child(root, Checkbox::new("Agree").required().boxed());

    // The optional note beside them: the honest absence — no word, the
    // AT reads the field exactly as it read the tree M105 handed over.
    let _ = pipeline.add_child(root, TextInput::new("Note").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads).
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = format!(
        "{} · {} · {} · {}",
        tree_word(snapshot.as_ref(), Role::TextInput, "Card"),
        tree_word(snapshot.as_ref(), Role::Slider, "Amount"),
        tree_word(snapshot.as_ref(), Role::CheckBox, "Agree"),
        tree_word(snapshot.as_ref(), Role::TextInput, "Note"),
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

    // Beat one: the word is a word, not a wall — the AT's write lands
    // on the required field exactly as any editable's.
    let consumed = pipeline.dispatch_a11y_action(
        snapshot
            .as_ref()
            .and_then(|t| t.find(Role::TextInput, "Card"))
            .map(|node| node.id)
            .expect("the card field rides the snapshot"),
        alacer_core::widget::A11yAction::SetValue("4242 4242".into()),
    );
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::required",
        consumed,
        "beat 1: the AT's write lands on the required field — required is not a wall"
    );

    // Beat two: the live door — the app's form contract moved, the word
    // the tree says following it (the re-announcement the mutate seam's
    // request paid for).
    set_required(&mut pipeline, card_id, false);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} · {} · {} · {} — the live door moved the card's word",
        tree_word(snapshot.as_ref(), Role::TextInput, "Card"),
        tree_word(snapshot.as_ref(), Role::Slider, "Amount"),
        tree_word(snapshot.as_ref(), Role::CheckBox, "Agree"),
        tree_word(snapshot.as_ref(), Role::TextInput, "Note"),
    ));
    tracing::info!(
        target: "alacer::required",
        card = "optional",
        "beat 2: the app's door moved the word the tree says — the honest absence restored"
    );

    // The window: live play — the required controls announce the word
    // beside their values, the note stays optional, every door answers
    // as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: the required controls say the word — every door answers as it always did",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer required",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
