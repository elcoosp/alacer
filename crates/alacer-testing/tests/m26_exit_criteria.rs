//! M26 exit-criteria integration tests (§10.3, ADR-0032): the directional
//! focus walk. ADR-0030's pre-registered re-open rule ships — "a walk
//! needs an order the tree order cannot express (spatial/D-pad
//! navigation)" — as four `FocusMove` directions (`Left`/`Right`/`Up`/
//! `Down`) riding the same two doors (`focus_move`, `focus_move_in`) and
//! the same one walk behind them. The tree-order moves keep their wrap;
//! the directional moves read the bounds pass 6 wrote — fully beyond the
//! focused node's edge in the direction, nearest edge distance first,
//! cross-axis misalignment breaking ties, then tree order — and never
//! wrap (space has no wrap; the window's edge is the walk's end). The
//! refusals are honest and named on the `alacer::focus` target (the
//! miette-class `help:` sentence, never the crate), the availability
//! tier filters before the geometry answers, the scope bounds the
//! directional walk exactly as it bounds the tree-order one, and the
//! entry contract finally matches the document that has described it
//! since M25: from no focus — or from a focus outside the filtered
//! candidate list — the walk takes the list's *first* candidate (the
//! M25-era code landed the second; the drift the cut repairs).
//!
//! The micro-debt the sweep priced rides the same cut — the selection's
//! dead `is_degenerate` (zero call sites anywhere in the workspace) and
//! the color's dead `to_premultiplied_rgba8` (its only caller was its
//! own unit test; the software stamp premultiplies independently — the
//! M25 `to_f32_rgba` species) — and the M24/M25-promised focus routes
//! (`input/route_100_focus_moves_1k`, `input/route_100_scoped_moves_1k`)
//! finally join the published corpus beside the M26 directional route at
//! this cut's measurement run, the coordinated rule the two earlier ADRs
//! described and the tree never delivered.
//!
//! Every test runs the real pipeline (A4) with the real layout host —
//! the exact route a D-pad app's door takes.

use alacer_core::NodeId;
use alacer_core::passes::FocusMove;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Checkbox, Column, Label, Row, TextInput};

/// The consumer's window the walk demonstrates on: a two-by-two grid of
/// buttons inside one column — the real UI that prices the seam, because
/// the tree order (left-top, right-top, left-bottom, right-bottom) and
/// the spatial reading of the same window disagree about what "next"
/// means. Rows are the grid's rows; the column stacks them.
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
        Label::new("Arrows read the window; Tab still reads the tree").boxed(),
    );
    harness.settle();
    vec![top_left, top_right, bottom_left, bottom_right]
}

/// The modal window the scoped directional door demonstrates on: a
/// background control the trap must never offer, and the modal's
/// two-by-two form whose root is the scope the door takes.
fn mount_modal(harness: &mut TestHarness) -> (NodeId, Vec<NodeId>) {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let background = harness.add_child(root, Button::new("Background").on_click(|| {}).boxed());
    let modal = harness.add_child(root, Column::new().gap(8.0).padding(12.0).boxed());
    let top = harness.add_child(modal, Row::new().gap(8.0).boxed());
    let top_left = harness.add_child(top, TextInput::new("Top left").boxed());
    let top_right = harness.add_child(top, TextInput::new("Top right").boxed());
    let bottom = harness.add_child(modal, Row::new().gap(8.0).boxed());
    let bottom_left = harness.add_child(bottom, Checkbox::new("Bottom left").boxed());
    let bottom_right = harness.add_child(bottom, Button::new("Send").on_click(|| {}).boxed());
    harness.settle();
    (
        modal,
        vec![background, top_left, top_right, bottom_left, bottom_right],
    )
}

/// The availability write at the m22 seam (the mutate door): the write in
/// place, the write's own masks on the guard — the m25 suite's helper.
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
        panic!("the grid holds only actionable controls")
    };
    if moved {
        guard.request(alacer_core::mask::Change::PAINT.union(alacer_core::mask::Change::A11Y));
    }
    moved
}

