//! M102 exit-criteria integration tests (§10.3, ADR-0106): the reveal
//! tier — ADR-0104's first pre-registered re-open rule cashed. Since
//! M100 a secured field's tree answer is AccessKit's `PasswordInput`
//! role and a masked value (one masking character per code point,
//! never the secret), and the constructor word was the whole story —
//! a "show password" toggle had no word: the tree kept saying
//! `PasswordInput` while the pixels showed the secret, and the AT kept
//! reading the mask the user had just revealed. The flag grows the
//! revealed state beside the constructor's —
//! [`alacer_core::Widget::secure_revealed`] beside
//! [`alacer_widgets::TextInput::set_revealed`] — AccessKit's own
//! contract naming the mapping ("when the password is revealed, use
//! `Role::TextInput` instead"): the sink's role branch reads the
//! ordinary editable word, and the value the tree resolves is the
//! field's ordinary value again — the mask yielding to the reveal, the
//! secret riding only what the app chose to show.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the app's reveal door rides the mutate seam whose
//! `PAINT | A11Y` request re-announces exactly the field.

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M102 cut —
/// the reveal route's case joining the scroll position route's case
/// (M101), the secure route's case (M100), the numeric route's case
/// (M99), the orientation route's case (M98), the range route's case
/// (M97), the placeholder route's pair (M96), the description route's
/// case (M95), the toggle route's pair (M94), the derive route's pair
/// (M92), and the recording route's pair (M93) on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 121;

/// The suite's own read door into the published corpus: the reveal
/// route's case — the 1,000-node revealed field arena (the coordinated
/// rule's bench half, ADR-0106's exit criteria). The ratio's honest
/// denominator is the same 1,000-node secured arena
/// (`secure/snapshot_1k_secured`, the M100 case), the M95 pricing
/// rule: the same corpus measuring what the tier adds.
const REVEALED_CASE: &str = "secure_revealed/snapshot_1k_revealed";
const SECURED_CASE: &str = "secure/snapshot_1k_secured";

/// The first text input in the snapshot whose name contains `name`.
fn input_node(harness: &TestHarness, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, name).cloned())
}

/// The app's reveal door: the mutate seam the app's code rides — the
/// flag moved, `PAINT | A11Y` requested on the field, the frames the
/// re-announcement rides settled before the read.
fn set_revealed(harness: &mut TestHarness, node: alacer_core::NodeId, revealed: bool) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_revealed(revealed);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

#[test]
fn m102_the_revealed_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The secured field, mounted revealed: the constructor names the
    // initial state, the tree's answer the ordinary editable word.
    let secured = TextInput::new("Passphrase").secure().revealed();
    let secured_id = harness.add_child(root, secured.boxed());
    // The secured field beside it, masked as M100 always answered.
    let masked = TextInput::new("Notes").secure();
    let _ = harness.add_child(root, masked.boxed());
    // The absence on a plain field: no secret entry declared, no
    // revealed word to say.
    let _ = harness.add_child(root, TextInput::new("Visible").boxed());
    harness.settle();

    let revealed = input_node(&harness, "Passphrase").expect("the revealed field rides");
    assert!(
        revealed.secure_entry,
        "the constructor's word is still the tree's truth"
    );
    assert!(
        revealed.secure_revealed,
        "the revealed state rides beside it — the compound word"
    );
    let masked = input_node(&harness, "Notes").expect("the masked field rides");
    assert!(
        masked.secure_entry && !masked.secure_revealed,
        "a field that revealed nothing says the secure word alone"
    );
    let plain = input_node(&harness, "Visible").expect("the plain field rides");
    assert!(
        !plain.secure_entry && !plain.secure_revealed,
        "a field that declared no secret entry answers no revealed word"
    );

    // The live door: the app's reveal moves the word, the re-announcement
    // the mutate seam's request paid for.
    set_revealed(&mut harness, secured_id, false);
    let hidden = input_node(&harness, "Passphrase").expect("the field survives");
    assert!(
        !hidden.secure_revealed,
        "the app's hide door moves the word the tree says"
    );
    harness.assert_all_named();
}

