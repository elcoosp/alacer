//! M111 exit-criteria integration tests (§10.3, ADR-0115): the
//! auto-complete tier — the field's suggestion contract, said by the
//! tree. Through M110 the snapshot said where a member sits (the M6
//! table slot), what a control *does* (the description, ADR-0099),
//! what an empty field *hints* (the placeholder, ADR-0100), the span a
//! value rides (the range tier, ADR-0101), the form's ask (the required
//! tier, ADR-0110), the form's rejection (the invalid tier, ADR-0111),
//! the collection's pick and capacity (the selected and
//! multiselectable tiers, ADR-0112/ADR-0113), the member's disclosure
//! (the expanded tier, ADR-0114) — and the word every field vocabulary
//! speaks about the suggestions it offers ("this one completes inline,
//! that one lists them") had no tier: the AT announced the hint, the
//! span, the form facts — and never the suggestion contract — the
//! declared-but-unreported gap (the M94 pattern), cashed at the
//! auto-complete tier: the word grows beside the field words it
//! explains — [`alacer_core::Widget::auto_complete`] beside
//! [`alacer_widgets::TextInput::auto_complete`], AccessKit's
//! `AutoComplete` property (ARIA's `aria-autocomplete`) the sink maps
//! in the same snapshot every other property rides.
//!
//! The tier prices the word and nothing else: an auto-completed field's
//! doors answer exactly as they always did — the word names the
//! suggestion contract, it walls nothing, the field stays editable.
//! ARIA's `none` value is the honest absence spelled by `None` — a
//! field that declared nothing announces no suggestion contract, never
//! a guessed mode; the kind the word carries (`Inline` / `List` /
//! `Both`) mirrors AccessKit's own enum values, so the mapping is one
//! branch with no translation.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the word's node.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, AutoCompleteKind, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, Slider, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M111 cut —
/// the auto-complete route's case joining the expanded route's pair
/// (M110), the multiselectable route's pair (M109), the selected
/// route's pair (M108), the invalid route's case (M107) and every
/// tier's before them on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 140;

/// The suite's own read door into the published corpus: the
/// auto-complete route's case — the 1,000-node field arena carrying the
/// suggestion contract — priced against the same 1,000-node field arena
/// bare (`placeholder/snapshot_1k_bare`, the M96 case; the M95 pricing
/// rule: the word is a field word, so the honest comparison is fields
/// against fields, the M106/M107 denominator).
const AUTO_COMPLETE_CASE: &str = "auto_complete/snapshot_1k_auto_complete";
const FIELDS_BARE_CASE: &str = "placeholder/snapshot_1k_bare";

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The app's auto-complete door: the mutate seam the app's code rides —
/// the word moved, `PAINT | A11Y` requested on the field, the frames
/// the re-announcement rides settled before the read. `None` clears the
/// word — the honest absence restored.
fn set_auto_complete(
    harness: &mut TestHarness,
    node_id: alacer_core::NodeId,
    auto_complete: Option<AutoCompleteKind>,
) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node_id);
    if let Some(field) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        field.set_auto_complete(auto_complete);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m111_the_auto_complete_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 320.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The worded fields, mounted through the constructor word: the
    // suggestion contract the tree says from the first frame — the
    // field that lists its completions, the field that completes
    // inline, the field that does both — beside the plain field, the
    // honest absence the tier keeps honest (ARIA's `none`, never a
    // guessed mode).
    let city = TextInput::new("City").auto_complete(AutoCompleteKind::List);
    let _ = harness.add_child(root, city.boxed());
    let street = TextInput::new("Street").auto_complete(AutoCompleteKind::Inline);
    let _ = harness.add_child(root, street.boxed());
    let query = TextInput::new("Query").auto_complete(AutoCompleteKind::Both);
    let _ = harness.add_child(root, query.boxed());
    // The value-tier control and the plain widgets beside them: no
    // suggestion contract to say.
    let _ = harness.add_child(root, Slider::new("Amount", 0.0, 100.0).boxed());
    let _ = harness.add_child(root, Label::new("Plain").boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    let city = node(&harness, Role::TextInput, "City").expect("the listing field rides");
    assert_eq!(
        city.auto_complete,
        Some(AutoCompleteKind::List),
        "the constructor's word is the tree's truth — the listing field said"
    );
    let street = node(&harness, Role::TextInput, "Street").expect("the inline field rides");
    assert_eq!(
        street.auto_complete,
        Some(AutoCompleteKind::Inline),
        "the kind the word carries is the field's own"
    );
    let query = node(&harness, Role::TextInput, "Query").expect("the both field rides");
    assert_eq!(
        query.auto_complete,
        Some(AutoCompleteKind::Both),
        "the compound contract rides the same word"
    );
    let note = node(&harness, Role::Slider, "Amount").expect("the value tier rides");
    assert!(
        note.auto_complete.is_none(),
        "a control that declared nothing answers no suggestion contract"
    );
    let save = node(&harness, Role::Button, "Save").expect("the button rides");
    assert!(
        save.auto_complete.is_none(),
        "a non-input role inherits the honest absence"
    );

    // The word prices nothing but the word: the worded field's write
    // door answers exactly as any editable's.
    assert!(
        harness.act(
            Role::TextInput,
            "City",
            A11yAction::SetValue("Paris".into())
        ),
        "the suggestion contract is not a wall — the field stays editable"
    );
    let filled = node(&harness, Role::TextInput, "City").expect("the field rides");
    assert_eq!(
        filled.value.as_deref(),
        Some("Paris"),
        "the AT's write lands on the auto-completed field as ever"
    );
    harness.assert_all_named();
}