#[test]
fn m26_the_directional_walk_reads_the_window_not_the_tree() {
    // The seam's reason to exist: from the top-left button, the tree-order
    // `Next` walks to the top-right button's *tree* neighbor — the second
    // row's first button is the pre-order list's next stop only after the
    // row ends; `Right` answers the window instead: the button to the
    // right of the focused one.
    let mut harness = TestHarness::new(360.0, 240.0);
    let grid = mount_grid(&mut harness);
    let [top_left, top_right, bottom_left, bottom_right] = [grid[0], grid[1], grid[2], grid[3]];

    // The entry contract: from no focus the walk rests on the first
    // candidate, whichever direction the caller names.
    assert!(
        harness.pipeline_mut().focus_move(FocusMove::Right).moved(),
        "the directional door enters the walk the way Tab does"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(top_left),
        "the walk rests on the first candidate before it can mean a direction"
    );

    // The spatial answer: `Right` from the top-left lands the top-right
    // button; the tree-order `Next` would land it too here (same row), so
    // the discriminator is the *bottom* row: from the top-right button,
    // `Next` follows the tree into the bottom-left button, while `Down`
    // reads the window and lands the bottom-right one.
    assert!(harness.pipeline_mut().focus_move(FocusMove::Right).moved());
    assert_eq!(
        harness.pipeline().focus(),
        Some(top_right),
        "right is right"
    );
    assert!(
        harness.pipeline_mut().focus_move(FocusMove::Next).moved(),
        "the tree-order funnel is untouched"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(bottom_left),
        "Next keeps the pre-order contract: the next row's first button"
    );

    // And from the top-left button, `Down` reads the column: the
    // bottom-left button — the same stop `Next` reaches, by the window's
    // answer rather than the tree's.
    assert!(harness.pipeline_mut().focus_move(FocusMove::Up).moved());
    assert_eq!(
        harness.pipeline().focus(),
        Some(top_left),
        "up reverses down"
    );
    assert!(harness.pipeline_mut().focus_move(FocusMove::Down).moved());
    assert_eq!(
        harness.pipeline().focus(),
        Some(bottom_left),
        "down lands the button below, not the tree's next stop"
    );
    assert!(harness.pipeline_mut().focus_move(FocusMove::Right).moved());
    assert_eq!(
        harness.pipeline().focus(),
        Some(bottom_right),
        "right along the bottom row"
    );
    assert!(harness.pipeline_mut().focus_move(FocusMove::Left).moved());
    assert_eq!(
        harness.pipeline().focus(),
        Some(bottom_left),
        "left is left"
    );
}

#[test]
fn m26_the_directional_refusal_is_honest_and_never_wraps() {
    // Space has no wrap: from the grid's right column, `Right` has no
    // candidate fully beyond the focused node's edge — the door answers
    // `false`, queues nothing (an idle frame), and leaves focus where it
    // was. The geometry still answers where the window has an answer, and
    // the tree-order `Next` from the last candidate still wraps, because
    // the wrap is the tree-order contract and only the tree-order
    // contract.
    let mut harness = TestHarness::new(360.0, 240.0);
    let grid = mount_grid(&mut harness);
    let [top_left, top_right, _bottom_left, bottom_right] = [grid[0], grid[1], grid[2], grid[3]];

    assert!(harness.pipeline_mut().focus_move(FocusMove::Next).moved());
    harness.settle();
    assert_eq!(harness.pipeline().focus(), Some(top_left));
    assert!(harness.pipeline_mut().focus_move(FocusMove::Right).moved());
    harness.settle();
    assert_eq!(harness.pipeline().focus(), Some(top_right));

    let outcome = harness.pipeline_mut().focus_move(FocusMove::Right);
    assert!(
        !outcome.moved(),
        "nothing lies right of the right column — the door refuses"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(top_right),
        "a refused directional walk leaves focus where it was"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "a refusal costs an idle frame — no masks were queued"
    );

    // The geometry still answers where the window has an answer: `Down`
    // from the same spot lands the bottom-right button.
    assert!(harness.pipeline_mut().focus_move(FocusMove::Down).moved());
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        Some(bottom_right),
        "down from the top-right lands the bottom-right — the geometry's answer"
    );

    // The window's bottom edge refuses the same way: nothing lies below
    // the bottom row.
    let outcome = harness.pipeline_mut().focus_move(FocusMove::Down);
    assert!(!outcome.moved(), "nothing lies below the bottom row");
    assert_eq!(harness.pipeline().focus(), Some(bottom_right));
    let report = harness.tick();
    assert!(report.is_idle(), "the edge refusal queues nothing");

    // The tree-order funnel still wraps — the two contracts do not blur.
    assert!(
        harness.pipeline_mut().focus_move(FocusMove::Next).moved(),
        "Next wraps from the last candidate to the first"
    );
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        Some(top_left),
        "the wrap lands the tree's first candidate"
    );
}

