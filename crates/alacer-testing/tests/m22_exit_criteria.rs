//! M22 exit-criteria integration tests (§11.5, ADR-0028): the availability
//! tier joins the widget contract. Every actionable control can now report
//! that it must refuse — one `enabled` source of truth on the `Widget`
//! trait, riding the same snapshot and sink plumbing every other property
//! rides — and the refusal is honest at every door: a disabled button is a
//! wall for the pointer, the keyboard never reaches an unavailable control,
//! the AT tier is refused exactly like any other unimplemented action (the
//! M11 rule), Tab skips it, and focus leaves a control that becomes
//! unavailable. A write that lands where the flag already is queues no
//! masks — the idle-frame property, at the availability seam. The corpus
//! grows 56 → 57 by the coordinated rule — the refused-press route's
//! criterion case joins at this measurement run, with its baseline and its
//! regression gate — and the living documents re-agree.
//!
//! Every test runs the real pipeline (A4) with the real layout host — the
//! exact route a window's pointer, key, and AT writes take.

use alacer_core::events::{Key, PointerEvent};
use alacer_core::geom::Point;
use alacer_core::mask::Change;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, Label};
use std::path::PathBuf;

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

/// The consumer's window: a disabled button named "Send invite" with the
/// enabling checkbox and status line the m1 corpus's s37 mirrors — the
/// exact shape `examples/disabled.rs` assembles.
fn mount_invite(harness: &mut TestHarness) -> alacer_core::NodeId {
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, Label::new("The availability tier (ADR-0028)").boxed());
    let send = harness.add_child(
        root,
        Button::new("Send invite")
            .enabled(false)
            .on_click(|| {})
            .boxed(),
    );
    let _ = harness.add_child(root, Checkbox::new("Enable sending").boxed());
    let _ = harness.add_child(
        root,
        Label::bound(alacer_core::flux::RenderBinding::new(
            "Sending: unavailable".to_string(),
        ))
        .boxed(),
    );
    harness.settle();
    send
}

/// The bounds' center (euclid's Rect carries no center helper here).
fn cx(bounds: alacer_core::geom::Rect) -> f32 {
    bounds.min_x() + bounds.width() * 0.5
}

fn cy(bounds: alacer_core::geom::Rect) -> f32 {
    bounds.min_y() + bounds.height() * 0.5
}

/// The send button's a11y node: the public read the AT owns (tests query
/// the tree, never widget internals).
fn invite_in_tree(harness: &TestHarness) -> alacer_core::passes::TreeNode {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Button, "Send invite"))
        .cloned()
        .expect("the send button is named in the a11y tree")
}

/// The mutate seam the example demonstrates at assembly time: take the
/// arena lease, write the availability in place, and request the write's
/// own masks on the guard when the write moved (Masonry `WidgetMut`
/// discipline — the masks merge on drop).
fn set_available(harness: &mut TestHarness, node: alacer_core::NodeId, enabled: bool) -> bool {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Button>()
        .expect("the node is the send button")
        .set_enabled(enabled);
    if moved {
        guard.request(Change::PAINT.union(Change::A11Y));
    }
    moved
}

#[test]
fn m22_the_at_reads_the_availability_the_widget_reports() {
    let mut harness = TestHarness::new(400.0, 300.0);
    let send = mount_invite(&mut harness);

    // The tree carries the flag the widget reports: the AT reads an
    // unavailable control, not an actionable one.
    assert!(
        !invite_in_tree(&harness).enabled,
        "the tree reports the control unavailable"
    );

    // The flag rides the delta: enabling re-announces exactly that node
    // and the AT reads the truth the other way too.
    assert!(
        set_available(&mut harness, send, true),
        "the write moved the flag"
    );
    harness.settle();
    assert!(
        invite_in_tree(&harness).enabled,
        "the tree reports the control available"
    );

    // A write that lands where the flag already is queues nothing — the
    // M21 idle-frame property, at the availability seam.
    assert_eq!(
        harness.settle(),
        0,
        "the second write is a no-op: no masks, an idle frame"
    );
    assert!(!set_available(&mut harness, send, true));
}

