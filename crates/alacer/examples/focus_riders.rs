//! The trail names the rider (§13.3, ADR-0036): every `FocusRecord` the
//! causal log streams carries a `FocusRider` beside its movement — the
//! keyboard's Tab fallback, the ring's directional keys, or the app's own
//! doors — so a trail reader answers "who moved focus — or why didn't it"
//! from the log, never from a re-implementation at the call site.
//! ADR-0035's pre-registered re-open rule cashed: one funnel, three
//! riders, one trail, and the attribution is data.
//!
//! The demo walks the M26 window (the two-by-two grid and the modal's
//! form) with all three riders at startup, before the window opens: the
//! code's doors, the keyboard's Tab through the real dispatch, and the
//! ring's arrows through the same dispatch. Every beat logs the streamed
//! record's rider beside the answer it names on the `alacer::focus_riders`
//! target, the workspace's per-seam idiom, and the final read-back shows
//! the attributed history the way a status line or an inspector client
//! would render it:
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example focus_riders
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path.
//!
//! Run with `cargo run -p alacer --example focus_riders`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Checkbox, Column, Label, Row, TextInput};
use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::geom::Size;
use alacer_core::inspect::{FocusRecord, FocusRider, InspectEvent, InspectObserver};
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
/// §14.3 — devtools never unbounded) of the records the riders streamed.
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
    /// The most recent record the riders streamed.
    fn last(&self) -> Option<FocusRecord> {
        self.focus.borrow().back().copied()
    }

    /// The whole trail so far, oldest first — the attributed history.
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

/// The rider as a trail reader renders it — the attribution a status
/// line leads with, the demo's log among them.
fn rider_name(rider: FocusRider) -> &'static str {
    match rider {
        FocusRider::Keyboard => "the keyboard's Tab",
        FocusRider::Ring => "the ring's arrow",
        FocusRider::Consumer => "the app's door",
    }
}

/// The record as the trail's reader renders it: the rider and the
/// movement it names, the landing's endpoints and wrap, or the refusal's
/// species — the answer and its attribution, one line.
fn describe(pipeline: &PassPipeline, record: &FocusRecord) -> String {
    let scope = record.scope.map_or_else(
        || "the window".to_string(),
        |node| format!("scope {}", tree_name(pipeline, node)),
    );
    let movement = record.movement.name();
    let rider = rider_name(record.rider);
    match record.outcome {
        FocusMoveOutcome::Moved(landing) => {
            let from = landing
                .from
                .map_or_else(|| "nothing".to_string(), |node| tree_name(pipeline, node));
            let wrapped = if landing.wrapped { " (wrapped)" } else { "" };
            format!(
                "{rider} asked {movement} @ {scope}: moved {from} → {}{wrapped}",
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
            format!("{rider} asked {movement} @ {scope}: refused — {species}")
        }
    }
}

/// One door beat: the app's own call, the record naming the consumer as
/// the rider — the log's attribution is the route's fact, one fact.
fn door_beat(
    pipeline: &mut PassPipeline,
    reader: &TrailReader,
    label: &'static str,
    movement: FocusMove,
) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_riders",
        "beat",
        route = "door",
        direction = movement.name()
    )
    .entered();
    let outcome = pipeline.focus_move(movement);
    settle(pipeline);
    let streamed = reader.last().expect("the door streamed its record");
    assert_eq!(
        streamed.rider,
        FocusRider::Consumer,
        "the door's ride names the consumer"
    );
    assert_eq!(
        outcome.moved(),
        streamed.outcome.moved(),
        "the log's answer is the door's answer"
    );
    tracing::info!(
        target: "alacer::focus_riders",
        moved = outcome.moved(),
        record = describe(pipeline, &streamed).as_str(),
        "{label}"
    );
}

/// One scoped door beat: the same attribution with the trap's scope
/// named beside the rider.
fn scoped_beat(
    pipeline: &mut PassPipeline,
    reader: &TrailReader,
    label: &'static str,
    movement: FocusMove,
    scope: alacer_core::NodeId,
) {
    let _route = tracing::trace_span!(
        target: "alacer::focus_riders",
        "beat",
        route = "scoped",
        direction = movement.name(),
        scope = scope.raw()
    )
    .entered();
    let outcome = pipeline.focus_move_in(scope, movement);
    settle(pipeline);
    let streamed = reader.last().expect("the scoped door streamed its record");
    assert_eq!(
        streamed.rider,
        FocusRider::Consumer,
        "the scoped door's ride names the consumer"
    );
    assert_eq!(streamed.scope, Some(scope), "the record names its scope");
    tracing::info!(
        target: "alacer::focus_riders",
        moved = outcome.moved(),
        record = describe(pipeline, &streamed).as_str(),
        "{label}"
    );
}

