//! The canon apps' AccessKit tree-shape proof (§11, architecture §1): the
//! *composed* todo and data-table canons — mounted through the real pass
//! pipeline, not isolated widgets — reach AccessKit with the structural
//! shape the a11y contract promises. Roles, names, toggle states, and grid
//! geometry are asserted on the actual `TreeUpdate` the sink publishes,
//! headless, with no blessed snapshot: the shape is the contract.

use accesskit::Role as AkRole;
use accesskit::Toggled;
use alacer_a11y::AccessKitSink;
use alacer_testing::TestHarness;
use alacer_widgets::{data_table_app, todo_app};

/// The erased-sink downcast (the m-suites' established seam): the harness
/// carries the AccessKit sink the windowed composition installs, and the
/// test reads the `TreeUpdate`s the pipeline actually published.
fn with_sink<R>(harness: &mut TestHarness, f: impl FnOnce(&mut AccessKitSink) -> R) -> R {
    let sink = harness
        .pipeline_mut()
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<AccessKitSink>())
        .expect("the harness carries the AccessKit sink — the windowed composition");
    f(sink)
}

/// The first node in the update carrying `role` and exactly `label` —
/// the AT's read of the composed tree, not the widget tree's.
fn find_node<'a>(
    update: &'a accesskit::TreeUpdate,
    role: AkRole,
    label: &str,
) -> Option<&'a accesskit::Node> {
    update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .map(|(_, node)| node)
}

/// How many nodes in the update carry `role` — the composition's shape in
/// counts (one checkbox per row, one row per data line, and no more).
fn count_role(update: &accesskit::TreeUpdate, role: AkRole) -> usize {
    update
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == role)
        .count()
}

/// The update's tree connectivity: how many nodes are nobody's child. A
/// well-formed AccessKit forest answers exactly one — the root — every
/// other node parented exactly once (a composed app ships a tree, not a
/// pile of nodes).
fn unparented_nodes(update: &accesskit::TreeUpdate) -> usize {
    let mut parented = std::collections::HashSet::new();
    for (_, node) in &update.nodes {
        for child in node.children() {
            parented.insert(child.0);
        }
    }
    update
        .nodes
        .iter()
        .filter(|(id, _)| !parented.contains(&id.0))
        .count()
}

#[test]
fn the_todo_canon_reaches_accesskit_named_and_toggled() {
    let mut harness = TestHarness::new(480.0, 640.0);
    let mut handles = todo_app(harness.pipeline_mut(), None);
    handles.add("write docs");
    handles.add("ship it");
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();

    let update = with_sink(&mut harness, |sink| sink.full_tree_update())
        .expect("the settled todo canon publishes a full AccessKit tree");
    assert_eq!(
        unparented_nodes(&update),
        1,
        "one root — a tree, not a pile"
    );

    // The entry field: the AT's way into the app, named for what it does.
    let input = update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == AkRole::TextInput)
        .map(|(_, node)| node)
        .expect("the todo entry rides as a TextInput");
    assert_eq!(input.label(), Some("Add a todo"), "the entry is named");

    // The rows: one checkbox per item, named by the item, both off at
    // creation — the honest initial state.
    assert_eq!(
        count_role(&update, AkRole::CheckBox),
        2,
        "one checkbox per todo row"
    );
    for label in ["write docs", "ship it"] {
        let checkbox = find_node(&update, AkRole::CheckBox, label)
            .unwrap_or_else(|| panic!("the {label:?} row's checkbox rides"));
        assert_eq!(
            checkbox.toggled(),
            Some(Toggled::False),
            "{label:?} starts unchecked"
        );
    }

    // The row action: a real button, named by what it deletes.
    let button = find_node(&update, AkRole::Button, "Delete write docs")
        .expect("the row's delete button rides");
    assert_eq!(button.role(), AkRole::Button, "the delete is pressable");

    // The toggle flip rides the tree end to end: signal → re-render →
    // settle → the update the AT reads says Toggled::True.
    handles.toggle(0);
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();
    let update = with_sink(&mut harness, |sink| sink.full_tree_update())
        .expect("the re-rendered todo publishes a full AccessKit tree");
    assert_eq!(
        find_node(&update, AkRole::CheckBox, "write docs")
            .expect("the toggled row survives")
            .toggled(),
        Some(Toggled::True),
        "the checkbox's AccessKit state follows the signal"
    );
    assert_eq!(
        find_node(&update, AkRole::CheckBox, "ship it")
            .expect("the untouched row survives")
            .toggled(),
        Some(Toggled::False),
        "the sibling keeps its honest state"
    );
}

#[test]
fn the_data_table_canon_reaches_accesskit_as_a_navigable_grid() {
    let mut harness = TestHarness::new(640.0, 480.0);
    let mut handles = data_table_app(
        harness.pipeline_mut(),
        vec![
            vec!["Nails".into(), "12".into(), "A1".into()],
            vec!["Screws".into(), "5".into(), "B2".into()],
        ],
        None,
    );
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();

    let update = with_sink(&mut harness, |sink| sink.full_tree_update())
        .expect("the settled data table publishes a full AccessKit tree");
    assert_eq!(
        unparented_nodes(&update),
        1,
        "one root — a tree, not a pile"
    );

    // The table itself: named, and its declared geometry published as the
    // grid counts the AT navigates by.
    let table = update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == AkRole::Table)
        .map(|(_, node)| node)
        .expect("the composed table rides as a Table");
    assert_eq!(table.label(), Some("Inventory"), "the table is named");
    assert_eq!(table.row_count(), Some(2), "grid rows published");
    assert_eq!(table.column_count(), Some(3), "grid columns published");

    // The rows: the header row plus the two data rows ride as Row nodes.
    assert_eq!(
        count_role(&update, AkRole::Row),
        3,
        "one header row + two data rows"
    );

    // The headers: named, each carrying its column's place.
    for (name, column) in [("Item", 0_usize), ("Quantity", 1), ("Location", 2)] {
        let header = find_node(&update, AkRole::ColumnHeader, name)
            .unwrap_or_else(|| panic!("the {name:?} header rides"));
        assert_eq!(
            header.column_index(),
            Some(column),
            "{name:?} names column {column}"
        );
    }

    // The cells: placed, not just present — row 0 column 0 is where the
    // AT lands when it walks the grid.
    let cell = find_node(&update, AkRole::Cell, "Nails").expect("the Nails cell rides");
    assert_eq!(cell.row_index(), Some(0), "the cell's row place");
    assert_eq!(cell.column_index(), Some(0), "the cell's column place");

    // A pushed row renumbers the grid through the update: the geometry
    // the AT reads follows the signal, row count 3, the new cell at row 2.
    handles.push_row(vec!["Bolts".into(), "3".into(), "C3".into()]);
    let _ = handles.rerender(harness.pipeline_mut());
    harness.settle();
    let update = with_sink(&mut harness, |sink| sink.full_tree_update())
        .expect("the grown table publishes a full AccessKit tree");
    let table = update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == AkRole::Table)
        .map(|(_, node)| node)
        .expect("the table survives the growth");
    assert_eq!(table.row_count(), Some(3), "the grid grew with the signal");
    let bolts = find_node(&update, AkRole::Cell, "Bolts").expect("the new cell rides");
    assert_eq!(bolts.row_index(), Some(2), "the appended row's place");
    assert_eq!(bolts.column_index(), Some(0), "the appended cell's column");
}
