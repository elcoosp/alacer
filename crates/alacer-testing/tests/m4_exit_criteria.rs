//! The M4 exit criteria, as executable spec (architecture §18, ADR-0010):
//!
//! - **The hot-patch contract** — a patched generation renders its *new*
//!   values with the *old* state intact; node identity is stable (the
//!   id-path contract, §6.3); a patch costs exactly one layout+paint cycle
//!   and leaves the tree idle after (ADR-6's contract, unchanged); patch
//!   churn with interleaved mutations stays consistent; untouched subtrees
//!   do not repaint (the scoped re-render, §13.2);
//! - **The Causality inspector** — the §13.3 questions answered from
//!   recorded truth through the real pipeline: edge activations carry the
//!   masks they queued ("why did this render"), provenance writes are
//!   logged ("who wrote this signal"), the edge table answers "who
//!   subscribes", the frame summary carries the pass ledger ("why is this
//!   frame slow") and the mask heatmap ("what did this mask touch").
//!
//! No baselines here: every comparison is dynamic, nothing to rot.

use alacer_core::arena::NodeId;
use alacer_core::flux::RenderBinding;
use alacer_core::widget::Role;
use alacer_inspect::CausalRecorder;
use alacer_testing::TestHarness;
use alacer_view::{HotView, PatchReport, PropValue, ViewNode};
use alacer_widgets::default_registry;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// The viewport every harness in this suite runs at.
const VIEWPORT: (f32, f32) = (320.0, 240.0);

/// The canon app as a view function of the generation: a generation marker
/// label, the bound count label, and the increment button. Generation 0 is
/// the shipped UI; anything else is "patched" and grows an extra label —
/// the shape a binary patch installs.
fn hot_app_view(
    binding: &RenderBinding<String>,
    generation: u32,
    on_click: &Rc<dyn Fn()>,
) -> ViewNode {
    let binding = binding.clone();
    let marker = if generation == 0 {
        "gen 0"
    } else {
        "gen patched"
    };
    // Keyed children (§6.2): the patched-in label inserts between keyed
    // siblings — the LIS keeps the survivors stable, exactly the contract a
    // hot patch exercises.
    let mut root = ViewNode::element("column").prop("gap", 8.0_f32);
    root = root.child(ViewNode::element("label").key(1_u64).prop("text", marker));
    root = root.child(
        ViewNode::element("label")
            .key(2_u64)
            .prop("binding", binding),
    );
    if generation != 0 {
        // The patched generation's new widget: created exactly once by the
        // first patch, then stable across further churn.
        root = root.child(
            ViewNode::element("label")
                .key(3_u64)
                .prop("text", "patched-in widget"),
        );
    }
    root = root.child(ViewNode::element("spacer").key(4_u64).prop("size", 4.0_f32));
    root = root.child(
        ViewNode::element("button")
            .key(5_u64)
            .prop("text", "Increment")
            .prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new({
                    let on_click = Rc::clone(on_click);
                    move || on_click()
                }))),
            ),
    );
    root
}

/// The assembly under test: harness + flux + hot view. The patch path is
/// the production path: swap the generation (what a binary patch does to
/// the code), regenerate (the scoped re-render), settle (one frame).
struct HotApp {
    harness: TestHarness,
    hot: HotView,
    count: alacer_core::FluxRwSignal<i32>,
    binding: RenderBinding<String>,
    generation: Rc<Cell<u32>>,
}

impl HotApp {
    fn mount() -> Self {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let count = harness.pipeline_mut().flux_mut().signal(0_i32);
        let binding = RenderBinding::new("Clicked 0 times".to_string());
        let generation = Rc::new(Cell::new(0_u32));

        let increment = move || {
            // One write batch per click: the ADR-6 contract at the seam.
            alacer_core::flux::Flux::batch_of(|| {
                let current = count.try_get_untracked().unwrap_or_default();
                count.set(current + 1);
            });
        };

        let gen_reader = Rc::clone(&generation);
        let gen_binding = binding.clone();
        let on_click_shared: Rc<dyn Fn()> = Rc::new(increment);
        let generate = move || {
            let generation_value = gen_reader.get();
            hot_app_view(&gen_binding, generation_value, &on_click_shared)
        };

        let hot = HotView::mount(harness.pipeline_mut(), default_registry(), generate);

        // The flux bridge drives the bound label across generations: one
        // batch ⇒ one label repaint, the M0 seam unchanged by M4.
        let label_node = bound_label_node_from(&harness);
        harness.pipeline_mut().flux_mut().bind_value(
            count,
            label_node,
            &binding,
            alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
            |n: &i32| format!("Clicked {n} times"),
        );

        harness.settle();
        Self {
            harness,
            hot,
            count,
            binding,
            generation,
        }
    }