#[test]
fn m111_the_auto_complete_door_moves_the_word_live() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let note = TextInput::new("Note");
    let note_id = harness.add_child(root, note.boxed());
    harness.settle();

    // The live door: the app's suggestion contract moved, the
    // re-announcement the mutate seam's request paid for — the kind
    // riding beside the word, and the honest absence restored on the
    // clear.
    set_auto_complete(&mut harness, note_id, Some(AutoCompleteKind::List));
    let listed = node(&harness, Role::TextInput, "Note").expect("the field survives");
    assert_eq!(
        listed.auto_complete,
        Some(AutoCompleteKind::List),
        "the app's door moves the word the tree says, kind beside it"
    );
    set_auto_complete(&mut harness, note_id, Some(AutoCompleteKind::Inline));
    let retyped = node(&harness, Role::TextInput, "Note").expect("the field survives");
    assert_eq!(
        retyped.auto_complete,
        Some(AutoCompleteKind::Inline),
        "the same door moves the kind — the AT reads the new contract"
    );
    set_auto_complete(&mut harness, note_id, None);
    let cleared = node(&harness, Role::TextInput, "Note").expect("the field survives");
    assert!(
        cleared.auto_complete.is_none(),
        "the same door carries the word away — the honest absence restored"
    );

    // The write that lands where the word already is queues nothing:
    // the door answers `false`, the caller requests no re-announcement.
    let mut guard = harness.pipeline_mut().arena_mut().mutate(note_id);
    let moved = guard
        .widget_mut()
        .as_any_mut()
        .downcast_mut::<TextInput>()
        .expect("the field survives")
        .set_auto_complete(None);
    drop(guard);
    harness.settle();
    assert!(
        !moved,
        "the write that lands where the word already is queues nothing — the M21 idle-frame property"
    );
    harness.assert_all_named();
}

#[test]
fn m111_the_corpus_grew_by_the_coordinated_rule() {
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
    let contracts = serde_json::from_str::<serde_json::Value>(
        &std::fs::read_to_string(root.join("benchmarks/contracts.json"))
            .expect("the contract set is readable"),
    )
    .expect("the contract set is valid JSON");

    // The published corpus carries the auto-complete route's case: the
    // 1,000-node field arena carrying the suggestion contract — the
    // word's cost the measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(AUTO_COMPLETE_CASE),
        "the auto-complete route's case is published ({AUTO_COMPLETE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M111 cut"
    );

    // The contract set gates the case and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    let gate = gates
        .iter()
        .find(|gate| gate["baseline"].as_str() == Some(AUTO_COMPLETE_CASE))
        .unwrap_or_else(|| panic!("the case carries a regression gate ({AUTO_COMPLETE_CASE})"));
    assert_eq!(
        gate["scope"].as_str(),
        Some("same-host"),
        "the gate is same-host scoped like the tier's own"
    );
    let budgets = contracts["budgets"]
        .as_array()
        .expect("the contract set carries budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"].as_str() == Some("auto_complete_snapshot_overhead"))
        .expect("the auto-complete budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(AUTO_COMPLETE_CASE),
        "the budget prices the auto-complete route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(FIELDS_BARE_CASE),
        "the budget prices it against the same field arena, bare"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the word must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M111 cut"
    );
}
