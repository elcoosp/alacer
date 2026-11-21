//! Headless AT simulation (the launch proof): the a11y contract exercised
//! the way a real screen reader session exercises it — a complex,
//! multi-widget form driven end to end through the real ten-pass pipeline
//! (A4). Focus walks in on Tab, typing reaches the value the tree says,
//! Enter activates the focused button, the data grid moves its declared
//! pick, and every step lands on a wire the AT reads: the AccessKit
//! `TreeUpdate`s the windowed sink drains (§11.1, ADR-0015) and the causal
//! trail the funnel streams (the focus landings and the keyboard
//! activation, answer-as-data). No widget in isolation: the whole surface,
//! headless — the "accessible by construction" claim priced on a real app
//! shape, not on an isolated control.

use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::color::Color;
use alacer_core::events::{EventCtx, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::geom::{Point, Rect};
use alacer_core::inspect::InspectEvent as Event;
use alacer_core::passes::FocusMoveOutcome;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::Style;
use alacer_core::trail::TrailLog;
use alacer_core::widget::{EventResponse, LabelSource, Role, TableSlot, Widget};
use alacer_core::{Change, NodeId};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Button, Column, TableCell, TableRow, TextInput};
use std::cell::Cell;
use std::rc::Rc;

/// The dashboard's inventory grid: the data table as one tab stop. The
/// widget owns the keyboard route the way a real dashboard's grid view
/// does — the focused widget's answer preempts the window's arrow walk —
/// and the pick is the collection's *declared* word: the grid moves its
/// own model, the app shell reconciles `TableRow::set_selected` through
/// the mutate seam, the tree says the word, the AT reads it. The widget
/// never touches the tree; the a11y contract is by construction.
struct InventoryGrid {
    name: &'static str,
    rows: usize,
    columns: usize,
    selected: usize,
    /// The handle the app shell reads to reconcile the pick — the model
    /// the grid's keyboard route moves.
    selection: Rc<Cell<usize>>,
}

impl Widget for InventoryGrid {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Table
    }
    fn label(&self) -> LabelSource {
        LabelSource::text(self.name)
    }
    fn style(&self) -> Style {
        Style {
            gap: 4.0,
            ..Style::column()
        }
    }
    /// The data grid is one tab stop: the walk lands here once, the grid's
    /// own arrows do the rest — the roving-focus shape real grids speak.
    fn focusable(&self) -> bool {
        true
    }
    fn table_slot(&self) -> Option<TableSlot> {
        Some(TableSlot::Table {
            rows: self.rows,
            columns: self.columns,
        })
    }
    fn on_key_event(&mut self, ctx: &mut EventCtx, event: &KeyEvent) -> EventResponse {
        // Only the press half acts; the release falls through honestly.
        if event.state != KeyState::Pressed {
            return EventResponse::Ignored;
        }
        let moved = match event.key {
            Key::ArrowDown if self.selected + 1 < self.rows => {
                self.selected += 1;
                true
            }
            Key::ArrowUp if self.selected > 0 => {
                self.selected -= 1;
                true
            }
            _ => false,
        };
        if moved {
            self.selection.set(self.selected);
            ctx.request(Change::PAINT);
            EventResponse::Handled
        } else {
            // The wall: the grid refuses, the window's walk may take it.
            EventResponse::Ignored
        }
    }
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        ctx.push_quad(Quad::filled(
            Rect::new(Point::zero(), ctx.size()),
            Color::rgb(244, 247, 252),
        ));
    }
}

/// The exact mechanism the platform loop calls at frame boundaries: reach
/// the pipeline's erased a11y sink, downcast, delegate (the m9 shape —
/// no test double, the sink itself runs here).
fn with_sink<R>(harness: &mut TestHarness, f: impl FnOnce(&mut AccessKitSink) -> R) -> R {
    let sink = harness
        .pipeline_mut()
        .a11y_sink_mut()
        .as_any_mut()
        .and_then(|any| any.downcast_mut::<AccessKitSink>())
        .expect("the harness carries the AccessKit sink — the windowed composition");
    f(sink)
}

fn drain_updates(harness: &mut TestHarness) -> Vec<accesskit::TreeUpdate> {
    with_sink(harness, AccessKitSink::drain_updates)
}

/// The node id the tree resolved for `role` named `name` — the id the
/// drained updates address.
fn widget_id(harness: &TestHarness, role: Role, name: &str) -> u64 {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name))
        .map_or_else(
            || panic!("no {role:?} named {name:?} in the resolved tree"),
            |node| node.id,
        )
}

