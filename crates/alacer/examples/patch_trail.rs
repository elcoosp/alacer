//! The view's patches answer on the causal log (§13.3, ADR-0078): the
//! view reconciliation — the tier every re-render rides — answered a
//! `PatchReport` the imperative caller read and dropped, and "what did
//! my re-render change — and how much?" was unanswerable from the log
//! alone. Since M72 the patch streams one
//! [`alacer_core::inspect::ViewPatchRecord`] the tick after it
//! resolves — the reconciler's four counts beside each other
//! (`created` / `removed` / `patched` / `moved`), named by its
//! lengths, the record riding *beside* the `PatchReport` the door
//! still answers (the payload rule's shape: the props a patch applied
//! never ride, the text they carry can).
//!
//! The demo drives the real reconciler from the consumer side: the
//! *Add row* button mounts one keyed row (the patch fact names
//! `created: 1`), the *Remove row* press unmounts one (`removed: 1`),
//! and a re-render that changes nothing streams nothing at all —
//! the idle-frame property, the m32 canon's own rule: a write that
//! lands where the value already is queues no masks, and no facts
//! (the flood rule at the view tier). The
//! *trail* label mirrors what the log streamed.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example patch_trail
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (named at startup, the
//! `BackendPreference::software_env` door).

use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_core::flux::RenderBinding;
use alacer_core::inspect::{InspectEvent, InspectObserver, ViewPatchRecord};
use alacer_core::passes::PassPipeline;
use alacer_view::{PropValue, ViewNode, ViewRoot};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: alacer_core::geom::Size = alacer_core::geom::Size::new(480.0, 340.0);

/// The demo's state: the keyed rows the view declares (the ids the
/// reconciler's key map routes through the LIS diff, §6.2).
type Rows = Rc<RefCell<Vec<u64>>>;

/// The next row id the *Add row* button mints (the keys' stable
/// identity across re-renders).
fn next_id(rows: &Rows) -> u64 {
    rows.borrow().iter().copied().max().unwrap_or(0) + 1
}

/// The demo's view function: the title, one keyed label per row, the
/// two buttons, and the *trail* label last (the arena scan finds it
/// there). The handlers ride the same `Rc` every generation — a
/// re-hung handler is the same handler, never a visual change (the
/// registry's patch rule) — so the patch facts price only the rows.
fn demo_view(
    rows: &Rows,
    trail_binding: &RenderBinding<String>,
    add: &PropValue,
    remove: &PropValue,
) -> ViewNode {
    let ids = rows.borrow().clone();
    let trail = trail_binding.clone();
    let mut column = ViewNode::element("column").prop("gap", 12.0_f32);
    column = column.child(ViewNode::element("label").prop("text", "The view-patch trail"));
    for id in ids {
        column = column.child(
            ViewNode::element("label")
                .key(id)
                .prop("text", format!("row {id}")),
        );
    }
    column = column
        .child(
            ViewNode::element("button")
                .prop("text", "Add row")
                .prop("on_click", add.clone()),
        )
        .child(
            ViewNode::element("button")
                .prop("text", "Remove row")
                .prop("on_click", remove.clone()),
        )
        .child(ViewNode::element("label").prop(
            "text",
            "Live: add and remove rows — the trail mirrors the log",
        ))
        .child(
            ViewNode::element("label")
                .prop("binding", trail)
                .prop("text", "the trail: (nothing streamed yet)"),
        );
    column
}

/// The fact as the trail's reader renders it: the four counts beside
/// each other — the log answering "what did my re-render change?" from
/// the row alone. The record is `Copy`, so it passes by value (the
/// pedantic group's rule).
fn describe(record: ViewPatchRecord) -> String {
    let ViewPatchRecord {
        created,
        removed,
        patched,
        moved,
    } = record;
    format!("created {created}, removed {removed}, patched {patched}, moved {moved}")
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    patches: Rc<RefCell<std::collections::VecDeque<ViewPatchRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<ViewPatchRecord> {
        self.patches.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    patches: Rc<RefCell<std::collections::VecDeque<ViewPatchRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::ViewPatch(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.patches.borrow_mut();
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
        let report = pipeline.tick(VIEWPORT).expect("the demo's frames lay out");
        if report.is_idle() {
            return;
        }
    }
}

/// Re-renders the demo's view through the consumer's own door: the
/// reconciler diffs the new tree, the `PatchReport` comes back, and
/// the patch fact queues where the diff resolved (the trail streams
/// it at the next tick's start).
fn re_render(
    root: &RefCell<ViewRoot>,
    rows: &Rows,
    pipeline: &mut PassPipeline,
    trail_binding: &RenderBinding<String>,
    add: &PropValue,
    remove: &PropValue,
) {
    let view = demo_view(rows, trail_binding, add, remove);
    let report = root.borrow_mut().update(&view, pipeline);
    let _ = report;
}

/// The demo's beats, walked before the window opens: the mount's build
/// fact (streamed the tick after the mount), the add's `created`, the
/// no-op re-render's silence, the remove's `removed` — one fact per
/// patch that moved something, never per frame.
fn run_beats(
    pipeline: &mut PassPipeline,
    root: &RefCell<ViewRoot>,
    rows: &Rows,
    trail: &Trail,
    trail_binding: &RenderBinding<String>,
    add: &PropValue,
    remove: &PropValue,
) {
    settle(pipeline);

    // Beat one: the add — one keyed row mounts, the fact names
    // `created: 1` (the LIS kept every existing row stable).
    rows.borrow_mut().push(next_id(rows));
    re_render(root, rows, pipeline, trail_binding, add, remove);
    settle(pipeline);

    // Beat two: the same view again — the reconciler still ran, and
    // the log carries none of it: the idle-frame property (a patch
    // that moved nothing is not a fact).
    re_render(root, rows, pipeline, trail_binding, add, remove);
    settle(pipeline);

    // Beat three: the remove — one keyed row unmounts, the fact names
    // `removed: 1`.
    rows.borrow_mut().pop();
    re_render(root, rows, pipeline, trail_binding, add, remove);
    settle(pipeline);

    tracing::info!(
        target: "alacer::patch_trail",
        facts = trail.all().len(),
        "the beats streamed: build, created, silence, removed — one fact per patch that moved something"
    );
}

fn main() {
    let mut pipeline = alacer::pipeline_default();
    let rows: Rows = Rc::new(RefCell::new(vec![1, 2]));

    // The trail label's text rides a bound label (the registry's
    // binding prop): the observer's signal writes re-render it the
    // same frame the drain rode.
    let trail_signal = pipeline
        .flux()
        .signal("the trail: (nothing streamed yet)".to_string());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());

    // The two buttons: plain handlers that mutate the demo's state and
    // wake the app loop — the patch hook owns the re-render. Built
    // once, reused every generation (a re-hung handler is the same
    // handler, never a visual change).
    let channel = PatchChannel::new();
    let add_state = Rc::clone(&rows);
    let add_fire = channel.clone();
    let add_handler = PropValue::Handler(Rc::new(RefCell::new(move || {
        let id = next_id(&add_state);
        add_state.borrow_mut().push(id);
        add_fire.fire();
    })));
    let remove_state = Rc::clone(&rows);
    let remove_fire = channel.clone();
    let remove_handler = PropValue::Handler(Rc::new(RefCell::new(move || {
        remove_state.borrow_mut().pop();
        remove_fire.fire();
    })));

    // The mount: the build fact queues here, the root rides in a cell
    // the beats and the patch hook re-render through.
    let view = demo_view(&rows, &trail_binding, &add_handler, &remove_handler);
    let root = RefCell::new(alacer::mount_view(&view, &mut pipeline));

    // The observer rides from here on: every patch's record streams
    // into the trail label the frame after the patch resolved.
    let trail = Trail {
        patches: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(TrailWriter {
        patches: Rc::clone(&trail.patches),
        trail_line: trail.trail_line,
    }));

    // The trail label's binding bridge: the signal's writes land as
    // paint on the last label the view mounted.
    let trail_node = {
        let arena = pipeline.arena();
        let mut labels = Vec::new();
        arena.for_each_pre_order(&mut |id, slot| {
            if slot.widget().role() == alacer_core::Role::Label {
                labels.push(id);
            }
        });
        *labels.last().expect("the view mounted the trail label")
    };
    pipeline.flux_mut().bind_value(
        trail.trail_line,
        trail_node,
        &trail_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );

    run_beats(
        &mut pipeline,
        &root,
        &rows,
        &trail,
        &trail_binding,
        &add_handler,
        &remove_handler,
    );

    // The window: live play — the buttons mutate the state and wake the
    // loop, the patch hook re-renders through the consumer's door, and
    // the trail label mirrors every streamed fact the whole while.
    let hook_rows = Rc::clone(&rows);
    let hook_binding = trail_binding.clone();
    let mut app = AlacerApp::new(
        pipeline,
        "Alacer view-patch trail",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = demo_view(&hook_rows, &hook_binding, &add_handler, &remove_handler);
        let _report = root.borrow_mut().update(&view, pipeline);
    }));

    if let Err(error) = app.with_patch_channel(channel).run() {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the view tier's history as data.
    for (index, record) in trail.all().iter().enumerate() {
        tracing::info!(
            target: "alacer::patch_trail",
            index,
            created = record.created,
            removed = record.removed,
            patched = record.patched,
            moved = record.moved,
            "{}", describe(*record)
        );
    }
}
