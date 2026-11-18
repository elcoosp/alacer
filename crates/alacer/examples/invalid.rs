//! The invalid tier as a consumer's vocabulary (M107, ADR-0111): the
//! form's rejection, said by the a11y tree at last — AccessKit's
//! `Invalid` property (ARIA's `aria-invalid`) beside the value, the
//! word every platform AT turns into "invalid" while the user edits. A
//! mini form beside one status line: the misspelled card field
//! (`spelling`), the out-of-contract amount slider (`true`), the
//! mismatched agree checkbox (`grammar`) — and the valid note beside
//! them, the honest absence the tier keeps honest.
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the invalid field's doors answer exactly as any
//! editable's (invalid is not a wall: the AT's write lands while the
//! word stands), and the app's own door (`TextInput::set_invalid`
//! through the mutate seam) moves the word and its kind live, `None`
//! the honest absence restored.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example invalid
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Checkbox, Column, Label, Slider, TextInput};
use alacer_core::InvalidKind;
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
/// snapshot — the same answer the sink maps onto AccessKit's property.
fn tree_word(tree: Option<&alacer_core::passes::TreeSnapshot>, role: Role, name: &str) -> String {
    let node = tree.and_then(|t| t.find(role, name).cloned());
    match node {
        Some(TreeNode {
            invalid: Some(kind),
            ..
        }) => format!("{name} (invalid: {})", kind_word(kind)),
        Some(_) => format!("{name} (valid)"),
        None => format!("{name}: absent"),
    }
}

/// The kind's spoken half — ARIA's own three values, the words the AT
/// reads.
fn kind_word(kind: InvalidKind) -> &'static str {
    match kind {
        InvalidKind::True => "true",
        InvalidKind::Grammar => "grammar",
        InvalidKind::Spelling => "spelling",
    }
}

/// The app's invalid door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the field, the
/// re-announcement paid for.
fn set_invalid(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    invalid: Option<InvalidKind>,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_invalid(invalid);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The invalid tier: the form's rejection, said by the tree (ADR-0111)").boxed(),
    );

    // The form's invalid controls: the word and its kind ride the tree
    // from the first frame, the doors answering exactly as they always
    // did.
    let card = TextInput::new("Card").invalid(InvalidKind::Spelling);
    let card_id = pipeline.add_child(root, card.boxed());
    let _ = pipeline.add_child(
        root,
        Slider::new("Amount", 0.0, 100.0)
            .value(88.0)
            .invalid(InvalidKind::True)
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Checkbox::new("Agree").invalid(InvalidKind::Grammar).boxed(),
    );

    // The valid note beside them: the honest absence — no word, the AT
    // reads the field exactly as it read the tree M106 handed over.
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
    // on the invalid field exactly as any editable's.
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
        target: "alacer::invalid",
        consumed,
        "beat 1: the AT's write lands on the invalid field — invalid is not a wall"
    );

    // Beat two: the live door — the app's form contract judged the
    // value valid, the word the tree says following it (the
    // re-announcement the mutate seam's request paid for).
    set_invalid(&mut pipeline, card_id, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} · {} · {} · {} — the live door cleared the card's word",
        tree_word(snapshot.as_ref(), Role::TextInput, "Card"),
        tree_word(snapshot.as_ref(), Role::Slider, "Amount"),
        tree_word(snapshot.as_ref(), Role::CheckBox, "Agree"),
        tree_word(snapshot.as_ref(), Role::TextInput, "Note"),
    ));
    tracing::info!(
        target: "alacer::invalid",
        card = "valid",
        "beat 2: the app's door cleared the word the tree says — the honest absence restored"
    );

    // The window: live play — the invalid controls announce the word
    // beside their values, the note stays valid, every door answers as
    // it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the invalid controls say the word — every door answers as it always did")
            .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer invalid",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
