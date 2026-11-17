//! M71 exit-criteria integration tests (§13.3, ADR-0077): the frame's
//! refusals join the causal log. The one tier that never answered on
//! the log — the frame itself — grows its own named fact: when the
//! tick's layout stage refuses ([`alacer_core::passes::LayoutError`],
//! the engine seam's own contract or the no-root wiring bug), the
//! refusal streams as [`alacer_core::inspect::FrameRefusalRecord`]
//! beside the tiers that already price themselves, and the present
//! path's failures name their species the same way.
//!
//! The record is the trail's own shape (ADR-0063's route-drop shape at
//! the frame tier): the species computed where the refusal resolves —
//! the tick's layout block, the backends' present ladder — the failing
//! node's id riding verbatim when the engine named one, the honest
//! absence when the frame had no node to name. The door's contract
//! never changes: the tick still answers `Err(LayoutError)` and the
//! platform still skips the frame it cannot lay out — the record rides
//! *beside* the refusal, never instead of it.
//!
//! Every test runs the real pipeline (A4): the layout refusal through
//! the real pass system, the records through the harness trail door.

use alacer_core::arena::NodeId;
use alacer_core::flux::Flux;
use alacer_core::geom::Size;
use alacer_core::inspect::{FrameRefusalRecord, InspectEvent};
use alacer_core::mask::Change;
use alacer_core::passes::{
    A11yDelta, A11ySink, LayoutEngine, LayoutError, PassPipeline, SceneJournal, TreeSnapshot,
};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use reactive_graph::traits::Set as _;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// A minimal widget: fixed-size, named, no events — the m-suite's
/// geometry stub.
struct Fixed {
    size: Size,
}

impl Widget for Fixed {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Container
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("fixed")
    }
    fn measure_spec(&self) -> Option<alacer_core::style::MeasureSpec> {
        Some(alacer_core::style::MeasureSpec::Fixed(self.size))
    }
}

/// A layout engine that lays out like the m-suite's `StackLayout` but
/// refuses while its gate is armed — the injected seam's own contract
/// (ADR-0023's error shape): the engine names the failing node, the
/// tick propagates the error, the frame goes unpainted. The suite
/// holds a clone of the gate, so a beat arms and disarms the refusal
/// without reaching into the pipeline.
struct GateLayout {
    fail: Arc<AtomicBool>,
    root: Mutex<Option<NodeId>>,
}

impl GateLayout {
    fn gated() -> (Self, Arc<AtomicBool>) {
        let fail = Arc::new(AtomicBool::new(false));
        (
            Self {
                fail: Arc::clone(&fail),
                root: Mutex::new(None),
            },
            fail,
        )
    }
}

/// One subtree laid out top-down: fixed-size widgets take their size
/// clamped to the viewport, everything else fills the width at zero
/// height — the gate's non-failing path, hoisted to the module so the
/// method's body stays statements (the pedantic gate's rule).
fn layout_subtree(
    arena: &mut alacer_core::arena::Arena,
    node: NodeId,
    origin: alacer_core::geom::Point,
    viewport: Size,
) {
    let size = arena
        .widget(node)
        .measure_spec()
        .and_then(|spec| match spec {
            alacer_core::style::MeasureSpec::Fixed(size) => Some(size),
            alacer_core::style::MeasureSpec::Text(_) => None,
        })
        .map_or(Size::new(viewport.width, 0.0), |s| {
            Size::new(s.width.min(viewport.width), s.height)
        });
    arena.update_bounds(node, alacer_core::geom::Rect::new(origin, size));
    let mut y = origin.y;
    for child in arena.children(node).to_vec() {
        layout_subtree(
            arena,
            child,
            alacer_core::geom::Point::new(origin.x, y),
            viewport,
        );
        y = arena.bounds(child).map_or(y, |b| b.max_y());
    }
}

impl LayoutEngine for GateLayout {
    fn sync_and_layout(
        &mut self,
        arena: &mut alacer_core::arena::Arena,
        root: NodeId,
        _dirty: &FxHashMap<NodeId, Change>,
        viewport: Size,
    ) -> Result<(), LayoutError> {
        *self.root.lock().expect("gate root lock") = Some(root);
        if self.fail.load(Ordering::SeqCst) {
            return Err(LayoutError {
                message: "the gate refused the frame — the M71 beat's honest refusal".to_string(),
                node: Some(root),
            });
        }
        layout_subtree(arena, root, alacer_core::geom::Point::zero(), viewport);
        Ok(())
    }