#[test]
fn m102_the_mask_yields_to_the_reveal() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let secured = TextInput::new("Passphrase").secure();
    let secured_id = harness.add_child(root, secured.boxed());
    harness.settle();

    // The write door stays open: the AT fills the field (a password
    // manager's ask), the tree answers the mask — the M100 contract.
    assert!(
        harness.act(
            Role::TextInput,
            "Passphrase",
            A11yAction::SetValue("hunter2".into())
        ),
        "a secure field is editable"
    );
    let masked = input_node(&harness, "Passphrase").expect("the field rides");
    assert_eq!(
        masked.value.as_deref(),
        Some("•••••••"),
        "the masked field carries the mask, one bullet per code point"
    );

    // The reveal: the app's door opens the field — the role the tree
    // implies is the ordinary editable (AccessKit's own contract names
    // it), and the value the tree resolves is the field's ordinary
    // value again: the pixels and the tree telling one truth.
    set_revealed(&mut harness, secured_id, true);
    let revealed = input_node(&harness, "Passphrase").expect("the field survives");
    assert!(
        revealed.secure_revealed,
        "the revealed word rides the re-announcement"
    );
    assert_eq!(
        revealed.value.as_deref(),
        Some("hunter2"),
        "the mask yields to the reveal — the value the pixels show is the value the tree says"
    );

    // The hide: the mask returns, the length tracking the value.
    set_revealed(&mut harness, secured_id, false);
    let hidden = input_node(&harness, "Passphrase").expect("the field survives");
    assert!(
        !hidden.secure_revealed,
        "the hide door restores the masked answer"
    );
    assert_eq!(
        hidden.value.as_deref(),
        Some("•••••••"),
        "the mask's length tracks the committed value's code points"
    );
}

#[test]
fn m102_the_revealed_field_answers_every_door() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);
    let root = harness.mount(Column::new().boxed());
    // The revealed field is still an editable: the AT's write and the
    // selection doors answer exactly as any plain input's — the tier
    // prices the word, never the field's ordinary honesty.
    let secured = TextInput::new("Passphrase").secure().revealed();
    let _ = harness.add_child(root, secured.boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    assert!(
        harness.act(
            Role::TextInput,
            "Passphrase",
            A11yAction::SetValue("correct horse".into())
        ),
        "the revealed field's write door stays open"
    );
    let node = input_node(&harness, "Passphrase").expect("the field survives");
    assert_eq!(
        node.value.as_deref(),
        Some("correct horse"),
        "the write lands raw — the field is revealed, the tree says the value"
    );
    let button = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Button, "Save").cloned())
        .expect("the button rides the snapshot");
    assert!(
        !button.secure_entry && !button.secure_revealed,
        "a non-input role inherits the absence on both words"
    );
    harness.assert_all_named();
}

#[test]
fn m102_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the reveal route's case: the
    // 1,000-node revealed field arena — the word's cost the
    // measurement, priced against the same arena masked.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(REVEALED_CASE) && benches.contains_key(SECURED_CASE),
        "the reveal route's case and its denominator are published ({REVEALED_CASE}, {SECURED_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M102 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [REVEALED_CASE, SECURED_CASE] {
        let gate = gates
            .iter()
            .find(|gate| gate["baseline"].as_str() == Some(case))
            .unwrap_or_else(|| panic!("the case carries a regression gate ({case})"));
        assert_eq!(
            gate["scope"].as_str(),
            Some("same-host"),
            "the gate is same-host scoped like the tier's own"
        );
    }
    let budgets = contracts["budgets"]
        .as_array()
        .expect("the contract set carries budgets");
    let budget = budgets
        .iter()
        .find(|budget| budget["id"].as_str() == Some("secure_revealed_snapshot_overhead"))
        .expect("the reveal budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(REVEALED_CASE),
        "the budget prices the revealed route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(SECURED_CASE),
        "the budget prices it against the same arena, masked"
    );
    assert_eq!(
        budget["op"].as_str(),
        Some("<="),
        "the reveal must not outprice the snapshot it rides"
    );

    // The M5 pin re-agrees: the source pin the M6-M12 suites assert
    // against reads the cut's count.
    let m5 = std::fs::read_to_string(root.join("crates/alacer-testing/tests/m5_exit_criteria.rs"))
        .expect("the m5 suite is readable");
    let expected = format!("const EXPECTED_BENCH_COUNT: usize = {EXPECTED_BENCH_COUNT};");
    assert!(
        m5.contains(&expected),
        "the m5 corpus pin re-agrees at the M102 cut"
    );
}
