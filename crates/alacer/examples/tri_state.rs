//! The tri-state summary as a consumer's vocabulary (M94, ADR-0098): the
//! select-all parent the `ToggleState::Mixed` state exists to tell. Three
//! task checkboxes and a parent — the parent's `state` word recomputes
//! from the children on every re-render: all checked announces `on`, none
//! announces `off`, otherwise `mixed` — the summary the a11y tree has
//! been able to carry since M6, produced by a widget at last.
//!
//! The resolution rule is the beat the example exists to show: click the
//! *mixed* parent and the first press lands fully **checked** — the
//! summary is never a third bit — and the cascade writes all three tasks
//! in one batch, so one interaction is one re-render, at most one
//! layout+paint cycle (ADR-6, the patch channel's §13.2 path). Toggle a
//! child back and the parent lands in `mixed` again; the write that
//! lands where the word already is patches nothing (the idle frame).
//!
//! The app shape is the todo canon's own: handlers write one signal and
//! fire the patch channel; the loop answers with exactly one scoped
//! re-render (`ViewRoot::update`, the retained tree patched in place).
//!
//! Run with `cargo run -p alacer --example tri_state`.
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

/// The tasks the summary summarizes — one bool per child checkbox.
const TASKS: [&str; 3] = ["Pack the crates", "Pin the release", "Publish the notes"];

/// The summary word the parent carries for the current task states —
/// the vocabulary the registry parses and the tree announces.
fn summary_word(tasks: &[bool]) -> &'static str {
    let done = tasks.iter().filter(|task| **task).count();
    if done == 0 {
        "off"
    } else if done == tasks.len() {
        "on"
    } else {
        "mixed"
    }
}

/// The view: the parent's summary recomputed from the signal, the
/// children keyed by their own flag, the status line counting what is
/// packed. Handlers write the signal and fire the wake — one batch, one
/// re-render (ADR-6's view edition).
fn tri_state_view(tasks: &FluxRwSignal<Vec<bool>>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = tasks.try_get_untracked().unwrap_or_default();
    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The tri-state summary (ADR-0098)"));

    // The parent: the summary word rides the re-render (the patch tier's
    // `set_state` door in place); the handler receives the *resolved*
    // boolean — the summary never hands the app a third bit.
    let parent_tasks = *tasks;
    let parent_wake = Rc::clone(wake);
    let resolve = PropValue::FlagHandler(Rc::new(RefCell::new(move |all: bool| {
        Flux::batch_of(|| {
            parent_tasks.set(vec![all; TASKS.len()]);
        });
        (parent_wake)();
    })));
    root = root.child(
        ViewNode::element("checkbox")
            .prop("name", "Select all")
            .prop("state", summary_word(&current))
            .prop("on_toggle", resolve),
    );

    // The children: one checkbox per task, each flipping its own bool.
    for (index, name) in TASKS.iter().enumerate() {
        let done = current.get(index).copied().unwrap_or(false);
        let child_tasks = *tasks;
        let child_wake = Rc::clone(wake);
        let toggle = PropValue::FlagHandler(Rc::new(RefCell::new(move |checked: bool| {
            Flux::batch_of(|| {
                let mut next = child_tasks.try_get_untracked().unwrap_or_default();
                if let Some(task) = next.get_mut(index) {
                    *task = checked;
                }
                child_tasks.set(next);
            });
            (child_wake)();
        })));
        root = root.child(
            ViewNode::element("checkbox")
                .prop("name", *name)
                .prop("checked", done)
                .prop("on_toggle", toggle),
        );
    }

    let done = current.iter().filter(|task| **task).count();
    root = root.child(
        ViewNode::element("label").prop("text", format!("{done} of {} packed", TASKS.len())),
    );
    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let tasks = pipeline.flux().signal(vec![true, true, false]);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = tri_state_view(&tasks, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer tri-state",
        (360.0, 340.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = tri_state_view(&tasks, &wake);
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
