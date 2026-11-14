//! M23 exit-criteria integration tests (§9.1, ADR-0029): the frame's
//! ledger becomes consumer vocabulary. The scene journal reports its own
//! ledger — commits, drops, live layers — through the required
//! `SceneJournal::stats()` seam, read via `PassPipeline::journal_stats()`,
//! and `examples/frame_ledger.rs` demonstrates it from the consumer side:
//! one write batch re-commits exactly one layer (the written widget's),
//! clean layers replay untouched, idle frames write nothing. The cut also
//! repairs the blessed list-scroll trio — the `list_scrolled` scenarios
//! never fed `set_content_height`, so `scroll_to` clamped every offset to
//! zero and three distinct scroll positions blessed one identical image —
//! and the micro-debt the sweep priced rides the same cut: the journal's
//! dead inherent accessors, the todo canon's dead `footer_node`, the
//! `rsx!` body-cardinality diagnostic that did not name its cardinality,
//! and the inspector accept loop's silently swallowed session errors. The
//! living documents re-agree at 0.14.0.
//!
//! Every test runs the real pipeline (A4) with the real layout host — the
//! exact route a window's writes take.

use alacer_core::flux::RenderBinding;
use alacer_core::mask::Change;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, List, Panel};
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

/// The consumer's window the example assembles: a static label whose
/// layer must never re-commit, and one bound ticker label — the write
/// that costs. The exact shape `examples/frame_ledger.rs` mounts.
/// Returns the ticker's signal handle (the write that costs).
fn mount_ledger_demo(harness: &mut TestHarness) -> alacer_core::FluxRwSignal<u32> {
    let root = harness.mount(Column::new().gap(8.0).padding(8.0).boxed());
    let _ = harness.add_child(root, Label::new("This layer never re-commits").boxed());
    let ticks = harness.pipeline_mut().flux_mut().signal(0_u32);
    let binding = RenderBinding::new("Ticks: 0".to_string());
    let ticker = harness.add_child(root, Label::bound(binding.clone()).boxed());
    harness.pipeline_mut().flux_mut().bind_value(
        ticks,
        ticker,
        &binding,
        Change::PAINT.union(Change::A11Y),
        |n: &u32| format!("Ticks: {n}"),
    );
    harness.settle();
    ticks
}

#[test]
fn m23_the_ledger_counts_one_write_layer() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let ticks = mount_ledger_demo(&mut harness);

    // The mount frame's ledger: the tree's full shape committed once —
    // the journal accounts every layer it holds.
    let before = harness.pipeline().journal_stats();
    assert!(
        before.commits > 0 && before.live_layers > 0,
        "the mount frame committed the tree's layers: {before:?}"
    );

    // One write batch: exactly one layer re-commits — the ticker's. The
    // static label's clean layer replays untouched, nothing churns, and
    // the tree's shape is unchanged.
    reactive_graph::traits::Set::set(&ticks, 1_u32);
    harness.settle();
    let after = harness.pipeline().journal_stats();
    assert_eq!(
        after.commits - before.commits,
        1,
        "one write batch re-commits exactly one layer"
    );
    assert_eq!(after.drops, before.drops, "no churn: nothing dropped");
    assert_eq!(
        after.live_layers, before.live_layers,
        "the tree's shape is unchanged"
    );
    assert!(
        harness.query(Role::Label).named("Ticks: 1").exists(),
        "the write landed where the ledger says — the tree carries the new text"
    );

    // The second write costs exactly one more — the ledger is linear in
    // the writes, not in the tree.
    reactive_graph::traits::Set::set(&ticks, 2_u32);
    harness.settle();
    let third = harness.pipeline().journal_stats();
    assert_eq!(
        third.commits - after.commits,
        1,
        "the second write re-commits the ticker's layer again — and nothing else"
    );
}

#[test]
fn m23_an_idle_frame_costs_nothing() {
    let mut harness = TestHarness::new(360.0, 240.0);
    let _ticks = mount_ledger_demo(&mut harness);
    let before = harness.pipeline().journal_stats();

    // Interaction-free frames do nothing (A7) — and write nothing: the
    // journal's ledger is bit-identical across idle frames.
    for frame in 0..4 {
        let report = harness.tick();
        assert!(report.is_idle(), "frame {frame} had work after settling");
    }
    assert_eq!(
        harness.pipeline().journal_stats(),
        before,
        "idle frames write no ledger entries"
    );
}

/// The m1 corpus's repaired shape (ADR-0029): a list of `panels` 32-px
/// panels in a 220 px viewport, ready to scroll — the way the
/// `list_scrolled` scenarios drive it, with the content height the clamp
/// clamps against. Returns the shared handle and the first child's node
/// (the call sites scroll and assert, with the literal targets).
fn mounted_panel_list(
    harness: &mut TestHarness,
    panels: usize,
) -> (
    alacer_widgets::ListHandle,
    alacer_core::NodeId,
    alacer_core::NodeId,
) {
    let root = harness.mount(Column::new().padding(8.0).boxed());
    let list = List::new(220.0);
    let handle = list.handle();
    let list_node = harness.add_child(root, list.boxed());
    for _ in 0..panels {
        let _ = harness.add_child(
            list_node,
            Panel::new(32.0, alacer_widgets::theme::PANEL).boxed(),
        );
    }
    harness.settle();

    handle.set_content_height(f32::from(u16::try_from(panels).unwrap_or(u16::MAX)) * 32.0);
    let first_child = harness.pipeline().arena().children(list_node)[0];
    (handle, list_node, first_child)
}

