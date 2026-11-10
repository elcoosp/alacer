//! The focus scope as a consumer's vocabulary (§10.3, ADR-0031): the walk
//! bounds to a subtree — the modal's focus trap. A background control and
//! a modal's form (two inputs, a checkbox, a send button) share one
//! window; every `PassPipeline::focus_move_in` beat walks *inside the
//! modal's subtree*: forward, backward, and the honest refusal while the
//! availability tier holds every control in the trap down. The background
//! control never becomes a stop, the read doors (`focus()`,
//! `focusables()`) keep answering the whole tree, and the unscoped Tab
//! stays the window's door — one walk, two scopes (ADR-0030's funnel is
//! the `None` scope of ADR-0031's walk).
//!
//! The demo walks the beats at startup, before the window opens, and
//! every landing rides the `alacer::focus_trap` tracing target — the
//! workspace's per-seam idiom, `RUST_LOG=alacer=info` to watch:
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example focus_trap
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path.
//!
//! Run with `cargo run -p alacer --example focus_trap`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Checkbox, Column, Label, TextInput};
use alacer_core::geom::Size;
use alacer_core::mask::Change;
use alacer_core::passes::{FocusMove, PassPipeline};
use alacer_core::widget::Widget;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The demo's viewport — the size the startup beats tick against (the
/// window re-ticks at its own size once it opens).
const VIEWPORT: Size = Size::new(420.0, 300.0);

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let report = pipeline.tick(VIEWPORT).expect("the demo frame lays out");
        if report.is_idle() {
            return;
        }
    }
}

/// The name the walk rests on, read through the pipeline's own doors —
/// `focus()` for the node, the arena's required `label()` for the name.
/// The a11y contract is the read: no widget internals, the name the tree
/// carries.
fn focused_name(pipeline: &PassPipeline) -> String {
    pipeline
        .focus()
        .map(|node| pipeline.arena().widget(node).label().resolve())
        .unwrap_or_else(|| "nothing".to_string())
}

/// One focus_move_in beat: the span names the scoped route, the door
/// answers, the frame settles, and the landing is announced on the
/// demo's target.
fn trap_beat(
    pipeline: &mut PassPipeline,
    beat: &'static str,
    movement: FocusMove,
    scope: alacer_core::NodeId,
) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_trap",
        "trap",
        route = "door",
        direction = movement.name(),
        scope = scope.raw()
    )
    .entered();
    let moved = pipeline.focus_move_in(scope, movement);
    settle(pipeline);
    let landing = focused_name(pipeline);
    tracing::info!(
        target: "alacer::focus_trap",
        moved,
        landing = landing.as_str(),
        "{beat}"
    );
}

/// The availability write at the trap: the mutate seam, the write in
/// place, the write's own masks on the guard — the door answers the move
/// it made, whichever control holds the flag (the m22/m24 example's
/// pattern, applied to the modal's controls).
fn hold_available(pipeline: &mut PassPipeline, node: alacer_core::NodeId, available: bool) -> bool {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        if input.set_enabled(available) {
            guard.request(Change::PAINT.union(Change::A11Y));
            return true;
        }
        return false;
    }
    if let Some(checkbox) = guard.widget_mut().as_any_mut().downcast_mut::<Checkbox>() {
        if checkbox.set_enabled(available) {
            guard.request(Change::PAINT.union(Change::A11Y));
            return true;
        }
        return false;
    }
    if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        if button.set_enabled(available) {
            guard.request(Change::PAINT.union(Change::A11Y));
            return true;
        }
    }
    false
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let _ = pipeline.add_child(root, Label::new("The focus trap (ADR-0031)").boxed());

    // The background control the trap must never offer: the unscoped door
    // sees it, the scoped door does not.
    let _ = pipeline.add_child(
        root,
        Button::new("Background").on_click(|| {
            let _route = tracing::trace_span!(
                target: "alacer::focus_trap",
                "background",
                route = "press"
            )
            .entered();
        }),
    );

    // The modal: the column whose root is the scope the demo's door takes.
    let modal = pipeline.add_child(root, Column::new().gap(8.0).padding(12.0).boxed());
    let _ = pipeline.add_child(modal, TextInput::new("Recipient").boxed());
    let _ = pipeline.add_child(modal, TextInput::new("Subject").boxed());
    let _ = pipeline.add_child(modal, Checkbox::new("Notify me").boxed());
    let _ = pipeline.add_child(
        modal,
        Button::new("Send modal").on_click(|| {
            let _route = tracing::trace_span!(
                target: "alacer::focus_trap",
                "send",
                route = "press"
            )
            .entered();
        }),
    );

    let _ = pipeline.add_child(
        root,
        Label::new("Tab still walks the whole window — the trap is the app's door").boxed(),
    );

    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::focus_trap",
        landing = focused_name(&pipeline).as_str(),
        scope = modal.raw(),
        "the mount frame: five candidates in the tree, none in the trap yet"
    );

    // Beat one: the walk forward, one stop per beat — inside the modal's
    // subtree only. From no focus the trap's first candidate takes the
    // walk; the background control is never a stop.
    trap_beat(
        &mut pipeline,
        "stop 1: the trap's first candidate — never the background control",
        FocusMove::Next,
        modal,
    );
    trap_beat(
        &mut pipeline,
        "stop 2: the next candidate inside the trap",
        FocusMove::Next,
        modal,
    );
    trap_beat(
        &mut pipeline,
        "stop 3: the checkbox answers the scoped walk too",
        FocusMove::Next,
        modal,
    );
    trap_beat(
        &mut pipeline,
        "stop 4: the send button is a stop like any other",
        FocusMove::Next,
        modal,
    );
    trap_beat(&mut pipeline, "stop 5: the walk wraps inside the trap", FocusMove::Next, modal);

    // Beat two: backward — the same scoped funnel, the reverse direction.
    trap_beat(&mut pipeline, "backward: the trap reverses", FocusMove::Prev, modal);

    // Beat three: the honest refusal. The availability tier holds every
    // control in the trap down; the scoped walk has no candidate to offer
    // and answers `false` — no masks, an idle frame, focus untouched.
    // Then the form is handed back: the window opens with the trap usable.
    let mut trap_nodes = Vec::new();
    pipeline
        .arena()
        .for_each_pre_order(&mut |id, _| trap_nodes.push(id));
    for node in &trap_nodes {
        if *node != modal && is_inside(&pipeline, *node, modal) {
            hold_available(&mut pipeline, *node, false);
        }
    }
    settle(&mut pipeline);
    trap_beat(
        &mut pipeline,
        "refused: every control in the trap unavailable — no candidate, an idle frame",
        FocusMove::Next,
        modal,
    );
    for node in &trap_nodes {
        if *node != modal && is_inside(&pipeline, *node, modal) {
            hold_available(&mut pipeline, *node, true);
        }
    }
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::focus_trap",
        "the trap handed back — the window's Tab rides the unscoped door"
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer focus trap",
        (420.0, 300.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}

/// `true` when `node` lies inside the subtree rooted at `scope` — the
/// same ancestor walk the scoped door's candidate filter rides.
fn is_inside(
    pipeline: &PassPipeline,
    node: alacer_core::NodeId,
    scope: alacer_core::NodeId,
) -> bool {
    let mut up = Some(node);
    while let Some(current) = up {
        if current == scope {
            return true;
        }
        up = pipeline.arena().parent(current);
    }
    false
}