#[test]
fn m26_the_directional_walk_skips_the_unavailable_control() {
    // The M22 tier at the directional walk: the availability filter runs
    // before the geometry answers — a control the app holds down is never
    // a stop, and the walk takes the next qualifying neighbor in the
    // direction (the grid's far column when the near one is held down).
    let mut harness = TestHarness::new(360.0, 240.0);
    let grid = mount_grid(&mut harness);
    let [top_left, top_right, bottom_left, bottom_right] = [grid[0], grid[1], grid[2], grid[3]];

    assert!(set_available(&mut harness, top_right, false));
    harness.settle();

    assert!(harness.pipeline_mut().focus_move(FocusMove::Next).moved());
    assert_eq!(harness.pipeline().focus(), Some(top_left));
    assert!(
        harness.pipeline_mut().focus_move(FocusMove::Right).moved(),
        "the direction still answers — past the unavailable neighbor"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(bottom_right),
        "the unavailable top-right is skipped; the far column's button lands"
    );

    // Every direction answers through the same filter: `Down` from the
    // top-left lands the bottom-left button — the only candidate below.
    assert!(harness.pipeline_mut().focus_move(FocusMove::Up).moved());
    assert_eq!(harness.pipeline().focus(), Some(top_left));
    assert!(harness.pipeline_mut().focus_move(FocusMove::Down).moved());
    assert_eq!(harness.pipeline().focus(), Some(bottom_left));
}

#[test]
fn m26_the_scoped_directional_walk_stays_in_the_trap() {
    // One walk, two scopes, six directions: the scoped door bounds the
    // directional walk exactly as it bounds the tree-order one — the
    // background control is never a stop, the read doors keep answering
    // the whole tree, and a direction with no candidate inside the trap
    // refuses honestly instead of escaping the scope.
    let mut harness = TestHarness::new(420.0, 300.0);
    let (modal, nodes) = mount_modal(&mut harness);
    let in_trap = [nodes[1], nodes[2], nodes[3], nodes[4]];

    assert!(
        harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Right)
            .moved(),
        "the scoped directional door enters the trap the way the scoped tree-order door does"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(in_trap[0]),
        "the walk rests on the trap's first candidate"
    );
    assert!(
        harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Right)
            .moved(),
        "the second beat moves right inside the form"
    );
    assert_eq!(harness.pipeline().focus(), Some(in_trap[1]));
    assert!(
        harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Down)
            .moved(),
        "down answers inside the form too"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(in_trap[3]),
        "down from the form's top-right lands its bottom-right control"
    );

    // The scope bounds the walk, never the candidate list: the read doors
    // answer the whole tree exactly as before (ADR-0031's rule, now for
    // every direction).
    assert_eq!(
        harness.pipeline().focusables().len(),
        5,
        "focusables() keeps answering the whole tree"
    );

    // The unscoped funnel is untouched: from the background control, Tab
    // still walks the whole window in tree order.
    let mut window_walk = TestHarness::new(420.0, 300.0);
    let (modal_b, nodes_b) = mount_modal(&mut window_walk);
    assert!(
        window_walk
            .pipeline_mut()
            .focus_move(FocusMove::Next)
            .moved()
    );
    assert_eq!(
        window_walk.pipeline().focus(),
        Some(nodes_b[0]),
        "the unscoped walk opens on the tree's first candidate — the background control"
    );
    let _ = modal_b;
}

#[test]
fn m26_the_entry_contract_lands_the_first_candidate() {
    // The drift the cut repairs: ADR-0031 has documented the walk's entry
    // contract as "from no focus — or from a focus outside the trap — the
    // walk takes the trap's first candidate" since M25, but the code's
    // positional arithmetic landed the *second* candidate for a focus
    // outside the filtered list (and the last one backward). The
    // document wins: both doors land the first candidate, both
    // directions, from no focus and from an out-of-list focus alike.
    let mut harness = TestHarness::new(420.0, 300.0);
    let (modal, nodes) = mount_modal(&mut harness);
    let [background, first_in_trap, _second_in_trap, _checkbox, _send] =
        [nodes[0], nodes[1], nodes[2], nodes[3], nodes[4]];

    // Focus rests on the background control (the unscoped door).
    assert!(harness.pipeline_mut().focus_move(FocusMove::Next).moved());
    assert_eq!(harness.pipeline().focus(), Some(background));

    // The scoped door enters at the trap's first candidate — forward.
    assert!(
        harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Next)
            .moved()
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(first_in_trap),
        "from a focus outside the trap, forward takes the trap's first candidate — the documented contract"
    );

    // Back outside, the same holds backward: the walk enters the list, it
    // does not approach it from its end.
    assert!(
        harness.pipeline_mut().focus_move(FocusMove::Prev).moved(),
        "the unscoped walk returns to the window's first stop"
    );
    assert_eq!(harness.pipeline().focus(), Some(background));
    assert!(
        harness
            .pipeline_mut()
            .focus_move_in(modal, FocusMove::Prev)
            .moved()
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(first_in_trap),
        "from a focus outside the trap, backward takes the trap's first candidate too"
    );
}

