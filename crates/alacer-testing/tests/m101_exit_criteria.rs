//! M101 exit-criteria integration tests (§8.2 item 4, ADR-0105): the
//! scroll position tier. Since M50 (ADR-0056) the scroll doors resolve
//! offsets and clamp them against declared spans, and since M89
//! (ADR-0093) the two-axis container keeps both stories — but the a11y
//! tree never said where a container *was*: the AT read the axis word
//! (M98, ADR-0102) and the walls the actions answered, never the
//! position and extent the doors landed — the declared-but-unreported
//! gap, the M94 pattern at the scroll position tier, cashed:
//! [`alacer_core::Widget::scroll_position`] beside the state the doors
//! already move, the snapshot carrying it (`TreeNode::scroll_position`),
//! and the sink mapping AccessKit's `ScrollY` / `ScrollYMin` /
//! `ScrollYMax` properties (their `ScrollX` halves beside them on the
//! two-axis container) — the word every platform AT turns into
//! "scrolled 200 of 0..600".
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the doors that pay the a11y currency are the only
//! movers the tree follows (the wheel and the AT's asks merge
//! `PAINT | A11Y` on the container; the app's own handle rides the
//! next paid pass), and the absence is the honest one.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role, ScrollPosition, ScrollUnit};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, List};

/// The suite's corpus pin: the coordinated rule's count at the M101 cut —
/// the scroll position route's case joining the secure route's case
/// (M100), the numeric route's case (M99), the orientation route's case
/// (M98), the range route's case (M97), the placeholder route's pair
/// (M96), the description route's case (M95), the toggle route's pair
/// (M94), the derive route's pair (M92), and the recording route's pair
/// (M93) on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 137;

/// The suite's own read door into the published corpus: the scroll
/// position route's case — the 1,000-node scrolled list arena (the
/// coordinated rule's bench half, ADR-0105's exit criteria). The
/// ratio's honest denominator is the same 1,000-node list arena
/// carrying only the axis word (`orientation/snapshot_1k_vertical`,
/// the M98 case), the M95 pricing rule: the same corpus measuring what
/// the tier adds.
const SCROLLED_CASE: &str = "scroll_position/snapshot_1k_scrolled";
const AXIS_CASE: &str = "orientation/snapshot_1k_vertical";

/// The first scroll area in the snapshot whose name contains `name`.
fn list_node(harness: &TestHarness, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::ScrollArea, name).cloned())
}

#[test]
fn m101_the_scrolled_container_names_its_position() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The scrolled container: content declared, the AT's page scroll
    // landing mid-range — the position the doors already resolve, the
    // tree saying it at last.
    let list = List::new(200.0);
    let handle = list.handle();
    handle.set_content_height(800.0);
    let _ = harness.add_child(root, list.boxed());
    // The bare container beside it: content never declared — the honest
    // absence, the tree reading exactly as it read before the tier.
    let bare = List::new(120.0);
    let _ = harness.add_child(root, bare.boxed());
    harness.settle();

    let scrolled = list_node(&harness, "list").expect("the scrolled list rides the snapshot");
    assert_eq!(
        scrolled.scroll_position,
        Some(ScrollPosition {
            vertical: Some(alacer_core::widget::ScrollAxisPosition {
                offset: 0.0,
                max: 600.0,
            }),
            horizontal: None,
        }),
        "the top of a declared span is a position — the extent the walls hold"
    );

    // The AT's ask: one page (the viewport, 200 px) rides the same
    // funnel every scroll door answers by — and the position rides the
    // re-announcement the door's own currency pays for.
    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollDown(ScrollUnit::Page)
        ),
        "the container answers the AT's unit scroll"
    );
    let moved = list_node(&harness, "list").expect("the scrolled list survives");
    assert_eq!(
        moved.scroll_position,
        Some(ScrollPosition {
            vertical: Some(alacer_core::widget::ScrollAxisPosition {
                offset: 200.0,
                max: 600.0,
            }),
            horizontal: None,
        }),
        "the position the door landed is the position the tree carries"
    );

    // The bare container beside it: the honest absence — the tree reads
    // exactly as it read before the tier.
    let named: Vec<TreeNode> = harness
        .pipeline()
        .a11y_snapshot()
        .map(|tree| {
            tree.find_all(Role::ScrollArea, "list")
                .into_iter()
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let bare_node = named
        .iter()
        .find(|node| node.scroll_position.is_none())
        .expect("the bare container rides the snapshot beside the scrolled one");
    assert!(
        bare_node.scroll_position.is_none(),
        "a container that declared no span announces no position, never a guessed zero"
    );
    harness.assert_all_named();
}

#[test]
fn m101_the_wheels_landing_and_wall_ride_the_tree() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    let list = List::new(200.0);
    let handle = list.handle();
    handle.set_content_height(800.0);
    let _ = harness.add_child(root, list.boxed());
    harness.settle();

    // The wheel's landing: 160 px down — the run the platform prices,
    // the position the wheel door's own PAINT | A11Y currency
    // re-announces. (The wheel rides the hit test — the m89 canon's
    // (100, 30) point, the one the translated viewport keeps under the
    // pointer after the landing.)
    harness.wheel(alacer_core::geom::Point::new(100.0, 30.0), (0.0, -160.0));
    let landed = list_node(&harness, "list").expect("the list rides the snapshot");
    let position = landed
        .scroll_position
        .expect("the wheel's landing is a position");
    let vertical = position.vertical.expect("the vertical half rides");
    assert_eq!(vertical.offset, 160.0, "the landing the wheel resolved");
    assert_eq!(vertical.max, 600.0, "the extent the declared span holds");

    // The wall: the slam past the end clamps at the span — the
    // position the tree carries is the wall the clamp pinned.
    harness.wheel(alacer_core::geom::Point::new(100.0, 30.0), (0.0, -10_000.0));
    let pinned = list_node(&harness, "list").expect("the list survives");
    let wall = pinned.scroll_position.expect("the wall is a position too");
    let wall_vertical = wall.vertical.expect("the vertical half rides");
    assert_eq!(
        wall_vertical.offset, wall_vertical.max,
        "the clamp's landing is the span's own edge — the tree says where it stopped"
    );
}

