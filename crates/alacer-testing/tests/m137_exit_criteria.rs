//! M137 exit-criteria integration tests (§11.1, ADR-0141): the menu
//! tier — the named strip of menus, the summoned list, and the command
//! entries, said by the tree. Through M136 the snapshot said the tooltip
//! tier (ADR-0140), the tree tier (ADR-0139), the dialog (ADR-0138), the
//! meter (ADR-0137), the tool strip (ADR-0133) — and the command
//! vocabulary had two shapes, the tool strip (M129: a horizontal run of
//! tools always on) and the has-popup word (M115, ADR-0119: the opener's
//! "this opens a menu") — while the menu itself had no word of its own:
//! an app wiring a menu bar reached for a [`Toolbar`](alacer_widgets::Toolbar)
//! wearing the wrong word (a toolbar *offers tools*, a menu bar *names
//! the menus the commands live in*), for a dialog wearing the wrong
//! transient (a dialog is a modal window the walk bounds itself into; a
//! menu is a transient list the opener summons), or for a bare button
//! wearing no entry word at all (the command the AT cannot name is a
//! command it cannot fire). [`Role::MenuBar`](alacer_core::widget::Role::MenuBar),
//! [`Role::Menu`](alacer_core::widget::Role::Menu) and
//! [`Role::MenuItem`](alacer_core::widget::Role::MenuItem) carry the
//! words AccessKit's own `MenuBar`, `Menu` and `MenuItem` roles spell.
//!
//! The tier prices the words and nothing else: the strip and the list
//! are the entries' layout, never a second channel to them (the children
//! ride the arena exactly a toolbar's and a pane's do, each keeping the
//! role and label it already carries), presence is the app's wiring (the
//! view mounts and unmounts the summoned list, the reconciler's own
//! shape — the tooltip tier's rule at the menu tier), and the entry is
//! the tier's only control: the activation funnel the button tier
//! published (the click on release, Enter while focused — the M10
//! parity, and the AT's default action the door answers), the
//! availability tier guarding it (M22: an unavailable entry refuses,
//! never munges). The strip and the list are not controls: they hold no
//! focus and answer no activation (the M126/M129 container rule, at the
//! menu tier — the entries hold the focus).
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: the a11y tree is the only official query
//! API (ADR-10), and the sink's role branches grow three beside the
//! tooltip's.

use alacer_core::events::{Key, KeyEvent, KeyState, Modifiers};
use alacer_core::passes::TreeNode;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Label, Menu, MenuBar, MenuItem, TextInput};

/// The suite's corpus pin: the coordinated rule's count at the M138
/// cut — unchanged from the M136 cut. The menu tier grows no snapshot
/// *route* (the words are the containers' and the entry's own, the
/// shape the M124/M127/M129/M131/M134/M135/M136 named containers rode),
/// and its pricing door is the widget-level bench the tier carries in
/// the widgets crate (`menu_bar/name_write_256` against
/// `menu_bar/idle_write_256`), which rides *outside* the published
/// corpus: the published baseline's routes price the a11y snapshot
/// machinery (the M95 rule), and the coordinated rule grows it at the
/// next published cut, never at a widget bench's. The pin re-agrees at
/// 155.
const EXPECTED_BENCH_COUNT: usize = 165;

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> Option<TreeNode> {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
}

/// The released half of an activation key (the pressed half arms, the
/// released half fires — the M10 parity the button tier published).
fn released(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Released,
        modifiers: Modifiers::default(),
        text: None,
    }
}

#[test]
fn m137_the_menu_words_land_in_the_a11y_snapshot() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 240.0);
    // The three words, mounted through the constructor words: the named
    // strip, the named list riding under it (the arena's own nesting,
    // never a second structure), the named entries riding the list —
    // each child keeping the word it already carries.
    let root = harness.mount(Column::new().gap(6.0).boxed());
    let strip_id = harness.add_child(root, MenuBar::new("Main").boxed());
    let list_id = harness.add_child(strip_id, Menu::new("File").boxed());
    let _new_id = harness.add_child(list_id, MenuItem::new("New").boxed());
    let _open_id = harness.add_child(list_id, MenuItem::new("Open").boxed());
    let _ = harness.add_child(root, TextInput::new("Subject").boxed());
    harness.settle();

    // The strip carries the menu-bar word; the list the menu word; the
    // entries the menu-item word — the vocabulary AccessKit's own roles
    // spell.
    let strip = node(&harness, Role::MenuBar, "Main").expect("the named strip rides the snapshot");
    assert_eq!(
        strip.role,
        Role::MenuBar,
        "the word AccessKit's own role spells"
    );
    let list = node(&harness, Role::Menu, "File").expect("the named list rides the strip");
    assert_eq!(list.role, Role::Menu, "the summoned list's own word");
    let entry = node(&harness, Role::MenuItem, "New").expect("the named entry rides the list");
    assert_eq!(entry.role, Role::MenuItem, "the command's own word");
    // One word per node: each word is the only word of its kind in the
    // tree — the neighbours keep the words they published (the help text
    // reads, the field edits).
    assert_eq!(
        harness.query(Role::MenuBar).count(),
        1,
        "no neighbour munged onto the strip word"
    );
    assert_eq!(
        harness.query(Role::Menu).count(),
        1,
        "no neighbour munged onto the list word"
    );
    assert_eq!(
        harness.query(Role::MenuItem).count(),
        2,
        "the entries' words theirs alone"
    );
    let hint = node(&harness, Role::TextInput, "Subject").expect("the field beside keeps its word");
    assert_eq!(
        hint.role,
        Role::TextInput,
        "the neighbour is the neighbour's own — never a menu wearing the field"
    );
    harness.assert_all_named();
}

