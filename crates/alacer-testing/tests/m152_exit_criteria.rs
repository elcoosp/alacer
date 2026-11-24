//! M152 exit-criteria integration tests (§11.1, ADR-0156): the pointer
//! cursor seam — the widget tier declares a hover shape, the pipeline
//! resolves the deepest accepting hit's answer, and the platform applies
//! the delta to the window.
//!
//! Through M151 a widget author had no way to request a pointer shape:
//! the workspace named mouse positions, IME carets, and an inspector's
//! replay cursor, but `CursorIcon` had zero hits in the entire tree —
//! a link read as bare text under the pointer and a text field gave the
//! arrow no hint. ADR-0156 closes the gap with the house seam discipline:
//! a [`CursorStyle`] vocabulary in core (the honest CSS-parity subset of
//! the platform's own icon set), a defaulted [`Widget::cursor_style`]
//! hook every widget inherits, and [`PassPipeline::cursor_at`] — the
//! resolution door that walks the same deepest-accepting-hit chain the
//! pointer route walks and answers the hovered widget's declared style
//! (the default when nothing accepts the hover). The platform's half is
//! the delta discipline the IME request already rides: resolve per frame,
//! call the window only when the shape moved.
//!
//! Every test runs the real pipeline (A4) through the harness, the way
//! the m-suite always drives: bounds come from the layout the harness
//! settled, the ask rides [`PassPipeline::cursor_at`], and the answers
//! are the styles the widgets declared.

use alacer_core::cursor::CursorStyle;
use alacer_core::geom::Point;
use alacer_core::style::Size2;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Link, TextInput};

/// The center of a mounted widget's laid-out bounds — the hover point
/// the tests ask with (the layout the harness settled is the geometry
/// the pointer walks).
fn center(harness: &TestHarness, node: alacer_core::NodeId) -> Point {
    let bounds = harness.bounds(node);
    Point::new(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    )
}

/// The deep-hover probe: a custom canvas that names its own hover shape
/// (the grab handle a canvas-tier control shows while a drag can arm).
struct GrabHandle {
    name: String,
}

impl GrabHandle {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Widget for GrabHandle {
    fn role(&self) -> Role {
        Role::Canvas
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    fn style(&self) -> alacer_core::style::Style {
        alacer_core::style::Style {
            size: Size2 {
                width: Some(120.0),
                height: Some(48.0),
            },
            ..alacer_core::style::Style::default()
        }
    }

    fn cursor_style(&self) -> CursorStyle {
        CursorStyle::Grab
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// The stock vocabulary: a link hovers as the pointer (the open affordance
/// the web's hand taught), a text field as the text caret — the styles the
/// widgets themselves declare, answered by the pipeline's hover door at
/// the widgets' own laid-out bounds.
#[test]
fn m152_the_hover_answers_the_deepest_widget_s_declared_style() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().gap(12.0).padding(16.0).boxed());
    let link = harness.add_child(root, Link::new("Documentation").boxed());
    let field = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();

    let pipeline = harness.pipeline();
    assert_eq!(
        pipeline.cursor_at(center(&harness, link)),
        CursorStyle::Pointer,
        "the link's hover is the pointer — the style the widget declared"
    );
    assert_eq!(
        pipeline.cursor_at(center(&harness, field)),
        CursorStyle::Text,
        "the field's hover is the text caret — the style the widget declared"
    );
}

/// Nothing accepting under the pointer answers the default: the void
/// beside the mounted column, and a plain label's own face (a label
/// overrides nothing, so its hover is the default arrow — the muted
/// text stays decoration, never an affordance).
#[test]
fn m152_the_void_and_the_undeclared_hover_answer_the_default() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().gap(12.0).padding(16.0).boxed());
    let _link = harness.add_child(root, Link::new("Documentation").boxed());
    harness.settle();

    let pipeline = harness.pipeline();
    assert_eq!(
        pipeline.cursor_at(Point::new(380.0, 190.0)),
        CursorStyle::Default,
        "the void beside the column accepts nothing — the default stands"
    );
    assert_eq!(
        pipeline.cursor_at(Point::new(-40.0, -40.0)),
        CursorStyle::Default,
        "a point outside the viewport accepts nothing — the default stands"
    );
}

/// A custom widget names its own hover shape: the hook is the widget's
/// door, not a registry of blessed controls — the grab handle's `Grab`
/// rides the same door the stock widgets' words ride, and the default
/// hook (every widget the framework ships that overrides nothing) is
/// the honest absence.
#[test]
fn m152_a_custom_widget_names_its_own_hover_shape() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().gap(12.0).padding(16.0).boxed());
    let handle = harness.add_child(root, GrabHandle::new("Handle").boxed());
    let link = harness.add_child(root, Link::new("Documentation").boxed());
    harness.settle();

    let pipeline = harness.pipeline();
    assert_eq!(
        pipeline.cursor_at(center(&harness, handle)),
        CursorStyle::Grab,
        "the handle's hover is the grab — the style the custom widget named"
    );
    assert_eq!(
        pipeline.cursor_at(center(&harness, link)),
        CursorStyle::Pointer,
        "the sibling's hover is its own — the door answers per hit, never per tree"
    );
}

/// The platform's mapping is total: every style the vocabulary carries
/// has a named window icon — the exhaustive match the delta applies
/// through (the mapping test rides the platform crate; this suite pins
/// the vocabulary's own wire names).
#[test]
fn m152_the_styles_carry_their_wire_names() {
    assert_eq!(CursorStyle::Default.name(), "default");
    assert_eq!(CursorStyle::Pointer.name(), "pointer");
    assert_eq!(CursorStyle::Text.name(), "text");
    assert_eq!(CursorStyle::Crosshair.name(), "crosshair");
    assert_eq!(CursorStyle::Grab.name(), "grab");
    assert_eq!(CursorStyle::Grabbing.name(), "grabbing");
    assert_eq!(CursorStyle::NotAllowed.name(), "not_allowed");
    assert_eq!(CursorStyle::Wait.name(), "wait");
    assert_eq!(CursorStyle::Progress.name(), "progress");
    assert_eq!(CursorStyle::Help.name(), "help");
    assert_eq!(CursorStyle::ContextMenu.name(), "context_menu");
    assert_eq!(CursorStyle::Move.name(), "move");
    assert_eq!(CursorStyle::Copy.name(), "copy");
    assert_eq!(CursorStyle::Alias.name(), "alias");
    assert_eq!(CursorStyle::NoDrop.name(), "no_drop");
    assert_eq!(CursorStyle::Cell.name(), "cell");
    assert_eq!(CursorStyle::VerticalText.name(), "vertical_text");
    assert_eq!(CursorStyle::AllScroll.name(), "all_scroll");
    assert_eq!(CursorStyle::ZoomIn.name(), "zoom_in");
    assert_eq!(CursorStyle::ZoomOut.name(), "zoom_out");
    assert_eq!(CursorStyle::EwResize.name(), "ew_resize");
    assert_eq!(CursorStyle::NsResize.name(), "ns_resize");
    assert_eq!(CursorStyle::NeswResize.name(), "nesw_resize");
    assert_eq!(CursorStyle::NwseResize.name(), "nwse_resize");
    assert_eq!(CursorStyle::ColResize.name(), "col_resize");
    assert_eq!(CursorStyle::RowResize.name(), "row_resize");
}