/// A programmatic scroll rides the mutate seam (§8.2, A1): the shared
/// state's write is the widget's, the currency is the pipeline's — the
/// guard's `request` queues the same `PAINT | A11Y` masks the wheel path
/// requests through `EventCtx`. A currency-less `scroll_to` would leave
/// the compose pass nothing to see (an idle frame that never translates).
fn scroll_via_mutate_seam(harness: &mut TestHarness, list_node: alacer_core::NodeId, offset: f32) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(list_node);
    if let Some(list) = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<alacer_widgets::List>()
    {
        list.handle().scroll_to(offset);
    }
    guard.request(alacer_core::mask::Change::PAINT.union(alacer_core::mask::Change::A11Y));
}

#[test]
fn m23_the_repaired_scroll_scenarios_reach_their_offsets() {
    // The s14 shape: twelve panels (content 384) scrolled by 120 — the
    // ten-panel original capped the clamp's range at 100, one half of the
    // broken trio's root cause.
    let mut harness = TestHarness::new(240.0, 320.0);
    let (handle, list_node, first_child) = mounted_panel_list(&mut harness, 12);
    let before_y = harness.bounds(first_child).origin.y;

    scroll_via_mutate_seam(&mut harness, list_node, 120.0);
    assert_eq!(
        handle.offset(),
        120.0,
        "twelve panels of 32 px leave 120 px inside the clamp's real range"
    );
    harness.settle();
    // Translation-only: the child moved by exactly the offset, its size
    // untouched (§8.2 item 4) — the pixels the re-blessed scenarios pin.
    assert_eq!(
        harness.bounds(first_child).origin.y,
        before_y - 120.0,
        "the scroll translated the subtree by the offset"
    );

    // The s15 shape: twenty-four panels (content 768) scrolled by 400 —
    // reachable, but only once the clamp knows the content height (the
    // other half: the scenario never fed it).
    let mut harness = TestHarness::new(240.0, 320.0);
    let (handle, list_node, first_child) = mounted_panel_list(&mut harness, 24);
    let before_y = harness.bounds(first_child).origin.y;

    scroll_via_mutate_seam(&mut harness, list_node, 400.0);
    assert_eq!(
        handle.offset(),
        400.0,
        "twenty-four panels of 32 px leave 400 px inside the clamp's real range"
    );
    harness.settle();
    assert_eq!(
        harness.bounds(first_child).origin.y,
        before_y - 400.0,
        "the far scroll translated the subtree by the offset"
    );
}

#[test]
fn m23_the_blessed_list_scenarios_are_distinct_images() {
    // The corpus's own guard against the trio's failure mode: three
    // scenarios named `top`, `scrolled_120`, and `scrolled_400` must be
    // three distinct images. They blessed byte-identical PNGs (the clamp
    // folded every offset to zero) — the repair lands with the scenarios,
    // and the next bless run (`ALACER_BLESS=1 cargo test -p
    // alacer-testing`) re-blesses them distinct, exactly as CONTRIBUTING's
    // snapshot rule prescribes: the diff is the review.
    let blessed = workspace_root().join("crates/alacer-testing/blessed");
    let read = |name: &str| {
        std::fs::read(blessed.join(name))
            .unwrap_or_else(|error| panic!("blessed snapshot {name} missing: {error}"))
    };
    let top = read("m1_12_list_top.png");
    let s120 = read("m1_13_list_scrolled_120.png");
    let s400 = read("m1_14_list_scrolled_400.png");
    assert_ne!(
        top, s120,
        "list_top and list_scrolled_120 are distinct images"
    );
    assert_ne!(
        s120, s400,
        "list_scrolled_120 and list_scrolled_400 are distinct images"
    );
    assert_ne!(
        top, s400,
        "list_top and list_scrolled_400 are distinct images"
    );
}