#[test]
fn m137_the_menu_item_is_a_control_and_the_containers_are_not() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    let strip_id = harness.add_child(root, MenuBar::new("Main").boxed());
    let list_id = harness.add_child(strip_id, Menu::new("File").boxed());
    let new_id = harness.add_child(list_id, MenuItem::new("New").boxed());
    let _open_id = harness.add_child(list_id, MenuItem::new("Open").boxed());
    harness.settle();

    // The entry is the tier's only control: the walk lands the first
    // entry, never the strip or the list (the M126/M129 container rule,
    // at the menu tier — the entries hold the focus).
    harness.key(&alacer_core::events::KeyEvent {
        key: Key::Tab,
        state: KeyState::Pressed,
        modifiers: alacer_core::events::Modifiers::default(),
        text: None,
    });
    harness.settle();
    assert_eq!(
        harness.pipeline().focus(),
        Some(new_id),
        "the walk landed the entry, never the containers"
    );

    // The keyboard's activation names itself (M70, the M10 parity at the
    // item tier): the focused entry answers Enter exactly like a
    // pointer press-and-release — the pressed half arms, the released
    // half fires the handler.
    let fired = std::rc::Rc::new(std::cell::Cell::new(0_usize));
    {
        let clicks = std::rc::Rc::clone(&fired);
        let mut guard = harness.pipeline_mut().arena_mut().mutate(new_id);
        let wired = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<MenuItem>()
            .is_some_and(|entry| {
                entry.set_on_click(Some(Box::new(move || {
                    clicks.set(clicks.get() + 1);
                })));
                true
            });
        let _ = wired;
        guard.request(alacer_core::Change::A11Y);
        drop(guard);
    }
    harness.key(&KeyEvent {
        key: Key::Enter,
        state: KeyState::Pressed,
        modifiers: alacer_core::events::Modifiers::default(),
        text: None,
    });
    harness.key(&released(Key::Enter));
    assert_eq!(
        fired.get(),
        1,
        "the release fired the entry's handler — the M10 parity"
    );

    // The AT's default action the door answers (M10, ADR-0016): the
    // reachable, named, actionable triple the harness drives the way the
    // platform drives it.
    harness.press(Role::MenuItem, "Open");
    assert_eq!(
        fired.get(),
        1,
        "the AT's press landed `Open`, never its sibling"
    );
    harness.settle();
    harness.assert_all_named();
}

#[test]
fn m137_an_unavailable_entry_refuses_never_munges() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 240.0);
    let root = harness.mount(Column::new().boxed());
    let strip_id = harness.add_child(root, MenuBar::new("Main").boxed());
    let list_id = harness.add_child(strip_id, Menu::new("File").boxed());
    let _entry_id = harness.add_child(list_id, MenuItem::new("New").boxed());
    let export_id = harness.add_child(list_id, MenuItem::new("Export").enabled(false).boxed());
    harness.settle();

    // The availability tier (M22, ADR-0028) at the item tier: an
    // unavailable entry reports itself before the AT tries the press it
    // will refuse, and the refusal is honest — the action consumed
    // nothing, the handler never fired (the M11 rule).
    let fired = std::rc::Rc::new(std::cell::Cell::new(0_usize));
    {
        let clicks = std::rc::Rc::clone(&fired);
        let mut guard = harness.pipeline_mut().arena_mut().mutate(export_id);
        let wired = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<MenuItem>()
            .is_some_and(|entry| {
                entry.set_on_click(Some(Box::new(move || {
                    clicks.set(clicks.get() + 1);
                })));
                true
            });
        let _ = wired;
        guard.request(alacer_core::Change::A11Y);
        drop(guard);
    }
    let consumed = harness.act(
        Role::MenuItem,
        "Export",
        alacer_core::widget::A11yAction::DefaultPress,
    );
    assert!(
        !consumed,
        "the unavailable entry refused the ask, never munged it"
    );
    assert_eq!(fired.get(), 0, "the refusal fired nothing");
    harness.settle();
    harness.assert_all_named();
}

