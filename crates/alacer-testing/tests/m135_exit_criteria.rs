//! M135 exit-criteria integration tests (§7.2 pass 9, ADR-0139): the
//! tree tier — the hierarchical collection the AT walks by disclosure
//! and depth, said by the tree. Through M134 the snapshot said the
//! dialog tier (ADR-0138), the meter (ADR-0137), the search field
//! (ADR-0136), the named pane (ADR-0135) — and the flat collections
//! had the words (the M6 table grid, the M108 picks, the M110
//! disclosures, the M113 depths, the M119 places) while the
//! *hierarchical* collection — the file explorer, the category outline,
//! the nested navigation — had no word of its own: an app mounting one
//! reached for the table's grid (a grid's geometry is rows × columns;
//! a tree's is parentage — the wrong geometry wearing the wrong word)
//! or for a bare list whose members could say nothing about the
//! branches they stood in. [`Role::Tree`](alacer_core::widget::Role::Tree)
//! and [`Role::TreeItem`](alacer_core::widget::Role::TreeItem) carry
//! the collection word and the member word AccessKit's own roles
//! spell — the member-and-container shape (ADR-0128) grown to the
//! hierarchical tier.
//!
//! The tier prices the words and nothing else: the hierarchy *is* the
//! arena's own nesting (a tree item's children ride under it, exactly
//! a row's cells do), the disclosure rides the M110 word, the pick the
//! M108 word, the depth the M113 word, the place the M119 word, the
//! span the M119 container word — every word a tier the tree already
//! published, never a second vocabulary. A tree and its items are not
//! controls: they hold no focus and answer no activation (the
//! M126/M129 container rule, at the tree tier — the controls the app
//! mounts inside the items hold the focus).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the sink's role branches grow one beside the
//! dialog's.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput, Tree, TreeItem};

/// The suite's corpus pin: the coordinated rule's count at the M135
/// cut — unchanged from the M134 cut. The tree tier grows no snapshot
/// *route* (the words are the container's and member's own, the shape
/// the M124/M127/M129/M131/M134 named containers rode), and its
/// pricing door is the widget-level bench the tier carries in the
/// widgets crate (`tree/name_write_256` against `tree/idle_write_256`),
/// which rides *outside* the published corpus: the published baseline's
/// routes price the a11y snapshot machinery (the M95 rule), and the
/// coordinated rule grows it at the next published cut, never at a
/// widget bench's. The pin re-agrees at 144.
const EXPECTED_BENCH_COUNT: usize = 144;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m135_the_tree_words_land_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    // The tree words, mounted through the constructor words: the named
    // collection beside its members — each member saying the depth, the
    // place, the pick, and the disclosure the tiers already published.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let tree_id = harness.add_child(
        root,
        Tree::new("Project files")
            .size_of_set(3)
            .multiselectable(false)
            .boxed(),
    );
    let src_id = harness.add_child(
        tree_id,
        TreeItem::new("src")
            .expanded(true)
            .level(0)
            .position_in_set(0)
            .boxed(),
    );
    let _ = harness.add_child(
        src_id,
        TreeItem::new("lib.rs").level(1).position_in_set(0).boxed(),
    );
    let _ = harness.add_child(
        tree_id,
        TreeItem::new("Cargo.toml")
            .selected(true)
            .level(0)
            .position_in_set(1)
            .boxed(),
    );
    harness.settle();

    // The collection and its members carry the tree words; the words
    // ride the tiers the tree already published — never a second
    // vocabulary.
    let branch = node(&harness, Role::TreeItem, "src").expect("the branch rides the snapshot");
    assert_eq!(
        branch.expanded,
        Some(true),
        "the disclosure the M110 tier published"
    );
    assert_eq!(branch.level, Some(0), "the depth the M113 tier published");
    assert_eq!(
        branch.position_in_set,
        Some(0),
        "the place the M119 tier published"
    );
    let leaf = node(&harness, Role::TreeItem, "Cargo.toml").expect("the leaf rides the snapshot");
    assert_eq!(
        leaf.selected,
        Some(true),
        "the pick the M108 tier published"
    );
    assert!(
        leaf.expanded.is_none(),
        "a leaf that opens and closes not announces the honest absence"
    );
    let collection = node(&harness, Role::Tree, "Project files")
        .expect("the named collection rides the snapshot");
    assert_eq!(
        collection.size_of_set,
        Some(3),
        "the span the M119 container word carries"
    );
    assert!(
        !collection.multiselectable,
        "the multiselectable tier's `false` is the absence — a single-pick tree announces nothing"
    );
    // One word per node: the tree's words are the only tree words in
    // the tree — the neighbours keep the words they published.
    assert_eq!(
        harness.query(Role::Tree).count(),
        1,
        "no neighbour munged onto the collection word"
    );
    harness.assert_all_named();
}

