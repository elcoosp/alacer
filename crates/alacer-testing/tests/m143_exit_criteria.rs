//! M143 exit-criteria integration tests (§11.1, ADR-0147): the landmark
//! tier — the page-structure words the AT walks a document by, said by
//! the tree. Through M138 the snapshot said the image tier (ADR-0142),
//! the menu (ADR-0141), the tooltip (ADR-0140), the tree (ADR-0139),
//! the dialog (ADR-0138) — and the app's *page furniture* had no words:
//! an app wiring a header, a navigation strip, a main region and a
//! footer reached for bare panels wearing no container word (the region
//! the AT cannot name is a region it cannot offer on demand, the ADR-5
//! rule the named roles carry) — or for a toolbar wearing the wrong
//! word (a toolbar *offers tools*, a navigation strip *names the way
//! through the document*). [`Role::Banner`](alacer_core::widget::Role::Banner),
//! [`Role::Navigation`](alacer_core::widget::Role::Navigation),
//! [`Role::Main`](alacer_core::widget::Role::Main) and
//! [`Role::ContentInfo`](alacer_core::widget::Role::ContentInfo) carry
//! the page-structure words AccessKit's own roles spell — the landmark
//! vocabulary the AT's page walk (its landmarks list) reads.
//!
//! The tier prices the words and nothing else: a landmark is the page
//! furniture's layout, never a second channel to it (the children ride
//! the arena exactly a pane's do, each keeping the role and label it
//! already carries), and a landmark is not a control: it holds no focus
//! and answers no activation (the M126/M129 container rule, at the
//! landmark tier — the controls the app mounts inside it hold the
//! focus).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the sink's role branches grow one beside the
//! image's.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Landmark, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M143
/// cut — the subscribe ask's case the M142 cut published (the wire
/// ask's fan-out price riding the corpus it gates). The landmark tier
/// grows no snapshot *route* (the words are the containers' own, the
/// shape the M124/M127/M129/M131/M134/M135/M136 named containers rode),
/// and its pricing door is the widget-level bench the tier carries in
/// the widgets crate (`landmark/name_write_256` against
/// `landmark/idle_write_256`), which rides *outside* the published
/// corpus: the published baseline's routes price the a11y snapshot
/// machinery (the M95 rule), and the coordinated rule grows it at the
/// next published cut, never at a widget bench's. The pin re-agrees at
/// 155.
const EXPECTED_BENCH_COUNT: usize = 204;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m143_the_landmark_words_land_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 400.0);
    // The page furniture, mounted through the constructor words: the
    // banner, the navigation strip, the main region and the footer —
    // each a named container, each child keeping the word it already
    // carries.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let banner_id = harness.add_child(root, Landmark::banner("Catalog").boxed());
    let _ = harness.add_child(banner_id, Label::new("The Alacer Catalog").boxed());
    let nav_id = harness.add_child(root, Landmark::navigation("Catalog sections").boxed());
    let _ = harness.add_child(nav_id, Label::new("Widgets").boxed());
    let main_id = harness.add_child(root, Landmark::main("The catalog").boxed());
    let _ = harness.add_child(
        main_id,
        Label::new("Everything the vocabulary ships").boxed(),
    );
    let footer_id = harness.add_child(root, Landmark::content_info("Catalog colophon").boxed());
    let _ = harness.add_child(footer_id, Label::new("The Alacer Developers").boxed());
    harness.settle();

    // Each landmark carries its own word — AccessKit's page-structure
    // facts said by the tree; the children keep the words they
    // published (the landmark their layout, never a second channel).
    let banner =
        node(&harness, Role::Banner, "Catalog").expect("the named banner rides the snapshot");
    assert_eq!(
        banner.role,
        Role::Banner,
        "the word AccessKit's own role spells"
    );
    let nav = node(&harness, Role::Navigation, "Catalog sections")
        .expect("the named navigation rides the snapshot");
    assert_eq!(nav.role, Role::Navigation);
    let main =
        node(&harness, Role::Main, "The catalog").expect("the named main rides the snapshot");
    assert_eq!(main.role, Role::Main);
    let footer = node(&harness, Role::ContentInfo, "Catalog colophon")
        .expect("the named footer rides the snapshot");
    assert_eq!(footer.role, Role::ContentInfo);
    let heading = node(&harness, Role::Label, "The Alacer Catalog")
        .expect("the banner's content keeps its own word");
    assert_eq!(
        heading.role,
        Role::Label,
        "the banner's content is the content's own — never a banner wearing the child"
    );
    // One word per node: each landmark's word is the only one of its
    // kind in the tree — the neighbours keep the words they published.
    assert_eq!(
        harness.query(Role::Banner).count(),
        1,
        "no neighbour munged onto the banner word"
    );
    assert_eq!(harness.query(Role::Navigation).count(), 1);
    assert_eq!(harness.query(Role::Main).count(), 1);
    assert_eq!(harness.query(Role::ContentInfo).count(), 1);
    harness.assert_all_named();
}