#[test]
fn m23_the_living_documents_re_agree() {
    // The README: the badge and status block quote the 0.14.0 family the
    // manifest ships, the register count moved with ADR-0029, the perf
    // bullet carries the ledger, and the example table carries the
    // demonstration.
    let readme = read_workspace_file("README.md");
    for fragment in [
        "version-0.17.0",
        "M0 through M61",
        "sixty-eight decision records",
        "Since M23 the frame carries a public ledger",
    ] {
        assert!(readme.contains(fragment), "the README names {fragment}");
    }
    assert!(
        readme.contains("| `frame_ledger` |"),
        "the README's example table carries the demonstration"
    );

    // The catalog: the example is listed with its contract.
    let catalog = read_workspace_file("docs/examples.md");
    assert!(
        catalog.contains("[`frame_ledger`](../crates/alacer/examples/frame_ledger.rs)"),
        "docs/examples.md carries the demonstration"
    );

    // The architecture carries the decision, the register row, the
    // roadmap row, and the count that agrees with the register.
    let architecture = read_workspace_file("docs/architecture.md");
    for fragment in [
        "ADR-0029",
        "68 decision records",
        "the scene journal becomes legible to its consumers",
    ] {
        assert!(
            architecture.contains(fragment),
            "docs/architecture.md names {fragment}"
        );
    }

    // The decision record itself documents the scope.
    let adr = read_workspace_file("docs/adr/ADR-0029-m23-scope.md");
    assert!(
        adr.contains("M23 scope") && adr.contains("frame's ledger"),
        "ADR-0029 documents the frame-ledger scope"
    );

    // The changelog stages the milestone.
    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("**M23 — the frame's ledger (ADR-0029)**"),
        "the CHANGELOG carries the M23 entry"
    );

    // The migration guides name the shipped family again.
    for guide in ["docs/migration/from-iced.md", "docs/migration/from-egui.md"] {
        let guide = read_workspace_file(guide);
        assert!(
            guide.contains("onto Alacer **0.16.0**"),
            "{guide} names the shipped family"
        );
    }

    // Every pin the version-and-count move touched moved together — the
    // chain cannot drift (the m-suite's own rule, applied to itself).
    let m20 = read_workspace_file("crates/alacer-testing/tests/m20_exit_criteria.rs");
    let m21 = read_workspace_file("crates/alacer-testing/tests/m21_exit_criteria.rs");
    let m22 = read_workspace_file("crates/alacer-testing/tests/m22_exit_criteria.rs");
    for (suite, source) in [("m20", &m20), ("m21", &m21), ("m22", &m22)] {
        assert!(
            source.contains("M0 through M61") && source.contains("sixty-eight decision records"),
            "the {suite} suite's README pins moved with the documents"
        );
        assert!(
            source.contains("68 decision records"),
            "the {suite} suite's architecture pin moved with the register"
        );
    }
    assert!(
        m20.contains("version-0.17.0") && m21.contains("version-0.17.0"),
        "the m20/m21 version pins moved with the badge"
    );
    assert!(
        m21.contains("onto Alacer **0.16.0**"),
        "the m21 suite's migration-guide pin moved with the guides"
    );
}

#[test]
fn m23_the_surface_rides_the_same_cut() {
    // The seam ships where the trait lives: the vocabulary, the required
    // method, and the pipeline's one read door.
    let passes = read_workspace_file("crates/alacer-core/src/passes.rs");
    assert!(
        passes.contains("pub struct JournalStats")
            && passes.contains("fn stats(&self) -> JournalStats")
            && passes.contains("pub fn journal_stats(&self) -> JournalStats"),
        "the ledger seam rides the SceneJournal trait and the pipeline door"
    );

    // The journal's dead inherent accessors are gone — the seam replaced
    // them (commits and live layers answer through `stats`; the slice-view
    // `layer` duplicate had no caller at all).
    let journal = read_workspace_file("crates/alacer-render/src/journal.rs");
    assert!(
        !journal.contains("pub const fn commits")
            && !journal.contains("pub const fn drops")
            && !journal.contains("pub fn layer_count")
            && !journal.contains("pub fn layer(&self"),
        "the journal's dead inherent accessors are gone"
    );
    assert!(
        journal.contains("fn stats(&self) -> JournalStats"),
        "the journal accounts its own ledger"
    );

    // The todo canon's dead getter is gone with its write-only field.
    let canon = read_workspace_file("crates/alacer-widgets/src/canon.rs");
    assert!(
        !canon.contains("fn footer_node"),
        "TodoHandles::footer_node is gone"
    );

    // The rsx! body-cardinality diagnostic names its cardinality (the
    // ADR-0025 species), and the no-op discards are gone with it.
    let macro_src = read_workspace_file("crates/alacer-macro/src/lib.rs");
    assert!(
        macro_src.contains("must contain exactly one element, found {found}"),
        "the cardinality diagnostic names what it found"
    );
    assert!(
        !macro_src.contains("let _ = &element") && !macro_src.contains("let _ = for_token"),
        "the macro's no-op discards are gone"
    );

    // The inspector's accept loop reports the session errors it used to
    // swallow — the seam's observability rides the adopted tracing core.
    let server = read_workspace_file("crates/alacer-inspect/src/server.rs");
    assert!(
        server.contains("tracing::warn!"),
        "the accept loop reports session errors"
    );
    let inspect_manifest = read_workspace_file("crates/alacer-inspect/Cargo.toml");
    assert!(
        inspect_manifest.contains("tracing.workspace = true"),
        "the inspector's observability rides the adopted core"
    );

    // The consumer demonstration names the decision it demonstrates and
    // reads the seam it demonstrates.
    let example = read_workspace_file("crates/alacer/examples/frame_ledger.rs");
    assert!(
        example.contains("ADR-0029") && example.contains("journal_stats"),
        "examples/frame_ledger.rs demonstrates the frame's ledger"
    );
}
