//! M132 exit-criteria integration tests (§10.3, ADR-0136): the
//! search-field tier — the field whose purpose is search, said by the
//! tree. Through M131 the snapshot said the tab-panel tier (ADR-0135),
//! the instant on/off control (ADR-0134), the tool strip (ADR-0133),
//! the bounded number field (ADR-0132) — and the text field's *purpose*
//! had no word: an app wiring a search box mounted a plain
//! [`alacer_widgets::TextInput`], and the AT read "edit text" where the
//! vocabulary says "search" — the field wearing no purpose the
//! platform's find conventions key off (AccessKit's `SearchInput` role
//! is its own fact, the word the vocabulary spells beside
//! `TextInput`). [`alacer_core::widget::Widget::search_field`] carries
//! the "this field takes search criteria" fact, the M100 secure tier's
//! own re-role shape (the flag the sink's role branch reads — a declared
//! purpose re-roles the field in the same snapshot every other property
//! rides).
//!
//! The word prices nothing but the word: a declared field's doors
//! answer exactly as they always did — the edit, the selection, the
//! IME contract unchanged — and the tree's other words keep their
//! precedence: the masked contract first (a secured field keeps the
//! `PasswordInput` word M100 mapped), the capacity word claiming a
//! field that declared both (a multiline search box announces
//! `MultilineTextInput` — AccessKit's vocabulary has no composite, the
//! tree never guesses one; the ADR pre-registers the re-open rule).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the sink's role branch grows one beside the
//! multiline/secure pair's.

use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M132
/// cut — unchanged from the M131 cut. The search-field tier grows no
/// snapshot *route* (the word rides the role branch the sink re-roles
/// in the snapshot every other property rides — a flag beside the
/// words the field already carries), and its pricing door is the
/// widget-level bench the tier carries in the widgets crate
/// (`search_field/purpose_write_256` against
/// `search_field/idle_write_256`), which rides *outside* the published
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
fn m132_the_purpose_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    // The search word, mounted through the constructor word: the
    // declared field beside the plain one and the button the canon
    // keeps honest — each field's word its own.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let _ = harness.add_child(
        root,
        TextInput::new("Mail search").search_field(true).boxed(),
    );
    let _ = harness.add_child(root, TextInput::new("Notes").boxed());
    let _ = harness.add_child(root, Button::new("Search").boxed());
    harness.settle();

    // The declared field carries the purpose word; the plain field's
    // honest absence rides beside it, and the button's word took
    // nothing.
    let search = node(&harness, Role::TextInput, "Mail search")
        .expect("the declared field rides the snapshot");
    assert!(
        search.search_field,
        "the tree carries the purpose word the app declared"
    );
    let plain = node(&harness, Role::TextInput, "Notes").expect("the plain field rides");
    assert!(
        !plain.search_field,
        "a field that declared nothing announces no purpose — the honest absence"
    );
    node(&harness, Role::Button, "Search").expect("the button keeps its word");
    harness.assert_all_named();
}

#[test]
fn m132_the_word_prices_nothing_but_the_word() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(360.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let field_id = harness.add_child(root, TextInput::new("Mail search").boxed());
    harness.settle();

    // The one programmatic write (view reconciliation): the purpose is
    // the app's declaration, flipped in place — the tree re-announces
    // the word the next frame, and the write that lands where the word
    // already is answers *not moved* (the M21 idle-frame property, at
    // the purpose tier).
    let moved = {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(field_id);
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<TextInput>()
            .is_some_and(|field| field.set_search_field(true));
        guard.request(alacer_core::Change::A11Y);
        drop(guard);
        moved
    };
    harness.settle();
    let field = node(&harness, Role::TextInput, "Mail search").expect("the field survives");
    assert!(moved, "the programmatic write reports the move");
    assert!(
        field.search_field,
        "the app's write landed — the word rides"
    );

    // The idle write: the write that lands where the word already is
    // answers *not moved* and queues no masks — the tick it would have
    // paid reports idle (the M21 idle-frame property, at the purpose
    // tier).
    let idle = {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(field_id);
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<TextInput>()
            .is_some_and(|field| field.set_search_field(true));
        drop(guard);
        moved
    };
    assert!(
        !idle,
        "the write landed where the word already is — not moved, no masks"
    );
    let report = harness.tick();
    assert!(
        report.is_idle(),
        "the idle write queues nothing — the frame after it is idle"
    );

    // The word walls nothing: the field edits exactly as it always did.
    let answers = harness.key(&alacer_core::events::KeyEvent {
        key: alacer_core::events::Key::Tab,
        state: alacer_core::events::KeyState::Pressed,
        modifiers: alacer_core::events::Modifiers::default(),
        text: None,
    });
    let _ = answers;
    harness.type_text("invoices");
    let field = node(&harness, Role::TextInput, "Mail search").expect("the field survives");
    assert_eq!(
        field.value.as_deref(),
        Some("invoices"),
        "the declared field edits like the field it always was — the word names, it walls nothing"
    );
    harness.assert_all_named();
}

#[test]
fn m132_the_dsl_search_word_declares_it() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    // The `search` word the text-input element parses — the tree's
    // truth rides the word, at mount, with no new prop class to pay
    // for (the M100 secure word's own shape).
    let view = alacer_macro::rsx! {
        column {
            text_input { name: "Mail search", search: true }
        }
    };
    let root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    let declared = node(&harness, Role::TextInput, "Mail search")
        .expect("the view's field rides the snapshot");
    assert!(
        declared.search_field,
        "the DSL's search word declares the field through the same door"
    );
    let _ = root;
}

#[test]
fn m132_the_corpus_pin_re_agrees_at_the_m132_cut() {
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

    // The published corpus pin re-agrees: the search-field tier grew no
    // snapshot route (the word rides the sink's role branch, a flag
    // beside the words the field already carries), and its pricing door
    // is the widgets crate's own bench — outside the corpus at its own cut, joined by the corpus catch-up's published cut — the coordinated rule's next published cut cashed.
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M132 cut"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M132 cut"
    );
}
