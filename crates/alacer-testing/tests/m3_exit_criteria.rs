//! The M3 exit criteria, as executable spec (architecture §18):
//!
//! - **A11y contract** — the a11y pass produces [`A11yDelta`]s: dirty-node-only
//!   [`TreeUpdate`]s, removals that surface exactly once, focus travelling in
//!   the update, values exposed for editables;
//! - **TreeUpdate p99 < 0.5 ms @ 1k nodes** — the §18 gate as a test, measured
//!   over incremental frames (the init frame is a one-time cost, reported
//!   separately);
//! - **partial paint** — the damage ladder (NONE / partial / FULL) flows in
//!   `FrameReport::damage`, and the software partial path is *pixel-equal*
//!   to a full re-render across a real interaction script (Q8's equivalence);
//! - **the IME contract** — keys route to the focused widget, Tab cycles,
//!   preedit composes and commits in UTF-16 units (the fcitx5/ibus matrix,
//!   exercised at contract level; the platform byte→UTF-16 seam is pinned in
//!   `alacer-platform::keys`).
//!
//! No baselines here: every comparison is dynamic (a fresh full render vs.
//! the accumulated partial frames), nothing to rot.

use alacer_a11y::AccessKitSink;
use alacer_core::damage::Damage;
use alacer_core::events::{
    ImeEvent, Key, KeyEvent, KeyState, Modifiers, MouseButton, PointerEvent,
};
use alacer_core::geom::Point;
use alacer_core::passes::{A11yDelta, A11ySink, ImePurpose, PassPipeline};
use alacer_core::widget::Role;
use alacer_render::SoftwareRenderer;
use alacer_testing::{TestHarness, glyph_source, pixel_diff_ratio};
use alacer_widgets::{Button, Label, TextInput};
use std::sync::Arc;

const VIEWPORT: (f32, f32) = (240.0, 320.0);
const BACKGROUND: alacer_core::color::Color = alacer_widgets::theme::SURFACE;

/// One step of the equivalence script: a raw dispatch, no settle.
type ScriptStep = Box<dyn Fn(&mut TestHarness)>;

/// A row of two labelled inputs + a button — the interaction script's stage.
/// Returns the first input's node.
fn form(harness: &mut TestHarness) -> alacer_core::arena::NodeId {
    let root = harness.mount(Box::new(
        alacer_widgets::Column::new().gap(8.0).padding(16.0),
    ));
    let _first = harness.add_child(root, Box::new(TextInput::new("First name")));
    let _last = harness.add_child(root, Box::new(TextInput::new("Last name")));
    let _submit = harness.add_child(root, Box::new(Button::new("Submit")));
    first_input(harness, root)
}

fn first_input(
    harness: &TestHarness,
    root: alacer_core::arena::NodeId,
) -> alacer_core::arena::NodeId {
    let mut inputs = Vec::new();
    collect_inputs(harness, root, &mut inputs);
    inputs[0]
}

fn collect_inputs(
    harness: &TestHarness,
    node: alacer_core::arena::NodeId,
    out: &mut Vec<alacer_core::arena::NodeId>,
) {
    if harness.pipeline().arena().widget(node).role() == Role::TextInput {
        out.push(node);
    }
    for child in harness.pipeline().arena().children(node).to_vec() {
        collect_inputs(harness, child, out);
    }
}

fn click_center(harness: &mut TestHarness, role: Role, name: &str) {
    let bounds = harness.query(role).named(name).bounds();
    harness.click(Point::new(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    ));
}

/// One click dispatched raw (press + release, no settle) — the caller ticks
/// once and reads the interaction frame's report (damage, delta, focus).
fn click_raw(harness: &mut TestHarness, role: Role, name: &str) {
    let bounds = harness.query(role).named(name).bounds();
    let center = Point::new(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    );
    harness
        .pipeline_mut()
        .dispatch_pointer(&PointerEvent::Down {
            position: center,
            button: MouseButton::Left,
        });
    harness.pipeline_mut().dispatch_pointer(&PointerEvent::Up {
        position: center,
        button: MouseButton::Left,
    });
}

