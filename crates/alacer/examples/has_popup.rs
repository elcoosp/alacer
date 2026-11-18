//! The has-popup tier as a consumer's vocabulary (M115, ADR-0119): the
//! opener's target said by the a11y tree at last — AccessKit's
//! `HasPopup` property (ARIA's `aria-haspopup`) beside the name, the
//! word every platform AT turns into "opens a menu" while it walks the
//! toolbar. Five openers beside one plain button and a status line:
//! the menu button, the combobox button, the tree toggle, the grid
//! picker, the dialog opener — the plain button between them, the
//! honest absence the tier keeps honest (the control that opens
//! nothing, never a guessed target).
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the has-popup word names a target, it walls nothing
//! (the openers stay put, the plain button beside them answers its
//! press as ever), and the app's own door (`Button::set_has_popup`
//! through the mutate seam) moves the word live, the `PAINT | A11Y`
//! request paying the re-announcement — the write that lands where the
//! word already is queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example has_popup
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{HasPopupKind, Role};
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

/// The targets the tree says for the openers, read back through the
/// snapshot in walk order — the same answer the sink maps onto
/// AccessKit's `HasPopup` property: the kind beside the word.
/// Buttons are nameless here in the read-back (the names ride the
/// labels), so the line walks the snapshot's openers by position.
fn opener_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let openers: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| n.role == Role::Button)
                .map(|n| match n.has_popup {
                    Some(kind) => format!("opener ({})", target_word(kind)),
                    None => "opener (plain)".to_owned(),
                })
                .collect()
        })
        .unwrap_or_default();
    openers.join(" · ")
}

/// The ARIA shape the kind spells — the word the AT reads beside the
/// name (`aria-haspopup="menu"` and friends).
fn target_word(kind: HasPopupKind) -> &'static str {
    match kind {
        HasPopupKind::Menu => "menu",
        HasPopupKind::Listbox => "listbox",
        HasPopupKind::Tree => "tree",
        HasPopupKind::Grid => "grid",
        HasPopupKind::Dialog => "dialog",
    }
}

/// The app's has-popup door: the mutate seam the app's code rides — the
/// word moved, `PAINT | A11Y` requested on the opener, the
/// re-announcement paid for. `None` clears the word.
fn set_opener_target(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    has_popup: Option<HasPopupKind>,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_has_popup(has_popup);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The has-popup tier: the opener's target, said by the tree (ADR-0119)").boxed(),
    );

    // The openers: the target rides the tree from the first frame —
    // the menu button, the combobox button, the tree toggle, the grid
    // picker, the dialog opener — beside the plain button the honest
    // absence between them.
    let _menu = pipeline.add_child(
        root,
        Button::new("Actions").has_popup(HasPopupKind::Menu).boxed(),
    );
    let _listbox = pipeline.add_child(
        root,
        Button::new("Pick a color")
            .has_popup(HasPopupKind::Listbox)
            .boxed(),
    );
    let _tree = pipeline.add_child(
        root,
        Button::new("Browse folders")
            .has_popup(HasPopupKind::Tree)
            .boxed(),
    );
    let _grid = pipeline.add_child(
        root,
        Button::new("Pick a date")
            .has_popup(HasPopupKind::Grid)
            .boxed(),
    );
    let dialog = pipeline.add_child(
        root,
        Button::new("Confirm")
            .has_popup(HasPopupKind::Dialog)
            .boxed(),
    );
    let plain = pipeline.add_child(root, Button::new("Send").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = opener_words(snapshot.as_ref());
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

    // Beat one: the word is a word, not a wall — the plain button
    // beside the worded openers answers its press exactly as it always
    // did.
    let consumed = pipeline.dispatch_a11y_action(
        snapshot
            .as_ref()
            .and_then(|t| t.find(Role::Button, "Send"))
            .map(|node| node.id)
            .expect("the plain button rides the snapshot"),
        alacer_core::widget::A11yAction::DefaultPress,
    );
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::has_popup",
        consumed,
        "beat 1: the AT's press lands unchanged beside the worded openers — the word names a target, it walls nothing"
    );

    // Beat two: the live door — the app's target moved, the word the
    // tree says following it (the re-announcement the mutate seam's
    // request paid for); then the honest absence restored.
    set_opener_target(&mut pipeline, plain, Some(HasPopupKind::Menu));
    set_opener_target(&mut pipeline, dialog, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the live door moved the openers' words",
        opener_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::has_popup",
        openers = "Send, Confirm",
        word = "has_popup",
        "beat 2: the app's door moved the words the tree says — the target follows the app's own"
    );
    set_opener_target(&mut pipeline, plain, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the words",
        opener_words(snapshot.as_ref()),
    ));

    // The window: live play — the worded openers say the target beside
    // the plain button, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the openers say the word — every door answers as it always did").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer has-popup",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
