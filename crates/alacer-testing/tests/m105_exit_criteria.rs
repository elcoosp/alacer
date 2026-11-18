//! M105 exit-criteria integration tests (§10.3, ADR-0109): the
//! read-only tier — the declared word behind the wall every editing
//! door already priced. Since M22 (ADR-0028) a field can be *held down*
//! (`enabled(false)`): the availability tier, the control that refuses
//! every door including focus. What no word said was the *field whose
//! value the app owns* — the field that reads, selects, copies and
//! walks its caret like any other, and refuses only the writes. The
//! flag grows the read-only word beside the availability's —
//! [`alacer_core::Widget::read_only`] beside
//! [`alacer_widgets::TextInput::read_only`] — AccessKit's `ReadOnly`
//! flag (ARIA's `aria-readonly`) the sink maps in the same snapshot
//! every other property rides, and the walls it names are the typing
//! door's and the AT write door's own refusal vocabulary: the ask the
//! open door would have answered streams
//! [`alacer_core::KeyEditRefusal::ReadOnly`] and
//! [`alacer_core::TextWriteRefusal::ReadOnly`] behind their records,
//! the shape priced at nothing, never a munged delivered edit.
//!
//! The tier's boundary is the doors themselves: a read-only field is
//! still an interactive control — focus holds, the caret walks, the
//! selection answers, the copy chord reads — so only the value writes
//! (typing, deletes, paste, cut, undo, redo, the IME's commit, the AT's
//! `SetValue`/`ReplaceSelectedText`) meet the wall, and the
//! availability tier stays the outer one: a field both disabled and
//! read-only names the availability wall (`Unavailable`), never the
//! read-only one. The app's own door (`set_value`) never asked the
//! field's permission and never will — the tier walls *user* doors,
//! not the app's.

use alacer_core::events::{ImeEvent, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::inspect::{InspectEvent, InspectObserver};
use alacer_core::passes::TreeNode;
use alacer_core::widget::{A11yAction, Role, TextSelection};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M105 cut —
/// the read-only route's case joining the reveal route's case (M102),
/// the scroll position route's case (M101), the secure route's case
/// (M100), the numeric route's case (M99), the orientation route's case
/// (M98), the range route's case (M97), the placeholder route's pair
/// (M96), the description route's case (M95), the toggle route's pair
/// (M94), the derive route's pair (M92), and the recording route's pair
/// (M93) — and every case M103 and M104 published — on the published
/// corpus.
const EXPECTED_BENCH_COUNT: usize = 134;

/// The suite's own read door into the published corpus: the read-only
/// route's case — the 1,000-node read-only field arena (the coordinated
/// rule's bench half, ADR-0109's exit criteria). The ratio's honest
/// denominator is the same 1,000-node input arena bare
/// (`placeholder/snapshot_1k_bare`, the M96 case), the M95 pricing
/// rule: the same corpus measuring what the tier adds.
const READ_ONLY_CASE: &str = "read_only/snapshot_1k_read_only";
const BARE_CASE: &str = "placeholder/snapshot_1k_bare";

/// The first text input in the snapshot whose name contains `name`.
fn input_node(harness: &TestHarness, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::TextInput, name).cloned())
}

/// The app's read-only door: the mutate seam the app's code rides —
/// the flag moved, `PAINT | A11Y` requested on the field, the frames
/// the re-announcement rides settled before the read.
fn set_read_only(harness: &mut TestHarness, node: alacer_core::NodeId, read_only: bool) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_read_only(read_only);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

/// The app's value door: the app's own write never asked the field's
/// permission (the tier walls user doors, not the app's) — the value
/// lands through the same mutate seam, `PAINT | A11Y` requested.
fn set_value(harness: &mut TestHarness, node: alacer_core::NodeId, value: &str) {
    let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
    if let Some(input) = guard.widget_mut().as_any_mut().downcast_mut::<TextInput>() {
        input.set_value(value);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    drop(guard);
    harness.settle();
}

/// The trail collector the tests install as the pipeline's observer:
/// every event streams into a bounded ring (the m-suite's helper shape
/// — the wall's facts are the assertion source).
struct TrailObserver {
    events: std::collections::VecDeque<InspectEvent>,
}

impl TrailObserver {
    fn new() -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(64),
        }
    }

    /// The streamed key-edit facts, oldest first.
    fn key_edits(&self) -> Vec<alacer_core::KeyEditRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::KeyEdit(record) => Some(*record),
                _ => None,
            })
            .collect()
    }

    /// The streamed AT text-write facts, oldest first.
    fn text_writes(&self) -> Vec<alacer_core::TextWriteRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                InspectEvent::TextWrite(record) => Some(*record),
                _ => None,
            })
            .collect()
    }
}

