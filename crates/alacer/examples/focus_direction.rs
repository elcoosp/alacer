//! The directional focus walk as a consumer's vocabulary (§10.3,
//! ADR-0032): the walk reads the window, not only the tree. A two-by-two
//! grid of buttons shares one window with a modal's form; every
//! `PassPipeline::focus_move` beat names a *direction* — `Left`/`Right`/
//! `Up`/`Down` — and the walk answers with the candidate the geometry
//! names: fully beyond the focused node's edge in the direction, nearest
//! edge distance first. The tree-order moves (`Next`/`Prev`) keep the
//! pre-order contract and their wrap; the directional moves never wrap —
//! the window's edge is the walk's end, and the refusal is honest: an
//! idle frame, focus untouched, the help carried as plain diagnostic text
//! on the `alacer::focus` target. The scoped door bounds the directional
//! walk exactly as it bounds the tree-order one: `focus_move_in(scope,
//! FocusMove::Right)` never escapes the modal's subtree.
//!
//! The demo walks the beats at startup, before the window opens, and
//! every landing rides the `alacer::focus_direction` tracing target —
//! the workspace's per-seam idiom, `RUST_LOG=alacer=info` to watch:
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example focus_direction
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path.
//!
//! Run with `cargo run -p alacer --example focus_direction`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Checkbox, Column, Label, Row, TextInput};
use alacer_core::geom::Size;
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
const VIEWPORT: Size = Size::new(420.0, 320.0);

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

/// One directional beat: the span names the route and the direction, the
/// door answers, the frame settles, and the landing — or the honest
/// refusal — is announced on the demo's target.
fn direction_beat(pipeline: &mut PassPipeline, beat: &'static str, movement: FocusMove) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_direction",
        "direction",
        route = "door",
        direction = movement.name()
    )
    .entered();
    let moved = pipeline.focus_move(movement);
    settle(pipeline);
    let landing = focused_name(pipeline);
    tracing::info!(
        target: "alacer::focus_direction",
        moved,
        landing = landing.as_str(),
        "{beat}"
    );
}

/// One scoped directional beat: the same door, the modal's subtree as the
/// scope — the walk answers inside the form or refuses inside it.
fn scoped_beat(
    pipeline: &mut PassPipeline,
    beat: &'static str,
    movement: FocusMove,
    scope: alacer_core::NodeId,
) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_direction",
        "direction",
        route = "scoped",
        direction = movement.name(),
        scope = scope.raw()
    )
    .entered();
    let moved = pipeline.focus_move_in(scope, movement);
    settle(pipeline);
    let landing = focused_name(pipeline);
    tracing::info!(
        target: "alacer::focus_direction",
        moved,
        landing = landing.as_str(),
        "{beat}"
    );
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let _ = pipeline.add_child(
        root,
        Label::new("The directional walk (ADR-0032): arrows read the window").boxed(),
    );

    // The grid that prices the seam: the tree order (top-left, top-right,
    // bottom-left, bottom-right) and the spatial reading of the same
    // window disagree about what "next" means — the row breaks the
    // pre-order list's stride, the direction does not.
    let grid = pipeline.add_child(root, Column::new().gap(8.0).boxed());
    let top = pipeline.add_child(grid, Row::new().gap(8.0).boxed());
    let _ = pipeline.add_child(top, Button::new("Top left").on_click(|| {}).boxed());
    let _ = pipeline.add_child(top, Button::new("Top right").on_click(|| {}).boxed());
    let bottom = pipeline.add_child(grid, Row::new().gap(8.0).boxed());
    let _ = pipeline.add_child(bottom, Button::new("Bottom left").on_click(|| {}).boxed());
    let _ = pipeline.add_child(bottom, Button::new("Bottom right").on_click(|| {}).boxed());

    // The modal whose form the scoped directional beats walk: the scope
    // bounds the walk, never the candidate list (ADR-0031's rule, now for
    // every direction).
    let modal = pipeline.add_child(root, Column::new().gap(8.0).padding(12.0).boxed());
    let form_top = pipeline.add_child(modal, Row::new().gap(8.0).boxed());
    let _ = pipeline.add_child(form_top, TextInput::new("Form left").boxed());
    let _ = pipeline.add_child(form_top, TextInput::new("Form right").boxed());
    let _ = pipeline.add_child(modal, Checkbox::new("Form below").boxed());

    let _ = pipeline.add_child(
        root,
        Label::new("Tab still walks the tree; the edges refuse — space never wraps").boxed(),
    );

    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::focus_direction",
        landing = focused_name(&pipeline).as_str(),
        "the mount frame: the grid and the form are mounted, nothing focused yet"
    );

    // Beat one: the entry contract — from no focus the walk rests on the
    // first candidate, whichever direction the caller names. The walk has
    // to rest somewhere before a direction can mean anything.
    direction_beat(
        &mut pipeline,
        "stop 1: the entry — the walk rests on the first candidate, Right or Tab alike",
        FocusMove::Right,
    );

    // Beat two: the spatial answer — Right along the top row lands the
    // top-right button; the tree-order Next would land the same stop here,
    // which is exactly why the discriminator below matters.
    direction_beat(&mut pipeline, "stop 2: right along the top row", FocusMove::Right);

    // Beat three: the window, not the tree — Down from the top-right
    // lands the bottom-right button. The pre-order list's next stop is
    // the bottom-LEFT button (the row breaks the tree's stride); the
    // direction reads the geometry instead.
    direction_beat(&mut pipeline, "stop 3: down the right column — the geometry's answer", FocusMove::Down);

    // Beat four: the honest refusal — the bottom-right button is the
    // window's corner; Right has no candidate fully beyond its edge, the
    // door answers false, and the frame after is idle. Space never wraps.
    direction_beat(
        &mut pipeline,
        "refused: the corner — nothing lies right of the right column, no masks",
        FocusMove::Right,
    );

    // Beat five: the tree-order funnel is untouched — Next from the same
    // corner wraps to the tree's first candidate, the wrap being the
    // tree-order contract and only the tree-order contract.
    direction_beat(
        &mut pipeline,
        "the tree still wraps: Next from the corner lands the tree's first stop",
        FocusMove::Next,
    );

    // Beat six: the scoped directional walk — inside the modal's form,
    // Right walks the form's row and never escapes the subtree; the
    // background grid is never a stop.
    scoped_beat(
        &mut pipeline,
        "scoped: the form's first stop — the walk enters the trap",
        FocusMove::Right,
        modal,
    );
    scoped_beat(
        &mut pipeline,
        "scoped: right along the form's row",
        FocusMove::Right,
        modal,
    );
    scoped_beat(
        &mut pipeline,
        "scoped: down to the form's checkbox",
        FocusMove::Down,
        modal,
    );
    scoped_beat(
        &mut pipeline,
        "scoped refused: the form's edge — the walk stays in the trap, an idle frame",
        FocusMove::Right,
        modal,
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer focus directions",
        (420.0, 320.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
