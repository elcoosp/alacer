//! The description tier as a consumer's vocabulary (M95, ADR-0099): the
//! help text the a11y tree carries beside the name — the sentence the
//! label is too short to answer "what does this control do?" with. Three
//! controls, each described: the button whose description names the
//! consequence, the checkbox whose description names the rule, the input
//! whose description names the format. The AT reads the description
//! after the name — the screen reader's help text — the property every
//! platform adapter maps, produced by a widget at last.
//!
//! The beat the example exists to show: the *Rewrite help* button rotates
//! the publish button's description through the view's `description`
//! word — the patch tier lands the new help text in place, and the frame
//! that moves only the tree's truth re-announces **without repainting**:
//! the a11y-only prop class queues exactly `A11Y` (ADR-0099's own rule),
//! the pixels never paying for a fact the painter did not write. The
//! status line mirrors what the tree carries.
//!
//! The app shape is the tri-state canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one scoped
//! re-render (`ViewRoot::update`, the retained tree patched in place).
//!
//! Run with `cargo run -p alacer --example descriptions`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_core::FluxRwSignal;
use alacer_core::flux::Flux;
use alacer_view::{PropValue, ViewNode, ViewRoot};
use alacer_widgets::default_registry;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::rc::Rc;

/// The help texts the *Rewrite help* door rotates through — the publish
/// button's description, one sentence per beat.
const HELP_TEXTS: [&str; 3] = [
    "Saves the draft and queues it for review",
    "Publishes immediately — no draft, no review step",
    "Schedules the note for the next morning digest",
];

/// The view: the described canon — the three producers beside their
/// undescribed twins (the honest absence) — and the status line reading
/// the current help text back. The *Rewrite help* handler writes the
/// signal and fires the wake: one batch, one re-render.
fn descriptions_view(help: &FluxRwSignal<usize>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = help.try_get_untracked().unwrap_or_default();
    let active = HELP_TEXTS[current % HELP_TEXTS.len()];

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The description tier (ADR-0099)"));

    // The described canon: the help text rides the `description` word —
    // the create write at mount, the patch tier's door in place.
    root = root.child(
        ViewNode::element("button")
            .prop("text", "Publish")
            .prop("description", active)
            .prop("on_click", PropValue::Handler(Rc::new(RefCell::new(|| {})))),
    );
    root = root.child(
        ViewNode::element("checkbox")
            .prop("name", "Ship in bulk")
            .prop("description", "Marks every open order for the carrier run")
            .prop(
                "on_toggle",
                PropValue::FlagHandler(Rc::new(RefCell::new(|_checked: bool| {}))),
            ),
    );
    root = root.child(
        ViewNode::element("text_input")
            .prop("name", "Card number")
            .prop("description", "The 16 digits on the front of the card"),
    );

    // The honest absences beside the canon: the same roles, undescribed —
    // the tree announces the absence, never a guess.
    root = root.child(ViewNode::element("button").prop("text", "Cancel"));
    root = root.child(ViewNode::element("text_input").prop("name", "Coupon code"));

    // The status line mirrors what the tree carries for the publish
    // button — the app's own truth, one write away.
    root = root.child(ViewNode::element("label").prop("text", format!("Publish help: {active}")));

    // The *Rewrite help* door: the description-only patch — the frame
    // that moves the tree's truth and repaints nothing.
    let next_help = *help;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("button")
            .prop("text", "Rewrite help")
            .prop("description", "Rotates the publish button's help text")
            .prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new(move || {
                    Flux::batch_of(|| {
                        let next = (next_help.try_get_untracked().unwrap_or_default() + 1)
                            % HELP_TEXTS.len();
                        next_help.set(next);
                    });
                    (next_wake)();
                }))),
            ),
    );

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let help = pipeline.flux().signal(0_usize);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = descriptions_view(&help, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer descriptions",
        (360.0, 380.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = descriptions_view(&help, &wake);
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