#[test]
fn m137_the_dsl_menu_words_declare_it() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 160.0);

    // The `menu_bar`, `menu` and `menu_item` words the elements parse —
    // the vocabulary's truth rides the words, at mount, with no new prop
    // class to pay for (the M134 dialog word's own shape).
    let view = alacer_macro::rsx! {
        column {
            menu_bar {
                name: "Main",
                menu {
                    name: "File",
                    menu_item {
                        text: "New",
                    }
                    menu_item {
                        text: "Open",
                    }
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
    let strip = node(&harness, Role::MenuBar, "Main").expect("the view's strip rides the snapshot");
    assert_eq!(
        strip.role,
        Role::MenuBar,
        "the DSL's word declares the strip"
    );
    let list = node(&harness, Role::Menu, "File").expect("the view's list rides the strip");
    assert_eq!(list.role, Role::Menu, "the DSL's word declares the list");
    let entry = node(&harness, Role::MenuItem, "New").expect("the view's entry rides the list");
    assert_eq!(
        entry.role,
        Role::MenuItem,
        "the DSL's word declares the entry"
    );
    let _ = root;
}

#[test]
fn m137_presence_is_the_apps_wiring_never_a_flag() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 240.0);
    // The summoned list is present exactly while the app mounts it: the
    // first frame mounts the strip alone; the reconciler's own shape
    // (the M1 conditional-mount precedent, the tooltip tier's rule)
    // brings the list the next frame — the strip never guessing at a
    // second `visible` flag.
    let root = harness.mount(Column::new().boxed());
    let strip_id = harness.add_child(root, MenuBar::new("Main").boxed());
    harness.settle();
    assert!(
        node(&harness, Role::Menu, "File").is_none(),
        "the list is down — the app has not summoned it"
    );
    let list_id = harness.add_child(strip_id, Menu::new("File").boxed());
    let _ = harness.add_child(list_id, MenuItem::new("New").boxed());
    harness.settle();
    node(&harness, Role::Menu, "File").expect("the list is up — the app mounted it");
    harness.assert_all_named();
}

#[test]
fn m137_the_corpus_pin_re_agrees_at_the_m138_cut() {
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

    // The published corpus pin re-agrees: the menu tier grows no
    // snapshot route (the words are the containers' and the entry's
    // own, the shape the named containers rode), and its pricing door is
    // the widgets crate's own bench — outside the published corpus, the
    // coordinated rule growing it at the next published cut.
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
fn m137_the_living_documents_re_agree() {
    let _guard = alacer_core::flux::batch_test_lock();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves");
    let read = |path: &str| std::fs::read_to_string(root.join(path)).expect("the file is readable");

    // The README status reflects the living cut, and the register grew
    // the menu tier's record.
    let readme = read("README.md");
    assert!(
        readme.contains("M0 through M155"),
        "the README status reflects the living cut — the chain cannot drift"
    );
    assert!(
        readme.contains("a hundred and sixty decision records"),
        "the register grew the M137 and M138 rows"
    );
    assert!(
        read("docs/architecture.md").contains("a hundred and sixty decision records"),
        "the architecture carries the menu fact's decision"
    );

    // The examples catalog grew the menu row, and the changelog names
    // the tier.
    assert!(
        read("docs/examples.md").contains("examples/menu_bar.rs"),
        "the catalog carries the menu demonstration"
    );
    assert!(
        read("CHANGELOG.md").contains("M137 — the menu tier"),
        "the changelog names the tier"
    );

    // The m81 suite's documents pin moved to M138's cut (the latest
    // suite carrying the living-documents pin — the M132+ cut's suites
    // pin the corpus, and their pins re-agreed with it).
    let m81 = read("crates/alacer-testing/tests/m81_exit_criteria.rs");
    assert!(
        m81.contains("M0 through M155"),
        "the m81 suite's pins moved with the documents"
    );
}

#[test]
fn m137_the_entry_walls_nothing_and_the_label_reads() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(420.0, 240.0);
    // The words price the words and nothing else: a help label beside
    // the strip reads as itself (the tier walls nothing), and the
    // entries keep the words they already carried.
    let root = harness.mount(Column::new().boxed());
    let strip_id = harness.add_child(root, MenuBar::new("Main").boxed());
    let list_id = harness.add_child(strip_id, Menu::new("File").boxed());
    let _ = harness.add_child(list_id, MenuItem::new("New").boxed());
    let _ = harness.add_child(root, Label::new("The commands live in File").boxed());
    harness.settle();
    let hint = node(&harness, Role::Label, "The commands live in File")
        .expect("the help label keeps its own word");
    assert_eq!(
        hint.role,
        Role::Label,
        "the label is the label's own — never a menu word"
    );
    harness.assert_all_named();
}