/// The id of the `index`-th row of the inventory grid in pre-order — rows
/// are unnamed members (the cells carry the names), the order is the
/// mount order the walk preserves.
fn row_id(harness: &TestHarness, index: usize) -> u64 {
    let tree = harness
        .pipeline()
        .a11y_snapshot()
        .expect("the tree resolved");
    let rows: Vec<u64> = tree
        .nodes
        .iter()
        .filter(|node| node.role == Role::TableRow)
        .map(|node| node.id)
        .collect();
    *rows
        .get(index)
        .unwrap_or_else(|| panic!("row {index} missing from the resolved tree"))
}

/// The app's pick door: the mutate seam the app's code rides — the word
/// moved, `PAINT | A11Y` requested on the row, the re-announcement the
/// frames settle (the m108 shape; the caller requests only on a move).
fn reconcile_pick(harness: &mut TestHarness, rows: &[NodeId], picked: usize) {
    for (index, row) in rows.iter().enumerate() {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(*row);
        let word = if index == picked {
            Some(true)
        } else if index == 0 {
            // The previously picked row stands explicitly unpicked beside
            // the new pick; rows that never declared stay silent.
            Some(false)
        } else {
            None
        };
        let moved = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<TableRow>()
            .is_some_and(|row| row.set_selected(word));
        if moved {
            guard.request(Change::PAINT.union(Change::A11Y));
        }
    }
    harness.settle();
}

/// The workflow's event log: one entry per wire fact, in session order —
/// the exact sequence the screen reader session produced.
fn workflow_events() -> Vec<String> {
    vec![
        "focus → TextInput 'Name'".to_owned(),
        "value → TextInput 'Name' = hello".to_owned(),
        "focus → TextInput 'Email'".to_owned(),
        "focus → Button 'Submit'".to_owned(),
        "press → Button 'Submit' (keyboard rider)".to_owned(),
        "focus → Table 'Inventory'".to_owned(),
        "select → Table 'Inventory' row 2".to_owned(),
    ]
}

/// The Tab step's landing, read the way the session produces it: the
/// funnel's own answer-as-data on the causal trail (the keyboard rider's
/// `FocusRecord`), matched against the form's widgets — the label the
/// event log names.
fn keyboard_landing(events: Vec<Event>, landings: &[(NodeId, String)]) -> String {
    let outcome = events.into_iter().find_map(|event| match event {
        Event::Focus(record) if record.rider == alacer_core::FocusRider::Keyboard => {
            Some(record.outcome)
        }
        _ => None,
    });
    let to = match outcome {
        Some(FocusMoveOutcome::Moved(focus_landing)) => focus_landing.to,
        other => panic!("Tab walked nowhere: {other:?}"),
    };
    landings.iter().find(|(node, _)| *node == to).map_or_else(
        || panic!("the walk landed outside the form: {to:?}"),
        |(_, label)| label.clone(),
    )
}

/// The mounted dashboard: the nodes the session addresses, the handles the
/// app shell reads, and the landing table the focus events name.
struct Dashboard {
    name_node: NodeId,
    email_node: NodeId,
    submit_node: NodeId,
    grid_node: NodeId,
    row_nodes: Vec<NodeId>,
    clicks: Rc<Cell<u32>>,
    selection: Rc<Cell<usize>>,
    landings: Vec<(NodeId, String)>,
}

/// Mounts the simplified operations dashboard: two fields, the submit
/// control, the inventory grid — the form the workflow drives.
fn mount_dashboard(harness: &mut TestHarness) -> Dashboard {
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let name_node = harness.add_child(root, TextInput::new("Name").boxed());
    let email_node = harness.add_child(root, TextInput::new("Email").boxed());
    let clicks = Rc::new(Cell::new(0_u32));
    let counter = Rc::clone(&clicks);
    let submit_node = harness.add_child(
        root,
        Button::new("Submit")
            .on_click(move || counter.set(counter.get() + 1))
            .boxed(),
    );
    let selection = Rc::new(Cell::new(0_usize));
    let grid_node = harness.add_child(
        root,
        InventoryGrid {
            name: "Inventory",
            rows: 3,
            columns: 1,
            selected: 0,
            selection: Rc::clone(&selection),
        }
        .boxed(),
    );
    let mut row_nodes = Vec::with_capacity(3);
    for index in 0..3 {
        let row = harness.add_child(grid_node, TableRow::new().selected(index == 0).boxed());
        let _ = harness.add_child(
            row,
            TableCell::new(format!("SKU-00{index}"), index, 0).boxed(),
        );
        row_nodes.push(row);
    }
    let landings = vec![
        (name_node, "TextInput 'Name'".to_owned()),
        (email_node, "TextInput 'Email'".to_owned()),
        (submit_node, "Button 'Submit'".to_owned()),
        (grid_node, "Table 'Inventory'".to_owned()),
    ];
    Dashboard {
        name_node,
        email_node,
        submit_node,
        grid_node,
        row_nodes,
        clicks,
        selection,
        landings,
    }
}