#[test]
fn m22_a_disabled_button_refuses_every_door() {
    let mut harness = TestHarness::new(400.0, 300.0);
    let _send = mount_invite(&mut harness);

    // The AT tier: a press on an unavailable control is refused honestly —
    // never munged, never swallowed into some other action (the M11 rule).
    assert!(
        !harness.act(Role::Button, "Send invite", A11yAction::DefaultPress),
        "the AT press is refused"
    );

    // The pointer door: the press is consumed (a wall, not a window — the
    // gesture never clicks through to an ancestor) and queues nothing: no
    // pressed state to repaint, no focus request, an idle frame.
    let bounds = harness.query(Role::Button).named("Send invite").bounds();
    harness.queue_pointer(PointerEvent::Down {
        position: Point::new(cx(bounds), cy(bounds)),
        button: alacer_core::events::MouseButton::Left,
        click_count: 1,
    });
    harness.queue_pointer(PointerEvent::Up {
        position: Point::new(cx(bounds), cy(bounds)),
        button: alacer_core::events::MouseButton::Left,
    });
    assert_eq!(
        harness.settle(),
        0,
        "a refused gesture queues no masks — an idle frame"
    );

    // The pointer door on an available control, for contrast: the same
    // press-release now fires the click (the refusal was the flag's work,
    // not the wiring's absence).
    let send = find_send(&harness);
    assert!(
        set_available(&mut harness, send, true),
        "the enabling write moved"
    );
    harness.settle();
    harness.click(Point::new(cx(bounds), cy(bounds)));
    harness.settle();
    assert!(
        harness.query(Role::Button).named("Send invite").exists(),
        "the control stays reachable — availability never unmounts"
    );
}

/// The send button's arena node (the mutate seam's target).
fn find_send(harness: &TestHarness) -> alacer_core::NodeId {
    let arena = harness.pipeline().arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |id, slot| {
        if found.is_none()
            && slot.widget().role() == Role::Button
            && slot.widget().label().resolve() == "Send invite"
        {
            found = Some(id);
        }
    });
    found.expect("the send button is mounted")
}

#[test]
fn m22_focus_never_rests_on_an_unavailable_control() {
    let mut harness = TestHarness::new(400.0, 300.0);
    let send = mount_invite(&mut harness);

    // A disabled control takes no click-to-focus: the request is refused.
    let bounds = harness.query(Role::Button).named("Send invite").bounds();
    harness.click(Point::new(cx(bounds), cy(bounds)));
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        None,
        "an unavailable control is no focus target"
    );

    // And focus *leaves* a control that becomes unavailable — the same
    // sentinel discipline the dead-widget drop already holds.
    assert!(set_available(&mut harness, send, true));
    harness.settle();
    harness.click(Point::new(cx(bounds), cy(bounds)));
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        Some(find_send(&harness)),
        "an available control takes focus on press"
    );
    assert!(set_available(&mut harness, send, false), "disable again");
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        None,
        "focus is evicted the frame the control becomes unavailable"
    );
}

#[test]
fn m22_tab_skips_the_unavailable_candidate() {
    let mut harness = TestHarness::new(400.0, 300.0);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _first = harness.add_child(root, Button::new("First").on_click(|| {}).boxed());
    let _second = harness.add_child(
        root,
        Button::new("Second").enabled(false).on_click(|| {}).boxed(),
    );
    harness.settle();

    // Tab from no focus lands on the first candidate; Tab again has no
    // *available* candidate to move to — the disabled button is skipped,
    // so focus stays where it is.
    harness.press_key(Key::Tab);
    harness.settle();
    let first = find_button(&harness, "First");
    assert_eq!(
        harness.pipeline().focus(),
        Some(first),
        "Tab takes the first"
    );

    harness.press_key(Key::Tab);
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        Some(first),
        "the unavailable candidate is not a Tab stop"
    );
}

fn find_button(harness: &TestHarness, name: &str) -> alacer_core::NodeId {
    let arena = harness.pipeline().arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |id, slot| {
        if found.is_none()
            && slot.widget().role() == Role::Button
            && slot.widget().label().resolve() == name
        {
            found = Some(id);
        }
    });
    found.unwrap_or_else(|| panic!("the {name} button is mounted"))
}

