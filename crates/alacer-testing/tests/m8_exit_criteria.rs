//! M8 exit-criteria integration tests (§10.2, ADR-0014): input storms cost
//! one frame of work, not N events — the coalescer routes through the
//! pipeline queue, the platform pushes and never drains, wheel deltas
//! accumulate on both axes, and the batching ratio is a published budget.
//!
//! Every test runs the real pipeline (A4); the platform wiring is pinned by
//! source, matching the M6/M7 suite convention.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::{Point, Size, Vector};
use alacer_core::passes::PassKind;
use alacer_core::style::{MeasureSpec, Size2, Style};
use alacer_core::widget::EventResponse;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;

/// The probe's shared move/witness log.
type ProbeLog<T> = Rc<RefCell<Vec<T>>>;

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

/// A probe widget recording every pointer event routed to it: the storm's
/// witness. Fixed-size so the hit-test chain finds it.
struct PointerProbe {
    size: f32,
    moves: Rc<RefCell<Vec<Point>>>,
    wheels: Rc<RefCell<Vec<Vector>>>,
}

impl PointerProbe {
    fn new(size: f32) -> (Self, ProbeLog<Point>, ProbeLog<Vector>) {
        let moves = Rc::new(RefCell::new(Vec::new()));
        let wheels = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                size,
                moves: Rc::clone(&moves),
                wheels: Rc::clone(&wheels),
            },
            moves,
            wheels,
        )
    }
}

impl Widget for PointerProbe {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("probe")
    }

    fn style(&self) -> Style {
        Style {
            size: Size2::uniform(self.size),
            ..Style::default()
        }
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(Size::new(self.size, self.size)))
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        match event {
            PointerEvent::Move { position } => self.moves.borrow_mut().push(*position),
            PointerEvent::Wheel { delta, .. } => self.wheels.borrow_mut().push(*delta),
            _ => {}
        }
        EventResponse::Handled
    }
}

fn mount_probe(harness: &mut TestHarness) -> (ProbeLog<Point>, ProbeLog<Vector>) {
    let (probe, moves, wheels) = PointerProbe::new(200.0);
    let _root = harness.mount(Box::new(probe));
    harness.settle();
    (moves, wheels)
}

fn event_runs(report: &alacer_core::passes::FrameReport) -> usize {
    report
        .passes
        .iter()
        .filter(|run| run.kind == PassKind::Event && run.ran)
        .count()
}

#[test]
fn m8_a_move_storm_routes_one_coalesced_event() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let (moves, _wheels) = mount_probe(&mut harness);

    let routed_before = harness.pipeline().counters().events_routed;
    for i in 0..50 {
        harness.queue_pointer(PointerEvent::Move {
            position: Point::new(alacer_core::num::usize_to_f32(i), 0.0),
        });
    }
    let report = harness.tick();

    assert_eq!(
        moves.borrow().len(),
        1,
        "50 queued moves coalesce to one routed event"
    );
    assert_eq!(
        moves.borrow()[0],
        Point::new(49.0, 0.0),
        "the latest position wins (§10.2)"
    );
    assert_eq!(
        harness.pipeline().counters().events_routed - routed_before,
        1,
        "one event pass, not 50 (ADR-6's one-frame currency, input half)"
    );
    assert_eq!(event_runs(&report), 1, "the frame ledger agrees");
}

#[test]
fn m8_a_wheel_burst_lands_as_one_accumulated_delta() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let (_moves, wheels) = mount_probe(&mut harness);

    for _ in 0..10 {
        harness.queue_pointer(PointerEvent::Wheel {
            delta: Vector::new(0.0, -12.0),
            position: Point::new(100.0, 100.0),
        });
    }
    let report = harness.tick();

    assert_eq!(wheels.borrow().len(), 1, "ten ticks of wheel, one event");
    assert_eq!(wheels.borrow()[0], Vector::new(0.0, -120.0));
    assert_eq!(event_runs(&report), 1);
}

#[test]
fn m8_direction_flip_splits_the_wheel_run() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let (_moves, wheels) = mount_probe(&mut harness);

    for _ in 0..3 {
        harness.queue_pointer(PointerEvent::Wheel {
            delta: Vector::new(0.0, -30.0),
            position: Point::zero(),
        });
    }
    for _ in 0..2 {
        harness.queue_pointer(PointerEvent::Wheel {
            delta: Vector::new(0.0, 15.0),
            position: Point::zero(),
        });
    }
    harness.tick();

    assert_eq!(
        wheels.borrow().as_slice(),
        &[Vector::new(0.0, -90.0), Vector::new(0.0, 30.0)],
        "the down-run and the up-run route as two events"
    );
}