    /// Delivers a "patch": swaps the generation and regenerates — the
    /// scoped re-render. Returns the reconciler's report.
    fn patch(&mut self, generation: u32) -> PatchReport {
        self.generation.set(generation);
        let report = self.hot.regenerate(self.harness.pipeline_mut());
        self.harness.settle();
        report
    }

    /// Clicks the "Increment" button through the a11y tree (the official
    /// query API — reachable, named, actionable).
    fn click_increment(&mut self) {
        self.harness.press(Role::Button, "Increment");
    }

    /// The count signal's current value.
    fn count_value(&self) -> i32 {
        self.count.try_get_untracked().unwrap_or_default()
    }

    /// The bound label's resolved a11y name — what a screen reader sees.
    fn bound_label_text(&self) -> String {
        let node = bound_label_node_from(&self.harness);
        self.harness
            .pipeline()
            .arena()
            .widget(node)
            .label()
            .resolve()
    }
}

/// The bound count label: the first Label-role node whose resolved name
/// starts with "Clicked" (the generation marker label never does).
fn bound_label_node_from(harness: &TestHarness) -> NodeId {
    let mut found = None;
    harness
        .pipeline()
        .arena()
        .for_each_pre_order(&mut |id, slot| {
            if found.is_none()
                && slot.widget().role() == Role::Label
                && slot.widget().label().resolve().starts_with("Clicked")
            {
                found = Some(id);
            }
        });
    found.expect("the view mounted a bound count label")
}

/// The last frame where any pass ran — the interaction/patch frame (the
/// settle loop's trailing idle confirm tick is never the interesting one).
fn last_active_frame(recorder: &CausalRecorder) -> alacer_core::inspect::FrameSummary {
    recorder
        .frames()
        .into_iter()
        .rev()
        .find(|recorded| {
            let frame = &recorded.summary;
            frame.layout_ran || frame.paint_ran || frame.a11y_ran
        })
        .map(|recorded| recorded.summary)
        .expect("an active frame was observed")
}

/// The untouched sibling: the spacer the generations never mention.
fn spacer_node(harness: &TestHarness) -> NodeId {
    let mut found = None;
    harness
        .pipeline()
        .arena()
        .for_each_pre_order(&mut |id, slot| {
            if found.is_none() && slot.widget().role() == Role::Decoration {
                found = Some(id);
            }
        });
    found.expect("the view mounted a spacer")
}

// ---------------------------------------------------------------------------
// The hot-patch contract
// ---------------------------------------------------------------------------

#[test]
fn a_patched_generation_renders_new_values_with_state_intact() {
    let mut app = HotApp::mount();
    let root_before = app.hot.root_id();
    let bound_before = bound_label_node_from(&app.harness);

    // State accumulates: three clicks, the binding carries the count.
    for _ in 0..3 {
        app.click_increment();
    }
    assert_eq!(app.count_value(), 3);
    assert_eq!(app.bound_label_text(), "Clicked 3 times");
    let _marker = app.harness.expect_a11y(Role::Label, "gen 0");
    assert_eq!(
        app.harness.query(Role::Label).named("patched-in").count(),
        0,
        "the pre-patch shape has no patched-in widget"
    );

    // The patch: new values, same state.
    let report = app.patch(1);
    assert_eq!(report.created, 1, "exactly the patched-in widget created");
    assert_eq!(report.removed, 0, "nothing rebuilt");
    assert_eq!(
        report.patched, 2,
        "the generation marker's text changed, and the bound label's \
         binding re-hung (bindings never compare equal by contract — the \
         patcher re-hangs idempotently)"
    );
    assert_eq!(
        app.hot.root_id(),
        root_before,
        "node identity stable across patches (§6.3)"
    );
    assert_eq!(
        bound_label_node_from(&app.harness),
        bound_before,
        "the bound label kept its node"
    );

    // The new generation renders...
    let _patched_marker = app.harness.expect_a11y(Role::Label, "gen patched");
    let _patched_in = app.harness.expect_a11y(Role::Label, "patched-in widget");
    // ...and the old state survives it: same signal, same binding, same
    // count. A patch swaps code, never state (§13.2 item 1).
    assert_eq!(app.count_value(), 3, "the signal survived the patch");
    assert_eq!(
        app.bound_label_text(),
        "Clicked 3 times",
        "the binding slot survived the patch"
    );

    // And the app keeps working: clicks after the patch still count.
    app.click_increment();
    assert_eq!(app.count_value(), 4);
    assert_eq!(app.bound_label_text(), "Clicked 4 times");
}