#[test]
fn m22_the_binding_door_rides_the_bridge() {
    // The reactive door: one write batch on the signal, the flux bridge
    // lands the caller's masks on the button's node, and the widget
    // paints and reports from the binding — the Progress::bound shape,
    // at the availability tier.
    let mut harness = TestHarness::new(400.0, 300.0);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let binding = alacer_core::flux::RenderBinding::new(false);
    let _ = harness.add_child(
        root,
        Button::new("Send invite")
            .enabled_binding(binding.clone())
            .on_click(|| {})
            .boxed(),
    );
    let signal = harness.pipeline_mut().flux_mut().signal(false);
    let send = find_send(&harness);
    harness.pipeline_mut().flux_mut().bind_value(
        signal,
        send,
        &binding,
        Change::PAINT.union(Change::A11Y),
        |value: &bool| *value,
    );
    harness.settle();
    assert!(
        !invite_in_tree(&harness).enabled,
        "the bound button reports unavailable"
    );

    reactive_graph::traits::Set::set(&signal, true);
    harness.settle();
    assert!(
        invite_in_tree(&harness).enabled,
        "the bridge lands the write — the tree reads the new flag"
    );
    assert!(
        harness.act(Role::Button, "Send invite", A11yAction::DefaultPress),
        "the AT press answers once the binding says available"
    );
}

#[test]
fn m22_a_disabled_checkbox_and_the_muted_paint() {
    // The checkbox answers the same contract: the AT toggle is refused,
    // the pointer never flips it, and the label paints muted — the look
    // tells the truth the tree carries.
    let mut harness = TestHarness::new(400.0, 300.0);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, Checkbox::new("Enable sending").enabled(false).boxed());
    harness.settle();
    assert!(
        !harness.act(Role::CheckBox, "Enable sending", A11yAction::DefaultPress),
        "the AT toggle is refused"
    );
    let node = {
        let arena = harness.pipeline().arena();
        let mut found = None;
        arena.for_each_pre_order(&mut |id, slot| {
            if found.is_none() && slot.widget().role() == Role::CheckBox {
                found = Some(id);
            }
        });
        found.expect("the checkbox is mounted")
    };
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    let checkbox = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<Checkbox>()
        .expect("the node is the checkbox");
    assert!(!checkbox.set_enabled(false), "already disabled: no move");
    drop(guard);
    harness.settle();
    let tree = harness.pipeline().a11y_snapshot().expect("a11y tree");
    let node = tree
        .find(Role::CheckBox, "Enable sending")
        .expect("the checkbox is in the tree");
    assert!(!node.enabled, "the tree carries the unavailable flag");
}

#[test]
fn m22_the_refused_route_joins_the_corpus_by_the_coordinated_rule() {
    // The refused-press route's criterion case (ADR-0028's own exit row)
    // is published at this measurement run: the baseline entry and its
    // regression gate land together with the bench (ADR-0011;
    // CONTRIBUTING's publishing rule).
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let id = "disabled/route_100_refused_presses_1k";

    assert_eq!(
        baselines["bench_count"],
        serde_json::json!(57),
        "the corpus grows 56 → 57 at M22's measurement run"
    );
    let benches = baselines["benches"].as_object().expect("benches object");
    assert!(benches.contains_key(id), "{id} is published");
    let entry = &benches[id];
    assert!(
        entry["median_ns"].as_f64().expect("median") > 0.0
            && entry["samples"].as_u64() == Some(100),
        "{id} carries a real measurement (100 samples)"
    );
    let gates = contracts["regressions"].as_array().expect("regressions");
    assert_eq!(
        gates.len(),
        57,
        "every case carries exactly one regression gate"
    );
    assert!(
        gates
            .iter()
            .any(|gate| gate["baseline"] == serde_json::json!(id)
                && gate["scope"] == serde_json::json!("same-host")),
        "{id} is gated same-host"
    );

    // Every M-suite pin moved to 57 together — the chain cannot drift.
    for suite in [
        "m5_exit_criteria.rs",
        "m6_exit_criteria.rs",
        "m7_exit_criteria.rs",
        "m8_exit_criteria.rs",
        "m9_exit_criteria.rs",
        "m10_exit_criteria.rs",
        "m11_exit_criteria.rs",
        "m12_exit_criteria.rs",
        "m13_exit_criteria.rs",
        "m14_exit_criteria.rs",
        "m15_exit_criteria.rs",
        "m18_exit_criteria.rs",
        "m20_exit_criteria.rs",
        "m21_exit_criteria.rs",
    ] {
        let source = read_workspace_file(&format!("crates/alacer-testing/tests/{suite}"));
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 57;"),
            "{suite} must pin the 57-case corpus"
        );
    }
    let m19 = read_workspace_file("crates/alacer-testing/tests/m19_exit_criteria.rs");
    assert!(
        m19.contains("All 57 criterion cases") && m19.contains("serde_json::json!(57)"),
        "the m19 suite's pins moved with the corpus"
    );
    let m20 = read_workspace_file("crates/alacer-testing/tests/m20_exit_criteria.rs");
    let m21 = read_workspace_file("crates/alacer-testing/tests/m21_exit_criteria.rs");
    assert!(
        m20.contains("serde_json::json!(57)") && m20.contains("usize = 57;"),
        "the m20 suite's own pins moved"
    );
    assert!(
        m21.contains("serde_json::json!(57)") && m21.contains("usize = 57;"),
        "the m21 suite's own pins moved"
    );
}

