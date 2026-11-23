//! The paint seam's sink half as a consumer's door (ADR-0002, A9): the
//! pipeline's one primitive path is whatever the journal adapter plays
//! — but [`PaintSink`] is a public trait, and a consumer bridging
//! Alacer's scene vocabulary into their own renderer (or a recorder, a
//! diff, a budget checker) implements it and receives the exact
//! primitives the widgets record, in paint order. This demo builds
//! that consumer side headlessly: a custom sink that prices what
//! flowed — quads, glyphs, images, the order the paint pass wrote —
//! with no window, no backend, and no journal: the widgets are painted
//! by hand through [`PaintCtx::new`], the same context the paint pass
//! builds once per frame.
//!
//! The beat the example exists to show: one scene representation
//! (axiom A8) — the CI renderer, the software backend, the GPU path and
//! *your* sink all consume the same `Primitive` stream the widgets
//! emit; a custom sink is a renderer seam, never a second scene.
//!
//! ```sh
//! cargo run -p alacer --example paint_sink_custom
//! ```

use alacer_core::geom::{Point, Size};
use alacer_core::scene::{PaintCtx, PaintSink, Primitive};
use alacer_core::theme::Palette;
use alacer_core::widget::Widget as _;
use alacer_text::ThreadLocalTextSeam;
use alacer_widgets::theme;
use alacer_widgets::{Label, Panel};
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo paints into (the trail demos' shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// What the custom sink prices: one row per primitive kind, the order
/// the paint pass wrote them in, and the fill colors the quads
/// carried — the consumer's own renderer would turn exactly this into
/// pixels.
#[derive(Default)]
struct Ledger {
    quads: usize,
    glyphs: usize,
    images: usize,
    quad_fills: Vec<String>,
    order: Vec<&'static str>,
}

impl PaintSink for Ledger {
    fn push(&mut self, primitive: Primitive) {
        // Order defines paint order (the trait's own rule): the ledger
        // records the stream as it crossed.
        match primitive {
            Primitive::Quad(quad) => {
                self.quads += 1;
                self.quad_fills.push(format!("{:?}", quad.color));
                self.order.push("quad");
            }
            Primitive::Glyph(_) => {
                self.glyphs += 1;
                self.order.push("glyph");
            }
            Primitive::Image(_) => {
                self.images += 1;
                self.order.push("image");
            }
        }
    }
}

fn main() {
    // The glyphs the Label paints ride the text seam the pipeline would
    // install (the assembly rule at the text tier): one thread-local
    // install, the same shape `pipeline_default` runs.
    alacer_core::text_seam::install(Rc::new(ThreadLocalTextSeam));

    let ledger = Rc::new(RefCell::new(Ledger::default()));

    // The widgets, painted by hand: the panel's card, the label's text
    // — the same `paint` methods the pass calls, the context the pass
    // builds (the canon palette, no OS facts overridden).
    let panel = Panel::new(VIEWPORT.height * 0.5, theme::PANEL);
    let label = Label::new("painted by your sink");

    let (quads, glyphs) = {
        let mut sink = ledger.borrow_mut();
        let mut ctx = PaintCtx::new(
            &mut *sink,
            Point::new(16.0, 16.0),
            Size::new(VIEWPORT.width - 32.0, VIEWPORT.height * 0.5),
            None,
            Palette::CANON,
            false,
            false,
            false,
            None,
        );
        panel.paint(&mut ctx);
        label.paint(&mut ctx);
        (sink.quads, sink.glyphs)
    };

    let ledger = ledger.borrow();
    println!(
        "paint sink: {quads} quad(s), {glyphs} glyph(s), {} image(s) — {}",
        ledger.images,
        ledger.order.join(" → "),
    );
    println!(
        "paint sink: quad fills {:?} — the fill colors the widgets resolved from the canon \
         palette (A8: one scene vocabulary, every consumer the same stream)",
        ledger.quad_fills,
    );
}