#[test]
fn m143_the_landmarks_are_not_controls_and_wall_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Landmark::banner("Catalog").boxed());
    let _ = harness.add_child(root, Landmark::navigation("Catalog sections").boxed());
    let field_id = harness.add_child(root, TextInput::new("Subject").boxed());
    let _ = harness.add_child(root, Landmark::content_info("Catalog colophon").boxed());
    harness.settle();

    // A landmark is not a control: the furniture holds no focus — the
    // controls the app mounts inside it do (the M126/M129 container
    // rule, at the landmark tier). Tab walks past the banner and the
    // navigation into the field; the furniture is never a stop.
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

    // The word's one write (view reconciliation): the name is the app's
    // declaration, rewritten in place — the landmark re-announces it
    // the next frame. The write that lands where the word already is
    // queues nothing (the M21 idle-frame property, at the landmark
    // tier).
    let main_id = harness.add_child(root, Landmark::main("The catalog").boxed());
    harness.settle();
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(main_id);
        let renamed = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Landmark>()
            .is_some_and(|landmark| {
                landmark.set_name("The whole catalog");
                true
            });
        let _ = renamed;
        guard.request(alacer_core::Change::A11Y);
        drop(guard);
    }
    harness.settle();
    node(&harness, Role::Main, "The whole catalog").expect("the rename landed — the word rides");
    harness.assert_all_named();
}

#[test]
fn m143_the_dsl_landmark_words_declare_them() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 300.0);

    // The `main`, `navigation`, `banner` and `content_info` words the
    // elements parse — the page furniture's truth rides the words, at
    // mount, with no new prop class to pay for (the M136 tooltip
    // word's own shape).
    let view = alacer_macro::rsx! {
        column {
            banner {
                name: "Catalog",
                label {
                    text: "The Alacer Catalog",
                }
            }
            navigation {
                name: "Catalog sections",
                label {
                    text: "Widgets",
                }
            }
            main {
                name: "The catalog",
                label {
                    text: "Everything the vocabulary ships",
                }
            }
            content_info {
                name: "Catalog colophon",
                label {
                    text: "The Alacer Developers",
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
    let banner =
        node(&harness, Role::Banner, "Catalog").expect("the view's banner rides the snapshot");
    assert_eq!(
        banner.role,
        Role::Banner,
        "the DSL's word declares the tier"
    );
    let nav = node(&harness, Role::Navigation, "Catalog sections")
        .expect("the view's navigation rides the snapshot");
    assert_eq!(nav.role, Role::Navigation);
    let main =
        node(&harness, Role::Main, "The catalog").expect("the view's main rides the snapshot");
    assert_eq!(main.role, Role::Main);
    let footer = node(&harness, Role::ContentInfo, "Catalog colophon")
        .expect("the view's footer rides the snapshot");
    assert_eq!(footer.role, Role::ContentInfo);
    let _ = root;
    harness.assert_all_named();
}

#[test]
fn m143_the_corpus_pin_re_agrees_at_the_m143_cut() {
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

    // The published corpus pin re-agrees: the landmark tier grows no
    // snapshot route (the words are the named containers' own), and its
    // pricing door is the widgets crate's own bench — outside the
    // published corpus, the coordinated rule growing it at the next
    // published cut. The count is the M142 cut's: the subscribe ask's
    // fan-out price the case the corpus grew by.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M143 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M143 cut"
    );
}

#[test]
fn m143_the_living_documents_re_agree() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).expect("the file is readable");

    // The README status reflects the living cut, and the register grew
    // the landmark tier's record.
    let readme = read("README.md");
    assert!(
        readme.contains("M0 through M155"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("a hundred and sixty decision records"),
        "the register grew the M143 row"
    );
    assert!(
        read("docs/architecture.md").contains("a hundred and sixty decision records"),
        "the architecture carries the landmark fact's decision"
    );

    // The examples catalog grew the landmark row, and the changelog
    // names the tier.
    assert!(
        read("docs/examples.md").contains("examples/landmarks.rs"),
        "the catalog carries the landmark demonstration"
    );
    assert!(
        read("CHANGELOG.md").contains("M143 — the landmark tier"),
        "the changelog names the tier"
    );

    // The m81 suite's documents pin moved to M143's cut (the latest
    // suite carrying the living-documents pin — the M132+ cut's suites
    // pin the corpus, and their pins re-agreed with it).
    let m81 = read("crates/alacer-testing/tests/m81_exit_criteria.rs");
    assert!(
        m81.contains("M0 through M155"),
        "the m81 suite's pins moved with the documents"
    );
}
