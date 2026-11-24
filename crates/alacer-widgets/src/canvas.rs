//! The canvas: free-form custom paint through the paint seam (M82).
//!
//! The first widget whose body is entirely the app's: a named surface whose
//! paint closure records primitives through [`alacer_core::scene::PaintCtx`]
//! — the same sink, origin, clip stack, and resolved facts every widget
//! paints with. The reconciler mounts and patches it like any other
//! element; the a11y tree carries its name (`Role::Canvas`) and nothing
//! more to write — a canvas has no write door, so every AT action is
//! refused honestly (the M11 rule).
//!
//! The repaint contract is the painter's identity: a rebuilt view that
//! hangs a *new* closure reports the canvas visual (the rebuild may carry
//! a new painter), while a *hoisted* [`Painter`] re-hung across runs
//! re-hangs quietly — the idle-frame tool for canvases whose painter
//! never changes. State that moves while the view does not rides the flux
//! bridge's masks (a signal write queues the write's own `PAINT`), the
//! same currency every widget repaints on.

use alacer_core::events::PointerEvent;
use alacer_core::geom::Size;
use alacer_core::scene::PaintCtx;
use alacer_core::style::{MeasureSpec, Size2, Style};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, Widget};
use alacer_view::SharedPainter;
use std::rc::Rc;

/// A named, free-form paint surface: the app's closure draws, the pipeline
/// records and materializes like any other primitive.
///
/// # Examples
///
/// ```
/// use alacer_widgets::Canvas;
///
/// let canvas = Canvas::new("The waveform").painter(|ctx| {
///     let size = ctx.size();
///     ctx.push_quad(alacer_core::scene::Quad {
///         rect: alacer_core::geom::Rect::new(
///             alacer_core::geom::Point::new(0.0, 0.0),
///             size,
///         ),
///         color: ctx.palette().accent,
///         corner_radius: 0.0,
///         border_color: alacer_core::Color::TRANSPARENT,
///         border_width: 0.0,
///         clip: None,
///     });
/// });
/// ```
pub struct Canvas {
    name: String,
    painter: Option<SharedPainter>,
    width: f32,
    height: f32,
}

impl Canvas {
    /// The default surface size (`size` overrides it).
    const DEFAULT_SIZE: f32 = 120.0;

    /// A canvas named `name` with no painter yet (it paints nothing until
    /// one hangs — the honest blank).
    ///
    /// The name is mandatory: a canvas is a named drawing surface, not a
    /// decoration (ADR-5; `Role::Canvas` joins the walker's named set).
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            painter: None,
            width: Self::DEFAULT_SIZE,
            height: Self::DEFAULT_SIZE,
        }
    }

    /// The surface's size in logical pixels (the intrinsic size layout
    /// reads; styles may grow or shrink it at the container's discretion).
    #[must_use]
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// The paint closure the surface runs at paint time: called once per
    /// canvas paint pass with the node's [`PaintCtx`] — local coordinates,
    /// the clip stack, and the frame's resolved facts (palette, OS facts).
    #[must_use]
    pub fn painter(mut self, paint: impl for<'ctx> Fn(&mut PaintCtx<'ctx>) + 'static) -> Self {
        self.painter = Some(Rc::new(paint));
        self
    }

    /// Hangs an already-shared painter (the registry's door: identity
    /// comparisons ride the `Rc`, so a hoisted painter re-hangs quietly).
    #[must_use]
    pub fn painter_from(mut self, painter: Option<SharedPainter>) -> Self {
        self.painter = painter;
        self
    }

    /// Replaces the accessible name in place (view reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Resizes the surface in place (view reconciliation).
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Hangs a painter in place; answers whether the surface's paint moved.
    ///
    /// The rule is the handlers': identity, not behavior — re-hanging the
    /// same shared painter is `false` (no masks, an idle frame); a new
    /// painter is `true` (a rebuild may carry a new closure). Installing
    /// one where none hung is `true` — the surface paints for the first
    /// time.
    pub fn set_painter(&mut self, painter: Option<SharedPainter>) -> bool {
        let moved = match (&self.painter, &painter) {
            (Some(current), Some(next)) => !Rc::ptr_eq(current, next),
            // Installed where none hung, or un-hung: the surface's paint
            // moved either way.
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        };
        if moved {
            self.painter = painter;
        }
        moved
    }
}

