//! M25 exit-criteria integration tests (§10.3, ADR-0031): the focus walk
//! bounds to a subtree. ADR-0030's re-open rule ships as
//! `PassPipeline::focus_move_in` — the scope is the subtree's root node,
//! the modal's focus trap is the real UI that prices it. The scoped door
//! rides the same walk as Tab and `focus_move` (one funnel, two scopes:
//! the pre-order candidate list, the M22 cycle-time `enabled` filter, the
//! wrap at both ends, the immediate resolution), the read doors
//! (`focus()`, `focusables()`) keep answering the whole tree, and the
//! refusal is honest — a dead scope, an empty trap, or a trap the
//! availability tier holds down answers `false` with no masks (an idle
//! frame, never a first-candidate reset). The micro-debt the sweep priced
//! rides the same cut — the layout mapping's dead `fixed_size_of` and the
//! color's dead `to_f32_rgba` (documented for a GPU-uniform upload that
//! never landed) — and the living documents re-agree at M25 and 0.16.0,
//! the m7 release-state pin among them: its M22-era literal missed two
//! pin sweeps and the suite went red against a coherent tree.
//!
//! Every test runs the real pipeline (A4) with the real layout host — the
//! exact route a modal app's door takes.

use alacer_core::mask::Change;
use alacer_core::passes::FocusMove;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, Label, TextInput};
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

/// The consumer's window the trap demonstrates on: a background control
/// the trap must never offer, and the modal's column — two inputs, a
/// checkbox, a send button — whose root is the scope the door takes. The
/// pre-order candidate order is
/// `[background, recipient, subject, notify, send]`.
fn mount_trap(harness: &mut TestHarness) -> (alacer_core::NodeId, Vec<alacer_core::NodeId>) {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let background = harness.add_child(root, Button::new("Background").on_click(|| {}).boxed());
    let modal = harness.add_child(root, Column::new().gap(8.0).padding(12.0).boxed());
    let recipient = harness.add_child(modal, TextInput::new("Recipient").boxed());
    let subject = harness.add_child(modal, TextInput::new("Subject").boxed());
    let notify = harness.add_child(modal, Checkbox::new("Notify me").boxed());
    let send = harness.add_child(modal, Button::new("Send modal").on_click(|| {}).boxed());
    let _ = harness.add_child(
        root,
        Label::new("Tab still walks the whole window — the trap is the app's door").boxed(),
    );
    harness.settle();
    (modal, vec![background, recipient, subject, notify, send])
}

/// One availability write at the trap: the mutate seam, the write in
/// place, the write's own masks on the guard (the m22/m24 idiom).
fn set_available(harness: &mut TestHarness, node: alacer_core::NodeId, available: bool) -> bool {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    let widget = guard.widget_mut();
    let moved = if let Some(button) = widget.as_any_mut().downcast_mut::<Button>() {
        button.set_enabled(available)
    } else if let Some(input) = widget.as_any_mut().downcast_mut::<TextInput>() {
        input.set_enabled(available)
    } else if let Some(checkbox) = widget.as_any_mut().downcast_mut::<Checkbox>() {
        checkbox.set_enabled(available)
    } else {
        panic!("the trap holds only actionable controls")
    };
    if moved {
        guard.request(Change::PAINT.union(Change::A11Y));
    }
    moved
}

