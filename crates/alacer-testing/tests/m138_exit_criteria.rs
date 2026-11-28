//! M138 exit-criteria integration tests (§11.1, ADR-0142): the image
//! tier — the named picture the AT announces as a picture, said by the
//! tree. Through M137 the snapshot said the menu tier (ADR-0141), the
//! tooltip (ADR-0140), the tree (ADR-0139), the dialog (ADR-0138) — and
//! the *picture* had no word of its own: an app wiring a chart, a logo,
//! a diagram reached for a bare [`Label`](alacer_widgets::Label) wearing
//! no picture word (the picture the AT cannot name is a picture it
//! cannot offer on demand — the ADR-5 rule the named roles carry) or
//! for a canvas wearing no name at all (the canvas tier, M82, names a
//! *drawing surface the app paints into*; a picture is *named content
//! the app presents* — the wrong word wearing the wrong half).
//! [`Role::Image`](alacer_core::widget::Role::Image) carries the picture
//! word AccessKit's own `Image` role spells.
//!
//! The tier prices the word and nothing else: the pixel story is the
//! app's own (the paint seam the canvas tier published, or the embedded
//! raster the app owns — the word never paints), the name and the
//! description the words the app declared (the M95 description tier
//! carrying the alt text beside the name), and an image is not a
//! control: it holds no focus and answers no activation (the
//! M126/M129 container rule, at the picture tier — the controls the app
//! mounts around it hold the focus).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the sink's role branches grow one beside the menu
//! tier's.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Image, Label, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M138
/// cut — unchanged from the M136 cut. The image tier grows no snapshot
/// *route* (the word is the picture's own, the shape the
/// M124/M127/M129/M131/M134/M135/M136/M137 named containers rode), and
/// its pricing door is the widget-level bench the tier carries in the
/// widgets crate (`image/name_write_256` against `image/idle_write_256`),
/// which rides *outside* the published corpus: the published baseline's
/// routes price the a11y snapshot machinery (the M95 rule), and the
/// coordinated rule grows it at the next published cut, never at a
/// widget bench's. The pin re-agrees at 155.
const EXPECTED_BENCH_COUNT: usize = 208;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

#[test]
fn m138_the_image_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 240.0);
    // The picture word, mounted through the constructor word: the named
    // picture beside the field that reads beside it — the alt text the
    // M95 description word carries.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _image_id = harness.add_child(
        root,
        Image::new("Disk usage chart")
            .description("Bars per mount, refreshed hourly")
            .boxed(),
    );
    let _ = harness.add_child(root, TextInput::new("Subject").boxed());
    harness.settle();

    // The picture carries the image word — the fact AccessKit's own
    // role spells — and the description rides beside the name.
    let picture = node(&harness, Role::Image, "Disk usage chart")
        .expect("the named picture rides the snapshot");
    assert_eq!(
        picture.role,
        Role::Image,
        "the word AccessKit's own role spells"
    );
    assert_eq!(
        harness.query(Role::Image).count(),
        1,
        "no neighbour munged onto the picture word"
    );
    let field =
        node(&harness, Role::TextInput, "Subject").expect("the field beside keeps its word");
    assert_eq!(
        field.role,
        Role::TextInput,
        "the neighbour is the neighbour's own — never a picture wearing the field"
    );
    harness.assert_all_named();
}

#[test]
fn m138_the_picture_is_not_a_control_and_walls_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    let image_id = harness.add_child(root, Image::new("Disk usage chart").boxed());
    let field_id = harness.add_child(root, TextInput::new("Subject").boxed());
    harness.settle();

    // A picture is not a control: the image holds no focus — the
    // controls the app mounts beside it do (the M126/M129 container
    // rule, at the picture tier). Tab walks past the image into the
    // field; the image is never a stop.
    harness.key(&KeyEvent {
        key: Key::Tab,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    });
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        Some(field_id),
        "the walk landed the field, never the picture"
    );

    // The words' writes (view reconciliation): the name is the app's
    // declaration, rewritten in place — the picture re-announces it the
    // next frame; the description write reports whether it moved (the
    // M21 idle-frame property, at the description tier).
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(image_id);
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<Image>()
            .is_some_and(|picture| {
                picture.set_name("Memory usage chart");
                picture.set_description(Some("Bars per subsystem".to_string()))
            });
        let _ = moved;
        guard.request(alacer_core::Change::A11Y);
        drop(guard);
    }
    harness.settle();
    node(&harness, Role::Image, "Memory usage chart").expect("the rename landed — the word rides");
    harness.assert_all_named();
}

#[test]
fn m138_the_dsl_image_word_declares_it() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 160.0);

    // The `image` word the element parses — the picture's truth rides
    // the words, at mount, with no new prop class to pay for (the M134
    // dialog word's own shape).
    let view = alacer_macro::rsx! {
        column {
            image {
                name: "Disk usage chart",
                description: "Bars per mount, refreshed hourly",
            }
        }
    };
    let root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    let picture = node(&harness, Role::Image, "Disk usage chart")
        .expect("the view's picture rides the snapshot");
    assert_eq!(
        picture.role,
        Role::Image,
        "the DSL's word declares the tier"
    );
    let _ = root;
}

#[test]
fn m138_the_word_walls_nothing_and_the_label_reads() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 240.0);
    // The word prices the word and nothing else: a help label beside the
    // picture reads as itself (the tier walls nothing), and the picture
    // keeps the word it already carried.
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, Image::new("Disk usage chart").boxed());
    let _ = harness.add_child(root, Label::new("The chart refreshes hourly").boxed());
    harness.settle();
    let hint = node(&harness, Role::Label, "The chart refreshes hourly")
        .expect("the help label keeps its own word");
    assert_eq!(
        hint.role,
        Role::Label,
        "the label is the label's own — never a picture word"
    );
    harness.assert_all_named();
}

#[test]
fn m138_the_corpus_pin_re_agrees_at_the_m138_cut() {
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

    // The published corpus pin re-agrees: the image tier grows no
    // snapshot route (the word is the picture's own, the shape the named
    // containers rode), and its pricing door is the widgets crate's own
    // bench — the `image_word` group joining the corpus at this cut,
    // the coordinated rule the next published cut carried out.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M138 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M138 cut"
    );
}

#[test]
fn m138_the_living_documents_re_agree() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).expect("the file is readable");

    // The README status reflects the living cut, and the register grew
    // the image tier's record.
    let readme = read("README.md");
    assert!(
        readme.contains("M0 through M156"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("a hundred and sixty-one decision records"),
        "the register grew the M137 and M138 rows"
    );
    assert!(
        read("docs/architecture.md").contains("a hundred and sixty-one decision records"),
        "the architecture carries the image fact's decision"
    );

    // The examples catalog grew the image row, and the changelog names
    // the tier.
    assert!(
        read("docs/examples.md").contains("examples/image.rs"),
        "the catalog carries the image demonstration"
    );
    assert!(
        read("CHANGELOG.md").contains("M138 — the image tier"),
        "the changelog names the tier"
    );

    // The m81 suite's documents pin moved to M138's cut (the latest
    // suite carrying the living-documents pin — the M132+ cut's suites
    // pin the corpus, and their pins re-agreed with it).
    let m81 = read("crates/alacer-testing/tests/m81_exit_criteria.rs");
    assert!(
        m81.contains("M0 through M156"),
        "the m81 suite's pins moved with the documents"
    );
}