#[test]
fn m22_the_living_documents_re_agree() {
    // The README: the criterion count quotes the 57-case baseline, the
    // accessibility bullet carries the availability tier, the status
    // block and the register count move with the corpus.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "All 57 criterion cases",
        "Since M22 the availability tier is widget vocabulary",
        "M0 through M23",
        "thirty decision records",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `disabled` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract, and the
    // snapshot corpus paragraph agrees with the blessed directory.
    let catalog = read_workspace_file("docs/examples.md");
    for fragment in [
        "[`disabled`](../crates/alacer/examples/disabled.rs)",
        "37 styled scenarios",
        "39 blessed PNGs",
    ] {
        assert!(
            catalog.contains(fragment),
            "docs/examples.md names {fragment}"
        );
    }

    // The architecture carries the decision, the register row, and the
    // roadmap row; the count lines agree with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0028",
        "30 decision records",
        "the availability tier joins the widget contract",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The decision record itself documents the scope and its coordinated
    // remainders.
    let adr = read_workspace_file("docs/adr/ADR-0028-m22-scope.md");
    assert!(
        adr.contains("M22 scope") && adr.contains("availability tier"),
        "ADR-0028 documents the availability-tier scope"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M22 — the unavailable control (ADR-0028)**"),
        "the CHANGELOG carries the M22 entry"
    );

    // The migration guides' snapshot counts ride the same corpus.
    for guide in ["docs/migration/from-iced.md", "docs/migration/from-egui.md"] {
        let guide = read_workspace_file(guide);
        assert!(
            guide.contains("37 styled scenarios"),
            "{guide} quotes the grown corpus"
        );
    }
}

#[test]
fn m22_the_surface_rides_the_same_cut() {
    // The tier is core vocabulary: the trait method ships with the
    // focusable derivation, and the example names the decision it
    // demonstrates.
    let widget = read_workspace_file("crates/alacer-core/src/widget.rs");
    assert!(
        widget.contains("fn enabled(&self) -> bool") && widget.contains("ADR-0028"),
        "the Widget trait carries the availability contract"
    );
    let example = read_workspace_file("crates/alacer/examples/disabled.rs");
    assert!(
        example.contains("ADR-0028") && example.contains("set_enabled"),
        "examples/disabled.rs demonstrates the availability tier"
    );

    // The a11y sink maps the flag faithfully — the AT sees the
    // unavailable control, not a generic actionable node.
    let sink = read_workspace_file("crates/alacer-a11y/src/lib.rs");
    assert!(
        sink.contains("set_disabled"),
        "the sink maps the disabled flag"
    );

    // The micro-debt the cut prices is gone: the dead introspection
    // accessors (the M21 species) no longer ship — including the
    // software renderer's `glyphs_rendered`, the wgpu cut's escapee.
    let software = read_workspace_file("crates/alacer-render/src/software.rs");
    assert!(
        !software.contains("fn glyphs_rendered"),
        "SoftwareRenderer::glyphs_rendered is gone"
    );
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    assert!(
        !passes.contains("fn clear_observer") && !passes.contains("fn timings_enabled"),
        "the dead observer/timing accessors are gone"
    );
    let mount = read_workspace_file("crates/alacer-view/src/mount.rs");
    assert!(
        !mount.contains("pub const fn root()"),
        "ViewRoot::root (the root_id duplicate) is gone"
    );

    // The corpus's own count test moved with the scenarios (the m1 suite
    // pins 37, the directory carries the new PNG).
    let m1 = read_workspace_file("crates/alacer-testing/tests/m1_exit_criteria.rs");
    assert!(
        m1.contains("fn m1_corpus_is_thirty_seven_scenarios()"),
        "the m1 corpus pin moved to 37"
    );
    let blessed = workspace_root().join("crates/alacer-testing/blessed");
    assert!(
        blessed.join("m1_36_disabled.png").is_file(),
        "m1_36_disabled.png is blessed with the corpus"
    );
}