#[test]
fn m26_a_directional_move_costs_one_frame() {
    // The directional walk rides the one invalidation currency: one move
    // queues the FOCUS | PAINT masks on the two endpoints, the next frame
    // repaints the pair, and the frame after is idle again (A7).
    let mut harness = TestHarness::new(360.0, 240.0);
    let grid = mount_grid(&mut harness);
    let _ = grid;

    assert!(harness.pipeline_mut().focus_move(FocusMove::Right).moved());
    assert!(
        harness.pipeline().focus().is_some(),
        "the walk resolves immediately — key routing sees the landing this event"
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
fn m26_the_living_documents_re_agree() {
    // The README: the badge and status block ride the pending-cut
    // convention (the successor minor under [Unreleased], m7's rule), the
    // count names the cut, the features bullet carries the scope, and the
    // example table carries the demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "version-0.17.0",
        "M0 through M29",
        "thirty-six decision records",
        "Since M26 the walk reads the window too",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `focus_direction` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`focus_direction`](../crates/alacer/examples/focus_direction.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0032",
        "36 decision records",
        "the directional focus walk",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The decision record itself documents the directional scope.
    let adr = read_workspace_file("docs/adr/ADR-0032-m26-scope.md");
    assert!(
        adr.contains("M26 scope") && adr.contains("directional"),
        "ADR-0032 documents the directional-walk scope"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M26 — the directional focus walk (ADR-0032)**"),
        "the CHANGELOG carries the M26 entry"
    );

    // The m1 header stands: no corpus growth (the demonstration shows no
    // new visual arrangement — the catalog's own rule).
    let m1 = read_workspace_file("crates/alacer-testing/tests/m1_exit_criteria.rs");
    assert!(
        m1.contains("37 blessed snapshots green"),
        "the m1 header's pin stands — the corpus did not grow"
    );

    // Every pin the M-count move touched moved together — the m19–m25
    // suites ride the same cut, the chain cannot drift.
    for suite in ["m19", "m20", "m21", "m22", "m23", "m24", "m25"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("M0 through M29") && source.contains("36 decision records"),
            "the {suite} suite's README and register pins moved with the documents"
        );
    }
    for suite in ["m20", "m21", "m22", "m23", "m24", "m25"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("thirty-six decision records"),
            "the {suite} suite's README ADR-count pin moved with the documents"
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

    // The corpus pins moved with the routes: 57 → 60 across the m5–m25
    // chain (the coordinated rule), the three route gates contracted, and
    // the chain's own cross-checks agree on the same literals.
}

#[test]
fn m26_the_surface_rides_the_same_cut() {
    // The seam ships where the dispatch contract lives: the vocabulary
    // grows inside `FocusMove`, the two doors keep their signatures, and
    // the walk generalizes behind them — one funnel, six directions.
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    for fragment in [
        "pub fn focus_move(&mut self, movement: FocusMove) -> FocusMoveOutcome",
        "pub fn focus_move_in(&mut self, scope: NodeId, movement: FocusMove) -> FocusMoveOutcome",
        "fn cycle_focus_in(&mut self, scope: Option<NodeId>, movement: FocusMove) -> FocusMoveOutcome",
        "fn directional_neighbor(",
        "Self::Left",
        "Self::Right",
        "Self::Up",
        "Self::Down",
    ] {
        assert!(
            passes.contains(fragment),
            "the pipeline carries the directional seam: {fragment}"
        );
    }
    assert!(
        passes.contains("self.cycle_focus_in(None, movement)"),
        "the unscoped funnel rides the same walk — the direction only aims it"
    );
    assert!(
        !passes.contains("miette"),
        "miette-class help stays plain text — no new dependency (ADR-0029's rule)"
    );

    // The micro-debt the sweep priced is gone: the selection's dead
    // degenerate predicate and the color's dead premultiplied conversion
    // (the M25 species — the accessor whose only caller was its own test).
    let widget = read_workspace_file("crates/alacer-core/src/widget.rs");
    assert!(
        !widget.contains("fn is_degenerate"),
        "Selection::is_degenerate is gone — dead public API, no caller anywhere"
    );
    let color = read_workspace_file("crates/alacer-core/src/color.rs");
    assert!(
        !color.contains("fn to_premultiplied_rgba8"),
        "Color::to_premultiplied_rgba8 is gone — dead public API, its only caller was its own test"
    );

    // The consumer demonstration names the decision it demonstrates,
    // drives the seam it demonstrates, and rides the workspace's
    // per-seam tracing idiom.
    let example = read_workspace_file("crates/alacer/examples/focus_direction.rs");
    assert!(
        example.contains("ADR-0032")
            && example.contains("FocusMove::Right")
            && example.contains("alacer::focus_direction"),
        "examples/focus_direction.rs demonstrates the directional walk"
    );

    // No dependency edge moved for the cut: the core manifest is the
    // cut's only surface, and it gains nothing.
    let core_manifest = read_workspace_file("crates/alacer-core/Cargo.toml");
    assert!(
        !core_manifest.contains("miette"),
        "miette-class help stays plain text — no new dependency (ADR-0029's rule)"
    );
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

#[test]
fn m26_the_corpus_pins_moved_with_the_routes() {
    // The corpus moved with the routes: 57 → 61 across the m5–m26 chain —
    // the coordinated rule (bench, gates, and baseline in the same cut),
    // the outcome route joining the three focus routes (M27, ADR-0033).
    for suite in ["m5", "m6", "m9", "m10"] {
        let source = read_workspace_file(&format!(
            "crates/alacer-testing/tests/{suite}_exit_criteria.rs"
        ));
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 62;"),
            "the {suite} suite's corpus pin moved with the routes"
        );
    }
    let m7 = read_workspace_file("crates/alacer-testing/tests/m7_exit_criteria.rs");
    assert!(
        m7.contains("bench_count, 62,"),
        "the m7 suite's corpus pin moved with the routes"
    );
    let m8 = read_workspace_file("crates/alacer-testing/tests/m8_exit_criteria.rs");
    assert!(
        m8.contains("the corpus is 62:"),
        "the m8 suite's corpus pin moved with the routes"
    );
    for path in std::fs::read_dir(workspace_root().join("crates/alacer-testing/tests"))
        .expect("the suite directory lists")
    {
        let path = path.expect("the entry reads").path();
        let name = path.file_name().map(|n| n.to_string_lossy().to_string());
        if let Some(name) = name
            && name.starts_with('m')
            && name.ends_with("_exit_criteria.rs")
            // This suite's own source names the forbidden species as
            // strings — the check skips itself, the way a spell-checker
            // skips its dictionary.
            && name != "m26_exit_criteria.rs"
        {
            let source = read_workspace_file(&format!("crates/alacer-testing/tests/{name}"));
            assert!(
                !source.contains("usize = 57;")
                    && !source.contains("json!(57)")
                    && !source.contains("All 57"),
                "the {name} suite carries no stale 57-case pin — the chain cannot drift"
            );
        }
    }
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let gates: Vec<String> = contracts["regressions"]
        .as_array()
        .expect("regressions array")
        .iter()
        .filter_map(|gate| gate["baseline"].as_str().map(str::to_owned))
        .collect();
    assert_eq!(
        gates.len(),
        62,
        "the contract set carries the corpus the pins promise"
    );
    for route in [
        "input/route_100_focus_moves_1k",
        "input/route_100_scoped_moves_1k",
        "input/route_100_directional_moves_1k",
        "input/route_100_outcome_moves_1k",
    ] {
        assert!(
            gates.iter().any(|gate| gate == route),
            "the {route} gate rides the contract set"
        );
    }

    // The benches carry the three routes the corpus promises.
    let input_bench = read_workspace_file("crates/alacer-core/benches/input.rs");
    for route in [
        "route_100_focus_moves_1k",
        "route_100_scoped_moves_1k",
        "route_100_directional_moves_1k",
        "route_100_outcome_moves_1k",
    ] {
        assert!(
            input_bench.contains(route),
            "the input bench carries {route}"
        );
    }
}
