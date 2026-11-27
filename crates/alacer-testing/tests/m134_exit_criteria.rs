//! M134 exit-criteria integration tests (§7.2 pass 9, ADR-0138): the
//! dialog tier — the window the modal vocabulary speaks as "this is a
//! dialog", said by the tree. Through M133 the snapshot said the meter
//! tier (ADR-0137), the search field (ADR-0136), the named pane
//! (ADR-0135) — and the modal window had no word: an app wiring a
//! confirmation, a settings sheet, a file picker reached for the bare
//! panel (a container wearing no name — the set the AT cannot name is
//! a set it cannot walk, the ADR-5 rule the named roles carry) or for
//! a second tab panel wearing the wrong word (a tab panel names *the
//! selected page's content*; a dialog names *the window the
//! conversation lives in*, and the AT's vocabulary names the
//! difference: AccessKit's `Dialog` role is its own fact).
//! [`Role::Dialog`](alacer_core::widget::Role::Dialog) carries the
//! window word the AT announces before it walks the conversation — the
//! named-container shape (ADR-0128) grown to the modal tier.
//!
//! The word prices the word and nothing else: the dialog is the
//! content's layout, never a second channel to it — the children ride
//! the arena exactly as a [`alacer_widgets::TabPanel`]'s do, each child
//! keeping its role, its label, and its doors. A dialog is not a
//! control: it holds no focus and answers no activation (the
//! M126/M129 container rule — the controls inside it hold the focus).
//! The modal *semantics* — the walk bounded to the dialog's subtree
//! while it is up — ride the focus scope the M25 tier published
//! (ADR-0031's `focus_move_in`), never a second channel; the word
//! names the window, the walk bounds it.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the sink's role branch grows one beside the tab
//! panel's.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Dialog, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M134
/// cut — unchanged from the M133 cut. The dialog tier grows no
/// snapshot *route* (the word is the container's own, the shape the
/// M124/M127/M129/M131 named containers rode), and its pricing door is
/// the widget-level bench the tier carries in the widgets crate
/// (`dialog/name_write_256` against `dialog/idle_write_256`), which
/// rides *outside* the published corpus: the published baseline's
/// routes price the a11y snapshot machinery (the M95 rule), and the
/// coordinated rule grows it at the next published cut, never at a
/// widget bench's. The pin re-agrees at 155.
const EXPECTED_BENCH_COUNT: usize = 207;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m134_the_window_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    // The dialog word, mounted through the constructor word: the named
    // window beside the bare panel the canon keeps honest — each
    // container's word its own.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let dialog_id = harness.add_child(root, Dialog::new("Confirm publish").boxed());
    let _ = harness.add_child(dialog_id, TextInput::new("File name").boxed());
    let _ = harness.add_child(dialog_id, Button::new("Publish").boxed());
    harness.settle();

    // The window carries the dialog word; the controls inside keep the
    // words and doors they always carried — the container is the
    // content's layout, never a second channel to them.
    node(&harness, Role::Dialog, "Confirm publish").expect("the named window rides the snapshot");
    node(&harness, Role::TextInput, "File name").expect("the field keeps its word");
    node(&harness, Role::Button, "Publish").expect("the button keeps its door");
    // One word per node: the window's word is the only dialog word in
    // the tree — the neighbours keep the words they published.
    assert_eq!(
        harness.query(Role::Dialog).count(),
        1,
        "the honest absence rides beside the word — no neighbour munged"
    );
    harness.assert_all_named();
}

#[test]
fn m134_the_word_walls_nothing_and_never_the_focusable_set() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let dialog_id = harness.add_child(root, Dialog::new("Confirm publish").boxed());
    let field_id = harness.add_child(dialog_id, TextInput::new("File name").boxed());
    let _ = harness.add_child(dialog_id, Button::new("Publish").boxed());
    harness.settle();

    // A dialog is not a control: the window holds no focus — the
    // controls inside it do (the M126/M129 container rule, at the
    // modal tier). Tab walks *into* the dialog's controls; the
    // container itself is never a stop.
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
        "the walk landed the field, never the container"
    );

    // The word's one write (view reconciliation): the name is the
    // app's declaration, rewritten in place — the tree re-announces
    // the word the next frame.
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(dialog_id);
        let renamed = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Dialog>()
            .is_some_and(|dialog| {
                dialog.set_name("Confirm deploy");
                true
            });
        let _ = renamed;
        guard.request(alacer_core::Change::A11Y);
        drop(guard);
    }
    harness.settle();
    node(&harness, Role::Dialog, "Confirm deploy").expect("the rename landed — the word rides");
    harness.assert_all_named();
}

#[test]
fn m134_the_dsl_dialog_word_declares_it() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    // The `dialog` word the element parses — the tree's truth rides the
    // word, at mount, with no new prop class to pay for (the M131
    // tab-panel word's own shape).
    let view = alacer_macro::rsx! {
        column {
            dialog {
                name: "Confirm publish",
                description: "The file goes live",
                button { text: "Publish" }
            }
        }
    };
    let root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    let declared = node(&harness, Role::Dialog, "Confirm publish")
        .expect("the view's window rides the snapshot");
    assert_eq!(
        declared.description.as_deref(),
        Some("The file goes live"),
        "the DSL's dialog word declares the window through the same door"
    );
    node(&harness, Role::Button, "Publish").expect("the member keeps its word");
    let _ = root;
}

#[test]
fn m134_the_corpus_pin_re_agrees_at_the_m134_cut() {
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

    // The published corpus pin re-agrees: the dialog tier grows no
    // snapshot route (the word is the container's own, the shape the
    // named containers rode), and its pricing door is the widgets
    // crate's own bench — outside the corpus at its own cut, joined by the corpus catch-up's published cut — the coordinated rule's next published cut cashed.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M134 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M134 cut"
    );
}

#[test]
fn m134_the_living_documents_re_agree() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).expect("the file is readable");

    // The README status reflects the living cut, and the register grew
    // the dialog tier's record.
    let readme = read("README.md");
    assert!(
        readme.contains("M0 through M155"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("a hundred and sixty decision records"),
        "the register grew the M134 row"
    );
    assert!(
        read("docs/architecture.md").contains("a hundred and sixty decision records"),
        "the architecture carries the dialog fact's decision"
    );

    // The examples catalog grew the dialog row, and the changelog names
    // the tier.
    assert!(
        read("docs/examples.md").contains("examples/dialog.rs"),
        "the catalog carries the dialog demonstration"
    );
    assert!(
        read("CHANGELOG.md").contains("M134 — the dialog tier"),
        "the changelog names the tier"
    );

    // The m81 suite's documents pin moved to M134's cut (the latest
    // suite carrying the living-documents pin — the M132+ cut's suites
    // pin the corpus, and their pins re-agreed with it).
    let m81 = read("crates/alacer-testing/tests/m81_exit_criteria.rs");
    assert!(
        m81.contains("M0 through M155"),
        "the m81 suite's pins moved with the documents"
    );
}
