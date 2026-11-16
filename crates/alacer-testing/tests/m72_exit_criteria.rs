//! M72 exit-criteria integration tests (§13.3, ADR-0078): the view's
//! patch joins the causal log. The tier that every re-render rides —
//! the view reconciliation whose `PatchReport` answered the imperative
//! caller and vanished — grows its own named fact: when
//! [`alacer_view::ViewRoot::mount`] builds the tree or
//! [`alacer_view::ViewRoot::update`] diffs it, the patch's accounting
//! streams as [`alacer_core::inspect::ViewPatchRecord`] beside the
//! tiers that already price themselves.
//!
//! The record is the trail's own shape (ADR-0063's route-drop shape at
//! the view tier): the four counts computed where the patch resolves —
//! the reconciler's own accounting, named by its lengths (the payload
//! rule's shape: a fact, not a message — the props a patch applied
//! never ride). The door's contract never changes: `update` still
//! answers the `PatchReport` it always answered — the record rides
//! *beside* the report, never instead of it.
//!
//! Every test runs the real pipeline (A4): the reconciler against the
//! real arena, the records through the harness trail door, the tick's
//! start drain streaming them one frame after they resolved.

use alacer_core::flux::Flux;
use alacer_core::geom::Size;
use alacer_core::inspect::{InspectEvent, ViewPatchRecord};
use alacer_core::mask::Change;
use alacer_core::passes::{
    A11yDelta, A11ySink, LayoutEngine, LayoutError, PassPipeline, SceneJournal, TreeSnapshot,
};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::trail::TrailLog;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_view::{ViewNode, ViewRoot, WidgetRegistry};
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

/// A minimal widget: the probe the view suite's registry registers —
/// named, no events, the text prop its whole state.
struct BoxProbe {
    text: String,
}

impl Widget for BoxProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Label
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.text.clone())
    }
}

fn box_create(props: &alacer_view::ElementProps<'_>) -> BoxProbe {
    BoxProbe {
        text: props.text("text").unwrap_or_default().to_string(),
    }
}

fn box_patch(probe: &mut BoxProbe, props: &alacer_view::ElementProps<'_>) -> bool {
    match props.text("text") {
        Some(text) => {
            probe.text = text.to_string();
            true
        }
        None => false,
    }
}

/// The suite's registry: one probe element, the second name proving
/// the element-swap path when a beat needs it.
fn probe_registry() -> WidgetRegistry {
    let mut registry = WidgetRegistry::new();
    registry.register("box", box_create, box_patch);
    registry.register("cell", box_create, box_patch);
    registry
}

/// A no-op layout engine (the suite drives the reconciler and the
/// tick's drains; layout itself never has work here).
struct NullLayout;

