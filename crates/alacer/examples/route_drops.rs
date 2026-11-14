//! The boundary's drops join the causal log (§13.3, ADR-0063):
//! ADR-0061's third kill rule cashes — a consumer needs the platform's
//! defensive drops on the log — the asks refused *before* any widget's
//! door stream their own fact now ([`alacer_core::RouteDropRecord`]):
//! the unknown action, the payloadless write, the off-target selection,
//! and the unknown target. A drop is never a munged widget-door fact:
//! the record is the route's own vocabulary, the action named by its
//! stable tag, the species naming why the boundary refused, the aimed
//! id riding verbatim.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example route_drops
//! ```
//!
//! The beats: an ask aimed at an id no tree node carries (the funnel's
//! own defensive drop, refused with the consumed verdict the m10
//! contract pins), a delivered ask beside it (a delivered ask streams
//! no drop — the honest absence). The platform boundary's species ride
//! the same vocabulary: a `SetValue` without a payload, a
//! `SetTextSelection` aimed at another node, an action no widget
//! contract answers — each names its fact where the boundary refuses
//! (pinned by `alacer-platform`'s own unit tests through the real
//! mapping). Then the window opens for live play.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label, TextInput};
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::{InspectEvent, InspectObserver, RouteDropSpecies};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yAction, Role};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: alacer_core::geom::Size = alacer_core::geom::Size::new(480.0, 300.0);

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the trail's reader renders it: the dropped action's tag
/// beside the species and the aimed id — "the AT asked; why did no door
/// ever answer" answered from one record.
fn describe(record: alacer_core::RouteDropRecord) -> String {
    let aim = record
        .target
        .map(|target| format!(" (aimed at {target})"))
        .unwrap_or_default();
    format!("dropped: {}{aim}", record.species.name())
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    drops: Rc<RefCell<std::collections::VecDeque<alacer_core::RouteDropRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The observer half: boxed into
    /// [`alacer_core::passes::PassPipeline::set_observer`], it receives
    /// every `InspectEvent`, keeps the route-drop facts, and mirrors
    /// the newest into the trail label's signal.
    fn writer(&self) -> TrailWriter {
        TrailWriter {
            drops: Rc::clone(&self.drops),
            trail_line: self.trail_line,
        }
    }

    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<alacer_core::RouteDropRecord> {
        self.drops.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    drops: Rc<RefCell<std::collections::VecDeque<alacer_core::RouteDropRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::RouteDrop(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.drops.borrow_mut();
            if ring.len() >= 64 {
                ring.pop_front();
            }
            ring.push_back(record);
        }
        self.trail_line.set(line);
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let Ok(report) = pipeline.tick(VIEWPORT) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// One beat on the funnel: the ask routed the real way — a consumed ask
/// answers `true` and streams no drop, an ask no tree node carries
/// refuses `false` and streams the drop's fact.
fn ask_beat(pipeline: &mut PassPipeline, label: &'static str, target: u64, action: A11yAction) {
    let _route =
        tracing::trace_span!(target: "alacer::route_drops", "beat", label = label).entered();
    let consumed = pipeline.dispatch_a11y_action(target, action);
    settle(pipeline);
    tracing::info!(target: "alacer::route_drops", consumed, "{label}");
}

/// The input's AccessKit-stable id, read off the real a11y snapshot —
/// the live address the honest-absence beat routes against.
fn input_id(pipeline: &PassPipeline, name: &str) -> u64 {
    let tree = pipeline
        .a11y_snapshot()
        .expect("the a11y snapshot after the settle");
    let node = tree
        .find(Role::TextInput, name)
        .unwrap_or_else(|| panic!("the AT can reach the input named `{name}`"));
    node.id
}

/// The beats the demo walks before the window opens: the unknown target
/// named on the log, the delivered ask beside it riding no drop, and a
/// second refusal at a different aim — the facts attributable.
fn run_beats(pipeline: &mut PassPipeline, status_signal: alacer_core::FluxRwSignal<String>) {
    // Beat one: an ask aimed at an id no tree node carries — the
    // funnel's own defensive drop, the verdict `false` the m10 contract
    // pins, the fact streaming now.
    ask_beat(
        pipeline,
        "beat 1: SetValue at an unknown target — the funnel's drop names its fact",
        0xDEAD_BEEF,
        A11yAction::SetValue("typed".into()),
    );
    status_signal.set("the boundary's drops: the asks no door ever answered".to_string());

    // Beat two: the same ask at the live input — the mapping lands, the
    // widget's door answers, and no drop streams (a delivered ask is
    // not a drop — the honest absence).
    let id = input_id(pipeline, "field");
    ask_beat(
        pipeline,
        "beat 2: SetValue at the live input — the ask lands, no drop rides",
        id,
        A11yAction::SetValue("written by the reader".into()),
    );

    // Beat three: another aim that exists nowhere — the second drop's
    // aim rides verbatim, the log answering "which ask was dropped"
    // beside "why".
    ask_beat(
        pipeline,
        "beat 3: a second unknown target — the aims stay tellable apart",
        0xCAFE_F00D,
        A11yAction::SetTextSelection(alacer_core::widget::TextSelection::new(0, 4)),
    );

    // Beat four: the IME's committed composition no field held (M59,
    // ADR-0065) — the route's other silence, the event named by its
    // stable tag, the aim riding its honest absence (the drop *is*
    // the missing field).
    ime_beat(
        pipeline,
        "beat 4: an IME commit with no focused field — the route's silence names its drop",
    );
}

/// One IME beat routed the real way with no field focused: the
/// committed composition drops before any widget's door, its fact
/// streaming from the route-drop trail.
fn ime_beat(pipeline: &mut PassPipeline, label: &'static str) {
    let _route =
        tracing::trace_span!(target: "alacer::route_drops", "beat", label = label).entered();
    let consumed =
        pipeline.dispatch_ime(&alacer_core::events::ImeEvent::Commit { text: "よ".into() });
    settle(pipeline);
    tracing::info!(target: "alacer::route_drops", ?consumed, "{label}");
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The boundary's drops join the causal log (ADR-0063)").boxed(),
    );
    let mut field = TextInput::new("field");
    field.set_value("hello trail");
    let _ = pipeline.add_child(root, field.boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("Live: an ask no door answers names its drop on the log").boxed(),
    );

    let status_signal = pipeline
        .flux_mut()
        .signal("the boundary: (the tree answers what it can)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let status_binding =
        RenderBinding::new("the boundary: (the tree answers what it can)".to_string());
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        trail_signal,
        trail_label,
        &trail_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );

    // The observer rides from here on: every route drop's record
    // streams into the trail label the frame after the boundary refuses.
    let trail = Trail {
        drops: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(trail.writer()));

    settle(&mut pipeline);
    run_beats(&mut pipeline, status_signal);
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer route drops",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the species beside every drop.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::route_drops",
        records = read_back.len(),
        unknown_targets = read_back
            .iter()
            .filter(|record| record.species == RouteDropSpecies::UnknownTarget)
            .count(),
        "the drops streamed as they resolved — the causal log carries the boundary's refusals"
    );
}
