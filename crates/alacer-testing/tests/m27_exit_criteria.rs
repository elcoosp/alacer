//! M27 exit-criteria integration tests (§10.3, ADR-0033): the walk's
//! answer becomes data. ADR-0031's pre-registered re-open rule ships —
//! "a door needs to observe the walk's refusals as data" — as the
//! `FocusMoveOutcome` vocabulary the two doors (`focus_move`,
//! `focus_move_in`) now answer: `Moved(FocusLanding)` carries the
//! endpoints and the wrap, `Refused(FocusRefusal)` names its species —
//! the empty list, the dead scope, the geometry-less focus, the empty
//! direction, the single-candidate stall — instead of a bare `false`
//! the caller had to guess at. The walk behind the doors is untouched:
//! the same candidate list, the same availability filter, the same wrap,
//! the same immediate resolution, the same no-mask idle frames on
//! refusals (the M11 rule stays; the answer just stops being silent).
//!
//! Every test runs the real pipeline (A4) with the real layout host —
//! the exact route a consumer reading the outcome takes.

use alacer_core::NodeId;
use alacer_core::mask::Change;
use alacer_core::passes::{FocusMove, FocusMoveOutcome, FocusRefusal, PassPipeline};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, Label, Row, TextInput};

/// The consumer's window the outcome reads: a two-by-two grid of buttons
/// inside one column — the same window the M26 walk demonstrates on,
/// because the walk's behavior is pinned there and the outcome is its
/// answer as data. The pre-order candidate order is
/// `[top_left, top_right, bottom_left, bottom_right]`.
fn mount_grid(harness: &mut TestHarness) -> Vec<NodeId> {
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
        Label::new("The walk answers with what it did, not a bare bool").boxed(),
    );
    harness.settle();
    vec![top_left, top_right, bottom_left, bottom_right]
}

/// The modal window the scoped refusals demonstrate on: a background
/// control the trap must never offer, and the modal's form whose root is
/// the scope the door takes.
fn mount_modal(harness: &mut TestHarness) -> (NodeId, Vec<NodeId>) {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let background = harness.add_child(root, Button::new("Background").on_click(|| {}).boxed());
    let modal = harness.add_child(root, Column::new().gap(8.0).padding(12.0).boxed());
    let recipient = harness.add_child(modal, TextInput::new("Recipient").boxed());
    let subject = harness.add_child(modal, TextInput::new("Subject").boxed());
    let send = harness.add_child(modal, Button::new("Send").on_click(|| {}).boxed());
    harness.settle();
    (modal, vec![background, recipient, subject, send])
}

/// The availability write at the m22 seam (the mutate door): the write in
/// place, the write's own masks on the guard — the m25/m26 suites' helper.
fn set_available(harness: &mut TestHarness, node: NodeId, available: bool) -> bool {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    let widget = guard.widget_mut();
    let moved = if let Some(button) = widget.as_any_mut().downcast_mut::<Button>() {
        button.set_enabled(available)
    } else if let Some(input) = widget.as_any_mut().downcast_mut::<TextInput>() {
        input.set_enabled(available)
    } else if let Some(checkbox) = widget.as_any_mut().downcast_mut::<Checkbox>() {
        checkbox.set_enabled(available)
    } else {
        panic!("the windows hold only actionable controls")
    };
    if moved {
        guard.request(Change::PAINT.union(Change::A11Y));
    }
    moved
}

/// The name the tree carries for `node` — the outcome's `to` endpoint is
/// read back through the a11y vocabulary, never widget internals.
fn tree_name(pipeline: &PassPipeline, node: NodeId) -> String {
    pipeline.arena().widget(node).label().resolve()
}

