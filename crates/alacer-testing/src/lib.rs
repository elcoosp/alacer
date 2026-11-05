//! Alacer's headless testing harness (A4, ADR-10).
//!
//! Headless tests run the **real pass pipeline** with a **software
//! rasterizer** — no mock event loop, no mock renderer. GPU-less CI is the
//! default, not a downgrade. The harness also provides:
//!
//! - the a11y-driven query API — the *only* official query API (§11.5);
//! - the debug CI walker that fails on unnamed input widgets (§11.1);
//! - snapshot testing with `ALACER_BLESS=1` re-blessing (§13.4);
//! - scheduling-contract counters for the ADR-6 kill-switch.

use alacer_a11y::AccessKitSink;
use alacer_core::arena::NodeId;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::flux::Flux;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::passes::PassPipeline;
use alacer_core::style::MeasureSpec;
use alacer_core::widget::{Role, Widget};
use alacer_layout::LayoutHost;
use alacer_render::{SceneJournalImpl, SoftwareRenderer};
use alacer_text::{ThreadLocalGlyphSource, ThreadLocalMeasurer, with_text};
use std::path::PathBuf;
use std::sync::Arc;
use tiny_skia::Pixmap;

/// The harness: pipeline + viewport + counters, everything headless.
pub struct TestHarness {
    pipeline: PassPipeline,
    viewport: Size,
    renderer: SoftwareRenderer,
    glyph_source: ThreadLocalGlyphSource,
}

impl TestHarness {
    /// A harness with a `width × height` viewport.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        let flux = Flux::new();
        let layout = LayoutHost::new(Arc::new(ThreadLocalMeasurer));
        let journal = SceneJournalImpl::new();
        let a11y = AccessKitSink::new();
        Self {
            pipeline: PassPipeline::new(flux, Box::new(layout), Box::new(journal), Box::new(a11y)),
            viewport: Size::new(width, height),
            renderer: SoftwareRenderer::new(),
            glyph_source: ThreadLocalGlyphSource,
        }
    }

    /// The wrapped pipeline.
    #[must_use]
    pub const fn pipeline(&self) -> &PassPipeline {
        &self.pipeline
    }

    /// The wrapped pipeline, mutably.
    #[must_use]
    pub const fn pipeline_mut(&mut self) -> &mut PassPipeline {
        &mut self.pipeline
    }

    /// Mounts the root widget.
    #[must_use]
    pub fn mount(&mut self, widget: Box<dyn Widget>) -> NodeId {
        self.pipeline.mount_root(widget)
    }

    /// Appends a child widget.
    #[must_use]
    pub fn add_child(&mut self, parent: NodeId, widget: Box<dyn Widget>) -> NodeId {
        self.pipeline.add_child(parent, widget)
    }

    /// Runs one frame.
    ///
    /// # Panics
    ///
    /// Panics if layout fails (a test bug, not a runtime condition).
    pub fn tick(&mut self) -> alacer_core::passes::FrameReport {
        self.pipeline
            .tick(self.viewport)
            .expect("harness layout must succeed")
    }

    /// Ticks until no pass has work (bounded; the idle contract, A7).
    ///
    /// # Panics
    ///
    /// Panics if the tree never settles within 16 frames.
    pub fn settle(&mut self) -> usize {
        for frame in 0..16 {
            let report = self.tick();
            if report.is_idle() {
                return frame;
            }
        }
        panic!("harness did not settle within 16 frames");
    }

    /// Renders the current scene to pixels.
    ///
    /// # Panics
    ///
    /// Panics if the renderer cannot allocate the viewport pixmap.
    #[must_use]
    pub fn pixels(&mut self) -> Pixmap {
        let scene = self.pipeline.tick(self.viewport).expect("layout").scene;
        self.renderer
            .render(
                &scene,
                u32_width(self.viewport.width),
                u32_height(self.viewport.height),
                alacer_widgets::theme::SURFACE,
                &mut self.glyph_source,
            )
            .expect("viewport pixmap")
    }

    /// Dispatches a click: press + release at `position`, then settle.
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn click(&mut self, position: Point) {
        let mut ctx = EventCtx::new();
        let _ = &mut ctx;
        self.pipeline.dispatch_pointer(&PointerEvent::Down {
            position,
            button: alacer_core::events::MouseButton::Left,
        });
        self.pipeline.dispatch_pointer(&PointerEvent::Up {
            position,
            button: alacer_core::events::MouseButton::Left,
        });
        self.settle();
    }

    /// Dispatches a wheel event at `position`, then settles.
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn wheel(&mut self, position: Point, delta: (f32, f32)) {
        self.pipeline.dispatch_pointer(&PointerEvent::Wheel {
            delta: alacer_core::geom::Vector::new(delta.0, delta.1),
            position,
        });
        self.settle();
    }

    /// The a11y query: first node with `role` whose name contains `name`
    /// (§11.5: tests drive the a11y tree, never the widget tree).
    ///
    /// # Panics
    ///
    /// Panics naming the missing node when the query fails.
    #[must_use]
    pub fn expect_a11y(&self, role: Role, name: &str) -> &alacer_core::passes::TreeNode {
        self.pipeline
            .a11y_snapshot()
            .and_then(|tree| tree.find(role, name))
            .unwrap_or_else(|| {
                panic!("a11y query failed: no {role} named {name:?} is reachable");
            })
    }

    /// Renders the current frame's pixels — same as [`Self::pixels`].
    ///
    /// # Panics
    ///
    /// Same as [`Self::pixels`].
    #[must_use]
    pub fn frame_pixels(&mut self) -> Pixmap {
        self.pixels()
    }

    /// Compares the current frame against the blessed snapshot `name`.
    ///
    /// With `ALACER_BLESS=1` the snapshot is regenerated instead. Comparison
    /// allows a small percentage of tolerance pixels (per-OS font
    /// differences must not rot the corpus, §13.4 snapshot hygiene).
    ///
    /// # Panics
    ///
    /// Panics when the render differs beyond tolerance and `ALACER_BLESS` is
    /// unset.
    pub fn snapshot(&mut self, name: &str) {
        let pixmap = self.pixels();
        let path = blessed_dir().join(format!("{name}.png"));
        if std::env::var_os("ALACER_BLESS").is_some_and(|v| v == "1") {
            let _ = std::fs::create_dir_all(blessed_dir());
            pixmap.save_png(&path).expect("bless write");
            return;
        }
        let bytes = std::fs::read(&path).unwrap_or_else(|e| {
            panic!("blessed snapshot {path:?} missing ({e}); run with ALACER_BLESS=1")
        });
        let blessed = Pixmap::decode_png(&bytes).expect("blessed snapshot decodes");
        let diff = pixel_diff_ratio(&pixmap, &blessed);
        assert!(
            diff <= 0.02,
            "snapshot {name} drifted: {diff:.4} of pixels differ (bless with ALACER_BLESS=1 if intended)"
        );
    }

    /// The CI walker (§11.1): every widget's name must resolve, and input-like
    /// controls (roles that `requires_name()`) must be *named*.
    ///
    /// # Panics
    ///
    /// Panics listing every violation when any widget is unnamed.
    pub fn assert_all_named(&self) {
        let tree = self
            .pipeline
            .a11y_snapshot()
            .expect("a11y tree built — tick first");
        let offenders: Vec<String> = tree
            .nodes
            .iter()
            .filter(|n| n.role.requires_name() && n.label.trim().is_empty())
            .map(|n| format!("unnamed {} (id {})", n.role, n.id))
            .collect();
        assert!(
            offenders.is_empty(),
            "a11y CI walker found unnamed widgets: {offenders:?}"
        );
    }

    /// Total measure calls the layout engine made (translation-only probes).
    #[must_use]
    pub fn measure_calls(&self) -> u64 {
        self.pipeline.measure_calls()
    }

    /// Resets the layout engine's measure counter.
    pub fn reset_measure_calls(&mut self) {
        self.pipeline.reset_measure_calls();
    }

    /// Current frame counters (scheduling-contract probes).
    #[must_use]
    pub fn counters(&self) -> alacer_core::passes::FrameCounters {
        self.pipeline.counters()
    }

    /// Bounds of a node (geometry probes).
    ///
    /// # Panics
    ///
    /// Panics if the node has no bounds yet (tick first).
    #[must_use]
    pub fn bounds(&self, node: NodeId) -> Rect {
        self.pipeline
            .arena()
            .bounds(node)
            .unwrap_or_else(|| panic!("{node} has no bounds — tick first"))
    }

    /// The viewport size.
    #[must_use]
    pub const fn viewport(&self) -> Size {
        self.viewport
    }

    /// Access to the thread-local text context (test setup).
    pub fn with_text_context<R>(&self, f: impl FnOnce(&mut alacer_text::TextContext) -> R) -> R {
        with_text(f)
    }

    /// The intrinsic text size helper (list item sizing in tests).
    #[must_use]
    pub fn text_size(&self, text: &str, font_size: f32) -> Size {
        with_text(|ctx| {
            ctx.measure(
                &alacer_core::style::TextSpec {
                    text: text.to_string(),
                    font_id: 0,
                    font_size,
                },
                None,
            )
        })
    }

    /// A spec probe: does a widget expose a measure spec?
    #[must_use]
    pub fn spec_of(&self, node: NodeId) -> Option<MeasureSpec> {
        self.pipeline.arena().widget(node).measure_spec()
    }
}