#[test]
fn m25_the_scoped_walk_stays_in_the_trap() {
    // From no focus, the scoped door takes the trap's *first* candidate —
    // never the background control the pre-order list offers first — and
    // every beat cycles inside the modal's subtree, wrapping at the trap's
    // own end.
    let mut harness = TestHarness::new(360.0, 240.0);
    let (modal, nodes) = mount_trap(&mut harness);
    let in_trap = [nodes[1], nodes[2], nodes[3], nodes[4]];

    let mut landings = Vec::new();
    for _ in 0..=in_trap.len() {
        let moved = harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Next)
            .moved();
        assert!(moved, "a four-candidate trap always has a next stop");
        landings.push(
            harness
                .pipeline()
                .focus()
                .expect("the scoped walk resolves immediately"),
        );
    }
    let expected: Vec<alacer_core::NodeId> = (0..=in_trap.len())
        .map(|step| in_trap[step % in_trap.len()])
        .collect();
    assert_eq!(
        landings, expected,
        "the scoped walk cycles the trap's candidates in tree order and wraps inside it"
    );
    assert!(
        !landings.contains(&nodes[0]),
        "the background control is never a stop — the scope bounds the walk"
    );

    // The scope bounds the walk, never the candidate list: the read doors
    // answer the whole tree exactly as before (ADR-0031).
    assert_eq!(
        harness.pipeline().focusables().len(),
        5,
        "focusables() keeps answering the whole tree"
    );
    let backward = {
        let moved = harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Prev)
            .moved();
        assert!(moved, "backward inside the trap always has a stop");
        harness.pipeline().focus()
    };
    assert_eq!(
        backward,
        Some(in_trap[in_trap.len() - 1]),
        "backward from the trap's first candidate wraps to its last — the same wrap rule"
    );
}

#[test]
fn m25_the_scoped_walk_skips_the_unavailable_control() {
    // The M22 tier at the scoped walk: a control the app holds unavailable
    // inside the trap is never a stop — the same cycle-time `enabled`
    // check the unscoped funnel rides, without a recompute.
    let mut harness = TestHarness::new(360.0, 240.0);
    let (modal, nodes) = mount_trap(&mut harness);
    assert!(
        set_available(&mut harness, nodes[4], false),
        "the write moves the flag — masks are owed"
    );
    harness.settle();

    let first = {
        assert!(
            harness
                .pipeline_mut()
                .focus_move_in(modal, FocusMove::Next)
                .moved(),
            "the trap still offers its first available candidate"
        );
        harness.pipeline().focus()
    };
    assert_eq!(first, Some(nodes[1]), "the walk opens on the recipient");
    assert!(
        harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Next)
            .moved(),
        "the second beat moves past the unavailable send"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(nodes[2]),
        "the unavailable send is skipped — recipient then subject"
    );
    assert!(
        harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Next)
            .moved(),
        "the third beat lands the notify checkbox — the trap's last available"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(nodes[3]),
        "the unavailable send is never offered — recipient, subject, notify"
    );
    assert!(
        harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Next)
            .moved(),
        "the wrap skips the unavailable send too"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(nodes[1]),
        "the walk wraps to the trap's first candidate, never the unavailable control"
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
        .find(|node| node.role == Role::Button && node.label.contains("Send modal"))
        .expect("the send button is in the tree");
    assert!(!node.enabled, "the unavailable control reports the flag");
}

#[test]
fn m25_the_refusal_is_honest() {
    // A trap the availability tier holds down answers `false` — no masks,
    // an idle frame — and focus stays wherever the unscoped walk left it
    // (never a first-candidate reset, the M11 rule at the focus seam).
    let mut harness = TestHarness::new(360.0, 240.0);
    let (modal, nodes) = mount_trap(&mut harness);

    // Focus rests on the background control first (the unscoped door).
    assert!(
        harness.pipeline_mut().focus_move(FocusMove::Next).moved(),
        "the unscoped walk takes the first candidate"
    );
    assert_eq!(harness.pipeline().focus(), Some(nodes[0]));

    for node in &nodes[1..] {
        assert!(set_available(&mut harness, *node, false));
    }
    harness.settle();

    let outcome = harness.pipeline_mut().focus_move_in(modal, FocusMove::Next);
    assert!(
        !outcome.moved(),
        "every control in the trap is unavailable — the door refuses"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(nodes[0]),
        "a refused scoped walk leaves focus where it was"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "a refusal costs an idle frame — no masks were queued"
    );

    // A dead scope refuses the same way: remove the modal's subtree, and
    // the door names the refusal on the `alacer::focus` target — plain
    // help text (ADR-0029's rule), no masks, focus untouched.
    harness.pipeline_mut().arena_mut().remove_subtree(modal);
    harness.settle();
    let outcome = harness.pipeline_mut().focus_move_in(modal, FocusMove::Prev);
    assert!(!outcome.moved(), "a dead scope has no walk to offer");
    assert_eq!(
        harness.pipeline().focus(),
        Some(nodes[0]),
        "the dead scope's refusal never munges the live focus"
    );
    let report = harness.tick();
    assert!(report.is_idle(), "the dead scope's refusal queues nothing");
}