#[test]
fn a_patch_costs_exactly_one_frame_and_leaves_the_tree_idle() {
    let mut app = HotApp::mount();
    let before = app.harness.pipeline().counters();

    let _report = app.patch(1);

    let after = app.harness.pipeline().counters();
    assert_eq!(
        after.layout_runs - before.layout_runs,
        1,
        "one patch ⇒ one layout pass (the scoped re-render queues masks; \
         the next tick consumes them once — ADR-6's contract, unchanged)"
    );
    assert_eq!(after.paint_runs - before.paint_runs, 1, "one paint pass");

    // The idle contract (A7) holds after a patch: a quiet tick does nothing.
    assert!(
        app.harness.tick().is_idle(),
        "the tree is idle after the patch frame"
    );
}

#[test]
fn patch_churn_with_interleaved_mutations_stays_consistent() {
    let mut app = HotApp::mount();
    let root_before = app.hot.root_id();

    // Ten patches with mutations interleaved: the churn the hot-patch suite
    // exists to catch (§13.2's CI suite).
    for generation in 1..=10_u32 {
        app.click_increment();
        let _report = app.patch(generation);
        assert_eq!(app.hot.root_id(), root_before, "identity never moved");
        assert_eq!(
            app.bound_label_text(),
            format!("Clicked {generation} times"),
            "state intact after patch {generation}"
        );
        assert!(
            app.harness.query(Role::Button).named("Increment").exists(),
            "the button survives every patch, still actionable"
        );
    }
    assert_eq!(app.count_value(), 10);

    // ...and one more click after all that churn.
    app.click_increment();
    assert_eq!(app.count_value(), 11);
}

#[test]
fn untouched_subtrees_do_not_enter_the_patch_frame() {
    // A sibling the generation never touches must not repaint: the scoped
    // re-render patches exactly the differing subtrees (§13.2 item 2).
    let recorder = CausalRecorder::new("scoped");
    let mut app = HotApp::mount();
    app.harness
        .pipeline_mut()
        .set_observer(Box::new(recorder.clone_handle()));
    let spacer = spacer_node(&app.harness);

    let _report = app.patch(1);
    let frame = last_active_frame(&recorder);
    assert!(
        !frame.dirty.iter().any(|node| node.node == spacer),
        "the untouched spacer stayed out of the patch frame's dirty set"
    );
    assert!(
        frame.dirty.iter().any(|node| node.role == Role::Label),
        "the patched labels did enter it (the heatmap is the truth)"
    );
}

// ---------------------------------------------------------------------------
// The Causality inspector, through the real pipeline
// ---------------------------------------------------------------------------

#[test]
fn the_inspector_answers_why_did_this_render() {
    let recorder = CausalRecorder::new("why");
    let mut app = HotApp::mount();
    app.harness
        .pipeline_mut()
        .set_observer(Box::new(recorder.clone_handle()));
    app.harness
        .pipeline_mut()
        .flux_mut()
        .set_causal_capture(true);

    recorder.clear();
    app.click_increment();

    // "Why did this render?" — an edge activation carrying the masks it
    // queued for the bound label (§13.3).
    let edges = recorder.edges();
    assert!(
        edges
            .iter()
            .any(|edge| edge.masks.contains(alacer_core::Change::PAINT)),
        "the click activated the binding edge: {edges:?}"
    );
    let frame = last_active_frame(&recorder);
    assert!(
        frame
            .dirty
            .iter()
            .any(|node| node.node == bound_label_node_from(&app.harness)),
        "the activation resolved into the frame's dirty set"
    );
}

