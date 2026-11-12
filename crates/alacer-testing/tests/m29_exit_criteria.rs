//! M29 exit-criteria integration tests (§10.3, ADR-0035): the directional
//! keys ride the walk. The rule ADR-0032 pre-registered (and ADR-0033
//! re-stated) ships — "a platform key needs to ride a direction (a D-pad
//! button, a TV remote's ring) → map the key onto `FocusMove` at the
//! platform's key-fallback seam, priced by a real backend — never a
//! second dispatch channel". The platform's key map already carries the
//! four arrows (`alacer-platform/src/keys.rs`: winit's `NamedKey` arrows
//! map onto `Key::ArrowLeft/Right/Up/Down`), so the seam is
//! [`PassPipeline::dispatch_key`]'s fallback — the same one Tab and
//! Shift+Tab ride since M24: when the focused widget's answer (or the
//! absence of focus) leaves the key ignored and no modifier is held, a
//! plain directional key maps onto its [`FocusMove`] and rides the same
//! funnel — the walk lands the geometry's answer, the refusal falls
//! through honestly (`EventResponse::Ignored`, the key the window never
//! claims), and the M28 trail records every ride. A focused widget's own
//! answer always takes precedence: a focused input's caret, a focused
//! slider's step — the widget's vocabulary is the widget's business; the
//! window's arrows are only the window's.
//!
//! Every test runs the real pipeline (A4) with the real layout host —
//! the exact route a TV remote's ring takes.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{FocusRecord, InspectEvent, InspectObserver};
use alacer_core::passes::{FocusMove, FocusMoveOutcome, FocusRefusal};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, Row, Slider};

/// The causal-trail collector the tests install as the pipeline's
/// observer: the focus records stream into a bounded ring (the M28
/// helper's shape — the m28 suite pins the ring; here it is the
/// assertion source for the arrows' rides).
struct TrailObserver {
    focus: std::collections::VecDeque<FocusRecord>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            focus: std::collections::VecDeque::with_capacity(64),
        }
    }
}

impl InspectObserver for TrailObserver {
    fn observe(&mut self, event: InspectEvent) {
        if let InspectEvent::Focus(record) = event {
            if self.focus.len() >= 64 {
                self.focus.pop_front();
            }
            self.focus.push_back(record);
        }
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the recorder's producer/consumer split, in the suite's own shape.
struct Share(std::rc::Rc<std::cell::RefCell<TrailObserver>>);

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The D-pad window the arrows walk: a two-by-two grid of buttons — the
/// same window the M26/M27/M28 suites pin, because the walk's geometry
/// is pinned there and the arrows ride the same walk. The pre-order
/// candidate order is `[top_left, top_right, bottom_left, bottom_right]`.
fn mount_grid(harness: &mut TestHarness) -> Vec<alacer_core::NodeId> {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let top = harness.add_child(root, Row::new().gap(8.0).boxed());
    let top_left = harness.add_child(top, Button::new("Top left").on_click(|| {}).boxed());
    let top_right = harness.add_child(top, Button::new("Top right").on_click(|| {}).boxed());
    let bottom = harness.add_child(root, Row::new().gap(8.0).boxed());
    let bottom_left = harness.add_child(bottom, Button::new("Bottom left").on_click(|| {}).boxed());
    let bottom_right =
        harness.add_child(bottom, Button::new("Bottom right").on_click(|| {}).boxed());
    let _ = harness.add_child(
        root,
        Label::new("The ring's arrows ride the same walk the code's doors do").boxed(),
    );
    harness.settle();
    vec![top_left, top_right, bottom_left, bottom_right]
}

/// A plain directional press: pressed state, no modifiers — the D-pad
/// button's own shape (the chords are the widgets' vocabulary).
fn dpad(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

/// The name the tree carries for `node` — the landing's endpoint is read
/// back through the a11y vocabulary, never widget internals.
fn tree_name(harness: &TestHarness, node: alacer_core::NodeId) -> String {
    harness.pipeline().arena().widget(node).label().resolve()
}

/// The slider's carried value, read back through the a11y tree — the
/// widget's answer is read through the vocabulary, never internals (the
/// m20 suite's own route).
fn slider_value(harness: &TestHarness) -> String {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| {
            tree.find(alacer_core::Role::Slider, "Volume")
                .map(|entry| entry.value.clone())
        })
        .flatten()
        .expect("the slider's value is in the tree")
}

#[test]
fn m29_the_arrows_walk_the_grid_through_the_real_dispatch() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let nodes = mount_grid(&mut harness);
    let [top_left, top_right, bottom_left, bottom_right] = [nodes[0], nodes[1], nodes[2], nodes[3]];

    // From no focus the walk takes the first candidate (the entry
    // contract, direction-agnostic — ADR-0032's repair).
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowRight));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the window claims the arrow it walked with"
    );
    assert_eq!(harness.pipeline().focus(), Some(top_left));
    assert_eq!(tree_name(&harness, top_left), "Top left");

    // The stride: the geometry's answer, not the tree's next stop. From
    // top_left, Right lands top_right; Down from there lands
    // bottom_right (the tree's next stop would be bottom_left — the row
    // breaks the stride); Left lands bottom_left; Up returns.
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowRight));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(harness.pipeline().focus(), Some(top_right));
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowDown));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(harness.pipeline().focus(), Some(bottom_right));
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowLeft));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(harness.pipeline().focus(), Some(bottom_left));
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowUp));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(
        harness.pipeline().focus(),
        Some(top_left),
        "Up from bottom_left lands the geometry's answer"
    );
}