#[test]
fn m135_the_words_wall_nothing_and_never_the_focusable_set() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let tree_id = harness.add_child(root, Tree::new("Project files").boxed());
    let _ = harness.add_child(tree_id, TreeItem::new("src").boxed());
    let field_id = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();

    // A tree and its items are not controls: the collection holds no
    // focus — the controls the app mounts beside and inside it do (the
    // M126/M129 container rule, at the tree tier). Tab walks past the
    // tree into the field; the containers are never stops.
    let answers = harness.key(&alacer_core::events::KeyEvent {
        key: alacer_core::events::Key::Tab,
        state: alacer_core::events::KeyState::Pressed,
        modifiers: alacer_core::events::Modifiers::default(),
        text: None,
    });
    let _ = answers;
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        Some(field_id),
        "the walk landed the field, never the collection or the member"
    );

    // The words' one write (view reconciliation): the names are the
    // app's declarations, rewritten in place — the tree re-announces
    // them the next frame. The write that lands where a word already
    // is queues nothing (the M21 idle-frame property, at the tree
    // tier).
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(tree_id);
        let renamed = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Tree>()
            .is_some_and(|tree| {
                tree.set_name("Workspace files");
                true
            });
        let _ = renamed;
        guard.request(alacer_core::Change::A11Y);
        drop(guard);
    }
    harness.settle();
    node(&harness, Role::Tree, "Workspace files").expect("the rename landed — the word rides");
    harness.assert_all_named();
}

#[test]
fn m135_the_dsl_tree_words_declare_it() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    // The `tree` and `tree_item` words the elements parse — the tree's
    // truth rides the words, at mount, with no new prop class to pay
    // for (the M134 dialog word's own shape).
    let view = alacer_macro::rsx! {
        column {
            tree {
                name: "Project files",
                description: "The workspace's files",
                size_of_set: 2.0,
                tree_item {
                    name: "src",
                    expanded: true,
                    level: 0.0,
                    position_in_set: 0.0,
                }
                tree_item {
                    name: "Cargo.toml",
                    selected: true,
                    level: 0.0,
                    position_in_set: 1.0,
                }
            }
        }
    };
    let root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    let declared = node(&harness, Role::Tree, "Project files")
        .expect("the view's collection rides the snapshot");
    assert_eq!(
        declared.description.as_deref(),
        Some("The workspace's files"),
        "the DSL's tree word declares the collection through the same door"
    );
    let branch = node(&harness, Role::TreeItem, "src").expect("the member rides the snapshot");
    assert_eq!(branch.expanded, Some(true), "the DSL's disclosure rides");
    let _ = root;
}

#[test]
fn m135_the_corpus_pin_re_agrees_at_the_m135_cut() {
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

    // The published corpus pin re-agrees: the tree tier grows no
    // snapshot route (the words are the container's and member's own,
    // the shape the named containers rode), and its pricing door is
    // the widgets crate's own bench — outside the published corpus,
    // the coordinated rule growing it at the next published cut.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M135 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M135 cut"
    );
}

#[test]
fn m135_the_living_documents_re_agree() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).expect("the file is readable");

    // The README status reflects the living cut, and the register grew
    // the tree tier's record.
    let readme = read("README.md");
    assert!(
        readme.contains("M0 through M138"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("a hundred and forty-three decision records"),
        "the register grew the M135 row"
    );
    assert!(
        read("docs/architecture.md").contains("a hundred and forty-three decision records"),
        "the architecture carries the tree fact's decision"
    );

    // The examples catalog grew the tree row, and the changelog names
    // the tier.
    assert!(
        read("docs/examples.md").contains("examples/tree.rs"),
        "the catalog carries the tree demonstration"
    );
    assert!(
        read("CHANGELOG.md").contains("M135 — the tree tier"),
        "the changelog names the tier"
    );

    // The m81 suite's documents pin moved to M135's cut (the latest
    // suite carrying the living-documents pin — the M132+ cut's suites
    // pin the corpus, and their pins re-agreed with it).
    let m81 = read("crates/alacer-testing/tests/m81_exit_criteria.rs");
    assert!(
        m81.contains("M0 through M138"),
        "the m81 suite's pins moved with the documents"
    );
}