    fn measure_calls(&self) -> u64 {
        0
    }

    fn reset_measure_calls(&mut self) {}
}

/// A journal that stores layers in memory — the core tests' shape.
#[derive(Default)]
struct MemoryJournal {
    layers: Mutex<FxHashMap<NodeId, Vec<Primitive>>>,
}

impl SceneJournal for MemoryJournal {
    fn commit_layer(&mut self, node: NodeId, primitives: Vec<Primitive>) {
        self.layers
            .lock()
            .expect("journal lock")
            .insert(node, primitives);
    }
    fn drop_layer(&mut self, node: NodeId) {
        self.layers.lock().expect("journal lock").remove(&node);
    }
    fn layer(&self, node: NodeId) -> Option<Vec<Primitive>> {
        self.layers
            .lock()
            .expect("journal lock")
            .get(&node)
            .cloned()
    }
    fn snapshot(&self, order: &[NodeId]) -> SceneSnapshot {
        let mut primitives = Vec::new();
        for node in order {
            if let Some(layer) = self.layer(*node) {
                primitives.extend(layer);
            }
        }
        SceneSnapshot { primitives }
    }
    fn retain(&mut self, order: &[NodeId]) {
        let keep: Vec<NodeId> = order.to_vec();
        self.layers
            .lock()
            .expect("journal lock")
            .retain(|node, _| keep.contains(node));
    }
    fn clear(&mut self) {
        self.layers.lock().expect("journal lock").clear();
    }
    fn stats(&self) -> alacer_core::passes::JournalStats {
        alacer_core::passes::JournalStats::default()
    }
}

/// An a11y sink that receives and drops — the harness queries nothing.
#[derive(Default)]
struct NullA11y;

impl A11ySink for NullA11y {
    fn apply(&mut self, _snapshot: Arc<TreeSnapshot>, _delta: &A11yDelta) {}
}

fn gated_pipeline() -> (PassPipeline, Arc<AtomicBool>) {
    let (layout, gate) = GateLayout::gated();
    let pipeline = PassPipeline::new(
        Flux::new(),
        Box::new(layout),
        Box::new(MemoryJournal::default()),
        Box::new(NullA11y),
    );
    (pipeline, gate)
}

/// The streamed frame-refusal facts, oldest first (the m-suite's
/// filter shape: the assertion source is the trail's own records,
/// never internals).
fn refusal_facts(trail: &TrailLog) -> Vec<FrameRefusalRecord> {
    trail
        .look()
        .into_iter()
        .filter_map(|event| match event {
            InspectEvent::FrameRefusal(record) => Some(record),
            _ => None,
        })
        .collect()
}

/// Empties the window between beats: each assertion reads the facts
/// the refusal under test streamed, nothing earlier.
fn clear_window(trail: &TrailLog) {
    let _ = trail.drain();
}

#[test]
fn m71_the_layout_refusal_streams_its_fact() {
    let _guard = alacer_core::flux::batch_test_lock();
    let (mut pipeline, gate) = gated_pipeline();
    let trail = TrailLog::new();
    pipeline.set_observer(Box::new(trail.clone()));

    let viewport = Size::new(400.0, 300.0);
    let root = pipeline.mount_root(Box::new(Fixed {
        size: Size::new(200.0, 400.0),
    }));
    let child = pipeline.add_child(
        root,
        Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }),
    );
    pipeline.tick(viewport).expect("the first frame lays out");

    // The gate arms; the mask re-dirties the child; the tick's layout
    // stage refuses — the door's contract unchanged: the caller reads
    // the `LayoutError`, the frame goes unpainted.
    clear_window(&trail);
    gate.store(true, Ordering::SeqCst);
    let signal = pipeline.flux().signal(0_i32);
    pipeline.flux_mut().bind(signal, child, |_v: &i32| {
        Change::LAYOUT.union(Change::PAINT)
    });
    pipeline.flux().batch(|| signal.set(1_i32));
    let error = pipeline
        .tick(viewport)
        .expect_err("the gated frame refuses");
    assert!(
        error.message.contains("M71 beat"),
        "the refusal's own prose rides the error: {error}"
    );
    assert_eq!(
        error.node,
        Some(root),
        "the engine named the root it failed on"
    );

    // The refusal's fact queued behind the failed frame; the next tick
    // drains it — the record beside the tiers that already stream.
    let report = pipeline
        .tick(viewport)
        .expect("the disarmed frame lays out");
    let _ = report;
    let facts = refusal_facts(&trail);
    assert_eq!(facts.len(), 1, "the refusal is one fact");
    assert_eq!(
        facts[0].species,
        alacer_core::inspect::FrameRefusalSpecies::LayoutRefused,
        "the species names the stage that refused"
    );
    assert_eq!(
        facts[0].node,
        Some(root.raw()),
        "the failing node's id rides verbatim (ADR-0023's vocabulary)"
    );
}