#[test]
fn m29_a_focused_widgets_answer_takes_precedence() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let volume = harness.add_child(root, Slider::new("Volume", 0.0, 100.0).step(1.0).boxed());
    harness.settle();

    // The press-to-focus seam focuses the slider; the arrow is the
    // widget's vocabulary — the value steps by the widget's own rule,
    // the focus stays, and the window's fallback never rides.
    let bounds = harness
        .query(alacer_core::Role::Slider)
        .named("Volume")
        .bounds();
    harness.pointer_press(alacer_core::geom::Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + 8.0,
    ));
    harness.pointer_release(alacer_core::geom::Point::new(
        bounds.min_x() + bounds.width() * 0.5,
        bounds.min_y() + 8.0,
    ));
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        Some(volume),
        "the press focused"
    );

    // Home pins the minimum first (the widget's own chord) — the arrow's
    // step is then exactly one.
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::Home));
    let _ = response;
    harness.settle();
    assert_eq!(
        slider_value(&harness),
        "0",
        "the widget's own chord stepped"
    );

    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowRight));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the widget handled its own arrow"
    );
    harness.settle();
    assert_eq!(
        slider_value(&harness),
        "1",
        "the slider stepped by its own rule, not the walk's"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(volume),
        "the widget's arrow never moved the focus"
    );

    // The m20 suite's End contract still stands beside the fallback: the
    // window's arrows are only the window's.
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::End));
    let _ = response;
    harness.settle();
    assert_eq!(
        slider_value(&harness),
        "100",
        "the widget's own chords stay"
    );
}

#[test]
fn m29_a_refused_direction_falls_through_honestly() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let nodes = mount_grid(&mut harness);
    let bottom_right = nodes[3];
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new()));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // Walk to the corner through the doors, then the frame settles (the
    // masks drain so the corner's refusal is the geometry's truth).
    let _ = harness.pipeline_mut().focus_move(FocusMove::Next);
    let _ = harness.pipeline_mut().focus_move(FocusMove::Right);
    let _ = harness.pipeline_mut().focus_move(FocusMove::Down);
    assert_eq!(harness.pipeline().focus(), Some(bottom_right));
    harness.settle();

    // The corner's arrow: buttons do not answer arrows, the walk refuses
    // (space never wraps — ADR-0032), the window does not claim the key,
    // and the M28 trail carries the refusal like every ride.
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowRight));
    assert!(
        matches!(response, alacer_core::EventResponse::Ignored),
        "a refused direction falls through — the key is nobody's"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(bottom_right),
        "the refusal never munges the focus"
    );
    let borrowed = observer.borrow();
    let last = borrowed.focus.back().expect("the funnel streamed the ride");
    assert_eq!(
        last.outcome,
        FocusMoveOutcome::Refused(FocusRefusal::NothingInDirection {
            node: bottom_right,
            direction: FocusMove::Right,
        }),
        "the trail names the empty direction"
    );
    drop(borrowed);
    assert!(
        harness.tick().is_idle(),
        "the refused arrow queues no masks — an idle frame, the M11 rule"
    );
}

