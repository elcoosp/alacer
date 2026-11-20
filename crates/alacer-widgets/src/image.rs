//! The image tier (M138, ADR-0142): the named picture the app
//! presents.
//!
//! "This holds a picture" was a vocabulary hole: an app wiring a
//! chart, a logo, a diagram reached for a bare [`crate::Label`]
//! wearing no picture word (the picture the AT cannot name is a
//! picture it cannot offer on demand, the ADR-5 rule the named roles
//! carry) — or for a [`crate::Canvas`] wearing the wrong half (the
//! canvas tier, M82, names a *drawing surface the app paints into*; a
//! picture is *named content the app presents* — AccessKit's `Image`
//! role is its own fact). [`Role::Image`] carries the picture word the
//! AT announces.
//!
//! The pixel story is the app's own, never the word's: the paint seam
//! the canvas tier published carries whatever the app paints, and the
//! raster tier carries whatever the app embeds — [`Image::pixels`]
//! embeds the premultiplied bytes, `paint` pushes them as the scene
//! journal's image primitive (§9.1's vocabulary growth), and the word's
//! slot stays the borrowed background behind them. The widget prices
//! no pixels of its own either way (the M126/M129 container rule's
//! honest shape: the word walls nothing).
//! The alt text rides the M95 description word beside the name — the
//! help text the AT reads when the picture needs explaining — and the
//! picture holds no focus and answers no activation: the controls the
//! app mounts around it hold the focus.

use alacer_core::color::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::num::u32_to_f32;
use alacer_core::scene::{Image as ImagePrimitive, ImageSampling, PaintCtx};
use alacer_core::style::Style;
use alacer_core::widget::{EventResponse, LabelSource, Role, Widget};
use std::sync::Arc;

/// A named picture the app presents (the image tier, M138, ADR-0142):
/// the picture word the AT announces — an unnamed picture reads as a
/// bare blank, the picture the AT cannot name is a picture it cannot
/// offer on demand (ADR-5).
pub struct Image {
    name: String,
    /// The alt text the a11y tree carries beside the name (M95,
    /// ADR-0099); `None` is the honest absence.
    description: Option<String>,
    style: Style,
    /// The embedded raster (the raster tier); `None` is the honest
    /// absence — the word's slot paints the borrowed background alone.
    raster: Option<Raster>,
    /// The source window, in bitmap pixel coordinates; `None` is the
    /// whole bitmap.
    src: Option<Rect>,
    /// How the raster samples when the box scales it (the default is the
    /// crisp [`ImageSampling::Nearest`]).
    sampling: ImageSampling,
}

/// The embedded raster: the byte contract the scene journal's image
/// primitive spells — premultiplied RGBA8, row-major, exactly
/// `width · height · 4` bytes.
struct Raster {
    bytes: Arc<[u8]>,
    width: u32,
    height: u32,
}

impl Raster {
    /// Whether the bytes can hold the declared dimensions — the
    /// paint-time honesty check. A short buffer (or one whose declared
    /// area overflows) refuses the primitive: renderers draw nothing,
    /// never a guessed pixel.
    fn holds(&self) -> bool {
        let pixels = usize::try_from(self.width).ok().and_then(|w| {
            usize::try_from(self.height)
                .ok()
                .and_then(|h| w.checked_mul(h))
        });
        pixels
            .and_then(|pixels| pixels.checked_mul(4))
            .is_some_and(|expected| self.bytes.len() >= expected)
    }
}