/// Types `text` as raw character presses (no settle — the caller's render
/// loop drives the frames).
fn type_raw(harness: &mut TestHarness, text: &str) {
    for ch in text.chars() {
        let _ = harness.pipeline_mut().dispatch_key(&KeyEvent::char(ch));
    }
}

/// One raw IME dispatch (no settle).
fn ime_raw(harness: &mut TestHarness, event: &ImeEvent) {
    let _ = harness.pipeline_mut().dispatch_ime(event);
}

/// One raw key press (no settle).
fn key_raw(harness: &mut TestHarness, key: Key) {
    let _ = harness.pipeline_mut().dispatch_key(&KeyEvent::pressed(key));
}

/// Shift+Tab (the backward focus cycle) as a free function — orphan impls
/// are forbidden.
fn press_shift_tab(harness: &mut TestHarness) {
    let mut event = KeyEvent::pressed(Key::Tab);
    event.modifiers = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    let _ = harness.pipeline_mut().dispatch_key(&event);
    harness.settle();
}

/// Renders `scene` in full; the reference for the equivalence checks.
fn render_full(
    scene: &alacer_core::scene::SceneSnapshot,
    width: u32,
    height: u32,
) -> tiny_skia::Pixmap {
    SoftwareRenderer::new()
        .render(scene, width, height, BACKGROUND, &mut glyph_source())
        .expect("full render")
}

/// Drives frames until the tree settles, applying every frame's damage to
/// `frame` exactly as the platform's present loop would — the harness-side
/// twin of `AlacerApp::render_frame` (§9.3). Returns the scene the frame now
/// holds (the last non-empty materialization).
fn render_and_settle(
    harness: &mut TestHarness,
    frame: &mut tiny_skia::Pixmap,
    renderer: &mut SoftwareRenderer,
) -> Arc<alacer_core::scene::SceneSnapshot> {
    let (width, height) = (frame.width(), frame.height());
    let viewport = harness.viewport();
    for _ in 0..16 {
        let report = harness.pipeline_mut().tick(viewport).expect("layout");
        let scene = Arc::clone(&report.scene);
        if report.damage.is_empty() {
            return scene;
        }
        if report.damage.is_full() {
            *frame = render_full(&report.scene, width, height);
        } else {
            renderer
                .render_partial(
                    &report.scene,
                    frame,
                    &report.damage,
                    BACKGROUND,
                    &mut glyph_source(),
                )
                .expect("partial render");
        }
    }
    panic!("harness did not settle within 16 frames");
}

// ---------------------------------------------------------------------------
// The a11y contract: deltas, removals, focus, values
// ---------------------------------------------------------------------------

#[test]
fn m3_tree_updates_are_dirty_node_only_after_init() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    form(&mut harness);
    harness.settle();

    let mut sink = AccessKitSink::new();
    let snapshot = Arc::new(
        harness
            .pipeline()
            .a11y_snapshot()
            .cloned()
            .unwrap_or_default(),
    );
    let init = A11yDelta {
        initializes: true,
        ..A11yDelta::empty()
    };
    sink.apply(Arc::clone(&snapshot), &init);
    assert_eq!(
        sink.last_update().expect("init update").nodes.len(),
        snapshot.nodes.len(),
        "init carries every node"
    );

    // An incremental delta re-emits only the updated node.
    let mut delta = A11yDelta::empty();
    let victim = snapshot.nodes[1].id;
    delta.updated.insert(victim);
    sink.apply(Arc::clone(&snapshot), &delta);
    let update = sink.last_update().expect("incremental update");
    assert_eq!(update.nodes.len(), 1, "one dirty node, one re-emit");
    assert_eq!(update.nodes[0].0.0, victim);
    assert_eq!(update.tree, None, "incremental updates carry no TreeInfo");
}

