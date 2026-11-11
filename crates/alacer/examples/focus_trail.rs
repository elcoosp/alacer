//! The focus walk joins the causal log (§13.3, ADR-0034): every
//! focus-move door ride — the keyboard's Tab fallback and the app's code
//! alike — streams one `InspectEvent::Focus` the moment the door
//! resolves, carrying a `FocusRecord` (the movement, the scope that
//! bounded the walk, and the door's own `FocusMoveOutcome` verbatim).
//! ADR-0033's pre-registered re-open rule cashed: the answer M27 made
//! data now reaches the causal log too, so a status line, an inspector
//! trail, or a test routes on "why did focus change — or why didn't it"
//! without re-implementing the observation at the call site.
//!
//! The demo walks the M26 window (the two-by-two grid and the modal's
//! form) one beat per `FocusMove` at startup, before the window opens,
//! with the causal observer riding — the same pipeline the window runs.
//! Every beat logs what the walk answered as data on the
//! `alacer::focus_trail` target, the workspace's per-seam idiom, and the
//! final trail read shows the records the doors streamed, the keyboard's
//! Tab rides among them:
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example focus_trail
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path.
//!
//! Run with `cargo run -p alacer --example focus_trail`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Checkbox, Column, Label, Row, TextInput};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::geom::Size;
use alacer_core::inspect::{FocusRecord, InspectEvent, InspectObserver};
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

/// The causal trail the demo shares between the pipeline's observer and
/// its own logging: a bounded ring (the recorder's own discipline,
/// §14.3 — devtools never unbounded) of the records the doors streamed.
#[derive(Clone, Default)]
struct Trail {
    focus: std::rc::Rc<std::cell::RefCell<std::collections::VecDeque<FocusRecord>>>,
}

impl Trail {
    const CAP: usize = 64;

    /// The observer half: boxed into [`PassPipeline::set_observer`], it
    /// receives every `InspectEvent` and keeps the focus records.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            focus: std::rc::Rc::clone(&self.focus),
        }
    }

    /// The read half: the same records, oldest first when drained.
    fn reader(&self) -> TrailReader {
        TrailReader {
            focus: std::rc::Rc::clone(&self.focus),
        }
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    focus: std::rc::Rc<std::cell::RefCell<std::collections::VecDeque<FocusRecord>>>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::Focus(record) = event {
            let mut ring = self.focus.borrow_mut();
            if ring.len() >= Trail::CAP {
                ring.pop_front();
            }
            ring.push_back(record);
        }
    }
}

/// The trail's read handle: the consumer's route over the streamed
/// records (a status line's source, here the demo's log).
struct TrailReader {
    focus: std::rc::Rc<std::cell::RefCell<std::collections::VecDeque<FocusRecord>>>,
}

impl TrailReader {
    /// The most recent record the doors streamed.
    fn last(&self) -> Option<FocusRecord> {
        self.focus.borrow().back().copied()
    }

    /// The whole trail so far, oldest first.
    fn all(&self) -> Vec<FocusRecord> {
        self.focus.borrow().iter().copied().collect()
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

/// The name the tree carries for `node` — the trail's endpoints are read
/// back through the a11y vocabulary, never widget internals.
fn tree_name(pipeline: &PassPipeline, node: alacer_core::NodeId) -> String {
    pipeline.arena().widget(node).label().resolve()
}

/// The record as the trail's reader renders it: the movement and scope
/// it names, the landing's endpoints and wrap, or the refusal's species.
fn describe(pipeline: &PassPipeline, record: &FocusRecord) -> String {
    let scope = record.scope.map_or_else(
        || "the window".to_string(),
        |node| format!("scope {}", tree_name(pipeline, node)),
    );
    let movement = record.movement.name();
    match record.outcome {
        FocusMoveOutcome::Moved(landing) => {
            let from = landing
                .from
                .map_or_else(|| "nothing".to_string(), |node| tree_name(pipeline, node));
            let wrapped = if landing.wrapped { " (wrapped)" } else { "" };
            format!(
                "{movement} @ {scope}: moved {from} → {}{wrapped}",
                tree_name(pipeline, landing.to)
            )
        }
        FocusMoveOutcome::Refused(reason) => {
            let species = match reason {
                FocusRefusal::NoCandidates => "no candidates".to_string(),
                FocusRefusal::DeadScope { scope } => {
                    format!("dead scope ({})", scope.raw())
                }
                FocusRefusal::NoGeometry { node } => {
                    format!("no geometry on {}", tree_name(pipeline, node))
                }
                FocusRefusal::NothingInDirection { node, direction } => format!(
                    "nothing lies {} of {}",
                    direction.name(),
                    tree_name(pipeline, node)
                ),
                FocusRefusal::AlreadyFocused { node } => {
                    format!("{} already holds focus", tree_name(pipeline, node))
                }
            };
            format!("{movement} @ {scope}: refused — {species}")
        }
    }
}

/// One beat: the span names the route, the door answers, the record
/// streams, the frame settles, and the log reads the streamed record
/// beside the door's own answer — the caller's fact and the log's fact,
/// one fact.
fn beat(
    pipeline: &mut PassPipeline,
    reader: &TrailReader,
    label: &'static str,
    movement: FocusMove,
) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_trail",
        "beat",
        route = "door",
        direction = movement.name()
    )
    .entered();
    let outcome = pipeline.focus_move(movement);
    settle(pipeline);
    let streamed = reader.last().expect("the door streamed its record");
    assert_eq!(
        streamed.movement, movement,
        "the record names the ride that streamed it"
    );
    assert_eq!(
        outcome.moved(),
        streamed.outcome.moved(),
        "the log's answer is the door's answer"
    );
    tracing::info!(
        target: "alacer::focus_trail",
        moved = outcome.moved(),
        record = describe(pipeline, &streamed).as_str(),
        "{label}"
    );
}