impl LayoutEngine for NullLayout {
    fn sync_and_layout(
        &mut self,
        _arena: &mut alacer_core::Arena,
        _root: alacer_core::NodeId,
        _dirty: &FxHashMap<alacer_core::NodeId, Change>,
        _viewport: Size,
    ) -> Result<(), LayoutError> {
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
    layers: Mutex<FxHashMap<alacer_core::NodeId, Vec<Primitive>>>,
}

impl SceneJournal for MemoryJournal {
    fn commit_layer(&mut self, node: alacer_core::NodeId, primitives: Vec<Primitive>) {
        self.layers
            .lock()
            .expect("journal lock")
            .insert(node, primitives);
    }
    fn drop_layer(&mut self, node: alacer_core::NodeId) {
        self.layers.lock().expect("journal lock").remove(&node);
    }
    fn layer(&self, node: alacer_core::NodeId) -> Option<Vec<Primitive>> {
        self.layers
            .lock()
            .expect("journal lock")
            .get(&node)
            .cloned()
    }
    fn snapshot(&self, order: &[alacer_core::NodeId]) -> SceneSnapshot {
        let mut primitives = Vec::new();
        for node in order {
            if let Some(layer) = self.layer(*node) {
                primitives.extend(layer);
            }
        }
        SceneSnapshot { primitives }
    }
    fn retain(&mut self, order: &[alacer_core::NodeId]) {
        let keep: Vec<alacer_core::NodeId> = order.to_vec();
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

fn observed_pipeline() -> (PassPipeline, TrailLog) {
    let mut pipeline = PassPipeline::new(
        Flux::new(),
        Box::new(NullLayout),
        Box::new(MemoryJournal::default()),
        Box::new(NullA11y),
    );
    let trail = TrailLog::new();
    pipeline.set_observer(Box::new(trail.clone()));
    (pipeline, trail)
}

/// The streamed view-patch facts, oldest first (the m-suite's filter
/// shape: the assertion source is the trail's own records, never
/// internals).
fn patch_facts(trail: &TrailLog) -> Vec<ViewPatchRecord> {
    trail
        .look()
        .into_iter()
        .filter_map(|event| match event {
            InspectEvent::ViewPatch(record) => Some(record),
            _ => None,
        })
        .collect()
}

/// Empties the window between beats: each assertion reads the facts
/// the patch under test streamed, nothing earlier.
fn clear_window(trail: &TrailLog) {
    let _ = trail.drain();
}

/// Ticks until no pass has work (the m-suite's settle shape): the
/// patch facts ride the tick's start drain, one frame after the
/// reconciler resolved them.
fn settle(pipeline: &mut PassPipeline) {
    let viewport = Size::new(400.0, 300.0);
    for _ in 0..16 {
        let report = pipeline.tick(viewport).expect("the suite's frames lay out");
        if report.is_idle() {
            return;
        }
    }
}

#[test]
fn m72_the_mount_streams_its_build_fact() {
    let _guard = alacer_core::flux::batch_test_lock();
    let (mut pipeline, trail) = observed_pipeline();

    // The build: one mount, three elements, one fact — the reconciler's
    // opening accounting, named by its lengths (the payload rule's
    // shape: the props the mount applied never ride).
    let view = ViewNode::element("box")
        .prop("text", "root")
        .child(ViewNode::element("box").prop("text", "a"))
        .child(ViewNode::element("box").prop("text", "b"));
    let _root = ViewRoot::mount(&view, &mut pipeline, probe_registry());
    settle(&mut pipeline);

    let facts = patch_facts(&trail);
    assert_eq!(
        facts.len(),
        1,
        "the mount's build is one fact (one fact per build, the log quiet between builds)"
    );
    assert_eq!(
        facts[0].created, 3,
        "the build names the tree it created — three elements"
    );
    assert_eq!(
        facts[0].removed, 0,
        "a build removes nothing — the counts are the fact"
    );
    assert_eq!(facts[0].patched, 0, "a build patches nothing");
    assert_eq!(facts[0].moved, 0, "a build moves nothing");

    // The flood rule at the view tier: the idle frames after the build
    // stream no patch fact.
    clear_window(&trail);
    settle(&mut pipeline);
    settle(&mut pipeline);
    assert!(
        patch_facts(&trail).is_empty(),
        "no re-render, no patch fact — the log is quiet between builds"
    );
}

#[test]
fn m72_the_rerender_patch_streams_its_counts_verbatim() {
    let _guard = alacer_core::flux::batch_test_lock();
    let (mut pipeline, trail) = observed_pipeline();

    let view = ViewNode::element("box").prop("text", "before");
    let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());
    settle(&mut pipeline);
    clear_window(&trail);

    // Beat one: the text prop changes — the patch lands in place, the
    // fact names exactly what moved (the PatchReport the door still
    // answers, beside it).
    let updated = ViewNode::element("box").prop("text", "after");
    let report = root.update(&updated, &mut pipeline);
    assert_eq!(report.patched, 1, "the door's own contract is unchanged");
    settle(&mut pipeline);
    let facts = patch_facts(&trail);
    assert_eq!(facts.len(), 1, "one re-render is one patch fact");
    assert_eq!(facts[0].patched, 1, "the patched count rides verbatim");
    assert_eq!(facts[0].created, 0, "an in-place patch creates nothing");
    assert_eq!(facts[0].removed, 0, "an in-place patch removes nothing");
    assert_eq!(facts[0].moved, 0, "an in-place patch moves nothing");

    // Beat two: a keyed insert and remove — created and removed named
    // beside each other, the LIS's minimal mutation set as data.
    clear_window(&trail);
    let grown = ViewNode::element("box").prop("text", "after").children([
        ViewNode::element("box").key(1_u64).prop("text", "k1"),
        ViewNode::element("box").key(2_u64).prop("text", "k2"),
    ]);
    let report = root.update(&grown, &mut pipeline);
    assert_eq!(report.created, 2, "two keyed children mount");
    settle(&mut pipeline);
    let facts = patch_facts(&trail);
    assert_eq!(facts.len(), 1, "the insert beat is one fact");
    assert_eq!(facts[0].created, 2, "the created count rides verbatim");
    assert_eq!(
        facts[0].patched, 0,
        "the root's unchanged text patched nothing — a no-op prop is no patch"
    );
    assert_eq!(facts[0].removed, 0);

    // Beat three: the keyed rotate — one move, LIS keeping the rest
    // stable (the reconciler's own rule, §6.2).
    clear_window(&trail);
    let rotated = ViewNode::element("box").prop("text", "after").children([
        ViewNode::element("box").key(2_u64).prop("text", "k2"),
        ViewNode::element("box").key(1_u64).prop("text", "k1"),
    ]);
    let report = root.update(&rotated, &mut pipeline);
    assert_eq!(report.moved, 1, "LIS keeps one of two stable");
    settle(&mut pipeline);
    let facts = patch_facts(&trail);
    assert_eq!(facts.len(), 1, "the rotate beat is one fact");
    assert_eq!(facts[0].moved, 1, "the moved count rides verbatim");
    assert_eq!(facts[0].removed, 0, "a rotate removes nothing");
}

#[test]
fn m72_the_noop_rerender_streams_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let (mut pipeline, trail) = observed_pipeline();

