//! The tree tier as a consumer's vocabulary (M135, ADR-0139): the
//! hierarchical collection the AT walks by disclosure and depth, said
//! by the tree — the word the AT announces before it walks the
//! branches.
//!
//! The beat the example exists to show: the tier prices the words and
//! nothing else — the hierarchy *is* the arena's own nesting (nested
//! items under a branch, the app's own controls under either), every
//! word the members say is a tier the tree already published (the M108
//! pick, the M110 disclosure, the M113 depth, the M119 place and span,
//! the M109 capacity), and the collection and its members hold no
//! focus and answer no activation (the M126/M129 container rule): the
//! app's own control beside the tree keeps the focus, the words name
//! facts, they wall nothing.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example tree`.
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

/// The view: the named collection with its nested members — the
/// disclosure, pick, depth and place words riding the tiers the tree
/// already published, the app's control beside it keeping the focus.
fn tree_view(picked: &FluxRwSignal<String>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = picked
        .try_get_untracked()
        .unwrap_or_else(|| "nothing".into());

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The tree tier (ADR-0139)"));

    // The named collection: the tree word the AT announces before it
    // walks the branches — the span and capacity words the container
    // carries, the members' words riding the published tiers, the
    // hierarchy the arena's own nesting (nested items under the
    // branch).
    root = root.child(
        ViewNode::element("tree")
            .prop("name", "Project files")
            .prop("description", "The workspace's files")
            .prop("size_of_set", 3.0_f32)
            .child(
                ViewNode::element("tree_item")
                    .prop("name", "src")
                    .prop("expanded", true)
                    .prop("level", 0.0_f32)
                    .prop("position_in_set", 0.0_f32)
                    .child(
                        ViewNode::element("tree_item")
                            .prop("name", "lib.rs")
                            .prop("level", 1.0_f32)
                            .prop("position_in_set", 0.0_f32),
                    )
                    .child(
                        ViewNode::element("tree_item")
                            .prop("name", "main.rs")
                            .prop("level", 1.0_f32)
                            .prop("position_in_set", 1.0_f32),
                    ),
            )
            .child(
                ViewNode::element("tree_item")
                    .prop("name", "Cargo.toml")
                    .prop("selected", true)
                    .prop("level", 0.0_f32)
                    .prop("position_in_set", 1.0_f32),
            ),
    );

    // The app's own control riding the doors it always carried: the
    // press is the app's wiring, the pick word the view's declaration
    // — one write batch, one scoped re-render.
    let next_picked = *picked;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("button")
            .prop("text", "Pick the manifest")
            .prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new(move || {
                    Flux::batch_of(|| next_picked.set("Cargo.toml".into()));
                    (next_wake.clone())();
                }))),
            ),
    );

    // The reads read back the way the app wired them.
    root = root.child(ViewNode::element("label").prop("text", format!("picked: {current}")));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let picked = pipeline.flux().signal("nothing".to_string());
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = tree_view(&picked, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer tree", (360.0, 220.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = tree_view(&picked, &wake);
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
