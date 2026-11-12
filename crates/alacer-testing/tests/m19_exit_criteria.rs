//! M19 exit-criteria integration tests (§10.1, ADR-0025): the pointer
//! capture joins the dispatch contract. A press holds its deepest
//! accepting hit; the gesture's Move/Up/Leave stream rides the captured
//! node's ancestor chain wherever the pointer goes; the Up, the Leave,
//! and the target's death release it; the wheel stays hit-tested; and a
//! frame with no capture routes exactly as the hit test always did. The
//! surface work rides the same milestone: the dead convenience API is
//! gone, the `rsx!` single-root diagnostic names what it found, and the
//! living documents re-agree.
//!
//! Every test runs the real pipeline (A4) with the real layout host — the
//! exact route a window's pointer events take.

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::{Point, Size};
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves")
}

fn read_workspace_file(path: &str) -> String {
    let path = workspace_root().join(path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required file {} missing: {error}", path.display()))
}

fn parse_workspace_json(path: &str) -> serde_json::Value {
    let text = read_workspace_file(path);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path} is not valid JSON: {error}"))
}

/// The probe: a fixed panel that records the tag of every routed pointer
/// event — the consumer-side witness the capture contract answers to, the
/// same shape the `drag_capture` example drives.
struct CaptureProbe {
    size: Size,
    log: Rc<RefCell<Vec<&'static str>>>,
}

impl Widget for CaptureProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("capture probe")
    }

    fn measure_spec(&self) -> Option<alacer_core::style::MeasureSpec> {
        Some(alacer_core::style::MeasureSpec::Fixed(self.size))
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        let tag = match event {
            PointerEvent::Down { .. } => "down",
            PointerEvent::Up { .. } => "up",
            PointerEvent::Move { .. } => "move",
            PointerEvent::Leave => "leave",
            PointerEvent::Wheel { .. } => "wheel",
        };
        self.log.borrow_mut().push(tag);
        EventResponse::Handled
    }
}

/// A viewport app with a 120×60 probe at the origin and empty space
/// beside and below it — the press lands inside the probe, the drag
/// wanders out.
fn mount_probe(harness: &mut TestHarness) -> Rc<RefCell<Vec<&'static str>>> {
    let root = harness.mount(Column::new().boxed());
    let log = Rc::new(RefCell::new(Vec::new()));
    let _ = harness.add_child(
        root,
        Box::new(CaptureProbe {
            size: Size::new(120.0, 60.0),
            log: Rc::clone(&log),
        }),
    );
    let _ = harness.add_child(root, Label::new("The drag capture (ADR-0025)").boxed());
    harness.settle();
    log
}

#[test]
fn m19_a_captured_drag_routes_outside_the_target() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let log = mount_probe(&mut harness);

    // The press lands inside the probe's frame and holds it.
    harness.pointer_press(Point::new(60.0, 30.0));
    assert!(
        harness.pipeline().captured().is_some(),
        "the press holds its deepest accepting hit"
    );

    // The drag wanders far past the probe's edge — the stream keeps
    // arriving: this is the route the hit test dropped before M19.
    harness.pointer_move(Point::new(380.0, 190.0));
    harness.pointer_move(Point::new(30.0, 180.0));
    harness.pointer_release(Point::new(30.0, 180.0));

    assert_eq!(
        log.borrow().as_slice(),
        ["down", "move", "move", "up"],
        "the captured gesture rides the target's chain, wherever the pointer is"
    );
    assert!(
        harness.pipeline().captured().is_none(),
        "the release ends it"
    );

    // The pointer is back in hit-test land: a hover past the probe
    // routes nothing to it.
    harness.pointer_move(Point::new(380.0, 190.0));
    assert_eq!(
        log.borrow().as_slice(),
        ["down", "move", "move", "up"],
        "the uncaptured hover follows the hit test again"
    );
}

#[test]
fn m19_the_leave_ends_the_capture_and_reaches_the_target() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let log = mount_probe(&mut harness);

    harness.pointer_press(Point::new(60.0, 30.0));
    // Leave carries no position: without a capture it routes nowhere;
    // with one it rides the captured chain and ends the gesture — the
    // target gets to unpress honestly.
    harness
        .pipeline_mut()
        .dispatch_pointer(&PointerEvent::Leave);
    harness.settle();

    assert_eq!(
        log.borrow().as_slice(),
        ["down", "leave"],
        "the captured target sees the leave"
    );
    assert!(
        harness.pipeline().captured().is_none(),
        "the leave releases"
    );
}

#[test]
fn m19_the_wheel_stays_hit_tested_during_a_capture() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let log = mount_probe(&mut harness);

    harness.pointer_press(Point::new(60.0, 30.0));
    // A wheel while the gesture is held: it scrolls what is under the
    // pointer, not what the press holds (the DOM's shape).
    harness.wheel(Point::new(380.0, 190.0), (0.0, -12.0));
    harness.pointer_release(Point::new(60.0, 30.0));

    assert_eq!(
        log.borrow().as_slice(),
        ["down", "up"],
        "the wheel rode the hover, never the capture"
    );
}