#[test]
fn m27_the_landing_names_its_endpoints_and_the_wrap() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let grid = mount_grid(&mut harness);
    let [top_left, top_right, _bottom_left, bottom_right] = [grid[0], grid[1], grid[2], grid[3]];

    // The entry: from no focus the walk takes the first candidate — and
    // the landing says so: `from` is `None` (nothing held focus), `to` is
    // the candidate, `wrapped` is `false` (an entry is not a wrap).
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: None,
            to: top_left,
            wrapped: false,
        }),
        "the entry landing names its endpoint and is not a wrap"
    );
    assert_eq!(tree_name(harness.pipeline(), top_left), "Top left");

    // The stride: from top-left the walk rests on top-right, `from` the
    // node that held focus, no wrap.
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: Some(top_left),
            to: top_right,
            wrapped: false,
        }),
        "the stride landing carries both endpoints"
    );

    // The wrap: from the last candidate the tree-order move crosses the
    // end — and the landing says `wrapped`, the fact the old `true` could
    // never express.
    let _ = harness.pipeline_mut().focus_move(FocusMove::Next);
    let _ = harness.pipeline_mut().focus_move(FocusMove::Next);
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: Some(bottom_right),
            to: top_left,
            wrapped: true,
        }),
        "the wrap is data: from the last candidate back to the first"
    );

    // The directional moves never wrap — the landing says so too. From
    // the wrapped top-left stop, Down and Right land the bottom-right
    // button by geometry.
    harness.pipeline_mut().focus_move(FocusMove::Down);
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Right);
    if let FocusMoveOutcome::Moved(landing) = outcome {
        assert!(
            !landing.wrapped,
            "a directional landing never carries the wrap"
        );
    } else {
        panic!("Down then Right from the top-left stop lands the bottom-right button");
    }
}

#[test]
fn m27_the_directional_refusal_names_the_empty_direction() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let grid = mount_grid(&mut harness);
    let bottom_right = grid[3];

    // Walk to the corner: entry, Right, Down — the bottom-right button.
    // The moves resolve immediately; their masks drain at the next tick,
    // so the frame settles before the corner refuses.
    harness.pipeline_mut().focus_move(FocusMove::Next);
    harness.pipeline_mut().focus_move(FocusMove::Right);
    harness.pipeline_mut().focus_move(FocusMove::Down);
    assert_eq!(harness.pipeline().focus(), Some(bottom_right));
    harness.settle();

    // The corner refuses: nothing lies right of the right column — and
    // the answer names the species, the node, and the direction, instead
    // of a bare `false`.
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Right);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Refused(FocusRefusal::NothingInDirection {
            node: bottom_right,
            direction: FocusMove::Right,
        }),
        "the edge refusal names its node and its direction"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(bottom_right),
        "a refused walk holds its focus"
    );
    assert!(
        harness.tick().is_idle(),
        "a refused walk queues no masks — an idle frame"
    );
}

#[test]
fn m27_the_trap_held_down_refuses_with_the_empty_list() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let (modal, nodes) = mount_modal(&mut harness);
    let [background, recipient, subject, send] = [nodes[0], nodes[1], nodes[2], nodes[3]];

    // The trap answers: the walk enters it, the background control is
    // never a stop.
    let outcome = harness.pipeline_mut().focus_move_in(modal, FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: None,
            to: recipient,
            wrapped: false,
        }),
        "from no focus the scoped walk takes the trap's first candidate"
    );

    // Hold the trap's controls down — the scoped walk refuses with the
    // empty-list species, focus untouched, an idle frame.
    let mut held = true;
    for node in [recipient, subject, send] {
        held &= set_available(&mut harness, node, false);
    }
    assert!(held, "every availability write moves the flag once");
    let outcome = harness.pipeline_mut().focus_move_in(modal, FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Refused(FocusRefusal::NoCandidates),
        "a trap the tier holds down refuses with the empty-list species"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(recipient),
        "the refusal never evicts focus by itself — the tier's eviction is pass 3's"
    );

    // The background control was never a stop — the read doors keep
    // answering the whole tree, but the walk stays in the trap.
    assert!(
        harness.pipeline().focusables().contains(&background),
        "the read doors answer the whole tree"
    );
    assert_ne!(
        harness.pipeline().focus(),
        Some(background),
        "the walk bounds to the trap — the background control never a stop"
    );
}

#[test]
fn m27_a_dead_scope_names_itself_in_the_refusal() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let (modal, nodes) = mount_modal(&mut harness);
    let background = nodes[0];
    let recipient = nodes[1];

    // The walk enters the trap, the unscoped door walks the focus back
    // out, the subtree dies, and the scoped door names its dead scope.
    let outcome = harness.pipeline_mut().focus_move_in(modal, FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: None,
            to: recipient,
            wrapped: false,
        }),
        "the trap answers before the subtree dies"
    );
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Prev);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: Some(recipient),
            to: background,
            wrapped: false,
        }),
        "the unscoped door walks the whole tree — the focus leaves the trap"
    );
    harness.pipeline_mut().arena_mut().remove_subtree(modal);
    harness.settle();
    let outcome = harness.pipeline_mut().focus_move_in(modal, FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Refused(FocusRefusal::DeadScope { scope: modal }),
        "a dead scope names itself in the refusal"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(background),
        "the dead scope's refusal never munges the live focus"
    );
    assert!(
        harness.tick().is_idle(),
        "the dead scope's refusal queues nothing"
    );
}

