//! The image widget's raster tier (the capability the image tier grew):
//! the pixels builders, the writes reporting the move (the M21 idle-frame
//! property at the raster tier), the paint pass recording the scene
//! journal's image primitive, and the honest refusal paths.
//!
//! The a11y contract is pinned unchanged beside the raster: the picture
//! word, the name, and the alt text are the tier's own truth — pixels
//! never move them.

use alacer_core::geom::{Point, Rect, Size};
use alacer_core::scene::{PaintCtx, PaintSink, Primitive};
use alacer_core::theme::Palette;
use alacer_core::widget::{Role, Widget};
use alacer_widgets::Image;
use std::sync::Arc;

/// A 4×2 opaque raster: red, red, blue, blue — the known bitmap the
/// paint contract pushes.
fn raster_bytes() -> Arc<[u8]> {
    let mut bytes = Vec::with_capacity(4 * 2 * 4);
    for _y in 0..2_u32 {
        for x in 0..4_u32 {
            if x < 2 {
                bytes.extend_from_slice(&[255, 0, 0, 255]);
            } else {
                bytes.extend_from_slice(&[0, 0, 255, 255]);
            }
        }
    }
    Arc::from(bytes.into_boxed_slice())
}

/// A 1×1 opaque white raster — the minimal legal bitmap.
fn tiny_bytes() -> Arc<[u8]> {
    Arc::from(vec![255_u8, 255, 255, 255].into_boxed_slice())
}

struct Collector(Vec<Primitive>);

impl PaintSink for Collector {
    fn push(&mut self, primitive: Primitive) {
        self.0.push(primitive);
    }
}

/// Paints `image` at origin (10, 5) over a 40×20 box, no clip.
fn painted(image: &Image) -> Vec<Primitive> {
    let mut sink = Collector(Vec::new());
    let mut ctx = PaintCtx::new(
        &mut sink,
        Point::new(10.0, 5.0),
        Size::new(40.0, 20.0),
        None,
        Palette::CANON,
        false,
        false,
        false,
        None,
        false,
    );
    image.paint(&mut ctx);
    sink.0
}

#[test]
fn the_raster_paints_the_image_primitive() {
    let bytes = raster_bytes();
    let picture = Image::new("Disk usage chart").pixels(Arc::clone(&bytes), 4, 2);
    let painted = painted(&picture);

    assert_eq!(
        painted.len(),
        2,
        "the box's borrowed background, then the raster"
    );
    let Primitive::Image(image) = &painted[1] else {
        panic!("expected the image primitive");
    };
    assert!(
        Arc::ptr_eq(&image.bytes, &bytes),
        "the raster rides by Arc, never copied"
    );
    assert_eq!(image.width, 4);
    assert_eq!(image.height, 2);
    // The destination is the widget's own box, translated to absolute
    // coordinates by the paint pass (the push_quad rule, at the raster tier).
    assert_eq!(
        image.dst,
        Rect::new(Point::new(10.0, 5.0), Size::new(40.0, 20.0))
    );
    assert_eq!(image.clip, None);
}

#[test]
fn set_pixels_reports_the_move_and_the_idle_write_queues_nothing() {
    let bytes = raster_bytes();
    let mut picture = Image::new("Disk usage chart").pixels(Arc::clone(&bytes), 4, 2);

    // The write that lands where the bytes already are: no move, no masks.
    assert!(
        !picture.set_pixels(Arc::clone(&bytes), 4, 2),
        "the same buffer at the same dimensions moved nothing"
    );

    // A new buffer at the same dimensions is a move (identity, not content).
    assert!(
        picture.set_pixels(raster_bytes(), 4, 2),
        "a rebuilt raster reports the move"
    );

    // A dimension change is a move even for the same buffer.
    assert!(
        picture.set_pixels(Arc::clone(&bytes), 2, 2),
        "the dimensions are the raster's identity too"
    );
}

#[test]
fn the_word_paints_without_pixels() {
    let picture = Image::new("Disk usage chart");
    let painted = painted(&picture);
    assert_eq!(painted.len(), 1, "the box's borrowed background alone");
    assert!(
        matches!(painted[0], Primitive::Quad(_)),
        "no raster, no image primitive — the M138 borrowed-roles paint stands"
    );
}

