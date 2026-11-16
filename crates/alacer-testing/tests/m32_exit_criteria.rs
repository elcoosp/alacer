//! M32 exit-criteria integration tests (§6.1, ADR-0038): the value tier
//! joins the view registry. The re-open rule the M6 canon's registry
//! work left open cashes — a widget vocabulary the `rsx!` DSL cannot
//! express is a second-class citizen of its own toolkit — so `slider`
//! and `progress` register as view elements with the create/patch pairs
//! every canon element rides, the patch answering the widgets' own
//! semantics: the funnel's clamp-and-snap at create and patch, the idle
//! frame on a write that lands where the value already is, the bound
//! progress re-hanging its binding slot idempotently.
//!
//! Every test runs the real pipeline (A4): the view mounted through the
//! real element registry, every assertion through the a11y tree — the
//! only official query API (ADR-10) — and the log where the idle-frame
//! property is the fact.

use alacer_core::flux::RenderBinding;
use alacer_core::geom::Point;
use alacer_core::widget::Role;
use alacer_macro::rsx;
use alacer_testing::TestHarness;
use alacer_view::{ViewNode, ViewRoot};

/// The harness-layer mount: the default element registry (the facade's
/// `mount_view` re-exports exactly this composition, ADR-9).
fn mount(view: &ViewNode, pipeline: &mut alacer_core::passes::PassPipeline) -> ViewRoot {
    ViewRoot::mount(view, pipeline, alacer_widgets::default_registry())
}

/// The value tier in a view (ADR-0038): a slider whose funnel routes to
/// the app's sink, and a progress whose value rides a binding the app's
/// effect side owns — the value tier's two doors, view vocabulary like
/// the M6 canon before them. `slider_value` rides the create/patch
/// write (clamped and snapped inside the widget); `level` rides the
/// binding.
fn value_view(
    sink: std::rc::Rc<std::cell::RefCell<Vec<f32>>>,
    level: RenderBinding<f32>,
    slider_value: f32,
) -> ViewNode {
    rsx! {
        column { gap: 8.0_f32, padding: 12.0_f32,
            slider {
                name: "Volume",
                min: 0.0_f32,
                max: 100.0_f32,
                step: 10.0_f32,
                value: slider_value,
                on_change: alacer_view::PropValue::NumberHandler(std::rc::Rc::new(
                    std::cell::RefCell::new(move |value: f32| {
                        sink.borrow_mut().push(value);
                    }),
                )),
            }
            progress {
                name: "Level",
                min: 0.0_f32,
                max: 100.0_f32,
                binding: level,
            }
        }
    }
}

#[test]
fn m32_both_elements_are_reachable_and_valued_from_the_first_frame() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let sink = std::rc::Rc::new(std::cell::RefCell::new(Vec::<f32>::new()));
    let level = RenderBinding::new(20.0_f32);
    // The create write rides the funnel: an out-of-range value clamps
    // and snaps inside the widget — 150 against a max of 100 lands at
    // the wall, never past it.
    let view = value_view(std::rc::Rc::clone(&sink), level.clone(), 150.0);
    let _root = mount(&view, harness.pipeline_mut());
    harness.settle();

    let slider = harness.expect_a11y(Role::Slider, "Volume");
    assert_eq!(
        slider.value,
        Some("100".to_string()),
        "the create write rode the funnel — clamped at the max"
    );
    let progress = harness.expect_a11y(Role::ProgressIndicator, "Level");
    assert_eq!(
        progress.value,
        Some("20".to_string()),
        "the binding's value is the report from the first frame"
    );
}