#[test]
fn m25_the_scoped_door_agrees_with_the_unscoped_door_at_the_root() {
    // One walk, two scopes: bounded at the tree's root, the scoped door is
    // the unscoped door — same candidates, same order, same wrap (the
    // funnel ADR-0030 shipped is the `None` scope of ADR-0031's walk).
    let mut unscoped = TestHarness::new(360.0, 240.0);
    let (root_a, _) = mount_trap(&mut unscoped);
    let mut by_door = Vec::new();
    for _ in 0..6 {
        assert!(unscoped.pipeline_mut().focus_move(FocusMove::Next).moved());
        by_door.push(unscoped.pipeline().focus());
    }

    let mut scoped = TestHarness::new(360.0, 240.0);
    let _ = mount_trap(&mut scoped);
    // The root scope is the arena's root — the whole tree, which is the
    // unscoped door's candidate list exactly (the modal subtree would
    // bound the walk to the trap, a different contract, pinned above).
    let root_b = scoped
        .pipeline()
        .arena()
        .root()
        .expect("the trap window mounts a root");
    let mut by_scoped = Vec::new();
    for _ in 0..6 {
        assert!(
            scoped
                .pipeline_mut()
                .focus_move_in(root_b, FocusMove::Next)
                .moved()
        );
        by_scoped.push(scoped.pipeline().focus());
    }
    let _ = root_a;
    // The two windows hold different node ids (the arena's counter is
    // process-global and this test mounts two windows) — the contract is
    // the landing *order*, read as positions in each window's own
    // candidate list: one walk, two scopes, the same stops.
    let door_order: Vec<Option<usize>> = by_door
        .iter()
        .map(|landing| {
            landing.map(|id| {
                unscoped
                    .pipeline()
                    .focusables()
                    .iter()
                    .position(|candidate| *candidate == id)
                    .expect("the unscoped walk rests on a candidate")
            })
        })
        .collect();
    let scoped_order: Vec<Option<usize>> = by_scoped
        .iter()
        .map(|landing| {
            landing.map(|id| {
                scoped
                    .pipeline()
                    .focusables()
                    .iter()
                    .position(|candidate| *candidate == id)
                    .expect("the scoped walk rests on a candidate")
            })
        })
        .collect();
    assert_eq!(
        door_order, scoped_order,
        "a root-scoped walk lands exactly where the unscoped door lands"
    );
}

#[test]
fn m25_a_move_costs_one_frame() {
    // The scoped walk rides the one invalidation currency: one move queues
    // the FOCUS | PAINT masks on the endpoints, the next frame repaints
    // the pair, and the frame after is idle again (A7).
    let mut harness = TestHarness::new(360.0, 240.0);
    let (modal, _) = mount_trap(&mut harness);

    assert!(
        harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Next)
            .moved()
    );
    assert!(
        harness.pipeline().focus().is_some(),
        "the scoped walk resolves immediately — key routing sees the landing this event"
    );
    let first = harness.tick();
    assert!(!first.is_idle(), "the move's masks cost one frame");
    assert!(
        first.paint_ran,
        "the ring repaint lands — the paint currency"
    );
    let second = harness.tick();
    assert!(second.is_idle(), "the frame after the move is idle again");
}