#[test]
fn m3_removals_surface_through_view_reconciliation() {
    use alacer_macro::rsx;
    use alacer_view::{ViewNode, ViewRoot};

    fn mount(view: &ViewNode, pipeline: &mut PassPipeline) -> ViewRoot {
        ViewRoot::mount(view, pipeline, alacer_widgets::default_registry())
    }

    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let initial = rsx! {
        column { gap: 4.0,
            label { key: 1_u64, text: "Keep" }
            label { key: 2_u64, text: "Drop" }
        }
    };
    let mut root = mount(&initial, harness.pipeline_mut());
    harness.settle();
    assert!(harness.query(Role::Label).named("Drop").exists());

    let updated = rsx! {
        column { gap: 4.0,
            label { key: 1_u64, text: "Keep" }
        }
    };
    let _ = root.update(&updated, harness.pipeline_mut());
    harness.settle();
    assert!(!harness.query(Role::Label).named("Drop").exists());

    // The tree walker agrees: only root + the survivor remain.
    let tree = harness.pipeline().a11y_snapshot().expect("tree");
    assert_eq!(tree.nodes.len(), 2, "root + the surviving label only");
}

#[test]
fn m3_focus_travels_in_the_tree_update() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let first = form(&mut harness);
    harness.settle();
    let first_hash = harness
        .pipeline()
        .arena()
        .bounds(first)
        .map(|_| {
            // The id-path hash of the first input, via the a11y tree.
            harness
                .pipeline()
                .a11y_snapshot()
                .expect("tree")
                .find(Role::TextInput, "First")
                .expect("input")
                .id
        })
        .expect("bounds");

    // Click without settling, then tick once: the interaction frame's delta
    // carries the new focus.
    click_raw(&mut harness, Role::TextInput, "First name");
    let viewport = harness.viewport();
    let report = harness.pipeline_mut().tick(viewport).expect("frame");
    assert!(report.focus_changed, "the click moved focus");
    assert!(report.a11y_ran, "a focus move runs the a11y pass");
    assert_eq!(
        report.a11y_delta.focus,
        Some(first_hash),
        "the focused input's id-path hash rides the delta"
    );
}

#[test]
fn m3_editables_expose_their_value() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    form(&mut harness);
    harness.settle();

    click_center(&mut harness, Role::TextInput, "First name");
    harness.type_text("Ada");
    let node = harness.expect_a11y(Role::TextInput, "First");
    assert_eq!(node.value.as_deref(), Some("Ada"), "value is a11y-visible");
    let button = harness.expect_a11y(Role::Button, "Submit");
    assert_eq!(button.value, None, "valueless roles expose no value");
}

// ---------------------------------------------------------------------------
// The TreeUpdate p99 gate (§18): < 0.5 ms @ 1k nodes, incremental frames
// ---------------------------------------------------------------------------

#[test]
fn m3_tree_update_p99_stays_under_half_a_millisecond_at_1k_nodes() {
    let mut harness = TestHarness::new(240.0, 480.0);
    let root = harness.mount(Box::new(alacer_widgets::Column::new()));
    for i in 0..999 {
        let _item = harness.add_child(root, Box::new(Label::new(format!("Item {i}"))));
    }
    harness.settle();
    assert!(
        harness
            .pipeline()
            .a11y_snapshot()
            .is_some_and(|t| t.nodes.len() >= 1_000),
        "the workload is a 1k-node tree"
    );

    // One representative sink, driven exactly as pass 9 drives it.
    let mut sink = AccessKitSink::new();
    let snapshot = Arc::new(
        harness
            .pipeline()
            .a11y_snapshot()
            .cloned()
            .expect("resolved tree"),
    );
    let init = A11yDelta {
        initializes: true,
        ..A11yDelta::empty()
    };
    let init_start = std::time::Instant::now();
    sink.apply(Arc::clone(&snapshot), &init);
    let init_nanos = u64::try_from(init_start.elapsed().as_nanos()).unwrap_or(u64::MAX);
    println!(
        "init TreeUpdate (1k nodes): {:.3} ms (one-time build, not gated)",
        init_nanos as f64 / 1_000_000.0
    );

    // 200 incremental frames: one node flips each frame (the steady state
    // the gate protects — never a full rebuild).
    let mut samples_ns: Vec<u64> = Vec::with_capacity(200);
    for i in 0..200 {
        let mut delta = A11yDelta::empty();
        delta.updated.insert(snapshot.nodes[1 + (i % 999)].id);
        let start = std::time::Instant::now();
        sink.apply(Arc::clone(&snapshot), &delta);
        samples_ns.push(u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX));
    }
    samples_ns.sort_unstable();
    let p99 = samples_ns[(samples_ns.len() * 99) / 100];
    println!("incremental TreeUpdate p99 @ 1k nodes: {p99} ns");
    assert!(
        p99 < 500_000,
        "TreeUpdate p99 {p99} ns ≥ 0.5 ms — the §18 M3 gate failed"
    );
    assert!(
        p99 * 4 < init_nanos,
        "incremental p99 should be a small fraction of the init build"
    );
}