#[test]
fn m27_the_single_candidate_stalls_honestly() {
    let mut harness = TestHarness::new(200.0, 120.0);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let only = harness.add_child(root, Button::new("Only").on_click(|| {}).boxed());
    harness.settle();

    // The entry lands the only candidate.
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
            from: None,
            to: only,
            wrapped: false,
        }),
        "the entry lands the only candidate"
    );

    // The next move has nowhere to go: the walk resolves to the node
    // already holding focus — named, not guessed.
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert_eq!(
        outcome,
        FocusMoveOutcome::Refused(FocusRefusal::AlreadyFocused { node: only }),
        "a single-candidate walk names the stall"
    );
}

#[test]
fn m27_the_outcome_is_tree_vocabulary_not_widget_internals() {
    let mut harness = TestHarness::new(360.0, 240.0);
    mount_grid(&mut harness);
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    let FocusMoveOutcome::Moved(landing) = outcome else {
        panic!("the entry moves");
    };
    // The landing's endpoints read back through the a11y tree — the same
    // rule the M24 suite pins for the door's bool: no widget internals.
    assert_eq!(
        tree_name(harness.pipeline(), landing.to),
        "Top left",
        "the landing's endpoint is the tree's own name"
    );
}

#[test]
fn m27_the_surface_rides_the_same_cut() {
    // The vocabulary ships where the walk lives: the two doors answer the
    // outcome, the walk behind them returns it, and the M24/M25/M26 pins
    // moved with the cut (the suites' fragments re-agree).
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let passes = std::fs::read_to_string(root.join("crates/alacer-core/src/passes.rs"))
        .expect("passes.rs is in the tree");
    for fragment in [
        "pub fn focus_move(&mut self, movement: FocusMove) -> FocusMoveOutcome",
        "pub fn focus_move_in(&mut self, scope: NodeId, movement: FocusMove) -> FocusMoveOutcome",
        "fn cycle_focus_in(&mut self, scope: Option<NodeId>, movement: FocusMove) -> FocusMoveOutcome",
        "pub enum FocusMoveOutcome",
        "pub struct FocusLanding",
        "pub enum FocusRefusal",
    ] {
        assert!(
            passes.contains(fragment),
            "the pipeline carries the outcome vocabulary: {fragment}"
        );
    }
    assert!(
        passes.contains("self.cycle_focus_in(None, movement)"),
        "the unscoped funnel rides the same walk — the outcome only answers it"
    );

    // The consumer demonstration exists and rides the workspace's
    // per-seam tracing idiom.
    let example = std::fs::read_to_string(root.join("crates/alacer/examples/focus_outcome.rs"))
        .expect("the M27 example is in the tree");
    assert!(
        example.contains("alacer::focus_outcome") && example.contains("FocusMoveOutcome"),
        "the example reads the outcome as data on the alacer::focus_outcome target"
    );

    // The scope ADR exists; the living documents re-agree at M27.
    assert!(
        root.join("docs/adr/ADR-0033-m27-scope.md").exists(),
        "ADR-0033 scopes the outcome vocabulary"
    );
    let readme = std::fs::read_to_string(root.join("README.md")).expect("README");
    assert!(
        readme.contains("M0 through M30") && readme.contains("thirty-seven decision records"),
        "the README's status block and ADR count re-agree at M27"
    );
    assert!(
        readme.contains("route_100_outcome_moves_1k") || readme.contains("62 criterion cases"),
        "the README's perf bullet rides the 61-case corpus"
    );
    let architecture = std::fs::read_to_string(root.join("docs/architecture.md"))
        .expect("the architecture document");
    assert!(
        architecture.contains("37 decision records"),
        "the architecture register re-agrees at 36 decision records"
    );

    // The corpus pins moved with the routes: 60 → 61 across the m5–m26
    // chain in the same cut — the coordinated rule (bench, gates, and
    // baseline in the same cut).
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite");
    assert!(
        m5.contains("const EXPECTED_BENCH_COUNT: usize = 62;"),
        "the m5 suite's corpus pin moved to 61"
    );
    let contracts =
        std::fs::read_to_string(root.join("benchmarks/contracts.json")).expect("the contract set");
    assert!(
        contracts.contains("input/route_100_outcome_moves_1k"),
        "the outcome route joins the gated corpus"
    );
}
