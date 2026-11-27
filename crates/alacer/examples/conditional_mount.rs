//! The M1 conditional-mount precedent as a consumer's vocabulary (§6.2,
//! ADR-0007's `rsx!` grammar): the keyed list mounted and unmounted by the
//! app's own wiring — one `if/else` branch in the view, the reconciler's
//! subtree replace doing the mount and the unmount, no second `visible`
//! flag the tree would have to guess at.
//!
//! The beat the example exists to show: presence is the app's wiring. The
//! view function runs the branch on every scoped re-render — `show` mounts
//! the keyed rows, `!show` mounts the empty-state label the branch falls
//! back to — and `ViewRoot::update` lands the flip in place: the mounted
//! subtree unmounts whole (its widgets leave the arena, the a11y tree
//! shrinks with it), the fallback mounts, and the surviving header and
//! footer never rebuild. The keyed rows keep their identity for as long as
//! the branch holds them (the peel → key map → LIS pipeline, §6.2). The
//! flip's price and its idle twin are the published pair
//! `rsx_conditional/flip/50` against `rsx_conditional/idle/50`
//! (benchmarks/baselines/baselines.json).
//!
//! The app shape is the tooltip tier's own (ADR-0140): handlers write one
//! signal and fire the patch channel; the loop answers with exactly one
//! scoped re-render (`ViewRoot::update`, the retained tree patched in
//! place).
//!
//! Run with `cargo run -p alacer --example conditional_mount`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_core::FluxRwSignal;
use alacer_core::flux::Flux;
use alacer_macro::rsx;
use alacer_view::ViewNode;
use alacer_view::ViewRoot;
use alacer_widgets::default_registry;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::rc::Rc;

/// The seeded rows the branch mounts: numbered labels, keyed so the
/// reconciler can hold their identity across the re-renders that keep
/// them.
fn rows() -> Vec<(u64, String)> {
    (0..6)
        .map(|index| {
            (
                u64::try_from(index).unwrap_or(u64::MAX),
                format!("row {index}"),
            )
        })
        .collect()
}

/// The view: the keyed list under one `if/else` branch, the footer
/// reading the presence back the way the app wired it.
fn conditional_view(show: &FluxRwSignal<bool>, wake: &Rc<dyn Fn()>) -> ViewNode {
    let current = show.try_get_untracked().unwrap_or(false);
    let list = rows();

    rsx! {
        column { padding: 16.0, gap: 12.0,
            label { text: "Conditional mount (§6.2, ADR-0007)" }
            button {
                text: if current { "Hide list" } else { "Show list" },
                on_click: {
                    let next_show = *show;
                    let next_wake = Rc::clone(wake);
                    move || {
                        Flux::batch_of(|| next_show.set(!next_show.get_untracked()));
                        (next_wake.clone())();
                    }
                }
            }
            if current {
                column { gap: 8.0,
                    for (key, text) in list.into_iter() {
                        row { key: key, gap: 8.0, label { text: text } }
                    }
                }
            } else {
                label { text: "Nothing to show" }
            }
            label {
                text: if current {
                    "branch: the keyed list is mounted"
                } else {
                    "branch: the empty state is mounted"
                }
            }
        }
    }
}

fn main() {
    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract). The wake fires the patch channel after every
    // write batch: one wake ⇒ one scoped re-render.
    let show = pipeline.flux().signal(true);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = conditional_view(&show, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer conditional mount",
        (360.0, 480.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = conditional_view(&show, &wake);
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