#[test]
fn m8_horizontal_wheel_deltas_reach_widgets() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let (_moves, wheels) = mount_probe(&mut harness);

    for _ in 0..4 {
        harness.queue_pointer(PointerEvent::Wheel {
            delta: Vector::new(6.0, 0.0),
            position: Point::new(50.0, 50.0),
        });
    }
    harness.tick();

    assert_eq!(
        wheels.borrow().as_slice(),
        &[Vector::new(24.0, 0.0)],
        "the x axis survives batching — the pre-M8 platform drop is fixed"
    );
}

#[test]
fn m8_the_list_scrolls_through_the_queue_without_measuring() {
    let mut harness = TestHarness::new(400.0, 400.0);
    let root = harness.mount(alacer_widgets::Column::new().boxed());
    let list = alacer_widgets::List::new(160.0);
    let handle = list.handle();
    let list_node = harness.add_child(root, list.boxed());
    for i in 0..24 {
        let colors = [
            alacer_core::color::Color::rgb(239, 68, 68),
            alacer_core::color::Color::rgb(59, 130, 246),
            alacer_core::color::Color::rgb(34, 197, 94),
            alacer_core::color::Color::rgb(234, 179, 8),
        ];
        let _ = harness.add_child(
            list_node,
            alacer_widgets::Panel::new(28.0, colors[i % colors.len()]).boxed(),
        );
    }
    harness.settle();
    // The app owns the content height (the ListHandle contract, §8.2).
    handle.set_content_height(24.0 * 28.0);
    harness.reset_measure_calls();

    for _ in 0..5 {
        harness.queue_pointer(PointerEvent::Wheel {
            delta: Vector::new(0.0, -40.0),
            position: Point::new(50.0, 80.0),
        });
    }
    harness.settle();

    let offset = handle.offset();
    assert!(
        (offset - 200.0).abs() < 0.001,
        "five coalesced wheel ticks scroll once, by the accumulated delta (got {offset})"
    );
    assert_eq!(
        harness.measure_calls(),
        0,
        "scroll stays translation-only through the queue (§8.2)"
    );
}

#[test]
fn m8_the_platform_pushes_and_never_drains() {
    let source = read_workspace_file("crates/alacer-platform/src/lib.rs");
    assert!(
        source.contains("queue_pointer("),
        "the platform queues input into the pipeline (ADR-0014)"
    );
    assert!(
        !source.contains("self.coalescer"),
        "the platform owns no coalescer — the pipeline does"
    );
    assert!(
        !source.contains(".drain()"),
        "no per-callback drain: batching happens once per frame"
    );
}

#[test]
fn m8_the_corpus_pin_agrees_with_the_published_data() {
    // M8 grows the corpus by the coordinated rule (the M5 suite's own doc):
    // one bench file, two criterion ids, two regression contracts, one
    // budget — and every pin suite moves together (M9 grew it again, by the
    // same rule: ADR-0015's a11y ratio pair).
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");

    let regressions = contracts["regressions"]
        .as_array()
        .expect("regressions array");
    let benches = baselines["benches"].as_object().expect("benches object");
    assert_eq!(
        benches.len(),
        55,
        "the corpus is 55: the M18 route and the M20 capture case ride it together"
    );
    assert_eq!(
        regressions.len(),
        55,
        "every bench carries a regression gate"
    );
    for id in ["input/storm_50_moves_queued", "input/storm_50_moves_direct"] {
        assert!(benches.contains_key(id), "{id} published");
        assert!(
            regressions
                .iter()
                .any(|gate| gate["baseline"] == serde_json::json!(id)),
            "{id} contracted"
        );
    }
    assert!(
        contracts["budgets"]
            .as_array()
            .expect("budgets")
            .iter()
            .any(|budget| budget["id"] == serde_json::json!("input_storm_batching")),
        "the batching ratio is a published budget"
    );

    // The three earlier suites pin the same number — the chain cannot drift.
    for suite in [
        "m5_exit_criteria.rs",
        "m6_exit_criteria.rs",
        "m7_exit_criteria.rs",
    ] {
        let source = read_workspace_file(&format!("crates/alacer-testing/tests/{suite}"));
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 55;"),
            "{suite} must pin the 55-case corpus"
        );
    }
}
