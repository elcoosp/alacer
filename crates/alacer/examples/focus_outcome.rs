//! The focus walk's answer as a consumer's vocabulary (§10.3, ADR-0033):
//! the two doors (`PassPipeline::focus_move`, `PassPipeline::focus_move_in`)
//! return one `FocusMoveOutcome` instead of the boolean they took through
//! M26 — ADR-0031's pre-registered re-open rule cashed. A `Moved` landing
//! carries its endpoints (`from`, `to`) and the wrap; a `Refused` names its
//! species: the empty list, the dead scope, the geometry-less focus, the
//! empty direction (with its node and direction), the single-candidate
//! stall. The walk behind the doors is untouched — the same candidate
//! list, the same availability filter, the same wrap, the same immediate
//! resolution, refusals that queue no masks — the answer just stops making
//! the caller guess at a bare `false`.
//!
//! The demo walks the M26 window (the two-by-two grid and the modal's
//! form) one beat per `FocusMove` at startup, before the window opens,
//! and every beat logs what the walk answered as data on the
//! `alacer::focus_outcome` target — the workspace's per-seam idiom,
//! `RUST_LOG=alacer=info` to watch:
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example focus_outcome
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path.
//!
//! Run with `cargo run -p alacer --example focus_outcome`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Checkbox, Column, Label, Row, TextInput};
use alacer_core::geom::Size;
use alacer_core::passes::{FocusMove, FocusMoveOutcome, FocusRefusal, PassPipeline};

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

/// The name the tree carries for `node` — the outcome's endpoints are read
/// back through the a11y vocabulary, never widget internals.
fn tree_name(pipeline: &PassPipeline, node: alacer_core::NodeId) -> String {
    pipeline.arena().widget(node).label().resolve()
}

/// The outcome as the consumer reads it: the landing's endpoints and wrap,
/// or the refusal's species — every field resolved through the tree's own
/// names, so the log line is the a11y truth, not node plumbing.
fn describe(pipeline: &PassPipeline, outcome: &FocusMoveOutcome) -> String {
    match *outcome {
        FocusMoveOutcome::Moved(landing) => {
            let from = landing
                .from
                .map_or_else(|| "nothing".to_string(), |node| tree_name(pipeline, node));
            let wrapped = if landing.wrapped { " (wrapped)" } else { "" };
            format!(
                "moved: {from} → {}{wrapped}",
                tree_name(pipeline, landing.to)
            )
        }
        FocusMoveOutcome::Refused(reason) => match reason {
            FocusRefusal::NoCandidates => {
                "refused: no candidates — an empty arena, or a trap the tier holds down".to_string()
            }
            FocusRefusal::DeadScope { scope } => {
                format!("refused: the scope is dead ({} left the tree)", scope.raw())
            }
            FocusRefusal::NoGeometry { node } => format!(
                "refused: {} carries no geometry yet — tick one frame first",
                tree_name(pipeline, node)
            ),
            FocusRefusal::NothingInDirection { node, direction } => format!(
                "refused: nothing lies {} of {} — space never wraps",
                direction.name(),
                tree_name(pipeline, node)
            ),
            FocusRefusal::AlreadyFocused { node } => format!(
                "refused: {} already holds focus — the single-candidate stall",
                tree_name(pipeline, node)
            ),
        },
    }
}

/// One beat: the span names the route, the door answers, the frame
/// settles, and the outcome — landing or species — is logged as data.
fn beat(pipeline: &mut PassPipeline, label: &'static str, movement: FocusMove) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_outcome",
        "beat",
        route = "door",
        direction = movement.name()
    )
    .entered();
    let outcome = pipeline.focus_move(movement);
    settle(pipeline);
    tracing::info!(
        target: "alacer::focus_outcome",
        moved = outcome.moved(),
        answer = describe(pipeline, &outcome).as_str(),
        "{label}"
    );
}

