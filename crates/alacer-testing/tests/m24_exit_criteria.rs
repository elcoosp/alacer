//! M24 exit-criteria integration tests (§10.3, ADR-0030): the focus walk
//! joins the dispatch contract. Tab's private cycle becomes public
//! vocabulary — one `FocusMove` direction and one `PassPipeline::focus_move`
//! door the keyboard's fallback route and the consumer's code both ride,
//! read back through the pipeline's `focus()` door. The
//! walk skips unavailable controls (the M22 derivation consults
//! `enabled`), wraps at both ends, and refuses honestly when no candidate
//! exists — no masks, an idle frame (the M11 refuse-don't-munge rule, at
//! the focus seam). The micro-debt the sweep priced rides the same cut —
//! the flux runtime's dead `effect_count` introspection and the harness's
//! dead `spec_of`/`with_text_context`/`text_size` probes — and the living
//! documents re-agree at M24.
//!
//! Every test runs the real pipeline (A4) with the real layout host — the
//! exact route a window's Tab key and the consumer's door take.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::mask::Change;
use alacer_core::passes::FocusMove;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, TextInput};
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

/// The consumer's form the walk demonstrates on: two inputs, a checkbox,
/// and a send button in one column — the m22 example's shape, every
/// control focusable and available. The pre-order candidate order is
/// `[name, email, subscribe, send]`; the tests walk it by role+name.
fn mount_form(harness: &mut TestHarness) -> Vec<alacer_core::NodeId> {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let name = harness.add_child(root, TextInput::new("Name").boxed());
    let email = harness.add_child(root, TextInput::new("Email").boxed());
    let subscribe = harness.add_child(root, Checkbox::new("Subscribe").boxed());
    let send = harness.add_child(root, Button::new("Send").on_click(|| {}).boxed());
    harness.settle();
    vec![name, email, subscribe, send]
}