#[test]
fn m101_the_two_axis_container_names_both_halves() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    // The two-axis container (M89, ADR-0093): both spans declared, both
    // halves named — the x story beside the y story on one node.
    let two_axis = List::two_axis(150.0, 120.0);
    let handle = two_axis.handle();
    handle.set_content_height(600.0);
    handle.set_content_width(450.0);
    let _ = harness.add_child(root, two_axis.boxed());
    // The vertical-only container beside it: the x half rides the
    // absence — the undeclared span is no scroll, never a guessed zero.
    let vertical_only = List::new(120.0);
    let vertical_handle = vertical_only.handle();
    vertical_handle.set_content_height(360.0);
    let _ = harness.add_child(root, vertical_only.boxed());
    harness.settle();

    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollDown(ScrollUnit::Page)
        ),
        "the y half answers"
    );
    assert!(
        harness.act(
            Role::ScrollArea,
            "list",
            A11yAction::ScrollRight(ScrollUnit::Page)
        ),
        "the x half answers (M91, ADR-0095)"
    );
    let both = list_node(&harness, "list").expect("the two-axis list rides the snapshot");
    assert_eq!(
        both.scroll_position,
        Some(ScrollPosition {
            vertical: Some(alacer_core::widget::ScrollAxisPosition {
                offset: 120.0,
                max: 480.0,
            }),
            horizontal: Some(alacer_core::widget::ScrollAxisPosition {
                offset: 150.0,
                max: 300.0,
            }),
        }),
        "both halves ride one node — the two-axis answer beside the axis word"
    );

    // The vertical-only container's x half: the honest absence, the y
    // half named beside it.
    let named: Vec<TreeNode> = harness
        .pipeline()
        .a11y_snapshot()
        .map(|tree| {
            tree.find_all(Role::ScrollArea, "list")
                .into_iter()
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let single = named
        .iter()
        .find(|node| {
            matches!(
                &node.scroll_position,
                Some(position) if position.horizontal.is_none()
            )
        })
        .expect("the vertical-only list rides the snapshot");
    let position = single.scroll_position.expect("the y half rides");
    assert!(
        position.horizontal.is_none(),
        "an undeclared x span is the absence, never a guessed zero"
    );
    harness.assert_all_named();
}

#[test]
fn m101_the_absence_is_honest_where_nothing_scrolls() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    // Content that fits the viewport: the span is empty — no scroll
    // exists, so no position exists (the same wall every door answers
    // `no_scrollable_content` with).
    let fits = List::new(200.0);
    let handle = fits.handle();
    handle.set_content_height(100.0);
    let _ = harness.add_child(root, fits.boxed());
    // A non-container: a button answers nothing — the trait door's
    // default is the honest absence.
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    let fits_node = list_node(&harness, "list").expect("the fitting list rides the snapshot");
    assert!(
        fits_node.scroll_position.is_none(),
        "a container whose content fits the viewport carries no position"
    );
    let button = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Button, "Save").cloned())
        .expect("the button rides the snapshot");
    assert!(
        button.scroll_position.is_none(),
        "a non-container inherits the absence"
    );
    harness.assert_all_named();
}

#[test]
fn m101_the_corpus_grew_by_the_coordinated_rule() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let baselines = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/baselines/baselines.json"))
            .expect("the published baseline is readable"),
    )
    .expect("the published baseline is valid JSON");
    let contracts = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/contracts.json"))
            .expect("the contract set is readable"),
    )
    .expect("the contract set is valid JSON");

    // The published corpus carries the scroll position route's case:
    // the 1,000-node scrolled list arena — the position's cost the
    // measurement, priced against the same arena carrying only the
    // axis word.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(SCROLLED_CASE) && benches.contains_key(AXIS_CASE),
        "the scroll position route's case and its denominator are published ({SCROLLED_CASE}, {AXIS_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M101 cut"
    );

    // The contract set gates both cases and prices the position's
    // budget — a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [SCROLLED_CASE, AXIS_CASE] {
        let gate = gates
            .iter()
            .find(|gate| gate["baseline"].as_str() == Some(case))
            .unwrap_or_else(|| panic!("the case carries a regression gate ({case})"));
        assert_eq!(
            gate["scope"].as_str(),
            Some("same-host"),
            "the gate is same-host scoped like the tier's own"
        );
    }
    let budgets = contracts["budgets"]
        .as_array()
        .expect("the contract set carries budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"].as_str() == Some("scroll_position_snapshot_overhead"))
        .expect("the scroll position budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(SCROLLED_CASE),
        "the budget prices the scrolled route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(AXIS_CASE),
        "the budget prices it against the same arena, axis word only"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the position must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M101 cut"
    );
}