impl InspectObserver for TrailObserver {
    fn observe(&mut self, event: InspectEvent) {
        if self.events.len() >= 64 {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }
}

/// The shared-ring wrapper the pipeline owns: the observer sink is
/// `&mut`-driven, so the tests share the ring through `Rc<RefCell>` —
/// the m-suite's shape, exactly.
struct Share(std::rc::Rc<std::cell::RefCell<TrailObserver>>);

impl Share {
    fn new() -> Self {
        Self(std::rc::Rc::new(std::cell::RefCell::new(
            TrailObserver::new(),
        )))
    }
}

impl InspectObserver for Share {
    fn observe(&mut self, event: InspectEvent) {
        self.0.borrow_mut().observe(event);
    }
}

/// The streamed key-edit facts, oldest first.
fn key_edits(observer: &Share) -> Vec<alacer_core::KeyEditRecord> {
    observer.0.borrow().key_edits()
}

/// The streamed AT text-write facts, oldest first.
fn text_writes(observer: &Share) -> Vec<alacer_core::TextWriteRecord> {
    observer.0.borrow().text_writes()
}

/// Empties the shared ring between beats: each assertion reads the
/// facts the wall under test streamed, nothing earlier.
fn clear_trail(observer: &Share) {
    observer.0.borrow_mut().events.clear();
}

/// A Ctrl press (the clipboard chords' command set; ADR-0020) — the
/// exact event the platform delivers when the user pastes, cuts or
/// copies.
fn ctrl(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        text: None,
    }
}

/// A bare press (typing, deletes) — the exact event the platform
/// delivers when the user types or deletes.
fn pressed(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        text: None,
    }
}

/// An input mounted through the real pipeline, focused and settled —
/// the doors' own mount. Returns the harness, the input's funnel stamp
/// id and its arena node.
fn mount_input(widget: TextInput) -> (TestHarness, u64, alacer_core::NodeId) {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, widget.boxed());
    harness.settle();
    let id = harness.query(Role::TextInput).named("field").first().id;
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    (harness, id, node)
}

#[test]
fn m105_the_read_only_word_lands_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(320.0, 240.0);
    let root = harness.mount(Column::new().gap(6.0).padding(12.0).boxed());
    // The read-only field, mounted through the constructor word: the
    // declaration the tree says from the first frame.
    let locked = TextInput::new("License").read_only();
    let locked_id = harness.add_child(root, locked.boxed());
    // The plain field beside it: no read-only word to say.
    let plain_id = harness.add_child(root, TextInput::new("Draft").boxed());
    let _ = harness.add_child(root, Button::new("Save").boxed());
    harness.settle();

    let locked = input_node(&harness, "License").expect("the read-only field rides");
    assert!(
        locked.read_only,
        "the constructor's word is the tree's truth"
    );
    let plain = input_node(&harness, "Draft").expect("the plain field rides");
    assert!(
        !plain.read_only,
        "a field that declared nothing answers no read-only word"
    );
    let button = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Button, "Save").cloned())
        .expect("the button rides the snapshot");
    assert!(
        !button.read_only,
        "a non-input role inherits the honest absence"
    );

    // The live door: the app's word moves, the re-announcement the
    // mutate seam's request paid for — and moves back.
    set_read_only(&mut harness, plain_id, true);
    let moved = input_node(&harness, "Draft").expect("the field survives");
    assert!(
        moved.read_only,
        "the app's door moves the word the tree says"
    );
    set_read_only(&mut harness, locked_id, false);
    let unlocked = input_node(&harness, "License").expect("the field survives");
    assert!(
        !unlocked.read_only,
        "the same door carries the word both ways"
    );
    harness.assert_all_named();
}

#[test]
fn m105_the_typing_door_refuses_the_read_only_field() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(TextInput::new("field").read_only());
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    // The app's own door put the value there; the tier walls user doors,
    // not the app's.
    set_value(&mut harness, node, "hello");
    clear_trail(&observer);

    // Typing meets the wall: the ask streams behind its species, the
    // shape priced at nothing.
    harness.type_text("x");
    harness.key(&pressed(Key::Backspace));
    harness.set_clipboard("seed");
    harness.key(&ctrl(Key::Char('v')));
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 3, "each refused ask streams one fact");
    assert!(
        edits.iter().all(|record| record.outcome
            == alacer_core::KeyEditOutcome::Refused(alacer_core::KeyEditRefusal::ReadOnly)),
        "every refused edit names the read-only wall"
    );
    assert_eq!(edits[0].action, alacer_core::KeyEditAction::Insert);
    assert_eq!(edits[0].driver, alacer_core::KeyEditDriver::Keyboard);
    assert_eq!(edits[1].action, alacer_core::KeyEditAction::Backspace);
    assert_eq!(
        edits[2].action,
        alacer_core::KeyEditAction::Paste,
        "the paste chord's ask names itself behind the wall"
    );
    assert_eq!(edits[2].driver, alacer_core::KeyEditDriver::Clipboard);

    // The value the tree carries is still the app's — the wall moved
    // nothing.
    let field = input_node(&harness, "field").expect("the field rides");
    assert_eq!(
        field.value.as_deref(),
        Some("hello"),
        "the wall moved nothing"
    );

    // The read doors stay open: the caret walks, the selection answers,
    // and the copy chord reads. A delivered copy asks nothing (the M68
    // rule), so the trail prices only the refusals above.
    harness.key(&ctrl(Key::Char('a')));
    harness.key(&ctrl(Key::Char('c')));
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("hello"),
        "the copy chord reads through the read-only word"
    );
    assert_eq!(
        key_edits(&observer).len(),
        3,
        "selection and copy are no walls — no facts queued beside the refusals"
    );
}

