//! The image role's `TreeUpdate` contract, headless: a picture in the tree
//! reaches AccessKit as the `Image` role with its name attached, and the
//! M95 alt-text tier's description rides beside it (the honest absence
//! sets nothing). Asserted against the update the sink actually builds —
//! no window, no platform adapter.

use accesskit::Role as AkRole;
use alacer_a11y::{AccessKitSink, find_in_update};
use alacer_core::Role;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::passes::{A11yDelta, A11ySink as _, TreeNode, TreeSnapshot};
use std::sync::Arc;

/// A quiet node: every field the tier under test does not speak rides the
/// honest absence (`None`/`false`), `enabled` the one truth.
fn node(
    id: u64,
    role: Role,
    label: &str,
    bounds: Option<Rect>,
    description: Option<String>,
    children: Vec<u64>,
) -> TreeNode {
    TreeNode {
        id,
        role,
        label: label.into(),
        bounds,
        children,
        description,
        value: None,
        toggled: None,
        table_slot: None,
        selection: None,
        live: None,
        enabled: true,
        placeholder: None,
        value_range: None,
        scroll_orientation: None,
        scroll_position: None,
        numeric_value: None,
        secure_entry: false,
        secure_revealed: false,
        read_only: false,
        required: false,
        invalid: None,
        selected: None,
        multiselectable: false,
        busy: false,
        expanded: None,
        auto_complete: None,
        sort_direction: None,
        level: None,
        current: None,
        has_popup: None,
        role_description: None,
        position_in_set: None,
        size_of_set: None,
        keyboard_shortcut: None,
        multiline: false,
        language: None,
        search_field: false,
        actions: alacer_core::widget::A11yActions::empty(),
    }
}

/// A container root carrying one picture whose word and help text are
/// declared when `declared` (the alt-text tier's honest presence).
fn snapshot(declared: bool) -> TreeSnapshot {
    TreeSnapshot {
        nodes: vec![
            node(
                0,
                Role::Container,
                "app",
                Some(Rect::new(Point::zero(), Size::new(320.0, 480.0))),
                None,
                vec![1],
            ),
            node(
                1,
                Role::Image,
                "Disk usage chart",
                Some(Rect::new(Point::new(0.0, 16.0), Size::new(320.0, 160.0))),
                declared.then(|| "Bars per mount".to_string()),
                Vec::new(),
            ),
        ],
        root: Some(0),
    }
}

#[test]
fn the_picture_reaches_the_tree_update_as_an_image() {
    let mut sink = AccessKitSink::new();
    sink.apply(Arc::new(snapshot(true)), &A11yDelta::empty());

    let update = sink.last_update().expect("the first frame initializes");
    let picture = find_in_update(update, 1).expect("the picture in the update");
    assert_eq!(picture.role(), AkRole::Image, "the AT reads the picture");
    assert_eq!(picture.label(), Some("Disk usage chart"));
    assert_eq!(
        picture.description(),
        Some("Bars per mount"),
        "the alt-text tier rides the update"
    );
}

#[test]
fn a_picture_with_nothing_to_explain_sets_no_description() {
    let mut sink = AccessKitSink::new();
    sink.apply(Arc::new(snapshot(false)), &A11yDelta::empty());

    let update = sink.last_update().expect("the first frame initializes");
    let picture = find_in_update(update, 1).expect("the picture in the update");
    assert_eq!(picture.role(), AkRole::Image);
    assert!(
        picture.description().is_none(),
        "the honest absence maps to the absence"
    );
}