#[test]
fn refusal_paths_paint_no_image_primitive() {
    // Zero-size raster.
    let zero_size = Image::new("Disk usage chart").pixels(raster_bytes(), 0, 2);
    assert!(
        painted(&zero_size).len() == 1,
        "a zero-width raster paints no image primitive"
    );

    // Bytes that cannot hold the declared dimensions.
    let short = Image::new("Disk usage chart").pixels(tiny_bytes(), 4, 2);
    assert!(
        painted(&short).len() == 1,
        "bytes shorter than width·height·4 refuse honestly"
    );
}

#[test]
fn src_builder_windows_the_bitmap_and_reports_the_move() {
    let mut picture = Image::new("Disk usage chart").pixels(raster_bytes(), 4, 2);
    // The default source window is the whole bitmap.
    let recorded = painted(&picture);
    let Primitive::Image(image) = &recorded[1] else {
        panic!("expected the image primitive");
    };
    assert_eq!(
        image.src.size,
        Size::new(4.0, 2.0),
        "full bitmap by default"
    );

    // The window builder: a sub-rect rides the primitive.
    let window = Rect::new(Point::new(1.0, 0.0), Size::new(2.0, 2.0));
    assert!(
        picture.set_src(Some(window)),
        "the window write reports the move"
    );
    let windowed = painted(&picture);
    let Primitive::Image(image) = &windowed[1] else {
        panic!("expected the image primitive");
    };
    assert_eq!(image.src, window);

    // The write that lands where the window already is queues nothing.
    assert!(!picture.set_src(Some(window)), "the idle window write");
    // Clearing returns the honest absence.
    assert!(picture.set_src(None), "clearing reports the withdrawal");
}

#[test]
fn the_a11y_contract_stands_unchanged_beside_the_raster() {
    let picture = Image::new("Disk usage chart")
        .description("Bars per mount")
        .pixels(raster_bytes(), 4, 2);
    assert_eq!(picture.role(), Role::Image);
    assert_eq!(picture.label().resolve(), "Disk usage chart");
    assert_eq!(Widget::description(&picture), Some("Bars per mount"));
    assert!(
        !picture.focusable(),
        "pixels never make the picture a control"
    );
    assert_eq!(picture.role().name(), "image");
}

#[test]
fn registry_pixels_prop_create_patch_and_the_idle_frame() {
    let registry = alacer_widgets::default_registry();
    let bytes = raster_bytes();
    let node = alacer_view::ViewNode::element("image")
        .prop("name", "Disk usage chart")
        .prop(
            "pixels",
            alacer_view::PropValue::Pixels(alacer_view::Pixels::new(Arc::clone(&bytes), 4, 2)),
        );
    let mut widget = registry.create(&node);
    assert_eq!(widget.label().resolve(), "Disk usage chart");

    // The created widget paints the raster through the same paint door.
    let Some(picture) = widget.as_any_mut().downcast_mut::<Image>() else {
        panic!("the image element's widget");
    };
    let painted = painted(picture);
    assert_eq!(painted.len(), 2, "the raster rides the element door");
    assert!(
        matches!(&painted[1], Primitive::Image(image) if Arc::ptr_eq(&image.bytes, &bytes)),
        "the prop's buffer is the primitive's buffer"
    );

    // Re-hanging the same buffer: the idle frame (no visual, no masks).
    let idle = alacer_view::ViewNode::element("image").prop(
        "pixels",
        alacer_view::PropValue::Pixels(alacer_view::Pixels::new(Arc::clone(&bytes), 4, 2)),
    );
    assert!(
        !registry.patch(widget.as_mut(), &idle),
        "the write that lands where the bytes already are patches to no masks"
    );

    // A rebuilt buffer is a move.
    let moved = alacer_view::ViewNode::element("image").prop(
        "pixels",
        alacer_view::PropValue::Pixels(alacer_view::Pixels::new(raster_bytes(), 4, 2)),
    );
    assert!(
        registry.patch(widget.as_mut(), &moved),
        "a new buffer is a move"
    );
}