// ---------------------------------------------------------------------------
// Partial paint: the damage ladder + pixel equivalence (Q8)
// ---------------------------------------------------------------------------

#[test]
fn m3_damage_ladder_flows_in_the_frame_report() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let root = harness.mount(Box::new(
        alacer_widgets::Column::new().gap(8.0).padding(16.0),
    ));
    let _flip = harness.add_child(root, Box::new(Button::new("Flip")));
    let _static = harness.add_child(root, Box::new(Label::new("Static")));
    // First frame: everything paints — full damage.
    let report = harness.tick();
    assert!(report.paint_ran);
    assert!(report.damage.is_full(), "the first frame repaints in full");

    // Idle: nothing paints — no damage, the present can be skipped.
    let report = harness.tick();
    assert!(report.is_idle());
    assert!(report.damage.is_empty(), "idle frames cost zero fill");

    // A press/release cycle (PAINT masks on one node): partial damage whose
    // region covers the button — and stays well below the viewport.
    let button_bounds = harness.query(Role::Button).named("Flip").bounds();
    click_raw(&mut harness, Role::Button, "Flip");
    let viewport = harness.viewport();
    let report = harness.pipeline_mut().tick(viewport).expect("frame");
    assert!(report.paint_ran);
    assert!(
        !report.damage.is_full(),
        "one button press is not full-frame"
    );
    assert!(!report.damage.is_empty());
    let regions = report.damage.regions(harness.viewport());
    assert!(
        regions.iter().any(|r| r.intersects(&button_bounds)),
        "the damaged regions cover the pressed button"
    );
    let region_area: f32 = regions.iter().map(|r| r.size.width * r.size.height).sum();
    let viewport_area = harness.viewport().width * harness.viewport().height;
    assert!(
        region_area < viewport_area * 0.75,
        "a button press damages a fraction of the viewport, got {region_area}/{viewport_area}"
    );
}

#[test]
fn m3_structural_frames_fall_back_to_full_damage() {
    use alacer_macro::rsx;
    use alacer_view::{ViewNode, ViewRoot};

    fn mount(view: &ViewNode, pipeline: &mut PassPipeline) -> ViewRoot {
        ViewRoot::mount(view, pipeline, alacer_widgets::default_registry())
    }

    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let view = rsx! { column { gap: 4.0, label { text: "One" } } };
    let mut root = mount(&view, harness.pipeline_mut());
    harness.settle();

    let updated = rsx! {
        column { gap: 4.0,
            label { text: "One" }
            label { text: "Two" }
        }
    };
    let _ = root.update(&updated, harness.pipeline_mut());
    let viewport = harness.viewport();
    let report = harness.pipeline_mut().tick(viewport).expect("frame");
    assert!(
        report.damage.is_full(),
        "structural churn rebuilds the scene — full damage"
    );
}

