//! The reveal tier as a consumer's vocabulary (M102, ADR-0106): the
//! "show password" toggle said by the a11y tree at last — ADR-0104's
//! first pre-registered re-open rule cashed. Two secured fields beside
//! one status line: the masked one, whose tree answer is exactly the
//! M100 word (`PasswordInput` and the mask, one bullet per code point),
//! and the revealed one, whose answer is AccessKit's ordinary editable
//! word and the field's ordinary value — the pixels and the tree
//! telling one truth.
//!
//! The beat the example exists to show: the app's own reveal door
//! (`TextInput::set_revealed` through the mutate seam) moves the word —
//! the `PAINT | A11Y` request pays the re-announcement, and the status
//! line mirrors the word the tree says: the mask yielding to the
//! reveal, and yielding back on the hide.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example revealed
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

/// The secret the demo fills — the payload the tree never carries while
/// the field holds the masked word (the clipboard trail's rule,
/// ADR-0045, at the tree tier).
const SECRET: &str = "correct horse";

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
/// snapshot — the same answer the sink maps onto AccessKit's roles and
/// values.
fn tree_word(tree: Option<&alacer_core::passes::TreeSnapshot>, name: &str) -> String {
    let node = tree.and_then(|t| t.find(Role::TextInput, name).cloned());
    match node {
        Some(TreeNode {
            secure_entry: true,
            secure_revealed: true,
            value,
            ..
        }) => format!("TextInput (revealed), value \"{value:?}\""),
        Some(TreeNode {
            secure_entry: true,
            secure_revealed: false,
            value,
            ..
        }) => format!("PasswordInput (masked), value \"{value:?}\""),
        Some(_) => format!("{name}: plain"),
        None => format!("{name}: absent"),
    }
}

/// The app's reveal door: the mutate seam the app's code rides — the
/// flag moved, `PAINT | A11Y` requested on the field, the
/// re-announcement paid for.
fn set_revealed(pipeline: &mut PassPipeline, node: alacer_core::NodeId, revealed: bool) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_revealed(revealed);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The reveal tier: the show-password toggle, said by the tree (ADR-0106)")
            .boxed(),
    );

    // The masked field: the M100 word — the AT filled it through the
    // write door every editable owns (a password manager's ask), and
    // the tree resolves the mask.
    let passphrase = TextInput::new("Passphrase").secure();
    let passphrase_id = pipeline.add_child(root, passphrase.boxed());
    pipeline
        .arena_mut()
        .mutate(passphrase_id)
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<TextInput>()
        .expect("the arena holds the field")
        .set_value(SECRET);

    // The revealed field beside it: mounted revealed — the ordinary
    // editable word, the value the pixels show.
    let shown = TextInput::new("Shown").secure().revealed();
    let shown_id = pipeline.add_child(root, shown.boxed());
    pipeline
        .arena_mut()
        .mutate(shown_id)
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<TextInput>()
        .expect("the arena holds the field")
        .set_value(SECRET);

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads).
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = format!(
        "Passphrase: {} · Shown: {}",
        tree_word(snapshot.as_ref(), "Passphrase"),
        tree_word(snapshot.as_ref(), "Shown"),
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

    // Beat one: the hide — the revealed field's toggle moves, the mask
    // returns, the word yields back (the re-announcement the mutate
    // seam's request paid for).
    set_revealed(&mut pipeline, shown_id, false);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "Passphrase: {} · Shown: {} — the hide restored the masked word",
        tree_word(snapshot.as_ref(), "Passphrase"),
        tree_word(snapshot.as_ref(), "Shown"),
    ));
    tracing::info!(
        target: "alacer::revealed",
        shown = "PasswordInput (masked)",
        "beat 1: the hide restored the M100 answer — the mask, the length tracking the value"
    );

    // Beat two: the reveal — the ordinary editable word and the
    // field's ordinary value again, the pixels and the tree telling
    // one truth.
    set_revealed(&mut pipeline, shown_id, true);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "Passphrase: {} · Shown: {} — the reveal yielded the mask to the value",
        tree_word(snapshot.as_ref(), "Passphrase"),
        tree_word(snapshot.as_ref(), "Shown"),
    ));
    tracing::info!(
        target: "alacer::revealed",
        shown = "TextInput (revealed)",
        "beat 2: the reveal yielded the mask — AccessKit's own contract, the value the pixels show"
    );

    // The window: live play — type into either field (the write door
    // every editable owns), the masked one announcing the mask's shape,
    // the revealed one its value.
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: type into the fields — the masked field's tree says the mask, the revealed one the value",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer reveal",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