#[test]
fn m32_a_drag_is_heard_in_gesture_order_and_the_rebuild_patches_in_place() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let trail = harness.trail();
    let sink = std::rc::Rc::new(std::cell::RefCell::new(Vec::<f32>::new()));
    let level = RenderBinding::new(20.0_f32);
    let view = value_view(std::rc::Rc::clone(&sink), level.clone(), 50.0);
    let mut root = mount(&view, harness.pipeline_mut());
    harness.settle();

    // The drag: press at the thumb, drag toward the track's end — the
    // funnel routes the gesture, the view handler observes the values
    // in gesture order, and the a11y tree carries what the funnel
    // wrote.
    let bounds = harness
        .expect_a11y(Role::Slider, "Volume")
        .bounds
        .expect("laid out slider");
    let start = Point::new(
        bounds.origin.x + bounds.size.width * 0.5,
        bounds.origin.y + bounds.size.height / 2.0,
    );
    let end = Point::new(
        bounds.origin.x + bounds.size.width * 0.95,
        bounds.origin.y + bounds.size.height / 2.0,
    );
    harness.drag(start, end);
    harness.settle();

    let heard = sink.borrow().clone();
    assert!(
        !heard.is_empty(),
        "the view handler observed the funnel's writes"
    );
    let last = heard[heard.len() - 1];
    assert!(last > 50.0, "the drag moved the value up, got {last}");
    assert!(
        heard.windows(2).all(|pair| pair[0] <= pair[1]),
        "the funnel's writes arrive in gesture order"
    );
    let slider = harness.expect_a11y(Role::Slider, "Volume");
    assert_eq!(
        slider.value,
        Some(format!("{last}")),
        "the a11y tree carries what the funnel wrote"
    );

    // The rebuild patches in place (created == removed == 0), the new
    // value prop landing through the same funnel.
    let again = value_view(std::rc::Rc::clone(&sink), level.clone(), 30.0);
    let report = root.update(&again, harness.pipeline_mut());
    harness.settle();
    assert_eq!(
        report.created, 0,
        "the value tier patches, never re-creates"
    );
    assert_eq!(report.removed, 0, "the value tier patches, never unmounts");
    let slider = harness.expect_a11y(Role::Slider, "Volume");
    assert_eq!(
        slider.value,
        Some("30".to_string()),
        "the rebuild's value prop landed through the funnel"
    );

    // A write that lands where the value already is queues no masks
    // (the M21 idle-frame property, the patch answering the widget's
    // own semantics): the identical rebuild streams nothing on the log
    // — the honest absence, the idle frame as data.
    let _ = trail.drain();
    let noop = value_view(sink, level, 30.0);
    let report = root.update(&noop, harness.pipeline_mut());
    harness.settle();
    assert_eq!(
        report.created + report.removed,
        0,
        "the no-op rebuild churns nothing"
    );
    let streamed: Vec<alacer_core::inspect::InspectEvent> = trail.drain();
    assert!(
        streamed
            .iter()
            .all(|event| matches!(event, alacer_core::inspect::InspectEvent::Frame(_))),
        "the no-op value write streams no fact — the idle frame on the log, got {streamed:?}"
    );
}

#[test]
fn m32_the_bound_progress_keeps_the_binding_across_rebuilds() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let sink = std::rc::Rc::new(std::cell::RefCell::new(Vec::<f32>::new()));
    let level = RenderBinding::new(20.0_f32);
    let view = value_view(std::rc::Rc::clone(&sink), level.clone(), 50.0);
    let mut root = mount(&view, harness.pipeline_mut());
    harness.settle();
    let progress = harness.expect_a11y(Role::ProgressIndicator, "Level");
    assert_eq!(
        progress.value,
        Some("20".to_string()),
        "the binding's value is the report"
    );

    // The rebuild re-hangs the same binding slot idempotently — never a
    // visual change, the bound label's rule at the value tier — and the
    // progress keeps reporting the binding's number.
    let again = value_view(sink, level, 30.0);
    let _report = root.update(&again, harness.pipeline_mut());
    harness.settle();
    let progress = harness.expect_a11y(Role::ProgressIndicator, "Level");
    assert_eq!(
        progress.value,
        Some("20".to_string()),
        "the bound progress keeps the binding's number across the rebuild"
    );
}

#[test]
fn m32_the_living_documents_re_agree() {
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M80",
        "eighty-seven decision records",
        "`progress` are `rsx!` elements",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }

    let architecture = read_workspace_file("docs/architecture.md");
    assert!(
        architecture.contains("ADR-0038") && architecture.contains("ADR-0077"),
        "docs/architecture.md carries the register rows"
    );

    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M71 — the frame's refusals join the causal log (ADR-0077)**"),
        "the CHANGELOG stages the current cut"
    );

    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("`value_view`") || catalog.contains("value_view"),
        "docs/examples.md carries the value-tier demonstration"
    );

    // The sibling suites' pins ride the same cut (one sweep, no drift).
    let m70 = read_workspace_file("crates/alacer-testing/tests/m70_exit_criteria.rs");
    assert!(
        m70.contains("M0 through M80"),
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
