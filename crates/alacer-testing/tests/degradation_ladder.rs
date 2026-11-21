//! The degradation ladder (§14.4): what a frame over budget gives up, and
//! what it never gives up. The thesis is "Sustained speed, not sprint
//! speed" (A7) — budgets are contracts, not aspirations — so a frame that
//! blows its budget must degrade in the ladder's documented order instead
//! of spiraling into lag: the input batch's remainder coalesces into the
//! next frame (rung 1), a behind-schedule paint presents one full frame
//! instead of paying region bookkeeping (rung 3), and a frame past the
//! halfway line defers the a11y pass's *dirty updates* one frame — their
//! masks re-queue, the next frame lands the delta, and no fact is lost.
//! Focus moves, announcements, first frames, and structural churn never
//! defer (§14.4's own contract: latency may degrade, correctness may
//! not).
//!
//! The ladder is armed only where a budget is installed
//! ([`alacer_core::passes::PassPipeline::set_frame_budget`]): the default
//! pipeline never degrades, so every shipped shape is byte-for-byte the
//! pre-ladder tick's. These beats install a small budget and let a
//! deliberately slow paint widget blow it deterministically — the sleep
//! dominates every machine the suite runs on.

use alacer_core::passes::{DegradationStep, PassKind};
use alacer_core::scene::PaintCtx;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_core::{Change, NodeId};
use alacer_testing::TestHarness;
use alacer_widgets::{Column, Label};
use std::time::Duration;

/// The budget-blower: sleeps inside its paint, then answers with one
/// filled quad over its own box — five milliseconds of paint work against
/// a two-millisecond frame budget. The overrun is the widget's own
/// contract, not a machine's mood, and the quad keeps the node's paint
/// extent real (its damage rides the §9.3 ladder like any widget's).
struct SlowPaint {
    millis: u64,
}

impl SlowPaint {
    fn new(millis: u64) -> Self {
        Self { millis }
    }
}