#[test]
fn m29_the_arrows_ride_streams_to_the_causal_log() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let nodes = mount_grid(&mut harness);
    let top_left = nodes[0];
    let top_right = nodes[1];
    let observer = std::rc::Rc::new(std::cell::RefCell::new(TrailObserver::new()));
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer))));

    // The ring's ride streams what the door's ride streams: one record
    // per arrow, the movement the key named, the landing's endpoints.
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowRight));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(harness.pipeline().focus(), Some(top_left));
    let response = harness.pipeline_mut().dispatch_key(&dpad(Key::ArrowRight));
    assert!(matches!(response, alacer_core::EventResponse::Handled));
    assert_eq!(harness.pipeline().focus(), Some(top_right));
    let borrowed = observer.borrow();
    assert_eq!(borrowed.focus.len(), 2, "two rides, two records");
    assert_eq!(borrowed.focus[0].movement, FocusMove::Right);
    assert_eq!(borrowed.focus[1].movement, FocusMove::Right);
    assert_eq!(
        borrowed.focus[0].outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: None,
            to: top_left,
            wrapped: false,
        }),
        "the entry landing streams like every entry"
    );
    assert_eq!(
        borrowed.focus[1].outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: Some(top_left),
            to: top_right,
            wrapped: false,
        }),
        "the stride landing streams like every stride"
    );
}

#[test]
fn m29_the_surface_rides_the_same_cut() {
    // The fallback ships where Tab's does: the dispatch funnel maps the
    // four arrows onto the walk's directions — no second channel.
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let passes = std::fs::read_to_string(root.join("crates/alacer-core/src/passes.rs"))
        .expect("passes.rs is in the tree");
    for fragment in [
        "Key::ArrowLeft => Some(FocusMove::Left)",
        "Key::ArrowRight => Some(FocusMove::Right)",
        "Key::ArrowUp => Some(FocusMove::Up)",
        "Key::ArrowDown => Some(FocusMove::Down)",
    ] {
        assert!(
            passes.contains(fragment),
            "the dispatch funnel maps the arrow: {fragment}"
        );
    }

    // The consumer demonstration exists and rides the workspace's
    // per-seam tracing idiom.
    let example = std::fs::read_to_string(root.join("crates/alacer/examples/dpad_navigation.rs"))
        .expect("the M29 example is in the tree");
    assert!(
        example.contains("alacer::dpad_navigation") && example.contains("dispatch_key"),
        "the example walks the grid through the real dispatch on the alacer::dpad_navigation target"
    );

    // The scope ADR exists; the living documents re-agree at M29.
    assert!(
        root.join("docs/adr/ADR-0035-m29-scope.md").exists(),
        "ADR-0035 scopes the directional keys' ride"
    );
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.contains("M0 through M34") && readme.contains("forty-one decision records"),
        "the README's status block and ADR count re-agree at M29"
    );
    let architecture = std::fs::read_to_string(root.join("docs/architecture.md"))
        .expect("the architecture document");
    assert!(
        architecture.contains("41 decision records"),
        "the architecture register re-agrees at 36 decision records"
    );

    // The corpus stands at 62: the arrows ride the published routes —
    // no new criterion case, the coordinated rule's no-growth arm.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 62;"),
        "the m5 suite's corpus pin stands at 62"
    );
}
