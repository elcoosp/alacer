//! M136 exit-criteria integration tests (§11.1, ADR-0140): the tooltip
//! tier — the hover-help bubble the pointer pair summons, said by the
//! tree. Through M135 the snapshot said the tree tier (ADR-0139), the
//! dialog (ADR-0138), the meter (ADR-0137), the search field
//! (ADR-0136), the named pane (ADR-0135) — and the help words had two
//! shapes, the always-on description tier (M95, ADR-0099: the help
//! text *beside a name*) and the modal window (M134, ADR-0138: the
//! conversation *in a window*) — while the *transient* help bubble had
//! no word of its own: an app wiring a hover-help reached for a second
//! label wearing no container word (the bubble the AT cannot name is a
//! bubble it cannot offer on demand) or for a dialog wearing the wrong
//! word (a dialog is a modal window the walk bounds itself into; a
//! tooltip is a transient bubble the pointer pair summons, present
//! exactly while the app mounts it).
//! [`Role::Tooltip`](alacer_core::widget::Role::Tooltip) carries the
//! bubble word AccessKit's own role spells.
//!
//! The tier prices the word and nothing else: the bubble is the help's
//! layout, never a second channel to it (the children ride the arena
//! exactly a pane's do, each keeping the role and label it already
//! carries), presence is the app's wiring (the view mounts and unmounts
//! the bubble, the reconciler's own shape), and a tooltip is not a
//! control: it holds no focus and answers no activation (the
//! M126/M129 container rule, at the hover tier — the controls the app
//! mounts around it hold the focus).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the sink's role branches grow one beside the
//! tree's.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, TextInput, Tooltip};

/// The suite's corpus pin: the coordinated rule's count at the M136
/// cut — unchanged from the M135 cut. The tooltip tier grows no
/// snapshot *route* (the word is the container's own, the shape the
/// M124/M127/M129/M131/M134/M135 named containers rode), and its
/// pricing door is the widget-level bench the tier carries in the
/// widgets crate (`tooltip/name_write_256` against
/// `tooltip/idle_write_256`), which rides *outside* the published
/// corpus: the published baseline's routes price the a11y snapshot
/// machinery (the M95 rule), and the coordinated rule grows it at the
/// next published cut, never at a widget bench's. The pin re-agrees at
/// 155.
const EXPECTED_BENCH_COUNT: usize = 208;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m136_the_tooltip_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    // The bubble word, mounted through the constructor word: the named
    // hover-help beside the control it helps — the bubble's content a
    // child riding the arena, keeping the word it already carries.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let tooltip_id = harness.add_child(root, Tooltip::new("Copy shortcut").boxed());
    let _ = harness.add_child(tooltip_id, Label::new("Ctrl+C").boxed());
    let _ = harness.add_child(root, TextInput::new("Subject").boxed());
    harness.settle();

    // The bubble carries the tooltip word; the child keeps the word it
    // published — the bubble their layout, never a second channel.
    let bubble = node(&harness, Role::Tooltip, "Copy shortcut")
        .expect("the named bubble rides the snapshot");
    assert_eq!(
        bubble.role,
        Role::Tooltip,
        "the word AccessKit's own role spells"
    );
    let hint = node(&harness, Role::Label, "Ctrl+C").expect("the help text keeps its word");
    assert_eq!(
        hint.role,
        Role::Label,
        "the bubble's content is the content's own — never a tooltip wearing the child"
    );
    // One word per node: the tooltip's word is the only tooltip word in
    // the tree — the neighbours keep the words they published.
    assert_eq!(
        harness.query(Role::Tooltip).count(),
        1,
        "no neighbour munged onto the bubble word"
    );
    harness.assert_all_named();
}

#[test]
fn m136_the_bubble_is_not_a_control_and_walls_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let tooltip_id = harness.add_child(root, Tooltip::new("Copy shortcut").boxed());
    let field_id = harness.add_child(root, TextInput::new("Subject").boxed());
    harness.settle();

    // A tooltip is not a control: the bubble holds no focus — the
    // controls the app mounts beside it do (the M126/M129 container
    // rule, at the hover tier). Tab walks past the bubble into the
    // field; the bubble is never a stop.
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
        "the walk landed the field, never the bubble"
    );

    // The word's one write (view reconciliation): the name is the app's
    // declaration, rewritten in place — the bubble re-announces it the
    // next frame. The write that lands where the word already is queues
    // nothing (the M21 idle-frame property, at the tooltip tier).
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(tooltip_id);
        let renamed = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Tooltip>()
            .is_some_and(|bubble| {
                bubble.set_name("Paste shortcut");
                true
            });
        let _ = renamed;
        guard.request(alacer_core::Change::A11Y);
        drop(guard);
    }
    harness.settle();
    node(&harness, Role::Tooltip, "Paste shortcut").expect("the rename landed — the word rides");
    harness.assert_all_named();
}

#[test]
fn m136_the_dsl_tooltip_word_declares_it() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    // The `tooltip` word the element parses — the bubble's truth rides
    // the words, at mount, with no new prop class to pay for (the M134
    // dialog word's own shape).
    let view = alacer_macro::rsx! {
        column {
            tooltip {
                name: "Copy shortcut",
                label {
                    text: "Ctrl+C",
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
    let declared = node(&harness, Role::Tooltip, "Copy shortcut")
        .expect("the view's bubble rides the snapshot");
    assert_eq!(
        declared.role,
        Role::Tooltip,
        "the DSL's word declares the tier"
    );
    let hint = node(&harness, Role::Label, "Ctrl+C")
        .expect("the declared child rides the bubble's arena slot");
    assert_eq!(hint.role, Role::Label, "the child keeps its own word");
    let _ = root;
}

#[test]
fn m136_the_corpus_pin_re_agrees_at_the_m136_cut() {
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

    // The published corpus pin re-agrees: the tooltip tier grows no
    // snapshot route (the word is the container's own, the shape the
    // named containers rode), and its pricing door is the widgets
    // crate's own bench — outside the corpus at its own cut, joined by the corpus catch-up's published cut — the coordinated rule's next published cut cashed.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M136 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M136 cut"
    );
}

#[test]
fn m136_the_living_documents_re_agree() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).expect("the file is readable");

    // The README status reflects the living cut, and the register grew
    // the tooltip tier's record.
    let readme = read("README.md");
    assert!(
        readme.contains("M0 through M156"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("a hundred and sixty-one decision records"),
        "the register grew the M136 row"
    );
    assert!(
        read("docs/architecture.md").contains("a hundred and sixty-one decision records"),
        "the architecture carries the tooltip fact's decision"
    );

    // The examples catalog grew the tooltip row, and the changelog
    // names the tier.
    assert!(
        read("docs/examples.md").contains("examples/tooltip.rs"),
        "the catalog carries the tooltip demonstration"
    );
    assert!(
        read("CHANGELOG.md").contains("M136 — the tooltip tier"),
        "the changelog names the tier"
    );

    // The m81 suite's documents pin moved to M136's cut (the latest
    // suite carrying the living-documents pin — the M132+ cut's suites
    // pin the corpus, and their pins re-agreed with it).
    let m81 = read("crates/alacer-testing/tests/m81_exit_criteria.rs");
    assert!(
        m81.contains("M0 through M156"),
        "the m81 suite's pins moved with the documents"
    );
}