impl Widget for SlowPaint {
    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("Slow paint")
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        std::thread::sleep(Duration::from_millis(self.millis));
        ctx.push_quad(alacer_core::scene::Quad::filled(
            alacer_core::geom::Rect::new(alacer_core::geom::Point::new(0.0, 0.0), ctx.size()),
            alacer_core::color::Color::new(0x40, 0x80, 0xC0, 0xFF),
        ));
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Mounts a column with the slow widget and a worded label, settles the
/// first frame (the ladder disarmed), and returns the nodes.
fn mount_overrunnable() -> (TestHarness, NodeId, NodeId) {
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let slow = harness.add_child(root, Box::new(SlowPaint::new(5)) as Box<dyn Widget>);
    let label = harness.add_child(root, Box::new(Label::new("First")) as Box<dyn Widget>);
    harness.settle();
    (harness, slow, label)
}

/// Installs the two-millisecond budget (the pass threshold is its
/// halfway line: one millisecond) and requests one write batch — the
/// slow widget's repaint and the label's text change — so the frame's
/// paint blows the budget while the a11y pass is due.
fn arm_and_dirty(harness: &mut TestHarness, slow: NodeId, label: NodeId) {
    harness
        .pipeline_mut()
        .set_frame_budget(Some(Duration::from_millis(2)));
    let mut guard = harness.pipeline_mut().arena_mut().mutate(slow);
    guard.request(Change::PAINT);
    drop(guard);
    let mut guard = harness.pipeline_mut().arena_mut().mutate(label);
    guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Label>()
        .expect("the label survives the lease")
        .set_text("Second");
    guard.request(Change::LAYOUT.union(Change::PAINT).union(Change::A11Y));
    drop(guard);
}

/// Beat 1: the budget-blowing frame defers the a11y pass and says so —
/// the report names [`DegradationStep::SkipA11y`], the pass never ran,
/// the paint did, and nothing panicked or hung. The next frame lands the
/// deferred delta (the masks re-queued) and reports [`DegradationStep::
/// None`] again: one frame of latency, never a lost fact.
#[test]
fn the_overrun_frame_defers_the_a11y_pass_and_the_next_frame_lands_it() {
    let _lock = alacer_core::flux::batch_test_lock();
    let (mut harness, slow, label) = mount_overrunnable();
    arm_and_dirty(&mut harness, slow, label);

    let report = harness.tick();
    assert_eq!(
        report.degradation_step,
        DegradationStep::SkipA11y,
        "the frame past the halfway line names the rung it took"
    );
    assert!(
        !report.a11y_ran,
        "the deferred pass did not run — the ledger stays honest"
    );
    assert!(
        report.paint_ran,
        "the paint that blew the budget still painted — the frame is complete, the delta is not"
    );
    let run = report
        .pass(PassKind::Accessibility)
        .expect("the a11y pass rides the ledger");
    assert!(
        !run.ran,
        "the ledger entry records the skip, not a phantom run"
    );

    // The deferral's own contract: the re-queued masks land the delta on
    // the next frame, and the ladder reports a clean frame again.
    let next = harness.tick();
    assert_eq!(
        next.degradation_step,
        DegradationStep::None,
        "the recovery frame fits its budget — no rung fires"
    );
    assert!(
        next.a11y_ran,
        "the re-queued masks make the a11y pass due — the delta lands one frame late, whole"
    );
    let _ = harness.expect_a11y(Role::Label, "Second");
}

/// Beat 2: the facts §14.4 walls never defer. A structural frame that
/// also blows the budget runs every pass anyway — structural churn
/// re-initializes the a11y tree, the paint rebuilds in full (the §9.3
/// damage ladder's own verdict), and the ladder reports no rung: nothing
/// was given up, the frame was merely late.
#[test]
fn structural_a11y_never_defers_under_overrun() {
    let _lock = alacer_core::flux::batch_test_lock();
    let (mut harness, slow, label) = mount_overrunnable();
    harness
        .pipeline_mut()
        .set_frame_budget(Some(Duration::from_millis(2)));
    // The write batch rides a structural mask: the rename under
    // STRUCTURE re-initializes the a11y tree, and the slow widget's
    // repaint is the budget blow — a structural frame rebuilds every
    // layer, the paint pass sleeps inside it.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(label);
    guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Label>()
        .expect("the label survives the lease")
        .set_text("Renamed");
    guard.request(
        Change::STRUCTURE
            .union(Change::LAYOUT)
            .union(Change::PAINT)
            .union(Change::A11Y),
    );
    drop(guard);
    let mut guard = harness.pipeline_mut().arena_mut().mutate(slow);
    guard.request(Change::PAINT);
    drop(guard);

    let report = harness.tick();
    assert_eq!(
        report.degradation_step,
        DegradationStep::None,
        "nothing was given up — the structural frame ran every pass and was merely late"
    );
    assert!(
        report.a11y_ran,
        "structural a11y never defers — §14.4's correctness wall holds under load"
    );
    assert!(
        report.damage.is_full(),
        "the structural frame's §9.3 verdict is already the full present"
    );
    let _ = harness.expect_a11y(Role::Label, "Renamed");
}

/// Beat 2b: the paint rung. A partial frame whose paint pass blows the
/// budget's halfway line collapses its merged regions to one full-frame
/// present (§14.4 rung 3) — the scene is unchanged, the verdict widens,
/// and the a11y pass (never due on a paint-only frame) rides untouched.
#[test]
fn the_paint_overrun_takes_the_full_frame_rung() {
    let _lock = alacer_core::flux::batch_test_lock();
    let (mut harness, slow, _label) = mount_overrunnable();
    harness
        .pipeline_mut()
        .set_frame_budget(Some(Duration::from_millis(2)));
    // Paint-only masks: the frame repaints the slow widget's region and
    // sleeps inside it — a partial frame (no structural churn, no
    // layout, no a11y due) past the halfway line.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(slow);
    guard.request(Change::PAINT);
    drop(guard);

    let report = harness.tick();
    assert_eq!(
        report.degradation_step,
        DegradationStep::ForceFullPaint,
        "the paint overrun takes rung 3 — the partial regions collapse to one full present"
    );
    assert!(
        report.damage.is_full(),
        "the widened present is the full-frame verdict the backend blits"
    );
    assert!(
        !report.a11y_ran,
        "a paint-only frame owes no a11y delta — the pass was never due, not deferred"
    );
}

/// Beat 3: the input rung. A storm the frame cannot afford routes its
/// head and keeps the remainder queued — the next frames' drains route
/// it whole (§14.4 rung 1: coalesce the flux work into the next frame).
/// Never a dropped event: a postponed one.
#[test]
fn the_input_storm_the_frame_cannot_afford_postpones_its_remainder() {
    let _lock = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let _root = harness.mount(Column::new().gap(8.0).boxed());
    harness.settle();
    harness
        .pipeline_mut()
        .set_frame_budget(Some(Duration::from_millis(2)));

    // Down/up pairs never coalesce (only moves do) — twenty thousand
    // events are twenty thousand dispatches, far past the input share's
    // half-millisecond line on any machine the suite meets.
    for index in 0..10_000_u32 {
        let x = alacer_core::num::u32_to_f32(index % 200);
        harness.queue_pointer(alacer_core::events::PointerEvent::Down {
            position: alacer_core::geom::Point::new(x, 10.0),
            button: alacer_core::events::MouseButton::Left,
            click_count: 1,
        });
        harness.queue_pointer(alacer_core::events::PointerEvent::Up {
            position: alacer_core::geom::Point::new(x, 10.0),
            button: alacer_core::events::MouseButton::Left,
        });
    }

    let report = harness.tick();
    assert_eq!(
        report.degradation_step,
        DegradationStep::DropInput,
        "the frame behind its input share names the rung — the batch's remainder stays queued"
    );
    let routed = harness.counters().events_routed;
    assert!(
        routed < 20_000,
        "the storm routed only its head — {routed} of 20 000 before the budget said stop"
    );

    // The postponed remainder routes on the following drains: the queue
    // drains to empty, nothing lost, no panic, no hang. Each frame
    // routes at least one slice before the budget says stop again.
    for _ in 0..4_000 {
        harness.tick();
        if harness.counters().events_routed >= 20_000 {
            break;
        }
    }
    assert!(
        harness.counters().events_routed >= 20_000,
        "every postponed event routed — the queue reached empty (routed {})",
        harness.counters().events_routed
    );
}