    let view = ViewNode::element("box").prop("text", "same");
    let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());
    settle(&mut pipeline);
    clear_window(&trail);

    // The app re-rendered and nothing changed: the reconciler still
    // ran, and the log carries none of it — the idle-frame property,
    // the m32 canon's own rule (a write that lands where the value
    // already is queues no masks, and no facts). The door's report is
    // the caller's window; the trail's silence is the log's.
    let updated = ViewNode::element("box").prop("text", "same");
    let report = root.update(&updated, &mut pipeline);
    assert_eq!(report, alacer_view::PatchReport::default());
    settle(&mut pipeline);
    settle(&mut pipeline);
    assert!(
        patch_facts(&trail).is_empty(),
        "the no-op re-render streams nothing — the idle frame on the log"
    );
}

#[test]
fn m72_the_late_observer_reads_the_patches_from_installation() {
    let _guard = alacer_core::flux::batch_test_lock();
    // The trail's disabled path is the A7 idle contract: facts before
    // the observer installs never queue, and installing mid-session
    // never replays a history that did not stream — the patches from
    // installation on are the ones the reader prices.
    let mut pipeline = PassPipeline::new(
        Flux::new(),
        Box::new(NullLayout),
        Box::new(MemoryJournal::default()),
        Box::new(NullA11y),
    );
    let trail = TrailLog::new();

    let view = ViewNode::element("box").prop("text", "before");
    let mut root = ViewRoot::mount(&view, &mut pipeline, probe_registry());
    let updated = ViewNode::element("box").prop("text", "unobserved");
    let _report = root.update(&updated, &mut pipeline);
    settle(&mut pipeline);
    assert!(
        patch_facts(&trail).is_empty(),
        "no observer installed, no fact — the disabled trail notes nothing"
    );

    pipeline.set_observer(Box::new(trail.clone()));
    let updated = ViewNode::element("box").prop("text", "observed");
    let _report = root.update(&updated, &mut pipeline);
    settle(&mut pipeline);
    let facts = patch_facts(&trail);
    assert_eq!(
        facts.len(),
        1,
        "the installation is the door: the patches from it on stream"
    );
    assert_eq!(facts[0].patched, 1, "the observed patch's own count rides");
}

#[test]
fn m72_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M81",
        "eighty-eight decision records",
        "Since M72 the view's patches answer on the log too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }

    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0078",
        "88 decision records",
        "the view's patch joins the causal log",
        // The M71 cut's own register row rides the same sweep (the
        // ADR-0077 row the register was missing, the m71 pin's phrase).
        "M71 scope: the frame's refusals join the causal log",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    let protocol = read_workspace_file("docs/inspector-protocol.md");
    assert!(
        protocol.contains("`view_patch`"),
        "the protocol document carries the view patch's kind"
    );

    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M72 — the view's patch joins the causal log (ADR-0078)**"),
        "the CHANGELOG carries the M72 entry"
    );

    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("patch_trail"),
        "docs/examples.md carries the view-patch demonstration"
    );

    // The corpus the whole cut publishes: 85 criterion cases — M72's
    // two note-cost gates beside the two `frame_refusal` gates the
    // M71 cut promised (bench, gate, and baseline in the same cut),
    // every pin re-agreed beside M71's.
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    assert_eq!(
        baselines["bench_count"],
        serde_json::json!(86),
        "the published corpus carries the four new cases the two cuts promised"
    );
    for case in [
        "view_patch/note_disabled",
        "view_patch/note_enabled_bounded",
        "frame_refusal/note_disabled",
        "frame_refusal/note_enabled_bounded",
    ] {
        assert!(
            baselines["benches"].get(case).is_some(),
            "the published baseline carries {case} (the coordinated rule, ADR-0011)"
        );
    }

    // The sibling suites' pins ride the same cut (one sweep, no drift):
    // the m71 suite's documents pin moved to M72's cut, and the m5
    // suite's corpus const re-agrees with the published data.
    let m71 = read_workspace_file("crates/alacer-testing/tests/m71_exit_criteria.rs");
    assert!(
        m71.contains("M0 through M81"),
        "the m71 suite's pins moved with the documents"
    );
    let m5 = read_workspace_file("crates/alacer-testing/tests/m5_exit_criteria.rs");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 86;"),
        "the m5 suite's corpus const re-agrees with the published baseline"
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

/// Parses a workspace JSON document (the m5 suite's helper shape).
fn parse_workspace_json(path: &str) -> serde_json::Value {
    let text = read_workspace_file(path);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path} is not valid JSON: {error}"))
}
