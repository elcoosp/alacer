//! The focus walk as a consumer's vocabulary (§10.3, ADR-0030): the
//! traversal order is the pipeline's business, the doors are the app's. A
//! four-control form — two inputs, a checkbox, a send button — walks one
//! stop per `PassPipeline::focus_move` beat: forward, backward, and the
//! honest refusal when the availability tier holds every control down
//! (no candidate, no masks — an idle frame, never a first-candidate
//! reset). In the window, Tab and Shift+Tab ride the *same* funnel: the
//! keyboard's fallback route and the consumer's door land on the same
//! stops, because there is only one walk.
//!
//! The demo walks the beats at startup, before the window opens, and
//! every landing rides the `alacer::focus_traversal` tracing target —
//! the workspace's per-seam idiom, `RUST_LOG=alacer=info` to watch:
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example focus_traversal
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path.
//!
//! Run with `cargo run -p alacer --example focus_traversal`.

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

/// One focus_move beat: the span names the route, the door answers, the
/// frame settles, and the landing is announced on the demo's target.
fn walk_beat(pipeline: &mut PassPipeline, beat: &'static str, movement: FocusMove) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_traversal",
        "walk",
        route = "door",
        direction = movement.name()
    )
    .entered();
    let moved = pipeline.focus_move(movement);
    settle(pipeline);
    let landing = focused_name(pipeline);
    tracing::info!(
        target: "alacer::focus_traversal",
        moved,
        landing = landing.as_str(),
        "{beat}"
    );
}

/// The availability write at the walk: the mutate seam, the write in
/// place, the write's own masks on the guard — the door answers the move
/// it made, whichever control holds the flag (the m22 example's pattern,
/// applied to the whole form).
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

    let _ = pipeline.add_child(root, Label::new("The focus walk (ADR-0030)").boxed());

    // The form the walk demonstrates on: every control focusable and
    // available — the candidate list pass 2 recomputes, in tree order.
    let _ = pipeline.add_child(root, TextInput::new("Name").boxed());
    let _ = pipeline.add_child(root, TextInput::new("Email").boxed());
    let _ = pipeline.add_child(root, Checkbox::new("Subscribe").boxed());
    let _ = pipeline.add_child(
        root,
        Button::new("Send").on_click(|| {
            let _route = tracing::trace_span!(
                target: "alacer::focus_traversal",
                "send",
                route = "press"
            )
            .entered();
        }),
    );

    let _ = pipeline.add_child(
        root,
        Label::new("Tab and Shift+Tab ride the same walk — the window proves the funnel")
            .boxed(),
    );

    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::focus_traversal",
        landing = focused_name(&pipeline).as_str(),
        "the mount frame: four candidates, no focus"
    );

    // Beat one: the walk forward, one stop per beat — the door answers,
    // the frame settles, the landing is announced.
    walk_beat(&mut pipeline, "stop 1: the walk takes the first candidate", FocusMove::Next);
    walk_beat(&mut pipeline, "stop 2: the next candidate in tree order", FocusMove::Next);
    walk_beat(&mut pipeline, "stop 3: the checkbox answers the walk too", FocusMove::Next);
    walk_beat(&mut pipeline, "stop 4: the send button is a stop like any other", FocusMove::Next);

    // Beat two: backward — the same funnel, the reverse direction.
    walk_beat(&mut pipeline, "backward: the walk reverses", FocusMove::Prev);

    // Beat three: the honest refusal. The availability tier holds every
    // control down; the walk has no candidate to offer and answers
    // `false` — no masks, an idle frame, focus stays where it was. Then
    // the form is handed back: the window opens with the walk usable.
    let mut form_nodes = Vec::new();
    pipeline
        .arena()
        .for_each_pre_order(&mut |id, _| form_nodes.push(id));
    for node in &form_nodes {
        hold_available(&mut pipeline, *node, false);
    }
    settle(&mut pipeline);
    walk_beat(
        &mut pipeline,
        "refused: every control unavailable — no candidate, an idle frame",
        FocusMove::Next,
    );
    for node in &form_nodes {
        hold_available(&mut pipeline, *node, true);
    }
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::focus_traversal",
        "the form handed back — the window's Tab rides the same funnel"
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer focus traversal",
        (420.0, 300.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
