//! The secure tier as a consumer's vocabulary (M100, ADR-0104): the
//! field's declared secret entry — the constructor word the IME
//! contract has carried since M4, said by the a11y tree at last.
//! Two fields beside each other: the secured one, whose tree answer is
//! AccessKit's `PasswordInput` role and a masked value (one masking
//! character per code point, never the secret — the length the AT
//! reads), and the bare one, whose tree answer is exactly what the tree
//! always said.
//!
//! The beat the example exists to show: the *Fill* button writes
//! through the AT's own door (the write a password manager rides) —
//! the field's value lands, and the tree's answer is the mask, the
//! length tracking the value, the secret itself never riding the tree.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one scoped
//! re-render (`ViewRoot::update`, the retained tree patched in place).
//!
//! Run with `cargo run -p alacer --example secure`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_core::FluxRwSignal;
use alacer_core::flux::Flux;
use alacer_core::widget::{A11yAction, Role};
use alacer_view::{PropValue, ViewNode, ViewRoot};
use alacer_widgets::default_registry;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::rc::Rc;

/// The secret the demo fills — the payload the tree never carries (the
/// clipboard trail's rule, ADR-0045, at the tree tier).
const SECRET: &str = "correct horse";

/// The view: the secured field, the bare field beside it, the status
/// line reading the word the tree says back, and the *Fill* door — the
/// write the mask answers.
fn secure_view(filled: &FluxRwSignal<bool>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let is_filled = filled.try_get_untracked().unwrap_or(false);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The secure tier (ADR-0104)"));

    // The secured field: the constructor's word is the tree's truth —
    // `PasswordInput` in the snapshot, the value the tree resolves the
    // mask. The `secure` word parsed since the M0 registry builds the
    // same constructor.
    root = root.child(
        ViewNode::element("text_input")
            .prop("name", "Passphrase")
            .prop("secure", true)
            .prop(
                "description",
                "Secret entry: the tree says the word, never the value",
            ),
    );

    // The bare field beside it: the honest absence — the tree reads
    // exactly as it read before the tier.
    root = root.child(ViewNode::element("text_input").prop("name", "Notes").prop(
        "description",
        "Plain entry: the value the tree carries is the AT's read",
    ));

    // The status line mirrors the pair the tree carries: the secured
    // field's word and the mask's length, beside the bare field's raw
    // value.
    let mask = "•".repeat(if is_filled { SECRET.chars().count() } else { 0 });
    root = root.child(ViewNode::element("label").prop(
        "text",
        format!(
            "Passphrase: PasswordInput, value \"{mask}\" · Notes: TextInput, value \"{}\"",
            if is_filled { "typed" } else { "" }
        ),
    ));

    // The *Fill* door: the write the AT's own tier rides — the value
    // lands, the mask answers. The signal is `Copy`; copy it out of the
    // view builder's reference.
    let next_filled = *filled;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("button")
            .prop("text", "Fill")
            .prop("description", "Writes the secret through the AT's own door")
            .prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new(move || {
                    Flux::batch_of(|| next_filled.set(true));
                    (next_wake)();
                }))),
            ),
    );

    root
}

/// The view layer carries no secrets: the signal says *whether* the
/// demo filled the field, the write itself rides the a11y action door
/// at mount time — the patch hook's one scoped re-render.
fn fill_secrets(pipeline: &mut alacer_core::passes::PassPipeline) {
    // The write the AT's door carries, driven from the app side: both
    // fields written through the same funnel a screen reader's SetValue
    // rides — the secured field's tree answer resolving to the mask.
    for (name, value) in [("Passphrase", SECRET), ("Notes", "typed")] {
        if let Some(node) = pipeline
            .a11y_snapshot()
            .and_then(|tree| tree.find(Role::TextInput, name).map(|node| node.id))
        {
            let _ = pipeline.dispatch_a11y_action(node, A11yAction::SetValue(value.to_string()));
        }
    }
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let filled = pipeline.flux().signal(false);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = secure_view(&filled, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer secure", (360.0, 280.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        if filled.try_get_untracked().unwrap_or(false) {
            fill_secrets(pipeline);
        }
        let view = secure_view(&filled, &wake);
        let _report = root.update(&view, pipeline);
    }));
    if let Err(error) = app.with_patch_channel(channel).run() {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