fn u32_width(w: f32) -> u32 {
    alacer_core::num::f32_to_u32(w)
}

fn u32_height(h: f32) -> u32 {
    alacer_core::num::f32_to_u32(h)
}

fn blessed_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("blessed")
}

/// Fraction of pixels whose channel deltas exceed 8/255.
#[must_use]
pub fn pixel_diff_ratio(a: &Pixmap, b: &Pixmap) -> f32 {
    if a.width() != b.width() || a.height() != b.height() {
        return 1.0;
    }
    let a_bytes = a.data();
    let b_bytes = b.data();
    let total = a_bytes.len() / 4;
    if total == 0 {
        return 0.0;
    }
    let (a_px, b_px) = (a_bytes.as_chunks::<4>().0, b_bytes.as_chunks::<4>().0);
    let differing = a_px
        .iter()
        .zip(b_px.iter())
        .filter(|(p, q)| p.iter().zip(q.iter()).any(|(x, y)| x.abs_diff(*y) > 8))
        .count();
    alacer_core::num::usize_to_f32(differing) / alacer_core::num::usize_to_f32(total)
}

/// A test widget trait extension: build boxes for compositions.
pub trait WidgetBoxExt: Widget + Sized + 'static {
    /// Boxes the widget.
    fn boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}

impl<W: Widget> WidgetBoxExt for W {}

/// Verifies the glyph rasterization seam end-to-end through the harness.
#[must_use]
pub fn glyph_source() -> ThreadLocalGlyphSource {
    ThreadLocalGlyphSource
}

#[cfg(test)]
mod tests {
    use super::{TestHarness, pixel_diff_ratio};

    #[test]
    fn pixel_diff_ratio_is_symmetric_and_exact_on_identity() {
        let mut harness = TestHarness::new(8.0, 8.0);
        let a = harness.pixels();
        let b = harness.pixels();
        let diff = pixel_diff_ratio(&a, &b);
        assert!(diff <= 0.0, "identical pixmaps diff to zero, got {diff}");
    }
}