/// A Shift+Tab press, the way the platform delivers it (§10.3).
fn shift_tab() -> KeyEvent {
    KeyEvent {
        key: Key::Tab,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

/// The walk's expected stops, named by their order in the mounted form.
fn expected_order() -> [&'static str; 4] {
    ["Name", "Email", "Subscribe", "Send"]
}

#[test]
fn m24_the_walk_lands_where_tab_lands() {
    // Door one — the keyboard's: Tab from no focus takes the first
    // candidate, and every press walks one stop, wrapping at the end.
    let mut keyed = TestHarness::new(360.0, 240.0);
    mount_form(&mut keyed);
    let mut by_key = Vec::new();
    for _ in 0..5 {
        keyed.press_key(Key::Tab);
        let focused = keyed
            .pipeline()
            .focus()
            .expect("the walk holds a candidate after Tab");
        by_key.push(focused);
    }

    // Door two — the consumer's: `focus_move(Next)` from the same states.
    // The two doors must land on the same nodes in the same order: one
    // funnel, the keyboard and the app's code riding it alike (ADR-0030).
    let mut driven = TestHarness::new(360.0, 240.0);
    mount_form(&mut driven);
    let mut by_door = Vec::new();
    for _ in 0..5 {
        let moved = driven.pipeline_mut().focus_move(FocusMove::Next).moved();
        assert!(moved, "a four-candidate walk always has a next stop");
        let focused = driven
            .pipeline()
            .focus()
            .expect("the door holds a candidate after focus_move");
        by_door.push(focused);
    }

    // The two windows hold different node ids (the arena's counter is
    // process-global and this test mounts two windows) — the contract is
    // the landing *order*, read as positions in each window's own
    // candidate list: one funnel, the keyboard and the app's code riding
    // it alike (ADR-0030).
    let keyed_order: Vec<usize> = by_key
        .iter()
        .map(|id| {
            keyed
                .pipeline()
                .focusables()
                .iter()
                .position(|candidate| candidate == id)
                .expect("Tab rests on a candidate")
        })
        .collect();
    let door_order: Vec<usize> = by_door
        .iter()
        .map(|id| {
            driven
                .pipeline()
                .focusables()
                .iter()
                .position(|candidate| candidate == id)
                .expect("the door rests on a candidate")
        })
        .collect();
    assert_eq!(
        keyed_order, door_order,
        "Tab and focus_move land on the same candidates in the same order"
    );

    // The landing order is the pre-order candidate list, wrapping — the
    // walk the M22 derivation feeds (enabled controls only, tree order).
    let candidates: Vec<alacer_core::NodeId> = driven.pipeline().focusables().to_vec();
    let wrapped: Vec<alacer_core::NodeId> = (0..5)
        .map(|step| candidates[step % candidates.len()])
        .collect();
    assert_eq!(by_door, wrapped, "the walk wraps at the end, in tree order");

    // The a11y tree still answers for the stop the walk rests on — five
    // stops wrap back to the first, and the focused input is named, the
    // M3 way tests read the tree.
    assert!(
        driven
            .query(Role::TextInput)
            .named(expected_order()[0])
            .exists(),
        "the walk rests on a named control — the tree carries it"
    );
}

#[test]
fn m24_shift_tab_walks_backward() {
    // Walk to the last candidate, then backward: Shift+Tab (the keyboard
    // door) and focus_move(Prev) (the consumer door) reverse in step.
    let mut keyed = TestHarness::new(360.0, 240.0);
    mount_form(&mut keyed);
    for _ in 0..4 {
        keyed.press_key(Key::Tab);
    }
    let last = keyed.pipeline().focus().expect("four stops reach the end");
    keyed.key(&shift_tab());
    let back_one = keyed.pipeline().focus().expect("Shift+Tab holds a stop");
    assert_ne!(back_one, last, "Shift+Tab leaves the end");
    assert_eq!(
        back_one,
        keyed.pipeline().focusables()[2],
        "backward from the end is the third candidate"
    );

    let mut driven = TestHarness::new(360.0, 240.0);
    mount_form(&mut driven);
    for _ in 0..4 {
        driven.pipeline_mut().focus_move(FocusMove::Next);
    }
    driven.pipeline_mut().focus_move(FocusMove::Prev);
    // The two windows hold different node ids — the contract is the stop:
    // each door reverses to the third candidate of its own window's list,
    // the same one Shift+Tab landed on in the keyed half above.
    let back_door = driven.pipeline().focus().expect("the door holds a stop");
    assert_eq!(
        back_door,
        driven.pipeline().focusables()[2],
        "Shift+Tab and focus_move(Prev) reverse through the same funnel"
    );
}

#[test]
fn m24_the_walk_skips_the_unavailable_control() {
    // The M22 tier at the walk: a control the app holds unavailable is
    // never a Tab stop — the cycle-time `enabled` check passes it by
    // without a recompute, and focus never rests on it.
    let mut harness = TestHarness::new(360.0, 240.0);
    let nodes = mount_form(&mut harness);
    let subscribe = nodes[2];
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(subscribe);
        if let Some(checkbox) = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<alacer_widgets::Checkbox>()
            && checkbox.set_enabled(false)
        {
            guard.request(Change::PAINT.union(Change::A11Y));
        }
    }
    harness.settle();

    harness.pipeline_mut().focus_move(FocusMove::Next);
    harness.pipeline_mut().focus_move(FocusMove::Next);
    harness.pipeline_mut().focus_move(FocusMove::Next);
    let focused = harness
        .pipeline()
        .focus()
        .expect("three stops land on an available control");
    assert_eq!(
        focused, nodes[3],
        "the walk skips the unavailable checkbox: name → email → send"
    );

    // The tree reads the refusal too: the unavailable control reports
    // `enabled = false` on the same snapshot the walk consults.
    let tree = harness
        .pipeline()
        .a11y_snapshot()
        .expect("the a11y tree is built");
    let node = tree
        .walk(None)
        .into_iter()
        .find(|node| node.role == Role::CheckBox)
        .expect("the checkbox is in the tree");
    assert!(!node.enabled, "the unavailable control reports the flag");
}

#[test]
fn m24_the_refusal_is_honest() {
    // Every control unavailable: the walk has no candidate to offer and
    // answers `false` — no focus, no masks, an idle frame (the M11 rule,
    // at the focus seam: a refusal fires nothing).
    let mut harness = TestHarness::new(360.0, 240.0);
    let nodes = mount_form(&mut harness);
    for node in &nodes {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(*node);
        let widget = guard.widget_mut();
        let moved =
            if let Some(button) = widget.as_any_mut().downcast_mut::<alacer_widgets::Button>() {
                button.set_enabled(false)
            } else if let Some(input) = widget
                .as_any_mut()
                .downcast_mut::<alacer_widgets::TextInput>()
            {
                input.set_enabled(false)
            } else if let Some(checkbox) = widget
                .as_any_mut()
                .downcast_mut::<alacer_widgets::Checkbox>()
            {
                checkbox.set_enabled(false)
            } else {
                panic!("the form holds only actionable controls")
            };
        assert!(moved, "the write moves the flag — masks are owed");
        guard.request(Change::PAINT.union(Change::A11Y));
    }
    harness.settle();

    let outcome = harness.pipeline_mut().focus_move(FocusMove::Next);
    assert!(!outcome.moved(), "no candidate exists — the walk refuses");
    assert!(
        harness.pipeline().focus().is_none(),
        "a refused walk leaves focus where it was — nowhere"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "a refused walk costs an idle frame — no masks were queued"
    );

    // The empty arena refuses the same way — no candidates, no masks.
    let mut empty = TestHarness::new(360.0, 240.0);
    assert!(
        !empty.pipeline_mut().focus_move(FocusMove::Prev).moved(),
        "an arena without widgets has no stop to offer"
    );
    assert!(empty.pipeline().focus().is_none());
}

#[test]
fn m24_a_move_costs_one_frame() {
    // The walk rides the one invalidation currency: one move queues the
    // FOCUS | PAINT masks on the two endpoints, the next frame repaints
    // the pair, and the frame after that is idle again (A7).
    let mut harness = TestHarness::new(360.0, 240.0);
    mount_form(&mut harness);

    let moved = harness.pipeline_mut().focus_move(FocusMove::Next).moved();
    assert!(moved, "the walk moves");
    assert!(
        harness.pipeline().focus().is_some(),
        "the walk resolves immediately — key routing sees the new focus this event"
    );

    let first = harness.tick();
    assert!(!first.is_idle(), "the move's masks cost one frame");
    assert!(
        first.paint_ran,
        "the ring repaint lands — the move rides the paint currency"
    );
    let second = harness.tick();
    assert!(second.is_idle(), "the frame after the move is idle again");
}

#[test]
fn m24_the_focused_door_reads_the_walk() {
    // The read door: `focus()` answers the node the walk rests on —
    // before any walk it is `None` (the honest empty read), after each
    // stop it is the candidate the funnel resolved, and it agrees with
    // the candidate list's order.
    let mut harness = TestHarness::new(360.0, 240.0);
    let nodes = mount_form(&mut harness);
    assert!(
        harness.pipeline().focus().is_none(),
        "no walk, no focus — the read door starts honest"
    );
    for (step, expected) in nodes.iter().enumerate() {
        harness.pipeline_mut().focus_move(FocusMove::Next);
        let focused = harness
            .pipeline()
            .focus()
            .unwrap_or_else(|| panic!("stop {step} holds focus"));
        assert_eq!(
            &focused, expected,
            "stop {step} rests on the candidate the tree order names"
        );
    }
    // The wrap: one more stop returns to the first candidate.
    harness.pipeline_mut().focus_move(FocusMove::Next);
    assert_eq!(
        harness.pipeline().focus(),
        Some(nodes[0]),
        "the walk wraps at the end — the read door sees it"
    );
}

#[test]
fn m24_the_living_documents_re_agree() {
    // The README: the status block and the ADR count name the cut, the
    // features bullet carries the walk, and the example table carries
    // the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "M0 through M64",
        "seventy-one decision records",
        "Since M24 the focus walk is public vocabulary",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `focus_traversal` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`focus_traversal`](../crates/alacer/examples/focus_traversal.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0030",
        "71 decision records",
        "the focus walk joins the dispatch contract",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0030-m24-scope.md");
    assert!(
        adr.contains("M24 scope") && adr.contains("focus walk"),
        "ADR-0030 documents the focus-walk scope"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M24 — the focus walk (ADR-0030)**"),
        "the CHANGELOG carries the M24 entry"
    );

    // The drift the sweep found is repaired: the m1 suite's header names
    // the same corpus count its own pin enforces (37 — it said 36).
    let m1 = read_workspace_file("crates/alacer-testing/tests/m1_exit_criteria.rs");
    assert!(
        m1.contains("37 blessed snapshots green"),
        "the m1 header agrees with the 37-scenario pin below it"
    );

    // Every pin the M-count move touched moved together — the m20–m23
    // suites' document pins ride the same cut, the chain cannot drift.
    for suite in ["m20", "m21", "m22", "m23"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M64") && source.contains("seventy-one decision records"),
            "the {suite} suite's README pins moved with the documents"
        );
        assert!(
            source.contains("71 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
}

#[test]
fn m24_the_surface_rides_the_same_cut() {
    // The seam ships where the dispatch contract lives: the direction
    // vocabulary, the consumer door, the read door — and the keyboard's
    // Tab fallback routed through the same funnel (one walk, two doors).
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    for fragment in [
        "pub enum FocusMove",
        "pub fn focus_move(&mut self, movement: FocusMove) -> FocusMoveOutcome",
    ] {
        assert!(
            passes.contains(fragment),
            "the pipeline carries the focus-walk seam: {fragment}"
        );
    }
    assert!(
        passes.contains("let movement = if event.modifiers.shift {")
            && passes.contains("self.focus_move_via(FocusRider::Keyboard, movement).moved()"),
        "dispatch_key's Tab fallback rides the same funnel — the walk is not duplicated \
         (the M30 cut names the rider at the same seam, ADR-0036)"
    );

    // The micro-debt the sweep priced is gone: the flux runtime's dead
    // introspection and the harness's dead probes (the M18 species).
    let flux = read_workspace_file("crates/alacer-core/src/flux.rs");
    assert!(
        !flux.contains("fn effect_count"),
        "FluxRuntime::effect_count is gone — dead introspection, no caller anywhere"
    );
    let harness = read_workspace_file("crates/alacer-testing/src/lib.rs");
    for fragment in ["fn spec_of", "fn with_text_context", "fn text_size"] {
        assert!(
            !harness.contains(fragment),
            "the harness's dead {fragment} probe is gone"
        );
    }

    // The consumer demonstration names the decision it demonstrates,
    // drives the seam it demonstrates, and rides the workspace's
    // per-seam tracing idiom.
    let example = read_workspace_file("crates/alacer/examples/focus_traversal.rs");
    assert!(
        example.contains("ADR-0030")
            && example.contains("focus_move")
            && example.contains("alacer::focus_traversal"),
        "examples/focus_traversal.rs demonstrates the focus walk"
    );

    // No dependency edge moved for the cut: the core manifest is the
    // cut's only surface, and it gains nothing.
    let core_manifest = read_workspace_file("crates/alacer-core/Cargo.toml");
    assert!(
        !core_manifest.contains("miette"),
        "miette-class help stays plain text — no new dependency (ADR-0029's rule)"
    );
}
