//! The adapter's echo rides the causal log (§13.3, ADR-0073): when an
//! assistive technology connects or reconnects, the a11y sink answers
//! with the whole resolved tree — the same builder the deltas use — and
//! since M67 the answer streams one `InspectEvent::A11yEcho` the tick
//! after it resolves, the tree named by its node count (the lengths
//! rule at the a11y tier: the payload never rides the log — a resolved
//! tree carries the app's labels, and a label can carry a secret).
//!
//! The demo routes the echo beat the way the platform's activation
//! route rides it — the erased a11y sink downcast to its concrete
//! `AccessKitSink`, the full-tree build — so the fact is deterministic:
//! on a real desktop the AT's connect fires the same route (the
//! platform answers `InitialTreeRequested` through the same door), and
//! the demo's startup beat beside it names the tree's size the log
//! carried. A reconnect on a real desktop streams a second echo, one
//! fact per build — the log stays quiet between connects (the flood
//! rule at a tier whose events are rare).
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example a11y_echo
//! ```
//!
//! Run with `cargo run -p alacer --example a11y_echo`. The trail label
//! mirrors what the log streamed — the echo's row reads "the AT
//! answered with the full tree (N nodes)" — the observer writes the
//! signal the label binds, the same one-batch currency every handler
//! rides. `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Column, Label};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, InspectObserver, Tee};
use alacer_core::trail::TrailLog;
use reactive_graph::traits::Set as _;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The echo as the trail's reader renders it: the tree the activation
/// answered, named by its node count — the payload never rides.
fn describe(record: alacer_core::A11yEchoRecord) -> String {
    format!(
        "the AT answered with the full tree ({} nodes)",
        record.nodes
    )
}

/// The observer the pipeline owns: the echo's fact mirrored into the
/// trail label's signal — one write batch, the label re-renders the
/// same frame the drain rode (axiom A1: signals in, never widgets).
/// The bounded ring beside it keeps the session's echoes for the
/// closing read-back, the recorder's own discipline (§14.3).
struct EchoTrail {
    echoes:
        std::rc::Rc<std::cell::RefCell<std::collections::VecDeque<alacer_core::A11yEchoRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for EchoTrail {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::A11yEcho(record) = event else {
            return;
        };
        {
            let mut ring = self.echoes.borrow_mut();
            if ring.len() >= 64 {
                ring.pop_front();
            }
            ring.push_back(record);
        }
        self.trail_line.set(describe(record));
    }
}

/// The echo beat the platform's activation route rides: the erased a11y
/// sink downcast to its concrete sink, the full-tree build — the same
/// triple the windowed loop runs on `InitialTreeRequested` (the m-suite
/// helper's shape, the demo's route). The echoed fact queues where the
/// build resolves and streams at the next tick's start drain.
fn route_activation_echo(pipeline: &mut alacer_core::passes::PassPipeline) -> usize {
    let built = pipeline
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<alacer::a11y::AccessKitSink>())
        .and_then(|sink| sink.full_tree_update());
    built.map_or(0, |update| update.nodes.len())
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut alacer_core::passes::PassPipeline) {
    for _ in 0..16 {
        let Ok(report) = pipeline.tick(Size::new(460.0, 260.0)) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The adapter's echo, streamed to the causal log (ADR-0073)").boxed(),
    );

    // The trail line: the log's half of the story (the observer writes
    // it at the drain; the app mirrors its own introspection beside it).
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        trail_signal,
        trail_label,
        &trail_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );

    // The observer rides from here on: the adapter tier's facts stream
    // into the trail label the frame after each build resolves. The
    // slot is single, so the in-process reader rides behind the tee
    // (M64, ADR-0070 — the inspector_demo wiring's shape): both doors
    // see every echo, the arrival order the causal reads rely on.
    let echoes: std::rc::Rc<
        std::cell::RefCell<std::collections::VecDeque<alacer_core::A11yEchoRecord>>,
    > = std::rc::Rc::new(std::cell::RefCell::new(std::collections::VecDeque::new()));
    let reader = TrailLog::new();
    pipeline.set_observer(Box::new(Tee::new(
        Box::new(EchoTrail {
            echoes: std::rc::Rc::clone(&echoes),
            trail_line: trail_signal,
        }),
        Box::new(reader.clone()),
    )));

    // The echo beat (M67, ADR-0073): the activation route rides — the
    // sink's full-tree build — and the echo streams the tick after it
    // resolves. On a real desktop the AT's connect fires the same route;
    // the demo names the size the log carried at startup.
    let nodes = route_activation_echo(&mut pipeline);
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::a11y_echo",
        nodes,
        "the activation's echo streamed: the AT answered with the full tree"
    );

    // The reconnect beat: a second build answers a second echo — one
    // fact per build, the flood rule at the a11y tier, both doors
    // hearing it through the same tee.
    let _ = route_activation_echo(&mut pipeline);
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer a11y echo",
        (460.0, 260.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the echoes read back — every fact the session
    // streamed, oldest first, the tree sizes as data.
    let read_back = echoes.borrow().iter().copied().collect::<Vec<_>>();
    tracing::info!(
        target: "alacer::a11y_echo",
        echoes = read_back.len(),
        nodes = ?read_back.iter().map(|record| record.nodes).collect::<Vec<_>>(),
        reader_drained = reader.drain().len(),
        "the adapter's echoes streamed as they resolved — the causal log carries the a11y trail, the tree named by its size"
    );
}