#[test]
fn m25_the_living_documents_re_agree() {
    // The README: the badge and status block ride the pending-cut
    // convention (the successor minor under [Unreleased], m7's rule), the
    // count names the cut, the features bullet carries the scope, and the
    // example table carries the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "version-0.17.0",
        "M0 through M57",
        "sixty-four decision records",
        "Since M25 the walk is bounded too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `focus_trap` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`focus_trap`](../crates/alacer/examples/focus_trap.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0031",
        "64 decision records",
        "the focus walk bounds to a subtree",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0031-m25-scope.md");
    assert!(
        adr.contains("M25 scope") && adr.contains("focus scope"),
        "ADR-0031 documents the focus-scope scope"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M25 — the focus scope (ADR-0031)**"),
        "the CHANGELOG carries the M25 entry"
    );

    // The m1 header stands: no corpus growth (the demonstration shows no
    // new visual arrangement — the catalog's own rule).
    let m1 = read_workspace_file("crates/alacer-testing/tests/m1_exit_criteria.rs");
    assert!(
        m1.contains("37 blessed snapshots green"),
        "the m1 header's pin stands — the corpus did not grow"
    );

    // Every pin the M-count move touched moved together — the m20–m24
    // suites ride the same cut, the chain cannot drift.
    for suite in ["m20", "m21", "m22", "m23", "m24"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M57") && source.contains("sixty-four decision records"),
            "the {suite} suite's README pins moved with the documents"
        );
        assert!(
            source.contains("64 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
    for suite in ["m20", "m21", "m23"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("version-0.17.0"),
            "the {suite} suite's badge pin moved with the pending-cut convention"
        );
    }
}

#[test]
fn m25_the_surface_rides_the_same_cut() {
    // The seam ships where the dispatch contract lives: the scoped door
    // beside the unscoped funnel, the walk generalized behind both — and
    // the M24 public surface exactly as pinned (one walk, two scopes; the
    // M26 vocabulary rides the same walk, so the generalized signature
    // names the movement, not a boolean — the M26 sweep moved the pin).
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    for fragment in [
        "pub fn focus_move_in(&mut self, scope: NodeId, movement: FocusMove) -> FocusMoveOutcome",
        "fn cycle_focus_in(&mut self, scope: Option<NodeId>, movement: FocusMove) -> FocusMoveOutcome",
        "pub fn focus_move(&mut self, movement: FocusMove) -> FocusMoveOutcome",
    ] {
        assert!(
            passes.contains(fragment),
            "the pipeline carries the focus-scope seam: {fragment}"
        );
    }
    assert!(
        passes.contains("self.cycle_focus_in(None, movement)"),
        "the unscoped funnel rides the same walk — the scope only bounds it"
    );
    assert!(
        !passes.contains("miette"),
        "miette-class help stays plain text — no new dependency (ADR-0029's rule)"
    );

    // The micro-debt the sweep priced is gone: the layout mapping's dead
    // fixed-size spec and the color's dead normalized conversion (the
    // M24 species — the accessor documented for a consumer that never
    // landed).
    let style_map = read_workspace_file("crates/alacer-layout/src/style_map.rs");
    assert!(
        !style_map.contains("fn fixed_size_of"),
        "style_map::fixed_size_of is gone — dead public API, no caller anywhere"
    );
    let color = read_workspace_file("crates/alacer-core/src/color.rs");
    assert!(
        !color.contains("fn to_f32_rgba"),
        "Color::to_f32_rgba is gone — dead public API, its GPU-uniform consumer never landed"
    );

    // The m7 release-state pin re-joins the chain (inside the surface
    // test below it also carries the milestone literal): the M22-era
    // literal missed the M23/M24 sweeps and the suite went red against a
    // coherent tree — the drift the M25 sweep repairs.
    let m7 = read_workspace_file("crates/alacer-testing/tests/m7_exit_criteria.rs");
    assert!(
        m7.contains("M0 through M57"),
        "the m7 suite's milestone pin moved with the documents"
    );

    // The consumer demonstration names the decision it demonstrates,
    // drives the seam it demonstrates, and rides the workspace's
    // per-seam tracing idiom.
    let example = read_workspace_file("crates/alacer/examples/focus_trap.rs");
    assert!(
        example.contains("ADR-0031")
            && example.contains("focus_move_in")
            && example.contains("alacer::focus_trap"),
        "examples/focus_trap.rs demonstrates the focus scope"
    );

    // No dependency edge moved for the cut: the core manifest is the
    // cut's only surface, and it gains nothing.
    let core_manifest = read_workspace_file("crates/alacer-core/Cargo.toml");
    assert!(
        !core_manifest.contains("miette"),
        "miette-class help stays plain text — no new dependency (ADR-0029's rule)"
    );
}