/// One scoped beat: the same door with the modal's subtree as the scope
/// — the record names the scope the walk was bounded to.
fn scoped_beat(
    pipeline: &mut PassPipeline,
    reader: &TrailReader,
    label: &'static str,
    movement: FocusMove,
    scope: alacer_core::NodeId,
) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_trail",
        "beat",
        route = "scoped",
        direction = movement.name(),
        scope = scope.raw()
    )
    .entered();
    let outcome = pipeline.focus_move_in(scope, movement);
    settle(pipeline);
    let streamed = reader.last().expect("the door streamed its record");
    assert_eq!(streamed.scope, Some(scope), "the record names its scope");
    assert_eq!(
        outcome.moved(),
        streamed.outcome.moved(),
        "the log's answer is the door's answer"
    );
    tracing::info!(
        target: "alacer::focus_trail",
        moved = outcome.moved(),
        record = describe(pipeline, &streamed).as_str(),
        "{label}"
    );
}

/// Holds the trap's controls down through the mutate seam (the m22
/// write in place, the write's own masks on the guard) — the held-down
/// trap the scoped beat walks against.
fn hold_down(
    pipeline: &mut PassPipeline,
    recipient: alacer_core::NodeId,
    notify: alacer_core::NodeId,
) {
    for (node, is_input) in [(recipient, true), (notify, false)] {
        let mut guard = pipeline.arena_mut().mutate(node);
        let widget = guard.widget_mut();
        let moved = if is_input {
            widget
                .as_any_mut()
                .downcast_mut::<TextInput>()
                .is_some_and(|input| input.set_enabled(false))
        } else {
            widget
                .as_any_mut()
                .downcast_mut::<Checkbox>()
                .is_some_and(|checkbox| checkbox.set_enabled(false))
        };
        if moved {
            guard.request(alacer_core::mask::Change::PAINT.union(alacer_core::mask::Change::A11Y));
        }
    }
}

/// One keyboard beat: Tab through the real dispatch path — the same
/// funnel, so the same records stream for the keyboard's walks.
fn keyboard_beat(pipeline: &mut PassPipeline, reader: &TrailReader) {
    let _route =
        tracing::trace_span!(target: "alacer::focus_trail", "beat", route = "keyboard").entered();
    let response = pipeline.dispatch_key(&KeyEvent {
        key: Key::Tab,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    });
    settle(pipeline);
    let tab_moved = matches!(response, alacer::core::EventResponse::Handled);
    let streamed = reader.last().expect("the funnel streamed Tab's ride");
    assert_eq!(
        tab_moved,
        streamed.outcome.moved(),
        "the keyboard's trail is the door's answer"
    );
    tracing::info!(
        target: "alacer::focus_trail",
        moved = tab_moved,
        record = describe(pipeline, &streamed).as_str(),
        "stop 7: the keyboard's Tab rides the same funnel — the same records stream"
    );
}

/// The trail, read back: every ride the demo took, oldest first — the
/// log's answers as data, ready for a status line or a test.
fn read_back(trail: &Trail) {
    let trail_read = trail.reader().all();
    tracing::info!(
        target: "alacer::focus_trail",
        records = trail_read.len(),
        refusals = trail_read
            .iter()
            .filter(|record| !record.outcome.moved())
            .count(),
        "the walk's answers streamed as they resolved — the causal log carries the trail"
    );
}

fn main() {
    let mut pipeline = pipeline_default();
    let trail = Trail::default();
    pipeline.set_observer(Box::new(trail.writer()));

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let _ = pipeline.add_child(
        root,
        Label::new("The walk's answers, streamed to the causal log (ADR-0034)").boxed(),
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
        Label::new("Tab rides the same funnel; the trail records every door").boxed(),
    );

    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::focus_trail",
        landing = pipeline
            .focus()
            .map_or_else(|| "nothing".to_string(), |node| tree_name(&pipeline, node)),
        "the mount frame: the grid and the form are mounted, nothing focused yet"
    );

    // Beat one: the entry — the record carries the landing's `from:
    // None` and the walk's resting node.
    beat(
        &mut pipeline,
        &trail.reader(),
        "stop 1: the entry streams its landing",
        FocusMove::Next,
    );

    // Beat two: the geometry's answer — Down lands what the bounds name.
    beat(
        &mut pipeline,
        &trail.reader(),
        "stop 2: down the grid — the geometry's answer streams",
        FocusMove::Down,
    );

    // Beat three: the edge refusal — the species streams too.
    beat(
        &mut pipeline,
        &trail.reader(),
        "stop 3: the corner refuses — the species streams",
        FocusMove::Right,
    );

    // Beat four: the tree still wraps from the corner — the landing says
    // wrapped, in the log as in the caller's hand.
    beat(
        &mut pipeline,
        &trail.reader(),
        "stop 4: the tree wraps — the wrapped landing streams",
        FocusMove::Next,
    );

    // Beat five: the scoped walk — the record names the scope.
    scoped_beat(
        &mut pipeline,
        &trail.reader(),
        "stop 5: the scoped ride names its scope",
        FocusMove::Next,
        modal,
    );

    // Beat six: hold the trap's controls down through the mutate seam —
    // the scoped door refuses, and the empty-list species streams.
    hold_down(&mut pipeline, recipient, notify);
    settle(&mut pipeline);
    scoped_beat(
        &mut pipeline,
        &trail.reader(),
        "stop 6: the held-down trap refuses — the species streams",
        FocusMove::Next,
        modal,
    );

    // The keyboard's ride, then the trail read back in full.
    keyboard_beat(&mut pipeline, &trail.reader());
    read_back(&trail);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer focus trail",
        (420.0, 320.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
