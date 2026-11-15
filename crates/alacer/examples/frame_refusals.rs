//! The frame's refusals answer on the causal log (§13.3, ADR-0077):
//! the frame itself — the tier every other tier's work rides — answered
//! a `tracing::error!` line and a skipped frame when it went unpainted,
//! and "why did nothing paint?" was unanswerable from the log alone.
//! Since M71 the refusal streams one `InspectEvent::FrameRefusal` the
//! tick after it resolves — the species naming the stage that refused
//! (`layout_refused` / `render_failed` / `partial_repaint_failed` /
//! `framebuffer_lost` / `present_failed`), the failing node beside it
//! when a widget is the fact, the record riding *beside* the refusal's
//! own door: the tick still answers `Err(LayoutError)`, the ladder
//! still skips the frame it cannot present.
//!
//! The demo drives the real frame tier from the consumer side: a
//! *Break layout* button arms a refusing layout engine behind the real
//! pipeline (the injected seam's own contract, ADR-0023's error shape —
//! the engine names the failing node), the next tick answers its `Err`
//! unchanged while the frame-refusal trail queues the record, and the
//! *Fix layout* press recovers — the recovered frames stream no refusal
//! fact (one fact per refusal, never per frame). The *trail* label
//! mirrors what the log streamed.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example frame_refusals
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (named at startup, the
//! `BackendPreference::software_env` door).

use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{FrameRefusalRecord, InspectEvent, InspectObserver};
use alacer_core::passes::{LayoutEngine, LayoutError, PassPipeline};
use alacer_layout::LayoutHost;
use alacer_text::ThreadLocalMeasurer;
use reactive_graph::traits::Set as _;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// The injected seam with a consumer's refusal gate: the real Taffy
/// host underneath (the pipeline's own layout, untouched while the
/// gate is open), the gate's honest `LayoutError` when armed — the
/// error shape ADR-0023 names, the failing node riding verbatim.
struct GateLayout {
    inner: LayoutHost,
    armed: Rc<Cell<bool>>,
}

impl LayoutEngine for GateLayout {
    fn sync_and_layout(
        &mut self,
        arena: &mut alacer_core::Arena,
        root: alacer_core::NodeId,
        dirty: &rustc_hash::FxHashMap<alacer_core::NodeId, alacer_core::Change>,
        viewport: Size,
    ) -> Result<(), LayoutError> {
        if self.armed.get() {
            return Err(LayoutError {
                message: "the demo's gate refused the frame — break the layout to recover"
                    .to_string(),
                node: Some(root),
            });
        }
        self.inner.sync_and_layout(arena, root, dirty, viewport)
    }

    fn measure_calls(&self) -> u64 {
        self.inner.measure_calls()
    }

    fn reset_measure_calls(&mut self) {
        self.inner.reset_measure_calls();
    }
}

/// The fact as the trail's reader renders it: the species beside the
/// failing node when one is the fact — the log answering "why did
/// nothing paint?" from the row alone. The record is `Copy`, so it
/// passes by value (the pedantic group's rule).
fn describe(record: FrameRefusalRecord) -> String {
    let node = record
        .node
        .map_or_else(|| "the window's own".to_string(), |id| format!("node {id}"));
    format!("the frame's {} — {node}", record.species.name())
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    refusals: Rc<RefCell<std::collections::VecDeque<FrameRefusalRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<FrameRefusalRecord> {
        self.refusals.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    refusals: Rc<RefCell<std::collections::VecDeque<FrameRefusalRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::FrameRefusal(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.refusals.borrow_mut();
            if ring.len() >= 64 {
                ring.pop_front();
            }
            ring.push_back(record);
        }
        self.trail_line.set(line);
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
/// A refused frame returns: the tick answered its `Err` — the demo
/// never ticks through a refusal.
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

/// The demo's beats, walked before the window opens: the gate arms, the
/// tick answers its `Err` unchanged while the record queues, the gate
/// disarms, the recovered frame drains the record — one fact per
/// refused frame (the flood rule), the recovered frames streaming none.
fn run_beats(
    pipeline: &mut PassPipeline,
    armed: &Rc<Cell<bool>>,
    churn_signal: alacer_core::FluxRwSignal<i32>,
    trail: &Trail,
) {
    settle(pipeline);

    // Beat one: the gate arms, the churn write dirties the title, the
    // tick's layout stage refuses — the door's `Err` unchanged, the
    // record queued one frame behind the refusal.
    armed.set(true);
    churn_signal.set(1);
    let error = pipeline
        .tick(VIEWPORT)
        .expect_err("the armed gate refuses the frame");
    tracing::info!(
        target: "alacer::frame_refusals",
        error = %error,
        "beat 1: the tick answered its Err — the door's contract unchanged"
    );

    // Beat two: the gate disarms, the frame recovers, and the drain
    // streams the refusal's record — one fact, the species beside the
    // failing node (ADR-0023's vocabulary, verbatim).
    armed.set(false);
    settle(pipeline);
    tracing::info!(
        target: "alacer::frame_refusals",
        facts = trail.all().len(),
        "beat 2: the recovered frame streamed the refusal's record"
    );
}

fn main() {
    // The pipeline assembled like `pipeline_default`, the layout stage
    // behind the demo's gate (the injected seam is the consumer's own
    // composition site — the journal and the a11y sink are the same
    // production pieces the facade installs).
    let armed = Rc::new(Cell::new(false));
    alacer_core::text_seam::install(Rc::new(alacer_text::ThreadLocalTextSeam));
    let clipboard = Rc::new(alacer_platform::clipboard::ArboardClipboard::new());
    alacer_core::clipboard::install(clipboard);
    let mut pipeline = PassPipeline::new(
        alacer_core::Flux::new(),
        Box::new(GateLayout {
            inner: LayoutHost::new(Arc::new(ThreadLocalMeasurer)),
            armed: Rc::clone(&armed),
        }),
        Box::new(alacer_render::SceneJournalImpl::new()),
        Box::new(alacer_a11y::AccessKitSink::new()),
    );

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let title_node = pipeline.add_child(root, Label::new("The frame refusal trail").boxed());
    let _ = pipeline.add_child(
        root,
        {
            let armed = Rc::clone(&armed);
            Button::new("Break layout").on_click(move || {
                armed.set(true);
            })
        }
        .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        {
            let armed = Rc::clone(&armed);
            Button::new("Fix layout").on_click(move || {
                armed.set(false);
            })
        }
        .boxed(),
    );

    // The dirt lever: the break button's handler writes this signal,
    // the bridge lands `LAYOUT | PAINT` on the title, and the next
    // tick's layout stage runs — into the gate's arms while armed.
    let churn_signal = pipeline.flux_mut().signal(0_i32);
    pipeline
        .flux_mut()
        .bind(churn_signal, title_node, |_v: &i32| {
            alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT)
        });

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

    // The observer rides from here on: every frame refusal's record
    // streams into the trail label the frame after the refusal
    // resolved.
    let trail = Trail {
        refusals: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(TrailWriter {
        refusals: Rc::clone(&trail.refusals),
        trail_line: trail.trail_line,
    }));

    run_beats(&mut pipeline, &armed, churn_signal, &trail);

    // The window: live play — *Break layout* arms the gate through the
    // real handler, the windowed loop skips the frame it cannot lay
    // out (the platform logs the refusal, the record queues), and *Fix
    // layout* recovers it — the trail label mirroring every streamed
    // fact the whole while.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: break the layout — the trail mirrors the log").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer frame refusals",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the frame tier's history as data.
    for (index, record) in trail.all().iter().enumerate() {
        tracing::info!(
            target: "alacer::frame_refusals",
            index,
            species = record.species.name(),
            node = record.node,
            "{}", describe(*record)
        );
    }
}
