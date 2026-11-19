//! The heading tier as a consumer's vocabulary (M126, ADR-0130): the
//! section title the document outline walks — the depth in the outline
//! said by the tree (AccessKit's `Level` property, zero-based beside
//! ARIA's one-based `aria-level`), the quick-nav every reader skims by
//! finally having something to land on.
//!
//! The beat the example exists to show: a heading names structure and
//! walls nothing — the label beside it reads, the button beside it
//! presses, and the level write is the M21 idle frame at the tier (a
//! write that lands where the depth already is queues nothing, the
//! moved write the one mask the outline costs). The depth is a fact
//! and never a font size: the glyphs are the app's styling, the tree
//! says the outline.
//!
//! The app shape is the numeric canon's own: handlers write one signal
//! and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example heading`.
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

/// The sections the demo declares — the outline the AT's quick-nav
/// walks, the depth each title rides.
const SECTIONS: [&str; 3] = ["Getting started", "Installation", "Troubleshooting"];

/// The view: the outline the tree carries — the root title wordless
/// (the honest absence, a title that declared no depth announces
/// none), the nested sections riding their zero-based depths, and the
/// status line reading the moved depth back the way the tree carries
/// it.
fn heading_view(section: &FluxRwSignal<usize>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = section.try_get_untracked().unwrap_or(0);

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 8.0_f32);

    root = root.child(ViewNode::element("label").prop("text", "The heading tier (ADR-0130)"));

    // The outline: the root title the honest absence beside, the
    // nested sections riding the depths the quick-nav walks — the
    // glyphs sized the app's styling, the tree saying the structure.
    root = root.child(
        ViewNode::element("heading")
            .prop("name", "The Alacer guide")
            .prop("font_size", 24.0_f32),
    );
    for (index, name) in SECTIONS.iter().enumerate() {
        root = root.child(
            ViewNode::element("heading")
                .prop("name", *name)
                .prop("level", alacer_core::num::usize_to_f32(index))
                .prop("font_size", 18.0_f32),
        );
    }

    // The section the demo shifts: the button mounts the outline's
    // one write door — the moved depth the re-announcement rides, the
    // idle write the frame that queues nothing. The signal is `Copy` —
    // copy it out of the reference the view builder holds, no clone
    // ceremony (the pedantic rule the facade's examples answer).
    let target = current.min(SECTIONS.len() - 1);
    let next_section = *section;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("button")
            .prop("text", "Deepen the outline")
            .prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new(move || {
                    Flux::batch_of(|| next_section.set((target + 1) % SECTIONS.len()));
                    (next_wake.clone())();
                }))),
            ),
    );

    // The status line mirrors the outline the tree carries: the
    // section's zero-based depth beside the one-based word ARIA speaks.
    root = root.child(ViewNode::element("label").prop(
        "text",
        format!(
            "outlined: {} (zero-based {}, aria-level {})",
            SECTIONS[target],
            target,
            target + 1
        ),
    ));

    root
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let section = pipeline.flux().signal(0_usize);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = heading_view(&section, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(pipeline, "Alacer heading", (360.0, 300.0), backend_choice());
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = heading_view(&section, &wake);
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