/// One scoped beat: the same door with the modal's subtree as the scope —
/// the walk answers inside the form or refuses inside it.
fn scoped_beat(
    pipeline: &mut PassPipeline,
    label: &'static str,
    movement: FocusMove,
    scope: alacer_core::NodeId,
) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_outcome",
        "beat",
        route = "scoped",
        direction = movement.name(),
        scope = scope.raw()
    )
    .entered();
    let outcome = pipeline.focus_move_in(scope, movement);
    settle(pipeline);
    tracing::info!(
        target: "alacer::focus_outcome",
        moved = outcome.moved(),
        answer = describe(pipeline, &outcome).as_str(),
        "{label}"
    );
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let _ = pipeline.add_child(
        root,
        Label::new("The walk's answer as data (ADR-0033)").boxed(),
    );

    // The M26 window: the two-by-two grid whose tree order and spatial
    // reading disagree, plus the modal's form the scoped beats walk.
    let grid = pipeline.add_child(root, Column::new().gap(8.0).boxed());
    let top = pipeline.add_child(grid, Row::new().gap(8.0).boxed());
    let _ = pipeline.add_child(top, Button::new("Top left").on_click(|| {}).boxed());
    let _ = pipeline.add_child(top, Button::new("Top right").on_click(|| {}).boxed());
    let bottom = pipeline.add_child(grid, Row::new().gap(8.0).boxed());
    let _ = pipeline.add_child(bottom, Button::new("Bottom left").on_click(|| {}).boxed());
    let _ = pipeline.add_child(bottom, Button::new("Bottom right").on_click(|| {}).boxed());

    let modal = pipeline.add_child(root, Column::new().gap(8.0).padding(12.0).boxed());
    let form_top = pipeline.add_child(modal, Row::new().gap(8.0).boxed());
    let recipient = pipeline.add_child(form_top, TextInput::new("Recipient").boxed());
    let _ = pipeline.add_child(form_top, TextInput::new("Subject").boxed());
    let notify = pipeline.add_child(modal, Checkbox::new("Notify me").boxed());

    let _ = pipeline.add_child(
        root,
        Label::new("Tab still walks the tree; the refusals name their species").boxed(),
    );

    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::focus_outcome",
        landing = pipeline
            .focus()
            .map_or_else(|| "nothing".to_string(), |node| tree_name(&pipeline, node)),
        "the mount frame: the grid and the form are mounted, nothing focused yet"
    );

    // Beat one: the entry — the landing names its endpoints, `from` being
    // `None` (nothing held focus) and the wrap never set.
    beat(
        &mut pipeline,
        "stop 1: the entry — the landing carries from: nothing",
        FocusMove::Next,
    );

    // Beat two: the stride — both endpoints named.
    beat(
        &mut pipeline,
        "stop 2: the stride — from Top left to Top right",
        FocusMove::Next,
    );

    // Beat three: the geometry's answer — Down the right column.
    beat(
        &mut pipeline,
        "stop 3: down the right column — the geometry's answer",
        FocusMove::Down,
    );

    // Beat four: the edge refusal names its node and its direction.
    beat(
        &mut pipeline,
        "refused: the corner — the species is NothingInDirection",
        FocusMove::Right,
    );

    // Beat five: the tree still wraps from the same corner — and the
    // landing says wrapped.
    beat(
        &mut pipeline,
        "the tree wraps: Next lands the first stop, the landing says wrapped",
        FocusMove::Next,
    );

    // Beat six: the scoped walk inside the form — the trap answers with a
    // landing whose endpoints stay in the subtree.
    scoped_beat(
        &mut pipeline,
        "scoped: the form's first stop — the trap answers",
        FocusMove::Next,
        modal,
    );

    // Beat seven: hold the trap's controls down through the mutate seam —
    // the scoped door refuses with the empty-list species.
    let mut guard = pipeline.arena_mut().mutate(recipient);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        let _ = input.set_enabled(false);
        guard.request(alacer_core::mask::Change::PAINT.union(alacer_core::mask::Change::A11Y));
    }
    drop(guard);
    let mut guard = pipeline.arena_mut().mutate(notify);
    if let Some(checkbox) = guard.widget_mut().as_any_mut().downcast_mut::<Checkbox>() {
        let _ = checkbox.set_enabled(false);
        guard.request(alacer_core::mask::Change::PAINT.union(alacer_core::mask::Change::A11Y));
    }
    drop(guard);
    settle(&mut pipeline);
    scoped_beat(
        &mut pipeline,
        "scoped refused: the held-down trap — the species is NoCandidates",
        FocusMove::Next,
        modal,
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer focus outcomes",
        (420.0, 320.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