#[test]
fn m105_the_at_write_door_refuses_the_read_only_field() {
    let observer = Share::new();
    let (mut harness, _id, node) = mount_input(TextInput::new("field").read_only());
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    set_value(&mut harness, node, "PIN");
    clear_trail(&observer);

    // The AT's value write meets the wall: refused honestly, never
    // munged, the ask streamed behind the write door's own species.
    assert!(
        !harness.act(
            Role::TextInput,
            "field",
            A11yAction::SetValue("9999".into())
        ),
        "the AT's write door is the wall"
    );
    let writes = text_writes(&observer);
    assert_eq!(writes.len(), 1, "one ask, one fact");
    assert!(
        writes[0].outcome
            == alacer_core::TextWriteOutcome::Refused(alacer_core::TextWriteRefusal::ReadOnly),
        "the write wall names the read-only species"
    );

    // The selection door stays open — a read-only field reads.
    assert!(
        harness.act(
            Role::TextInput,
            "field",
            A11yAction::SetTextSelection(TextSelection::new(0, 2))
        ),
        "the AT's selection door answers"
    );
    let field = input_node(&harness, "field").expect("the field rides");
    assert_eq!(
        field.selection.map(|s| (s.anchor, s.focus)),
        Some((0, 2)),
        "the selection landed — the read door's own answer"
    );
    assert_eq!(
        field.value.as_deref(),
        Some("PIN"),
        "the value moved nothing"
    );

    // The IME's commit meets the same wall (the typing door's driver).
    clear_trail(&observer);
    harness.ime(&ImeEvent::Commit {
        text: "x".to_string(),
    });
    let edits = key_edits(&observer);
    assert_eq!(edits.len(), 1, "the commit the wall turned away streams");
    assert_eq!(edits[0].driver, alacer_core::KeyEditDriver::Ime);
    assert!(
        edits[0].outcome
            == alacer_core::KeyEditOutcome::Refused(alacer_core::KeyEditRefusal::ReadOnly),
        "the IME's ask names the read-only wall"
    );
    let field = input_node(&harness, "field").expect("the field rides");
    assert_eq!(
        field.value.as_deref(),
        Some("PIN"),
        "the commit moved nothing"
    );
}

#[test]
fn m105_the_availability_tier_stays_the_outer_wall() {
    let observer = Share::new();
    // A field both disabled and read-only: the availability tier is the
    // outer one — the field is never a focus candidate (M22's own
    // derivation), so the key route's boundary answers before any
    // field door does (the M59/M62 boundary shape: the route's
    // fallback, never a munged typing-door fact), and the read-only
    // word rides the tree beside the availability's.
    let (mut harness, _id, node) = mount_input(TextInput::new("field").enabled(false).read_only());
    harness
        .pipeline_mut()
        .set_observer(Box::new(Share(std::rc::Rc::clone(&observer.0))));
    set_value(&mut harness, node, "kept");
    clear_trail(&observer);

    harness.type_text("x");
    harness.settle();
    assert!(
        key_edits(&observer).is_empty(),
        "the unavailable field holds no focus — the boundary answers, no field wall streams"
    );
    let field = input_node(&harness, "field").expect("the field rides");
    assert!(
        field.read_only,
        "the read-only word rides the tree beside the availability's"
    );
    assert!(
        !field.enabled,
        "the availability word rides — the outer tier's own truth"
    );
    assert_eq!(
        field.value.as_deref(),
        Some("kept"),
        "the wall moved nothing"
    );
}

#[test]
fn m105_the_corpus_grew_by_the_coordinated_rule() {
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

    // The published corpus carries the read-only route's case: the
    // 1,000-node read-only field arena — the word's cost the
    // measurement, priced against the same arena bare.
    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert!(
        benches.contains_key(READ_ONLY_CASE) && benches.contains_key(BARE_CASE),
        "the read-only route's case and its denominator are published ({READ_ONLY_CASE}, {BARE_CASE})"
    );
    assert_eq!(
        baselines["bench_count"].as_u64(),
        Some(u64::try_from(EXPECTED_BENCH_COUNT).expect("fits")),
        "the published corpus pin re-agrees at the M105 cut"
    );

    // The contract set gates both cases and prices the word's budget —
    // a ratio, the host-independent contract class.
    let gates = contracts["regressions"]
        .as_array()
        .expect("the contract set carries regressions");
    for case in [READ_ONLY_CASE, BARE_CASE] {
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
        .find(|budget| budget["id"].as_str() == Some("read_only_snapshot_overhead"))
        .expect("the read-only budget is published");
    assert_eq!(
        budget["numerator"].as_str(),
        Some(READ_ONLY_CASE),
        "the budget prices the read-only route"
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
        "the m5 corpus pin re-agrees at the M105 cut"
    );
}
