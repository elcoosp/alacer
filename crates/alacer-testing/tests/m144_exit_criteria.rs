//! M144 exit-criteria integration tests (§11.1, ADR-0148): the landmark
//! tier's remaining words — the ARIA facts ADR-0147 pre-registered as
//! constructors beside the four the M143 cut shipped. Through M143 the
//! tier carried the page's four furniture words (the banner, the way
//! through, the main, the colophon — ADR-0147), and the rest of the
//! landmark vocabulary had no words: an app wiring a complementary
//! aside, a form, a generic named region or a search strip reached for
//! bare panels wearing no container word (the region the AT cannot
//! name is a region it cannot offer on demand, the ADR-5 rule the
//! named roles carry) — or for a main wearing the wrong word (a main
//! *names the document's one central region*; a complementary aside
//! *names the content the main reads beside*).
//! [`Role::Complementary`](alacer_core::widget::Role::Complementary),
//! [`Role::Form`](alacer_core::widget::Role::Form),
//! [`Role::Region`](alacer_core::widget::Role::Region) and
//! [`Role::Search`](alacer_core::widget::Role::Search) carry the
//! remaining page-structure facts AccessKit's own roles spell — the
//! landmark vocabulary the AT's landmarks list reads whole.
//!
//! The tier prices the words and nothing else (the M143 rule): a
//! landmark is the page furniture's layout, never a second channel to
//! it (the children ride the arena exactly a pane's do, each keeping
//! the role and label it already carries), and a landmark is not a
//! control: it holds no focus and answers no activation (the
//! M126/M129 container rule, at the landmark tier — the controls the
//! app mounts inside it hold the focus). The growth is constructor
//! words, never machinery: one widget, now eight facts, the map
//! growing one branch per fact (ADR-0147's own re-open rule).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the sink's role branches grow beside the M143
//! ones.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Landmark, TextInput};

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m144_the_remaining_landmark_words_land_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 400.0);
    // The page furniture's remaining words, mounted through the
    // constructor words: the complementary aside, the form, the generic
    // named region and the search strip — each a named container, each
    // child keeping the word it already carries.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let aside_id = harness.add_child(root, Landmark::complementary("Related reading").boxed());
    let _ = harness.add_child(aside_id, Label::new("Reading list").boxed());
    let form_id = harness.add_child(root, Landmark::form("Checkout details").boxed());
    let _ = harness.add_child(form_id, TextInput::new("Cardholder name").boxed());
    let region_id = harness.add_child(root, Landmark::region("Featured panel").boxed());
    let _ = harness.add_child(region_id, Label::new("This week's picks").boxed());
    let search_id = harness.add_child(root, Landmark::search("Catalog search").boxed());
    let _ = harness.add_child(search_id, TextInput::new("Query").boxed());
    harness.settle();

    // Each landmark carries its own word — AccessKit's remaining
    // page-structure facts said by the tree; the children keep the
    // words they published (the landmark their layout, never a second
    // channel).
    let aside = node(&harness, Role::Complementary, "Related reading")
        .expect("the named complementary rides the snapshot");
    assert_eq!(
        aside.role,
        Role::Complementary,
        "the word AccessKit's own role spells"
    );
    let form =
        node(&harness, Role::Form, "Checkout details").expect("the named form rides the snapshot");
    assert_eq!(form.role, Role::Form);
    let region = node(&harness, Role::Region, "Featured panel")
        .expect("the named region rides the snapshot");
    assert_eq!(region.role, Role::Region);
    let search = node(&harness, Role::Search, "Catalog search")
        .expect("the named search rides the snapshot");
    assert_eq!(search.role, Role::Search);
    let inner = node(&harness, Role::Label, "Reading list")
        .expect("the aside's content keeps its own word");
    assert_eq!(
        inner.role,
        Role::Label,
        "the aside's content is the content's own — never an aside wearing the child"
    );
    // One word per node: each landmark's word is the only one of its
    // kind in the tree — the neighbours keep the words they published.
    assert_eq!(
        harness.query(Role::Complementary).count(),
        1,
        "no neighbour munged onto the complementary word"
    );
    assert_eq!(harness.query(Role::Form).count(), 1);
    assert_eq!(harness.query(Role::Region).count(), 1);
    assert_eq!(harness.query(Role::Search).count(), 1);
    harness.assert_all_named();
}