/// The typing step: the value the field holds is the value the tree says —
/// the edit re-emits through the sink carrying the value AND the focus the
/// AT voices it under. Answers the event log's label.
fn type_and_assert(harness: &mut TestHarness, field: u64) -> String {
    harness.type_text("hello");
    let drained = drain_updates(harness);
    let update = drained
        .iter()
        .rev()
        .find(|update| find_in_update(update, field).is_some())
        .expect("the edited field re-emits");
    assert_eq!(
        update.focus,
        accesskit::NodeId(field),
        "the flowing update names the field the AT voices"
    );
    let node = find_in_update(update, field).expect("the edited field re-emits");
    assert_eq!(
        node.value(),
        Some("hello"),
        "'Name' carries the typed value to the AT"
    );
    "value → TextInput 'Name' = hello".to_owned()
}

/// The activation step: the funnel routes both halves of Enter to the
/// focused control; the click fires on the release (GTK semantics), the
/// app's handler runs, and the causal trail names the ask with its
/// keyboard rider — the activation a screen reader session produced.
fn activate_on_enter(harness: &mut TestHarness, trail: &TrailLog, clicks: &Cell<u32>) -> String {
    let response = harness.press_key(Key::Enter);
    assert!(
        matches!(response, EventResponse::Handled),
        "the focused button takes the Enter press"
    );
    let release = KeyEvent {
        key: Key::Enter,
        state: KeyState::Released,
        modifiers: Modifiers::default(),
        text: None,
    };
    let response = harness.key(&release);
    assert!(
        matches!(response, EventResponse::Handled),
        "the focused button takes the Enter release"
    );
    assert_eq!(clicks.get(), 1, "the keyboard activation fired 'Submit'");
    let activation = trail.drain().into_iter().find_map(|event| match event {
        Event::Action(record) if record.rider == alacer_core::ActionRider::Keyboard => Some(record),
        _ => None,
    });
    let record = activation.expect("the keyboard activation named itself on the trail");
    assert_eq!(
        record.ask,
        alacer_core::ActionAsk::Press,
        "the activation is the press ask"
    );
    assert_eq!(
        record.outcome,
        alacer_core::ActionOutcome::Delivered,
        "the ask landed — the handler ran"
    );
    "press → Button 'Submit' (keyboard rider)".to_owned()
}

/// The grid step: the focused grid's answer takes precedence — the pick
/// moves in the grid's own model, the window's walk does not. The app
/// shell reconciles the declared pick through the mutate seam — the door
/// the real dashboard's wiring rides — and the AT hears exactly the two
/// rows whose word moved, under the focus the session holds.
fn arrow_moves_pick(harness: &mut TestHarness, dashboard: &Dashboard, table: u64) -> String {
    let response = harness.press_key(Key::ArrowDown);
    assert!(
        matches!(response, EventResponse::Handled),
        "the focused grid takes the arrow"
    );
    assert_eq!(
        harness.pipeline().focus(),
        Some(dashboard.grid_node),
        "the arrow moved the pick, never the focus"
    );
    assert_eq!(
        dashboard.selection.get(),
        1,
        "the grid moved to the second row"
    );
    reconcile_pick(harness, &dashboard.row_nodes, dashboard.selection.get());
    let drained = drain_updates(harness);
    let picked = row_id(harness, 1);
    let dropped = row_id(harness, 0);
    let update = drained
        .iter()
        .rev()
        .find(|update| find_in_update(update, picked).is_some())
        .expect("the picked row re-emits");
    assert_eq!(
        update.focus,
        accesskit::NodeId(table),
        "the selection update names the table the session is in"
    );
    let node = find_in_update(update, picked).expect("the picked row re-emits");
    assert_eq!(
        node.is_selected(),
        Some(true),
        "'Inventory' row 2 selected — the AT hears the pick"
    );
    let node = drained
        .iter()
        .rev()
        .find_map(|update| find_in_update(update, dropped))
        .expect("the dropped row re-emits");
    assert_eq!(
        node.is_selected(),
        Some(false),
        "row 1 stands explicitly unpicked beside the new pick"
    );
    "select → Table 'Inventory' row 2".to_owned()
}

/// The resolved tree agrees with what the window heard — one truth, two
/// readers (the §11.1 contract at the whole-surface scale).
fn assert_tree_agrees(harness: &TestHarness) {
    let tree = harness
        .pipeline()
        .a11y_snapshot()
        .expect("the tree resolved");
    let rows: Vec<_> = tree
        .nodes
        .iter()
        .filter(|node| node.role == Role::TableRow)
        .cloned()
        .collect();
    assert_eq!(
        rows[0].selected,
        Some(false),
        "the dropped pick is the tree's truth"
    );
    assert_eq!(
        rows[1].selected,
        Some(true),
        "the new pick is the tree's truth"
    );
    assert_eq!(rows[2].selected, None, "a row of no pick announces nothing");
}