#[test]
fn m71_one_fact_per_refusal_the_recovered_frames_stream_none() {
    let _guard = alacer_core::flux::batch_test_lock();
    let (mut pipeline, gate) = gated_pipeline();
    let trail = TrailLog::new();
    pipeline.set_observer(Box::new(trail.clone()));

    let viewport = Size::new(400.0, 300.0);
    let root = pipeline.mount_root(Box::new(Fixed {
        size: Size::new(200.0, 400.0),
    }));
    let child = pipeline.add_child(
        root,
        Box::new(Fixed {
            size: Size::new(100.0, 30.0),
        }),
    );
    pipeline.tick(viewport).expect("the first frame lays out");

    // One refused frame: the gate arms once, one mask, one `Err`, one
    // fact. The record rides one frame behind the refusal (the tick's
    // start drain), and the recovered frames — the disarmed relayout,
    // the idle frame after it — stream no refusal fact: one fact per
    // refusal, never per frame (the flood rule at the frame tier).
    clear_window(&trail);
    gate.store(true, Ordering::SeqCst);
    let signal = pipeline.flux().signal(0_i32);
    pipeline.flux_mut().bind(signal, child, |_v: &i32| {
        Change::LAYOUT.union(Change::PAINT)
    });
    pipeline.flux().batch(|| signal.set(1_i32));
    pipeline
        .tick(viewport)
        .expect_err("the gated frame refuses");
    gate.store(false, Ordering::SeqCst);
    pipeline
        .tick(viewport)
        .expect("the recovered frame lays out");
    pipeline
        .tick(viewport)
        .expect("the idle frame paints nothing");

    let facts = refusal_facts(&trail);
    assert_eq!(facts.len(), 1, "one refused frame is one fact");
    assert_eq!(
        facts[0].species,
        alacer_core::inspect::FrameRefusalSpecies::LayoutRefused,
        "the species names the stage that refused"
    );
    assert_eq!(
        facts[0].node,
        Some(root.raw()),
        "the failing node's id rides verbatim (ADR-0023's vocabulary)"
    );
}

#[test]
fn m71_the_frame_refusals_carry_their_stable_tags() {
    // The wire vocabulary's `snake_case` rule: the names the protocol
    // document pins, one per species, never a prose re-statement.
    use alacer_core::inspect::FrameRefusalSpecies;
    assert_eq!(FrameRefusalSpecies::LayoutRefused.name(), "layout_refused");
    assert_eq!(FrameRefusalSpecies::RenderFailed.name(), "render_failed");
    assert_eq!(
        FrameRefusalSpecies::PartialRepaintFailed.name(),
        "partial_repaint_failed"
    );
    assert_eq!(
        FrameRefusalSpecies::FramebufferLost.name(),
        "framebuffer_lost"
    );
    assert_eq!(FrameRefusalSpecies::PresentFailed.name(), "present_failed");
}

#[test]
fn m71_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M100",
        "a hundred and five decision records",
        "Since M71 the frame's refusals answer on the log too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }

    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0077",
        "a hundred and five decision records",
        "the frame's refusals join the causal log",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`frame_refusal`"),
        "the protocol document carries the frame refusal's kind"
    );

    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M71 — the frame's refusals join the causal log (ADR-0077)**"),
        "the CHANGELOG carries the M71 entry"
    );

    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("frame_refusals"),
        "docs/examples.md carries the frame refusal demonstration"
    );

    // The sibling suites' pins ride the same cut (one sweep, no drift).
    let m70 = read_workspace_file("crates/alacer-testing/tests/m70_exit_criteria.rs");
    assert!(
        m70.contains("M0 through M100"),
        "the m70 suite's pins moved with the documents"
    );
}

/// Reads a workspace file relative to the manifest the suite runs from
/// (the m-suite's own helper shape: `CARGO_MANIFEST_DIR` walks up to
/// the workspace root).
fn read_workspace_file(path: &str) -> String {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir");
    let workspace = std::path::Path::new(&manifest)
        .ancestors()
        .nth(2)
        .expect("the suite rides two directories under the workspace root");
    std::fs::read_to_string(workspace.join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}