#[test]
fn m144_the_words_are_not_controls_and_the_dsl_declares_them() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 300.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Landmark::complementary("Related reading").boxed());
    let _ = harness.add_child(root, Landmark::form("Checkout details").boxed());
    let field_id = harness.add_child(root, TextInput::new("Cardholder name").boxed());
    let _ = harness.add_child(root, Landmark::search("Catalog search").boxed());
    harness.settle();

    // A landmark is not a control (the M143 rule, at the growth tier):
    // the furniture holds no focus — the controls the app mounts inside
    // it do. Tab walks past the aside and the form into the field; the
    // furniture is never a stop.
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
        "the walk landed the field, never the furniture"
    );
    harness.assert_all_named();
}

#[test]
fn m144_the_dsl_landmark_words_declare_them() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 300.0);

    // The DSL doors parse the new words: the page furniture's truth
    // rides the words, at mount, with no new prop class to pay for (the
    // M143 landmark doors' own shape).
    let view = alacer_macro::rsx! {
        column {
            complementary {
                name: "Related reading",
                label {
                    text: "Reading list",
                }
            }
            form {
                name: "Checkout details",
                label {
                    text: "Pay here",
                }
            }
            region {
                name: "Featured panel",
                label {
                    text: "This week's picks",
                }
            }
            search {
                name: "Catalog search",
                label {
                    text: "Find a widget",
                }
            }
        }
    };
    let _root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    let aside = node(&harness, Role::Complementary, "Related reading")
        .expect("the view's complementary rides the snapshot");
    assert_eq!(
        aside.role,
        Role::Complementary,
        "the DSL's word declares the tier"
    );
    let form =
        node(&harness, Role::Form, "Checkout details").expect("the view's form rides the snapshot");
    assert_eq!(form.role, Role::Form);
    let region = node(&harness, Role::Region, "Featured panel")
        .expect("the view's region rides the snapshot");
    assert_eq!(region.role, Role::Region);
    let search = node(&harness, Role::Search, "Catalog search")
        .expect("the view's search rides the snapshot");
    assert_eq!(search.role, Role::Search);
    harness.assert_all_named();
}

#[test]
fn m144_the_corpus_pin_holds_at_the_m144_cut() {
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

    // The published corpus pin holds: the growth prices the words and
    // nothing else (the M143 rule), and its pricing door is the widgets
    // crate's own bench — the word-write pair the M143 cut carried
    // beside the growth's vocabulary case — riding *outside* the
    // published corpus: the published baseline's routes price the a11y
    // snapshot machinery (the M95 rule), and the coordinated rule grows
    // it at the next published cut, never at a widget bench's.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(152),
        "the published corpus pin holds at the M144 cut"
    );
}

#[test]
fn m144_the_living_documents_re_agree() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).expect("the file is readable");

    // The README status reflects the living cut, the register grew the
    // growth's and the gate's records, and the shipped-vocabulary line
    // carries the landmark words the M143 cut had left out (the drift
    // the growth fixes beside the words).
    let readme = read("README.md");
    assert!(
        readme.contains("M0 through M145"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("a hundred and fifty decision records"),
        "the register grew the M144 and M145 rows"
    );
    assert!(
        readme.contains("`Image` · `Landmark`"),
        "the shipped-vocabulary line carries the menu, image and landmark tiers"
    );
    assert!(
        read("docs/architecture.md").contains("a hundred and fifty decision records"),
        "the architecture carries the growth's decision"
    );

    // The examples catalog grew the growth's rows: the landmarks demo
    // reads the tier's eight words, and the reconnect walkthrough
    // joined beside the subscribe one.
    let catalog = read("docs/examples.md");
    assert!(
        catalog.contains("examples/landmarks.rs"),
        "the catalog carries the landmark demonstration"
    );
    assert!(
        catalog.contains("examples/inspector_reconnect.rs"),
        "the catalog carries the reconnect demonstration"
    );

    // The changelog names the growth and the gate.
    let changelog = read("CHANGELOG.md");
    assert!(
        changelog.contains("M144 — the landmark tier's remaining words"),
        "the changelog names the growth"
    );
    assert!(
        changelog.contains("M145 — the gate persists across reconnects"),
        "the changelog names the gate"
    );

    // The m81 suite's documents pin moved to the living cut (the latest
    // suite carrying the living-documents pin).
    let m81 = read("crates/alacer-testing/tests/m81_exit_criteria.rs");
    assert!(
        m81.contains("M0 through M145"),
        "the m81 suite's pins moved with the documents"
    );
}
