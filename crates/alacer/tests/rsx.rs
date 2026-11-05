//! The `rsx!` macro against the real headless pipeline (§13.1): mounts,
//! clicks, keyed `for` loops, dynamic text, `if`/`else` swaps — every
//! assertion drives the a11y tree, the only official query API (ADR-10).

use alacer::prelude::*;
use alacer_testing::TestHarness;
use reactive_graph::traits::{Get, Set};

/// The M1 canon: the counter, written as a view function.
fn counter_view(count: alacer_core::FluxRwSignal<i32>) -> ViewNode {
    rsx! {
        column { gap: 8.0, padding: 12.0,
            label { text: "Clicked {count.get()} times" }
            button { text: "Increment", on_click: move || count.set(count.get() + 1) }
        }
    }
}

/// A keyed list — every sibling keyed (title + items), so the LIS pipeline
/// owns the whole child list (§6.2; mixed keyed/unkeyed siblings reconcile
/// positionally by contract).
fn list_view(title: &str, items: &[u64]) -> ViewNode {
    let items = items.to_vec();
    rsx! {
        column { gap: 4.0,
            label { key: 0_u64, text: title }
            for item in items.iter() {
                label { key: *item * 10 + 1, text: "Item {item}" }
            }
        }
    }
}

fn branch_view(show_button: bool) -> ViewNode {
    rsx! {
        column { gap: 2.0,
            if show_button {
                button { text: "Enabled", on_click: || {} }
            } else {
                label { text: "Disabled" }
            }
        }
    }
}

#[test]
fn rsx_mounts_and_renders_the_declared_tree() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let signal = harness.pipeline_mut().flux_mut().signal(0_i32);
    let view = counter_view(signal);
    let _root = alacer::mount_view(&view, harness.pipeline_mut());
    harness.settle();

    // Reachable and named through the a11y tree — the only query API.
    let _found = harness.expect_a11y(Role::Label, "Clicked 0 times");
    let _found = harness.expect_a11y(Role::Button, "Increment");
    harness.assert_all_named();
}

#[test]
fn rsx_clicks_flow_and_the_next_view_renders_the_new_state() {
    let mut harness = TestHarness::new(320.0, 240.0);
    let signal = harness.pipeline_mut().flux_mut().signal(0_i32);
    let view = counter_view(signal);
    let mut root = alacer::mount_view(&view, harness.pipeline_mut());
    harness.settle();

    // Click the button at the center of its a11y bounds.
    let button = harness.expect_a11y(Role::Button, "Increment");
    let bounds = button.bounds.expect("laid out button");
    let center = alacer_core::geom::Point::new(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    );
    harness.click(center);

    // The M1 view loop: re-run the view closure, reconcile, settle.
    let next = counter_view(signal);
    let report = root.update(&next, harness.pipeline_mut());
    harness.settle();

    assert_eq!(report.patched, 1, "the label's interpolated text changed");
    assert_eq!(report.created, 0);
    assert_eq!(report.moved, 0);
    let _found = harness.expect_a11y(Role::Label, "Clicked 1 times");
}

#[test]
fn rsx_for_loops_mount_keyed_lists_and_reconcile() {
    let mut harness = TestHarness::new(320.0, 480.0);
    let view = list_view("First batch", &[1, 2, 3, 4, 5]);
    let mut root = alacer::mount_view(&view, harness.pipeline_mut());
    harness.settle();
    let _found = harness.expect_a11y(Role::Label, "Item 3");

    // Reverse the batch: keyed LIS keeps every widget (same key set).
    let reversed = list_view("Reversed", &[5, 4, 3, 2, 1]);
    let report = root.update(&reversed, harness.pipeline_mut());
    harness.settle();

    assert_eq!(report.created, 0);
    assert_eq!(report.removed, 0);
    let _found = harness.expect_a11y(Role::Label, "Item 5");
    let _found = harness.expect_a11y(Role::Label, "Item 1");

    // A batch change: some keys drop, some mount.
    let next_batch = list_view("Second batch", &[3, 7, 1]);
    let report = root.update(&next_batch, harness.pipeline_mut());
    harness.settle();
    assert_eq!(report.created, 1, "item 7 mounts");
    assert_eq!(report.removed, 3, "items 5, 4 and 2 unmount");
    assert_eq!(
        report.moved, 0,
        "item 3 is LIS-stable and item 1 is suffix-peeled: no moves"
    );
    let _found = harness.expect_a11y(Role::Label, "Item 7");
    let _found = harness.expect_a11y(Role::Label, "Item 3");
    assert!(
        harness
            .pipeline()
            .a11y_snapshot()
            .and_then(|tree| tree.find(Role::Label, "Item 5"))
            .is_none(),
        "item 5 is gone from the a11y tree"
    );
}

#[test]
fn rsx_if_else_swaps_subtrees_in_place() {
    let mut harness = TestHarness::new(240.0, 160.0);
    let view = branch_view(true);
    let mut root = alacer::mount_view(&view, harness.pipeline_mut());
    harness.settle();
    let _found = harness.expect_a11y(Role::Button, "Enabled");

    let flipped = branch_view(false);
    let report = root.update(&flipped, harness.pipeline_mut());
    harness.settle();

    assert_eq!(report.created, 1, "the label mounts");
    assert_eq!(report.removed, 1, "the button unmounts");
    let _found = harness.expect_a11y(Role::Label, "Disabled");
}

#[test]
fn rsx_string_children_become_the_text_prop() {
    let mut harness = TestHarness::new(240.0, 120.0);
    // §5.2's footer shape: a string child IS the element's text.
    let footer = rsx! { label { "3 items left" } };
    let _root = alacer::mount_view(&footer, harness.pipeline_mut());
    harness.settle();
    let _found = harness.expect_a11y(Role::Label, "3 items left");
}