impl Widget for Canvas {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Canvas
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(self.width),
                height: Some(self.height),
            },
            ..Style::default()
        }
    }

    /// Runs the app's paint closure, when one hangs; a painter-less canvas
    /// records nothing (the honest blank).
    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        if let Some(painter) = &self.painter {
            painter(ctx);
        }
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(Size::new(self.width, self.height)))
    }

    fn on_pointer_event(
        &mut self,
        _ctx: &mut alacer_core::events::EventCtx,
        _event: &PointerEvent,
    ) -> EventResponse {
        // A drawing surface is not a control: the pointer is the hover's
        // passenger (the panel's rule). Pointer gestures a canvas needs
        // ride the app's own listeners at the composition seam.
        EventResponse::Ignored
    }

    /// No write door: a canvas reports no value and takes no write — every
    /// AT action is refused honestly, never munged into a press or a step
    /// (the M11 rule). The name is the whole a11y contract.
    fn on_a11y_action(&mut self, _action: A11yAction) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::Canvas;
    use alacer_core::Widget as _;
    use alacer_core::color::Color;
    use alacer_core::geom::{Point, Rect};
    use alacer_core::scene::{Image, ImageSampling, PaintSink, Primitive, Quad};
    use alacer_core::style::MeasureSpec;
    use alacer_core::theme::Palette;
    use alacer_core::widget::Role;
    use alacer_view::Painter;

    /// A recording paint sink: the test's window into what the paint pass
    /// would have recorded (the trait is public, implementable anywhere).
    #[derive(Default)]
    struct Record {
        primitives: Vec<Primitive>,
    }

    impl PaintSink for Record {
        fn push(&mut self, primitive: Primitive) {
            self.primitives.push(primitive);
        }
    }

    /// The context the tests paint into: canon palette, no OS facts —
    /// the direct-caller shape [`PaintCtx::new`] documents.
    fn test_ctx(
        sink: &mut Record,
        origin: Point,
        clip: Option<Rect>,
    ) -> alacer_core::scene::PaintCtx<'_> {
        alacer_core::scene::PaintCtx::new(
            sink,
            origin,
            alacer_core::geom::Size::new(120.0, 120.0),
            clip,
            Palette::CANON,
            false,
            false,
            false,
            None,
            false,
        )
    }

    #[test]
    fn the_painter_records_through_the_paint_seam() {
        let canvas = Canvas::new("The waveform").painter(|ctx| {
            ctx.push_quad(Quad::filled(
                Rect::new(Point::new(0.0, 0.0), ctx.size()),
                Color::rgb(31, 122, 77),
            ));
        });

        let mut sink = Record::default();
        let mut ctx = test_ctx(&mut sink, Point::new(40.0, 60.0), None);
        canvas.paint(&mut ctx);

        let [Primitive::Quad(quad)] = &sink.primitives[..] else {
            panic!(
                "the painter recorded exactly one quad, got {:?}",
                sink.primitives
            );
        };
        assert_eq!(quad.color, Color::rgb(31, 122, 77));
        // The pass translates to absolute coordinates: the canvas sits at
        // (40, 60), so that is where the wash lands.
        assert_eq!(quad.rect.origin, Point::new(40.0, 60.0));
        assert_eq!(quad.rect.size, alacer_core::geom::Size::new(120.0, 120.0));
    }

    #[test]
    fn the_recorded_quad_carries_the_frames_clip() {
        let canvas = Canvas::new("The waveform").painter(|ctx| {
            ctx.push_quad(Quad::filled(
                Rect::new(Point::new(0.0, 0.0), ctx.size()),
                Color::rgb(31, 122, 77),
            ));
        });

        let clip = Rect::new(
            Point::new(0.0, 0.0),
            alacer_core::geom::Size::new(60.0, 120.0),
        );
        let mut sink = Record::default();
        let mut ctx = test_ctx(&mut sink, Point::new(0.0, 0.0), Some(clip));
        canvas.paint(&mut ctx);

        let [Primitive::Quad(quad)] = &sink.primitives[..] else {
            panic!("the painter recorded exactly one quad");
        };
        assert_eq!(quad.clip, Some(clip), "the clip stack rides the record");
    }

    #[test]
    fn a_painter_less_canvas_records_nothing() {
        let canvas = Canvas::new("The blank");
        let mut sink = Record::default();
        let mut ctx = test_ctx(&mut sink, Point::new(0.0, 0.0), None);
        canvas.paint(&mut ctx);
        assert!(
            sink.primitives.is_empty(),
            "the honest blank paints nothing"
        );
    }

    #[test]
    fn the_painter_records_the_raster_tier() {
        // An opaque 2 × 2 checker: premultiplied RGBA8 with a=255 — the
        // premultiplied form equals the straight one, the byte contract
        // the image primitive documents.
        let bytes: std::sync::Arc<[u8]> = vec![
            255, 31, 122, 77, // a red-green cell
            255, 31, 122, 77, 255, 12, 12, 16, // a near-black cell
            255, 12, 12, 16,
        ]
        .into();
        let expected = std::sync::Arc::clone(&bytes);
        let canvas = Canvas::new("The waveform").painter(move |ctx| {
            ctx.push_image(Image {
                bytes: std::sync::Arc::clone(&bytes),
                width: 2,
                height: 2,
                src: Rect::new(Point::new(0.0, 0.0), alacer_core::geom::Size::new(2.0, 2.0)),
                dst: Rect::new(Point::new(0.0, 0.0), ctx.size()),
                clip: None,
                sampling: ImageSampling::Nearest,
            });
        });

        let clip = Rect::new(
            Point::new(0.0, 0.0),
            alacer_core::geom::Size::new(60.0, 120.0),
        );
        let mut sink = Record::default();
        let mut ctx = test_ctx(&mut sink, Point::new(40.0, 60.0), Some(clip));
        canvas.paint(&mut ctx);

        let [Primitive::Image(image)] = &sink.primitives[..] else {
            panic!(
                "the painter recorded exactly one image, got {:?}",
                sink.primitives
            );
        };
        // The app's own closure reaches the raster tier through the same
        // paint seam the `Image` widget rides: the blit translates to the
        // canvas's absolute origin and the clip stack rides the record.
        assert_eq!(image.dst.origin, Point::new(40.0, 60.0));
        assert_eq!(image.dst.size, alacer_core::geom::Size::new(120.0, 120.0));
        assert_eq!(image.clip, Some(clip), "the clip stack rides the blit");
        assert_eq!(image.bytes, expected, "the app's pixels ride untouched");
        assert_eq!(image.sampling, ImageSampling::Nearest);
    }

    #[test]
    fn the_painter_sees_the_frames_resolved_facts() {
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let observer = std::rc::Rc::clone(&seen);
        let canvas = Canvas::new("The facts").painter(move |ctx| {
            observer.borrow_mut().push((
                ctx.dark(),
                ctx.reduced_motion(),
                ctx.high_contrast(),
                ctx.accent_color(),
            ));
        });

        let mut sink = Record::default();
        let mut ctx = test_ctx(&mut sink, Point::new(0.0, 0.0), None);
        canvas.paint(&mut ctx);

        assert_eq!(
            *seen.borrow(),
            vec![(false, false, false, None)],
            "the canon facts ride the context"
        );
    }

    #[test]
    fn the_canvas_is_a_named_surface() {
        let canvas = Canvas::new("The waveform");
        assert_eq!(canvas.role(), Role::Canvas);
        assert!(canvas.role().requires_name(), "a canvas must be named");
        assert_eq!(
            canvas.label().resolve(),
            "The waveform",
            "the name is the whole a11y contract"
        );
        assert_eq!(
            canvas.measure_spec(),
            Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(
                120.0, 120.0
            ))),
            "the default surface is 120 × 120"
        );
    }

    #[test]
    fn painter_identity_is_the_repaint_contract() {
        let mut canvas = Canvas::new("The waveform");

        let first = Painter::new(|_ctx| {}).shared();
        let hoisted = Painter::new(|_ctx| {}).shared();

        // Installing where none hung: the surface paints for the first time.
        assert!(canvas.set_painter(Some(first.clone())));
        // Re-hanging the same shared painter: nothing moved.
        assert!(!canvas.set_painter(Some(first.clone())));
        // A new painter: a rebuild may carry a new closure.
        assert!(canvas.set_painter(Some(hoisted.clone())));
        assert!(!canvas.set_painter(Some(hoisted.clone())));
        // Un-hanging: the blank returns.
        assert!(canvas.set_painter(None));
        assert!(!canvas.set_painter(None));
    }

    #[test]
    fn set_size_resizes_in_place() {
        let mut canvas = Canvas::new("The waveform");
        canvas.set_size(160.0, 90.0);
        assert_eq!(
            canvas.measure_spec(),
            Some(MeasureSpec::Fixed(alacer_core::geom::Size::new(
                160.0, 90.0
            )))
        );
    }
}