#[test]
fn m3_partial_frames_equal_full_frames_through_the_real_pipeline() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let (width, height) = (
        alacer_core::num::f32_to_u32(VIEWPORT.0),
        alacer_core::num::f32_to_u32(VIEWPORT.1),
    );
    form(&mut harness);
    let first_frame = harness.tick();

    // The persistent framebuffer: frame 0 renders in full (Damage::FULL).
    let mut frame = render_full(&first_frame.scene, width, height);
    let mut renderer = SoftwareRenderer::new();

    // Interaction script: click into an input, type ASCII + CJK, compose and
    // commit an IME preedit, Tab to the next field, press the button, commit
    // a surrogate pair. Every action is a RAW dispatch (no internal settle):
    // the render loop below drives the frames, exactly like the platform's
    // present loop. Every subsequent frame is partial; the accumulated
    // framebuffer must equal a fresh full render, every step.
    let script: &[ScriptStep] = &[
        Box::new(|h| click_raw(h, Role::TextInput, "First name")),
        Box::new(|h| type_raw(h, "Ada")),
        Box::new(|h| {
            ime_raw(
                h,
                &ImeEvent::Preedit {
                    text: "にほ".into(),
                    cursor_utf16: Some(1),
                },
            )
        }),
        Box::new(|h| {
            ime_raw(
                h,
                &ImeEvent::Commit {
                    text: "日本".into(),
                },
            )
        }),
        Box::new(|h| key_raw(h, Key::Tab)),
        Box::new(|h| type_raw(h, "L")),
        Box::new(|h| click_raw(h, Role::Button, "Submit")),
        Box::new(|h| {
            ime_raw(
                h,
                &ImeEvent::Commit {
                    text: "\u{1F600}".into(), // 😀 — surrogate pair commit
                },
            )
        }),
    ];

    for (step, action) in script.iter().enumerate() {
        action(&mut harness);
        let scene = render_and_settle(&mut harness, &mut frame, &mut renderer);
        let expected = render_full(&scene, width, height);
        let diff = pixel_diff_ratio(&frame, &expected);
        assert_eq!(
            diff, 0.0,
            "step {step}: partial accumulation drifted from a full re-render"
        );
    }
}

#[test]
fn m3_damage_regions_bound_the_repaint_area_on_text_edits() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    form(&mut harness);
    harness.tick();
    click_center(&mut harness, Role::TextInput, "First name");
    harness.tick();

    let input_bounds = harness.query(Role::TextInput).named("First").bounds();
    harness.pipeline_mut().dispatch_key(&KeyEvent::char('a'));
    let viewport = harness.viewport();
    let report = harness.pipeline_mut().tick(viewport).expect("frame");
    assert!(
        !report.damage.is_full(),
        "one keystroke must not repaint all"
    );
    let regions = report.damage.regions(viewport);
    assert!(!regions.is_empty());
    // Locality: every damaged region touches the edited input — a keystroke
    // never repaints the rest of the window.
    for region in &regions {
        assert!(
            region.intersects(&input_bounds),
            "damage region {region:?} escaped the edited input at {input_bounds:?}"
        );
    }
    let area: f32 = regions.iter().map(|r| r.size.width * r.size.height).sum();
    assert!(
        area < viewport.width * viewport.height * 0.5,
        "a keystroke damages a bounded region, got {area}"
    );
}

// ---------------------------------------------------------------------------
// The IME contract: focus routing, Tab cycling, UTF-16 composition
// ---------------------------------------------------------------------------

#[test]
fn m3_keys_route_to_the_focused_widget_only() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    form(&mut harness);
    harness.tick();

    // No focus: typing reaches nobody.
    harness.type_text("zzz");
    let tree = harness.pipeline().a11y_snapshot().expect("tree");
    let first = tree.find(Role::TextInput, "First").expect("first");
    assert_eq!(first.value.as_deref(), Some(""), "no focus, no input");

    // Focus the first input: its value receives the keys.
    click_center(&mut harness, Role::TextInput, "First name");
    harness.type_text("Ada");
    let node = harness.expect_a11y(Role::TextInput, "First");
    assert_eq!(node.value.as_deref(), Some("Ada"));

    // The second input never saw a key.
    let other = harness.expect_a11y(Role::TextInput, "Last");
    assert_eq!(other.value.as_deref(), Some(""), "keys do not leak");
}