#[test]
fn the_inspector_answers_who_wrote_this_signal() {
    let write_recorder = CausalRecorder::new("who-wrote");
    let mut app = HotApp::mount();
    app.harness
        .pipeline_mut()
        .set_observer(Box::new(write_recorder.clone_handle()));
    app.harness
        .pipeline_mut()
        .flux_mut()
        .set_causal_capture(true);

    // A provenance-tagged write (the FluxWriter seam): the declared origin
    // is what the inspector reports.
    let writer = app.harness.pipeline().flux().writer();
    writer.write(&app.count, 42, "the M4 suite");

    app.harness.tick();
    let recorded_writes = write_recorder.writes();
    assert!(
        recorded_writes
            .iter()
            .any(|write| write.origin == "the M4 suite"),
        "the provenance write landed: {recorded_writes:?}"
    );
}

#[test]
fn the_inspector_answers_who_subscribes() {
    let mut app = HotApp::mount();
    app.harness
        .pipeline_mut()
        .flux_mut()
        .set_causal_capture(true);
    // Re-bind post-mount so the registration happens with capture on (the
    // edge table is the "who subscribes" answer, §13.3).
    let label_node = bound_label_node_from(&app.harness);
    app.harness.pipeline_mut().flux_mut().bind_value(
        app.count,
        label_node,
        &app.binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );
    let edges = app.harness.pipeline().flux().edge_table();
    assert!(
        edges.iter().any(|(_, _, node)| *node == label_node),
        "the binding registered as a subscribable edge: {edges:?}"
    );
}

#[test]
fn the_frame_summary_carries_the_ledger_and_the_heatmap() {
    const TICK_PASSES: [&str; 8] = [
        "mutate",
        "update_focusable",
        "update_focus",
        "style",
        "layout",
        "compose",
        "paint",
        "accessibility",
    ];
    let recorder = CausalRecorder::new("ledger");
    let mut app = HotApp::mount();
    app.harness.pipeline_mut().set_timings(true);
    app.harness
        .pipeline_mut()
        .set_observer(Box::new(recorder.clone_handle()));

    // A real pointer click (the a11y press path routes through pass 10, not
    // the pointer router): Down and Up both record as inputs.
    let bounds = app.harness.query(Role::Button).named("Increment").bounds();
    let center = alacer_core::geom::Point::new(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    );
    recorder.clear();
    app.harness.click(center);

    let frame = last_active_frame(&recorder);
    assert!(
        frame.layout_ran && frame.paint_ran,
        "the click frame ran layout and paint"
    );
    // The ledger is in contract order: the tick's eight passes, in §7.2
    // order, as a subsequence of whatever event passes preceded them.
    let names: Vec<&str> = frame.passes.iter().map(|pass| pass.kind.name()).collect();
    let mut cursor = 0_usize;
    for expected in TICK_PASSES {
        let position = names[cursor..]
            .iter()
            .position(|name| *name == expected)
            .unwrap_or_else(|| panic!("ledger lost {expected} in {names:?}"));
        cursor += position + 1;
    }
    assert!(
        frame.passes.iter().any(|pass| pass.ran && pass.nanos > 0),
        "with timings on, ran passes carry wall times"
    );
    assert!(
        !frame.dirty.is_empty(),
        "the heatmap carries the frame's dirty nodes"
    );
    assert!(
        frame
            .dirty
            .iter()
            .all(|node| node.mask != alacer_core::Change::empty()),
        "every heatmap entry names its masks"
    );
    // Inputs are recorded too: the click's Down and Up both routed.
    let inputs = recorder.inputs();
    assert!(
        inputs
            .iter()
            .any(|(kind, _)| *kind == alacer_core::inspect::InputKind::PointerDown),
        "the pointer-down was recorded: {inputs:?}"
    );
}

#[test]
fn an_unobserved_pipeline_stays_silent_and_the_capture_stays_off() {
    // The overhead contract (ADR-0010): no observer, no capture — the
    // default path costs nothing beyond the branch, and nothing records.
    let mut app = HotApp::mount();
    assert!(
        !app.harness.pipeline().flux().causal_capture_enabled(),
        "capture defaults off"
    );
    app.click_increment();
    let before = app.harness.pipeline().counters();
    let _report = app.patch(1);
    let after = app.harness.pipeline().counters();
    assert_eq!(after.layout_runs - before.layout_runs, 1, "still one frame");
}
