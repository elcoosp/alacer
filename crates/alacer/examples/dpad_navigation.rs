//! The directional keys ride the walk (§10.3, ADR-0035): a plain arrow
//! the focused widget did not answer — a D-pad button, a TV remote's
//! ring — maps onto its `FocusMove` at the same fallback seam Tab and
//! Shift+Tab ride since M24, and lands the geometry's answer through the
//! real dispatch path. The rule ADR-0032 pre-registered (and ADR-0033
//! re-stated) cashed: the platform's key map already carries the four
//! arrows (`alacer-platform/src/keys.rs`), so the seam is one `match` in
//! `PassPipeline::dispatch_key` — never a second dispatch channel.
//!
//! The contract the demo walks: the widget's answer always takes
//! precedence (a focused slider steps by its own rule — the window's
//! arrows are only the window's); a modifier held is a chord, and the
//! chords are the widgets' vocabulary; a refused direction falls through
//! honestly (`EventResponse::Ignored`, the key nobody's), and the M28
//! causal trail carries the arrows' rides like every ride on the funnel.
//!
//! The demo walks the M26 window (the two-by-two grid whose tree order
//! and spatial reading disagree, plus a slider the widget's own arrows
//! answer) with arrow presses through `dispatch_key` at startup, before
//! the window opens, and every press logs what the walk answered on the
//! `alacer::dpad_navigation` target — the workspace's per-seam idiom,
//! `RUST_LOG=alacer=info` to watch:
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example dpad_navigation
//! ```
//!
//! In the window, the arrows walk the grid live; Tab walks the tree; the
//! focused slider's arrows stay the slider's. `ALACER_SOFTWARE=1` forces
//! the softbuffer path.
//!
//! Run with `cargo run -p alacer --example dpad_navigation`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Row, Slider};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::geom::Size;
use alacer_core::passes::{FocusMove, PassPipeline};

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

/// A plain directional press: pressed state, no modifiers — the D-pad
/// button's own shape (the chords are the widgets' vocabulary).
fn dpad(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let report = pipeline.tick(VIEWPORT).expect("the demo frame lays out");
        if report.is_idle() {
            return;
        }
    }
}

/// The name the tree carries for `node` — the landing's endpoint is read
/// back through the a11y vocabulary, never widget internals.
fn tree_name(pipeline: &PassPipeline, node: alacer_core::NodeId) -> String {
    pipeline.arena().widget(node).label().resolve()
}

/// One press: the span names the route, the real dispatch answers, and
/// the landing — or the honest fall-through — is logged.
fn press(pipeline: &mut PassPipeline, label: &'static str, key: Key, movement: FocusMove) {
    let _route = tracing::trace_span!(
        target: "alacer::dpad_navigation",
        "press",
        direction = movement.name()
    )
    .entered();
    let response = pipeline.dispatch_key(&dpad(key));
    settle(pipeline);
    let landing = pipeline
        .focus()
        .map_or_else(|| "nothing".to_string(), |node| tree_name(pipeline, node));
    let claimed = matches!(response, alacer::core::EventResponse::Handled);
    tracing::info!(
        target: "alacer::dpad_navigation",
        claimed,
        landing = landing.as_str(),
        "{label}"
    );
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let _ = pipeline.add_child(
        root,
        Label::new("The ring's arrows ride the walk (ADR-0035)").boxed(),
    );

    // The M26 window: the two-by-two grid whose tree order and spatial
    // reading disagree, plus the slider whose own arrows are its
    // vocabulary — the widget's answer takes precedence.
    let grid = pipeline.add_child(root, Column::new().gap(8.0).boxed());
    let top = pipeline.add_child(grid, Row::new().gap(8.0).boxed());
    let _ = pipeline.add_child(top, Button::new("Top left").on_click(|| {}).boxed());
    let _ = pipeline.add_child(top, Button::new("Top right").on_click(|| {}).boxed());
    let bottom = pipeline.add_child(grid, Row::new().gap(8.0).boxed());
    let _ = pipeline.add_child(bottom, Button::new("Bottom left").on_click(|| {}).boxed());
    let _ = pipeline.add_child(bottom, Button::new("Bottom right").on_click(|| {}).boxed());

    let volume = pipeline.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .step(1.0)
            .on_change(|_| {})
            .boxed(),
    );

    let _ = pipeline.add_child(
        root,
        Label::new("Arrows walk the grid; the focused slider keeps its own").boxed(),
    );

    settle(&mut pipeline);

    // Beat one: the entry — from no focus the walk takes the first
    // candidate (the entry contract, direction-agnostic).
    press(
        &mut pipeline,
        "press 1: Right from nothing — the walk takes the first candidate",
        Key::ArrowRight,
        FocusMove::Right,
    );

    // Beat two: the stride — the geometry's answer, not the tree's next
    // stop.
    press(
        &mut pipeline,
        "press 2: Right again — the geometry's answer",
        Key::ArrowRight,
        FocusMove::Right,
    );

    // Beat three: the edge refusal falls through — the key is nobody's.
    press(
        &mut pipeline,
        "press 3: Right at the edge — refused, the key falls through",
        Key::ArrowRight,
        FocusMove::Right,
    );

    // Beat four: the widget's own arrows — focus the slider through the
    // tree-order walk (Tab's direction), then the slider's Right steps
    // the value and never moves the focus.
    press(
        &mut pipeline,
        "press 4: Next walks to the slider — Tab's direction",
        Key::Tab,
        FocusMove::Next,
    );
    let focused = pipeline.focus();
    if focused == Some(volume) {
        let _route = tracing::trace_span!(
            target: "alacer::dpad_navigation",
            "press",
            direction = "right"
        )
        .entered();
        let response = pipeline.dispatch_key(&dpad(Key::ArrowRight));
        settle(&mut pipeline);
        let claimed = matches!(response, alacer::core::EventResponse::Handled);
        tracing::info!(
            target: "alacer::dpad_navigation",
            claimed,
            landing = tree_name(&pipeline, volume).as_str(),
            "press 5: Right on the focused slider — the widget's answer takes precedence"
        );
    }

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer D-pad navigation",
        (420.0, 320.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