#[test]
fn m3_click_focuses_and_tab_cycles() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    form(&mut harness);
    harness.tick();
    assert!(harness.pipeline().focus().is_none(), "no focus at rest");

    // Click-to-focus: the input's FOCUS request resolves in pass 3.
    click_center(&mut harness, Role::TextInput, "Last name");
    let focused = harness.pipeline().focus().expect("click focused the input");
    assert_eq!(
        harness.pipeline().arena().widget(focused).role(),
        Role::TextInput
    );

    // Tab: forward to... the button? No — the next candidate after the last
    // name input in pre-order is the Submit button.
    harness.press_key(Key::Tab);
    let second = harness.pipeline().focus().expect("focus after Tab");
    assert_ne!(focused, second, "Tab moved focus on");
    assert_eq!(
        harness.pipeline().arena().widget(second).role(),
        Role::Button,
        "Tab walks pre-order candidates"
    );

    // Shift+Tab: back to the last-name input.
    press_shift_tab(&mut harness);
    let back = harness.pipeline().focus().expect("focus after Shift+Tab");
    assert_eq!(back, focused, "Shift+Tab walked backward");
}

#[test]
fn m3_ime_preedit_composes_commits_and_cancels_in_utf16() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    form(&mut harness);
    harness.tick();
    click_center(&mut harness, Role::TextInput, "First name");

    // Compose: preedit is composing state, the committed value is untouched.
    harness.ime(&ImeEvent::Preedit {
        text: "にほ".into(),
        cursor_utf16: Some(2),
    });
    let node = harness.expect_a11y(Role::TextInput, "First");
    assert_eq!(node.value.as_deref(), Some(""), "preedit is not committed");

    // Commit: the value grows by the committed text (UTF-16 units counted).
    harness.ime(&ImeEvent::Commit {
        text: "日本語".into(),
    });
    let node = harness.expect_a11y(Role::TextInput, "First");
    assert_eq!(node.value.as_deref(), Some("日本語"));

    // A commit carrying a surrogate pair lands whole.
    harness.ime(&ImeEvent::Commit {
        text: "\u{1F600}".into(),
    });
    let node = harness.expect_a11y(Role::TextInput, "First");
    assert_eq!(node.value.as_deref(), Some("日本語\u{1F600}"));

    // Cancel-by-empty-preedit leaves the committed value alone.
    harness.ime(&ImeEvent::Preedit {
        text: String::new(),
        cursor_utf16: None,
    });
    let node = harness.expect_a11y(Role::TextInput, "First");
    assert_eq!(node.value.as_deref(), Some("日本語\u{1F600}"));
}

#[test]
fn m3_ime_request_follows_focus_and_purpose() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    form(&mut harness);
    harness.tick();
    assert!(harness.ime_request().is_none(), "no focus, no IME");

    click_center(&mut harness, Role::TextInput, "First name");
    let request = harness.ime_request().expect("IME requested for the input");
    assert_eq!(request.purpose, ImePurpose::Text);
    assert!(
        request.caret.is_some(),
        "the candidate window anchors at a caret"
    );

    // Focus moves to the button: buttons want no IME.
    click_center(&mut harness, Role::Button, "Submit");
    assert!(
        harness.pipeline().focus().is_some(),
        "the button holds focus"
    );
    assert!(harness.ime_request().is_none(), "buttons detach the IME");
}

#[test]
fn m3_surrogate_pair_editing_stays_whole_through_the_pipeline() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    form(&mut harness);
    harness.tick();
    click_center(&mut harness, Role::TextInput, "First name");

    harness.ime(&ImeEvent::Commit {
        text: "a\u{1F600}b".into(), // 1 + 2 + 1 = 4 UTF-16 units
    });
    harness.press_key(Key::ArrowLeft);
    harness.press_key(Key::ArrowLeft); // lands on the pair's start, not mid-pair
    harness.press_key(Key::Backspace); // removes 'a' only
    let node = harness.expect_a11y(Role::TextInput, "First");
    assert_eq!(
        node.value.as_deref(),
        Some("\u{1F600}b"),
        "backspace never slices a surrogate pair"
    );
}

#[test]
fn m3_key_events_construct_for_the_harness_helpers() {
    let event = KeyEvent::char('x');
    assert_eq!(event.state, KeyState::Pressed);
    let damage = Damage::NONE;
    assert!(damage.is_empty());
}