/// One keyboard beat: Tab through the real dispatch path — the same
/// funnel, the record naming the keyboard as the rider (M30, ADR-0036).
fn keyboard_beat(pipeline: &mut PassPipeline, reader: &TrailReader) {
    let _route =
        tracing::trace_span!(target: "alacer::focus_riders", "beat", route = "keyboard").entered();
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
        streamed.rider,
        FocusRider::Keyboard,
        "the keyboard's ride names the keyboard"
    );
    tracing::info!(
        target: "alacer::focus_riders",
        moved = tab_moved,
        record = describe(pipeline, &streamed).as_str(),
        "the keyboard's Tab: the trail names the rider"
    );
}

/// One ring beat: a plain arrow through the real dispatch path — the
/// same funnel, the record naming the ring as the rider.
fn ring_beat(pipeline: &mut PassPipeline, reader: &TrailReader, key: Key) {
    let _route =
        tracing::trace_span!(target: "alacer::focus_riders", "beat", route = "ring").entered();
    let response = pipeline.dispatch_key(&KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    });
    settle(pipeline);
    let arrow_moved = matches!(response, alacer::core::EventResponse::Handled);
    let streamed = reader.last().expect("the funnel streamed the arrow's ride");
    assert_eq!(
        streamed.rider,
        FocusRider::Ring,
        "the ring's ride names the ring"
    );
    tracing::info!(
        target: "alacer::focus_riders",
        moved = arrow_moved,
        record = describe(pipeline, &streamed).as_str(),
        "the ring's arrow: the trail names the rider"
    );
}

/// The trail, read back: every ride the demo took, oldest first, each
/// with its rider — the attributed history a status line or an inspector
/// client renders from the log alone.
fn read_back(trail: &Trail) {
    let trail_read = trail.reader().all();
    let by = |rider: FocusRider| {
        trail_read
            .iter()
            .filter(|record| record.rider == rider)
            .count()
    };
    tracing::info!(
        target: "alacer::focus_riders",
        records = trail_read.len(),
        keyboard = by(FocusRider::Keyboard),
        ring = by(FocusRider::Ring),
        consumer = by(FocusRider::Consumer),
        "one funnel, three riders, one trail — the log answers who rode"
    );
}

fn main() {
    let mut pipeline = pipeline_default();
    let trail = Trail::default();
    pipeline.set_observer(Box::new(trail.writer()));

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let _ = pipeline.add_child(
        root,
        Label::new("The trail names the rider (ADR-0036): keyboard, ring, or door").boxed(),
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
    let _ = pipeline.add_child(modal, TextInput::new("Recipient").boxed());
    let _ = pipeline.add_child(modal, Checkbox::new("Notify me").boxed());

    let _ = pipeline.add_child(
        root,
        Label::new("Tab and the arrows ride the same funnel; the trail names them").boxed(),
    );

    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::focus_riders",
        landing = pipeline
            .focus()
            .map_or_else(|| "nothing".to_string(), |node| tree_name(&pipeline, node)),
        "the mount frame: the grid and the form are mounted, nothing focused yet"
    );

    // The code's doors: the entry and the geometry's answer, the records
    // naming the consumer as the rider.
    door_beat(
        &mut pipeline,
        &trail.reader(),
        "stop 1: the app's door — the record names the consumer",
        FocusMove::Next,
    );
    door_beat(
        &mut pipeline,
        &trail.reader(),
        "stop 2: the app's door again — the geometry's answer, still the consumer",
        FocusMove::Down,
    );

    // The keyboard's ride: Tab through the real dispatch, the record
    // naming the keyboard as the rider.
    keyboard_beat(&mut pipeline, &trail.reader());

    // The ring's ride: a plain arrow through the same dispatch, the
    // record naming the ring as the rider.
    ring_beat(&mut pipeline, &trail.reader(), Key::ArrowRight);

    // The scoped door: the trap walked by the app, the rider beside the
    // scope.
    scoped_beat(
        &mut pipeline,
        &trail.reader(),
        "stop 5: the scoped door — the rider beside the scope",
        FocusMove::Next,
        modal,
    );

    // The attributed history, read back in full.
    read_back(&trail);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer focus riders",
        (420.0, 320.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