#[test]
fn m19_the_uncaptured_path_is_unchanged() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let log = mount_probe(&mut harness);

    // Fifty moves across and past the probe's frame with no press in
    // flight: exactly the ones inside the frame arrive — the capture adds
    // no route of its own while it holds nothing (the zero-cost idle
    // property the storm budget leans on).
    for i in 0..50 {
        harness
            .pipeline_mut()
            .dispatch_pointer(&alacer_core::events::PointerEvent::Move {
                position: Point::new(60.0, 20.0 + alacer_core::num::usize_to_f32(i)),
            });
    }
    harness.settle();

    let routed = log.borrow().iter().filter(|tag| **tag == "move").count();
    assert_eq!(
        routed, 40,
        "the moves inside the 120×60 frame, and only those"
    );
    assert!(
        log.borrow().iter().all(|tag| *tag == "move"),
        "moves only — no synthetic down or up"
    );
    assert!(
        harness.pipeline().captured().is_none(),
        "no press, no capture"
    );
}

#[test]
fn m19_a_captured_release_completes_the_button_click() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let clicks = Rc::new(std::cell::Cell::new(0_u32));
    let counter = Rc::clone(&clicks);
    let root = harness.mount(Column::new().boxed());
    let button = harness.add_child(
        root,
        Button::new("Grab")
            .on_click(move || counter.set(counter.get() + 1))
            .boxed(),
    );
    let _ = harness.add_child(root, Label::new("The captured release").boxed());
    harness.settle();

    // Press inside the button, drag off it, release far outside: the
    // gesture began there, so it ends there — the armed button answers
    // the release that ends its own capture.
    let bounds = harness.bounds(button);
    let center = Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + bounds.height() * 0.5,
    );
    harness.pointer_press(center);
    let outside = Point::new(bounds.max_x() + 60.0, bounds.max_y() + 60.0);
    assert_eq!(clicks.get(), 0, "the drag alone fires nothing");
    harness.pointer_release(outside);

    assert_eq!(
        clicks.get(),
        1,
        "the captured release completes the press-and-release on its target"
    );
    assert!(
        harness.pipeline().captured().is_none(),
        "the gesture is over"
    );
}

#[test]
fn m19_the_surface_work_rides_the_same_cut() {
    // The dead convenience API is gone (the M18 pattern's continuation):
    // the coalescer's `pending()` had zero call sites workspace-wide.
    let input = read_workspace_file("crates/alacer-core/src/input.rs");
    assert!(
        !input.contains("pub fn pending"),
        "the dead pending() accessor is gone"
    );

    // The rsx! single-root diagnostic names what it found and carries its
    // help text — the weak-error cleanup (ADR-0025).
    let macro_src = read_workspace_file("crates/alacer-macro/src/lib.rs");
    assert!(
        macro_src.contains("rsx! expects one root element, found"),
        "the rsx! single-root error names the offending token"
    );
    assert!(
        macro_src.contains("help: wrap siblings in a container"),
        "the rsx! sibling error carries its help text"
    );

    // The capture seam's observability rides the documented target — the
    // workspace's tracing idiom (ADR-0022/0023/0024's shape).
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    assert!(
        passes.contains("target: \"alacer::capture\""),
        "the capture route traces at the alacer::capture target"
    );
}

#[test]
fn m19_the_living_documents_re_agree() {
    // The README's criterion count agrees with the published baseline:
    // 60 cases since M20's measurement run joined the capture case (the
    // M19-promised IOU), 61 since M27's outcome route joined — the pin
    // moves with the corpus.
    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("All 62 criterion cases"),
        "the README quotes the baselines.json bench_count (61)"
    );
    assert!(
        readme.contains("Since M19 the drag holds its target"),
        "the README's input bullet carries the capture contract"
    );
    assert!(
        readme.contains("M0 through M34"),
        "the README status reflects the living cut — the M26 sweep joins the \
         milestone literal to this suite's pin set, the chain cannot drift"
    );

    // The capture milestone's demonstration is cataloged with the others.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("drag_capture"),
        "docs/examples.md lists the capture demonstration"
    );

    // The architecture carries the decision, the register row, and the
    // capture paragraph in the routing spec.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0025",
        "The captured drag",
        "41 decision records",
        "implicitly holds its deepest accepting hit",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The ADR file itself documents the scope and its coordinated
    // remainders.
    let adr = read_workspace_file("docs/adr/ADR-0025-m19-scope.md");
    assert!(
        adr.contains("M19 scope") && adr.contains("pointer capture"),
        "ADR-0025 documents the captured-drag scope"
    );

    // The corpus is 61: the refused route's criterion case joined at
    // M20's measurement run by the coordinated rule, the value-write
    // route joined at M21's, and the M27 outcome route joined at this
    // cut's measurement run — the checker fails an uncontracted
    // benchmark, so each bench and its baseline landed together
    // (ADR-0011, CONTRIBUTING's publishing rule).
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    assert_eq!(
        baselines["bench_count"],
        serde_json::json!(62),
        "the published corpus carries the capture case the code half promised"
    );
}
