//! M100 exit-criteria integration tests (§10.3, ADR-0104): the secure
//! tier. Since M4 (ADR-7) a field can declare its own secret entry —
//! `TextInput::secure()` naming the IME contract's `ImePurpose::Secure`,
//! the platform suppressing preedit echo and suggestions — and the a11y
//! tree could never say it: the AT read a plain text input whose value
//! was the raw secret itself, a payload no tree carries (the clipboard
//! trail's rule, ADR-0045, at the tree tier). The declared-but-unreported
//! gap, the M94 pattern at the secure tier, cashed:
//! [`alacer_core::Widget::secure_entry`] beside the constructor that
//! already told the truth, the snapshot carrying it
//! (`TreeNode::secure_entry`) *and* masking the value the tree resolves
//! (one masking character per code point — never the secret), and the
//! sink mapping it — AccessKit's `PasswordInput` role, the word every
//! platform AT announces, whose own contract the masked value now
//! satisfies.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), the counters are the pass ledger's own (A4), the view
//! tier is the consumer's route (§13.1).

use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M100 cut —
/// the secure route's case joining the numeric route's case (M99), the
/// orientation route's case (M98), the range route's case (M97), the
/// placeholder route's pair (M96), the description route's case (M95),
/// the toggle route's pair (M94), the derive route's pair (M92), and the
/// recording route's pair (M93) on the published corpus.
const EXPECTED_BENCH_COUNT: usize = 120;

/// The suite's own read door into the published corpus: the secure
/// route's case — the 1,000-node secured input arena (the coordinated
/// rule's bench half, ADR-0104's exit criteria). The ratio's honest
/// denominator is the same 1,000-node input arena bare
/// (`placeholder/snapshot_1k_bare`, the M96 bare case), the M95 pricing
/// rule: the same corpus measuring what the tier adds.
const SECURED_CASE: &str = "secure/snapshot_1k_secured";
const BARE_CASE: &str = "placeholder/snapshot_1k_bare";

/// The first text input in the snapshot whose name contains `name`.
fn input_node(harness: &TestHarness, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, name).cloned())
}

#[test]
fn m100_the_secure_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The secured constructor: the field the platform already treats as
    // a password — the tree says the word at last.
    let _ = harness.add_child(root, TextInput::new("Passphrase").secure().boxed());
    // The bare field beside it: the honest absence — the tree reads
    // exactly as it read before the tier.
    let _ = harness.add_child(root, TextInput::new("Notes").boxed());
    // The absence on a non-input role: a button answers nothing.
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    let secured = input_node(&harness, "Passphrase").expect("the secured field rides the snapshot");
    assert!(
        secured.secure_entry,
        "the constructor's word is the tree's truth"
    );
    let bare = input_node(&harness, "Notes").expect("the bare field rides the snapshot");
    assert!(
        !bare.secure_entry,
        "a field that declared nothing announces nothing"
    );
    let button = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Button, "Save").cloned())
        .expect("the button rides the snapshot");
    assert!(
        !button.secure_entry,
        "a non-input role inherits the absence"
    );
    harness.assert_all_named();
}

#[test]
fn m100_the_mask_the_tree_carries_is_never_the_secret() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 160.0);
    let root = harness.mount(Column::new().boxed());
    let _ = harness.add_child(root, TextInput::new("Passphrase").secure().boxed());
    let _ = harness.add_child(root, TextInput::new("Notes").boxed());
    harness.settle();

    // The AT's write: a password manager fills the field — the door
    // every editable owns. The value the tree resolves afterwards is
    // the mask, one character per code point of the committed value:
    // the shape the AT reads, never the secret itself.
    assert!(
        harness.act(
            Role::TextInput,
            "Passphrase",
            A11yAction::SetValue("hunter2".into())
        ),
        "a secure field is editable — the AT's write is consumed"
    );
    let secured = input_node(&harness, "Passphrase").expect("the secured field survives");
    assert_eq!(
        secured.value.as_deref(),
        Some("•••••••"),
        "the tree carries the mask, one bullet per character — the length, not the value"
    );
    assert_ne!(
        secured.value.as_deref(),
        Some("hunter2"),
        "the secret never rides the tree — the payload rule at the tree tier"
    );

    // The mask tracks the value: a longer secret, a longer mask.
    assert!(
        harness.act(
            Role::TextInput,
            "Passphrase",
            A11yAction::SetValue("correct horse battery staple".into())
        ),
        "the rewrite is consumed"
    );
    let grown = input_node(&harness, "Passphrase").expect("the secured field survives");
    assert_eq!(
        grown.value.as_deref(),
        Some("••••••••••••••••••••••••••••"),
        "the mask's length tracks the committed value's code points"
    );

    // The bare field's value rides raw: a plain field's value is the
    // AT's read, not a secret — the tier prices the declaration, never
    // the field's ordinary honesty.
    assert!(
        harness.act(
            Role::TextInput,
            "Notes",
            A11yAction::SetValue("visible".into())
        ),
        "the bare field's write is consumed"
    );
    let bare = input_node(&harness, "Notes").expect("the bare field survives");
    assert_eq!(
        bare.value.as_deref(),
        Some("visible"),
        "a field that declared nothing carries its value as always"
    );
}

#[test]
fn m100_the_dsl_secure_word_declares_it() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 120.0);

    // The `secure` word the text-input element has parsed since the M0
    // registry builds the secured constructor — the tree's truth rides
    // the word, at mount, with no new prop class to pay for.
    let view = alacer_macro::rsx! {
        column {
            text_input { name: "Passphrase", secure: true }
        }
    };
    let root = alacer_view::ViewRoot::mount(
        &view,
        harness.pipeline_mut(),
        alacer_widgets::default_registry(),
    );
    harness.settle();
    let secured = input_node(&harness, "Passphrase").expect("the view's field rides the snapshot");
    assert!(
        secured.secure_entry,
        "the DSL's secure word declares the field through the same door"
    );
    let _ = root;
}

#[test]
fn m100_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the secure route's case: the
    // 1,000-node secured input arena — the word's cost the measurement,
    // priced against the same input arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(SECURED_CASE) && benches.contains_key(BARE_CASE),
        "the secure route's case and its denominator are published ({SECURED_CASE}, {BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M100 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [SECURED_CASE, BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("secure_snapshot_overhead"))
        .expect("the secure budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(SECURED_CASE),
        "the budget prices the secured route"
    );
    assert_eq!(
        budget["denominator"].as_str(),
        Some(BARE_CASE),
        "the budget prices it against the same arena, bare"
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
        "the m5 corpus pin re-agrees at the M100 cut"
    );
}