#[test]
fn the_screen_reader_workflow_rides_the_real_pipeline_end_to_end() {
    let _guard = alacer_core::flux::batch_test_lock();
    let mut harness = TestHarness::new(480.0, 360.0);
    let trail = harness.trail();
    let dashboard = mount_dashboard(&mut harness);
    harness.settle();
    harness.assert_all_named();

    // The AT connects: the init tree covers the whole forest — one code
    // path materializes what the adapter voices (§11.1).
    let init = drain_updates(&mut harness);
    assert!(
        init.last().is_some_and(|update| update.tree.is_some()),
        "the connect answered with the full tree"
    );

    // The ids the drained updates address, resolved once the tree settled.
    let name = widget_id(&harness, Role::TextInput, "Name");
    let inventory = widget_id(&harness, Role::Table, "Inventory");
    let mut events: Vec<String> = Vec::new();

    // 1. Tab: the walk lands on the first field — the landing is the
    // funnel's answer as data, on the trail, and the pipeline's focus.
    harness.press_key(Key::Tab);
    let named = keyboard_landing(trail.drain(), &dashboard.landings);
    events.push(format!("focus → {named}"));
    assert_eq!(harness.pipeline().focus(), Some(dashboard.name_node));

    // 2. Typing into the focused field.
    events.push(type_and_assert(&mut harness, name));

    // 3. Tab again: two fields, the walk reads the tree's order — the
    // second stop is the second field.
    harness.press_key(Key::Tab);
    let named = keyboard_landing(trail.drain(), &dashboard.landings);
    events.push(format!("focus → {named}"));
    assert_eq!(harness.pipeline().focus(), Some(dashboard.email_node));

    // 4. Tab: the submit control.
    harness.press_key(Key::Tab);
    let named = keyboard_landing(trail.drain(), &dashboard.landings);
    events.push(format!("focus → {named}"));
    assert_eq!(harness.pipeline().focus(), Some(dashboard.submit_node));

    // 5. Enter: the keyboard activation.
    events.push(activate_on_enter(&mut harness, &trail, &dashboard.clicks));

    // 6. Tab: the inventory grid — the data table as one tab stop.
    harness.press_key(Key::Tab);
    let named = keyboard_landing(trail.drain(), &dashboard.landings);
    events.push(format!("focus → {named}"));
    assert_eq!(harness.pipeline().focus(), Some(dashboard.grid_node));

    // 7. ArrowDown: the pick moves, the focus does not; the shell
    // reconciles, the AT hears the pick.
    events.push(arrow_moves_pick(&mut harness, &dashboard, inventory));

    // The exact sequence the session produced — every entry a wire fact
    // (a trail record or a drained update), in session order.
    assert_eq!(
        events,
        workflow_events(),
        "the session's exact event sequence"
    );
    assert_tree_agrees(&harness);
}

#[test]
fn the_walks_landing_is_the_trees_word_and_a_quiet_frame_queues_nothing() {
    let _guard = alacer_core::flux::batch_test_lock();
    // The parity property itself: the walk's landing names a node the tree
    // words as the table, a focus move that carries no a11y word queues no
    // update (the idle contract at the window — "no a11y work, no
    // update"), and a re-connecting AT gets one full-tree build from the
    // same sink that served the session.
    let mut harness = TestHarness::new(480.0, 360.0);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let _ = harness.add_child(root, TextInput::new("Name").boxed());
    let selection = Rc::new(Cell::new(0_usize));
    let grid_node = harness.add_child(
        root,
        InventoryGrid {
            name: "Inventory",
            rows: 3,
            columns: 1,
            selected: 0,
            selection: Rc::clone(&selection),
        }
        .boxed(),
    );
    harness.settle();
    let _ = drain_updates(&mut harness);

    harness.press_key(Key::Tab);
    harness.press_key(Key::Tab);
    assert_eq!(
        harness.pipeline().focus(),
        Some(grid_node),
        "the walk landed on the grid node"
    );
    let resolved = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Table, "Inventory").cloned())
        .expect("the tree words the node the walk landed on");
    assert_eq!(
        resolved.role,
        Role::Table,
        "the tree's word for the focused node is the table the AT voices"
    );
    let drained = drain_updates(&mut harness);
    assert!(
        drained.is_empty(),
        "a focus move carrying no a11y word queues no update — the idle contract"
    );

    // A re-connecting AT gets one full-tree build from the same sink.
    let live = with_sink(&mut harness, |sink| {
        sink.full_tree_update().expect("the tree is resolved")
    });
    let snapshot_nodes: usize = harness
        .pipeline()
        .a11y_snapshot()
        .map(|tree| tree.nodes.len())
        .unwrap_or_default();
    assert_eq!(
        live.nodes.len(),
        snapshot_nodes,
        "the activation response covers the whole resolved forest"
    );
}