impl Image {
    /// An image labeled `name` — the picture the AT announces as a
    /// picture, the pixels the app's own.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            style: Style::default(),
            raster: None,
            src: None,
            sampling: ImageSampling::Nearest,
        }
    }

    /// The alt text (M95, ADR-0099): the help the picture's name is too
    /// short to carry (a summary, a source, a refresh cadence). Without
    /// it the image announces the honest absence.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Fixed height.
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.style.size.height = Some(height);
        self
    }

    /// Fixed width.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.style.size.width = Some(width);
        self
    }

    /// Grow factor.
    #[must_use]
    pub fn grow(mut self, grow: f32) -> Self {
        self.style.grow = grow;
        self
    }

    /// Uniform padding.
    #[must_use]
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding = padding;
        self
    }

    /// Shared-borrow layout style access for view reconciliation (§6.1):
    /// patches write styles in place instead of rebuilding the widget.
    #[must_use]
    pub const fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    /// Replaces the picture's accessible name in place (view
    /// reconciliation).
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// The one description write (M95, ADR-0099, view reconciliation):
    /// `None` clears to the honest absence. Report whether the text
    /// moved — the caller requests `A11Y` only when this answers `true`,
    /// so a write that lands where the text already is queues nothing
    /// (the M21 idle-frame property, at the description tier).
    pub fn set_description(&mut self, description: Option<String>) -> bool {
        if self.description == description {
            return false;
        }
        self.description = description;
        true
    }

    /// Embeds the raster the app presents (the raster tier): premultiplied
    /// RGBA8 bytes, row-major, `width · height · 4` bytes long — the byte
    /// contract the scene journal's image primitive spells. The bytes ride
    /// by `Arc`, the app and the journal sharing one allocation.
    #[must_use]
    pub fn pixels(mut self, bytes: Arc<[u8]>, width: u32, height: u32) -> Self {
        self.raster = Some(Raster {
            bytes,
            width,
            height,
        });
        self
    }

    /// The source window, in bitmap pixel coordinates (a sprite sheet's
    /// cell, a thumbnail's crop); the default is the whole bitmap.
    #[must_use]
    pub fn src(mut self, src: Rect) -> Self {
        self.src = Some(src);
        self
    }

    /// How the raster samples when the box scales it. The default is the
    /// crisp [`ImageSampling::Nearest`]; scaled thumbnails read as
    /// [`ImageSampling::Bilinear`].
    #[must_use]
    pub fn sampling(mut self, sampling: ImageSampling) -> Self {
        self.sampling = sampling;
        self
    }

    /// The one raster write (view reconciliation, the raster tier):
    /// reports whether the bytes or their dimensions moved — the caller
    /// requests `PAINT` only when this answers `true`, so a write that
    /// lands where the raster already is queues nothing (the M21
    /// idle-frame property, at the raster tier). Identity is the contract:
    /// a rebuilt buffer is a move even when the pixels are the same (the
    /// painter's own rule).
    pub fn set_pixels(&mut self, bytes: Arc<[u8]>, width: u32, height: u32) -> bool {
        if let Some(raster) = self.raster.as_ref()
            && Arc::ptr_eq(&raster.bytes, &bytes)
            && raster.width == width
            && raster.height == height
        {
            return false;
        }
        self.raster = Some(Raster {
            bytes,
            width,
            height,
        });
        true
    }

    /// The one source-window write: `None` clears to the whole bitmap (the
    /// honest absence). Reports whether the window moved — the idle write
    /// queues nothing (the M21 idle-frame property, at the raster tier).
    pub fn set_src(&mut self, src: Option<Rect>) -> bool {
        if self.src == src {
            return false;
        }
        self.src = src;
        true
    }

    /// The one sampling write: reports whether the mode moved — the idle
    /// write queues nothing (the M21 idle-frame property, at the raster
    /// tier).
    pub fn set_sampling(&mut self, sampling: ImageSampling) -> bool {
        if self.sampling == sampling {
            return false;
        }
        self.sampling = sampling;
        true
    }

    /// Builds the boxed widget.
    #[must_use]
    pub fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl Widget for Image {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Image
    }

    fn label(&self) -> LabelSource {
        LabelSource::text(self.name.clone())
    }

    /// The alt text tier (M95, ADR-0099): the help beside the name, the
    /// honest absence when the picture has nothing to explain.
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        // The word's slot borrows the surface role so it reads as itself
        // in the frame — the same borrowed-roles discipline every canon
        // widget paints by (ADR-0079). The raster rides on top of it: the
        // pixel story is the app's own, the widget pushing exactly what
        // the app embedded, never a pixel of its own.
        let palette = ctx.palette();
        let size = (ctx.size().width, ctx.size().height);
        crate::paint_background(ctx, size, palette.surface, 0.0, (Color::TRANSPARENT, 0.0));

        // The raster tier: the embedded pixels join the scene journal as
        // the image primitive. Missing bytes, zero dimensions, or a
        // buffer that cannot hold its declared dimensions refuse
        // honestly — no primitive, the borrowed background stands.
        let Some(raster) = self.raster.as_ref() else {
            return;
        };
        if raster.width == 0 || raster.height == 0 || !raster.holds() {
            return;
        }
        let src = self.src.unwrap_or(Rect::new(
            Point::zero(),
            Size::new(u32_to_f32(raster.width), u32_to_f32(raster.height)),
        ));
        ctx.push_image(ImagePrimitive {
            bytes: Arc::clone(&raster.bytes),
            width: raster.width,
            height: raster.height,
            src,
            dst: Rect::new(Point::zero(), ctx.size()),
            clip: None,
            sampling: self.sampling,
        });
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, _event: &PointerEvent) -> EventResponse {
        EventResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::Image;
    use alacer_core::widget::{Role, Widget};

    #[test]
    fn role_name_and_picture_contract_satisfy_the_tier() {
        let picture = Image::new("Disk usage chart");
        assert_eq!(picture.role(), Role::Image);
        assert_eq!(picture.label().resolve(), "Disk usage chart");
        assert!(picture.role().requires_name());
        assert!(
            !picture.focusable(),
            "the picture is the content's word — the controls around it hold the focus, never the picture"
        );
        assert_eq!(picture.role().name(), "image");
    }

    #[test]
    fn the_name_write_moves_the_word() {
        let mut picture = Image::new("Disk usage chart");
        picture.set_name("Memory usage chart");
        assert_eq!(picture.label().resolve(), "Memory usage chart");
    }

    #[test]
    fn the_description_write_reports_the_move() {
        let mut picture = Image::new("Disk usage chart");
        // The reader rides the trait (`Widget::description`) — the
        // builder word owns the inherent name, the trait call reads
        // what the builder wrote.
        assert!(
            Widget::description(&picture).is_none(),
            "the honest absence at mount"
        );
        assert!(
            picture.set_description(Some("Bars per mount".to_string())),
            "the move answers true"
        );
        assert_eq!(Widget::description(&picture), Some("Bars per mount"));
        assert!(
            !picture.set_description(Some("Bars per mount".to_string())),
            "the write that lands where the text already is queues nothing"
        );
        assert!(
            picture.set_description(None),
            "clearing answers true — the absence returns honestly"
        );
        assert!(Widget::description(&picture).is_none());
    }
}
